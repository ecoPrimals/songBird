// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Primal availability checking and alternative suggestions
//!
//! This module integrates with the service registry (v3.20.0) to check if required
//! primals are available for graph execution and suggest alternatives when needed.
//!
//! # Design Principles
//!
//! - **Zero Hardcoding**: Uses service registry for runtime discovery
//! - **Capability-Based**: Discovers by capability, not primal name
//! - **Health-Aware**: Considers primal health status in decisions
//! - **Protocol-Agnostic**: Supports multiple protocols with compatibility scoring

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::types::{Graph, GraphNode};
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
///     "test".to_string(),
///     "Test".to_string(),
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
    ///
    /// Queries the service registry for each node's required capability and
    /// categorizes nodes as:
    /// - Available: Primal registered and healthy
    /// - Unavailable: No primal with required capability
    /// - Unhealthy: Primal registered but health status is "down" or "unknown"
    /// - Degraded: Primal registered but health status is "degraded"
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

            match node_availability.status {
                NodeAvailabilityStatus::Available => {
                    report.available.push(node.id.clone());
                    report.summary.available_nodes += 1;
                }
                NodeAvailabilityStatus::Unavailable => {
                    report.unavailable.push(node.id.clone());
                }
                NodeAvailabilityStatus::Unhealthy => {
                    report.unhealthy.push(node.id.clone());
                }
                NodeAvailabilityStatus::Degraded => {
                    report.degraded.push(node.id.clone());
                }
            }

            report.details.insert(node.id.clone(), node_availability);
        }

        // Calculate availability percentage
        if report.summary.total_nodes > 0 {
            report.summary.availability_percent =
                (report.summary.available_nodes as f64 / report.summary.total_nodes as f64) * 100.0;
        }

        #[allow(clippy::float_cmp)] // exact 100.0 is an intentional sentinel
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

    /// Check availability of a single node
    async fn check_node_availability(&self, node: &GraphNode) -> Result<NodeAvailability> {
        debug!("Checking availability for node {} (capability: {})", node.id, node.capability);

        // Query service registry for primals with this capability
        // NOTE: We pass None for protocol to get all primals with the capability,
        // then filter/rank by protocol preference ourselves
        let primals = self.service_registry.discover_by_capability(&node.capability, None).await?;

        if primals.is_empty() {
            // No primal registered with this capability
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

        // Filter by protocol preference if specified
        let preferred_primals: Vec<_> =
            if let Some(ref preferred_protocol) = node.preferred_protocol {
                primals.iter().filter(|p| p.protocol == *preferred_protocol).collect()
            } else {
                primals.iter().collect()
            };

        // Choose the best primal (prefer preferred protocol and healthy status)
        // NOTE: "unknown" health status is treated as available (newly registered service)
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
                "healthy" | "unknown" => NodeAvailabilityStatus::Available, // "unknown" = newly registered, assume available
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
            // This shouldn't happen since we checked primals.is_empty() above
            Ok(NodeAvailability {
                status: NodeAvailabilityStatus::Unavailable,
                primal: None,
                service_id: None,
                endpoint: None,
                protocol: None,
                health_status: None,
                last_seen: None,
                required_capability: Some(node.capability.clone()),
                reason: Some("Internal error: primal list empty".to_string()),
                suggested_action: None,
            })
        }
    }

    /// Suggest alternative primals for a node
    ///
    /// Ranks alternatives based on:
    /// 1. Health status (healthy > degraded > down)
    /// 2. Protocol compatibility (exact match > compatible > incompatible)
    /// 3. Last seen timestamp (more recent = better)
    pub async fn suggest_alternatives(&self, node: &GraphNode) -> Result<AlternativeSuggestions> {
        debug!("Finding alternatives for node {} (capability: {})", node.id, node.capability);

        // Query service registry for all primals with this capability
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

        // Score and rank each primal
        let mut alternatives: Vec<AlternativePrimal> = primals
            .iter()
            .map(|primal| {
                let score = self.calculate_compatibility_score(node, primal);
                let reason = self.generate_suggestion_reason(node, primal, score);

                AlternativePrimal {
                    rank: 0, // Will be set after sorting
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

        // Sort by compatibility score (highest first)
        alternatives.sort_by(|a, b| b.compatibility_score.cmp(&a.compatibility_score));

        // Assign ranks
        for (i, alt) in alternatives.iter_mut().enumerate() {
            alt.rank = i + 1;
        }

        // Generate recommendation (best alternative)
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

    /// Calculate compatibility score for a primal
    ///
    /// Score breakdown:
    /// - Health status: healthy=50, unknown=45, degraded=30, down=0
    /// - Protocol match: exact=40, any=20, none=0
    /// - Recency: recent=10, old=5
    ///
    /// Maximum score: 100 (perfect match)
    fn calculate_compatibility_score(&self, node: &GraphNode, primal: &PrimalEndpoint) -> u32 {
        let mut score = 0u32;

        // Health status (0-50 points)
        score += match primal.health_status.as_str() {
            "healthy" => 50,
            "unknown" => 45, // Newly registered, assume available (slightly lower than healthy)
            "degraded" => 30,
            _ => 0,
        };

        // Protocol compatibility (0-40 points)
        if let Some(ref preferred_protocol) = node.preferred_protocol {
            if &primal.protocol == preferred_protocol {
                score += 40; // Exact match
            } else if primal.protocol == "json-rpc" {
                score += 20; // Universal fallback
            }
        } else {
            score += 20; // No preference, any protocol acceptable
        }

        // Recency (0-10 points)
        // NOTE: last_health_check is available from service registry
        // For now, we'll give 10 points if there's a timestamp (all healthy primals have it)
        if !primal.last_health_check.is_empty() {
            score += 10;
        }

        score
    }

    /// Generate a human-readable reason for an alternative suggestion
    fn generate_suggestion_reason(
        &self,
        node: &GraphNode,
        primal: &PrimalEndpoint,
        score: u32,
    ) -> String {
        let mut reasons = Vec::new();

        // Health status
        match primal.health_status.as_str() {
            "healthy" => reasons.push("healthy"),
            "degraded" => reasons.push("degraded but functional"),
            _ => reasons.push("unhealthy"),
        }

        // Protocol compatibility
        if let Some(ref preferred_protocol) = node.preferred_protocol {
            if &primal.protocol == preferred_protocol {
                reasons.push("protocol match");
            } else if primal.protocol == "json-rpc" {
                reasons.push("universal protocol (json-rpc)");
            }
        }

        // Capability match
        let capability_reason = format!("capability '{}'", node.capability);

        format!("{}, {} (score: {})", reasons.join(", "), capability_reason, score)
    }
}

/// Availability report for a graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilityReport {
    /// Node IDs that are available (healthy primals)
    pub available: Vec<String>,

    /// Node IDs that are unavailable (no primal registered)
    pub unavailable: Vec<String>,

    /// Node IDs with unhealthy primals
    pub unhealthy: Vec<String>,

    /// Node IDs with degraded primals
    pub degraded: Vec<String>,

    /// Detailed availability information for each node
    pub details: std::collections::HashMap<String, NodeAvailability>,

    /// Summary statistics
    pub summary: AvailabilitySummary,
}

/// Summary statistics for availability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvailabilitySummary {
    /// Total number of nodes in the graph
    pub total_nodes: usize,

    /// Number of available nodes
    pub available_nodes: usize,

    /// Availability percentage (0-100)
    pub availability_percent: f64,
}

/// Availability status for a single node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeAvailability {
    /// Availability status
    pub status: NodeAvailabilityStatus,

    /// Primal name (if available)
    pub primal: Option<String>,

    /// Service ID (if available)
    pub service_id: Option<String>,

    /// Endpoint (if available)
    pub endpoint: Option<String>,

    /// Protocol (if available)
    pub protocol: Option<String>,

    /// Health status (if available)
    pub health_status: Option<String>,

    /// Last seen timestamp (if available)
    pub last_seen: Option<String>,

    /// Required capability (if unavailable)
    pub required_capability: Option<String>,

    /// Reason for unavailability (if unavailable)
    pub reason: Option<String>,

    /// Suggested action (if unavailable)
    pub suggested_action: Option<String>,
}

