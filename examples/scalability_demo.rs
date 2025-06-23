/*!
 * Scalability Demo - Songbird Orchestrator
 *
 * Demonstrates advanced scalability and performance optimization features:
 * - Auto-scaling based on CPU, memory, and request rate thresholds
 * - Load balancing with multiple algorithms
 * - Performance monitoring and metrics collection
 * - Resource management and allocation
 * - Scaling decision evaluation and execution
 * - Comprehensive scalability statistics
 */

use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use songbird_orchestrator::{
    prelude::*,
    scalability_types::{
        PerformanceConfig, PerformanceMetrics, PerformanceThresholds, ResourceConfig, 
        ResourcePool, ResourceUsage, ScalabilityConfig, ScalabilityManager, ScalingAction, 
        ScalingDecision, ScalingStrategy, ServiceScalingConfig,
    },
};

// Import ScalingGroup, LoadBalancingConfig, and LoadBalancingAlgorithm from scalability module
use songbird_orchestrator::scalability::{
    LoadBalancingAlgorithm, LoadBalancingConfig, ScalingGroup,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,songbird_orchestrator=debug")
        .init();

    println!("📈 Songbird Orchestrator - Scalability Demo");
    println!("============================================");

    // Demo 1: Scalability Configuration
    println!("\n📋 Demo 1: Scalability Configuration");
    demo_scalability_configuration().await?;

    // Demo 2: Service Instance Management
    println!("\n📋 Demo 2: Service Instance Management");
    demo_service_instance_management().await?;

    // Demo 3: Auto-Scaling Evaluation
    println!("\n📋 Demo 3: Auto-Scaling Evaluation");
    demo_auto_scaling_evaluation().await?;

    // Demo 4: Performance Monitoring
    println!("\n📋 Demo 4: Performance Monitoring");
    demo_performance_monitoring().await?;

    // Demo 5: Resource Management
    println!("\n📋 Demo 5: Resource Management");
    demo_resource_management().await?;

    // Demo 6: Scalability Statistics
    println!("\n📋 Demo 6: Scalability Statistics");
    demo_scalability_statistics().await?;

    println!("\n✅ All scalability demos completed successfully!");
    Ok(())
}

/// Demonstrate scalability configuration options
async fn demo_scalability_configuration() -> Result<()> {
    println!("Testing scalability configuration options...");

    // High-performance configuration
    let config = ScalabilityConfig {
        enabled: true,
        strategy: ScalingStrategy::Automatic,
        performance_config: PerformanceConfig {
            response_time_threshold_ms: 1000,
            throughput_threshold_rps: 100,
            error_rate_threshold: 0.05,
            monitoring_interval: Duration::from_secs(30),
        },
        resource_config: ResourceConfig {
            cpu_request: 1.0,
            cpu_limit: 4.0,
            memory_request_mb: 512,
            memory_limit_mb: 2048,
            disk_limit_mb: 10240,
        },
        load_balancing_config: LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(10),
            session_affinity: false,
        },
        thresholds: PerformanceThresholds {
            max_response_time_ms: 2000,
            min_throughput_rps: 50,
            max_error_rate: 0.1,
            max_cpu_utilization: 0.8,
            max_memory_utilization: 0.85,
        },
    };

    println!("Scalability Configuration:");
    println!("  - Enabled: {}", config.enabled);
    println!("  - Strategy: {:?}", config.strategy);
    println!("  - CPU Limit: {} cores", config.resource_config.cpu_limit);
    println!(
        "  - Memory Limit: {} MB",
        config.resource_config.memory_limit_mb
    );
    println!(
        "  - Load Balancing: {:?}",
        config.load_balancing_config.algorithm
    );
    println!(
        "  - Max CPU Threshold: {:.1}%",
        config.thresholds.max_cpu_utilization * 100.0
    );
    println!(
        "  - Response Time Threshold: {} ms",
        config.thresholds.max_response_time_ms
    );

    println!("✓ Scalability configuration demo completed");
    Ok(())
}

