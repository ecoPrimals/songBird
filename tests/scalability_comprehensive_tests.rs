//! Comprehensive tests for scalability module
//!
//! This test suite provides extensive coverage for auto-scaling, performance optimization,
//! and resource management functionality in the Songbird orchestrator.

use std::time::{Duration, Instant};
use songbird_lib::scalability::*;
use songbird_lib::errors::SongbirdError;

/// Test helper to create test resource pools
fn create_test_resource_pool() -> ResourcePool {
    ResourcePool {
        total_cpu_cores: 16,
        total_memory_mb: 32768,
        available_cpu_cores: 8,
        available_memory_mb: 16384,
    }
}

/// Test helper to create test resource usage
fn create_test_resource_usage(cpu: f64, memory: f64) -> ResourceUsage {
    ResourceUsage {
        cpu_percent: cpu,
        memory_percent: memory,
        network_io_mbps: 100.0,
        disk_io_mbps: 50.0,
    }
}

/// Test helper to create test performance metrics
fn create_test_performance_metrics() -> PerformanceMetrics {
    PerformanceMetrics {
        average_response_time: 150.0,
        request_rate: 1000.0,
        error_rate: 0.02,
        timeout_rate: 0.01,
        connection_pool_utilization: 0.75,
        cache_hit_rate: 0.85,
        memory_usage_percent: 60.0,
        cpu_usage_percent: 70.0,
    }
}

#[cfg(test)]
mod struct_creation_tests {
    use super::*;

    #[test]
    fn test_service_scaling_config_creation() {
        let config = ServiceScalingConfig {
            min_instances: 2,
            max_instances: 20,
            target_cpu_percent: 75.0,
            target_memory_percent: 85.0,
            scale_up_threshold: 85.0,
            scale_down_threshold: 25.0,
        };
        
        assert_eq!(config.min_instances, 2);
        assert_eq!(config.max_instances, 20);
        assert_eq!(config.target_cpu_percent, 75.0);
        assert_eq!(config.target_memory_percent, 85.0);
        assert_eq!(config.scale_up_threshold, 85.0);
        assert_eq!(config.scale_down_threshold, 25.0);
    }

    #[test]
    fn test_service_scaling_config_default() {
        let config = ServiceScalingConfig::default();
        
        assert_eq!(config.min_instances, 1);
        assert_eq!(config.max_instances, 10);
        assert_eq!(config.target_cpu_percent, 70.0);
        assert_eq!(config.target_memory_percent, 80.0);
        assert_eq!(config.scale_up_threshold, 80.0);
        assert_eq!(config.scale_down_threshold, 30.0);
    }

    #[test]
    fn test_scalability_stats_creation() {
        let stats = ScalabilityStats {
            total_scale_events: 100,
            scale_up_events: 60,
            scale_down_events: 40,
            average_response_time: 120.5,
            current_load: 0.75,
            resource_utilization: ResourceUsage::default(),
        };
        
        assert_eq!(stats.total_scale_events, 100);
        assert_eq!(stats.scale_up_events, 60);
        assert_eq!(stats.scale_down_events, 40);
        assert_eq!(stats.average_response_time, 120.5);
        assert_eq!(stats.current_load, 0.75);
    }

    #[test]
    fn test_resource_pool_creation() {
        let pool = create_test_resource_pool();
        
        assert_eq!(pool.total_cpu_cores, 16);
        assert_eq!(pool.total_memory_mb, 32768);
        assert_eq!(pool.available_cpu_cores, 8);
        assert_eq!(pool.available_memory_mb, 16384);
    }

    #[test]
    fn test_resource_usage_creation() {
        let usage = create_test_resource_usage(80.0, 70.0);
        
        assert_eq!(usage.cpu_percent, 80.0);
        assert_eq!(usage.memory_percent, 70.0);
        assert_eq!(usage.network_io_mbps, 100.0);
        assert_eq!(usage.disk_io_mbps, 50.0);
    }

    #[test]
    fn test_resource_usage_default() {
        let usage = ResourceUsage::default();
        
        assert_eq!(usage.cpu_percent, 0.0);
        assert_eq!(usage.memory_percent, 0.0);
        assert_eq!(usage.network_io_mbps, 0.0);
        assert_eq!(usage.disk_io_mbps, 0.0);
    }

