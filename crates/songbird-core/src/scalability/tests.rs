//! Tests for Scalability Components
//!
//! Comprehensive test suite for scalability functionality

#[cfg(test)]
mod tests {
    use crate::scalability::{
        autoscaler::AutoScaler, manager::ScalabilityManager, optimizer::PerformanceOptimizer,
        types::*,
    };
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_scalability_manager_creation() {
        let config = ScalabilityConfig::default();
        let manager = ScalabilityManager::new(config);

        assert!(manager.get_current_instances().await.is_empty());
        assert!(manager.is_healthy().await);
        assert_eq!(manager.current_instance_count(), 0);
    }

    #[tokio::test]
    async fn test_auto_scaling_up() {
        let config = ScalabilityConfig {
            min_instances: 1,
            max_instances: 10,
            target_cpu_threshold: 70.0,
            target_memory_threshold: 80.0,
            scale_up_cooldown: Duration::from_secs(60),
            scale_down_cooldown: Duration::from_secs(120),
            ..Default::default()
        };

        let mut manager = ScalabilityManager::new(config);

        // Simulate high CPU usage that should trigger scaling
        let metrics = ResourceUsage {
            cpu_percent: 85.0,
            memory_percent: 75.0,
            network_io_mbps: 50.0,
            disk_io_mbps: 20.0,
        };

        manager.add_metrics(metrics).await;

        let scaling_action = manager.evaluate_scaling().await.unwrap();
        assert_eq!(scaling_action.action_type, ScalingActionType::ScaleUp);
        assert_eq!(scaling_action.target_instances, 1);

        // Apply scaling action
        let result = manager.execute_scaling_action(scaling_action).await;
        assert!(result.is_ok());
        assert_eq!(manager.current_instance_count(), 1);
    }

    #[tokio::test]
    async fn test_auto_scaling_down() {
        let config = ScalabilityConfig {
            min_instances: 1,
            max_instances: 10,
            target_cpu_threshold: 70.0,
            target_memory_threshold: 80.0,
            ..Default::default()
        };

        let mut manager = ScalabilityManager::new(config);

        // Start with some instances
        let high_usage = ResourceUsage {
            cpu_percent: 85.0,
            memory_percent: 75.0,
            network_io_mbps: 50.0,
            disk_io_mbps: 20.0,
        };
        manager.add_metrics(high_usage).await;
        let scale_up = manager.evaluate_scaling().await.unwrap();
        manager.execute_scaling_action(scale_up).await.unwrap();

        // Now simulate low usage that should trigger scale down
        let low_usage = ResourceUsage {
            cpu_percent: 20.0,
            memory_percent: 25.0,
            network_io_mbps: 10.0,
            disk_io_mbps: 5.0,
        };

        manager.add_metrics(low_usage).await;

        let scaling_action = manager.evaluate_scaling().await.unwrap();
        assert_eq!(scaling_action.action_type, ScalingActionType::ScaleDown);

        let result = manager.execute_scaling_action(scaling_action).await;
        assert!(result.is_ok());
        assert_eq!(manager.current_instance_count(), 0);
    }

    #[tokio::test]
    async fn test_predictive_scaling() {
        let config = ScalabilityConfig {
            enable_predictive_scaling: true,
            ..Default::default()
        };

        let manager = ScalabilityManager::new(config);

        // Test prediction with empty history
        let prediction = manager
            .predict_future_load(Duration::from_secs(15 * 60))
            .await;
        assert!(prediction.is_ok());
        assert_eq!(prediction.unwrap(), 0.0);
    }

    #[tokio::test]
    async fn test_prediction_with_history() {
        let config = ScalabilityConfig {
            enable_predictive_scaling: true,
            ..Default::default()
        };

        let mut manager = ScalabilityManager::new(config);

        // Add historical metrics
        manager
            .add_metrics(ResourceUsage {
                cpu_percent: 60.0,
                memory_percent: 50.0,
                network_io_mbps: 30.0,
                disk_io_mbps: 15.0,
            })
            .await;

        manager
            .add_metrics(ResourceUsage {
                cpu_percent: 70.0,
                memory_percent: 55.0,
                network_io_mbps: 35.0,
                disk_io_mbps: 18.0,
            })
            .await;

        let prediction = manager
            .predict_future_load(Duration::from_secs(15 * 60))
            .await
            .unwrap();

        assert!(prediction >= 0.0 && prediction <= 100.0);
    }

