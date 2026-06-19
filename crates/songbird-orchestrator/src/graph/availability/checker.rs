// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `AvailabilityChecker` implementation — registry queries and compatibility scoring.

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::types::{
    AlternativePrimal, AlternativeRecommendation, AlternativeSuggestions, AvailabilityReport,
    AvailabilitySummary, NodeAvailability, NodeAvailabilityStatus,
};
use crate::graph::types::{Graph, GraphNode};
use crate::ipc::registry::ServiceRegistry;
use crate::ipc::types::PrimalEndpoint;

/// Availability checker for graph nodes
///
/// Queries the service registry to determine if required primals are available
/// and suggests alternatives when needed.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use songbird_orchestrator::graph::{AvailabilityChecker, Graph, GraphMetadata};
/// use songbird_orchestrator::ipc::registry::ServiceRegistry;
///
/// # async fn example() -> anyhow::Result<()> {
/// let registry = Arc::new(ServiceRegistry::new());
/// let checker = AvailabilityChecker::new(registry);
///
/// let graph = Graph::new(
///     String::from("test"),
///     String::from("Test"),
///     vec![],
///     vec![],
///     GraphMetadata::default(),
/// );
///
/// let report = checker.check_availability(&graph).await?;
/// println!("Available nodes: {}", report.available.len());
/// # Ok(())
/// # }
/// ```
pub struct AvailabilityChecker {
    service_registry: Arc<ServiceRegistry>,
}

impl AvailabilityChecker {
    /// Create a new availability checker
    #[must_use]
    pub const fn new(service_registry: Arc<ServiceRegistry>) -> Self {
        Self {
            service_registry,
        }
    }

