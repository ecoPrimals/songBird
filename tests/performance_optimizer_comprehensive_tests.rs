//! Comprehensive tests for performance optimizer implementation
//!
//! This test suite provides extensive coverage for communication performance
//! optimizations, including request batching, metrics tracking, string building
//! optimizations, and performance measurement capabilities.

use songbird_lib::communication::performance_optimizer::*;
use std::time::Duration;

/// Test helper to create performance config with custom settings
fn create_test_config() -> PerformanceConfig {
    PerformanceConfig {
        enable_connection_reuse: true,
        enable_request_batching: true,
        max_batch_size: 5,
        batch_timeout: Duration::from_millis(50),
    }
}

/// Test helper to create minimal config for edge case testing
#[allow(dead_code)] // Helper function for future tests
fn create_minimal_config() -> PerformanceConfig {
    PerformanceConfig {
        enable_connection_reuse: false,
        enable_request_batching: false,
        max_batch_size: 1,
        batch_timeout: Duration::from_millis(1),
    }
}

/// Test helper to create high-performance config
fn create_high_performance_config() -> PerformanceConfig {
    PerformanceConfig {
        enable_connection_reuse: true,
        enable_request_batching: true,
        max_batch_size: 100,
        batch_timeout: Duration::from_millis(500),
    }
}

#[cfg(test)]
mod performance_config_tests {
    use super::*;

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert!(config.enable_connection_reuse);
        assert!(config.enable_request_batching);
        assert_eq!(config.max_batch_size, 10);
        assert_eq!(config.batch_timeout, Duration::from_millis(100));
    }

    #[test]
    fn test_performance_config_custom() {
        let config = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: true,
            max_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
        };

        assert!(!config.enable_connection_reuse);
        assert!(config.enable_request_batching);
        assert_eq!(config.max_batch_size, 5);
        assert_eq!(config.batch_timeout, Duration::from_millis(50));
    }

    #[test]
    fn test_performance_config_clone() {
        let config1 = PerformanceConfig::default();
        let config2 = config1.clone();

        assert_eq!(
            config1.enable_connection_reuse,
            config2.enable_connection_reuse
        );
        assert_eq!(
            config1.enable_request_batching,
            config2.enable_request_batching
        );
        assert_eq!(config1.max_batch_size, config2.max_batch_size);
        assert_eq!(config1.batch_timeout, config2.batch_timeout);
    }

    #[test]
    fn test_performance_config_debug() {
        let config = create_test_config();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("PerformanceConfig"));
        assert!(debug_str.contains("enable_connection_reuse"));
        assert!(debug_str.contains("max_batch_size"));
    }

    #[test]
    fn test_performance_config_extreme_values() {
        let config = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: false,
            max_batch_size: 0,
            batch_timeout: Duration::from_nanos(1),
        };

        assert!(!config.enable_connection_reuse);
        assert!(!config.enable_request_batching);
        assert_eq!(config.max_batch_size, 0);
        assert!(config.batch_timeout < Duration::from_millis(1));
    }
}

#[cfg(test)]
mod performance_metrics_tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();

        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.avg_response_time, Duration::from_secs(0));
        assert_eq!(metrics.requests_per_second, 0.0);
        assert_eq!(metrics.connection_reuse_ratio, 0.0);
        assert_eq!(metrics.allocations_saved, 0);
    }

    #[test]
    fn test_performance_metrics_debug() {
        let metrics = PerformanceMetrics::default();
        let debug_str = format!("{metrics:?}");

        assert!(debug_str.contains("PerformanceMetrics"));
        assert!(debug_str.contains("total_requests"));
        assert!(debug_str.contains("avg_response_time"));
    }

    #[test]
    fn test_performance_metrics_field_access() {
        let mut metrics = PerformanceMetrics {
            total_requests: 100,
            avg_response_time: Duration::from_millis(200),
            requests_per_second: 50.0,
            connection_reuse_ratio: 0.8,
            allocations_saved: 25,
        };

        assert_eq!(metrics.total_requests, 100);
        assert_eq!(metrics.avg_response_time, Duration::from_millis(200));
        assert_eq!(metrics.requests_per_second, 50.0);
        assert_eq!(metrics.connection_reuse_ratio, 0.8);
        assert_eq!(metrics.allocations_saved, 25);
    }
}

#[cfg(test)]
mod communication_optimizer_tests {
    use super::*;

