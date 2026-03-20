// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::must_use_candidate
)]

//! Federation configuration tests

#![allow(clippy::unwrap_used)]
use songbird_network_federation::FederationConfig;

#[test]
fn test_default_federation_config() {
    let config = FederationConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.node_timeout_secs, 60);
}

#[test]
fn test_federation_config_creation() {
    let config = FederationConfig {
        discovery_mode: None,
        rendezvous_url: None,
        enabled: true,
        bootstrap_address: None,
        self_registration: None,
        heartbeat_interval_secs: 30,
        node_timeout_secs: 120,
    };
    assert!(config.enabled);
    assert_eq!(config.node_timeout_secs, 120);
}

#[test]
fn test_federation_config_clone() {
    let config1 = FederationConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.enabled, config2.enabled);
    assert_eq!(config1.heartbeat_interval_secs, config2.heartbeat_interval_secs);
    assert_eq!(config1.node_timeout_secs, config2.node_timeout_secs);
}

#[test]
fn test_federation_config_debug() {
    let config = FederationConfig::default();
    let debug_output = format!("{:?}", config);
    assert!(debug_output.contains("FederationConfig"));
}

#[test]
fn test_federation_enabled() {
    let config = FederationConfig {
        enabled: true,
        ..Default::default()
    };
    assert!(config.enabled);
}

#[test]
fn test_federation_disabled() {
    let config = FederationConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(!config.enabled);
}

#[test]
fn test_replica_limits() {
    let config = FederationConfig::default();
    assert!(i64::try_from(config.heartbeat_interval_secs).unwrap() < config.node_timeout_secs);
}

#[test]
fn test_health_check_interval() {
    let config = FederationConfig::default();
    assert!(config.heartbeat_interval_secs > 0);
}

#[test]
fn test_node_timeout() {
    let config = FederationConfig {
        node_timeout_secs: 300,
        ..Default::default()
    };
    assert_eq!(config.node_timeout_secs, 300);
}

#[test]
fn test_sync_interval() {
    let config = FederationConfig::default();
    assert!(config.heartbeat_interval_secs > 0);
}

#[test]
fn test_serialization() {
    let config = FederationConfig::default();
    let json = serde_json::to_string(&config).expect("test precondition");
    assert!(json.contains("enabled"));
}

#[test]
fn test_deserialization() {
    let json = r#"{
        "enabled": true,
        "bootstrap_address": null,
        "self_registration": null,
        "heartbeat_interval_secs": 30,
        "node_timeout_secs": 120
    }"#;
    let config: FederationConfig = serde_json::from_str(json).expect("should parse valid input");
    assert!(config.enabled);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.node_timeout_secs, 120);
}

#[test]
fn test_high_availability_config() {
    let config = FederationConfig {
        bootstrap_address: Some("http://ha-node-1:8080".to_string()),
        node_timeout_secs: 60,
        ..Default::default()
    };
    assert!(config.bootstrap_address.is_some());
}

#[test]
fn test_minimal_config() {
    let config = FederationConfig::default();
    assert!(!config.enabled);
}

#[test]
fn test_config_validation_relationships() {
    let config = FederationConfig::default();
    // Verify logical relationships
    assert!(i64::try_from(config.heartbeat_interval_secs).unwrap() <= config.node_timeout_secs);
}

#[test]
fn test_full_config_roundtrip() {
    let original = FederationConfig {
        enabled: true,
        node_timeout_secs: 180,
        ..Default::default()
    };
    let json = serde_json::to_string(&original).expect("test precondition");
    let deserialized: FederationConfig =
        serde_json::from_str(&json).expect("should parse valid input");
    assert_eq!(original.enabled, deserialized.enabled);
    assert_eq!(original.heartbeat_interval_secs, deserialized.heartbeat_interval_secs);
    assert_eq!(original.node_timeout_secs, deserialized.node_timeout_secs);
}

#[test]
fn test_varying_intervals() {
    for interval in [15, 30, 60, 120, 300] {
        let config = FederationConfig {
            heartbeat_interval_secs: interval,
            ..Default::default()
        };
        assert_eq!(config.heartbeat_interval_secs, interval);
    }
}

#[test]
fn test_varying_timeouts() {
    for timeout in [60, 120, 300, 600] {
        let config = FederationConfig {
            node_timeout_secs: timeout,
            ..Default::default()
        };
        assert_eq!(config.node_timeout_secs, timeout);
    }
}
