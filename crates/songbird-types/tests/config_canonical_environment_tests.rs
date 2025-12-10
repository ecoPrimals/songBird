//! Tests for Canonical Environment Configuration
//!
//! Comprehensive tests for environment configuration structures
//!
//! **Concurrency**: All tests are concurrent-safe using `TestEnv` for isolation.
//! No `#[serial]` annotations needed!

use songbird_test_utils::test_bind_address;
use songbird_test_utils::test_discovery_port;
use songbird_test_utils::test_federation_port;
use songbird_test_utils::test_health_port;
use songbird_test_utils::test_orchestrator_port;
use songbird_test_utils::TestEnv;
use songbird_types::config::environment::{
    CanonicalEnvironmentConfig, CapabilityEndpoints, DeploymentMode, DeprecationWarningsConfig,
    EnvironmentHealthCheckConfig, LegacyCompatibilityConfig, MemoryPoolConfig,
    NetworkBindingConfig, PortRange, ResourceLimits, ServiceDiscoveryConfig,
};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_canonical_environment_config_default() {
    // Uses default environment (development mode)
    let config = CanonicalEnvironmentConfig::default();

    assert!(matches!(config.deployment_mode, DeploymentMode::Development));
    assert_eq!(config.resource_limits.max_connections, 1000);
    assert!(config.environment_overrides.is_empty());
}

