// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for network connection limits configuration

use songbird_config::canonical::network::limits::*;

#[test]
fn test_connection_limits_default() {
    let limits = ConnectionLimits::default();
    assert_eq!(limits.max_connections_per_host, 10);
    assert_eq!(limits.max_total_connections, 100);
    assert_eq!(limits.max_retries, 3);
    assert_eq!(limits.pool_idle_timeout_secs, 300);
}

#[test]
fn test_connection_limits_clone() {
    let limits = ConnectionLimits::default();
    let cloned = limits.clone();
    assert_eq!(limits.max_connections_per_host, cloned.max_connections_per_host);
    assert_eq!(limits.max_total_connections, cloned.max_total_connections);
}

#[test]
fn test_connection_limits_debug() {
    let limits = ConnectionLimits::default();
    let debug_str = format!("{:?}", limits);
    assert!(debug_str.contains("ConnectionLimits"));
    assert!(debug_str.contains("max_connections_per_host"));
}

#[test]
fn test_connection_limits_serialization() {
    let limits = ConnectionLimits::default();
    let json = serde_json::to_string(&limits).expect("Serialization should succeed");
    assert!(json.contains("max_connections_per_host"));
    assert!(json.contains("max_total_connections"));
}

#[test]
fn test_connection_limits_deserialization() {
    let json = r#"{"max_connections_per_host":20,"max_total_connections":200,"max_retries":5,"pool_idle_timeout_secs":600}"#;
    let limits: ConnectionLimits =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(limits.max_connections_per_host, 20);
    assert_eq!(limits.max_total_connections, 200);
    assert_eq!(limits.max_retries, 5);
    assert_eq!(limits.pool_idle_timeout_secs, 600);
}

#[test]
fn test_load_balancing_config_default() {
    let config = LoadBalancingConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.strategy, "round_robin");
    assert_eq!(config.health_check_interval_secs, 30);
    assert!(config.backends.is_empty());
}

#[test]
fn test_load_balancing_config_clone() {
    let config = LoadBalancingConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.strategy, cloned.strategy);
}

#[test]
fn test_load_balancing_config_debug() {
    let config = LoadBalancingConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("LoadBalancingConfig"));
}

#[test]
fn test_load_balancing_config_serialization() {
    let config = LoadBalancingConfig {
        enabled: true,
        strategy: "least_connections".to_string(),
        health_check_interval_secs: 60,
        backends: vec!["server1:8080".to_string(), "server2:8080".to_string()],
    };
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("least_connections"));
    assert!(json.contains("server1:8080"));
}

#[test]
fn test_load_balancing_config_deserialization() {
    let json = r#"{"enabled":true,"strategy":"random","health_check_interval_secs":15,"backends":["host1","host2"]}"#;
    let config: LoadBalancingConfig =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert!(config.enabled);
    assert_eq!(config.strategy, "random");
    assert_eq!(config.backends.len(), 2);
}

#[test]
fn test_rate_limiting_config_default() {
    let config = RateLimitingConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.requests_per_second, 100);
    assert_eq!(config.burst_size, 200);
}

#[test]
fn test_rate_limiting_config_clone() {
    let config = RateLimitingConfig::default();
    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.requests_per_second, cloned.requests_per_second);
}

#[test]
fn test_rate_limiting_config_debug() {
    let config = RateLimitingConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("RateLimitingConfig"));
}

#[test]
fn test_rate_limiting_config_serialization() {
    let config = RateLimitingConfig {
        enabled: true,
        requests_per_second: 1000,
        burst_size: 500,
    };
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("1000"));
    assert!(json.contains("500"));
}

#[test]
fn test_rate_limiting_config_deserialization() {
    let json = r#"{"enabled":true,"requests_per_second":50,"burst_size":100}"#;
    let config: RateLimitingConfig =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert!(config.enabled);
    assert_eq!(config.requests_per_second, 50);
    assert_eq!(config.burst_size, 100);
}

#[test]
fn test_connection_pool_config_default() {
    let config = ConnectionPoolConfig::default();
    assert_eq!(config.max_size, 100);
    assert_eq!(config.min_idle, 10);
    assert_eq!(config.max_lifetime_secs, 1800);
    assert_eq!(config.idle_timeout_secs, 600);
    assert_eq!(config.connection_timeout_secs, 30);
}

#[test]
fn test_connection_pool_config_clone() {
    let config = ConnectionPoolConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_size, cloned.max_size);
    assert_eq!(config.min_idle, cloned.min_idle);
}

#[test]
fn test_connection_pool_config_debug() {
    let config = ConnectionPoolConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ConnectionPoolConfig"));
    assert!(debug_str.contains("max_size"));
}

#[test]
fn test_connection_pool_config_serialization() {
    let config = ConnectionPoolConfig::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    assert!(json.contains("max_size"));
    assert!(json.contains("min_idle"));
}

#[test]
fn test_connection_pool_config_deserialization() {
    let json = r#"{"max_size":50,"min_idle":5,"max_lifetime_secs":900,"idle_timeout_secs":300,"connection_timeout_secs":15}"#;
    let config: ConnectionPoolConfig =
        serde_json::from_str(json).expect("Deserialization should succeed");
    assert_eq!(config.max_size, 50);
    assert_eq!(config.min_idle, 5);
    assert_eq!(config.max_lifetime_secs, 900);
    assert_eq!(config.idle_timeout_secs, 300);
    assert_eq!(config.connection_timeout_secs, 15);
}