/// Demonstrate service instance management
async fn demo_service_instance_management() -> Result<()> {
    println!("Testing service instance management...");

    let config = ScalabilityConfig {
        enabled: true,
        strategy: ScalingStrategy::Automatic,
        performance_config: PerformanceConfig {
            response_time_threshold_ms: 1000,
            throughput_threshold_rps: 100,
            error_rate_threshold: 0.05,
            monitoring_interval: Duration::from_secs(30),
        },
        resource_config: ResourceConfig {
            cpu_request: 1.0,
            cpu_limit: 4.0,
            memory_request_mb: 512,
            memory_limit_mb: 2048,
            disk_limit_mb: 10240,
        },
        load_balancing_config: LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(10),
            session_affinity: false,
        },
        thresholds: PerformanceThresholds {
            max_response_time_ms: 2000,
            min_throughput_rps: 50,
            max_error_rate: 0.1,
            max_cpu_utilization: 0.8,
            max_memory_utilization: 0.85,
        },
    };

    let mut scalability_manager = ScalabilityManager::new(config);

    // Create mock service instances using the actual ServiceInstance structure
    let services = ["web-api", "auth-service", "database"];
    let service_id = services[0];

    // Create service info for the instance
    let _service_info = ServiceInfo {
        id: service_id.to_string(),
        name: format!("{} Instance", service_id),
        version: "1.0.0".to_string(),
        service_type: "web".to_string(),
        description: "Mock service instance".to_string(),
        endpoints: vec![],
        capabilities: vec!["http".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };

    // Create a scaling group for the service
    let scaling_config = ServiceScalingConfig {
        min_instances: 1,
        max_instances: 10,
        target_cpu_utilization: 70.0,
        scale_up_threshold: 80.0,
        scale_down_threshold: 30.0,
        cooldown_period: Duration::from_secs(300),
    };

    let _scaling_group = ScalingGroup::new("demo-service".to_string(), scaling_config);
    scalability_manager
        .add_scaling_group(service_id.to_string(), _scaling_group)
        .await?;

    println!("Created scaling group for service: {}", service_id);
    println!("  - Min Instances: 1");
    println!("  - Max Instances: 10");
    println!("  - CPU Target: 70%");

    println!("✓ Service instance management demo completed");
    Ok(())
}

/// Demonstrate auto-scaling evaluation
async fn demo_auto_scaling_evaluation() -> Result<()> {
    println!("Testing auto-scaling evaluation...");

    let config = ScalabilityConfig {
        enabled: true,
        strategy: ScalingStrategy::Automatic,
        performance_config: PerformanceConfig {
            response_time_threshold_ms: 1000,
            throughput_threshold_rps: 100,
            error_rate_threshold: 0.05,
            monitoring_interval: Duration::from_secs(30),
        },
        resource_config: ResourceConfig {
            cpu_request: 1.0,
            cpu_limit: 4.0,
            memory_request_mb: 512,
            memory_limit_mb: 2048,
            disk_limit_mb: 10240,
        },
        load_balancing_config: LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(10),
            session_affinity: false,
        },
        thresholds: PerformanceThresholds {
            max_response_time_ms: 2000,
            min_throughput_rps: 50,
            max_error_rate: 0.1,
            max_cpu_utilization: 0.8,
            max_memory_utilization: 0.85,
        },
    };

    let _scalability_manager = ScalabilityManager::new(config);

    // Simulate scaling decisions with different performance metrics
    let metrics_scenarios = vec![
        (
            "Low Load",
            PerformanceMetrics {
                avg_response_time_ms: 150,
                throughput_rps: 25,
                error_rate: 0.01,
                cpu_utilization: 0.3,
                memory_utilization: 0.4,
                timestamp: Utc::now(),
            },
        ),
        (
            "Normal Load",
            PerformanceMetrics {
                avg_response_time_ms: 400,
                throughput_rps: 75,
                error_rate: 0.02,
                cpu_utilization: 0.65,
                memory_utilization: 0.6,
                timestamp: Utc::now(),
            },
        ),
        (
            "High Load",
            PerformanceMetrics {
                avg_response_time_ms: 800,
                throughput_rps: 150,
                error_rate: 0.03,
                cpu_utilization: 0.85,
                memory_utilization: 0.8,
                timestamp: Utc::now(),
            },
        ),
    ];

    // Create a scaling group with some instances
    let scaling_config = ServiceScalingConfig {
        min_instances: 2,
        max_instances: 10,
        target_cpu_utilization: 70.0,
        scale_up_threshold: 80.0,
        scale_down_threshold: 30.0,
        cooldown_period: Duration::from_secs(300),
    };

    let _scaling_group = ScalingGroup::new("demo-service".to_string(), scaling_config);

    for (scenario, metrics) in metrics_scenarios {
        // Note: We can't call make_scaling_decision without adding the group first,
        // but we can simulate the logic
        let cpu_high = metrics.cpu_utilization > 0.8;
        let cpu_low = metrics.cpu_utilization < 0.3;

        let decision = if cpu_high {
            ScalingDecision {
                action: ScalingAction::ScaleUp,
                target_instances: 5,
                reason: format!("High CPU: {:.1}%", metrics.cpu_utilization * 100.0),
                timestamp: Utc::now(),
            }
        } else if cpu_low {
            ScalingDecision {
                action: ScalingAction::ScaleDown,
                target_instances: 2,
                reason: format!("Low CPU: {:.1}%", metrics.cpu_utilization * 100.0),
                timestamp: Utc::now(),
            }
        } else {
            ScalingDecision {
                action: ScalingAction::NoAction,
                target_instances: 3,
                reason: format!("Stable CPU: {:.1}%", metrics.cpu_utilization * 100.0),
                timestamp: Utc::now(),
            }
        };

        println!("Scenario: {}", scenario);
        println!("  - CPU: {:.1}%", metrics.cpu_utilization * 100.0);
        println!("  - Memory: {:.1}%", metrics.memory_utilization * 100.0);
        println!("  - Response Time: {} ms", metrics.avg_response_time_ms);
        println!("  - Decision: {:?}", decision.action);
        println!("  - Target Instances: {}", decision.target_instances);
        println!("  - Reason: {}", decision.reason);
        println!();
    }

    println!("✓ Auto-scaling evaluation demo completed");
    Ok(())
}