    #[test]
    fn test_resource_config_creation() {
        let config = ResourceConfig {
            cpu_request: 1.0,
            memory_request_mb: 1024,
            cpu_limit: 2.0,
            memory_limit_mb: 2048,
        };
        
        assert_eq!(config.cpu_request, 1.0);
        assert_eq!(config.memory_request_mb, 1024);
        assert_eq!(config.cpu_limit, 2.0);
        assert_eq!(config.memory_limit_mb, 2048);
    }

    #[test]
    fn test_resource_config_default() {
        let config = ResourceConfig::default();
        
        assert_eq!(config.cpu_request, 0.5);
        assert_eq!(config.memory_request_mb, 512);
        assert_eq!(config.cpu_limit, 1.0);
        assert_eq!(config.memory_limit_mb, 1024);
    }

    #[test]
    fn test_performance_config_creation() {
        let config = PerformanceConfig {
            max_concurrent_requests: 500,
            request_timeout_ms: 60000,
            connection_pool_size: 50,
            cache_size_mb: 256,
        };
        
        assert_eq!(config.max_concurrent_requests, 500);
        assert_eq!(config.request_timeout_ms, 60000);
        assert_eq!(config.connection_pool_size, 50);
        assert_eq!(config.cache_size_mb, 256);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.request_timeout_ms, 30000);
        assert_eq!(config.connection_pool_size, 10);
        assert_eq!(config.cache_size_mb, 128);
    }

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = create_test_performance_metrics();
        
        assert_eq!(metrics.average_response_time, 150.0);
        assert_eq!(metrics.request_rate, 1000.0);
        assert_eq!(metrics.error_rate, 0.02);
        assert_eq!(metrics.timeout_rate, 0.01);
        assert_eq!(metrics.connection_pool_utilization, 0.75);
        assert_eq!(metrics.cache_hit_rate, 0.85);
        assert_eq!(metrics.memory_usage_percent, 60.0);
        assert_eq!(metrics.cpu_usage_percent, 70.0);
    }
}

#[cfg(test)]
mod auto_scaler_tests {
    use super::*;

    #[test]
    fn test_auto_scaler_creation() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let scaler = AutoScaler::new(config, resource_pool);
        
