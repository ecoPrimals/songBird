// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for Network core functionality
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_network_federation::{NetworkConfig, NetworkManager};
use songbird_types::SongbirdError;
use std::net::IpAddr;

#[test]
fn test_network_config_default() {
    let config = NetworkConfig::default();

    // Verify default configuration
    assert!(config.gaming.enabled, "Gaming should be enabled by default");
    assert_eq!(config.interface.port, 8080, "Default port should be 8080");
}

#[test]
fn test_network_config_clone() {
    let config = NetworkConfig::default();
    let cloned = config.clone();

    assert_eq!(config.interface.bind_address, cloned.interface.bind_address);
    assert_eq!(config.gaming.enabled, cloned.gaming.enabled);
}

#[test]
fn test_network_config_custom_bind_address() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = NetworkConfig::default();
    config.interface.bind_address = "192.168.1.100"
        .parse::<IpAddr>()
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {e}")))?;

    assert!(config.interface.bind_address.to_string().contains("192.168.1.100"));
    Ok(())
}

#[test]
fn test_network_config_gaming_enabled() {
    let mut config = NetworkConfig::default();
    config.gaming.enabled = true;

    assert!(config.gaming.enabled);
}

#[test]
fn test_network_config_ipv6_bind() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = NetworkConfig::default();
    config.interface.bind_address = "::1"
        .parse::<IpAddr>()
        .map_err(|e| SongbirdError::configuration(format!("Test operation failed: {e}")))?;

    assert!(config.interface.bind_address.to_string().contains("::1"));
    Ok(())
}

#[test]
fn test_network_manager_new() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);

    assert!(std::mem::size_of_val(&manager) > 0);
}

#[tokio::test]
async fn test_network_manager_initialize_without_gaming() {
    let mut config = NetworkConfig::default();
    config.gaming.enabled = false;

    let mut manager = NetworkManager::new(config);
    let result = manager.initialize().await;

    assert!(result.is_ok(), "Initialize should succeed without gaming");
}

#[test]
fn test_multiple_network_managers() {
    let config1 = NetworkConfig::default();
    let config2 = NetworkConfig::default();

    let _manager1 = NetworkManager::new(config1);
    let _manager2 = NetworkManager::new(config2);

    // Both should be created successfully
}

#[test]
fn test_network_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;

    assert!(!json.is_empty());
    Ok(())
}

#[test]
fn test_network_config_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let config = NetworkConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {e}")))?;
    let deserialized: NetworkConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {e}")))?;

    assert_eq!(config.interface.bind_address, deserialized.interface.bind_address);
    Ok(())
}

#[test]
fn test_network_config_debug_format() {
    let config = NetworkConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("NetworkConfig"));
}

#[test]
fn test_network_config_with_different_ports() {
    let mut config = NetworkConfig::default();

    // Test various port configurations
    let ports = vec![8080, 9000, 3000, 8443];
    for port in ports {
        config.interface.port = port;
        assert_eq!(config.interface.port, port);
    }
}

#[test]
fn test_network_config_consistency() {
    let config1 = NetworkConfig::default();
    let config2 = NetworkConfig::default();

    // Default configs should be consistent
    assert_eq!(config1.interface.bind_address, config2.interface.bind_address);
    assert_eq!(config1.gaming.enabled, config2.gaming.enabled);
}

#[test]
fn test_network_manager_size() {
    let config = NetworkConfig::default();
    let manager = NetworkManager::new(config);
    let size = std::mem::size_of_val(&manager);

    // Manager should have reasonable size
    assert!(size > 0);
    assert!(size < 10_000); // Reasonable upper bound
}

#[test]
fn test_config_independence() {
    let mut config1 = NetworkConfig::default();
    let mut config2 = NetworkConfig::default();

    config1.interface.port = 8080;
    config2.interface.port = 9000;

    assert_ne!(config1.interface.port, config2.interface.port);
}
