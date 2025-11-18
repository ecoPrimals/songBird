//! Comprehensive tests for canonical performance configuration
//!
//! Phase 3 Test Coverage Expansion - Week 1
//! Target: Additional coverage for performance.rs

use super::performance::*;

// ============================================================================
// OBJECT POOL SIZES TESTS
// ============================================================================

#[test]
fn test_object_pool_sizes_custom() {
    let pools = ObjectPoolSizes {
        message: 2000,
        buffer: 1000,
        connection: 200,
    };

    assert_eq!(pools.message, 2000);
    assert_eq!(pools.buffer, 1000);
    assert_eq!(pools.connection, 200);
}

#[test]
fn test_object_pool_sizes_clone() {
    let pools = ObjectPoolSizes::default();
    let cloned = pools.clone();

    assert_eq!(pools.message, cloned.message);
    assert_eq!(pools.buffer, cloned.buffer);
    assert_eq!(pools.connection, cloned.connection);
}

#[test]
fn test_object_pool_sizes_equality() {
    let pools1 = ObjectPoolSizes::default();
    let pools2 = ObjectPoolSizes::default();

    assert_eq!(pools1, pools2);
}

#[test]
fn test_object_pool_sizes_serialization() {
    let pools = ObjectPoolSizes {
        message: 1500,
        buffer: 750,
        connection: 150,
    };

    let json = serde_json::to_string(&pools).expect("Should serialize");
    let deserialized: ObjectPoolSizes = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(pools, deserialized);
}

#[test]
fn test_object_pool_sizes_large_pools() {
    let pools = ObjectPoolSizes {
        message: 10000,
        buffer: 5000,
        connection: 1000,
    };

    assert_eq!(pools.message, 10000);
    assert_eq!(pools.buffer, 5000);
    assert_eq!(pools.connection, 1000);
}

#[test]
fn test_object_pool_sizes_minimal_pools() {
    let pools = ObjectPoolSizes {
        message: 10,
        buffer: 5,
        connection: 1,
    };

    assert_eq!(pools.message, 10);
    assert_eq!(pools.buffer, 5);
    assert_eq!(pools.connection, 1);
}

// ============================================================================
// CACHE CONFIG TESTS
// ============================================================================

#[test]
fn test_cache_config_custom() {
    let cache = CacheConfig {
        enabled: false,
        max_size: 5000,
        ttl_secs: 600,
    };

    assert!(!cache.enabled);
    assert_eq!(cache.max_size, 5000);
    assert_eq!(cache.ttl_secs, 600);
}

#[test]
fn test_cache_config_disabled() {
    let cache = CacheConfig {
        enabled: false,
        max_size: 0,
        ttl_secs: 0,
    };

    assert!(!cache.enabled);
}

#[test]
fn test_cache_config_large_cache() {
    let cache = CacheConfig {
        enabled: true,
        max_size: 100000,
        ttl_secs: 3600,
    };

    assert!(cache.enabled);
    assert_eq!(cache.max_size, 100000);
    assert_eq!(cache.ttl_secs, 3600);
}

#[test]
fn test_cache_config_short_ttl() {
    let cache = CacheConfig {
        enabled: true,
        max_size: 1000,
        ttl_secs: 10,
    };

    assert_eq!(cache.ttl_secs, 10);
}

#[test]
fn test_cache_config_long_ttl() {
    let cache = CacheConfig {
        enabled: true,
        max_size: 1000,
        ttl_secs: 86400, // 1 day
    };

    assert_eq!(cache.ttl_secs, 86400);
}

#[test]
fn test_cache_config_clone() {
    let cache = CacheConfig::default();
    let cloned = cache.clone();

    assert_eq!(cache.enabled, cloned.enabled);
    assert_eq!(cache.max_size, cloned.max_size);
    assert_eq!(cache.ttl_secs, cloned.ttl_secs);
}

#[test]
fn test_cache_config_equality() {
    let cache1 = CacheConfig::default();
    let cache2 = CacheConfig::default();

    assert_eq!(cache1, cache2);
}

// ============================================================================
// METRICS CONFIG TESTS
// ============================================================================

#[test]
fn test_metrics_config_custom() {
    let metrics = MetricsConfig {
        enabled: false,
        collection_interval_secs: 30,
        export_prometheus: false,
    };

    assert!(!metrics.enabled);
    assert_eq!(metrics.collection_interval_secs, 30);
    assert!(!metrics.export_prometheus);
}

#[test]
fn test_metrics_config_with_prometheus() {
    let metrics = MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        export_prometheus: true,
    };

    assert!(metrics.enabled);
    assert!(metrics.export_prometheus);
}

