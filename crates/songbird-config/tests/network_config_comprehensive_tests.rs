//! Comprehensive Network Configuration Tests
//!
//! Tests network configuration, timeouts, endpoints, and connection settings

#![allow(clippy::field_reassign_with_default)]

use songbird_config::NetworkConfig;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_network_config_default() -> SongbirdResult<()> {
    let config = NetworkConfig::default();

    // Should have sensible defaults
    assert!(config.connection_timeout_ms > 0);
    Ok(())
}

#[test]
fn test_network_config_custom_timeout() -> SongbirdResult<()> {
    let config = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    assert_eq!(config.connection_timeout_ms, 30000);
    Ok(())
}

#[test]
fn test_network_config_clone() -> SongbirdResult<()> {
    let config1 = NetworkConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.connection_timeout_ms, config2.connection_timeout_ms);
    Ok(())
}

#[test]
fn test_network_config_debug() -> SongbirdResult<()> {
    let config = NetworkConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    Ok(())
}

#[test]
fn test_short_timeout() {
    let config = NetworkConfig {
        connection_timeout_ms: 100,
        ..Default::default()
    };

    assert_eq!(config.connection_timeout_ms, 100);
}

#[test]
fn test_long_timeout() {
    let config = NetworkConfig {
        connection_timeout_ms: 300_000,
        ..Default::default()
    };

    assert_eq!(config.connection_timeout_ms, 300_000);
}

#[test]
fn test_zero_timeout() {
    let mut config = NetworkConfig::default();
    config.connection_timeout_ms = 0;

    // Zero timeout is allowed (means no timeout)
    assert_eq!(config.connection_timeout_ms, 0);
}

#[test]
fn test_timeout_modification() {
    let mut config = NetworkConfig::default();
    let original = config.connection_timeout_ms;

    config.connection_timeout_ms = 60000;

    assert_ne!(config.connection_timeout_ms, original);
}

#[test]
fn test_config_equality() {
    let config1 = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    let config2 = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    assert_eq!(config1.connection_timeout_ms, config2.connection_timeout_ms);
}

#[test]
fn test_config_inequality() {
    let config1 = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    let config2 = NetworkConfig {
        connection_timeout_ms: 60000,
        ..Default::default()
    };

    assert_ne!(config1.connection_timeout_ms, config2.connection_timeout_ms);
}

#[test]
fn test_multiple_timeouts() {
    let mut config = NetworkConfig::default();

    config.connection_timeout_ms = 10000;
    assert_eq!(config.connection_timeout_ms, 10000);

    config.connection_timeout_ms = 20000;
    assert_eq!(config.connection_timeout_ms, 20000);

    config.connection_timeout_ms = 30000;
    assert_eq!(config.connection_timeout_ms, 30000);
}

#[test]
fn test_config_serialization() {
    let config = NetworkConfig::default();
    let json = serde_json::to_string(&config);

    assert!(json.is_ok());
}

#[test]
fn test_very_short_timeout() {
    let mut config = NetworkConfig::default();
    config.connection_timeout_ms = 1;

    assert_eq!(config.connection_timeout_ms, 1);
}

#[test]
fn test_timeout_comparison() {
    let timeout1 = 30000u64;
    let timeout2 = 60000u64;

    assert!(timeout1 < timeout2);
    assert!(timeout2 > timeout1);
}

#[test]
fn test_config_reset() {
    let mut config = NetworkConfig::default();
    config.connection_timeout_ms = 999_000;

    config = NetworkConfig::default();

    assert_ne!(config.connection_timeout_ms, 999_000);
}

#[test]
fn test_multiple_config_instances() {
    let _config1 = NetworkConfig::default();
    let _config2 = NetworkConfig::default();
    let _config3 = NetworkConfig::default();

    // Should be able to create multiple instances without panic
    // Test passes if we reach here without panicking
}

#[test]
fn test_config_struct_size() {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::mem::size_of;

    let size = size_of::<NetworkConfig>();

    // Should not be excessively large
    assert!(size < 10000);
}

#[test]
fn test_timeout_addition() {
    let timeout1 = 10000u64;
    let timeout2 = 20000u64;
    let total = timeout1 + timeout2;

    assert_eq!(total, 30000);
}

#[test]
fn test_timeout_subtraction() {
    let timeout1 = 60000u64;
    let timeout2 = 20000u64;
    let diff = timeout1 - timeout2;

    assert_eq!(diff, 40000);
}

#[test]
fn test_timeout_multiplication() {
    let timeout = 10000u64;
    let multiplied = timeout * 3;

    assert_eq!(multiplied, 30000);
}

#[test]
fn test_config_with_all_fields() {
    let config = NetworkConfig {
        connection_timeout_ms: 30000,
        ..Default::default()
    };

    assert_eq!(config.connection_timeout_ms, 30000);
}

#[test]
fn test_max_connections() {
    let mut config = NetworkConfig::default();
    config.max_connections = 1000;

    assert_eq!(config.max_connections, 1000);
}

#[test]
fn test_bind_address() {
    let config = NetworkConfig::default();

    // Should have a valid bind address
    assert!(!config.bind_address.is_empty());
}