    #[test]
    fn test_communication_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = CommunicationOptimizer::new(config);

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.avg_response_time, Duration::from_secs(0));
        assert_eq!(metrics.requests_per_second, 0.0);
    }

    #[test]
    fn test_communication_optimizer_debug() {
        let config = create_test_config();
        let optimizer = CommunicationOptimizer::new(config);

        let debug_str = format!("{optimizer:?}");
        assert!(debug_str.contains("CommunicationOptimizer"));
    }

    #[test]
    fn test_single_request_recording() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_request(Duration::from_millis(100));

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.avg_response_time, Duration::from_millis(100));
    }

    #[test]
    fn test_multiple_request_recording() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_request(Duration::from_millis(100));
        optimizer.record_request(Duration::from_millis(200));
        optimizer.record_request(Duration::from_millis(300));

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.avg_response_time, Duration::from_millis(200));
    }

    #[test]
    fn test_request_recording_accuracy() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        // Record requests with varying response times
        let durations = vec![
            Duration::from_millis(50),
            Duration::from_millis(150),
            Duration::from_millis(100),
            Duration::from_millis(200),
        ];

        for duration in &durations {
            optimizer.record_request(*duration);
        }

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 4);

        // Calculate expected average: (50+150+100+200)/4 = 125
        assert_eq!(metrics.avg_response_time, Duration::from_millis(125));
    }

    #[test]
    fn test_allocation_recording() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_allocation_saved();
        optimizer.record_allocation_saved();
        optimizer.record_allocation_saved();

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.allocations_saved, 3);
    }

    #[test]
    fn test_combined_recording() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_request(Duration::from_millis(100));
        optimizer.record_allocation_saved();
        optimizer.record_request(Duration::from_millis(200));
        optimizer.record_allocation_saved();

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.avg_response_time, Duration::from_millis(150));
        assert_eq!(metrics.allocations_saved, 2);
    }

    #[test]
    fn test_zero_duration_request() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_request(Duration::from_nanos(0));

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.avg_response_time, Duration::from_nanos(0));
    }

    #[test]
    fn test_high_precision_timing() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        optimizer.record_request(Duration::from_nanos(1500));
        optimizer.record_request(Duration::from_nanos(2500));

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.avg_response_time, Duration::from_nanos(2000));
    }

    #[test]
    fn test_should_batch_requests_enabled() {
        let config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
        };
        let optimizer = CommunicationOptimizer::new(config);

        assert!(optimizer.should_batch_requests(1));
        assert!(optimizer.should_batch_requests(3));
        assert!(optimizer.should_batch_requests(4));
        assert!(!optimizer.should_batch_requests(5));
        assert!(!optimizer.should_batch_requests(10));
    }

    #[test]
    fn test_should_batch_requests_disabled() {
        let config = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: false,
            max_batch_size: 1,
            batch_timeout: Duration::from_millis(1),
        };
        let optimizer = CommunicationOptimizer::new(config);

        assert!(!optimizer.should_batch_requests(1));
        assert!(!optimizer.should_batch_requests(3));
        assert!(!optimizer.should_batch_requests(10));
    }

    #[test]
    fn test_get_optimal_batch_size() {
        let config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
        };
        let optimizer = CommunicationOptimizer::new(config);

        assert_eq!(optimizer.get_optimal_batch_size(3), 3);
        assert_eq!(optimizer.get_optimal_batch_size(5), 5);
        assert_eq!(optimizer.get_optimal_batch_size(10), 5); // Capped at max
    }

    #[test]
    fn test_get_optimal_batch_size_disabled() {
        let config = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: false,
            max_batch_size: 1,
            batch_timeout: Duration::from_millis(1),
        };
        let optimizer = CommunicationOptimizer::new(config);

        assert_eq!(optimizer.get_optimal_batch_size(5), 1);
        assert_eq!(optimizer.get_optimal_batch_size(10), 1);
    }

    #[test]
    fn test_get_optimal_batch_size_high_latency() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        // Record high latency requests
        optimizer.record_request(Duration::from_millis(600)); // > 500ms threshold

        // Should reduce batch size due to high latency
        let optimal_size = optimizer.get_optimal_batch_size(10);
        assert!(optimal_size <= 5); // Should be reduced
    }

    #[test]
    fn test_get_optimal_batch_size_normal_latency() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        // Record normal latency requests
        optimizer.record_request(Duration::from_millis(100)); // < 500ms threshold

        // Should use full batch size for normal latency
        let optimal_size = optimizer.get_optimal_batch_size(10);
        assert_eq!(optimal_size, 5); // Max batch size
    }

    #[test]
    fn test_batching_with_zero_pending() {
        let config = create_test_config();
        let optimizer = CommunicationOptimizer::new(config);

        assert_eq!(optimizer.get_optimal_batch_size(0), 0);
        assert!(optimizer.should_batch_requests(0));
    }

    #[test]
    fn test_large_batch_size_config() {
        let config = create_high_performance_config(); // max_batch_size = 100
        let optimizer = CommunicationOptimizer::new(config);

        assert!(optimizer.should_batch_requests(50));
        assert!(optimizer.should_batch_requests(99));
        assert!(!optimizer.should_batch_requests(100));

        assert_eq!(optimizer.get_optimal_batch_size(50), 50);
        assert_eq!(optimizer.get_optimal_batch_size(150), 100);
    }
}

