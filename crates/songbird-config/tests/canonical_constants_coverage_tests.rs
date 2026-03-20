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

//! Coverage tests for `songbird_config::canonical::constants`
//!
//! Tests the environment-aware configuration functions that provide
//! zero-hardcoding defaults for network, port, timeout, and resource config.

use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Helper to safely set/unset env vars for a test
struct ScopedEnv {
    vars: Vec<(String, Option<String>)>,
}

impl ScopedEnv {
    fn new() -> Self {
        Self {
            vars: Vec::new(),
        }
    }

    fn set(&mut self, key: &str, value: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        songbird_process_env::set_var(key, value);
        self
    }

    fn remove(&mut self, key: &str) -> &mut Self {
        let old = std::env::var(key).ok();
        self.vars.push((key.to_string(), old));
        songbird_process_env::remove_var(key);
        self
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        for (key, old) in self.vars.drain(..).rev() {
            match old {
                Some(val) => songbird_process_env::set_var(&key, &val),
                None => songbird_process_env::remove_var(&key),
            }
        }
    }
}

// ==================== BIND ADDRESS TESTS ====================

#[test]
fn test_bind_address_default_is_localhost() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BIND_ADDRESS");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.remove("CONTAINER");
    env.remove("SONGBIRD_ENV");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "127.0.0.1", "Default should be localhost");
}

#[test]
fn test_bind_address_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_BIND_ADDRESS", "10.0.0.5");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "10.0.0.5");
}

#[test]
fn test_bind_address_invalid_env_falls_through() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_BIND_ADDRESS", "not-an-ip");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.remove("CONTAINER");
    env.remove("SONGBIRD_ENV");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "127.0.0.1", "Invalid IP should fall through to default");
}

#[test]
fn test_bind_address_kubernetes_uses_wildcard() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BIND_ADDRESS");
    env.set("KUBERNETES_SERVICE_HOST", "10.96.0.1");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_bind_address_container_uses_wildcard() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BIND_ADDRESS");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.set("CONTAINER", "docker");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_bind_address_production_uses_wildcard() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BIND_ADDRESS");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.remove("CONTAINER");
    env.set("SONGBIRD_ENV", "production");

    let addr = songbird_config::canonical::constants::get_bind_address();
    assert_eq!(addr, "0.0.0.0");
}

#[test]
fn test_canonical_bind_address_is_alias() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BIND_ADDRESS");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.remove("CONTAINER");
    env.remove("SONGBIRD_ENV");

    let addr1 = songbird_config::canonical::constants::get_bind_address();
    let addr2 = songbird_config::canonical::constants::get_canonical_bind_address();
    let addr3 = songbird_config::canonical::constants::get_default_bind_address();
    let addr4 = songbird_config::canonical::constants::default_bind_address();
    assert_eq!(addr1, addr2);
    assert_eq!(addr1, addr3);
    assert_eq!(addr1, addr4);
}

// ==================== PORT CONFIGURATION TESTS ====================

#[test]
fn test_port_range_start_default() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_PORT_START");
    env.remove("SONGBIRD_ALLOW_PRIVILEGED_PORTS");

    let port = songbird_config::canonical::constants::get_port_range_start();
    assert!(port >= 8000, "Default port should be >= 8000, got {port}");
    assert!(port < 9000, "Default port should be < 9000, got {port}");
}

#[test]
fn test_port_range_start_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_PORT_START", "9500");

    let port = songbird_config::canonical::constants::get_port_range_start();
    assert_eq!(port, 9500);
}

#[test]
fn test_port_range_end_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_PORT_END", "10000");

    let port = songbird_config::canonical::constants::get_port_range_end();
    assert_eq!(port, 10000);
}

#[test]
fn test_port_range_end_calculated() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_PORT_END");

    let start = songbird_config::canonical::constants::get_port_range_start();
    let end = songbird_config::canonical::constants::get_port_range_end();
    assert!(end > start, "End ({end}) should be > start ({start})");
}

#[test]
fn test_dashboard_port_production() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_DASHBOARD_PORT");
    env.set("SONGBIRD_ENV", "production");

    let port = songbird_config::canonical::constants::get_dashboard_port();
    assert_eq!(port, 3000);
}

#[test]
fn test_dashboard_port_staging() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_DASHBOARD_PORT");
    env.set("SONGBIRD_ENV", "staging");

    let port = songbird_config::canonical::constants::get_dashboard_port();
    assert_eq!(port, 3001);
}

#[test]
fn test_dashboard_port_development() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_DASHBOARD_PORT");
    env.remove("SONGBIRD_ENV");

    let port = songbird_config::canonical::constants::get_dashboard_port();
    assert_eq!(port, 8083);
}