#[test]
fn test_metrics_config_without_prometheus() {
    let metrics = MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        export_prometheus: false,
    };

    assert!(metrics.enabled);
    assert!(!metrics.export_prometheus);
}

#[test]
fn test_metrics_config_fast_collection() {
    let metrics = MetricsConfig {
        enabled: true,
        collection_interval_secs: 5,
        export_prometheus: true,
    };

    assert_eq!(metrics.collection_interval_secs, 5);
}

#[test]
fn test_metrics_config_slow_collection() {
    let metrics = MetricsConfig {
        enabled: true,
        collection_interval_secs: 300,
        export_prometheus: false,
    };

    assert_eq!(metrics.collection_interval_secs, 300);
}

#[test]
fn test_metrics_config_clone() {
    let metrics = MetricsConfig::default();
    let cloned = metrics.clone();

    assert_eq!(metrics.enabled, cloned.enabled);
    assert_eq!(metrics.collection_interval_secs, cloned.collection_interval_secs);
    assert_eq!(metrics.export_prometheus, cloned.export_prometheus);
}

#[test]
fn test_metrics_config_equality() {
    let metrics1 = MetricsConfig::default();
    let metrics2 = MetricsConfig::default();

    assert_eq!(metrics1, metrics2);
}

#[test]
fn test_metrics_config_serialization() {
    let metrics = MetricsConfig {
        enabled: true,
        collection_interval_secs: 45,
        export_prometheus: true,
    };

    let json = serde_json::to_string(&metrics).expect("Should serialize");
    let deserialized: MetricsConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(metrics, deserialized);
}

// ============================================================================
// BENCHMARK CONFIG TESTS
// ============================================================================

#[test]
fn test_benchmark_config_custom() {
    let benchmark = BenchmarkConfig {
        enabled: true,
        duration_secs: 120,
        concurrent_requests: 500,
        warmup_duration_secs: 15,
        output_format: "csv".to_string(),
        batch_test_size: 2000,
    };

    assert!(benchmark.enabled);
    assert_eq!(benchmark.duration_secs, 120);
    assert_eq!(benchmark.concurrent_requests, 500);
    assert_eq!(benchmark.warmup_duration_secs, 15);
    assert_eq!(benchmark.output_format, "csv");
    assert_eq!(benchmark.batch_test_size, 2000);
}

#[test]
fn test_benchmark_config_disabled() {
    let benchmark = BenchmarkConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(!benchmark.enabled);
}

#[test]
fn test_benchmark_config_output_formats() {
    let formats = vec!["json", "text", "csv"];

    for format in formats {
        let benchmark = BenchmarkConfig {
            enabled: true,
            output_format: format.to_string(),
            ..Default::default()
        };

        assert_eq!(benchmark.output_format, format);
    }
}

#[test]
fn test_benchmark_config_short_duration() {
    let benchmark = BenchmarkConfig {
        enabled: true,
        duration_secs: 10,
        ..Default::default()
    };

    assert_eq!(benchmark.duration_secs, 10);
}

#[test]
fn test_benchmark_config_long_duration() {
    let benchmark = BenchmarkConfig {
        enabled: true,
        duration_secs: 3600, // 1 hour
        ..Default::default()
    };

    assert_eq!(benchmark.duration_secs, 3600);
}

#[test]
fn test_benchmark_config_concurrent_request_levels() {
    let levels = vec![10, 50, 100, 500, 1000];

    for level in levels {
        let benchmark = BenchmarkConfig {
            enabled: true,
            concurrent_requests: level,
            ..Default::default()
        };

        assert_eq!(benchmark.concurrent_requests, level);
    }
}

#[test]
fn test_benchmark_config_warmup_durations() {
    let durations = vec![0, 5, 10, 30, 60];

    for duration in durations {
        let benchmark = BenchmarkConfig {
            enabled: true,
            warmup_duration_secs: duration,
            ..Default::default()
        };

        assert_eq!(benchmark.warmup_duration_secs, duration);
    }
}

#[test]
fn test_benchmark_config_batch_sizes() {
    let sizes = vec![100, 500, 1000, 5000, 10000];

    for size in sizes {
        let benchmark = BenchmarkConfig {
            enabled: true,
            batch_test_size: size,
            ..Default::default()
        };

        assert_eq!(benchmark.batch_test_size, size);
    }
}

#[test]
fn test_benchmark_config_clone() {
    let benchmark = BenchmarkConfig::default();
    let cloned = benchmark.clone();

    assert_eq!(benchmark.enabled, cloned.enabled);
    assert_eq!(benchmark.duration_secs, cloned.duration_secs);
    assert_eq!(benchmark.output_format, cloned.output_format);
}

#[test]
fn test_benchmark_config_equality() {
    let benchmark1 = BenchmarkConfig::default();
    let benchmark2 = BenchmarkConfig::default();

    assert_eq!(benchmark1, benchmark2);
}

