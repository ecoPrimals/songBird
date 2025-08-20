//! Migration Module
//!
//! Provides backward compatibility aliases for configuration structs that have been
//! consolidated into the unified configuration system

/// Backward compatibility aliases for migrated configuration structs
pub mod backward_compat {
    // API Configuration Aliases
    pub use crate::unified::api::ConnectionConfig as ConnectionConfiguration;
    pub use crate::unified::api::HealthCheckConfiguration;
    pub use crate::unified::api::HealthMonitoringConfig;
    pub use crate::unified::api::MonitoringConfiguration;
    pub use crate::unified::api::PerformanceAnalysisConfig;
    pub use crate::unified::api::SessionConfig as SessionConfiguration;

    // Robustness Configuration Aliases
    pub use crate::unified::robustness::BulkheadConfig;
    pub use crate::unified::robustness::CircuitBreakerConfig;
    pub use crate::unified::robustness::LoadBalancerConfig;
    pub use crate::unified::robustness::RateLimitingConfig;
    pub use crate::unified::robustness::RetryConfig;
    pub use crate::unified::robustness::ZeroCostRouterConfig;

    // Core System Configuration Aliases
    pub use crate::unified::core::HealthCheckConfig;
    pub use crate::unified::core::HookSystemConfig;
    pub use crate::unified::core::ResourceManagementConfig;
    pub use crate::unified::core::ValidationConfig;

    // Network Configuration Aliases (already unified but for completeness)
    pub use crate::unified::network::UnifiedNetworkConfig as NetworkConfig;
    pub use crate::unified::network::UnifiedSslConfig as TlsConfig;

    // Performance Configuration Aliases
    pub use crate::unified::performance::CacheConfig;
    pub use crate::unified::performance::MetricsConfig;
    pub use crate::unified::performance::UnifiedPerformanceConfig as PerformanceConfig;

    // Security Configuration Aliases
    pub use crate::unified::security::AuthenticationConfig;
    pub use crate::unified::security::EncryptionConfig;
    pub use crate::unified::security::UniversalSecurityConfig;
    pub use crate::unified::security::UniversalSecurityConfig as SecurityConfig;

    // Universal Primal Configuration Aliases
    pub use crate::unified::primals::PrimalDiscoveryConfig as AdaptiveDiscoveryConfig;
    pub use crate::unified::primals::PrimalEndpointConfig as CapabilityOrchestratorConfig;
    pub use crate::unified::primals::PrimalRoutingConfig as RoutingConfig;
    pub use crate::unified::primals::UniversalPrimalsConfig as UniversalPrimalConfig;

    // Discovery Configuration Aliases
    pub use crate::unified::discovery::NetworkDiscoveryConfig;
    pub use crate::unified::discovery::ServiceDiscoveryConfig;
    pub use crate::unified::discovery::UnifiedDiscoveryConfig as DiscoveryConfig;

    // Federation Configuration Aliases
    pub use crate::unified::federation::ClusterConfig;
    pub use crate::unified::federation::NodeConfig;
    pub use crate::unified::federation::UnifiedFederationConfig as FederationConfig;

    // Observability Configuration Aliases
    pub use crate::unified::observability::DashboardConfig;
    pub use crate::unified::observability::LoggingConfig;
    pub use crate::unified::observability::TracingConfig;
    pub use crate::unified::observability::UnifiedObservabilityConfig as ObservabilityConfig;

    // CLI Configuration Aliases
    pub use crate::unified::cli::GamingCliConfig;
    pub use crate::unified::cli::UnifiedCliConfig as CliConfig;
}

/// Migration helper functions for converting from legacy configuration structs
pub mod migration_helpers {
    use crate::unified::SongbirdConfig;

    /// Create a SongbirdConfig with API-focused defaults
    #[must_use]
    pub fn create_api_focused_config() -> SongbirdConfig {
        let mut config = SongbirdConfig::default();

        // Enhanced API settings
        config.api.session.max_concurrent_sessions = 2000;
        config.api.connection.max_connections_per_client = 20;
        config.api.mesh.enable_mesh = true;

        // Enhanced robustness settings
        config.robustness.circuit_breaker.enabled = true;
        config.robustness.rate_limiting.max_requests_per_second = 2000;
        config.robustness.retry.max_attempts = 5;

        config
    }

    /// Create a SongbirdConfig with robustness-focused defaults
    #[must_use]
    pub fn create_robustness_focused_config() -> SongbirdConfig {
        let mut config = SongbirdConfig::default();

        // Enhanced robustness settings
        config.robustness.circuit_breaker.failure_threshold = 3;
        config.robustness.rate_limiting.enabled = true;
        config.robustness.bulkhead.max_concurrent_operations = 200;
        config.robustness.retry.backoff_multiplier = 1.5;

        // Enhanced monitoring
        config.api.mesh.health_monitoring.check_interval = std::time::Duration::from_secs(15);
        config.api.mesh.performance_analysis.enabled = true;

        config
    }

    /// Create a SongbirdConfig with performance-focused defaults
    #[must_use]
    pub fn create_performance_focused_config() -> SongbirdConfig {
        let mut config = SongbirdConfig::default();

        // Enhanced performance settings
        config.performance.cache.enabled = true;
        config.performance.cache.max_size = 100000;
        config.performance.metrics.enabled = true;

        // Enhanced API performance
        config.api.session.buffer_size = 16384;
        config.api.connection.pool_size = 200;

        // Enhanced robustness for performance
        config.robustness.load_balancer.algorithm =
            crate::unified::robustness::LoadBalancingAlgorithm::LeastConnections;
        config.robustness.zero_cost_router.route_cache_size = 20000;

        config
    }
}

/// Deprecation warnings for legacy configuration usage
pub mod deprecation_warnings {
    /// Issue a deprecation warning for legacy configuration usage
    pub fn warn_legacy_config_usage(config_name: &str) {
        eprintln!(
            "⚠️  DEPRECATION WARNING: {config_name} is deprecated. Please migrate to SongbirdConfig."
        );
        eprintln!(
            "   See migration guide: https://github.com/ecoPrimals/songbird/blob/main/migration-guides/CONFIG_MIGRATION_GUIDE.md"
        );
    }

    /// Issue a deprecation warning with migration suggestion
    pub fn warn_with_migration_path(legacy_name: &str, new_path: &str) {
        eprintln!("⚠️  DEPRECATION WARNING: {legacy_name} is deprecated.");
        eprintln!("   NEW: Use SongbirdConfig and access via: config.{new_path}");
    }
}

/// Re-export the main unified configuration for easy migration
pub use crate::unified::SongbirdConfig;