/// Demonstrate performance monitoring
async fn demo_performance_monitoring() -> Result<()> {
    println!("Testing performance monitoring...");

    // Simulate collecting performance metrics over time
    let mut metrics_history = Vec::new();

    for i in 0..5 {
        let metrics = PerformanceMetrics {
            avg_response_time_ms: 200 + (i * 50),
            throughput_rps: 50 + (i * 25),
            error_rate: 0.01 + (i as f64 * 0.005),
            cpu_utilization: 0.4 + (i as f64 * 0.1),
            memory_utilization: 0.3 + (i as f64 * 0.15),
            timestamp: Utc::now(),
        };

        metrics_history.push(metrics.clone());

        println!("Metrics Collection #{}", i + 1);
        println!("  - Response Time: {} ms", metrics.avg_response_time_ms);
        println!("  - Throughput: {} rps", metrics.throughput_rps);
        println!("  - Error Rate: {:.1}%", metrics.error_rate * 100.0);
        println!("  - CPU: {:.1}%", metrics.cpu_utilization * 100.0);
        println!("  - Memory: {:.1}%", metrics.memory_utilization * 100.0);

        // Check thresholds
        if metrics.cpu_utilization > 0.75 {
            println!(
                "    🚨 CPU Alert: {:.1}% exceeds threshold of 75.0%",
                metrics.cpu_utilization * 100.0
            );
        }
        if metrics.avg_response_time_ms > 300 {
            println!(
                "    ⚠️  Response Time Alert: {} ms exceeds threshold of 300 ms",
                metrics.avg_response_time_ms
            );
        }
        println!();

        sleep(Duration::from_millis(100)).await;
    }

    println!("✓ Performance monitoring demo completed");
    Ok(())
}

/// Demonstrate resource management
async fn demo_resource_management() -> Result<()> {
    println!("Testing resource management...");

    // Create resource pools
    let mut frontend_pool = ResourcePool {
        max_cpu_cores: 16,
        max_memory_mb: 32768,
        available_cpu_cores: 14,
        available_memory_mb: 28672,
        allocated_instances: HashMap::new(),
    };

    let backend_pool = ResourcePool {
        max_cpu_cores: 32,
        max_memory_mb: 65536,
        available_cpu_cores: 24,
        available_memory_mb: 49152,
        allocated_instances: HashMap::new(),
    };

    println!("Resource Pools:");
    println!("Frontend Pool:");
    println!("  - Max CPU: {} cores", frontend_pool.max_cpu_cores);
    println!("  - Max Memory: {} MB", frontend_pool.max_memory_mb);
    println!(
        "  - Available CPU: {} cores",
        frontend_pool.available_cpu_cores
    );
    println!(
        "  - Available Memory: {} MB",
        frontend_pool.available_memory_mb
    );

    println!("Backend Pool:");
    println!("  - Max CPU: {} cores", backend_pool.max_cpu_cores);
    println!("  - Max Memory: {} MB", backend_pool.max_memory_mb);
    println!(
        "  - Available CPU: {} cores",
        backend_pool.available_cpu_cores
    );
    println!(
        "  - Available Memory: {} MB",
        backend_pool.available_memory_mb
    );

    // Simulate resource allocation
    let resource_usage = ResourceUsage {
        cpu_percentage: 50.0,
        memory_usage_mb: 1024,
        disk_io_bytes_per_sec: 5368709120,
        network_bytes_per_sec: 104857600,
    };

    frontend_pool
        .allocated_instances
        .insert("web-api-1".to_string(), resource_usage.clone());

    println!("\nResource Allocation:");
    println!("Allocated to web-api-1:");
    println!("  - CPU: {}%", resource_usage.cpu_percentage);
    println!("  - Memory: {} MB", resource_usage.memory_usage_mb);
    println!("  - Disk I/O: {} bytes/sec", resource_usage.disk_io_bytes_per_sec);
    println!("  - Network: {} bytes/sec", resource_usage.network_bytes_per_sec);

    // Calculate utilization
    let cpu_utilization = (frontend_pool.max_cpu_cores - frontend_pool.available_cpu_cores) as f64
        / frontend_pool.max_cpu_cores as f64
        * 100.0;
    let memory_utilization = (frontend_pool.max_memory_mb - frontend_pool.available_memory_mb)
        as f64
        / frontend_pool.max_memory_mb as f64
        * 100.0;

    println!("\nPool Utilization:");
    println!("  - CPU Utilization: {:.1}%", cpu_utilization);
    println!("  - Memory Utilization: {:.1}%", memory_utilization);

    println!("✓ Resource management demo completed");
    Ok(())
}

