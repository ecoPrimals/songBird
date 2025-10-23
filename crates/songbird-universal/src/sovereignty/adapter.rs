//! Sovereignty-Aware Universal Adapter
//!
//! This module provides the main sovereignty-aware adapter that orchestrates
//! routing, federation, and network optimization while maintaining sovereignty.

use super::network_optimizer::NetworkEffectsOptimizer;
use super::router::SovereigntyRouter;
use super::types::{RoutingPath, SovereigntyAdapterConfig, SovereigntyAwareRoutingDecision};
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
    #[allow(dead_code)] // Reserved for future delegation to base adapter
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
    ///
    /// # Errors
    ///
    /// Returns an error if routing decision fails or request execution fails
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
        use super::types::{
            DecisionFactor, ExpectedNetworkEffect, FederationCapability, FederationCapabilityType,
            NetworkEffectType, PathSovereigntyAssessment, PerformanceCharacteristics,
            RoutingDecisionMetadata, SovereigntyAwareRoutingDecision,
        };

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
        _path: &RoutingPath,
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
    #[must_use]
    pub fn get_config(&self) -> &SovereigntyAdapterConfig {
        &self.config
    }

    /// Update adapter configuration
    pub fn update_config(&mut self, config: SovereigntyAdapterConfig) {
        self.config = config;
    }

    /// Get adapter statistics
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_creation_default() {
        let adapter = SovereigntyAwareAdapter::new().await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_adapter_creation_with_config() {
        let config = SovereigntyAdapterConfig::default();
        let adapter = SovereigntyAwareAdapter::with_config(config).await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_adapter_config_custom_settings() -> Result<(), Box<dyn std::error::Error>> {
        let config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: false,
            enable_federation_routing: false,
            enable_network_optimization: true,
            sovereignty_timeout: std::time::Duration::from_secs(10),
            sovereignty_preference_weight: 0.5,
        };

        let adapter = SovereigntyAwareAdapter::with_config(config).await?;
        let retrieved_config = adapter.get_config();
        assert!(!retrieved_config.enable_sovereignty_routing);
        assert!(!retrieved_config.enable_federation_routing);
        assert!(retrieved_config.enable_network_optimization);
        assert_eq!(retrieved_config.sovereignty_timeout, std::time::Duration::from_secs(10));
        assert_eq!(retrieved_config.sovereignty_preference_weight, 0.5);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_config() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = SovereigntyAwareAdapter::new().await?;
        let config = adapter.get_config();

        assert!(config.enable_sovereignty_routing);
        assert!(config.enable_federation_routing);
        assert!(config.enable_network_optimization);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_config() -> Result<(), Box<dyn std::error::Error>> {
        let mut adapter = SovereigntyAwareAdapter::new().await?;

        let new_config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: false,
            enable_federation_routing: true,
            enable_network_optimization: false,
            sovereignty_timeout: std::time::Duration::from_secs(5),
            sovereignty_preference_weight: 0.6,
        };

        adapter.update_config(new_config);
        let config = adapter.get_config();

        assert!(!config.enable_sovereignty_routing);
        assert!(config.enable_federation_routing);
        assert!(!config.enable_network_optimization);
        assert_eq!(config.sovereignty_timeout, std::time::Duration::from_secs(5));
        assert_eq!(config.sovereignty_preference_weight, 0.6);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_stats() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = SovereigntyAwareAdapter::new()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;
        let stats = adapter.get_stats().await;

        assert!(stats.is_ok());
        let stats = stats.map_err(|e| {
            SongbirdError::configuration(format!("Test: stats should be available: {}", e))
        })?;

        assert!(stats.sovereignty_routing_enabled);
        assert!(stats.federation_routing_enabled);
        assert!(stats.network_optimization_enabled);
        assert!(stats.base_adapter_healthy);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_stats_with_disabled_features() -> Result<(), Box<dyn std::error::Error>> {
        let config = SovereigntyAdapterConfig {
            enable_sovereignty_routing: false,
            enable_federation_routing: false,
            enable_network_optimization: false,
            sovereignty_timeout: std::time::Duration::from_secs(3),
            sovereignty_preference_weight: 0.8,
        };

        let adapter = SovereigntyAwareAdapter::with_config(config)
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;
        let stats = adapter
            .get_stats()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: stats retrieval: {}", e)))?;

        assert!(!stats.sovereignty_routing_enabled);
        assert!(!stats.federation_routing_enabled);
        assert!(!stats.network_optimization_enabled);
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_fully_compliant() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.95);
        assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::FullyCompliant));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_mostly_compliant() -> Result<(), Box<dyn std::error::Error>>
    {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.75);
        assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::MostlyCompliant));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_partially_compliant(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.55);
        assert!(matches!(
            level,
            super::super::types::SovereigntyComplianceLevel::PartiallyCompliant
        ));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_non_compliant() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.3);
        assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::NonCompliant));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_boundary_90() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.9);
        assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::FullyCompliant));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_boundary_70() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.7);
        assert!(matches!(level, super::super::types::SovereigntyComplianceLevel::MostlyCompliant));
        Ok(())
    }

    #[test]
    fn test_determine_compliance_level_boundary_50() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let level = adapter.determine_compliance_level(0.5);
        assert!(matches!(
            level,
            super::super::types::SovereigntyComplianceLevel::PartiallyCompliant
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_basic_paths_empty_services() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = SovereigntyAwareAdapter::new()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;
        let services = vec![];

        let result = adapter.generate_basic_paths(&services).await;
        assert!(result.is_ok());

        let paths = result
            .map_err(|e| SongbirdError::configuration(format!("Test: paths generation: {}", e)))?;
        assert!(paths.is_empty());
        Ok(())
    }

    #[test]
    fn test_select_best_path_empty_list() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let paths = vec![];
        let result = adapter.select_best_path(&paths);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_select_best_path_single_path() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let path = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.8,
            efficiency_score: 0.7,
            combined_score: 0.75,
            security_level: super::super::types::SecurityLevel::High,
        };

        let paths = vec![path.clone()];
        let result = adapter.select_best_path(&paths);

        assert!(result.is_ok());
        let selected = result
            .map_err(|e| SongbirdError::configuration(format!("Test: path selection: {}", e)))?;
        assert_eq!(selected.combined_score, 0.75);
        Ok(())
    }

    #[test]
    fn test_select_best_path_multiple_paths() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = futures::executor::block_on(SovereigntyAwareAdapter::new())
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter creation: {}", e)))?;

        let path1 = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.8,
            efficiency_score: 0.7,
            combined_score: 0.75,
            security_level: super::super::types::SecurityLevel::High,
        };

        let path2 = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.9,
            efficiency_score: 0.85,
            combined_score: 0.88,
            security_level: super::super::types::SecurityLevel::Maximum,
        };

        let path3 = RoutingPath {
            segments: vec![],
            sovereignty_score: 0.6,
            efficiency_score: 0.9,
            combined_score: 0.70,
            security_level: super::super::types::SecurityLevel::Medium,
        };

        let paths = vec![path1, path2, path3];
        let result = adapter.select_best_path(&paths);

        assert!(result.is_ok());
        let selected = result
            .map_err(|e| SongbirdError::configuration(format!("Test: path selection: {}", e)))?;
        assert_eq!(selected.combined_score, 0.88); // Should select path2
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_stats_structure() {
        let stats = AdapterStats {
            sovereignty_routing_enabled: true,
            federation_routing_enabled: false,
            network_optimization_enabled: true,
            base_adapter_healthy: true,
        };

        assert!(stats.sovereignty_routing_enabled);
        assert!(!stats.federation_routing_enabled);
        assert!(stats.network_optimization_enabled);
        assert!(stats.base_adapter_healthy);
    }

    #[tokio::test]
    async fn test_multiple_adapters_independent() -> Result<(), Box<dyn std::error::Error>> {
        let config1 = SovereigntyAdapterConfig {
            enable_sovereignty_routing: true,
            enable_federation_routing: false,
            enable_network_optimization: true,
            sovereignty_timeout: std::time::Duration::from_secs(3),
            sovereignty_preference_weight: 0.8,
        };

        let config2 = SovereigntyAdapterConfig {
            enable_sovereignty_routing: false,
            enable_federation_routing: true,
            enable_network_optimization: false,
            sovereignty_timeout: std::time::Duration::from_secs(5),
            sovereignty_preference_weight: 0.5,
        };

        let adapter1 = SovereigntyAwareAdapter::with_config(config1)
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter1 creation: {}", e)))?;
        let adapter2 = SovereigntyAwareAdapter::with_config(config2)
            .await
            .map_err(|e| SongbirdError::configuration(format!("Test: adapter2 creation: {}", e)))?;

        // Verify they're independent
        assert!(adapter1.get_config().enable_sovereignty_routing);
        assert!(!adapter2.get_config().enable_sovereignty_routing);

        assert!(!adapter1.get_config().enable_federation_routing);
        assert!(adapter2.get_config().enable_federation_routing);

        Ok(())
    }
}
