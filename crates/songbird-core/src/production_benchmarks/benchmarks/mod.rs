//! Individual Benchmark Modules
//!
//! Specialized benchmark implementations for different system components

pub mod batch_processing;
pub mod cache;
pub mod load_balancer;
pub mod memory;
pub mod object_pool;

// Re-export benchmark types for convenience
pub use batch_processing::{
    BatchProcessingBenchmarker, BatchSizeOptimizationResult, ConcurrentBatchBenchmark,
};
pub use cache::{CacheBenchmarker, CacheEvictionBenchmark};
pub use load_balancer::{LoadBalancerBenchmarker, LoadBalancerStressTestResult};
pub use memory::{AllocationBenchmark, FragmentationBenchmark, MemoryBenchmarker};
pub use object_pool::{ObjectPoolBenchmarker, PoolConcurrencyBenchmark};
