// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Sovereignty-Aware Universal Adapter
//!
//! This module provides the main sovereignty-aware adapter that orchestrates
//! routing, federation, and network optimization while maintaining sovereignty.

#![allow(
    clippy::unused_self,
    clippy::struct_excessive_bools,
    clippy::unused_async,
    reason = "unused bindings/imports in this compilation unit"
)]

use super::network_optimizer::NetworkEffectsOptimizer;
use super::router::SovereigntyRouter;
use super::types::{RoutingPath, SovereigntyAdapterConfig, SovereigntyAwareRoutingDecision};
use crate::types::{HealthStatus, ServiceInfo, UniversalRequest, UniversalResponse};
use crate::unified_adapter::UnifiedUniversalAdapter;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info, warn};
/// Sovereignty-aware enhancement to existing universal adapter
///
/// This adds sovereignty-aware routing and federation capabilities
/// while maintaining each primal's self-knowledge and independence
#[derive(Debug)]
pub struct SovereigntyAwareAdapter {
    /// Base universal adapter for service discovery and routing
    base_adapter: UnifiedUniversalAdapter,

    /// Sovereignty-aware routing engine
    sovereignty_router: SovereigntyRouter,

    /// Network effects optimizer
    network_optimizer: NetworkEffectsOptimizer,

    /// Configuration
    config: SovereigntyAdapterConfig,
}

impl SovereigntyAwareAdapter {
    /// Create new sovereignty-aware adapter
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn new() -> SongbirdResult<Self> {
        let base_adapter = UnifiedUniversalAdapter::new();
        let sovereignty_router = SovereigntyRouter::new();
        let network_optimizer = NetworkEffectsOptimizer::new();
        let config = SovereigntyAdapterConfig::default();

        Ok(Self {
            base_adapter,
            sovereignty_router,
            network_optimizer,
            config,
        })
    }

    /// Create with custom configuration
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn with_config(config: SovereigntyAdapterConfig) -> SongbirdResult<Self> {
        let base_adapter = UnifiedUniversalAdapter::new();
        let sovereignty_router = SovereigntyRouter::new();
        let network_optimizer = NetworkEffectsOptimizer::new();

        Ok(Self {
            base_adapter,
            sovereignty_router,
            network_optimizer,
            config,
        })
    }

    /// Route request with sovereignty awareness
    ///
    /// # Errors
    ///
    /// Returns an error if path finding, optimization, or routing decision creation fails
    pub async fn route_request(
        &self,
        request: UniversalRequest,
    ) -> SongbirdResult<SovereigntyAwareRoutingDecision> {
        debug!("Processing sovereignty-aware request: {:?}", request);

        // Discover available services via base adapter (registry, discovery endpoints)
        let available_services = self.base_adapter.discover_services().await.unwrap_or_default();

        // Find sovereignty-aware paths
        let mut candidate_paths = if self.config.enable_sovereignty_routing {
            self.sovereignty_router
                .find_sovereignty_aware_paths(&request, &available_services)
                .await?
        } else {
            // Fallback to basic routing
            self.generate_basic_paths(&available_services).await?
        };

        // Apply network effects optimization
        if self.config.enable_network_optimization {
            candidate_paths =
                self.network_optimizer.optimize_for_network_effects(&candidate_paths).await?;
        }

        // Select best path
        let selected_path = self.select_best_path(&candidate_paths)?;

        // Create routing decision
        let decision = self.create_routing_decision(selected_path, &candidate_paths).await?;

        info!(
            "Selected sovereignty-aware routing path with score: {}",
            decision.selected_path.combined_score
        );

        Ok(decision)
    }

    /// Execute request through sovereignty-aware routing
    ///
    /// # Errors
    ///
    /// Returns an error if routing decision fails or request execution fails
    pub async fn execute_request(
        &self,
        request: UniversalRequest,
    ) -> SongbirdResult<UniversalResponse> {
        // Get routing decision (borrow request for analysis)
        let routing_decision = self.route_request_borrowed(&request).await?;

        // Execute through selected path (consume request)
        self.execute_through_path(request, &routing_decision.selected_path).await
    }

