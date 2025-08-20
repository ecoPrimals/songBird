//! Performance configuration structures

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Object pool size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolSizes {
    pub message: usize,
    pub buffer: usize,
    pub connection: usize,
}

impl Default for ObjectPoolSizes {
    fn default() -> Self {
        Self {
            message: 1000,
            buffer: 500,
            connection: 100,
        }
    }
}

/// Unified performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct UnifiedPerformanceConfig {
    pub thread_pool_size: usize,
    pub max_concurrent_requests: usize,
    pub request_buffer_size: usize,
    pub enable_zero_copy: bool,
    pub connection_pool_size: usize,
    pub cache: CacheConfig,
    pub metrics: MetricsConfig,

    // Backward compatibility fields
    pub enable_fast_load_balancing: bool,
    pub enable_adaptive_caching: bool,
    pub enable_memory_optimization: bool,
    pub enable_async_batching: bool,
    pub cache_size_mb: usize,
    pub object_pool_sizes: ObjectPoolSizes,
    pub monitoring_interval: Duration,
    pub auto_tuning_sensitivity: f64,
}

impl Default for UnifiedPerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: env::var("SONGBIRD_THREAD_POOL_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(num_cpus::get),
            max_concurrent_requests: 1000,
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
            monitoring_interval: Duration::from_secs(60),
            auto_tuning_sensitivity: 0.5,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size: usize,
    pub ttl_secs: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1000,
            ttl_secs: 300,
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval_secs: u64,
    pub export_prometheus: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: 60,
            export_prometheus: env::var("SONGBIRD_PROMETHEUS_ENABLED").is_ok(),
        }
    }
}

/// Benchmark configuration for performance testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub enabled: bool,
    pub duration_secs: u64,
    pub concurrent_requests: usize,
    pub warmup_duration_secs: u64,
    pub output_format: String,
    pub batch_test_size: usize, // Added for backward compatibility
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            duration_secs: 60,
            concurrent_requests: 100,
            warmup_duration_secs: 10,
            output_format: "json".to_string(),
            batch_test_size: 1000, // Default batch test size
        }
    }
}

/// Performance configuration (backward compatibility alias)
pub type PerformanceConfig = UnifiedPerformanceConfig;
