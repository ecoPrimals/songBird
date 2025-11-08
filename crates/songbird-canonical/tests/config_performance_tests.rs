//! Tests for Performance Configuration
//!
//! Comprehensive tests for performance optimization configuration structures

use songbird_canonical::config::performance::{
    LatencyConfig, MemoryConfig, PerformanceConfig, ThroughputConfig,
};
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_performance_config_default() {
    let config = PerformanceConfig::default();

    assert!(config.enable_zero_cost);
    assert!(config.memory.enable_pooling);
    assert!(config.throughput.enable_batching);
    assert!(config.latency.enable_pipelining);
}

#[test]
fn test_memory_config_defaults() {
    let config = MemoryConfig::default();

    assert!(config.enable_pooling);
    assert_eq!(config.pool_size, 1000);
    assert!(config.enable_zero_copy);
    assert_eq!(config.memory_limit_mb, Some(1024));
    assert!(!config.enable_profiling);
}

#[test]
fn test_memory_config_custom() {
    let config = MemoryConfig {
        enable_pooling: true,
        pool_size: 5000,
        enable_zero_copy: true,
        memory_limit_mb: Some(2048),
        enable_profiling: true,
    };

    assert_eq!(config.pool_size, 5000);
    assert_eq!(config.memory_limit_mb, Some(2048));
    assert!(config.enable_profiling);
}

#[test]
fn test_memory_config_no_limit() {
    let config = MemoryConfig {
        enable_pooling: true,
        pool_size: 1000,
        enable_zero_copy: true,
        memory_limit_mb: None,
        enable_profiling: false,
    };

    assert!(config.memory_limit_mb.is_none());
}

#[test]
fn test_throughput_config_defaults() {
    let config = ThroughputConfig::default();

    assert!(config.enable_batching);
    assert_eq!(config.batch_size, 100);
    assert!(config.worker_threads.is_none());
    assert!(config.enable_async);
    assert_eq!(config.queue_capacity, 10000);
}

#[test]
fn test_throughput_config_custom() {
    let config = ThroughputConfig {
        enable_batching: true,
        batch_size: 500,
        worker_threads: Some(16),
        enable_async: true,
        queue_capacity: 50000,
    };

    assert_eq!(config.batch_size, 500);
    assert_eq!(config.worker_threads, Some(16));
    assert_eq!(config.queue_capacity, 50000);
}

#[test]
fn test_throughput_config_sync_mode() {
    let config = ThroughputConfig {
        enable_batching: false,
        batch_size: 1,
        worker_threads: Some(1),
        enable_async: false,
        queue_capacity: 100,
    };

    assert!(!config.enable_batching);
    assert!(!config.enable_async);
}

#[test]
fn test_latency_config_defaults() {
    let config = LatencyConfig::default();

    assert!(config.enable_pipelining);
    assert_eq!(config.keep_alive_timeout, 60);
    assert!(config.enable_connection_pooling);
    assert_eq!(config.max_connection_pool_size, 100);
    assert!(config.enable_caching);
    assert_eq!(config.cache_ttl, 300);
}

#[test]
fn test_latency_config_aggressive() {
    let config = LatencyConfig {
        enable_pipelining: true,
        keep_alive_timeout: 30,
        enable_connection_pooling: true,
        max_connection_pool_size: 500,
        enable_caching: true,
        cache_ttl: 600,
    };

    assert_eq!(config.max_connection_pool_size, 500);
    assert_eq!(config.cache_ttl, 600);
}

#[test]
fn test_latency_config_no_caching() -> SongbirdResult<()> {
    let config = LatencyConfig {
        enable_pipelining: true,
        keep_alive_timeout: 60,
        enable_connection_pooling: true,
        max_connection_pool_size: 100,
        enable_caching: false,
        cache_ttl: 0,
    };

    assert!(!config.enable_caching);
    assert_eq!(config.cache_ttl, 0);
    Ok(())
}

#[test]
fn test_performance_config_serialization() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("enable_zero_cost"));
    assert!(json.contains("memory"));
    assert!(json.contains("throughput"));
    assert!(json.contains("latency"));

    let deserialized: PerformanceConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.enable_zero_cost, deserialized.enable_zero_cost);
    Ok(())
}