/// Node availability status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NodeAvailabilityStatus {
    /// Primal available and healthy
    Available,
    /// No primal registered with required capability
    Unavailable,
    /// Primal registered but unhealthy (down/unknown)
    Unhealthy,
    /// Primal registered but degraded
    Degraded,
}

/// Alternative primal suggestions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativeSuggestions {
    /// List of alternative primals, ranked by compatibility
    pub alternatives: Vec<AlternativePrimal>,

    /// Recommended alternative (best match)
    pub recommendation: Option<AlternativeRecommendation>,

    /// Reason if no alternatives available
    pub unavailable_reason: Option<String>,
}

/// An alternative primal suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativePrimal {
    /// Rank (1 = best, 2 = second best, etc.)
    pub rank: usize,

    /// Service ID
    pub service_id: String,

    /// Primal name
    pub primal_name: String,

    /// Endpoint
    pub endpoint: String,

    /// Protocol
    pub protocol: String,

    /// Health status
    pub health_status: String,

    /// Last seen timestamp
    pub last_seen: String,

    /// Reason for suggestion
    pub reason: String,

    /// Compatibility score (0-100)
    pub compatibility_score: u32,
}

/// Recommended alternative
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlternativeRecommendation {
    /// Service ID of recommended primal
    pub service_id: String,

    /// Reason for recommendation
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{GraphMetadata, GraphNode};

    fn create_test_node(id: &str, capability: &str, protocol: Option<&str>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            primal_name: None,
            capability: capability.to_string(),
            inputs: vec![],
            outputs: vec![],
            config: serde_json::json!({}),
            preferred_protocol: protocol.map(std::string::ToString::to_string),
            timeout_secs: None,
        }
    }

    #[tokio::test]
    async fn test_all_available() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register a primal
        registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Create a graph with one node needing encryption
        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![create_test_node("node-1", "encryption", None)],
            vec![],
            GraphMetadata::default(),
        );

        let report = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report.available.len(), 1);
        assert_eq!(report.unavailable.len(), 0);
        assert_eq!(report.summary.availability_percent, 100.0);
    }

    #[tokio::test]
    async fn test_some_unavailable() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register only one primal
        registry
            .register_service(
                "BearDog".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/beardog.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Create a graph needing two capabilities, but only one is registered
        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![
                create_test_node("node-1", "encryption", None),
                create_test_node("node-2", "storage", None), // Not registered
            ],
            vec![],
            GraphMetadata::default(),
        );

        let report = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report.available.len(), 1);
        assert_eq!(report.unavailable.len(), 1);
        assert!(report.unavailable.contains(&"node-2".to_string()));
        assert_eq!(report.summary.availability_percent, 50.0);
    }

    #[tokio::test]
    async fn test_no_primals_registered() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry);

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![create_test_node("node-1", "encryption", None)],
            vec![],
            GraphMetadata::default(),
        );

        let report = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report.available.len(), 0);
        assert_eq!(report.unavailable.len(), 1);
        assert_eq!(report.summary.availability_percent, 0.0);
    }

    #[tokio::test]
    async fn test_protocol_filtering() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register primal with tarpc protocol
        registry
            .register_service(
                "FastPrimal".to_string(),
                vec!["encryption".to_string()],
                "tcp://localhost:5000".to_string(),
                "tarpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Node prefers json-rpc but tarpc should still work
        let node = create_test_node("node-1", "encryption", Some("json-rpc"));

        let report = checker.check_node_availability(&node).await.unwrap();
        // Should still find the tarpc primal
        assert_eq!(report.status, NodeAvailabilityStatus::Available);
        assert_eq!(report.protocol, Some("tarpc".to_string()));
    }

    #[tokio::test]
    async fn test_suggest_alternatives_ranking() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register multiple primals with different health statuses
        let healthy_service_id = registry
            .register_service(
                "HealthyPrimal".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/healthy.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        let degraded_service_id = registry
            .register_service(
                "DegradedPrimal".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/degraded.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Mark one as healthy and one as degraded (use actual service IDs)
        registry.update_health(&healthy_service_id, "healthy".to_string()).await.unwrap();

        registry.update_health(&degraded_service_id, "degraded".to_string()).await.unwrap();

        let node = create_test_node("node-1", "encryption", Some("json-rpc"));
        let suggestions = checker.suggest_alternatives(&node).await.unwrap();

        assert_eq!(suggestions.alternatives.len(), 2);
        // Healthy primal should rank first
        assert_eq!(suggestions.alternatives[0].rank, 1);
        assert!(suggestions.alternatives[0].primal_name.contains("Healthy"));
        assert!(
            suggestions.alternatives[0].compatibility_score
                > suggestions.alternatives[1].compatibility_score
        );
    }

    #[tokio::test]
    async fn test_unhealthy_primal() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register a primal
        let service_id = registry
            .register_service(
                "UnhealthyPrimal".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/unhealthy.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Mark it as down
        registry.update_health(&service_id, "down".to_string()).await.unwrap();

        let node = create_test_node("node-1", "encryption", None);
        let report = checker.check_node_availability(&node).await.unwrap();

        // Should be unhealthy (not available for use)
        assert_eq!(report.status, NodeAvailabilityStatus::Unhealthy);
        assert_eq!(report.health_status, Some("down".to_string()));
    }

    #[tokio::test]
    async fn test_degraded_primal() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register a primal
        let service_id = registry
            .register_service(
                "DegradedPrimal".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/degraded.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        // Mark it as degraded
        registry.update_health(&service_id, "degraded".to_string()).await.unwrap();

        let node = create_test_node("node-1", "encryption", None);
        let report = checker.check_node_availability(&node).await.unwrap();

        // Should be degraded (available but not ideal)
        assert_eq!(report.status, NodeAvailabilityStatus::Degraded);
        assert_eq!(report.health_status, Some("degraded".to_string()));
    }

    #[tokio::test]
    async fn test_health_status_changes() {
        let registry = Arc::new(ServiceRegistry::new());
        let checker = AvailabilityChecker::new(registry.clone());

        // Register a primal
        let service_id = registry
            .register_service(
                "FlakeyPrimal".to_string(),
                vec!["encryption".to_string()],
                "/run/user/1000/flakey.sock".to_string(),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();

        let graph = Graph::new(
            "test".to_string(),
            "Test".to_string(),
            vec![create_test_node("node-1", "encryption", None)],
            vec![],
            GraphMetadata::default(),
        );

        // Initial: unknown (newly registered) -> treated as available
        let report1 = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report1.summary.availability_percent, 100.0);

        // Mark as healthy
        registry.update_health(&service_id, "healthy".to_string()).await.unwrap();
        let report2 = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report2.summary.availability_percent, 100.0);

        // Mark as degraded
        registry.update_health(&service_id, "degraded".to_string()).await.unwrap();
        let report3 = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report3.available.len(), 0);
        assert_eq!(report3.degraded.len(), 1);

        // Mark as down
        registry.update_health(&service_id, "down".to_string()).await.unwrap();
        let report4 = checker.check_availability(&graph).await.unwrap();
        assert_eq!(report4.available.len(), 0);
        assert_eq!(report4.unhealthy.len(), 1);
        assert_eq!(report4.summary.availability_percent, 0.0);
    }
}