#[test]
fn test_deployment_mode_variants() {
    let dev = DeploymentMode::Development;
    let test = DeploymentMode::Testing;
    let staging = DeploymentMode::Staging;
    let prod = DeploymentMode::Production;
    let custom = DeploymentMode::Custom("MyMode".to_string());

    assert!(matches!(dev, DeploymentMode::Development));
    assert!(matches!(test, DeploymentMode::Testing));
    assert!(matches!(staging, DeploymentMode::Staging));
    assert!(matches!(prod, DeploymentMode::Production));
    assert!(matches!(custom, DeploymentMode::Custom(_)));
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deployment_mode_default_development() {
    let env = TestEnv::development();
    let mode = DeploymentMode::from_env_map(env.as_map());

    assert!(matches!(mode, DeploymentMode::Development));
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deployment_mode_from_env_production() {
    let env = TestEnv::production();
    let mode = DeploymentMode::from_env_map(env.as_map());

    assert!(matches!(mode, DeploymentMode::Production));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deployment_mode_from_env_staging() {
    let env = TestEnv::staging();
    let mode = DeploymentMode::from_env_map(env.as_map());

    assert!(matches!(mode, DeploymentMode::Staging));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deployment_mode_from_env_testing() {
    let env = TestEnv::testing();
    let mode = DeploymentMode::from_env_map(env.as_map());

    assert!(matches!(mode, DeploymentMode::Testing));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deployment_mode_custom() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_ENV", "custom-env");

    let mode = DeploymentMode::from_env_map(env.as_map());

    if let DeploymentMode::Custom(name) = mode {
        assert_eq!(name, "custom-env");
    } else {
        panic!("Expected Custom variant");
    }
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_resource_limits_default() {
    let env = TestEnv::new(); // Empty environment
    let limits = ResourceLimits::from_env_map(env.as_map());

    assert_eq!(limits.max_connections, 1000);
    assert_eq!(limits.max_memory_mb, 2048);
    assert_eq!(limits.max_cpu_cores, 4);
    assert_eq!(limits.max_file_descriptors, 1024);
    assert_eq!(limits.max_threads, 100);
    assert_eq!(limits.disk_space_gb, 100);
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_resource_limits_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_MAX_CONNECTIONS", "5000");
    env.set("SONGBIRD_MAX_MEMORY_MB", "4096");
    env.set("SONGBIRD_MAX_CPU_CORES", "8");

    let limits = ResourceLimits::from_env_map(env.as_map());

    assert_eq!(limits.max_connections, 5000);
    assert_eq!(limits.max_memory_mb, 4096);
    assert_eq!(limits.max_cpu_cores, 8);
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_memory_pool_config_default() {
    let env = TestEnv::new(); // Empty environment
    let config = MemoryPoolConfig::from_env_map(env.as_map());

    assert!(config.enabled);
    assert_eq!(config.initial_size_mb, 64);
    assert_eq!(config.max_size_mb, 512);
    assert_eq!(config.growth_increment_mb, 32);
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_memory_pool_config_disabled() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_MEMORY_POOL_ENABLED", "false");

    let config = MemoryPoolConfig::from_env_map(env.as_map());
    assert!(!config.enabled);
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_service_discovery_config_default() {
    let env = TestEnv::new(); // Empty environment
    let config = ServiceDiscoveryConfig::from_env_map(env.as_map());

    assert!(config.auto_discovery);
    assert_eq!(config.refresh_interval, Duration::from_secs(30));
    assert_eq!(config.discovery_timeout, Duration::from_secs(10));
    assert!(config.fallback_endpoints.is_empty());
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_service_discovery_config_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_AUTO_DISCOVERY", "false");
    env.set("SONGBIRD_DISCOVERY_REFRESH_INTERVAL", "60");

    let config = ServiceDiscoveryConfig::from_env_map(env.as_map());

    assert!(!config.auto_discovery);
    assert_eq!(config.refresh_interval, Duration::from_secs(60));
    // No cleanup needed - env is local!
}

#[test]
fn test_environment_health_check_config_default() {
    let config = EnvironmentHealthCheckConfig::default();

    assert!(config.enabled);
    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.endpoint_path, "/health");
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_network_binding_config_default() {
    let env = TestEnv::new(); // Empty environment
    let config = NetworkBindingConfig::from_env_map(env.as_map());

    assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    assert_eq!(config.production_bind_address, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(config.bind_port, 8080);
    assert_eq!(config.interface_preferences.len(), 2);
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_network_binding_config_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_BIND_ADDRESS", &test_bind_address());
    env.set("SONGBIRD_BIND_PORT", "3000");

    let config = NetworkBindingConfig::from_env_map(env.as_map());

    assert_eq!(config.bind_address, IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(config.bind_port, 3000);
    // No cleanup needed - env is local!
}

#[test]
fn test_port_range_default() {
    let range = PortRange::default();

    assert_eq!(range.start, 8000);
    assert_eq!(range.end, 9000);
    assert_eq!(range.reserved.len(), 3);
    assert!(range.reserved.contains(&8080));
    assert!(range.start < range.end);
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_capability_endpoints_default() {
    let env = TestEnv::new(); // Empty environment
    let endpoints = CapabilityEndpoints::from_env_map(env.as_map());

    assert!(endpoints.storage.is_none());
    assert!(endpoints.compute.is_none());
    assert!(endpoints.ai.is_none());
    assert!(endpoints.security.is_none());
    assert!(endpoints.orchestration.is_none());
    assert!(endpoints.custom.is_empty());
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_capability_endpoints_from_env() {
    let mut env = TestEnv::new();
    env.set("SONGBIRD_STORAGE_ENDPOINT", &format!("http://storage:{}", test_orchestrator_port()));
    env.set("SONGBIRD_COMPUTE_ENDPOINT", &format!("http://compute:{}", test_discovery_port()));

    let endpoints = CapabilityEndpoints::from_env_map(env.as_map());

    assert_eq!(endpoints.storage, Some(format!("http://storage:{}", test_orchestrator_port())));
    assert_eq!(endpoints.compute, Some(format!("http://compute:{}", test_discovery_port())));
    // No cleanup needed - env is local!
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_legacy_compatibility_config_default() {
    let env = TestEnv::new(); // Empty environment
    let config = LegacyCompatibilityConfig::from_env_map(env.as_map());

    assert!(config.enable_legacy_primal_names);
    assert!(config.legacy_endpoints.is_empty());
}

#[test] // ✅ NO #[serial]! Fully concurrent!
fn test_deprecation_warnings_config_default() {
    let env = TestEnv::new(); // Empty environment
    let config = DeprecationWarningsConfig::from_env_map(env.as_map());

    assert!(config.enabled);
    assert_eq!(config.log_level, "warn");
    assert!(config.suppress_warnings.is_empty());
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_get_capability_endpoint() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints.storage =
        Some(format!("http://storage:{}", test_orchestrator_port()));
    config
        .capability_endpoints
        .custom
        .insert("custom".to_string(), "http://custom:9000".to_string());

    assert_eq!(
        config.get_capability_endpoint("storage"),
        Some(format!("http://storage:{}", test_orchestrator_port()))
    );
    assert_eq!(config.get_capability_endpoint("custom"), Some("http://custom:9000".to_string()));
    assert_eq!(config.get_capability_endpoint("nonexistent"), None);
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_get_all_endpoints() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints.storage =
        Some(format!("http://storage:{}", test_orchestrator_port()));
    config.capability_endpoints.compute = Some(format!("http://compute:{}", test_discovery_port()));

    let endpoints = config.get_all_endpoints();

    assert_eq!(endpoints.len(), 2);
    assert_eq!(
        endpoints.get("storage"),
        Some(&format!("http://storage:{}", test_orchestrator_port()))
    );
    assert_eq!(
        endpoints.get("compute"),
        Some(&format!("http://compute:{}", test_discovery_port()))
    );
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_is_production() {
    let mut config = CanonicalEnvironmentConfig::default();
    assert!(!config.is_production());

    config.deployment_mode = DeploymentMode::Production;
    assert!(config.is_production());
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_is_development() {
    let config = CanonicalEnvironmentConfig::default();
    assert!(config.is_development());
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_get_bind_address() {
    let mut config = CanonicalEnvironmentConfig::default();

    // Development mode should return localhost
    assert_eq!(config.get_bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));

    // Production mode should return unspecified (0.0.0.0)
    config.deployment_mode = DeploymentMode::Production;
    assert_eq!(config.get_bind_address(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_resource_limits_serialization() -> SongbirdResult<()> {
    let limits = ResourceLimits {
        max_connections: 5000,
        max_memory_mb: 4096,
        max_cpu_cores: 8,
        max_file_descriptors: 2048,
        max_threads: 200,
        disk_space_gb: 500,
        memory_pool: MemoryPoolConfig::default(),
    };

    let json = serde_json::to_string(&limits)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    let deserialized: ResourceLimits = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;

    assert_eq!(limits.max_connections, deserialized.max_connections);
    Ok(())
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_config_clone() {
    let config = CanonicalEnvironmentConfig::default();
    let cloned = config.clone();

    assert_eq!(config.resource_limits.max_connections, cloned.resource_limits.max_connections);
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_canonical_environment_config_debug() {
    let config = CanonicalEnvironmentConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("CanonicalEnvironmentConfig"));
}

#[test]
fn test_port_range_with_reservations() {
    let range = PortRange {
        start: 8000,
        end: 9000,
        reserved: vec![8080, 8443, 8888, 8080], // Duplicate for testing
    };

    assert_eq!(range.reserved.len(), 4);
    assert!(range.reserved.contains(&8080));
}

#[test]
fn test_service_discovery_with_fallbacks() {
    let mut config = ServiceDiscoveryConfig::default();
    config
        .fallback_endpoints
        .insert("service1".to_string(), format!("http://fallback:{}", test_orchestrator_port()));
    config
        .fallback_endpoints
        .insert("service2".to_string(), format!("http://fallback:{}", test_discovery_port()));

    assert_eq!(config.fallback_endpoints.len(), 2);
}

#[test] // ✅ NO #[serial]! Fully concurrent! (doesn't mutate global state)
fn test_environment_overrides() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.environment_overrides.insert("KEY1".to_string(), "value1".to_string());
    config.environment_overrides.insert("KEY2".to_string(), "value2".to_string());

    assert_eq!(config.environment_overrides.len(), 2);
    assert_eq!(config.environment_overrides.get("KEY1"), Some(&"value1".to_string()));
}

#[test]
fn test_deprecation_warnings_suppression() {
    let config = DeprecationWarningsConfig {
        enabled: true,
        log_level: "warn".to_string(),
        suppress_warnings: vec!["DEPRECATED_API_V1".to_string(), "OLD_ENDPOINT".to_string()],
    };

    assert_eq!(config.suppress_warnings.len(), 2);
}

#[test]
fn test_capability_endpoints_all_set() {
    let endpoints = CapabilityEndpoints {
        storage: Some(format!("http://storage:{}", test_orchestrator_port())),
        compute: Some(format!("http://compute:{}", test_discovery_port())),
        ai: Some(format!("http://ai:{}", test_health_port())),
        security: Some(format!("http://security:{}", test_federation_port())),
        orchestration: Some("http://orchestration:8084".to_string()),
        custom: HashMap::from([
            ("custom1".to_string(), "http://custom1:9000".to_string()),
            ("custom2".to_string(), "http://custom2:9001".to_string()),
        ]),
    };

    assert!(endpoints.storage.is_some());
    assert!(endpoints.compute.is_some());
    assert!(endpoints.ai.is_some());
    assert!(endpoints.security.is_some());
    assert!(endpoints.orchestration.is_some());
    assert_eq!(endpoints.custom.len(), 2);
}

#[test]
fn test_legacy_compatibility_with_mappings() {
    let config = LegacyCompatibilityConfig {
        enable_legacy_primal_names: true,
        legacy_endpoints: HashMap::from([
            ("old_api".to_string(), format!("http://new-api:{}", test_orchestrator_port())),
            (
                "deprecated_service".to_string(),
                format!("http://new-service:{}", test_discovery_port()),
            ),
        ]),
        deprecation_warnings: DeprecationWarningsConfig::default(),
    };

    assert_eq!(config.legacy_endpoints.len(), 2);
}
