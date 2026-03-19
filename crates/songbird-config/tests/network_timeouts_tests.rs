//! Tests for network timeout configurations
//!
//! Covers `NetworkTimeouts` and `TimeoutConfig` from canonical network module.

#![allow(clippy::unwrap_used)]

use songbird_config::canonical::network::{NetworkTimeouts, TimeoutConfig};
use std::time::Duration;

// ==================== NETWORK TIMEOUTS TESTS ====================

#[test]
fn test_network_timeouts_default() {
    let timeouts = NetworkTimeouts::default();
    assert_eq!(timeouts.connection, Duration::from_secs(10));
    assert_eq!(timeouts.request, Duration::from_secs(60));
    assert_eq!(timeouts.health_check, Duration::from_secs(5));
    assert_eq!(timeouts.default, Duration::from_secs(30));
}

#[test]
fn test_network_timeouts_clone() {
    let timeouts = NetworkTimeouts::default();
    let cloned = timeouts.clone();
    assert_eq!(timeouts.connection, cloned.connection);
    assert_eq!(timeouts.request, cloned.request);
}

#[test]
fn test_network_timeouts_debug() {
    let timeouts = NetworkTimeouts::default();
    let debug = format!("{:?}", timeouts);
    assert!(debug.contains("NetworkTimeouts"));
}

#[test]
fn test_network_timeouts_serialization() {
    let timeouts = NetworkTimeouts::default();
    let json = serde_json::to_string(&timeouts).unwrap();
    assert!(json.contains("connection"));
    assert!(json.contains("request"));

    let deserialized: NetworkTimeouts = serde_json::from_str(&json).unwrap();
    assert_eq!(timeouts.connection, deserialized.connection);
}

#[test]
fn test_network_timeouts_custom_values() {
    let timeouts = NetworkTimeouts {
        connection: Duration::from_secs(5),
        request: Duration::from_secs(120),
        health_check: Duration::from_secs(2),
        default: Duration::from_secs(15),
    };

    assert_eq!(timeouts.connection, Duration::from_secs(5));
    assert_eq!(timeouts.request, Duration::from_secs(120));
}

// ==================== TIMEOUT CONFIG TESTS ====================

#[test]
fn test_timeout_config_default() {
    let config = TimeoutConfig::default();
    assert_eq!(config.default_timeout_secs, 30);
    assert_eq!(config.connection_timeout_secs, 10);
    assert_eq!(config.health_check_timeout_secs, 5);
    assert_eq!(config.registration_timeout_secs, 15);
    assert_eq!(config.discovery_timeout_secs, 30);
}

#[test]
fn test_timeout_config_clone() {
    let config = TimeoutConfig::default();
    let cloned = config.clone();
    assert_eq!(config.default_timeout_secs, cloned.default_timeout_secs);
    assert_eq!(config.connection_timeout_secs, cloned.connection_timeout_secs);
}

#[test]
fn test_timeout_config_debug() {
    let config = TimeoutConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("TimeoutConfig"));
}

#[test]
fn test_timeout_config_serialization() {
    let config = TimeoutConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("default_timeout_secs"));
    assert!(json.contains("connection_timeout_secs"));

    let deserialized: TimeoutConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.default_timeout_secs, deserialized.default_timeout_secs);
}

#[test]
fn test_timeout_config_custom_values() {
    let config = TimeoutConfig {
        default_timeout_secs: 60,
        connection_timeout_secs: 20,
        health_check_timeout_secs: 10,
        registration_timeout_secs: 30,
        discovery_timeout_secs: 45,
    };

    assert_eq!(config.default_timeout_secs, 60);
    assert_eq!(config.connection_timeout_secs, 20);
    assert_eq!(config.discovery_timeout_secs, 45);
}

#[test]
fn test_timeout_config_serde_snake_case() {
    // Verify that snake_case serialization is used
    let config = TimeoutConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("default_timeout_secs"));
    assert!(json.contains("connection_timeout_secs"));
    // No camelCase should be present
    assert!(!json.contains("defaultTimeoutSecs"));
}