#[test]
fn test_dashboard_port_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_DASHBOARD_PORT", "4000");

    let port = songbird_config::canonical::constants::get_dashboard_port();
    assert_eq!(port, 4000);
}

#[test]
fn test_default_discovery_port() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_DISCOVERY_PORT");

    let port = songbird_config::canonical::constants::default_discovery_port();
    assert_eq!(port, 5678);
}

#[test]
fn test_discovery_port_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_DISCOVERY_PORT", "6789");

    let port = songbird_config::canonical::constants::default_discovery_port();
    assert_eq!(port, 6789);
}

// ==================== TIMEOUT TESTS ====================

#[test]
fn test_connection_timeout_production() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env.set("SONGBIRD_ENV", "production");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 30000);
}

#[test]
fn test_connection_timeout_staging() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env.set("SONGBIRD_ENV", "staging");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 45000);
}

#[test]
fn test_connection_timeout_development() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env.set("SONGBIRD_ENV", "development");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 60000);
}

#[test]
fn test_connection_timeout_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_CONNECTION_TIMEOUT_MS", "5000");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 5000);
}

#[test]
fn test_connection_timeout_cloud_detection() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env.remove("SONGBIRD_ENV");
    env.set("KUBERNETES_SERVICE_HOST", "10.96.0.1");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 15000, "Cloud environment should use 15s timeout");
}

#[test]
fn test_connection_timeout_aws_detection() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env.remove("SONGBIRD_ENV");
    env.remove("KUBERNETES_SERVICE_HOST");
    env.set("AWS_EXECUTION_ENV", "AWS_ECS_FARGATE");

    let timeout = songbird_config::canonical::constants::get_connection_timeout_ms();
    assert_eq!(timeout, 15000);
}

// ==================== RESOURCE MANAGEMENT TESTS ====================

#[test]
fn test_max_connections_production() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_MAX_CONNECTIONS");
    env.set("SONGBIRD_ENV", "production");

    let max = songbird_config::canonical::constants::get_max_connections();
    assert_eq!(max, 10000);
}

#[test]
fn test_max_connections_testing() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_MAX_CONNECTIONS");
    env.set("SONGBIRD_ENV", "testing");

    let max = songbird_config::canonical::constants::get_max_connections();
    assert_eq!(max, 1000);
}

#[test]
fn test_max_connections_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_MAX_CONNECTIONS", "500");

    let max = songbird_config::canonical::constants::get_max_connections();
    assert_eq!(max, 500);
}

#[test]
fn test_worker_threads_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_WORKER_THREADS", "8");

    let threads = songbird_config::canonical::constants::get_worker_threads();
    assert_eq!(threads, 8);
}

#[test]
fn test_worker_threads_default_uses_cpu_count() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_WORKER_THREADS");

    let threads = songbird_config::canonical::constants::get_worker_threads();
    assert!(threads >= 1, "Must have at least 1 thread");
}

#[test]
fn test_buffer_pool_size_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_BUFFER_POOL_SIZE", "3000");

    let size = songbird_config::canonical::constants::get_buffer_pool_size();
    assert_eq!(size, 3000);
}

#[test]
fn test_buffer_pool_size_production() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BUFFER_POOL_SIZE");
    env.remove("MEMORY_LIMIT");
    env.set("SONGBIRD_ENV", "production");

    let size = songbird_config::canonical::constants::get_buffer_pool_size();
    assert_eq!(size, 10000);
}

#[test]
fn test_buffer_pool_with_memory_limit() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BUFFER_POOL_SIZE");
    env.set("SONGBIRD_ENV", "production");
    env.set("MEMORY_LIMIT", "512"); // 512 MB

    let size = songbird_config::canonical::constants::get_buffer_pool_size();
    assert!(size > 0, "Should have non-zero buffer pool");
}

#[test]
fn test_batch_size_default() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BATCH_SIZE");
    env.remove("MEMORY_LIMIT");

    let size = songbird_config::canonical::constants::get_batch_size();
    // Calculated as cpu_count * 1000, clamped to [100, 5000]
    assert!(size >= 100, "Batch size should be >= 100, got {size}");
    assert!(size <= 5000, "Batch size should be <= 5000, got {size}");
}

#[test]
fn test_batch_size_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_BATCH_SIZE", "50");

    let size = songbird_config::canonical::constants::get_batch_size();
    assert_eq!(size, 50);
}

#[test]
fn test_enable_zero_copy() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_ENABLE_ZERO_COPY");

    let enabled = songbird_config::canonical::constants::enable_zero_copy();
    assert!(enabled, "Zero copy should be enabled by default");
}

#[test]
fn test_disable_zero_copy() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ENABLE_ZERO_COPY", "false");

    let enabled = songbird_config::canonical::constants::enable_zero_copy();
    assert!(!enabled, "Zero copy should be disabled when env says false");
}

