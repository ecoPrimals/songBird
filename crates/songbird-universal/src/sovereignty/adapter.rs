//! Sovereignty-Aware Universal Adapter
//!
//! This module provides the main sovereignty-aware adapter that orchestrates
//! routing, federation, and network optimization while maintaining sovereignty.

use super::network_optimizer::NetworkEffectsOptimizer;
use super::router::SovereigntyRouter;
use super::types::{
    RoutingPath, SovereigntyAdapterConfig,
    SovereigntyAwareRoutingDecision,
};
use crate::types::{ServiceInfo, UniversalRequest, UniversalResponse};
use crate::unified_adapter::UnifiedUniversalAdapter;
use songbird_types::{SongbirdError, SongbirdResult};
use tracing::{debug, info};
/// Sovereignty-aware enhancement to existing universal adapter
///
/// This adds sovereignty-aware routing and federation capabilities
/// while maintaining each primal's self-knowledge and independence
#[derive(Debug)]
pub struct SovereigntyAwareAdapter {
    /// Base universal adapter (already exists)
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
    pub async fn route_request(
        &self,
        request: UniversalRequest,
    ) -> SongbirdResult<SovereigntyAwareRoutingDecision> {
        debug!("Processing sovereignty-aware request: {:?}", request);

        // Get available services from base adapter
        let available_services = vec![]; // Placeholder for service discovery

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
    pub async fn execute_request(
        &self,
        request: UniversalRequest,
    ) -> SongbirdResult<UniversalResponse> {
        // Get routing decision
        let routing_decision = self.route_request(request.clone()).await?;

        // Execute through selected path
        self.execute_through_path(request, &routing_decision.selected_path).await
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
        use super::types::*;

        // Create sovereignty assessment
        let sovereignty_assessment = PathSovereigntyAssessment {
            overall_score: selected_path.sovereignty_score,
            segment_assessments: vec![], // Would be populated in full implementation
            compliance_level: self.determine_compliance_level(selected_path.sovereignty_score),
            sovereignty_risks: vec![], // Would be populated in full implementation
        };

        // Create federation capabilities (placeholder)
        let federation_capabilities = vec![FederationCapability {
            capability_id: "cross_node_comm".to_string(),
            capability_type: FederationCapabilityType::CrossNodeCommunication,
            availability_score: 0.9,
            performance_characteristics: PerformanceCharacteristics {
                latency_ms: 10.0,
                throughput_ops_per_sec: 1000.0,
                reliability_score: 0.95,
            },
        }];

        // Create expected network effects (placeholder)
        let expected_network_effects = vec![ExpectedNetworkEffect {
            effect_id: "performance_boost".to_string(),
            effect_type: NetworkEffectType::PerformanceImprovement,
            impact_magnitude: 0.15,
            confidence_level: 0.8,
        }];

        // Create decision metadata
        let decision_metadata = RoutingDecisionMetadata {
            decision_timestamp: std::time::SystemTime::now(),
            algorithm_version: "sovereignty-aware-v1.0".to_string(),
            decision_factors: vec![DecisionFactor {
                factor_name: "sovereignty_score".to_string(),
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
        // For now, delegate to base adapter
        // In a full implementation, this would route through the specific path
        // For now, return a success response - will be implemented with proper routing
        Ok(UniversalResponse {
            request_id: request.request_id.clone(),
            status: crate::types::ResponseStatus::Success,
            data: Some(serde_json::json!({"sovereignty": "routed"})),
            metadata: std::collections::HashMap::new(),
            error: None,
        })
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
    pub fn get_config(&self) -> &SovereigntyAdapterConfig {
        &self.config
    }

    /// Update adapter configuration
    pub fn update_config(&mut self, config: SovereigntyAdapterConfig) {
        self.config = config;
    }

    /// Get adapter statistics
    pub async fn get_stats(&self) -> SongbirdResult<AdapterStats> {
        Ok(AdapterStats {
            sovereignty_routing_enabled: self.config.enable_sovereignty_routing,
            federation_routing_enabled: self.config.enable_federation_routing,
            network_optimization_enabled: self.config.enable_network_optimization,
            base_adapter_healthy: true, // Would check base adapter health
        })
    }
}

/// Adapter statistics
#[derive(Debug, Clone)]
pub struct AdapterStats {
    pub sovereignty_routing_enabled: bool,
    pub federation_routing_enabled: bool,
    pub network_optimization_enabled: bool,
    pub base_adapter_healthy: bool,
}