        assert_eq!(scaler.get_stats().total_scale_events, 0);
        assert_eq!(scaler.get_stats().scale_up_events, 0);
        assert_eq!(scaler.get_stats().scale_down_events, 0);
        assert_eq!(scaler.get_resource_pool().total_cpu_cores, 16);
        assert_eq!(scaler.get_scaling_history().len(), 0);
    }

    #[test]
    fn test_evaluate_scaling_scale_up_cpu() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(90.0, 60.0); // High CPU
        let result = scaler.evaluate_scaling("test-service", 2, &usage, 1000.0);
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected ScaleUp decision"),
        }
    }

    #[test]
    fn test_evaluate_scaling_scale_up_memory() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(60.0, 90.0); // High memory
        let result = scaler.evaluate_scaling("test-service", 2, &usage, 1000.0);
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected ScaleUp decision"),
        }
    }

    #[test]
    fn test_evaluate_scaling_scale_down() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(20.0, 25.0); // Low resource usage
        let result = scaler.evaluate_scaling("test-service", 5, &usage, 100.0);
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleDown(instances) => {
                assert_eq!(instances, 1);
            }
            _ => panic!("Expected ScaleDown decision"),
        }
    }

    #[test]
    fn test_evaluate_scaling_no_action() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(50.0, 60.0); // Normal usage
        let result = scaler.evaluate_scaling("test-service", 3, &usage, 500.0);
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::NoAction => {
                // Expected result
            }
            _ => panic!("Expected NoAction decision"),
        }
    }

    #[test]
    fn test_evaluate_scaling_max_instances_limit() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(95.0, 90.0); // Very high usage
        let result = scaler.evaluate_scaling("test-service", 10, &usage, 2000.0); // At max instances
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::NoAction => {
                // Expected - can't scale beyond max
            }
            _ => panic!("Expected NoAction due to max instances limit"),
        }
    }

    #[test]
    fn test_evaluate_scaling_min_instances_limit() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(10.0, 15.0); // Very low usage
        let result = scaler.evaluate_scaling("test-service", 1, &usage, 10.0); // At min instances
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::NoAction => {
                // Expected - can't scale below min
            }
            _ => panic!("Expected NoAction due to min instances limit"),
        }
    }

    #[tokio::test]
    async fn test_execute_scaling_scale_up() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let decision = ScalingDecision::ScaleUp(2);
        let result = scaler.execute_scaling("test-service", &decision).await;
        
        assert!(result.is_ok());
        assert_eq!(scaler.get_stats().scale_up_events, 1);
        assert_eq!(scaler.get_stats().total_scale_events, 1);
        assert_eq!(scaler.get_scaling_history().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_scaling_scale_down() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let decision = ScalingDecision::ScaleDown(1);
        let result = scaler.execute_scaling("test-service", &decision).await;
        
        assert!(result.is_ok());
        assert_eq!(scaler.get_stats().scale_down_events, 1);
        assert_eq!(scaler.get_stats().total_scale_events, 1);
        assert_eq!(scaler.get_scaling_history().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_scaling_no_action() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let decision = ScalingDecision::NoAction;
        let result = scaler.execute_scaling("test-service", &decision).await;
        
        assert!(result.is_ok());
        assert_eq!(scaler.get_stats().scale_up_events, 0);
        assert_eq!(scaler.get_stats().scale_down_events, 0);
        assert_eq!(scaler.get_stats().total_scale_events, 0);
        assert_eq!(scaler.get_scaling_history().len(), 0);
    }

    #[test]
    fn test_cooldown_period_functionality() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        // Set a short cooldown period for testing
        scaler.set_cooldown_period(Duration::from_millis(100));
        
        let usage = create_test_resource_usage(90.0, 60.0);
        
        // First evaluation should allow scaling
        let result1 = scaler.evaluate_scaling("test-service", 2, &usage, 1000.0);
        assert!(result1.is_ok());
        match result1.unwrap() {
            ScalingDecision::ScaleUp(_) => {
                // Expected
            }
            _ => panic!("Expected ScaleUp decision"),
        }
        
        // Simulate that scaling just happened
        scaler.set_last_scaling_time(Some(Instant::now()));
        
        // Second evaluation should be blocked by cooldown
        let result2 = scaler.evaluate_scaling("test-service", 2, &usage, 1000.0);
        assert!(result2.is_ok());
        match result2.unwrap() {
            ScalingDecision::NoAction => {
                // Expected due to cooldown
            }
            _ => panic!("Expected NoAction due to cooldown"),
        }
    }

    #[test]
    fn test_update_resource_pool() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let new_pool = ResourcePool {
            total_cpu_cores: 32,
            total_memory_mb: 65536,
            available_cpu_cores: 16,
            available_memory_mb: 32768,
        };
        
        scaler.update_resource_pool(new_pool);
        
        assert_eq!(scaler.get_resource_pool().total_cpu_cores, 32);
        assert_eq!(scaler.get_resource_pool().total_memory_mb, 65536);
        assert_eq!(scaler.get_resource_pool().available_cpu_cores, 16);
        assert_eq!(scaler.get_resource_pool().available_memory_mb, 32768);
    }
}

#[cfg(test)]
mod performance_optimizer_tests {
    use super::*;

    #[test]
    fn test_performance_optimizer_creation() {
        let config = PerformanceConfig::default();
        let optimizer = PerformanceOptimizer::new(config);
        
        assert_eq!(optimizer.get_optimization_history().len(), 0);
    }

    #[test]
    fn test_optimize_performance_high_response_time() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        let mut metrics = create_test_performance_metrics();
        metrics.average_response_time = 500.0; // High response time
        metrics.connection_pool_utilization = 0.95; // High utilization
        
        let result = optimizer.optimize_performance("test-service", &metrics);
        
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert!(!recommendations.is_empty());
        
