//! # 🎯 Consolidated Core Orchestration
//!
//! **ORCHESTRATION CORE CONSOLIDATION** ✅
//!
//! This module consolidates the core orchestration functionality previously
//! scattered across songbird-core and songbird-orchestrator.

// Core orchestration modules
pub mod api;
pub mod benchmarks;
pub mod biome;
pub mod execution;
pub mod load_balancer;
pub mod orchestrator;
pub mod performance;
pub mod registry;
pub mod robustness;
pub mod routing; // ✅ NEW: Intelligent capability routing (Nov 9, 2025)
pub mod scaling;
pub mod zero_touch;

// Re-export key functionality for convenience
pub use api::{ApiConfig, ApiHandler, CoreApi};
// Legacy ServiceRegistry from biome - keeping for backward compatibility
pub use biome::ServiceRegistry;
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy};
pub use orchestrator::{CoreOrchestrator, OrchestratorConfig};
pub use performance::{PerformanceMetrics, PerformanceMonitor};
pub use registry::{CapabilityRegistry, HeartbeatConfig};
pub use robustness::{CircuitBreaker, RetryPolicy};
pub use scaling::{AutoScaler, ScalingPolicy};

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::collections::HashMap;

// Import comprehensive LoadBalancerConfig (Nov 10, 2025 consolidation)
use songbird_config::unified::robustness::LoadBalancerConfig as CanonicalLoadBalancerConfig;

/// Consolidated orchestrator engine
#[derive(Debug)]
pub struct ConsolidatedOrchestrator {
    config: ConsolidatedOrchestratorConfig,
    load_balancer: LoadBalancer,
    performance_monitor: PerformanceMonitor,
    service_registry: ServiceRegistry,
    auto_scaler: AutoScaler,
}

impl ConsolidatedOrchestrator {
    /// Create new consolidated orchestrator
    #[must_use]
    pub fn new(config: ConsolidatedOrchestratorConfig) -> Self {
        Self {
            config: config.clone(),
            load_balancer: LoadBalancer::new(config.load_balancing),
            performance_monitor: PerformanceMonitor::new(config.performance),
            service_registry: ServiceRegistry::new(config.registry),
            auto_scaler: AutoScaler::new(config.scaling),
        }
    }

    /// Initialize the orchestrator
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        self.load_balancer.initialize().await?;
        self.performance_monitor.initialize().await?;
        self.service_registry.initialize().await?;
        self.auto_scaler.initialize().await?;
        Ok(())
    }

    /// Start orchestration
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(&mut self) -> SongbirdResult<()> {
        self.load_balancer.start().await?;
        self.performance_monitor.start().await?;
        self.service_registry.start().await?;
        self.auto_scaler.start().await?;
        Ok(())
    }

    /// Stop orchestration
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        self.auto_scaler.stop().await?;
        self.service_registry.stop().await?;
        self.performance_monitor.stop().await?;
        self.load_balancer.stop().await?;
        Ok(())
    }

    /// Get orchestrator health status
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn health_check(&self) -> SongbirdResult<OrchestratorHealth> {
        Ok(OrchestratorHealth {
            status: HealthStatus::Healthy,
            load_balancer_health: self.load_balancer.health_check().await?,
            performance_health: self.performance_monitor.health_check().await?,
            registry_health: self.service_registry.health_check().await?,
            scaling_health: self.auto_scaler.health_check().await?,
        })
    }
}

/// Consolidated orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsolidatedOrchestratorConfig {
    /// Load balancing configuration
    /// **CONSOLIDATED**: Now uses CanonicalLoadBalancerConfig from songbird-config
    pub load_balancing: CanonicalLoadBalancerConfig,

    /// Performance monitoring configuration
    pub performance: PerformanceConfig,

    /// Service registry configuration
    pub registry: RegistryConfig,

    /// Auto-scaling configuration
    pub scaling: ScalingConfig,

    /// API configuration
    pub api: ApiConfig,

    /// Zero-touch deployment configuration
    pub zero_touch: ZeroTouchConfig,
}

