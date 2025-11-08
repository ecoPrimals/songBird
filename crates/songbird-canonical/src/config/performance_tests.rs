//! Tests for Performance Configuration
//!
//! Comprehensive test coverage for performance optimization configuration.

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// PerformanceConfig Tests
// ============================================================================

#[test]
fn test_performance_config_default() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();

    assert!(config.enable_zero_cost);
    assert!(config.memory.enable_pooling);
    assert!(config.throughput.enable_batching);
    assert!(config.latency.enable_pipelining);
    Ok(())
}

#[test]
fn test_performance_config_serialization() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    let deserialized: PerformanceConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;

    assert_eq!(config.enable_zero_cost, deserialized.enable_zero_cost);
    Ok(())
}

#[test]
fn test_performance_config_clone() {
    let config = PerformanceConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_zero_cost, cloned.enable_zero_cost);
}

#[test]
fn test_performance_config_zero_cost_disabled() {
    let mut config = PerformanceConfig::default();
    config.enable_zero_cost = false;

    assert!(!config.enable_zero_cost);
}

// ============================================================================
// MemoryConfig Tests
// ============================================================================

#[test]
fn test_memory_config_default() {
    let config = MemoryConfig::default();

    assert!(config.enable_pooling);
    assert_eq!(config.pool_size, 1000);
    assert!(config.enable_zero_copy);
    assert_eq!(config.memory_limit_mb, Some(1024));
    assert!(!config.enable_profiling);
}

#[test]
fn test_memory_config_custom_pool_size() {
    let mut config = MemoryConfig::default();
    config.pool_size = 5000;

    assert_eq!(config.pool_size, 5000);
}

#[test]
fn test_memory_config_pooling_disabled() {
    let mut config = MemoryConfig::default();
    config.enable_pooling = false;

    assert!(!config.enable_pooling);
}

#[test]
fn test_memory_config_zero_copy_disabled() {
    let mut config = MemoryConfig::default();
    config.enable_zero_copy = false;

    assert!(!config.enable_zero_copy);
}

#[test]
fn test_memory_config_custom_limit() {
    let mut config = MemoryConfig::default();
    config.memory_limit_mb = Some(2048);

    assert_eq!(config.memory_limit_mb, Some(2048));
}

#[test]
fn test_memory_config_no_limit() {
    let mut config = MemoryConfig::default();
    config.memory_limit_mb = None;

    assert!(config.memory_limit_mb.is_none());
}

#[test]
fn test_memory_config_profiling_enabled() {
    let mut config = MemoryConfig::default();
    config.enable_profiling = true;

    assert!(config.enable_profiling);
}

// ============================================================================
// ThroughputConfig Tests
// ============================================================================

#[test]
fn test_throughput_config_default() {
    let config = ThroughputConfig::default();

    assert!(config.enable_batching);
    assert_eq!(config.batch_size, 100);
    assert!(config.worker_threads.is_none());
    assert!(config.enable_async);
    assert_eq!(config.queue_capacity, 10000);
}

#[test]
fn test_throughput_config_custom_batch_size() {
    let mut config = ThroughputConfig::default();
    config.batch_size = 500;

    assert_eq!(config.batch_size, 500);
}

#[test]
fn test_throughput_config_batching_disabled() {
    let mut config = ThroughputConfig::default();
    config.enable_batching = false;

    assert!(!config.enable_batching);
}

#[test]
fn test_throughput_config_custom_workers() {
    let mut config = ThroughputConfig::default();
    config.worker_threads = Some(8);

    assert_eq!(config.worker_threads, Some(8));
}

#[test]
fn test_throughput_config_async_disabled() {
    let mut config = ThroughputConfig::default();
    config.enable_async = false;

    assert!(!config.enable_async);
}

#[test]
fn test_throughput_config_custom_queue_capacity() {
    let mut config = ThroughputConfig::default();
    config.queue_capacity = 50000;

    assert_eq!(config.queue_capacity, 50000);
}

// ============================================================================
// LatencyConfig Tests
// ============================================================================

#[test]
fn test_latency_config_default() {
    let config = LatencyConfig::default();

    assert!(config.enable_pipelining);
    assert_eq!(config.keep_alive_timeout, 60);
    assert!(config.enable_connection_pooling);
    assert_eq!(config.max_connection_pool_size, 100);
    assert!(config.enable_caching);
    assert_eq!(config.cache_ttl, 300);
}