        // Should recommend increasing connection pool size
        let pool_recommendation = recommendations.iter()
            .find(|r| matches!(r.optimization_type, OptimizationType::ConnectionPoolSize));
        assert!(pool_recommendation.is_some());
    }

    #[test]
    fn test_optimize_performance_low_cache_hit_rate() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        let mut metrics = create_test_performance_metrics();
        metrics.cache_hit_rate = 0.4; // Low cache hit rate
        
        let result = optimizer.optimize_performance("test-service", &metrics);
        
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert!(!recommendations.is_empty());
        
        // Should recommend increasing cache size
        let cache_recommendation = recommendations.iter()
            .find(|r| matches!(r.optimization_type, OptimizationType::CacheSize));
        assert!(cache_recommendation.is_some());
    }

    #[test]
    fn test_optimize_performance_high_timeout_rate() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        let mut metrics = create_test_performance_metrics();
        metrics.timeout_rate = 0.1; // High timeout rate
        
        let result = optimizer.optimize_performance("test-service", &metrics);
        
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert!(!recommendations.is_empty());
        
        // Should recommend increasing request timeout
        let timeout_recommendation = recommendations.iter()
            .find(|r| matches!(r.optimization_type, OptimizationType::RequestTimeout));
        assert!(timeout_recommendation.is_some());
    }

    #[test]
    fn test_optimize_performance_no_recommendations() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        let metrics = PerformanceMetrics {
            average_response_time: 50.0,  // Good response time
            request_rate: 500.0,
            error_rate: 0.001,            // Low error rate
            timeout_rate: 0.001,          // Low timeout rate
            connection_pool_utilization: 0.5, // Normal utilization
            cache_hit_rate: 0.95,         // High cache hit rate
            memory_usage_percent: 40.0,   // Low memory usage
            cpu_usage_percent: 30.0,      // Low CPU usage
        };
        
        let result = optimizer.optimize_performance("test-service", &metrics);
        
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_optimization_types_enum() {
        // Test that all optimization types can be created
        let _connection_pool = OptimizationType::ConnectionPoolSize;
        let _cache_size = OptimizationType::CacheSize;
        let _request_timeout = OptimizationType::RequestTimeout;
        let _concurrent_requests = OptimizationType::ConcurrentRequests;
        
        // Test Debug trait
        let opt_type = OptimizationType::ConnectionPoolSize;
        let debug_string = format!("{:?}", opt_type);
        assert!(debug_string.contains("ConnectionPoolSize"));
    }

    #[test]
    fn test_optimization_recommendation_creation() {
        let recommendation = OptimizationRecommendation {
            optimization_type: OptimizationType::ConnectionPoolSize,
            current_value: 10.0,
            recommended_value: 20.0,
            expected_improvement: 25.0,
            reason: "High connection pool utilization".to_string(),
        };
        
        assert!(matches!(recommendation.optimization_type, OptimizationType::ConnectionPoolSize));
        assert_eq!(recommendation.current_value, 10.0);
        assert_eq!(recommendation.recommended_value, 20.0);
        assert_eq!(recommendation.expected_improvement, 25.0);
        assert_eq!(recommendation.reason, "High connection pool utilization");
    }
}

#[cfg(test)]
mod scaling_decision_tests {
    use super::*;

    #[test]
    fn test_scaling_decision_scale_up() {
        let decision = ScalingDecision::ScaleUp(5);
        match decision {
            ScalingDecision::ScaleUp(instances) => {
                assert_eq!(instances, 5);
            }
            _ => panic!("Expected ScaleUp decision"),
        }
    }

    #[test]
    fn test_scaling_decision_scale_down() {
        let decision = ScalingDecision::ScaleDown(2);
        match decision {
            ScalingDecision::ScaleDown(instances) => {
                assert_eq!(instances, 2);
            }
            _ => panic!("Expected ScaleDown decision"),
        }
    }

    #[test]
    fn test_scaling_decision_no_action() {
        let decision = ScalingDecision::NoAction;
        match decision {
            ScalingDecision::NoAction => {
                // Expected
            }
            _ => panic!("Expected NoAction decision"),
        }
    }

    #[test]
    fn test_scaling_decision_debug() {
        let decision = ScalingDecision::ScaleUp(3);
        let debug_string = format!("{:?}", decision);
        assert!(debug_string.contains("ScaleUp"));
        assert!(debug_string.contains("3"));
    }
}

#[cfg(test)]
mod scaling_event_tests {
    use super::*;

