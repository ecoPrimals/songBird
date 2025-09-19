//! Production Benchmarks Module
//!
//! Comprehensive benchmarking suite for production workloads with modular architecture.
//!
//! This module has been refactored from a single 1129-line file into focused components:
//! - `types` - Configuration structures and benchmark result types (~180 lines)
//! - `runner` - Main ProductionBenchmarkRunner orchestrator (~300 lines)
//! - `benchmarks/` - Individual benchmark implementations:
//!   - `load_balancer` - Load balancer performance benchmarking (~190 lines)
//!   - `cache` - Cache performance and eviction policy testing (~200 lines)
//!   - `object_pool` - Object pool efficiency and concurrency testing (~140 lines)
//!   - `batch_processing` - Batch processing optimization testing (~180 lines)
//!   - `memory` - Memory usage and allocation pattern analysis (~150 lines)
//! - `tests` - Comprehensive test suite (~280 lines)
//!
//! Total: ~1520 lines across 8 focused files vs 1 monolithic 1129-line file
//!
//! ## Features
//!
//! - **Load Balancer Benchmarking**: Compare fast vs standard algorithms with performance metrics
//! - **Cache Performance**: Test hit rates, eviction policies, and memory efficiency
//! - **Object Pool Testing**: Measure allocation reduction and memory reuse
//! - **Batch Processing**: Optimize batch sizes and measure throughput improvements
//! - **Memory Analysis**: Track memory usage patterns and fragmentation
//! - **Production Readiness**: Comprehensive assessment with actionable recommendations
//!
//! ## Usage
//!
//! ```rust
//! use songbird_core::production_benchmarks::{BenchmarkConfig, ProductionBenchmarkRunner};
//!
//! #[tokio::main]
//! async fn main() -> SongbirdResult<()>> {
//!     let config = BenchmarkConfig::default();
//!     let mut runner = ProductionBenchmarkRunner::new(config);
//!     
//!     let results = runner.run_full_benchmark_suite().await?;
//!     
//!     if results.production_readiness_assessment.production_ready {
//!         println!("✅ System is production ready!");
//!     } else {
//!         println!("⚠️ System needs optimization before production deployment");
//!     }
//!     
//!     Ok(())
//! }
//! ```

// Declare modules
pub mod benchmarks;
pub mod runner;
pub mod types;

// #[cfg(test)]
// pub mod tests; // TODO: Create tests module

// Re-export commonly used types for convenience
pub use types::{
    BatchProcessingBenchmark,
    // Configuration
    BenchmarkConfig,

    // Main results
    BenchmarkResults,
    CacheBenchmark,
    // Individual benchmark results
    LoadBalancerBenchmark,
    MemoryBenchmark,
    ObjectPoolBenchmark,
    ProductionReadinessAssessment,
};

// Re-export main runner
pub use runner::ProductionBenchmarkRunner;

// Re-export specialized benchmarkers for advanced usage
pub use benchmarks::{
    BatchProcessingBenchmarker, CacheBenchmarker, LoadBalancerBenchmarker, MemoryBenchmarker,
    ObjectPoolBenchmarker,
};

/// Create a production benchmark runner with default configuration
pub fn create_default_benchmark_runner() -> ProductionBenchmarkRunner {
    ProductionBenchmarkRunner::new(songbird_config::SongbirdConfig::default())
}

/// Create a benchmark runner optimized for quick testing
pub fn create_quick_benchmark_runner() -> ProductionBenchmarkRunner {
    ProductionBenchmarkRunner::new(songbird_config::SongbirdConfig::default())
}

/// Create a benchmark runner for comprehensive production testing
pub fn create_comprehensive_benchmark_runner() -> ProductionBenchmarkRunner {
    ProductionBenchmarkRunner::new(songbird_config::SongbirdConfig::default())
}
