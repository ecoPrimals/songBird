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
    clippy::unnecessary_literal_unwrap
)]

//! Tests for canonical performance configuration
//!
//! Comprehensive tests for `PerformanceConfig`, `CacheConfig`, `MetricsConfig`,
//! `ObjectPoolSizes`, and `BenchmarkConfig`.

use songbird_config::canonical::performance::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

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

// ==================== PERFORMANCE CONFIG TESTS ====================

#[test]
fn test_performance_config_default_values() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_THREAD_POOL_SIZE");
    env.remove("SONGBIRD_MAX_CONCURRENT_REQUESTS");
    env.remove("SONGBIRD_ZERO_COPY_ENABLED");

    let config = PerformanceConfig::default();
    assert!(config.thread_pool_size >= 1);
    assert_eq!(config.max_concurrent_requests, 1000);
    assert_eq!(config.request_buffer_size, 8192);
    assert!(!config.enable_zero_copy);
}

#[test]
fn test_performance_config_thread_pool_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_THREAD_POOL_SIZE", "8");

    let config = PerformanceConfig::default();
    assert_eq!(config.thread_pool_size, 8);
}

#[test]
fn test_performance_config_max_concurrent_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_MAX_CONCURRENT_REQUESTS", "5000");

    let config = PerformanceConfig::default();
    assert_eq!(config.max_concurrent_requests, 5000);
}

#[test]
fn test_performance_config_zero_copy_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_ZERO_COPY_ENABLED", "1");

    let config = PerformanceConfig::default();
    assert!(config.enable_zero_copy);
}

#[test]
fn test_performance_config_clone() {
    let config = PerformanceConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_concurrent_requests, cloned.max_concurrent_requests);
}

#[test]
fn test_performance_config_debug() {
    let config = PerformanceConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("PerformanceConfig"));
}

#[test]
fn test_performance_config_serialization() {
    let config = PerformanceConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("max_concurrent_requests"));

    let deserialized: PerformanceConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.max_concurrent_requests, deserialized.max_concurrent_requests);
}

// ==================== CACHE CONFIG TESTS ====================

#[test]
fn test_cache_config_default_values() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_CACHE_ENABLED");
    env.remove("SONGBIRD_CACHE_MAX_SIZE");
    env.remove("SONGBIRD_CACHE_TTL_SECS");

    let config = CacheConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_size, 1000);
    assert_eq!(config.ttl_secs, 300);
}

#[test]
fn test_cache_config_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_CACHE_ENABLED", "false");
    env.set("SONGBIRD_CACHE_MAX_SIZE", "5000");
    env.set("SONGBIRD_CACHE_TTL_SECS", "600");

    let config = CacheConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.max_size, 5000);
    assert_eq!(config.ttl_secs, 600);
}

#[test]
fn test_cache_config_clone() {
    let config = CacheConfig::default();
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_cache_config_equality() {
    let config1 = CacheConfig {
        enabled: true,
        max_size: 1000,
        ttl_secs: 300,
    };
    let config2 = CacheConfig {
        enabled: true,
        max_size: 1000,
        ttl_secs: 300,
    };
    assert_eq!(config1, config2);
}

#[test]
fn test_cache_config_debug() {
    let config = CacheConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("CacheConfig"));
}

// ==================== METRICS CONFIG TESTS ====================

#[test]
fn test_metrics_config_default_values() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_METRICS_ENABLED");
    env.remove("SONGBIRD_METRICS_INTERVAL_SECS");
    env.remove("SONGBIRD_PROMETHEUS_ENABLED");

    let config = MetricsConfig::default();
    assert!(config.enabled);
    assert_eq!(config.collection_interval_secs, 60);
    assert!(!config.export_prometheus);
}

#[test]
fn test_metrics_config_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_METRICS_ENABLED", "false");
    env.set("SONGBIRD_METRICS_INTERVAL_SECS", "30");
    env.set("SONGBIRD_PROMETHEUS_ENABLED", "1");

    let config = MetricsConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.collection_interval_secs, 30);
    assert!(config.export_prometheus);
}

#[test]
fn test_metrics_config_clone() {
    let config = MetricsConfig::default();
    let cloned = config.clone();
    assert_eq!(config, cloned);
}

#[test]
fn test_metrics_config_equality() {
    let config1 = MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        export_prometheus: false,
    };
    let config2 = MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        export_prometheus: false,
    };
    assert_eq!(config1, config2);
}

#[test]
fn test_metrics_config_debug() {
    let config = MetricsConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("MetricsConfig"));
}

// ==================== OBJECT POOL SIZES TESTS ====================

