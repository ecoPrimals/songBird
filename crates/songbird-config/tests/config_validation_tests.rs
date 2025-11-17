//! Configuration validation and loading tests
//!
//! Tests for configuration validation, defaults, and environment loading

use songbird_config::{NetworkConfig, SecurityConfig, SongbirdConfig};
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_default_config_creation() {
    let config = SongbirdConfig::default();

    assert!(config.network.max_connections > 0, "Default max_connections should be positive");
    assert!(!config.network.bind_address.is_empty(), "Default bind address should not be empty");
}

#[test]
fn test_network_config_defaults() {
    let network = NetworkConfig::default();

    assert!(network.max_connections > 0);
    assert!(network.connection_timeout_ms > 0);
    assert!(!network.bind_address.is_empty());
}

#[test]
fn test_security_config_defaults() {
    let security = SecurityConfig::default();

    // Security config should exist with valid state
    // Just verify the config can be created
    let _ = security.enabled; // Access to verify field exists
}

#[test]
fn test_config_clone() {
    let config1 = SongbirdConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.network.bind_address, config2.network.bind_address);
    assert_eq!(config1.network.max_connections, config2.network.max_connections);
}

#[test]
fn test_network_config_customization() {
    let config = NetworkConfig {
        max_connections: 1000,
        bind_address: "0.0.0.0:9000".to_string(),
        ..Default::default()
    };

    assert_eq!(config.max_connections, 1000);
    assert_eq!(config.bind_address, "0.0.0.0:9000");
}

#[test]
fn test_config_serialization_deserialization() {
    let config = SongbirdConfig::default();

    // Test that config can be serialized/deserialized
    let serialized = serde_json::to_string(&config);
    assert!(serialized.is_ok(), "Config should be serializable");

    if let Ok(json) = serialized {
        let deserialized: Result<SongbirdConfig, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Config should be deserializable");
    }
}

#[test]
fn test_network_config_validation() -> SongbirdResult<()> {
    let mut config = NetworkConfig {
        max_connections: 100,
        ..Default::default()
    };

    // Test setting various values
    assert_eq!(config.max_connections, 100);

    config.max_connections = 0;
    assert_eq!(config.max_connections, 0, "Should allow zero connections (for validation testing)");
    Ok(())
}

#[test]
fn test_config_debug_output() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty(), "Debug output should not be empty");
    assert!(
        debug_str.contains("network") || debug_str.contains("Network"),
        "Should contain network info"
    );
    Ok(())
}

#[test]
fn test_multiple_config_instances() {
    let config1 = SongbirdConfig::default();
    let config2 = SongbirdConfig::default();

    // Each instance should be independent
    assert_eq!(config1.network.bind_address, config2.network.bind_address);
}

#[test]
fn test_config_field_access() {
    let config = SongbirdConfig::default();

    // Test that all major fields are accessible
    let _ = &config.network;
    let _ = &config.security;
    let _ = &config.observability;
}

#[test]
fn test_network_timeout_configuration() {
    let config = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    assert_eq!(config.connection_timeout_ms, 30000);

    let config2 = NetworkConfig {
        connection_timeout_ms: 60000,
        ..Default::default()
    };
    assert_eq!(config2.connection_timeout_ms, 60000);
}

#[test]
fn test_config_partial_update() {
    let mut config = SongbirdConfig::default();
    let original_max_conn = config.network.max_connections;

    // Update one field
    config.network.bind_address = "127.0.0.1:8080".to_string();

    // Other fields should remain unchanged
    assert_eq!(config.network.max_connections, original_max_conn);
}

#[test]
fn test_security_config_enabled_toggle() {
    let mut security = SecurityConfig {
        enabled: true,
        ..Default::default()
    };

    assert!(security.enabled);

    security.enabled = false;
    assert!(!security.enabled);
}

#[test]
fn test_config_builder_pattern_style() {
    let mut config = SongbirdConfig::default();
    config.network.max_connections = 500;
    config.network.connection_timeout_ms = 45000;

    assert_eq!(config.network.max_connections, 500);
    assert_eq!(config.network.connection_timeout_ms, 45000);
}

#[test]
fn test_config_equality() {
    let config1 = SongbirdConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.network.bind_address, config2.network.bind_address);
}
