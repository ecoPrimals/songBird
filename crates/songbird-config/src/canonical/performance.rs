//! Performance Configuration Module
//!
//! Canonical performance configuration for thread pools, caching, metrics,
//! and benchmarking across the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::env;

// ============================================================================
// PERFORMANCE CONFIGURATION
// ============================================================================

/// Unified performance configuration
///
/// **Canonical Source**: This is the definitive performance configuration  
/// **Migrated from**: `unified/performance.rs`  
/// **Purpose**: Thread pool, caching, and runtime performance settings
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::performance::PerformanceConfig;
///
/// let config = PerformanceConfig::default();
/// assert_eq!(config.max_concurrent_requests, 1000);
/// assert!(config.cache.enabled);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PerformanceConfig {
    /// Thread pool size for async runtime
    pub thread_pool_size: usize,

    /// Maximum concurrent requests allowed
    pub max_concurrent_requests: usize,

    /// Request buffer size in bytes
    pub request_buffer_size: usize,

    /// Enable zero-copy optimizations
    pub enable_zero_copy: bool,

    /// Connection pool size
    pub connection_pool_size: usize,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Metrics collection configuration
    pub metrics: MetricsConfig,

    /// Enable fast load balancing
    pub enable_fast_load_balancing: bool,

    /// Enable adaptive caching strategies
    pub enable_adaptive_caching: bool,

    /// Enable memory optimization techniques
    pub enable_memory_optimization: bool,

    /// Enable async batching for efficiency
    pub enable_async_batching: bool,

    /// Cache size in megabytes
    pub cache_size_mb: usize,

    /// Object pool sizes
    pub object_pool_sizes: ObjectPoolSizes,

    /// Monitoring interval
    pub monitoring_interval_secs: u64,

    /// Auto-tuning sensitivity (0.0 to 1.0)
    pub auto_tuning_sensitivity: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: env::var("SONGBIRD_THREAD_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| {
                    // Default to number of CPU cores
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(4)
                }),
            max_concurrent_requests: env::var("SONGBIRD_MAX_CONCURRENT_REQUESTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            request_buffer_size: 8192,
            enable_zero_copy: env::var("SONGBIRD_ZERO_COPY_ENABLED").is_ok(),
            connection_pool_size: 50,
            cache: CacheConfig::default(),
            metrics: MetricsConfig::default(),
            enable_fast_load_balancing: false,
            enable_adaptive_caching: false,
            enable_memory_optimization: false,
            enable_async_batching: false,
            cache_size_mb: 100,
            object_pool_sizes: ObjectPoolSizes::default(),
            monitoring_interval_secs: 60,
            auto_tuning_sensitivity: 0.5,
        }
    }
}

// ============================================================================
// OBJECT POOL CONFIGURATION
// ============================================================================

/// Object pool size configuration
///
/// **Migrated from**: `unified/performance.rs`  
/// **Purpose**: Configure object pooling for memory efficiency
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::performance::ObjectPoolSizes;
///
/// let pools = ObjectPoolSizes {
///     message: 2000,
///     buffer: 1000,
///     connection: 200,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectPoolSizes {
    /// Message object pool size
    pub message: usize,

    /// Buffer object pool size
    pub buffer: usize,

    /// Connection object pool size
    pub connection: usize,
}

impl Default for ObjectPoolSizes {
    fn default() -> Self {
        Self {
            message: env::var("SONGBIRD_MESSAGE_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            buffer: env::var("SONGBIRD_BUFFER_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500),
            connection: env::var("SONGBIRD_CONNECTION_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        }
    }
}

// ============================================================================
// CACHE CONFIGURATION
// ============================================================================

/// Cache configuration
///
/// **Migrated from**: `unified/performance.rs`  
/// **Purpose**: Configure caching behavior for performance optimization
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::performance::CacheConfig;
///
/// let cache = CacheConfig {
///     enabled: true,
///     max_size: 5000,
///     ttl_secs: 600,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Maximum cache size (number of entries)
    pub max_size: usize,

    /// Time-to-live in seconds
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_CACHE_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            max_size: env::var("SONGBIRD_CACHE_MAX_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            ttl_secs: env::var("SONGBIRD_CACHE_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        }
    }
}

