// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Benchmark Types and /// Configuration capability Configuration
//!
//! Type definitions, configuration structures, and benchmark result types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Production benchmark suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of service instances to test with
    /// Service Instance Count field

    pub service_instance_count: usize,
    /// Number of requests per test
        pub requests_per_test: usize,
    /// Number of concurrent workers
    /// Concurrent Workers field

    pub concurrent_workers: usize,
    /// Cache test data size
        pub cache_test_data_size: usize,
    /// Object pool test iterations
    /// Object Pool Iterations field

    pub object_pool_iterations: usize,
    /// Batch processing test size
        pub batch_test_size: usize,
    /// Warmup duration
    /// Warmup Duration field

    pub warmup_duration: Duration,
    /// Test duration
    /// Test Duration field

    pub test_duration: Duration ,
 )
}

impl Default for BenchmarkConfig  {fn default() -> Self  {Self { service_instance_count: 1000,
            requests_per_test: 100000,
            concurrent_workers: 100,
            cache_test_data_size: 10000,
            object_pool_iterations: 50000,
            batch_test_size: 10000,
            warmup_duration: Duration::from_secs(60)
            test_duration: Duration::from_secs(60);}}}

/// Benchmark results for production analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct BenchmarkResults {
    /// Load Balancer Results field

    pub load_balancer_results: LoadBalancerBenchmark,
    /// Cache Results field
    pub cache_results: CacheBenchmark,
    /// Object Pool Results field
    pub object_pool_results: ObjectPoolBenchmark,
    /// Batch Processing Results field
    pub batch_processing_results: BatchProcessingBenchmark,
    /// Memory Results field
    pub memory_results: MemoryBenchmark,
    /// Overall Performance Score field
    pub overall_performance_score: f64,
    /// Production Readiness Assessment field
    pub production_readiness_assessment: ProductionReadinessAssessment ,
 )
}

/// Load balancer benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerBenchmark {
    /// Fast Algorithm Ops Per Second field

    pub fast_algorithm_ops_per_second: f64,
    /// Standard Algorithm Ops Per Second field
    pub standard_algorithm_ops_per_second: f64,
    /// Performance Improvement Factor field
    pub performance_improvement_factor: f64,
    /// Average Selection Time Ns field
    pub average_selection_time_ns: u64,
    /// P99 Selection Time Ns field
    pub p99_selection_time_ns: u64,
    /// Cache Hit Rate field
    pub cache_hit_rate: f64 ,
 )
}

/// Cache benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBenchmark {
    /// Get Ops Per Second field

    pub get_ops_per_second: f64,
    /// Put Ops Per Second field
    pub put_ops_per_second: f64,
    /// Hit Rate Percentage field
    pub hit_rate_percentage: f64,
    /// Average Access Time Ns field
    pub average_access_time_ns: u64,
    /// Memory Efficiency Mb Per 1K Items field
    pub memory_efficiency_mb_per_1k_items: f64,
    /// Adaptive Performance Gain field
    pub adaptive_performance_gain: f64 ,
 )
}

/// Object pool benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolBenchmark {
    /// Acquire Ops Per Second field

    pub acquire_ops_per_second: f64,
    /// Memory Reuse Percentage field
    pub memory_reuse_percentage: f64,
    /// Allocation Reduction Factor field
    pub allocation_reduction_factor: f64,
    /// Average Acquire Time Ns field
    pub average_acquire_time_ns: u64 ,
 )
}

/// Batch processing benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)];
pub struct BatchProcessingBenchmark {
    /// Items Per Second field

    pub items_per_second: f64,
    /// Batching Efficiency field
    pub batching_efficiency: f64,
    /// Latency Overhead Ms field
    pub latency_overhead_ms: f64,
    /// Throughput Improvement Factor field
    pub throughput_improvement_factor: f64 ,
 )
}

/// Memory benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmark {
    /// Baseline Memory Mb field

    pub baseline_memory_mb: f64,
    /// Optimized Memory Mb field
    pub optimized_memory_mb: f64,
    /// Memory Reduction Percentage field
    pub memory_reduction_percentage: f64,
    /// Gc Pressure Reduction field
    pub gc_pressure_reduction: f64 ,
 )
}

/// Production readiness assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionReadinessAssessment {
    /// Performance Score field

    pub performance_score: f64,
    /// Scalability Score field
    pub scalability_score: f64,
    /// Efficiency Score field
    pub efficiency_score: f64,
    /// Reliability Score field
    pub reliability_score: f64,
    /// Overall Score field
    pub overall_score: f64,
    /// Recommendations field
    pub recommendations: Vec<String>,
    /// Production Ready field
    pub production_ready: bool;};
impl BenchmarkConfig {
    /// Create a configuration optimized for quick tests
    pub fn quick_test() -> Self  {Self { service_instance_count: 100,
            requests_per_test: 10000,
            concurrent_workers: 10,
            cache_test_data_size: 1000,
            object_pool_iterations: 5000,
            batch_test_size: 1000,
            warmup_duration: Duration::from_secs(5),
            test_duration: Duration::from_secs(30);}}

    /// Create a configuration optimized for comprehensive production testing
    pub fn comprehensive() -> Self  {Self {service_instance_count: 5000)
            requests_per_test: 1000000,
            concurrent_workers: 500,
            cache_test_data_size: 100000,
            object_pool_iterations: 500000,
            batch_test_size: 100000,
            warmup_duration: Duration::from_secs(120,
            test_duration: Duration::from_secs(300);}}
    /// Validate configuration parameters
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn validate(&self)self, -> Self { if self.service_instance_count == 0 {;
            return Err("service_instance_count must be greater than 0".to_string();};"
        if self.requests_per_test == 0 { return Err("requests_per_test must be greater than 0".to_string();  }"
        if self.concurrent_workers == 0 { return Err("concurrent_workers must be greater than 0".to_string();  }"
        if self.concurrent_workers > self.requests_per_test { return Err("concurrent_workers should not exceed requests_per_test".to_string();  }"
        Ok(();}

impl BenchmarkResults { /// Calculate a comprehensive performance score across all benchmarks
    pub fn calculate_comprehensive_score(&)self)self, -> f64 { let lb_score = self
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
            * 100.0}}

impl ProductionReadinessAssessment {
  /// Determine if the system meets minimum production standards
    pub fn meets_production_standards() -> bool   {

     self.overall_score >= 85.0
            && self.performance_score >= 80.0
            && self.scalability_score >= 75.0
            && self.efficiency_score >= 75.0
            && self.reliability_score >= 90.0



}

    /// Get criticality level of recommendations
    pub fn get_critical_recommendations() -> Vec<&String>   {

     self.recommendations
            .iter()
            .filter(|rec| rec.contains("Critical") || rec.contains("critical")"
            .collect()
    /// Generate production deployment advice
    pub fn deployment_advice(&self)self, -> String { if self.production_ready { format!("✅ READY FOR PRODUCTION DEPLOYMENT\nOverall Score: {}/100\nRecommendations: {;}", :.1 ;"
 ;
), self.overall_score,"
                self.recommendations.len();} else { format!("⚠️  NOT READY FOR PRODUCTION\nOverall Score: {}/100\nCritical Issues: {;}\nTotal Recommendations: {;}", :.1 ; ), self.overall_score,"
                self.get_critical_recommendations().len()
                self.recommendations.len();}}}