    /// Check availability of all nodes in a graph
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn check_availability(&self, graph: &Graph) -> Result<AvailabilityReport> {
        debug!("Checking availability for graph: {}", graph.id);

        let mut report = AvailabilityReport {
            available: Vec::new(),
            unavailable: Vec::new(),
            unhealthy: Vec::new(),
            degraded: Vec::new(),
            details: std::collections::HashMap::new(),
            summary: AvailabilitySummary {
                total_nodes: graph.nodes.len(),
                available_nodes: 0,
                availability_percent: 0.0,
            },
        };

        for node in &graph.nodes {
            let node_availability = self.check_node_availability(node).await?;
            let node_id = node.id.clone();

            match node_availability.status {
                NodeAvailabilityStatus::Available => {
                    report.available.push(node_id.clone());
                    report.summary.available_nodes += 1;
                }
                NodeAvailabilityStatus::Unavailable => {
                    report.unavailable.push(node_id.clone());
                }
                NodeAvailabilityStatus::Unhealthy => {
                    report.unhealthy.push(node_id.clone());
                }
                NodeAvailabilityStatus::Degraded => {
                    report.degraded.push(node_id.clone());
                }
            }

            report.details.insert(node_id, node_availability);
        }

        if report.summary.total_nodes > 0 {
            report.summary.availability_percent =
                (report.summary.available_nodes as f64 / report.summary.total_nodes as f64) * 100.0;
        }

        #[expect(
            clippy::float_cmp,
            reason = "intentional pattern; clippy false positive for this API"
        )] // exact 100.0 is an intentional sentinel
        if report.summary.availability_percent == 100.0 {
            info!(
                "Graph {} has 100% availability ({}/{})",
                graph.id, report.summary.available_nodes, report.summary.total_nodes
            );
        } else {
            warn!(
                "Graph {} has {:.1}% availability ({}/{})",
                graph.id,
                report.summary.availability_percent,
                report.summary.available_nodes,
                report.summary.total_nodes
            );
        }

        Ok(report)
    }

    /// Check availability of a single node (`pub(crate)` for unit tests in this module).
    pub(crate) async fn check_node_availability(
        &self,
        node: &GraphNode,
    ) -> Result<NodeAvailability> {
        debug!("Checking availability for node {} (capability: {})", node.id, node.capability);

        let primals = self.service_registry.discover_by_capability(&node.capability, None).await?;

        if primals.is_empty() {
            return Ok(NodeAvailability {
                status: NodeAvailabilityStatus::Unavailable,
                primal: None,
                service_id: None,
                endpoint: None,
                protocol: None,
                health_status: None,
                last_seen: None,
                required_capability: Some(node.capability.clone()),
                reason: Some(format!("No primal registered with capability '{}'", node.capability)),
                suggested_action: Some(format!(
                    "Register a primal with capability '{}' or use an alternative capability",
                    node.capability
                )),
            });
        }

        let preferred_primals: Vec<_> =
            if let Some(ref preferred_protocol) = node.preferred_protocol {
                primals.iter().filter(|p| p.protocol == *preferred_protocol).collect()
            } else {
                primals.iter().collect()
            };

        let is_available =
            |p: &PrimalEndpoint| p.health_status == "healthy" || p.health_status == "unknown";

        let best_primal = if preferred_primals.is_empty() {
            primals.iter().find(|p| is_available(p)).or_else(|| primals.first())
        } else {
            preferred_primals
                .iter()
                .find(|&&p| is_available(p))
                .or_else(|| preferred_primals.first())
                .copied()
        };

        if let Some(primal) = best_primal {
            let status = match primal.health_status.as_str() {
                "healthy" | "unknown" => NodeAvailabilityStatus::Available,
                "degraded" => NodeAvailabilityStatus::Degraded,
                "down" => NodeAvailabilityStatus::Unhealthy,
                _ => NodeAvailabilityStatus::Unhealthy,
            };

            Ok(NodeAvailability {
                status,
                primal: Some(primal.primal_name.clone()),
                service_id: Some(primal.service_id.clone()),
                endpoint: Some(primal.endpoint.clone()),
                protocol: Some(primal.protocol.clone()),
                health_status: Some(primal.health_status.clone()),
                last_seen: Some(primal.last_health_check.clone()),
                required_capability: None,
                reason: None,
                suggested_action: None,
            })
        } else {
            Ok(NodeAvailability {
                status: NodeAvailabilityStatus::Unavailable,
                primal: None,
                service_id: None,
                endpoint: None,
                protocol: None,
                health_status: None,
                last_seen: None,
                required_capability: Some(node.capability.clone()),
                reason: Some(String::from("Internal error: primal list empty")),
                suggested_action: None,
            })
        }
    }

    /// Suggest alternative primals for a node
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn suggest_alternatives(&self, node: &GraphNode) -> Result<AlternativeSuggestions> {
        debug!("Finding alternatives for node {} (capability: {})", node.id, node.capability);

        let primals = self.service_registry.discover_by_capability(&node.capability, None).await?;

        if primals.is_empty() {
            return Ok(AlternativeSuggestions {
                alternatives: Vec::new(),
                recommendation: None,
                unavailable_reason: Some(format!(
                    "No primal registered with capability '{}'",
                    node.capability
                )),
            });
        }

        let mut alternatives: Vec<AlternativePrimal> = primals
            .iter()
            .map(|primal| {
                let score = self.calculate_compatibility_score(node, primal);
                let reason = self.generate_suggestion_reason(node, primal, score);

                AlternativePrimal {
                    rank: 0,
                    service_id: primal.service_id.clone(),
                    primal_name: primal.primal_name.clone(),
                    endpoint: primal.endpoint.clone(),
                    protocol: primal.protocol.clone(),
                    health_status: primal.health_status.clone(),
                    last_seen: primal.last_health_check.clone(),
                    reason,
                    compatibility_score: score,
                }
            })
            .collect();

        alternatives.sort_by(|a, b| b.compatibility_score.cmp(&a.compatibility_score));

        for (i, alt) in alternatives.iter_mut().enumerate() {
            alt.rank = i + 1;
        }

        let recommendation = alternatives.first().map(|best| AlternativeRecommendation {
            service_id: best.service_id.clone(),
            reason: format!(
                "Best match: {} (compatibility score: {})",
                best.primal_name, best.compatibility_score
            ),
        });

        Ok(AlternativeSuggestions {
            alternatives,
            recommendation,
            unavailable_reason: None,
        })
    }

    /// Calculate compatibility score for a primal (`pub(crate)` for unit tests).
    pub(crate) fn calculate_compatibility_score(
        &self,
        node: &GraphNode,
        primal: &PrimalEndpoint,
    ) -> u32 {
        let _ = self;
        let mut score = 0u32;

        score += match primal.health_status.as_str() {
            "healthy" => 50,
            "unknown" => 45,
            "degraded" => 30,
            _ => 0,
        };

        if let Some(ref preferred_protocol) = node.preferred_protocol {
            if &primal.protocol == preferred_protocol {
                score += 40;
            } else if primal.protocol == "json-rpc" {
                score += 20;
            }
        } else {
            score += 20;
        }

        if !primal.last_health_check.is_empty() {
            score += 10;
        }

        score
    }

    /// Generate a human-readable reason for an alternative suggestion (`pub(crate)` for unit tests).
    pub(crate) fn generate_suggestion_reason(
        &self,
        node: &GraphNode,
        primal: &PrimalEndpoint,
        score: u32,
    ) -> String {
        let _ = self;
        let mut reasons = Vec::new();

        match primal.health_status.as_str() {
            "healthy" => reasons.push("healthy"),
            "degraded" => reasons.push("degraded but functional"),
            _ => reasons.push("unhealthy"),
        }

        if let Some(ref preferred_protocol) = node.preferred_protocol {
            if &primal.protocol == preferred_protocol {
                reasons.push("protocol match");
            } else if primal.protocol == "json-rpc" {
                reasons.push("universal protocol (json-rpc)");
            }
        }

        let capability_reason = format!("capability '{}'", node.capability);

        format!("{}, {} (score: {})", reasons.join(", "), capability_reason, score)
    }
}