    #[tokio::test]
    async fn test_autoscaler_creation() {
        let config = ServiceScalingConfig::default();
        let resource_pool = ResourcePool {
            total_cpu_cores: 8,
            total_memory_mb: 16384,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        };

        let autoscaler = AutoScaler::new(config, resource_pool);
        assert!(autoscaler.get_stats().total_scale_events == 0);
        assert!(autoscaler.get_scaling_history().is_empty());
    }

    #[tokio::test]
    async fn test_autoscaler_evaluation() {
        let config = ServiceScalingConfig {
            min_instances: 1,
            max_instances: 5,
            scale_up_threshold: 80.0,
            scale_down_threshold: 30.0,
            ..Default::default()
        };

        let resource_pool = ResourcePool {
            total_cpu_cores: 8,
            total_memory_mb: 16384,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        };

        let mut autoscaler = AutoScaler::new(config, resource_pool);

        let high_usage = ResourceUsage {
            cpu_percent: 90.0,
            memory_percent: 85.0,
            network_io_mbps: 100.0,
            disk_io_mbps: 50.0,
        };

        let decision = autoscaler
            .evaluate_scaling("test-service", 2, &high_usage, 500.0)
            .unwrap();

        match decision {
            ScalingDecision::ScaleUp(instances) => {
                assert!(instances > 0);
            }
            _ => panic!("Expected scale up decision"),
        }
    }