#[cfg(test)]
mod string_builder_optimizer_tests {
    use super::*;

    #[test]
    fn test_string_builder_creation() {
        let _builder = StringBuilderOptimizer::with_capacity(100);
        // Basic creation test
    }

    #[test]
    fn test_simple_string_building() {
        let mut builder = StringBuilderOptimizer::with_capacity(50);

        let result = builder.build_string(|s| {
            s.push_str("hello");
        });

        assert_eq!(result, "hello");
    }

    #[test]
    fn test_complex_string_building() {
        let mut builder = StringBuilderOptimizer::with_capacity(100);

        let result = builder.build_string(|s| {
            s.push_str("Hello");
            s.push(' ');
            s.push_str("world");
            s.push('!');
        });

        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_multiple_string_builds() {
        let mut builder = StringBuilderOptimizer::with_capacity(50);

        let result1 = builder.build_string(|s| {
            s.push_str("first");
        });

        let result2 = builder.build_string(|s| {
            s.push_str("second");
            s.push_str(" string");
        });

        assert_eq!(result1, "first");
        assert_eq!(result2, "second string");
    }

    #[test]
    fn test_string_builder_reuse() {
        let mut builder = StringBuilderOptimizer::with_capacity(20);

        // Build multiple strings to test buffer reuse
        for i in 0..5 {
            let result = builder.build_string(|s| {
                s.push_str("test");
                s.push_str(&i.to_string());
            });
            assert_eq!(result, format!("test{i}"));
        }
    }

    #[test]
    fn test_string_builder_capacity_growth() {
        let mut builder = StringBuilderOptimizer::with_capacity(5); // Small capacity

        let result = builder.build_string(|s| {
            s.push_str("this is a much longer string than the initial capacity");
        });

        assert_eq!(
            result,
            "this is a much longer string than the initial capacity"
        );
    }

    #[test]
    fn test_empty_string_building() {
        let mut builder = StringBuilderOptimizer::with_capacity(10);

        let result = builder.build_string(|_s| {
            // Don't add anything
        });

        assert_eq!(result, "");
    }

    #[test]
    fn test_string_builder_with_numbers() {
        let mut builder = StringBuilderOptimizer::with_capacity(50);

        let result = builder.build_string(|s| {
            for i in 0..5 {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&i.to_string());
            }
        });

        assert_eq!(result, "0,1,2,3,4");
    }

    #[test]
    fn test_zero_capacity_string_builder() {
        let mut builder = StringBuilderOptimizer::with_capacity(0);

        let result = builder.build_string(|s| {
            s.push_str("test");
        });

        assert_eq!(result, "test");
    }
}

#[cfg(test)]
mod performance_scenarios_tests {
    use super::*;

    use std::time::Instant;

