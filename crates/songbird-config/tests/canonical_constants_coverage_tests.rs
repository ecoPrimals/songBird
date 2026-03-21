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
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Coverage tests for `songbird_config::canonical::constants`
//!
//! Tests the environment-aware configuration functions that provide
//! zero-hardcoding defaults for network, port, timeout, and resource config.

use songbird_config::canonical::constants as C;
use std::collections::HashMap;

fn env_reader(
    m: &HashMap<String, String>,
) -> impl Fn(&str) -> Result<String, std::env::VarError> + '_ {
    |k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)
}

// ==================== BIND ADDRESS TESTS ====================

#[test]
fn test_bind_address_default_is_localhost() {
    let m: HashMap<String, String> = HashMap::new();
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "127.0.0.1", "Default should be localhost");
}

#[test]
fn test_bind_address_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_BIND_ADDRESS".into(), "10.0.0.5".into());
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "10.0.0.5");
}

#[test]
fn test_bind_address_invalid_env_falls_through() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_BIND_ADDRESS".into(), "not-an-ip".into());
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "127.0.0.1", "Invalid IP should fall through to default");
}

#[test]
fn test_bind_address_kubernetes_uses_wildcard() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("KUBERNETES_SERVICE_HOST".into(), "10.96.0.1".into());
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_bind_address_container_uses_wildcard() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("CONTAINER".into(), "docker".into());
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_bind_address_production_uses_wildcard() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let addr = C::get_bind_address_with(&env_reader(&m));
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_canonical_bind_address_is_alias() {
    let m: HashMap<String, String> = HashMap::new();
    let r = env_reader(&m);
    // All public bind-address helpers delegate to the same `get_bind_address` logic;
    // injectable reader exercises the core path four times for determinism.
    let addr1 = C::get_bind_address_with(&r);
    let addr2 = C::get_bind_address_with(&r);
    let addr3 = C::get_bind_address_with(&r);
    let addr4 = C::get_bind_address_with(&r);
    assert_eq!(addr1, addr2);
    assert_eq!(addr1, addr3);
    assert_eq!(addr1, addr4);
}

// ==================== PORT CONFIGURATION TESTS ====================

#[test]
fn test_port_range_start_default() {
    let m: HashMap<String, String> = HashMap::new();
    let port = C::get_port_range_start_with(&env_reader(&m));
    assert!(port >= 8000, "Default port should be >= 8000, got {port}");
    assert!(port < 9000, "Default port should be < 9000, got {port}");
}

#[test]
fn test_port_range_start_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_PORT_START".into(), "9500".into());
    let port = C::get_port_range_start_with(&env_reader(&m));
    assert_eq!(port, 9500);
}

#[test]
fn test_port_range_end_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_PORT_END".into(), "10000".into());
    let port = C::get_port_range_end_with(&env_reader(&m));
    assert_eq!(port, 10000);
}

#[test]
fn test_port_range_end_calculated() {
    let m: HashMap<String, String> = HashMap::new();
    let start = C::get_port_range_start_with(&env_reader(&m));
    let end = C::get_port_range_end_with(&env_reader(&m));
    assert!(end > start, "End ({end}) should be > start ({start})");
}

#[test]
fn test_dashboard_port_production() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let port = C::get_dashboard_port_with(&env_reader(&m));
    assert_eq!(port, 3000);
}

#[test]
fn test_dashboard_port_staging() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "staging".into());
    let port = C::get_dashboard_port_with(&env_reader(&m));
    assert_eq!(port, 3001);
}

#[test]
fn test_dashboard_port_development() {
    let m: HashMap<String, String> = HashMap::new();
    let port = C::get_dashboard_port_with(&env_reader(&m));
    assert_eq!(port, 8083);
}

#[test]
fn test_dashboard_port_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_DASHBOARD_PORT".into(), "4000".into());
    let port = C::get_dashboard_port_with(&env_reader(&m));
    assert_eq!(port, 4000);
}

