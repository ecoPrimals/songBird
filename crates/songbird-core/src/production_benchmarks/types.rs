//! Production Benchmark Types and Configuration
//!
//! Type definitions, configuration structures, and benchmark result types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Production benchmark suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of service instances to test with
    pub service_instance_count: usize,
    /// Number of requests per test
    pub requests_per_test: usize,
    /// Number of concurrent workers
    pub concurrent_workers: usize,
    /// Cache test data size
    pub cache_test_data_size: usize,
    /// Object pool test iterations
    pub object_pool_iterations: usize,
    /// Batch processing test size
    pub batch_test_size: usize,
    /// Warmup duration
    pub warmup_duration: Duration,
    /// Test duration
    pub test_duration: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            service_instance_count: 1000,
            requests_per_test: 100000,
            concurrent_workers: 100,
            cache_test_data_size: 10000,
            object_pool_iterations: 50000,
            batch_test_size: 10000,
            warmup_duration: Duration::from_secs(60),
            test_duration: Duration::from_secs(60),
        }
    }
}

/// Benchmark results for production analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub load_balancer_results: LoadBalancerBenchmark,
    pub cache_results: CacheBenchmark,
    pub object_pool_results: ObjectPoolBenchmark,
    pub batch_processing_results: BatchProcessingBenchmark,
    pub memory_results: MemoryBenchmark,
    pub overall_performance_score: f64,
    pub production_readiness_assessment: ProductionReadinessAssessment,
}

/// Load balancer benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerBenchmark {
    pub fast_algorithm_ops_per_second: f64,
    pub standard_algorithm_ops_per_second: f64,
    pub performance_improvement_factor: f64,
    pub average_selection_time_ns: u64,
    pub p99_selection_time_ns: u64,
    pub cache_hit_rate: f64,
}

/// Cache benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBenchmark {
    pub get_ops_per_second: f64,
    pub put_ops_per_second: f64,
    pub hit_rate_percentage: f64,
    pub average_access_time_ns: u64,
    pub memory_efficiency_mb_per_1k_items: f64,
    pub adaptive_performance_gain: f64,
}

/// Object pool benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolBenchmark {
    pub acquire_ops_per_second: f64,
    pub memory_reuse_percentage: f64,
    pub allocation_reduction_factor: f64,
    pub average_acquire_time_ns: u64,
}

/// Batch processing benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingBenchmark {
    pub items_per_second: f64,
    pub batching_efficiency: f64,
    pub latency_overhead_ms: f64,
    pub throughput_improvement_factor: f64,
}

/// Memory benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmark {
    pub baseline_memory_mb: f64,
    pub optimized_memory_mb: f64,
    pub memory_reduction_percentage: f64,
    pub gc_pressure_reduction: f64,
}

/// Production readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionReadinessAssessment {
    pub performance_score: f64,
    pub scalability_score: f64,
    pub efficiency_score: f64,
    pub reliability_score: f64,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
    pub production_ready: bool,
}

impl BenchmarkConfig {
    /// Create a configuration optimized for quick tests
    pub fn quick_test() -> Self {
        Self {
            service_instance_count: 100,
            requests_per_test: 10000,
            concurrent_workers: 10,
            cache_test_data_size: 1000,
            object_pool_iterations: 5000,
            batch_test_size: 1000,
            warmup_duration: Duration::from_secs(5),
            test_duration: Duration::from_secs(30),
        }
    }

    /// Create a configuration optimized for comprehensive production testing
    pub fn comprehensive() -> Self {
        Self {
            service_instance_count: 5000,
            requests_per_test: 1000000,
            concurrent_workers: 500,
            cache_test_data_size: 100000,
            object_pool_iterations: 500000,
            batch_test_size: 100000,
            warmup_duration: Duration::from_secs(120),
            test_duration: Duration::from_secs(300),
        }
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.service_instance_count == 0 {
            return Err("service_instance_count must be greater than 0".to_string());
        }
        if self.requests_per_test == 0 {
            return Err("requests_per_test must be greater than 0".to_string());
        }
        if self.concurrent_workers == 0 {
            return Err("concurrent_workers must be greater than 0".to_string());
        }
        if self.concurrent_workers > self.requests_per_test {
            return Err("concurrent_workers should not exceed requests_per_test".to_string());
        }
        Ok(())
    }
}

impl BenchmarkResults {
    /// Calculate a comprehensive performance score across all benchmarks
    pub fn calculate_comprehensive_score(&self) -> f64 {
        let lb_score = self
            .load_balancer_results
            .performance_improvement_factor
            .min(10.0)
            / 10.0;
        let cache_score = self.cache_results.hit_rate_percentage;
        let pool_score = self.object_pool_results.memory_reuse_percentage;
        let batch_score = self.batch_processing_results.batching_efficiency;
        let memory_score = self.memory_results.memory_reduction_percentage / 100.0;

        (lb_score * 0.25
            + cache_score * 0.25
            + pool_score * 0.2
            + batch_score * 0.15
            + memory_score * 0.15)
            * 100.0
    }
}

impl ProductionReadinessAssessment {
    /// Determine if the system meets minimum production standards
    pub fn meets_production_standards(&self) -> bool {
        self.overall_score >= 85.0
            && self.performance_score >= 80.0
            && self.scalability_score >= 75.0
            && self.efficiency_score >= 75.0
            && self.reliability_score >= 90.0
    }

    /// Get criticality level of recommendations
    pub fn get_critical_recommendations(&self) -> Vec<&String> {
        self.recommendations
            .iter()
            .filter(|rec| rec.contains("Critical") || rec.contains("critical"))
            .collect()
    }

    /// Generate production deployment advice
    pub fn deployment_advice(&self) -> String {
        if self.production_ready {
            format!(
                "✅ READY FOR PRODUCTION DEPLOYMENT\nOverall Score: {:.1}/100\nRecommendations: {}",
                self.overall_score,
                self.recommendations.len()
            )
        } else {
            format!(
                "⚠️  NOT READY FOR PRODUCTION\nOverall Score: {:.1}/100\nCritical Issues: {}\nTotal Recommendations: {}",
                self.overall_score,
                self.get_critical_recommendations().len(),
                self.recommendations.len()
            )
        }
    }
}