#[test]
fn test_latency_config_pipelining_disabled() {
    let mut config = LatencyConfig::default();
    config.enable_pipelining = false;

    assert!(!config.enable_pipelining);
}

#[test]
fn test_latency_config_custom_keep_alive() {
    let mut config = LatencyConfig::default();
    config.keep_alive_timeout = 120;

    assert_eq!(config.keep_alive_timeout, 120);
}

#[test]
fn test_latency_config_connection_pooling_disabled() {
    let mut config = LatencyConfig::default();
    config.enable_connection_pooling = false;

    assert!(!config.enable_connection_pooling);
}

#[test]
fn test_latency_config_custom_pool_size() {
    let mut config = LatencyConfig::default();
    config.max_connection_pool_size = 500;

    assert_eq!(config.max_connection_pool_size, 500);
}

#[test]
fn test_latency_config_caching_disabled() {
    let mut config = LatencyConfig::default();
    config.enable_caching = false;

    assert!(!config.enable_caching);
}

#[test]
fn test_latency_config_custom_cache_ttl() {
    let mut config = LatencyConfig::default();
    config.cache_ttl = 600;

    assert_eq!(config.cache_ttl, 600);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_performance_config_high_performance_preset() {
    let config = PerformanceConfig {
        enable_zero_cost: true,
        memory: MemoryConfig {
            enable_pooling: true,
            pool_size: 10000,
            enable_zero_copy: true,
            memory_limit_mb: Some(8192), // 8GB
            enable_profiling: false,
        },
        throughput: ThroughputConfig {
            enable_batching: true,
            batch_size: 1000,
            worker_threads: Some(16),
            enable_async: true,
            queue_capacity: 100_000,
        },
        latency: LatencyConfig {
            enable_pipelining: true,
            keep_alive_timeout: 30,
            enable_connection_pooling: true,
            max_connection_pool_size: 1000,
            enable_caching: true,
            cache_ttl: 600,
        },
    };

    assert!(config.enable_zero_cost);
    assert_eq!(config.memory.pool_size, 10000);
    assert_eq!(config.throughput.batch_size, 1000);
    assert_eq!(config.latency.max_connection_pool_size, 1000);
}

#[test]
fn test_performance_config_conservative_preset() {
    let config = PerformanceConfig {
        enable_zero_cost: false,
        memory: MemoryConfig {
            enable_pooling: false,
            pool_size: 100,
            enable_zero_copy: false,
            memory_limit_mb: Some(512), // 512MB
            enable_profiling: true,
        },
        throughput: ThroughputConfig {
            enable_batching: false,
            batch_size: 10,
            worker_threads: Some(2),
            enable_async: false,
            queue_capacity: 1000,
        },
        latency: LatencyConfig {
            enable_pipelining: false,
            keep_alive_timeout: 300,
            enable_connection_pooling: false,
            max_connection_pool_size: 10,
            enable_caching: false,
            cache_ttl: 60,
        },
    };

    assert!(!config.enable_zero_cost);
    assert!(!config.memory.enable_pooling);
    assert!(!config.throughput.enable_batching);
    assert!(!config.latency.enable_pipelining);
    assert!(config.memory.enable_profiling);
}

#[test]
fn test_performance_config_balanced_preset() {
    let config = PerformanceConfig {
        enable_zero_cost: true,
        memory: MemoryConfig {
            enable_pooling: true,
            pool_size: 2000,
            enable_zero_copy: true,
            memory_limit_mb: Some(2048), // 2GB
            enable_profiling: false,
        },
        throughput: ThroughputConfig {
            enable_batching: true,
            batch_size: 200,
            worker_threads: None, // Auto-detect
            enable_async: true,
            queue_capacity: 20000,
        },
        latency: LatencyConfig {
            enable_pipelining: true,
            keep_alive_timeout: 60,
            enable_connection_pooling: true,
            max_connection_pool_size: 200,
            enable_caching: true,
            cache_ttl: 300,
        },
    };

    assert!(config.enable_zero_cost);
    assert_eq!(config.memory.pool_size, 2000);
    assert!(config.throughput.worker_threads.is_none());
    assert_eq!(config.latency.cache_ttl, 300);
}
