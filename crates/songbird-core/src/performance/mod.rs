//! Production Performance Optimizer
//!
//! Advanced performance optimization for production workloads including:
//! - High-performance load balancing with O(log n) algorithms
//! - Intelligent caching layers with LRU and adaptive algorithms
//! - Memory optimization with object pooling
//! - Async batching and pipeline optimization
//! - Real-time performance monitoring and auto-tuning
//!
//! ## Refactored Architecture
//!
//! The performance optimization system is organized into focused modules:
//! - `config` - All configuration structures and defaults
//! - `load_balancer` - FastLoadBalancer with O(log n) algorithms  
//! - `cache` - AdaptiveCache with LRU and performance-based eviction
//! - `object_pool` - ObjectPool for memory optimization
//! - `batch_processor` - AsyncBatchProcessor for pipeline optimization
//! - `monitor` - PerformanceMonitor for real-time monitoring
//! - `optimizer` - Main ProductionPerformanceOptimizer coordinator

pub mod batch_processor;
pub mod cache;
pub mod config;
pub mod load_balancer;
pub mod monitor;
pub mod object_pool;
pub mod optimizer;

// Re-export main types for backward compatibility
pub use config::{
    CacheConfig, CacheMetrics, LoadBalancingStrategy, ObjectPoolSizes, PerformanceConfig,
    PerformanceTuningResult,
};

pub use load_balancer::{
    FastLoadBalancer, InstanceMetrics, LoadBalancerStats, LruCache, ServiceInstanceMeta,
};

pub use cache::{AccessPattern, AdaptiveCache, CacheEntry};

pub use object_pool::{ByteBufferPool, ObjectPool, PooledObject, StringBufferPool, VecBufferPool};

pub use batch_processor::{
    AsyncBatchProcessor, BatchError, BatchProcessorBuilder, BatchStats, ByteBatchProcessor,
    StringBatchProcessor,
};

pub use monitor::{MetricUpdate, PerformanceMonitor, SystemMetrics};

pub use optimizer::{
    ComponentHealth, ComprehensiveMetrics, HealthCheckResult, ProductionPerformanceOptimizer,
};

// Legacy compatibility - Re-export the main optimizer as the original name
pub use optimizer::ProductionPerformanceOptimizer as PerformanceOptimizer;

/// Create a production-ready performance optimizer with default configuration
pub fn create_production_optimizer() -> ProductionPerformanceOptimizer {
    let config = PerformanceConfig::default();
    ProductionPerformanceOptimizer::new(config)
}

/// Create a gaming-optimized performance configuration
pub fn create_gaming_config() -> PerformanceConfig {
    PerformanceConfig {
        enable_fast_load_balancing: true,
        enable_adaptive_caching: true,
        enable_memory_optimization: true,
        enable_async_batching: false, // Disabled for lowest latency
        cache_size_mb: 256,           // Larger cache for gaming
        object_pool_sizes: ObjectPoolSizes {
            connection_pool: 2000,
            buffer_pool: 4000,
            message_pool: 10000,
            request_pool: 20000,
        },
        monitoring_interval: std::time::Duration::from_millis(100), // More frequent monitoring
        auto_tuning_sensitivity: 0.9,                               // Aggressive tuning for gaming
    }
}

/// Create a memory-optimized performance configuration
pub fn create_memory_optimized_config() -> PerformanceConfig {
    PerformanceConfig {
        enable_fast_load_balancing: true,
        enable_adaptive_caching: true,
        enable_memory_optimization: true,
        enable_async_batching: true,
        cache_size_mb: 64, // Smaller cache
        object_pool_sizes: ObjectPoolSizes {
            connection_pool: 500,
            buffer_pool: 1000,
            message_pool: 2500,
            request_pool: 5000,
        },
        monitoring_interval: std::time::Duration::from_secs(5),
        auto_tuning_sensitivity: 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer_creation() {
        let optimizer = create_production_optimizer();
        assert!(optimizer.get_config().enable_fast_load_balancing);
    }

    #[tokio::test]
    async fn test_gaming_config() {
        let config = create_gaming_config();
        assert_eq!(config.cache_size_mb, 256);
        assert!(!config.enable_async_batching); // Disabled for gaming
    }

    #[tokio::test]
    async fn test_memory_optimized_config() {
        let config = create_memory_optimized_config();
        assert_eq!(config.cache_size_mb, 64);
        assert_eq!(config.object_pool_sizes.connection_pool, 500);
    }

    #[tokio::test]
    async fn test_object_pool_basic_operations() {
        let pool = ObjectPool::new(|| String::with_capacity(100), 10);

        // Get object from pool
        let mut obj = pool.get().await;
        obj.push_str("test");
        assert_eq!(obj.as_str(), "test");

        // Object should be returned to pool when dropped
        drop(obj);

        // Give time for async drop to complete
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Pool should have one object now
        assert_eq!(pool.size().await, 1);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        use super::batch_processor::BatchProcessorBuilder;

        let processor = BatchProcessorBuilder::new()
            .batch_size(5)
            .batch_timeout(std::time::Duration::from_millis(100))
            .build(|batch: Vec<String>| Ok(batch.into_iter().map(|s| s.to_uppercase()).collect()));

        // Submit items for processing
        for i in 0..3 {
            let result = processor.submit(format!("item_{}", i)).await;
            assert!(result.is_ok());
        }

        // Check stats
        let stats = processor.get_stats().await;
        assert!(stats.items_processed >= 3);
    }
}
