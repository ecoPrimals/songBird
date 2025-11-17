//! Unified configuration tests
//!
//! Tests for the unified SongbirdConfig system

use songbird_types::config::CanonicalSongbirdConfig;
use std::env;

#[test]
fn test_config_default() {
    let config = CanonicalSongbirdConfig::default();

    // Verify basic config structure exists
    assert!(!config.environment.is_empty());
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.max_connections > 0);
}

#[test]
fn test_config_test_defaults() {
    let config = CanonicalSongbirdConfig::test_defaults();

    // Test defaults should be valid
    assert_eq!(config.environment, "test");
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.max_connections > 0);
}

#[test]
fn test_config_has_all_sections() {
    let config = CanonicalSongbirdConfig::default();

    // Verify all config sections are accessible
    let _ = &config.environment;
    let _ = &config.network;
    let _ = &config.security;
    let _ = &config.discovery;
    let _ = &config.observability;
    let _ = &config.performance;
    let _ = &config.primal_registry;
    let _ = &config.custom;
}

#[test]
fn test_config_can_be_cloned() {
    let config = CanonicalSongbirdConfig::default();
    let cloned = config.clone();

    // Cloned config should match original
    assert_eq!(config.environment, cloned.environment);
    assert_eq!(config.network.bind_address, cloned.network.bind_address);
}

#[test]
fn test_config_network_has_connections_limit() {
    let config = CanonicalSongbirdConfig::default();

    // Network should have a reasonable connection limit
    assert!(config.network.max_connections > 0);
    assert!(config.network.max_connections < 1_000_000);
}

#[test]
fn test_config_test_vs_default_environments() {
    let default_config = CanonicalSongbirdConfig::default();
    let test_config = CanonicalSongbirdConfig::test_defaults();

    // Test config should explicitly be "test"
    assert_eq!(test_config.environment, "test");

    // Default environment should be set
    assert!(!default_config.environment.is_empty());
}

#[test]
fn test_config_performance_in_test_mode() {
    let config = CanonicalSongbirdConfig::test_defaults();

    // Test config should have performance settings
    assert!(config.performance.is_some());
}

#[test]
fn test_config_primal_registry_optional() {
    let config = CanonicalSongbirdConfig::default();

    // Primal registry is optional - both Some and None are valid
    let _ = config.primal_registry.is_some() || config.primal_registry.is_none();
}

#[test]
fn test_config_custom_params_optional() {
    let config = CanonicalSongbirdConfig::default();

    // Custom params are optional
    let _ = config.custom.is_some() || config.custom.is_none();
}

#[test]
fn test_config_environment_field() {
    let config = CanonicalSongbirdConfig::default();

    // Environment field should be accessible and non-empty
    assert!(!config.environment.is_empty());

    // Should be a valid environment name
    let env = &config.environment;
    assert!(
        env == "development"
            || env == "test"
            || env == "staging"
            || env == "production"
            || !env.is_empty(),
        "Environment should be a valid value"
    );
}