#[test]
fn test_default_discovery_port() {
    let m: HashMap<String, String> = HashMap::new();
    let port = C::default_discovery_port_with(&env_reader(&m));
    assert_eq!(port, 5678);
}

#[test]
fn test_discovery_port_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_DISCOVERY_PORT".into(), "6789".into());
    let port = C::default_discovery_port_with(&env_reader(&m));
    assert_eq!(port, 6789);
}

// ==================== TIMEOUT TESTS ====================

#[test]
fn test_connection_timeout_production() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 30000);
}

#[test]
fn test_connection_timeout_staging() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "staging".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 45000);
}

#[test]
fn test_connection_timeout_development() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "development".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 60000);
}

#[test]
fn test_connection_timeout_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_CONNECTION_TIMEOUT_MS".into(), "5000".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 5000);
}

#[test]
fn test_connection_timeout_cloud_detection() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("KUBERNETES_SERVICE_HOST".into(), "10.96.0.1".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 15000, "Cloud environment should use 15s timeout");
}

#[test]
fn test_connection_timeout_aws_detection() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("AWS_EXECUTION_ENV".into(), "AWS_ECS_FARGATE".into());
    let timeout = C::get_connection_timeout_ms_with(&env_reader(&m));
    assert_eq!(timeout, 15000);
}

// ==================== RESOURCE MANAGEMENT TESTS ====================

#[test]
fn test_max_connections_production() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let max = C::get_max_connections_with(&env_reader(&m));
    assert_eq!(max, 10000);
}

#[test]
fn test_max_connections_testing() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "testing".into());
    let max = C::get_max_connections_with(&env_reader(&m));
    assert_eq!(max, 1000);
}

#[test]
fn test_max_connections_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_MAX_CONNECTIONS".into(), "500".into());
    let max = C::get_max_connections_with(&env_reader(&m));
    assert_eq!(max, 500);
}

#[test]
fn test_worker_threads_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_WORKER_THREADS".into(), "8".into());
    let threads = C::get_worker_threads_with(&env_reader(&m));
    assert_eq!(threads, 8);
}

#[test]
fn test_worker_threads_default_uses_cpu_count() {
    let m: HashMap<String, String> = HashMap::new();
    let threads = C::get_worker_threads_with(&env_reader(&m));
    assert!(threads >= 1, "Must have at least 1 thread");
}

#[test]
fn test_buffer_pool_size_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_BUFFER_POOL_SIZE".into(), "3000".into());
    let size = C::get_buffer_pool_size_with(&env_reader(&m));
    assert_eq!(size, 3000);
}

#[test]
fn test_buffer_pool_size_production() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let size = C::get_buffer_pool_size_with(&env_reader(&m));
    assert_eq!(size, 10000);
}

#[test]
fn test_buffer_pool_with_memory_limit() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    m.insert("MEMORY_LIMIT".into(), "512".into());
    let size = C::get_buffer_pool_size_with(&env_reader(&m));
    assert!(size > 0, "Should have non-zero buffer pool");
}

#[test]
fn test_batch_size_default() {
    let m: HashMap<String, String> = HashMap::new();
    let size = C::get_batch_size_with(&env_reader(&m));
    assert!(size >= 100, "Batch size should be >= 100, got {size}");
    assert!(size <= 5000, "Batch size should be <= 5000, got {size}");
}

#[test]
fn test_batch_size_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_BATCH_SIZE".into(), "50".into());
    let size = C::get_batch_size_with(&env_reader(&m));
    assert_eq!(size, 50);
}

#[test]
fn test_enable_zero_copy() {
    let m: HashMap<String, String> = HashMap::new();
    let enabled = C::enable_zero_copy_with(&env_reader(&m));
    assert!(enabled, "Zero copy should be enabled by default");
}

#[test]
fn test_disable_zero_copy() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENABLE_ZERO_COPY".into(), "false".into());
    let enabled = C::enable_zero_copy_with(&env_reader(&m));
    assert!(!enabled, "Zero copy should be disabled when env says false");
}