/// Demonstrate scalability statistics
async fn demo_scalability_statistics() -> Result<()> {
    println!("Testing scalability statistics...");

    let config = ScalabilityConfig {
        enabled: true,
        strategy: ScalingStrategy::Automatic,
        performance_config: PerformanceConfig {
            response_time_threshold_ms: 1000,
            throughput_threshold_rps: 100,
            error_rate_threshold: 0.05,
            monitoring_interval: Duration::from_secs(30),
        },
        resource_config: ResourceConfig {
            cpu_request: 1.0,
            cpu_limit: 4.0,
            memory_request_mb: 512,
            memory_limit_mb: 2048,
            disk_limit_mb: 10240,
        },
        load_balancing_config: LoadBalancingConfig {
            algorithm: LoadBalancingAlgorithm::LeastConnections,
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(10),
            session_affinity: false,
        },
        thresholds: PerformanceThresholds {
            max_response_time_ms: 2000,
            min_throughput_rps: 50,
            max_error_rate: 0.1,
            max_cpu_utilization: 0.8,
            max_memory_utilization: 0.85,
        },
    };

    let scalability_manager = ScalabilityManager::new(config);

    // Get current statistics
    let stats = scalability_manager.get_stats().await?;

    println!("Scalability Statistics:");
    println!("  📊 Total Instances: {}", stats.total_instances);
    println!("  ✅ Healthy Instances: {}", stats.healthy_instances);
    println!(
        "  ❌ Unhealthy Instances: {}",
        stats.total_instances - stats.healthy_instances
    );
    println!(
        "  🔥 Average CPU Usage: {:.1}%",
        stats.avg_cpu_utilization * 100.0
    );
    println!(
        "  💾 Average Memory Usage: {:.1}%",
        stats.avg_memory_utilization * 100.0
    );
    println!("  📈 Total Requests: {}", stats.total_requests);
    println!("  ✅ Successful Requests: {}", stats.successful_requests);
    println!("  ❌ Failed Requests: {}", stats.failed_requests);

    // Calculate derived statistics
    let health_percentage = if stats.total_instances > 0 {
        (stats.healthy_instances as f64 / stats.total_instances as f64) * 100.0
    } else {
        100.0
    };

    let success_rate = if stats.total_requests > 0 {
        (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
    } else {
        100.0
    };

    println!("\nDerived Statistics:");
    println!("  💚 Health Percentage: {:.1}%", health_percentage);
    println!("  📈 Success Rate: {:.1}%", success_rate);

    // Performance assessment
    let resource_status = if stats.avg_cpu_utilization > 0.8 || stats.avg_memory_utilization > 0.8 {
        "🔴 High Load - Consider Scaling Up"
    } else if stats.avg_cpu_utilization > 0.6 || stats.avg_memory_utilization > 0.6 {
        "🟡 Moderate Load - Monitor Closely"
    } else {
        "🟢 Normal Load - System Healthy"
    };

    println!("\nSystem Assessment:");
    println!("  {}", resource_status);

    // Recommendations
    if stats.avg_cpu_utilization > 0.75 {
        println!("  💡 Recommendation: Add more CPU resources or scale horizontally");
    }
    if stats.avg_memory_utilization > 0.75 {
        println!("  💡 Recommendation: Increase memory allocation or optimize memory usage");
    }
    if health_percentage < 90.0 {
        println!("  💡 Recommendation: Investigate unhealthy instances");
    }
    if success_rate < 95.0 {
        println!("  💡 Recommendation: Review error logs and improve error handling");
    }

    println!("✓ Scalability statistics demo completed");
    Ok(())
}
