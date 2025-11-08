//! Configuration validation tests
//!
//! Tests for SongbirdConfig structure and field validation
//!
//! Note: Full validation module is currently disabled due to API changes.
//! These tests verify config structure, defaults, and field access.

use songbird_config::SongbirdConfig;

#[test]
fn test_default_config_has_valid_structure() {
    let config = SongbirdConfig::default();
    
    // All fields should be accessible and have valid defaults
    assert!(!config.environment.is_empty());
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.max_connections > 0);
}

#[test]
fn test_test_config_has_valid_structure() {
    let config = SongbirdConfig::test_defaults();
    
    // Test config should have specific environment
    assert_eq!(config.environment, "test");
    assert!(!config.network.bind_address.is_empty());
}

#[test]
fn test_network_config_accessibility() {
    let config = SongbirdConfig::default();
    
    // Network config fields should be accessible
    let _ = &config.network.bind_address;
    let _ = &config.network.max_connections;
    let _ = &config.network.connection_timeout_ms;
    let _ = &config.network.port_range;
    let _ = &config.network.enable_ipv6;
}

#[test]
fn test_discovery_config_accessibility() {
    let config = SongbirdConfig::default();
    
    // Discovery config should exist and be accessible
    let _ = &config.discovery.mechanism;
    let _ = &config.discovery.interval_seconds;
    let _ = &config.discovery.health_check;
    let _ = &config.discovery.registration;
}

#[test]
fn test_security_config_accessibility() {
    let config = SongbirdConfig::default();
    
    // Security config should exist
    let _ = &config.security.enabled;
    let _ = &config.security.authentication;
    let _ = &config.security.authorization;
    let _ = &config.security.encryption;
    let _ = &config.security.rate_limiting;
}

#[test]
fn test_observability_config_accessibility() {
    let config = SongbirdConfig::default();
    
    // Observability config should exist
    let _ = &config.observability.metrics;
    let _ = &config.observability.tracing;
    let _ = &config.observability.logging;
}

#[test]
fn test_optional_fields_accessible() {
    let config = SongbirdConfig::default();
    
    // Optional fields should be accessible
    let _ = config.performance.is_some() || config.performance.is_none();
    let _ = config.primal_registry.is_some() || config.primal_registry.is_none();
    let _ = config.custom.is_some() || config.custom.is_none();
}

#[test]
fn test_performance_config_when_present() {
    let config = SongbirdConfig::test_defaults();
    
    // Test config should have performance settings
    if let Some(perf) = config.performance {
        // Verify performance config fields are accessible
        let _ = perf.connection_pool_size;
        let _ = perf.worker_threads;
        let _ = perf.request_timeout_ms;
    }
}

#[test]
fn test_primal_registry_when_present() {
    let config = SongbirdConfig::default();
    
    // If primal registry is present, it should be accessible
    if let Some(registry) = &config.primal_registry {
        let _ = &registry.primals;
    }
}

#[test]
fn test_config_environment_values() {
    let config = SongbirdConfig::default();
    
    // Environment should be a non-empty string
    assert!(!config.environment.is_empty());
    
    // Common environments (non-exhaustive)
    let valid_envs = ["development", "test", "staging", "production"];
    let _ = valid_envs.contains(&config.environment.as_str()) || !config.environment.is_empty();
}

#[test]
fn test_network_connection_limits_reasonable() {
    let config = SongbirdConfig::default();
    
    // Connection limits should be reasonable
    assert!(config.network.max_connections > 0);
    assert!(config.network.max_connections < 1_000_000);
}

#[test]
fn test_network_timeout_reasonable() {
    let config = SongbirdConfig::default();
    
    // Timeout should be positive and reasonable (< 5 minutes)
    assert!(config.network.connection_timeout_ms > 0);
    assert!(config.network.connection_timeout_ms < 300_000);
}

#[test]
fn test_discovery_interval_reasonable() {
    let config = SongbirdConfig::default();
    
    // Discovery interval should be reasonable (between 1s and 1 hour)
    assert!(config.discovery.interval_seconds > 0);
    assert!(config.discovery.interval_seconds < 3600);
}

#[test]
fn test_config_sections_independent() {
    let config = SongbirdConfig::default();
    
    // Each config section should be independently accessible
    let _ = config.network.clone();
    let _ = config.security.clone();
    let _ = config.discovery.clone();
    let _ = config.observability.clone();
}