#[test]
fn test_benchmark_config_serialization() {
    let benchmark = BenchmarkConfig {
        enabled: true,
        duration_secs: 90,
        concurrent_requests: 250,
        warmup_duration_secs: 20,
        output_format: "json".to_string(),
        batch_test_size: 1500,
    };

    let json = serde_json::to_string(&benchmark).expect("Should serialize");
    let deserialized: BenchmarkConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(benchmark, deserialized);
}

// ============================================================================
// PERFORMANCE CONFIG INTEGRATION TESTS
// ============================================================================

#[test]
fn test_performance_config_custom() {
    let config = PerformanceConfig {
        thread_pool_size: 8,
        max_concurrent_requests: 2000,
        request_buffer_size: 16384,
        enable_zero_copy: true,
        connection_pool_size: 100,
        cache: CacheConfig {
            enabled: true,
            max_size: 5000,
            ttl_secs: 600,
        },
        metrics: MetricsConfig {
            enabled: true,
            collection_interval_secs: 30,
            export_prometheus: true,
        },
        enable_fast_load_balancing: true,
        enable_adaptive_caching: true,
        enable_memory_optimization: true,
        enable_async_batching: true,
        cache_size_mb: 200,
        object_pool_sizes: ObjectPoolSizes {
            message: 2000,
            buffer: 1000,
            connection: 200,
        },
        monitoring_interval_secs: 30,
        auto_tuning_sensitivity: 0.7,
    };

    assert_eq!(config.thread_pool_size, 8);
    assert_eq!(config.max_concurrent_requests, 2000);
    assert!(config.enable_zero_copy);
    assert!(config.enable_fast_load_balancing);
}

#[test]
fn test_performance_config_all_optimizations_enabled() {
    let config = PerformanceConfig {
        enable_zero_copy: true,
        enable_fast_load_balancing: true,
        enable_adaptive_caching: true,
        enable_memory_optimization: true,
        enable_async_batching: true,
        ..Default::default()
    };

    assert!(config.enable_zero_copy);
    assert!(config.enable_fast_load_balancing);
    assert!(config.enable_adaptive_caching);
    assert!(config.enable_memory_optimization);
    assert!(config.enable_async_batching);
}

#[test]
fn test_performance_config_all_optimizations_disabled() {
    let config = PerformanceConfig {
        enable_zero_copy: false,
        enable_fast_load_balancing: false,
        enable_adaptive_caching: false,
        enable_memory_optimization: false,
        enable_async_batching: false,
        ..Default::default()
    };

    assert!(!config.enable_zero_copy);
    assert!(!config.enable_fast_load_balancing);
    assert!(!config.enable_adaptive_caching);
    assert!(!config.enable_memory_optimization);
    assert!(!config.enable_async_batching);
}

#[test]
fn test_performance_config_clone() {
    let config = PerformanceConfig::default();
    let cloned = config.clone();

    assert_eq!(config.thread_pool_size, cloned.thread_pool_size);
    assert_eq!(config.max_concurrent_requests, cloned.max_concurrent_requests);
}

#[test]
fn test_performance_config_serialization() {
    let config = PerformanceConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: PerformanceConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.thread_pool_size, deserialized.thread_pool_size);
    assert_eq!(config.max_concurrent_requests, deserialized.max_concurrent_requests);
}

#[test]
fn test_performance_config_auto_tuning_sensitivity_range() {
    let sensitivities = vec![0.0, 0.25, 0.5, 0.75, 1.0];

    for sensitivity in sensitivities {
        let config = PerformanceConfig {
            auto_tuning_sensitivity: sensitivity,
            ..Default::default()
        };

        assert_eq!(config.auto_tuning_sensitivity, sensitivity);
        assert!(config.auto_tuning_sensitivity >= 0.0 && config.auto_tuning_sensitivity <= 1.0);
    }
}

#[test]
fn test_performance_config_cache_size_variants() {
    let sizes = vec![50, 100, 200, 500, 1000];

    for size in sizes {
        let config = PerformanceConfig {
            cache_size_mb: size,
            ..Default::default()
        };

        assert_eq!(config.cache_size_mb, size);
    }
}

#[test]
fn test_performance_config_monitoring_intervals() {
    let intervals = vec![10, 30, 60, 120, 300];

    for interval in intervals {
        let config = PerformanceConfig {
            monitoring_interval_secs: interval,
            ..Default::default()
        };

        assert_eq!(config.monitoring_interval_secs, interval);
    }
}

#[test]
fn test_unified_performance_config_alias() {
    // Test backward compatibility alias
    let _config: UnifiedPerformanceConfig = PerformanceConfig::default();
}