    #[test]
    fn test_scaling_event_creation() {
        let event = ScalingEvent {
            timestamp: Instant::now(),
            service_id: "test-service".to_string(),
            decision: ScalingDecision::ScaleUp(2),
            reason: "High CPU usage".to_string(),
            current_instances: 3,
            target_instances: 5,
        };
        
        assert_eq!(event.service_id, "test-service");
        assert_eq!(event.reason, "High CPU usage");
        assert_eq!(event.current_instances, 3);
        assert_eq!(event.target_instances, 5);
        
        match event.decision {
            ScalingDecision::ScaleUp(instances) => {
                assert_eq!(instances, 2);
            }
            _ => panic!("Expected ScaleUp decision"),
        }
    }

    #[test]
    fn test_scaling_event_debug() {
        let event = ScalingEvent {
            timestamp: Instant::now(),
            service_id: "test-service".to_string(),
            decision: ScalingDecision::ScaleDown(1),
            reason: "Low resource usage".to_string(),
            current_instances: 5,
            target_instances: 4,
        };
        
        let debug_string = format!("{:?}", event);
        assert!(debug_string.contains("test-service"));
        assert!(debug_string.contains("ScaleDown"));
        assert!(debug_string.contains("Low resource usage"));
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_zero_instances_scaling() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(90.0, 60.0);
        let result = scaler.evaluate_scaling("test-service", 0, &usage, 1000.0);
        
        assert!(result.is_ok());
        // Should scale up from 0 instances
        match result.unwrap() {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected ScaleUp decision"),
        }
    }