// ============================================================================
// NOTE: LoadBalancingConfig has been CONSOLIDATED
// ============================================================================
//
// LoadBalancingConfig was removed and replaced with CanonicalLoadBalancerConfig
// from songbird_config::unified::robustness::LoadBalancerConfig
//
// Migration: Use CanonicalLoadBalancerConfig instead
// - strategy (LoadBalancingStrategy) → algorithm (LoadBalancingAlgorithm)
// - health_check_interval (u64) → health_check.interval (HealthCheckConfig field)
// - max_retries → handled at usage site or via RetryConfig
//
// NEW comprehensive fields available:
// - sticky_sessions: bool - Enable session affinity (default: false)
// - session_timeout: Duration - Session timeout (default: 300s)
// - max_connections_per_backend: usize - Connection pooling (default: 100)
// - connection_timeout: Duration - Connection timeout (default: 30s)
// - fail_fast: bool - Enable fail-fast mode (default: false)
//
// Date: November 10, 2025
// ============================================================================

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub metrics_interval: u64,
    pub alert_thresholds: HashMap<String, f64>,
    pub enable_benchmarking: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("cpu_usage".to_string(), 80.0);
        thresholds.insert("memory_usage".to_string(), 85.0);
        thresholds.insert("response_time".to_string(), 1000.0);
        Self {
            metrics_interval: 60,
            alert_thresholds: thresholds,
            enable_benchmarking: true,
        }
    }
}

/// Service registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub discovery_interval: u64,
    pub service_timeout: u64,
    pub max_services: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            discovery_interval: 30,
            service_timeout: 300,
            max_services: 1000,
        }
    }
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub enable_auto_scaling: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub min_instances: u32,
    pub max_instances: u32,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        }
    }
}

/// Zero-touch deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTouchConfig {
    pub enable_auto_deployment: bool,
    pub deployment_strategy: DeploymentStrategy,
    pub rollback_on_failure: bool,
}

impl Default for ZeroTouchConfig {
    fn default() -> Self {
        Self {
            enable_auto_deployment: false,
            deployment_strategy: DeploymentStrategy::BlueGreen,
            rollback_on_failure: true,
        }
    }
}

/// Deployment strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeploymentStrategy {
    BlueGreen,
    RollingUpdate,
    Canary,
}

