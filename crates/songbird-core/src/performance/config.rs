//! Performance optimization configuration

use serde::{Deserialize, Serialize};
use songbird_config::constants::{
    DEFAULT_CACHE_TTL, DEFAULT_EVALUATION_TIMEOUT, DEFAULT_METRICS_INTERVAL,
};
use std::time::Duration;

/// Production performance optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable high-performance load balancing
    pub enable_fast_load_balancing: bool,
    /// Enable intelligent caching
    pub enable_adaptive_caching: bool,
    /// Enable memory optimization
    pub enable_memory_optimization: bool,
    /// Enable async batching
    pub enable_async_batching: bool,
    /// Cache size limit (MB)
    pub cache_size_mb: usize,
    /// Object pool sizes
    pub object_pool_sizes: ObjectPoolSizes,
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
    /// Auto-tuning sensitivity (0.0-1.0)
    pub auto_tuning_sensitivity: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_fast_load_balancing: true,
            enable_adaptive_caching: true,
            enable_memory_optimization: true,
            enable_async_batching: true,
            cache_size_mb: 128,
            object_pool_sizes: ObjectPoolSizes::default(),
            monitoring_interval: DEFAULT_EVALUATION_TIMEOUT,
            auto_tuning_sensitivity: 0.7,
        }
    }
}

/// Object pool size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolSizes {
    pub connection_pool: usize,
    pub buffer_pool: usize,
    pub message_pool: usize,
    pub request_pool: usize,
}

impl Default for ObjectPoolSizes {
    fn default() -> Self {
        Self {
            connection_pool: 1000,
            buffer_pool: 2000,
            message_pool: 5000,
            request_pool: 10000,
        }
    }
}

/// Load balancing strategy enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin with performance weights
    WeightedRoundRobin,
    /// Least connections with adaptive weighting
    AdaptiveLeastConnections,
    /// Performance-based selection with machine learning
    PerformanceBased,
    /// Latency-optimized for real-time workloads
    LatencyOptimized,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: usize,
    pub max_memory_mb: usize,
    pub ttl: Duration,
    pub frequency_window: Duration,
    pub adaptive_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            max_memory_mb: 64,
            ttl: DEFAULT_CACHE_TTL,
            frequency_window: DEFAULT_METRICS_INTERVAL,
            adaptive_threshold: 0.8,
        }
    }
}

/// Cache metrics structure
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size_bytes: usize,
    pub avg_access_time: Duration,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            total_size_bytes: 0,
            avg_access_time: Duration::from_nanos(0),
        }
    }
}

impl CacheMetrics {
    /// Calculate cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate average memory usage per entry
    pub fn avg_entry_size(&self) -> usize {
        if self.hits + self.misses == 0 {
            0
        } else {
            self.total_size_bytes / (self.hits + self.misses) as usize
        }
    }

    /// Check if cache performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.hit_ratio() >= 0.8 && self.avg_access_time < Duration::from_millis(1)
    }
}

/// Performance tuning result
#[derive(Debug, Clone)]
pub struct PerformanceTuningResult {
    pub cache_hit_ratio: f64,
    pub avg_response_time: Duration,
    pub memory_usage_mb: usize,
    pub cpu_usage_percent: f64,
    pub recommendations: Vec<String>,
}

impl PerformanceTuningResult {
    /// Create new tuning result
    pub fn new() -> Self {
        Self {
            cache_hit_ratio: 0.0,
            avg_response_time: Duration::from_millis(0),
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            recommendations: Vec::new(),
        }
    }

    /// Add performance recommendation
    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }

    /// Check if overall performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.cache_hit_ratio >= 0.8
            && self.avg_response_time < Duration::from_millis(10)
            && self.cpu_usage_percent < 80.0
    }

    /// Get performance score (0.0-1.0)
    pub fn performance_score(&self) -> f64 {
        let cache_score = self.cache_hit_ratio;
        let response_score = if self.avg_response_time.as_millis() <= 1 {
            1.0
        } else {
            1.0 / (self.avg_response_time.as_millis() as f64 / 1000.0)
        };
        let cpu_score = (100.0 - self.cpu_usage_percent) / 100.0;

        (cache_score + response_score + cpu_score) / 3.0
    }
}