#[test]
fn test_object_pool_sizes_default_values() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_MESSAGE_POOL_SIZE");
    env.remove("SONGBIRD_BUFFER_POOL_SIZE");
    env.remove("SONGBIRD_CONNECTION_POOL_SIZE");

    let pools = ObjectPoolSizes::default();
    assert_eq!(pools.message, 1000);
    assert_eq!(pools.buffer, 500);
    assert_eq!(pools.connection, 100);
}

#[test]
fn test_object_pool_sizes_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_MESSAGE_POOL_SIZE", "2000");
    env.set("SONGBIRD_BUFFER_POOL_SIZE", "1000");
    env.set("SONGBIRD_CONNECTION_POOL_SIZE", "200");

    let pools = ObjectPoolSizes::default();
    assert_eq!(pools.message, 2000);
    assert_eq!(pools.buffer, 1000);
    assert_eq!(pools.connection, 200);
}

#[test]
fn test_object_pool_sizes_clone() {
    let pools = ObjectPoolSizes::default();
    let cloned = pools.clone();
    assert_eq!(pools, cloned);
}

#[test]
fn test_object_pool_sizes_equality() {
    let pools1 = ObjectPoolSizes {
        message: 1000,
        buffer: 500,
        connection: 100,
    };
    let pools2 = ObjectPoolSizes {
        message: 1000,
        buffer: 500,
        connection: 100,
    };
    assert_eq!(pools1, pools2);
}

#[test]
fn test_object_pool_sizes_debug() {
    let pools = ObjectPoolSizes::default();
    let debug = format!("{:?}", pools);
    assert!(debug.contains("ObjectPoolSizes"));
}

#[test]
fn test_object_pool_sizes_serialization() {
    let pools = ObjectPoolSizes::default();
    let json = serde_json::to_string(&pools).unwrap();
    let deserialized: ObjectPoolSizes = serde_json::from_str(&json).unwrap();
    assert_eq!(pools, deserialized);
}

// ==================== BENCHMARK CONFIG TESTS ====================

#[test]
fn test_benchmark_config_default_values() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_BENCHMARK_ENABLED");
    env.remove("SONGBIRD_BENCHMARK_DURATION_SECS");
    env.remove("SONGBIRD_BENCHMARK_CONCURRENT");
    env.remove("SONGBIRD_BENCHMARK_WARMUP_SECS");
    env.remove("SONGBIRD_BENCHMARK_OUTPUT");

    let config = BenchmarkConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.duration_secs, 60);
    assert_eq!(config.concurrent_requests, 100);
    assert_eq!(config.warmup_duration_secs, 10);
    assert_eq!(config.output_format, "json");
    assert_eq!(config.batch_test_size, 1000);
}

#[test]
fn test_benchmark_config_from_env() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.set("SONGBIRD_BENCHMARK_ENABLED", "1");
    env.set("SONGBIRD_BENCHMARK_DURATION_SECS", "120");
    env.set("SONGBIRD_BENCHMARK_CONCURRENT", "500");
    env.set("SONGBIRD_BENCHMARK_WARMUP_SECS", "15");
    env.set("SONGBIRD_BENCHMARK_OUTPUT", "csv");

    let config = BenchmarkConfig::default();
    assert!(config.enabled);
    assert_eq!(config.duration_secs, 120);
    assert_eq!(config.concurrent_requests, 500);
    assert_eq!(config.warmup_duration_secs, 15);
    assert_eq!(config.output_format, "csv");
}

#[test]
fn test_benchmark_config_clone() {
    let config = BenchmarkConfig::default();
    let cloned = config.clone();
    assert_eq!(config.duration_secs, cloned.duration_secs);
}

#[test]
fn test_benchmark_config_equality() {
    let config1 = BenchmarkConfig {
        enabled: true,
        duration_secs: 60,
        concurrent_requests: 100,
        warmup_duration_secs: 10,
        output_format: "json".to_string(),
        batch_test_size: 1000,
    };
    let config2 = BenchmarkConfig {
        enabled: true,
        duration_secs: 60,
        concurrent_requests: 100,
        warmup_duration_secs: 10,
        output_format: "json".to_string(),
        batch_test_size: 1000,
    };
    assert_eq!(config1, config2);
}

#[test]
fn test_benchmark_config_debug() {
    let config = BenchmarkConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("BenchmarkConfig"));
}

#[test]
fn test_benchmark_config_serialization() {
    let config = BenchmarkConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: BenchmarkConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

// ==================== TYPE ALIAS TESTS ====================

#[test]
fn test_unified_performance_config_alias() {
    let _g = lock_env();
    let mut env = ScopedEnv::new();
    env.remove("SONGBIRD_MAX_CONCURRENT_REQUESTS");

    // UnifiedPerformanceConfig should be identical to PerformanceConfig
    let config: UnifiedPerformanceConfig = PerformanceConfig::default();
    assert_eq!(config.max_concurrent_requests, 1000);
}