    #[tokio::test]
    async fn test_autoscaler_execution() {
        let config = ServiceScalingConfig::default();
        let resource_pool = ResourcePool {
            total_cpu_cores: 8,
            total_memory_mb: 16384,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        };

        let mut autoscaler = AutoScaler::new(config, resource_pool);

        let scale_up_decision = ScalingDecision::ScaleUp(2);
        let result = autoscaler
            .execute_scaling("test-service", &scale_up_decision)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_performance_optimizer() {
        let config = PerformanceConfig::default();
        let mut optimizer = PerformanceOptimizer::new(config);

        let metrics = PerformanceMetrics {
            average_response_time: 150.0,
            request_rate: 500.0,
            error_rate: 1.0,
            timeout_rate: 2.0,
            connection_pool_utilization: 0.9, // High utilization
            cache_hit_rate: 0.6,              // Low hit rate
            memory_usage_percent: 60.0,
            cpu_usage_percent: 70.0,
        };

        let recommendations = optimizer
            .optimize_performance("test-service", &metrics)
            .unwrap();

        assert!(!recommendations.is_empty());
        assert!(optimizer.get_optimization_history().len() > 0);
    }

    #[tokio::test]
    async fn test_optimizer_connection_pool_scaling() {
        let config = PerformanceConfig {
            connection_pool_size: 10,
            ..Default::default()
        };
        let mut optimizer = PerformanceOptimizer::new(config);

        let high_utilization_metrics = PerformanceMetrics {
            connection_pool_utilization: 0.95,
            average_response_time: 100.0,
            request_rate: 1000.0,
            error_rate: 0.5,
            timeout_rate: 0.2,
            cache_hit_rate: 0.8,
            memory_usage_percent: 50.0,
            cpu_usage_percent: 60.0,
        };

        let recommendations = optimizer
            .optimize_performance("test-service", &high_utilization_metrics)
            .unwrap();

        let connection_pool_rec = recommendations
            .iter()
            .find(|r| matches!(r.optimization_type, OptimizationType::ConnectionPoolSize));

        assert!(connection_pool_rec.is_some());
        assert!(connection_pool_rec.unwrap().recommended_value > 10.0);
    }

    #[tokio::test]
    async fn test_optimizer_cache_sizing() {
        let config = PerformanceConfig {
            cache_size_mb: 64,
            ..Default::default()
        };
        let mut optimizer = PerformanceOptimizer::new(config);

        let low_hit_rate_metrics = PerformanceMetrics {
            cache_hit_rate: 0.5, // Low hit rate
            connection_pool_utilization: 0.6,
            average_response_time: 200.0,
            request_rate: 500.0,
            error_rate: 1.0,
            timeout_rate: 0.5,
            memory_usage_percent: 40.0,
            cpu_usage_percent: 50.0,
        };

        let recommendations = optimizer
            .optimize_performance("test-service", &low_hit_rate_metrics)
            .unwrap();

        let cache_rec = recommendations
            .iter()
            .find(|r| matches!(r.optimization_type, OptimizationType::CacheSize));

        assert!(cache_rec.is_some());
        assert!(cache_rec.unwrap().recommended_value > 64.0);
    }

    #[tokio::test]
    async fn test_scaling_cooldown() {
        let config = ServiceScalingConfig {
            min_instances: 1,
            max_instances: 5,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            ..Default::default()
        };

        let resource_pool = ResourcePool {
            total_cpu_cores: 8,
            total_memory_mb: 16384,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        };

        let mut autoscaler = AutoScaler::new(config, resource_pool);
        autoscaler.set_cooldown_period(Duration::from_millis(100));

        let high_usage = ResourceUsage {
            cpu_percent: 90.0,
            memory_percent: 85.0,
            network_io_mbps: 100.0,
            disk_io_mbps: 50.0,
        };

        // First scaling should work
        let decision1 = autoscaler
            .evaluate_scaling("test-service", 2, &high_usage, 500.0)
            .unwrap();
        assert!(matches!(decision1, ScalingDecision::ScaleUp(_)));

        // Immediate second scaling should be blocked by cooldown
        let decision2 = autoscaler
            .evaluate_scaling("test-service", 3, &high_usage, 500.0)
            .unwrap();
        assert!(matches!(decision2, ScalingDecision::NoAction));

        // After cooldown, scaling should work again
        sleep(Duration::from_millis(150)).await;
        let decision3 = autoscaler
            .evaluate_scaling("test-service", 3, &high_usage, 500.0)
            .unwrap();
        assert!(matches!(decision3, ScalingDecision::ScaleUp(_)));
    }

    #[tokio::test]
    async fn test_resource_constraints() {
        let config = ServiceScalingConfig::default();
        let resource_pool = ResourcePool {
            total_cpu_cores: 2,
            total_memory_mb: 1024,
            available_cpu_cores: 1,   // Limited resources
            available_memory_mb: 256, // Limited resources
        };

        let mut autoscaler = AutoScaler::new(config, resource_pool);

        let scale_up_decision = ScalingDecision::ScaleUp(5); // Try to scale up many instances
        let result = autoscaler
            .execute_scaling("test-service", &scale_up_decision)
            .await;

        // Should fail due to insufficient resources
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metrics_history_management() {
        let config = ScalabilityConfig::default();
        let mut manager = ScalabilityManager::new(config);

        // Add more than 100 metrics to test history trimming
        for i in 0..150 {
            let metrics = ResourceUsage {
                cpu_percent: (i % 100) as f64,
                memory_percent: 50.0,
                network_io_mbps: 30.0,
                disk_io_mbps: 15.0,
            };
            manager.add_metrics(metrics).await;
        }

        // History should be trimmed to 100 entries
        assert_eq!(manager.metrics_history().len(), 100);
    }

    // Helper functions for creating test data
    fn create_high_usage_metrics() -> ResourceUsage {
        ResourceUsage {
            cpu_percent: 90.0,
            memory_percent: 85.0,
            network_io_mbps: 100.0,
            disk_io_mbps: 50.0,
        }
    }

    fn create_low_usage_metrics() -> ResourceUsage {
        ResourceUsage {
            cpu_percent: 20.0,
            memory_percent: 25.0,
            network_io_mbps: 10.0,
            disk_io_mbps: 5.0,
        }
    }

    fn create_test_resource_pool() -> ResourcePool {
        ResourcePool {
            total_cpu_cores: 8,
            total_memory_mb: 16384,
            available_cpu_cores: 8,
            available_memory_mb: 16384,
        }
    }
}
