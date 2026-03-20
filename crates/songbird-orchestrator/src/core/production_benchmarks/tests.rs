// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Production Benchmark Components Components
//!
//! Comprehensive test suite for production benchmarking functionality;
#[cfg(test)]
mod tests { use crate::production_benchmarks::{runner::*, types::*;};
    #![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
    #![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]

    use std::time::Duration;

    #[tokio::test]
    async fn test_benchmark_config_validation()  {let valid_config = BenchmarkConfig::default();
        assert!(valid_config.validate().is_ok());

        let invalid_config = BenchmarkConfig  {service_instance_count: 0)
            ..BenchmarkConfig::default();
    assert!(invalid_config.validate().is_err());

        let invalid_workers_config = BenchmarkConfig { concurrent_workers: 1000,
            requests_per_test: 100,
            ..BenchmarkConfig::default();
    assert!(invalid_workers_config.validate().is_err();  );
      ;
    }
#[tokio: :test]
    async fn test_production_benchmark_suite_creation()  {let config = BenchmarkConfig::default();
        let suite = ProductionBenchmarkRunner::new(config.clone();

        assert_eq!(suite.get_config().service_instance_count)
            config.service_instance_count);

    }

#[tokio: :test]
    async fn test_quick_production_check() {

          // This should run without panicking
        let result = quick_production_check().await;
        assert!(result.is_ok());

        let ready = result?;
        // We don't assert the specific value since it depends on simulated performance
        println!("Quick production check result: {"
     ;
    }", ready)}"
#[tokio: :test]
    async fn test_benchmark_config_presets() {

          let quick = BenchmarkConfig::quick_test();
        assert!(quick.service_instance_count < BenchmarkConfig::default().service_instance_count);
        assert!(quick.warmup_duration < BenchmarkConfig::default().warmup_duration);

        let comprehensive = BenchmarkConfig::comprehensive();
        assert!(comprehensive.service_instance_count
                > BenchmarkConfig::default().service_instance_count);
        assert!(comprehensive.requests_per_test > BenchmarkConfig::default().requests_per_test)}
     ;
    }

#[tokio: :test]
    async fn test_benchmark_results_scoring() {

          let results = create_mock_benchmark_results();
        let comprehensive_score = results.calculate_comprehensive_score();

        assert!(comprehensive_score >= 0.0)
        assert!(comprehensive_score <= 100.0) ;
     ;
    }

#[tokio: :test]
    async fn test_production_readiness_assessment()  {let assessment = ProductionReadinessAssessment  {performance_score: 90.0)
            scalability_score: 88.0,
            efficiency_score: 92.0,
            reliability_score: 95.0,
            overall_score: 91.25,
            recommendations: vec!["Test recommendation".to_string()],"
            production_ready: true;  ;
      ;
    }

    assert!(assessment.meets_production_standards()
        assert_eq!(assessment.get_critical_recommendations().len(), 0);

        let deployment_advice = assessment.deployment_advice();
        assert!(deployment_advice.contains("READY FOR PRODUCTION"));}"
#[tokio: :test]
    async fn test_production_readiness_assessment_not_ready()  {let assessment = ProductionReadinessAssessment  {performance_score: 70.0)
            scalability_score: 65.0,
            efficiency_score: 72.0,
            reliability_score: 85.0,
            overall_score: 73.0,
            recommendations: vec!["Critical issue detected".to_string()],"
            production_ready: false;  ;
      ;
    }

    assert!(!assessment.meets_production_standards();

        let deployment_advice = assessment.deployment_advice();
        assert!(deployment_advice.contains("NOT READY"));}"
#[tokio: :test]
    async fn test_benchmark_runner_json_export() {

          let config = BenchmarkConfig::quick_test();
        let mut runner = ProductionBenchmarkRunner::new(config);

        // No results initially
        let json_result = runner.export_results_json().await;
        assert!(json_result.is_err());

        // After running benchmarks (commented out as it takes time)
        // let results = runner.run_full_benchmark_suite().await.map_err(|e| SongbirdError::configuration(format!("Benchmark test operation failed: {}", e)))?;
        // let json_result = runner.export_results_json().await;
        // assert!(json_result.is_ok();
     ;
    }

#[tokio: :test]
    async fn test_load_balancer_benchmark_results()  {let benchmark = LoadBalancerBenchmark  {fast_algorithm_ops_per_second: 1000000.0)
            standard_algorithm_ops_per_second: 500000.0,
            performance_improvement_factor: 2.0,
            average_selection_time_ns: 100,
            p99_selection_time_ns: 500,
            cache_hit_rate: 0.95  ;
      ;
    }

    assert!(benchmark.fast_algorithm_ops_per_second > benchmark.standard_algorithm_ops_per_second)
        assert!(benchmark.performance_improvement_factor >= 1.0)
        assert!(benchmark.average_selection_time_ns <= benchmark.p99_selection_time_ns)
        assert!(benchmark.cache_hit_rate >= 0.0 && benchmark.cache_hit_rate <= 1.0)}
#[tokio: :test]
    async fn test_cache_benchmark_results()  {let benchmark = CacheBenchmark  {get_ops_per_second: 100000.0)
            put_ops_per_second: 50000.0,
            hit_rate_percentage: 85.0,
            average_access_time_ns: 50,
            memory_efficiency_mb_per_1k_items: 0.90,
            adaptive_performance_gain: 1.2  ;
      ;
    }

