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

//! Tests for canonical performance configuration
//!
//! Comprehensive tests for `PerformanceConfig`, `CacheConfig`, `MetricsConfig`,
//! `ObjectPoolSizes`, and `BenchmarkConfig`.

use songbird_config::canonical::performance::*;
use std::collections::HashMap;

fn env_reader(
    m: &HashMap<String, String>,
) -> impl Fn(&str) -> Result<String, std::env::VarError> + '_ {
    |k| m.get(k).cloned().ok_or(std::env::VarError::NotPresent)
}

// ==================== PERFORMANCE CONFIG TESTS ====================

#[test]
fn test_performance_config_default_values() {
    let m: HashMap<String, String> = HashMap::new();
    let config = PerformanceConfig::from_env_reader(env_reader(&m));
    assert!(config.thread_pool_size >= 1);
    assert_eq!(config.max_concurrent_requests, 1000);
    assert_eq!(config.request_buffer_size, 8192);
    assert!(!config.enable_zero_copy);
}

#[test]
fn test_performance_config_thread_pool_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_THREAD_POOL_SIZE".into(), "8".into());
    let config = PerformanceConfig::from_env_reader(env_reader(&m));
    assert_eq!(config.thread_pool_size, 8);
}

#[test]
fn test_performance_config_max_concurrent_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_MAX_CONCURRENT_REQUESTS".into(), "5000".into());
    let config = PerformanceConfig::from_env_reader(env_reader(&m));
    assert_eq!(config.max_concurrent_requests, 5000);
}

#[test]
fn test_performance_config_zero_copy_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_ZERO_COPY_ENABLED".into(), "1".into());
    let config = PerformanceConfig::from_env_reader(env_reader(&m));
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
    let m: HashMap<String, String> = HashMap::new();
    let config = CacheConfig::from_env_reader(env_reader(&m));
    assert!(config.enabled);
    assert_eq!(config.max_size, 1000);
    assert_eq!(config.ttl_secs, 300);
}

#[test]
fn test_cache_config_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_CACHE_ENABLED".into(), "false".into());
    m.insert("SONGBIRD_CACHE_MAX_SIZE".into(), "5000".into());
    m.insert("SONGBIRD_CACHE_TTL_SECS".into(), "600".into());
    let config = CacheConfig::from_env_reader(env_reader(&m));
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
    let m: HashMap<String, String> = HashMap::new();
    let config = MetricsConfig::from_env_reader(env_reader(&m));
    assert!(config.enabled);
    assert_eq!(config.collection_interval_secs, 60);
    assert!(!config.export_prometheus);
}

#[test]
fn test_metrics_config_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_METRICS_ENABLED".into(), "false".into());
    m.insert("SONGBIRD_METRICS_INTERVAL_SECS".into(), "30".into());
    m.insert("SONGBIRD_PROMETHEUS_ENABLED".into(), "1".into());
    let config = MetricsConfig::from_env_reader(env_reader(&m));
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
    let m: HashMap<String, String> = HashMap::new();
    let pools = ObjectPoolSizes::from_env_reader(env_reader(&m));
    assert_eq!(pools.message, 1000);
    assert_eq!(pools.buffer, 500);
    assert_eq!(pools.connection, 100);
}

#[test]
fn test_object_pool_sizes_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_MESSAGE_POOL_SIZE".into(), "2000".into());
    m.insert("SONGBIRD_BUFFER_POOL_SIZE".into(), "1000".into());
    m.insert("SONGBIRD_CONNECTION_POOL_SIZE".into(), "200".into());
    let pools = ObjectPoolSizes::from_env_reader(env_reader(&m));
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
    let m: HashMap<String, String> = HashMap::new();
    let config = BenchmarkConfig::from_env_reader(env_reader(&m));
    assert!(!config.enabled);
    assert_eq!(config.duration_secs, 60);
    assert_eq!(config.concurrent_requests, 100);
    assert_eq!(config.warmup_duration_secs, 10);
    assert_eq!(config.output_format, "json");
    assert_eq!(config.batch_test_size, 1000);
}

#[test]
fn test_benchmark_config_from_env() {
    let mut m: HashMap<String, String> = HashMap::new();
    m.insert("SONGBIRD_BENCHMARK_ENABLED".into(), "1".into());
    m.insert("SONGBIRD_BENCHMARK_DURATION_SECS".into(), "120".into());
    m.insert("SONGBIRD_BENCHMARK_CONCURRENT".into(), "500".into());
    m.insert("SONGBIRD_BENCHMARK_WARMUP_SECS".into(), "15".into());
    m.insert("SONGBIRD_BENCHMARK_OUTPUT".into(), "csv".into());
    let config = BenchmarkConfig::from_env_reader(env_reader(&m));
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
    let m: HashMap<String, String> = HashMap::new();
    let config: UnifiedPerformanceConfig = PerformanceConfig::from_env_reader(env_reader(&m));
    assert_eq!(config.max_concurrent_requests, 1000);
}