    /// Route request using borrowed reference (zero-copy analysis)
    async fn route_request_borrowed(
        &self,
        request: &UniversalRequest,
    ) -> SongbirdResult<SovereigntyAwareRoutingDecision> {
        debug!("Processing sovereignty-aware request: {:?}", request);

        // Discover available services via base adapter (registry, discovery endpoints)
        let available_services = self.base_adapter.discover_services().await.unwrap_or_default();

        // Find sovereignty-aware paths
        let mut candidate_paths = if self.config.enable_sovereignty_routing {
            self.sovereignty_router
                .find_sovereignty_aware_paths(request, &available_services)
                .await?
        } else {
            // Fallback to basic routing
            self.generate_basic_paths(&available_services).await?
        };

        // Apply network effects optimization
        if self.config.enable_network_optimization {
            candidate_paths =
                self.network_optimizer.optimize_for_network_effects(&candidate_paths).await?;
        }

        // Select best path
        let selected_path = self.select_best_path(&candidate_paths)?;

        // Create routing decision
        let decision = self.create_routing_decision(selected_path, &candidate_paths).await?;

        info!(
            "Selected sovereignty-aware routing path with score: {}",
            decision.selected_path.combined_score
        );

        Ok(decision)
    }

    async fn generate_basic_paths(
        &self,
        services: &[ServiceInfo],
    ) -> SongbirdResult<Vec<RoutingPath>> {
        // Generate simple single-hop paths as fallback
        let mut paths = Vec::new();

        for service in services {
            let segment = super::types::PathSegment {
                service: service.clone(),
                sovereignty_level: super::types::SovereigntyLevel::ModeratelySovereign,
                efficiency_score: 0.7,
                security_capabilities: vec![
                    super::types::SecurityCapability::Encryption,
                    super::types::SecurityCapability::Authentication,
                ],
                metadata: std::collections::HashMap::new(),
            };

            let path = RoutingPath {
                segments: vec![segment],
                sovereignty_score: 0.6,
                efficiency_score: 0.7,
                combined_score: 0.65,
                security_level: super::types::SecurityLevel::Medium,
            };

            paths.push(path);
        }

        Ok(paths)
    }