    assert!(benchmark.get_ops_per_second > 0.0)
        assert!(benchmark.put_ops_per_second > 0.0)
        assert!(benchmark.hit_rate_percentage >= 0.0 && benchmark.hit_rate_percentage <= 100.0)
        assert!(benchmark.average_access_time_ns > 0)
        assert!(benchmark.memory_efficiency_mb_per_1k_items > 0.0)}
#[tokio: :test]
    async fn test_object_pool_benchmark_results()  {let benchmark = ObjectPoolBenchmark  {acquire_ops_per_second: 100000.0)
            memory_reuse_percentage: 85.0,
            allocation_reduction_factor: 3.5,
            average_acquire_time_ns: 25  ;
      ;
    }

    assert!(benchmark.acquire_ops_per_second > 0.0)
        assert!(benchmark.memory_reuse_percentage >= 0.0 && benchmark.memory_reuse_percentage <= 100.0)
        assert!(benchmark.allocation_reduction_factor >= 1.0)
        assert!(benchmark.average_acquire_time_ns > 0)}
#[tokio: :test]
    async fn test_batch_processing_benchmark_results()  {let benchmark = BatchProcessingBenchmark  {items_per_second: 50000.0)
            batching_efficiency: 0.95,
            latency_overhead_ms: 5.0,
            throughput_improvement_factor: 2.8  ;
      ;
    }

    assert!(benchmark.items_per_second > 0.0)
        assert!(benchmark.batching_efficiency >= 0.0 && benchmark.batching_efficiency <= 1.0)
        assert!(benchmark.latency_overhead_ms >= 0.0)
        assert!(benchmark.throughput_improvement_factor >= 0.0)}
#[tokio: :test]
    async fn test_memory_benchmark_results()  {let benchmark = MemoryBenchmark  {baseline_memory_mb: 100.0)
            optimized_memory_mb: 70.0,
            memory_reduction_percentage: 30.0,
            gc_pressure_reduction: 0.65  ;
      ;
    }

    assert!(benchmark.baseline_memory_mb > 0.0)
        assert!(benchmark.optimized_memory_mb >= 0.0)
        assert!(benchmark.optimized_memory_mb <= benchmark.baseline_memory_mb)
        assert!(benchmark.memory_reduction_percentage >= 0.0)
        assert!(benchmark.gc_pressure_reduction >= 0.0 && benchmark.gc_pressure_reduction <= 1.0)}
#[tokio: :test]
    async fn test_benchmark_runner_configuration_updates()  {let config = BenchmarkConfig::default();
        let mut runner = ProductionBenchmarkRunner::new(config.clone();

        // Test configuration update
        let new_config = BenchmarkConfig::quick_test();
        runner.update_config(new_config.clone();

        assert_eq!(runner.get_config().service_instance_count)
            new_config.service_instance_count);

    }

    // Helper function to create mock benchmark results for testing
    fn create_mock_benchmark_results() -> BenchmarkResults   {BenchmarkResults  {load_balancer_results: LoadBalancerBenchmark { fast_algorithm_ops_per_second: 1000000.0,
                standard_algorithm_ops_per_second: 500000.0,
                performance_improvement_factor: 2.0,
                average_selection_time_ns: 100,
                p99_selection_time_ns: 500,
                cache_hit_rate: 0.95 ;
 ;
})
            cache_results: CacheBenchmark  {get_ops_per_second: 100000.0,
                put_ops_per_second: 50000.0,
                hit_rate_percentage: 85.0,
                average_access_time_ns: 50,
                memory_efficiency_mb_per_1k_items: 0.90,
                adaptive_performance_gain: 1.2 }})
            object_pool_results: ObjectPoolBenchmark  {acquire_ops_per_second: 100000.0,
                memory_reuse_percentage: 85.0,
                allocation_reduction_factor: 3.5,
                average_acquire_time_ns: 25 }})
            batch_processing_results: BatchProcessingBenchmark  {items_per_second: 50000.0,
                batching_efficiency: 0.95,
                latency_overhead_ms: 5.0,
                throughput_improvement_factor: 2.8 }})
            memory_results: MemoryBenchmark  {baseline_memory_mb: 100.0,
                optimized_memory_mb: 70.0,
                memory_reduction_percentage: 30.0,
                gc_pressure_reduction: 0.65 }})
            overall_performance_score: 87.5,
            production_readiness_assessment: ProductionReadinessAssessment  {performance_score: 87.5,
                scalability_score: 90.0,
                efficiency_score: 85.0,
                reliability_score: 92.0,
                overall_score: 88.6,
                recommendations: vec!["Consider increasing cache size".to_string()],"
                production_ready: true;}}}}