    #[test]
    fn test_high_volume_request_recording() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);

        let start = Instant::now();
        for i in 0..1000 {
            optimizer.record_request(Duration::from_millis(i % 100));
        }
        let elapsed = start.elapsed();

        // Should complete quickly
        assert!(elapsed < Duration::from_millis(100));

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 1000);
    }

    #[test]
    fn test_memory_efficiency() {
        let config = create_test_config();
        let mut optimizer = CommunicationOptimizer::new(config);

        // Record many allocation savings
        for _ in 0..10000 {
            optimizer.record_allocation_saved();
        }

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.allocations_saved, 10000);
    }

    #[test]
    fn test_string_builder_performance() {
        let mut builder = StringBuilderOptimizer::with_capacity(1000);

        let start = Instant::now();
        for i in 0..100 {
            let _result = builder.build_string(|s| {
                s.push_str("performance_test_");
                s.push_str(&i.to_string());
                s.push_str("_with_longer_content");
            });
        }
        let elapsed = start.elapsed();

        // Should complete quickly
        assert!(elapsed < Duration::from_millis(50));
    }

    #[test]
    fn test_batch_size_optimization_patterns() {
        let config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
        };
        let optimizer = CommunicationOptimizer::new(config);

        // Test different load patterns
        let test_cases = vec![
            (1, 1),  // Single request
            (3, 3),  // Small batch
            (5, 5),  // Max batch size
            (10, 5), // Over limit
            (0, 0),  // No requests
        ];

        for (pending, expected) in test_cases {
            let optimal = optimizer.get_optimal_batch_size(pending);
            assert_eq!(optimal, expected, "Failed for pending: {pending}");
        }
    }

    #[test]
    fn test_latency_based_batch_adjustment() {
        let config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
        };
        let mut optimizer = CommunicationOptimizer::new(config);

        // Normal latency should allow full batching
        optimizer.record_request(Duration::from_millis(100));
        let normal_batch = optimizer.get_optimal_batch_size(10);

        // High latency should reduce batching
        optimizer.record_request(Duration::from_millis(600));
        let reduced_batch = optimizer.get_optimal_batch_size(10);

        assert!(reduced_batch <= normal_batch);
    }

    #[test]
    fn test_edge_case_configurations() {
        // Test with extreme configurations
        let extreme_config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 1000,
            batch_timeout: Duration::from_secs(10),
        };

        let optimizer = CommunicationOptimizer::new(extreme_config);

        assert!(optimizer.should_batch_requests(500));
        assert_eq!(optimizer.get_optimal_batch_size(2000), 1000);
    }

    #[test]
    fn test_metrics_consistency() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);

        let mut total_requests = 0;
        let mut total_allocations = 0;

        for i in 0..50 {
            optimizer.record_request(Duration::from_millis(i * 10));
            total_requests += 1;

            if i % 2 == 0 {
                optimizer.record_allocation_saved();
                total_allocations += 1;
            }
        }

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, total_requests);
        assert_eq!(metrics.allocations_saved, total_allocations);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_optimization_workflow() {
        let config = PerformanceConfig::default();
        let mut optimizer = CommunicationOptimizer::new(config);
        let mut string_builder = StringBuilderOptimizer::with_capacity(100);

        // Simulate a complete request processing workflow
        for i in 0..10 {
            // Check if we should batch this request
            let should_batch = optimizer.should_batch_requests(i % 3);

            // Get optimal batch size
            let batch_size = optimizer.get_optimal_batch_size(i % 6);

            // Build a request string
            let request = string_builder.build_string(|s| {
                s.push_str("request_");
                s.push_str(&i.to_string());
                s.push_str("_batch_");
                s.push_str(&batch_size.to_string());
            });

            // Record the request processing
            optimizer.record_request(Duration::from_millis(50 + i as u64));

            if should_batch {
                optimizer.record_allocation_saved();
            }

            assert!(request.contains(&i.to_string()));
            assert!(request.contains(&batch_size.to_string()));
        }

        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_requests, 10);
        assert!(metrics.avg_response_time > Duration::from_millis(50));
    }

    #[test]
    fn test_optimizer_state_isolation() {
        let config1 = PerformanceConfig::default();
        let config2 = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: false,
            max_batch_size: 1,
            batch_timeout: Duration::from_millis(1),
        };

        let mut optimizer1 = CommunicationOptimizer::new(config1);
        let mut optimizer2 = CommunicationOptimizer::new(config2);

        // Record different patterns in each optimizer
        optimizer1.record_request(Duration::from_millis(100));
        optimizer1.record_allocation_saved();

        optimizer2.record_request(Duration::from_millis(200));

        // Verify isolation
        let metrics1 = optimizer1.get_metrics();
        let metrics2 = optimizer2.get_metrics();

        assert_eq!(metrics1.total_requests, 1);
        assert_eq!(metrics1.allocations_saved, 1);
        assert_eq!(metrics1.avg_response_time, Duration::from_millis(100));

        assert_eq!(metrics2.total_requests, 1);
        assert_eq!(metrics2.allocations_saved, 0);
        assert_eq!(metrics2.avg_response_time, Duration::from_millis(200));
    }

    #[test]
    fn test_configuration_impact_on_behavior() {
        let high_perf_config = PerformanceConfig {
            enable_connection_reuse: true,
            enable_request_batching: true,
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(500),
        };
        let minimal_config = PerformanceConfig {
            enable_connection_reuse: false,
            enable_request_batching: false,
            max_batch_size: 1,
            batch_timeout: Duration::from_millis(1),
        };

        let high_perf_optimizer = CommunicationOptimizer::new(high_perf_config);
        let minimal_optimizer = CommunicationOptimizer::new(minimal_config);

        // Test batching behavior differences
        assert!(high_perf_optimizer.should_batch_requests(50));
        assert!(!minimal_optimizer.should_batch_requests(50));

        assert_eq!(high_perf_optimizer.get_optimal_batch_size(50), 50);
        assert_eq!(minimal_optimizer.get_optimal_batch_size(50), 1);
    }
}