#[test]
fn test_performance_config_clone() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let cloned = config.clone();

    assert_eq!(config.enable_zero_cost, cloned.enable_zero_cost);
    assert_eq!(config.memory.pool_size, cloned.memory.pool_size);
    assert_eq!(config.throughput.batch_size, cloned.throughput.batch_size);
    Ok(())
}

#[test]
fn test_performance_config_debug() -> SongbirdResult<()> {
    let config = PerformanceConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("PerformanceConfig"));
    assert!(debug_str.contains("memory"));
    Ok(())
}

#[test]
fn test_memory_config_extreme_values() -> SongbirdResult<()> {
    let minimal = MemoryConfig {
        enable_pooling: false,
        pool_size: 10,
        enable_zero_copy: false,
        memory_limit_mb: Some(64),
        enable_profiling: false,
    };

    let maximal = MemoryConfig {
        enable_pooling: true,
        pool_size: 100_000,
        enable_zero_copy: true,
        memory_limit_mb: Some(65536), // 64GB
        enable_profiling: true,
    };

    assert!(minimal.pool_size < maximal.pool_size);
    assert!(
        minimal.memory_limit_mb.ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))? < maximal.memory_limit_mb.ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?
    );
    Ok(())
}

#[test]
fn test_throughput_worker_threads() {
    let auto = ThroughputConfig::default();
    assert!(auto.worker_threads.is_none()); // System default

    let manual = ThroughputConfig {
        enable_batching: true,
        batch_size: 100,
        worker_threads: Some(32),
        enable_async: true,
        queue_capacity: 10000,
    };
    assert_eq!(manual.worker_threads, Some(32));
}

#[test]
fn test_latency_connection_pooling_disabled() {
    let config = LatencyConfig {
        enable_pipelining: false,
        keep_alive_timeout: 0,
        enable_connection_pooling: false,
        max_connection_pool_size: 0,
        enable_caching: false,
        cache_ttl: 0,
    };

    assert!(!config.enable_connection_pooling);
    assert_eq!(config.max_connection_pool_size, 0);
}

#[test]
fn test_performance_all_disabled() {
    let config = PerformanceConfig {
        enable_zero_cost: false,
        memory: MemoryConfig {
            enable_pooling: false,
            pool_size: 0,
            enable_zero_copy: false,
            memory_limit_mb: None,
            enable_profiling: false,
        },
        throughput: ThroughputConfig {
            enable_batching: false,
            batch_size: 1,
            worker_threads: Some(1),
            enable_async: false,
            queue_capacity: 1,
        },
        latency: LatencyConfig {
            enable_pipelining: false,
            keep_alive_timeout: 0,
            enable_connection_pooling: false,
            max_connection_pool_size: 0,
            enable_caching: false,
            cache_ttl: 0,
        },
    };

    assert!(!config.enable_zero_cost);
    assert!(!config.memory.enable_pooling);
    assert!(!config.throughput.enable_batching);
    assert!(!config.latency.enable_pipelining);
}

#[test]
fn test_performance_config_high_performance() {
    let config = PerformanceConfig {
        enable_zero_cost: true,
        memory: MemoryConfig {
            enable_pooling: true,
            pool_size: 10000,
            enable_zero_copy: true,
            memory_limit_mb: Some(8192),
            enable_profiling: true,
        },
        throughput: ThroughputConfig {
            enable_batching: true,
            batch_size: 1000,
            worker_threads: Some(64),
            enable_async: true,
            queue_capacity: 100_000,
        },
        latency: LatencyConfig {
            enable_pipelining: true,
            keep_alive_timeout: 120,
            enable_connection_pooling: true,
            max_connection_pool_size: 1000,
            enable_caching: true,
            cache_ttl: 3600,
        },
    };

    assert!(config.enable_zero_cost);
    assert!(config.memory.enable_zero_copy);
    assert_eq!(config.throughput.worker_threads, Some(64));
    assert_eq!(config.latency.max_connection_pool_size, 1000);
}
