// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "test assertions and harness ergonomics"
)]

use super::*;
use crate::SongbirdError;
use std::collections::HashMap;

#[test]
fn test_canonical_environment_config_default() {
    let config = CanonicalEnvironmentConfig::default();
    assert!(matches!(config.deployment_mode, DeploymentMode::Development));
    assert_eq!(config.environment_overrides.len(), 0);
}

#[test]
fn test_deployment_mode_from_string() {
    // Test standard modes
    assert!(matches!(DeploymentMode::from_env_string("production"), DeploymentMode::Production));
    assert!(matches!(DeploymentMode::from_env_string("staging"), DeploymentMode::Staging));
    assert!(matches!(DeploymentMode::from_env_string("testing"), DeploymentMode::Testing));
    assert!(matches!(DeploymentMode::from_env_string("development"), DeploymentMode::Development));
}

#[test]
fn test_deployment_mode_custom() {
    let mode = DeploymentMode::from_env_string("custom-env");
    assert!(matches!(mode, DeploymentMode::Custom(_)));
    if let DeploymentMode::Custom(name) = mode {
        assert_eq!(name, "custom-env");
    }
}

#[test]
fn test_deployment_mode_from_env_map_reads_songbird_env() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_ENV".to_string(), "staging".to_string());
    assert!(matches!(DeploymentMode::from_env_map(&env), DeploymentMode::Staging));

    let mut env2 = HashMap::new();
    env2.insert("SONGBIRD_ENV".to_string(), "production".to_string());
    assert!(matches!(DeploymentMode::from_env_map(&env2), DeploymentMode::Production));
}

#[test]
fn test_deployment_mode_from_env_map_defaults_when_missing() {
    let env = HashMap::new();
    assert!(matches!(DeploymentMode::from_env_map(&env), DeploymentMode::Development));
}

#[test]
fn test_resource_limits_invalid_numeric_falls_back_to_defaults() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_MAX_CONNECTIONS".to_string(), "not-a-number".to_string());
    let limits = ResourceLimits::from_env_map(&env);
    assert_eq!(limits.max_connections, 1000);
}

#[test]
fn test_network_binding_config_from_env_map_parses_bind() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_BIND_ADDRESS".to_string(), "10.0.0.1".to_string());
    env.insert("SONGBIRD_BIND_PORT".to_string(), "9090".to_string());
    let nb = NetworkBindingConfig::from_env_map(&env);
    assert_eq!(nb.bind_port, 9090);
    assert_eq!(nb.bind_address.to_string(), "10.0.0.1");
}

#[test]
fn test_resource_limits_default() {
    let env = HashMap::new();
    let limits = ResourceLimits::from_env_map(&env);
    assert_eq!(limits.max_connections, 1000);
    assert_eq!(limits.max_memory_mb, 2048);
    assert_eq!(limits.max_cpu_cores, 4);
    assert_eq!(limits.max_file_descriptors, 1024);
    assert_eq!(limits.max_threads, 100);
    assert_eq!(limits.disk_space_gb, 100);
}

#[test]
fn test_resource_limits_from_env() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_MAX_CONNECTIONS".to_string(), "5000".to_string());
    env.insert("SONGBIRD_MAX_MEMORY_MB".to_string(), "4096".to_string());
    let limits = ResourceLimits::from_env_map(&env);
    assert_eq!(limits.max_connections, 5000);
    assert_eq!(limits.max_memory_mb, 4096);
}

#[test]
fn test_memory_pool_config_default() {
    let config = MemoryPoolConfig::default();
    assert!(config.enabled);
    assert_eq!(config.initial_size_mb, 64);
    assert_eq!(config.max_size_mb, 512);
    assert_eq!(config.growth_increment_mb, 32);
}

#[test]
fn test_memory_pool_config_sizes_valid() {
    let config = MemoryPoolConfig::default();
    assert!(config.initial_size_mb <= config.max_size_mb);
    assert!(config.growth_increment_mb > 0);
}

#[test]
fn test_service_discovery_config_default() {
    let config = ServiceDiscoveryConfig::default();
    assert!(config.auto_discovery);
    assert!(config.refresh_interval.as_secs() > 0);
    assert!(config.discovery_timeout.as_secs() > 0);
    assert_eq!(config.fallback_endpoints.len(), 0);
}

#[test]
fn test_health_check_config_default() {
    let config = EnvironmentHealthCheckConfig::default();
    assert!(config.enabled);
    assert_eq!(config.interval.as_secs(), 30);
    assert_eq!(config.timeout.as_secs(), 5);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.endpoint_path, "/health");
}

#[test]
fn test_network_binding_config_default() {
    let config = NetworkBindingConfig::default();
    assert_eq!(config.bind_port, 8080);
    assert_eq!(config.interface_preferences.len(), 2);
}

#[test]
fn test_port_range_default() {
    let range = PortRange::default();
    assert_eq!(range.start, 8000);
    assert_eq!(range.end, 9000);
    assert!(range.end > range.start);
    assert_eq!(range.reserved.len(), 3);
}