// ============================================================================
// METRICS CONFIGURATION
// ============================================================================

/// Metrics configuration
///
/// **Migrated from**: `unified/performance.rs`  
/// **Purpose**: Configure metrics collection and export
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::performance::MetricsConfig;
///
/// let metrics = MetricsConfig {
///     enabled: true,
///     collection_interval_secs: 30,
///     export_prometheus: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,

    /// Export metrics in Prometheus format
    pub export_prometheus: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_METRICS_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            collection_interval_secs: env::var("SONGBIRD_METRICS_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            export_prometheus: env::var("SONGBIRD_PROMETHEUS_ENABLED").is_ok(),
        }
    }
}

// ============================================================================
// BENCHMARK CONFIGURATION
// ============================================================================

/// Benchmark configuration for performance testing
///
/// **Migrated from**: `unified/performance.rs`  
/// **Purpose**: Configure performance benchmarking and load testing
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::performance::BenchmarkConfig;
///
/// let benchmark = BenchmarkConfig {
///     enabled: true,
///     duration_secs: 120,
///     concurrent_requests: 500,
///     warmup_duration_secs: 15,
///     output_format: "json".to_string(),
///     batch_test_size: 2000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkConfig {
    /// Enable benchmarking
    pub enabled: bool,

    /// Benchmark duration in seconds
    pub duration_secs: u64,

    /// Number of concurrent requests
    pub concurrent_requests: usize,

    /// Warmup duration in seconds
    pub warmup_duration_secs: u64,

    /// Output format (json, text, csv)
    pub output_format: String,

    /// Batch test size for bulk operations
    pub batch_test_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_BENCHMARK_ENABLED").is_ok(),
            duration_secs: env::var("SONGBIRD_BENCHMARK_DURATION_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(60),
            concurrent_requests: env::var("SONGBIRD_BENCHMARK_CONCURRENT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            warmup_duration_secs: env::var("SONGBIRD_BENCHMARK_WARMUP_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            output_format: env::var("SONGBIRD_BENCHMARK_OUTPUT")
                .unwrap_or_else(|_| "json".to_string()),
            batch_test_size: 1000,
        }
    }
}

// ============================================================================
// BACKWARD COMPATIBILITY ALIASES
// ============================================================================

/// Backward compatibility alias for UnifiedPerformanceConfig
pub type UnifiedPerformanceConfig = PerformanceConfig;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.max_concurrent_requests, 1000);
        assert_eq!(config.request_buffer_size, 8192);
        assert_eq!(config.connection_pool_size, 50);
        assert!(config.cache.enabled);
        assert!(config.metrics.enabled);
    }

    #[test]
    fn test_object_pool_sizes_default() {
        let pools = ObjectPoolSizes::default();
        assert_eq!(pools.message, 1000);
        assert_eq!(pools.buffer, 500);
        assert_eq!(pools.connection, 100);
    }

    #[test]
    fn test_cache_config_default() {
        let cache = CacheConfig::default();
        assert!(cache.enabled);
        assert_eq!(cache.max_size, 1000);
        assert_eq!(cache.ttl_secs, 300);
    }

    #[test]
    fn test_metrics_config_default() {
        let metrics = MetricsConfig::default();
        assert!(metrics.enabled);
        assert_eq!(metrics.collection_interval_secs, 60);
    }

    #[test]
    fn test_benchmark_config_default() {
        let benchmark = BenchmarkConfig::default();
        assert_eq!(benchmark.duration_secs, 60);
        assert_eq!(benchmark.concurrent_requests, 100);
        assert_eq!(benchmark.warmup_duration_secs, 10);
        assert_eq!(benchmark.output_format, "json");
        assert_eq!(benchmark.batch_test_size, 1000);
    }

    #[test]
    fn test_cache_config_serialization() {
        let cache = CacheConfig {
            enabled: true,
            max_size: 2000,
            ttl_secs: 600,
        };

        let json = serde_json::to_string(&cache).unwrap();
        let deserialized: CacheConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(cache, deserialized);
    }

    #[test]
    fn test_performance_config_thread_pool_size() {
        let config = PerformanceConfig::default();
        // Should be at least 1 (single core) or more
        assert!(config.thread_pool_size >= 1);
    }
}