    #[test]
    fn test_extreme_resource_usage() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(100.0, 100.0); // Maximum usage
        let result = scaler.evaluate_scaling("test-service", 1, &usage, 10000.0);
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected ScaleUp decision with extreme usage"),
        }
    }

    #[test]
    fn test_negative_resource_usage() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = ResourceUsage {
            cpu_percent: -10.0,  // Invalid negative value
            memory_percent: -5.0, // Invalid negative value
            network_io_mbps: 0.0,
            disk_io_mbps: 0.0,
        };
        
        let result = scaler.evaluate_scaling("test-service", 3, &usage, 500.0);
        
        assert!(result.is_ok());
        // Should handle negative values gracefully
        match result.unwrap() {
            ScalingDecision::ScaleDown(_) | ScalingDecision::NoAction => {
                // Expected - negative values should be treated as very low usage
            }
            _ => panic!("Unexpected decision with negative usage"),
        }
    }

    #[test]
    fn test_very_high_request_rate() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(85.0, 60.0);
        let result = scaler.evaluate_scaling("test-service", 5, &usage, 1000000.0); // Very high request rate
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected ScaleUp decision with high request rate"),
        }
    }

    #[test]
    fn test_zero_request_rate() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(20.0, 15.0);
        let result = scaler.evaluate_scaling("test-service", 5, &usage, 0.0); // Zero request rate
        
        assert!(result.is_ok());
        match result.unwrap() {
            ScalingDecision::ScaleDown(_) => {
                // Expected - zero request rate should trigger scale down
            }
            _ => panic!("Expected ScaleDown decision with zero request rate"),
        }
    }

    #[test]
    fn test_empty_service_id() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(50.0, 60.0);
        let result = scaler.evaluate_scaling("", 3, &usage, 500.0); // Empty service ID
        
        assert!(result.is_ok());
        // Should handle empty service ID gracefully
    }

    #[test]
    fn test_custom_scaling_config_extreme_values() {
        let config = ServiceScalingConfig {
            min_instances: 0,
            max_instances: 1000,
            target_cpu_percent: 99.0,
            target_memory_percent: 99.0,
            scale_up_threshold: 99.5,
            scale_down_threshold: 1.0,
        };
        
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        let usage = create_test_resource_usage(50.0, 60.0);
        let result = scaler.evaluate_scaling("test-service", 10, &usage, 500.0);
        
        assert!(result.is_ok());
        // Should handle extreme config values
    }

    #[test]
    fn test_performance_optimizer_extreme_metrics() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        let metrics = PerformanceMetrics {
            average_response_time: 10000.0,  // Very high response time
            request_rate: 0.1,               // Very low request rate
            error_rate: 0.5,                 // Very high error rate
            timeout_rate: 0.5,               // Very high timeout rate
            connection_pool_utilization: 1.0, // Max utilization
            cache_hit_rate: 0.0,             // Zero cache hit rate
            memory_usage_percent: 100.0,     // Max memory usage
            cpu_usage_percent: 100.0,        // Max CPU usage
        };
        
        let result = optimizer.optimize_performance("test-service", &metrics);
        
        assert!(result.is_ok());
        let recommendations = result.unwrap();
        assert!(!recommendations.is_empty());
        
        // Should generate multiple recommendations for extreme metrics
        assert!(recommendations.len() >= 2);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_scaling_workflow() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        // Step 1: Evaluate scaling decision
        let usage = create_test_resource_usage(90.0, 60.0);
        let decision = scaler.evaluate_scaling("test-service", 2, &usage, 1000.0).unwrap();
        
        // Step 2: Execute scaling decision
        let execute_result = scaler.execute_scaling("test-service", &decision).await;
        assert!(execute_result.is_ok());
        
        // Step 3: Verify statistics were updated
        let stats = scaler.get_stats();
        assert!(stats.total_scale_events > 0);
        
        // Step 4: Verify scaling history
        let history = scaler.get_scaling_history();
        assert!(!history.is_empty());
        assert_eq!(history[0].service_id, "test-service");
    }

    #[tokio::test]
    async fn test_multiple_scaling_decisions() {
        let config = ServiceScalingConfig::default();
        let resource_pool = create_test_resource_pool();
        let mut scaler = AutoScaler::new(config, resource_pool);
        
        // Set short cooldown for testing
        scaler.set_cooldown_period(Duration::from_millis(1));
        
        // Execute multiple scaling decisions
        let services = vec!["service-1", "service-2", "service-3"];
        let mut total_events = 0;
        
        for service in services {
            let usage = create_test_resource_usage(90.0, 60.0);
            let decision = scaler.evaluate_scaling(service, 2, &usage, 1000.0).unwrap();
            
            match decision {
                ScalingDecision::ScaleUp(_) => {
                    let result = scaler.execute_scaling(service, &decision).await;
                    assert!(result.is_ok());
                    total_events += 1;
                }
                _ => {}
            }
            
            // Small delay to avoid cooldown
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        // Verify all events were recorded
        assert_eq!(scaler.get_stats().total_scale_events, total_events);
        assert_eq!(scaler.get_scaling_history().len(), total_events as usize);
    }

    #[test]
    fn test_performance_optimization_workflow() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);
        
        // Test with different metric scenarios
        let scenarios = vec![
            ("high-latency", create_performance_metrics_high_latency()),
            ("low-cache", create_performance_metrics_low_cache()),
            ("high-timeout", create_performance_metrics_high_timeout()),
        ];
        
        for (service_id, metrics) in scenarios {
            let result = optimizer.optimize_performance(service_id, &metrics);
            assert!(result.is_ok());
            
            let recommendations = result.unwrap();
            if !recommendations.is_empty() {
                // Verify recommendation structure
                for rec in &recommendations {
                    assert!(rec.current_value >= 0.0);
                    assert!(rec.recommended_value >= 0.0);
                    assert!(rec.expected_improvement >= 0.0);
                    assert!(!rec.reason.is_empty());
                }
            }
        }
    }
}

// Helper functions for integration tests
fn create_performance_metrics_high_latency() -> PerformanceMetrics {
    PerformanceMetrics {
        average_response_time: 500.0,
        request_rate: 1000.0,
        error_rate: 0.02,
        timeout_rate: 0.01,
        connection_pool_utilization: 0.95,
        cache_hit_rate: 0.85,
        memory_usage_percent: 70.0,
        cpu_usage_percent: 80.0,
    }
}

fn create_performance_metrics_low_cache() -> PerformanceMetrics {
    PerformanceMetrics {
        average_response_time: 200.0,
        request_rate: 800.0,
        error_rate: 0.01,
        timeout_rate: 0.005,
        connection_pool_utilization: 0.6,
        cache_hit_rate: 0.3,
        memory_usage_percent: 60.0,
        cpu_usage_percent: 65.0,
    }
}

fn create_performance_metrics_high_timeout() -> PerformanceMetrics {
    PerformanceMetrics {
        average_response_time: 300.0,
        request_rate: 600.0,
        error_rate: 0.03,
        timeout_rate: 0.15,
        connection_pool_utilization: 0.7,
        cache_hit_rate: 0.8,
        memory_usage_percent: 65.0,
        cpu_usage_percent: 70.0,
    }
} 