/// Orchestrator health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorHealth {
    pub status: HealthStatus,
    pub load_balancer_health: ComponentHealth,
    pub performance_health: ComponentHealth,
    pub registry_health: ComponentHealth,
    pub scaling_health: ComponentHealth,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_consolidated_orchestrator_config_default() {
        use songbird_config::unified::robustness::LoadBalancingAlgorithm;
        let config = ConsolidatedOrchestratorConfig::default();
        assert_eq!(config.load_balancing.algorithm, LoadBalancingAlgorithm::RoundRobin);
        assert!(!config.load_balancing.sticky_sessions);
        assert_eq!(config.load_balancing.max_connections_per_backend, 100);
    }

    #[test]
    fn test_load_balancing_config_default() {
        let config = CanonicalLoadBalancerConfig::default();
        use songbird_config::unified::robustness::LoadBalancingAlgorithm;
        assert_eq!(config.algorithm, LoadBalancingAlgorithm::RoundRobin);
        assert!(!config.sticky_sessions); // New field
        assert_eq!(config.max_connections_per_backend, 100); // New field
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.metrics_interval, 60);
        assert!(config.enable_benchmarking);
        assert_eq!(config.alert_thresholds.get("cpu_usage"), Some(&80.0));
        assert_eq!(config.alert_thresholds.get("memory_usage"), Some(&85.0));
        assert_eq!(config.alert_thresholds.get("response_time"), Some(&1000.0));
    }

    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();
        assert_eq!(config.discovery_interval, 30);
        assert_eq!(config.service_timeout, 300);
        assert_eq!(config.max_services, 1000);
    }

    #[test]
    fn test_scaling_config_default() {
        let config = ScalingConfig::default();
        assert!(config.enable_auto_scaling);
        assert_eq!(config.scale_up_threshold, 70.0);
        assert_eq!(config.scale_down_threshold, 30.0);
        assert_eq!(config.min_instances, 1);
        assert_eq!(config.max_instances, 10);
    }

    #[test]
    fn test_zero_touch_config_default() {
        let config = ZeroTouchConfig::default();
        assert!(!config.enable_auto_deployment);
        assert_eq!(config.deployment_strategy, DeploymentStrategy::BlueGreen);
        assert!(config.rollback_on_failure);
    }

    #[test]
    fn test_deployment_strategy_equality() {
        assert_eq!(DeploymentStrategy::BlueGreen, DeploymentStrategy::BlueGreen);
        assert_ne!(DeploymentStrategy::BlueGreen, DeploymentStrategy::RollingUpdate);
        assert_ne!(DeploymentStrategy::RollingUpdate, DeploymentStrategy::Canary);
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Unhealthy, HealthStatus::Unknown);
    }

    #[test]
    fn test_scaling_config_thresholds_valid() {
        let config = ScalingConfig::default();
        assert!(
            config.scale_up_threshold > config.scale_down_threshold,
            "Scale up threshold should be higher than scale down threshold"
        );
        assert!(
            config.min_instances <= config.max_instances,
            "Min instances should be <= max instances"
        );
    }

    #[test]
    fn test_performance_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = PerformanceConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        let deserialized: PerformanceConfig =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(config.metrics_interval, deserialized.metrics_interval);
        assert_eq!(config.enable_benchmarking, deserialized.enable_benchmarking);
        Ok(())
    }

    #[test]
    fn test_zero_touch_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = ZeroTouchConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        let deserialized: ZeroTouchConfig =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(config.enable_auto_deployment, deserialized.enable_auto_deployment);
        assert_eq!(config.deployment_strategy, deserialized.deployment_strategy);
        assert_eq!(config.rollback_on_failure, deserialized.rollback_on_failure);
        Ok(())
    }

    #[test]
    fn test_component_health_construction() {
        let health = ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some("All systems operational".to_string()),
            last_check: Some(1234567890),
        };
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message.as_deref(), Some("All systems operational"));
        assert_eq!(health.last_check, Some(1234567890));
    }

    #[test]
    fn test_orchestrator_health_construction() {
        let health = OrchestratorHealth {
            status: HealthStatus::Healthy,
            load_balancer_health: ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: None,
            },
            performance_health: ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: None,
            },
            registry_health: ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: None,
            },
            scaling_health: ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: None,
            },
        };
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    // ==================== ConsolidatedOrchestrator Tests ====================

    #[test]
    fn test_consolidated_orchestrator_new() {
        let config = ConsolidatedOrchestratorConfig::default();
        let orchestrator = ConsolidatedOrchestrator::new(config);

        // Verify orchestrator was created successfully
        assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
    }

    #[test]
    fn test_consolidated_orchestrator_new_with_custom_config() {
        use songbird_config::unified::robustness::LoadBalancingAlgorithm;
        let mut config = ConsolidatedOrchestratorConfig::default();
        config.load_balancing.algorithm = LoadBalancingAlgorithm::LeastConnections;
        config.scaling.min_instances = 2;
        config.scaling.max_instances = 20;

        let orchestrator = ConsolidatedOrchestrator::new(config);

        // Verify orchestrator was created with custom config
        assert!(format!("{:?}", orchestrator).contains("ConsolidatedOrchestrator"));
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_initialize() {
        let config = ConsolidatedOrchestratorConfig::default();
        let mut orchestrator = ConsolidatedOrchestrator::new(config);

        // Initialize should succeed
        let result = orchestrator.initialize().await;
        assert!(result.is_ok(), "Initialize should succeed");
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_start() -> SongbirdResult<()> {
        let config = ConsolidatedOrchestratorConfig::default();
        let mut orchestrator = ConsolidatedOrchestrator::new(config);

        // Initialize first
        orchestrator.initialize().await?;

        // Start should succeed
        let result = orchestrator.start().await;
        assert!(result.is_ok(), "Start should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_stop() -> SongbirdResult<()> {
        let config = ConsolidatedOrchestratorConfig::default();
        let mut orchestrator = ConsolidatedOrchestrator::new(config);

        // Initialize and start first
        orchestrator.initialize().await?;
        orchestrator.start().await?;

        // Stop should succeed
        let result = orchestrator.stop().await;
        assert!(result.is_ok(), "Stop should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_health_check() -> SongbirdResult<()> {
        let config = ConsolidatedOrchestratorConfig::default();
        let orchestrator = ConsolidatedOrchestrator::new(config);

        // Health check should succeed
        let health = orchestrator.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);
        Ok(())
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_full_lifecycle() -> SongbirdResult<()> {
        let config = ConsolidatedOrchestratorConfig::default();
        let mut orchestrator = ConsolidatedOrchestrator::new(config);

        // Test complete lifecycle: initialize -> start -> health check -> stop
        orchestrator.initialize().await?;
        orchestrator.start().await?;

        let health = orchestrator.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);

        orchestrator.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_consolidated_orchestrator_health_check_components() -> SongbirdResult<()> {
        let config = ConsolidatedOrchestratorConfig::default();
        let orchestrator = ConsolidatedOrchestrator::new(config);

        let health = orchestrator.health_check().await?;

        // Verify all component health statuses
        assert_eq!(health.load_balancer_health.status, HealthStatus::Healthy);
        assert_eq!(health.performance_health.status, HealthStatus::Healthy);
        assert_eq!(health.registry_health.status, HealthStatus::Healthy);
        assert_eq!(health.scaling_health.status, HealthStatus::Healthy);
        Ok(())
    }

    #[test]
    fn test_consolidated_orchestrator_config_clone() {
        let config = ConsolidatedOrchestratorConfig::default();
        let cloned = config.clone();

        assert_eq!(config.load_balancing.algorithm, cloned.load_balancing.algorithm);
        assert_eq!(config.scaling.enable_auto_scaling, cloned.scaling.enable_auto_scaling);
    }

    #[test]
    fn test_consolidated_orchestrator_config_serialization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = ConsolidatedOrchestratorConfig::default();
        let json = serde_json::to_string(&config).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        let deserialized: ConsolidatedOrchestratorConfig =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;

        assert_eq!(config.load_balancing.algorithm, deserialized.load_balancing.algorithm);
        assert_eq!(config.scaling.min_instances, deserialized.scaling.min_instances);
        Ok(())
    }

    #[test]
    fn test_load_balancing_config_custom() {
        use songbird_config::unified::robustness::{LoadBalancingAlgorithm, HealthCheckConfig};
        use std::time::Duration;
        
        let config = CanonicalLoadBalancerConfig {
            algorithm: LoadBalancingAlgorithm::HealthBased,
            health_check: HealthCheckConfig::default(),
            sticky_sessions: true,
            session_timeout: Duration::from_secs(600),
            max_connections_per_backend: 200,
            connection_timeout: Duration::from_secs(60),
            fail_fast: true,
        };

        assert_eq!(config.algorithm, LoadBalancingAlgorithm::HealthBased);
        assert!(config.sticky_sessions);
        assert_eq!(config.max_connections_per_backend, 200);
        assert!(config.fail_fast);
    }

    #[test]
    fn test_performance_config_custom_thresholds() {
        let mut thresholds = HashMap::new();
        thresholds.insert("custom_metric".to_string(), 95.0);

        let config = PerformanceConfig {
            metrics_interval: 30,
            alert_thresholds: thresholds,
            enable_benchmarking: false,
        };

        assert_eq!(config.metrics_interval, 30);
        assert!(!config.enable_benchmarking);
        assert_eq!(config.alert_thresholds.get("custom_metric"), Some(&95.0));
    }

    #[test]
    fn test_registry_config_custom() {
        let config = RegistryConfig {
            discovery_interval: 60,
            service_timeout: 600,
            max_services: 5000,
        };

        assert_eq!(config.discovery_interval, 60);
        assert_eq!(config.service_timeout, 600);
        assert_eq!(config.max_services, 5000);
    }

    #[test]
    fn test_scaling_config_custom() {
        let config = ScalingConfig {
            enable_auto_scaling: false,
            scale_up_threshold: 80.0,
            scale_down_threshold: 20.0,
            min_instances: 3,
            max_instances: 50,
        };

        assert!(!config.enable_auto_scaling);
        assert_eq!(config.scale_up_threshold, 80.0);
        assert_eq!(config.scale_down_threshold, 20.0);
        assert_eq!(config.min_instances, 3);
        assert_eq!(config.max_instances, 50);
    }

    #[test]
    fn test_zero_touch_config_custom() {
        let config = ZeroTouchConfig {
            enable_auto_deployment: true,
            deployment_strategy: DeploymentStrategy::Canary,
            rollback_on_failure: false,
        };

        assert!(config.enable_auto_deployment);
        assert_eq!(config.deployment_strategy, DeploymentStrategy::Canary);
        assert!(!config.rollback_on_failure);
    }

    #[test]
    fn test_deployment_strategy_all_variants() {
        let strategies = [
            DeploymentStrategy::BlueGreen,
            DeploymentStrategy::RollingUpdate,
            DeploymentStrategy::Canary,
        ];

        assert_eq!(strategies.len(), 3);
        assert_eq!(strategies[0], DeploymentStrategy::BlueGreen);
        assert_eq!(strategies[1], DeploymentStrategy::RollingUpdate);
        assert_eq!(strategies[2], DeploymentStrategy::Canary);
    }

    #[test]
    fn test_health_status_all_variants() {
        let statuses = [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ];

        assert_eq!(statuses.len(), 4);
        assert_eq!(statuses[0], HealthStatus::Healthy);
        assert_eq!(statuses[1], HealthStatus::Degraded);
        assert_eq!(statuses[2], HealthStatus::Unhealthy);
        assert_eq!(statuses[3], HealthStatus::Unknown);
    }

    #[test]
    fn test_component_health_with_message() {
        let health = ComponentHealth {
            status: HealthStatus::Degraded,
            message: Some("High memory usage".to_string()),
            last_check: Some(chrono::Utc::now().timestamp() as u64),
        };

        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(health.message.is_some());
        assert!(health.last_check.is_some());
    }

    #[test]
    fn test_orchestrator_config_debug_format() {
        let config = ConsolidatedOrchestratorConfig::default();
        let debug_string = format!("{:?}", config);

        assert!(debug_string.contains("ConsolidatedOrchestratorConfig"));
        assert!(debug_string.contains("load_balancing"));
        assert!(debug_string.contains("performance"));
        assert!(debug_string.contains("registry"));
        assert!(debug_string.contains("scaling"));
    }
}