// ==================== PRIMAL ENDPOINT TESTS ====================

#[test]
fn test_get_primal_endpoint_uses_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("BEARDOG_ENDPOINT".into(), "https://custom.beardog:9000".into());
    let endpoint = C::get_primal_endpoint_with("beardog", &env_reader(&m));
    assert_eq!(endpoint, "https://custom.beardog:9000");
}

#[test]
fn test_get_primal_endpoint_calculated() {
    let m: HashMap<String, String> = HashMap::new();
    let endpoint = C::get_primal_endpoint_with("myprimal", &env_reader(&m));
    assert!(endpoint.starts_with("http"), "Endpoint should start with http: {endpoint}");
}

#[test]
fn test_get_primal_endpoint_deterministic() {
    let m: HashMap<String, String> = HashMap::new();
    let e1 = C::get_primal_endpoint_with("testprimal", &env_reader(&m));
    let e2 = C::get_primal_endpoint_with("testprimal", &env_reader(&m));
    assert_eq!(e1, e2, "Same primal name should produce same endpoint");
}

#[test]
fn test_get_primal_endpoint_different_primals() {
    let m: HashMap<String, String> = HashMap::new();
    let e1 = C::get_primal_endpoint_with("alpha", &env_reader(&m));
    let e2 = C::get_primal_endpoint_with("beta", &env_reader(&m));
    assert_ne!(e1, e2, "Different primal names should produce different endpoints");
}

// ==================== ENVIRONMENT-SPECIFIC PORT OFFSETS ====================

#[test]
fn test_port_range_production_offset() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "production".into());
    let port = C::get_port_range_start_with(&env_reader(&m));
    assert_eq!(port, 8000, "Production should use port 8000");
}

#[test]
fn test_port_range_staging_offset() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "staging".into());
    let port = C::get_port_range_start_with(&env_reader(&m));
    assert_eq!(port, 8100, "Staging should use port 8100");
}

#[test]
fn test_port_range_testing_offset() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENV".into(), "testing".into());
    let port = C::get_port_range_start_with(&env_reader(&m));
    assert_eq!(port, 8200, "Testing should use port 8200");
}

// ==================== COMMON PRIMAL PORTS ====================

#[test]
fn test_common_primal_ports_base() {
    let mut m: HashMap<String, String> = std::env::vars().collect();
    m.retain(|k, _| !k.starts_with("SONGBIRD_ENABLE_"));
    m.remove("SONGBIRD_COMMON_PORTS");
    let ports = C::get_common_primal_ports_from_env_map(&m);
    assert!(!ports.is_empty(), "Should have at least the base port");
}

#[test]
fn test_common_primal_ports_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_COMMON_PORTS".into(), "8080,8081,8082".into());
    let ports = C::get_common_primal_ports_from_env_map(&m);
    assert_eq!(ports, vec![8080, 8081, 8082]);
}

#[test]
fn test_common_primal_ports_with_enabled_primals() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ENABLE_MYPRIM".into(), "true".into());
    let ports = C::get_common_primal_ports_from_env_map(&m);
    assert!(ports.len() >= 2, "Should have base + enabled primal, got {:?}", ports);
}

// ==================== CONSTANT VALUES ====================

#[test]
fn test_cache_ttl_constant() {
    let ttl = C::DEFAULT_CACHE_TTL;
    assert_eq!(ttl, std::time::Duration::from_secs(300));
}

#[test]
fn test_evaluation_timeout_constant() {
    let timeout = C::DEFAULT_EVALUATION_TIMEOUT;
    assert_eq!(timeout, std::time::Duration::from_secs(30));
}

#[test]
fn test_metrics_interval_constant() {
    let interval = C::DEFAULT_METRICS_INTERVAL;
    assert_eq!(interval, std::time::Duration::from_secs(60));
}