#[test]
fn test_capability_endpoints_default() {
    let env = HashMap::new();
    let endpoints = CapabilityEndpoints::from_env_map(&env);
    assert!(endpoints.storage.is_none());
    assert!(endpoints.compute.is_none());
    assert!(endpoints.ai.is_none());
    assert!(endpoints.security.is_none());
    assert!(endpoints.orchestration.is_none());
    assert_eq!(endpoints.custom.len(), 0);
}

#[test]
fn test_capability_endpoints_from_env() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_STORAGE_ENDPOINT".to_string(), "http://storage:8001".to_string());
    env.insert("SONGBIRD_AI_ENDPOINT".to_string(), "http://ai:8002".to_string());
    let endpoints = CapabilityEndpoints::from_env_map(&env);
    assert_eq!(endpoints.storage, Some("http://storage:8001".to_string()));
    assert_eq!(endpoints.ai, Some("http://ai:8002".to_string()));
}

#[test]
fn test_legacy_compatibility_config_default() {
    let config = LegacyCompatibilityConfig::default();
    assert!(config.enable_legacy_primal_names);
    assert_eq!(config.legacy_endpoints.len(), 0);
}

#[test]
fn test_deprecation_warnings_config_default() {
    let config = DeprecationWarningsConfig::default();
    assert!(config.enabled);
    assert_eq!(config.log_level, "warn");
    assert_eq!(config.suppress_warnings.len(), 0);
}

#[test]
fn test_get_capability_endpoint_storage() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints.storage = Some("http://storage:8001".to_string());
    let endpoint = config.get_capability_endpoint("storage");
    assert_eq!(endpoint, Some("http://storage:8001".to_string()));
}

#[test]
fn test_get_capability_endpoint_compute() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints.compute = Some("http://compute:8002".to_string());
    let endpoint = config.get_capability_endpoint("compute");
    assert_eq!(endpoint, Some("http://compute:8002".to_string()));
}

#[test]
fn test_get_capability_endpoint_custom() {
    let mut config = CanonicalEnvironmentConfig::default();
    config
        .capability_endpoints
        .custom
        .insert("custom".to_string(), "http://custom:9000".to_string());
    let endpoint = config.get_capability_endpoint("custom");
    assert_eq!(endpoint, Some("http://custom:9000".to_string()));
}

#[test]
fn test_get_capability_endpoint_none() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints = CapabilityEndpoints::from_env_map(&HashMap::new());
    let endpoint = config.get_capability_endpoint("storage");
    assert_eq!(endpoint, None);
}

#[test]
fn test_get_all_endpoints_empty() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints = CapabilityEndpoints::from_env_map(&HashMap::new());
    let endpoints = config.get_all_endpoints();
    assert_eq!(endpoints.len(), 0);
}

#[test]
fn test_get_all_endpoints_with_values() {
    let mut config = CanonicalEnvironmentConfig::default();
    config.capability_endpoints.storage = Some("http://storage:8001".to_string());
    config.capability_endpoints.ai = Some("http://ai:8002".to_string());
    config
        .capability_endpoints
        .custom
        .insert("metrics".to_string(), "http://metrics:9090".to_string());

    let endpoints = config.get_all_endpoints();
    assert_eq!(endpoints.len(), 3);
    assert_eq!(endpoints.get("storage"), Some(&"http://storage:8001".to_string()));
    assert_eq!(endpoints.get("ai"), Some(&"http://ai:8002".to_string()));
    assert_eq!(endpoints.get("metrics"), Some(&"http://metrics:9090".to_string()));
}

#[test]
fn test_is_production() {
    let config = CanonicalEnvironmentConfig {
        deployment_mode: DeploymentMode::Production,
        ..Default::default()
    };
    assert!(config.is_production());
    assert!(!config.is_development());
}

#[test]
fn test_is_development() {
    let config = CanonicalEnvironmentConfig {
        deployment_mode: DeploymentMode::Development,
        ..Default::default()
    };
    assert!(config.is_development());
    assert!(!config.is_production());
}

#[test]
fn test_get_bind_address_production() {
    let config = CanonicalEnvironmentConfig {
        deployment_mode: DeploymentMode::Production,
        ..Default::default()
    };
    let addr = config.get_bind_address();
    assert_eq!(addr, IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
}

#[test]
fn test_get_bind_address_development() {
    let config = CanonicalEnvironmentConfig {
        deployment_mode: DeploymentMode::Development,
        ..Default::default()
    };
    let addr = config.get_bind_address();
    assert_eq!(addr, IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
}

#[test]
fn test_staging_is_not_production_or_development_flags() {
    let config = CanonicalEnvironmentConfig {
        deployment_mode: DeploymentMode::Staging,
        ..Default::default()
    };
    assert!(!config.is_production());
    assert!(!config.is_development());
    assert_eq!(config.get_bind_address(), IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
}

#[test]
fn test_serialization_canonical_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = CanonicalEnvironmentConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalEnvironmentConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;
    assert!(matches!(deserialized.deployment_mode, DeploymentMode::Development));
    Ok(())
}

#[test]
fn test_port_range_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let range = PortRange::default();
    let json = serde_json::to_string(&range)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: PortRange =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(deserialized.start, range.start);
    assert_eq!(deserialized.end, range.end);
    Ok(())
}