    fn select_best_path(&self, paths: &[RoutingPath]) -> SongbirdResult<RoutingPath> {
        paths
            .iter()
            .max_by(|a, b| {
                a.combined_score.partial_cmp(&b.combined_score).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .ok_or_else(|| SongbirdError::network("No valid routing paths available"))
    }

    async fn create_routing_decision(
        &self,
        selected_path: RoutingPath,
        all_paths: &[RoutingPath],
    ) -> SongbirdResult<SovereigntyAwareRoutingDecision> {
        use super::types::{
            DecisionFactor, PathSovereigntyAssessment, RoutingDecisionMetadata, SecurityAssessment,
            SegmentSovereigntyAssessment, SovereigntyAwareRoutingDecision,
        };

        if selected_path.segments.is_empty() {
            warn!(
                "sovereignty routing decision: selected path has no service segments; \
                 federation capability list and segment assessments are empty (fail-closed metadata)"
            );
        } else {
            info!(
                segment_count = selected_path.segments.len(),
                "federation capabilities and network-effect hints derived from universal adapter discovery (per path segment)"
            );
        }

        let segment_assessments: Vec<SegmentSovereigntyAssessment> = selected_path
            .segments
            .iter()
            .enumerate()
            .map(|(i, seg)| SegmentSovereigntyAssessment {
                segment_id: format!("{}-{}", seg.service.name, i),
                sovereignty_score: seg.sovereignty_level.score(),
                sovereignty_level: seg.sovereignty_level.clone(),
                security_assessment: SecurityAssessment {
                    security_score: seg.efficiency_score.clamp(0.0, 1.0),
                    security_level: selected_path.security_level.clone(),
                    identified_vulnerabilities: vec![],
                },
            })
            .collect();

        // Create sovereignty assessment
        let sovereignty_assessment = PathSovereigntyAssessment {
            overall_score: selected_path.sovereignty_score,
            segment_assessments,
            compliance_level: self.determine_compliance_level(selected_path.sovereignty_score),
            sovereignty_risks: vec![],
        };

        let federation_capabilities = Self::federation_capabilities_from_path(&selected_path);

        let expected_network_effects = Self::network_effects_from_path(&selected_path);
        if !expected_network_effects.is_empty() {
            info!(
                effect_count = expected_network_effects.len(),
                "expected network effects are score-derived estimates until federation telemetry is integrated"
            );
        }

        // Create decision metadata
        let decision_metadata = RoutingDecisionMetadata {
            decision_timestamp: std::time::SystemTime::now(),
            algorithm_version: String::from("sovereignty-aware-v1.0"),
            decision_factors: vec![DecisionFactor {
                factor_name: String::from("sovereignty_score"),
                factor_weight: self.config.sovereignty_preference_weight,
                factor_value: selected_path.sovereignty_score,
            }],
            alternative_paths_count: all_paths.len().saturating_sub(1),
        };

        Ok(SovereigntyAwareRoutingDecision {
            selected_path,
            sovereignty_assessment,
            federation_capabilities,
            expected_network_effects,
            decision_metadata,
        })
    }

    async fn execute_through_path(
        &self,
        request: UniversalRequest,
        path: &RoutingPath,
    ) -> SongbirdResult<UniversalResponse> {
        if path.segments.is_empty() {
            warn!(
                request_id = %request.request_id,
                "execute_through_path: empty path has no discovered services; refusing execution"
            );
            return Err(SongbirdError::network(
                "Sovereignty path has no segments; cannot execute against a target service",
            ));
        }

        if path.segments.len() > 1 {
            warn!(
                request_id = %request.request_id,
                hop_count = path.segments.len(),
                "multi-hop path execution not implemented; delegating to single-hop UnifiedUniversalAdapter::route_request"
            );
        }

        info!(
            request_id = %request.request_id,
            "delegating request execution to UnifiedUniversalAdapter (per-hop chain execution tracked for songbird-universal)"
        );

        self.base_adapter.route_request(request).await.map_err(Into::into)
    }

    fn health_to_score(health: &HealthStatus) -> f64 {
        match health {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Degraded => 0.65,
            HealthStatus::Unhealthy => 0.0,
            HealthStatus::Unknown => 0.5,
        }
    }

    fn federation_type_from_capability_name(name: &str) -> super::types::FederationCapabilityType {
        use super::types::FederationCapabilityType;
        let n = name.to_lowercase();
        if n.contains("health") {
            FederationCapabilityType::HealthMonitoring
        } else if n.contains("consensus") {
            FederationCapabilityType::ConsensusParticipation
        } else if n.contains("replic") {
            FederationCapabilityType::DataReplication
        } else if n.contains("load") || n.contains("balance") {
            FederationCapabilityType::LoadDistribution
        } else if n.contains("route") || n.contains("optim") {
            FederationCapabilityType::RouteOptimization
        } else {
            FederationCapabilityType::CrossNodeCommunication
        }
    }

    fn federation_capabilities_from_path(
        path: &RoutingPath,
    ) -> Vec<super::types::FederationCapability> {
        use super::types::{FederationCapability, PerformanceCharacteristics};

        let mut out = Vec::new();
        for (hop, seg) in path.segments.iter().enumerate() {
            if seg.service.capabilities.is_empty() {
                out.push(FederationCapability {
                    capability_id: format!("service:{}:hop{}", seg.service.name, hop),
                    capability_type: super::types::FederationCapabilityType::CrossNodeCommunication,
                    availability_score: Self::health_to_score(&seg.service.health),
                    performance_characteristics: PerformanceCharacteristics {
                        latency_ms: (1.0 - seg.efficiency_score.clamp(0.0, 1.0)) * 100.0,
                        throughput_ops_per_sec: 1000.0 * seg.efficiency_score.clamp(0.0, 1.0),
                        reliability_score: Self::health_to_score(&seg.service.health),
                    },
                });
            } else {
                for cap in &seg.service.capabilities {
                    out.push(FederationCapability {
                        capability_id: format!("{}::{}", seg.service.name, cap.name),
                        capability_type: Self::federation_type_from_capability_name(&cap.name),
                        availability_score: Self::health_to_score(&cap.health_status),
                        performance_characteristics: PerformanceCharacteristics {
                            latency_ms: cap.qos_metrics.latency_ms.unwrap_or(0.0),
                            throughput_ops_per_sec: cap
                                .qos_metrics
                                .throughput_ops_sec
                                .unwrap_or(0.0),
                            reliability_score: cap
                                .qos_metrics
                                .reliability
                                .unwrap_or_else(|| Self::health_to_score(&cap.health_status)),
                        },
                    });
                }
            }
        }
        out
    }

    fn network_effects_from_path(path: &RoutingPath) -> Vec<super::types::ExpectedNetworkEffect> {
        use super::types::{ExpectedNetworkEffect, NetworkEffectType};

        let mut effects = Vec::new();
        effects.push(ExpectedNetworkEffect {
            effect_id: String::from("routing_efficiency_hint"),
            effect_type: NetworkEffectType::PerformanceImprovement,
            impact_magnitude: (path.efficiency_score - 0.5).abs(),
            confidence_level: 0.45,
        });
        if path.sovereignty_score > 0.6 {
            effects.push(ExpectedNetworkEffect {
                effect_id: String::from("routing_sovereignty_hint"),
                effect_type: NetworkEffectType::SecurityEnhancement,
                impact_magnitude: (path.sovereignty_score - 0.5).abs(),
                confidence_level: 0.45,
            });
        }
        effects
    }

    fn determine_compliance_level(
        &self,
        sovereignty_score: f64,
    ) -> super::types::SovereigntyComplianceLevel {
        use super::types::SovereigntyComplianceLevel;

        match sovereignty_score {
            score if score >= 0.9 => SovereigntyComplianceLevel::FullyCompliant,
            score if score >= 0.7 => SovereigntyComplianceLevel::MostlyCompliant,
            score if score >= 0.5 => SovereigntyComplianceLevel::PartiallyCompliant,
            _ => SovereigntyComplianceLevel::NonCompliant,
        }
    }

    /// Get adapter configuration
    #[must_use]
    pub const fn get_config(&self) -> &SovereigntyAdapterConfig {
        &self.config
    }

    /// Update adapter configuration
    pub const fn update_config(&mut self, config: SovereigntyAdapterConfig) {
        self.config = config;
    }

    /// Get adapter statistics
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_stats(&self) -> SongbirdResult<AdapterStats> {
        let registry = self.base_adapter.get_registry_stats().await;
        Ok(AdapterStats {
            sovereignty_routing_enabled: self.config.enable_sovereignty_routing,
            federation_routing_enabled: self.config.enable_federation_routing,
            network_optimization_enabled: self.config.enable_network_optimization,
            base_adapter_healthy: registry.total_services > 0,
        })
    }
}

/// Adapter statistics
#[derive(Debug, Clone)]
pub struct AdapterStats {
    /// Whether sovereignty routing is enabled
    pub sovereignty_routing_enabled: bool,
    /// Whether federation routing is enabled
    pub federation_routing_enabled: bool,
    /// Whether network optimization is enabled
    pub network_optimization_enabled: bool,
    /// Whether the base adapter is healthy
    pub base_adapter_healthy: bool,
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod tests;