// ==================== PRIMAL ENDPOINT TESTS ====================

#[test]
fn test_get_primal_endpoint_uses_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("BEARDOG_ENDPOINT", "https://custom.beardog:9000");

    let endpoint = songbird_config::canonical::constants::get_primal_endpoint("beardog");
    assert_eq!(endpoint, "https://custom.beardog:9000");
}

#[test]
fn test_get_primal_endpoint_calculated() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("MYPRIMAL_ENDPOINT");
    env.remove("SONGBIRD_ENV");

    let endpoint = songbird_config::canonical::constants::get_primal_endpoint("myprimal");
    // Should be http://[bind_address]:[calculated_port]
    assert!(endpoint.starts_with("http"), "Endpoint should start with http: {endpoint}");
}

#[test]
fn test_get_primal_endpoint_deterministic() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("TESTPRIMAL_ENDPOINT");
    env.remove("SONGBIRD_ENV");

    let e1 = songbird_config::canonical::constants::get_primal_endpoint("testprimal");
    let e2 = songbird_config::canonical::constants::get_primal_endpoint("testprimal");
    assert_eq!(e1, e2, "Same primal name should produce same endpoint");
}

#[test]
fn test_get_primal_endpoint_different_primals() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("ALPHA_ENDPOINT");
    env.remove("BETA_ENDPOINT");
    env.remove("SONGBIRD_ENV");

    let e1 = songbird_config::canonical::constants::get_primal_endpoint("alpha");
    let e2 = songbird_config::canonical::constants::get_primal_endpoint("beta");
    // Different primals should get different ports (with very high probability)
    assert_ne!(e1, e2, "Different primal names should produce different endpoints");
}

// ==================== ENVIRONMENT-SPECIFIC PORT OFFSETS ====================

#[test]
fn test_port_range_production_offset() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_PORT_START");
    env.remove("SONGBIRD_ALLOW_PRIVILEGED_PORTS");
    env.set("SONGBIRD_ENV", "production");

    let port = songbird_config::canonical::constants::get_port_range_start();
    assert_eq!(port, 8000, "Production should use port 8000");
}

#[test]
fn test_port_range_staging_offset() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_PORT_START");
    env.remove("SONGBIRD_ALLOW_PRIVILEGED_PORTS");
    env.set("SONGBIRD_ENV", "staging");

    let port = songbird_config::canonical::constants::get_port_range_start();
    assert_eq!(port, 8100, "Staging should use port 8100");
}

#[test]
fn test_port_range_testing_offset() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_PORT_START");
    env.remove("SONGBIRD_ALLOW_PRIVILEGED_PORTS");
    env.set("SONGBIRD_ENV", "testing");

    let port = songbird_config::canonical::constants::get_port_range_start();
    assert_eq!(port, 8200, "Testing should use port 8200");
}

// ==================== COMMON PRIMAL PORTS ====================

#[test]
fn test_common_primal_ports_base() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_COMMON_PORTS");
    // Clear any SONGBIRD_ENABLE_* vars
    for (key, _) in std::env::vars() {
        if key.starts_with("SONGBIRD_ENABLE_") {
            env.remove(&key);
        }
    }

    let ports = songbird_config::canonical::constants::get_common_primal_ports();
    assert!(!ports.is_empty(), "Should have at least the base port");
}

#[test]
fn test_common_primal_ports_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_COMMON_PORTS", "8080,8081,8082");

    let ports = songbird_config::canonical::constants::get_common_primal_ports();
    assert_eq!(ports, vec![8080, 8081, 8082]);
}

#[test]
fn test_common_primal_ports_with_enabled_primals() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_COMMON_PORTS");
    env.set("SONGBIRD_ENABLE_MYPRIM", "true");

    let ports = songbird_config::canonical::constants::get_common_primal_ports();
    assert!(ports.len() >= 2, "Should have base + enabled primal, got {:?}", ports);
}

// ==================== CONSTANT VALUES ====================

#[test]
fn test_cache_ttl_constant() {
    let ttl = songbird_config::canonical::constants::DEFAULT_CACHE_TTL;
    assert_eq!(ttl, std::time::Duration::from_secs(300));
}

#[test]
fn test_evaluation_timeout_constant() {
    let timeout = songbird_config::canonical::constants::DEFAULT_EVALUATION_TIMEOUT;
    assert_eq!(timeout, std::time::Duration::from_secs(30));
}

#[test]
fn test_metrics_interval_constant() {
    let interval = songbird_config::canonical::constants::DEFAULT_METRICS_INTERVAL;
    assert_eq!(interval, std::time::Duration::from_secs(60));
}
