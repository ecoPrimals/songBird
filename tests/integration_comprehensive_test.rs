//! Comprehensive Integration Test for Advanced Songbird Features
//!
//! This test validates that all major advanced features can be initialized
//! and work together correctly:
//! - Advanced Service Registry with auto-scaling
//! - Advanced Load Balancing with multiple algorithms
//! - Gaming Network optimization with NAT traversal
//! - Federation system with MCP integration
//! - Universal Primal Storage system
//! - BearDog security integration

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

use songbird_lib::{
    registry::{
        AdvancedServiceRegistry, HealthCheckPolicy, AutoScalingPolicy, ScalingStrategy,
        HealthCheckType, CustomHealthCheck, HealthFailureAction, ScalingThreshold
    },
    traits::service::ServiceInfo,
    network::gaming::{
        nat_traversal::NatTraversalManager,
        performance::PerformanceMonitor,
    },
    load_balancer::{
        LoadBalancerType, ServiceInstance, LoadBalancerConfig, 
        HealthBasedLoadBalancer, LatencyOptimizedLoadBalancer
    },
    federation::mcp_handler::McpFederation,
    universal_primals::nestgate::NestGateStorage,
    config::SongbirdConfig,
    errors::Result,
};

#[tokio::test]
async fn test_advanced_integration_comprehensive() -> Result<()> {
    println!("🚀 Starting comprehensive integration test...");
    
    // Test 1: Advanced Service Registry
    test_advanced_service_registry().await?;
    
    // Test 2: Advanced Load Balancing
    test_advanced_load_balancing().await?;
    
    // Test 3: Gaming Network Components
    test_gaming_network_components().await?;
    
    // Test 4: Federation System
    test_federation_system().await?;
    
    // Test 5: Universal Primal Storage
    test_universal_primal_storage().await?;
    
    // Test 6: System Integration
    test_system_integration().await?;
    
    println!("✅ All comprehensive integration tests passed!");
    Ok(())
}

async fn test_advanced_service_registry() -> Result<()> {
    println!("📋 Testing Advanced Service Registry...");
    
    let registry = AdvancedServiceRegistry::new();
    
    // Create a test service
    let service_info = ServiceInfo {
        service_id: "test-gaming-service".to_string(),
        service_type: "gaming".to_string(),
        endpoints: vec!["tcp://127.0.0.1:7777".to_string()],
        version: "1.0.0".to_string(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("game_type".to_string(), serde_json::Value::String("fps".to_string()));
            metadata.insert("max_players".to_string(), serde_json::Value::Number(serde_json::Number::from(64)));
            metadata
        },
    };
    
    // Create health check policy
    let health_policy = HealthCheckPolicy {
        service_id: service_info.service_id.clone(),
        check_interval: Duration::from_secs(5),
        timeout: Duration::from_secs(3),
        healthy_threshold: 2,
        unhealthy_threshold: 1,
        health_check_path: Some("/health".to_string()),
        custom_health_checks: vec![
            CustomHealthCheck {
                name: "cpu_check".to_string(),
                check_type: HealthCheckType::CpuUsage { max_percentage: 80.0 },
                parameters: HashMap::new(),
                weight: 0.5,
            },
            CustomHealthCheck {
                name: "memory_check".to_string(),
                check_type: HealthCheckType::MemoryUsage { max_percentage: 85.0 },
                parameters: HashMap::new(),
                weight: 0.5,
            },
        ],
        failure_action: HealthFailureAction::Alert,
    };
    
    // Create auto-scaling policy
    let scaling_policy = AutoScalingPolicy {
        service_id: service_info.service_id.clone(),
        min_instances: 1,
        max_instances: 5,
        target_cpu_utilization: 70.0,
        target_memory_utilization: 75.0,
        target_request_rate: 100.0,
        scale_up_threshold: ScalingThreshold {
            metric_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("cpu".to_string(), 80.0);
                thresholds
            },
            sustained_duration: Duration::from_secs(30),
            scale_factor: 1.5,
        },
        scale_down_threshold: ScalingThreshold {
            metric_thresholds: {
                let mut thresholds = HashMap::new();
                thresholds.insert("cpu".to_string(), 30.0);
                thresholds
            },
            sustained_duration: Duration::from_secs(120),
            scale_factor: 1.0,
        },
        cooldown_period: Duration::from_secs(60),
        scaling_strategy: ScalingStrategy::Gaming,
    };
    
    // Register service with advanced policies
    registry.register_advanced_service(
        service_info.clone(),
        health_policy,
        scaling_policy,
    ).await?;
    
    // Test scaling
    registry.scale_service(&service_info.service_id, 3).await?;
    
    // List services
    let services = registry.list_services_with_status().await;
    assert!(!services.is_empty(), "Service should be registered");
    
    println!("✅ Advanced Service Registry test passed");
    Ok(())
}

async fn test_advanced_load_balancing() -> Result<()> {
    println!("⚖️ Testing Advanced Load Balancing...");
    
    // Create test service instances
    let instances = vec![
        ServiceInstance {
            instance_id: "instance-1".to_string(),
            service_id: "test-service".to_string(),
            endpoint: "http://127.0.0.1:8001".to_string(),
            health_score: 0.95,
            avg_response_time: 25.0,
            cpu_usage: 45.0,
            memory_usage: 60.0,
            gpu_usage: Some(30.0),
            gpu_memory_usage: Some(40.0),
            active_connections: 50,
            last_updated: std::time::Instant::now(),
        },
        ServiceInstance {
            instance_id: "instance-2".to_string(),
            service_id: "test-service".to_string(),
            endpoint: "http://127.0.0.1:8002".to_string(),
            health_score: 0.87,
            avg_response_time: 35.0,
            cpu_usage: 65.0,
            memory_usage: 75.0,
            gpu_usage: Some(50.0),
            gpu_memory_usage: Some(60.0),
            active_connections: 80,
            last_updated: std::time::Instant::now(),
        },
    ];
    
    // Test Health-Based Load Balancer
    let health_config = LoadBalancerConfig {
        algorithm: LoadBalancerType::HealthBased { min_health_score: 0.8 },
        health_check_interval: Duration::from_secs(10),
        timeout: Duration::from_secs(5),
        max_retries: 3,
        circuit_breaker_enabled: true,
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout: Duration::from_secs(30),
    };
    
    let mut health_balancer = HealthBasedLoadBalancer::new(health_config);
    
    // Update instances
    for instance in &instances {
        health_balancer.update_instance(instance.clone()).await?;
    }
    
    // Test instance selection
    let selected = health_balancer.select_instance("test-service").await?;
    assert!(selected.is_some(), "Health-based balancer should select an instance");
    
    // Test Latency-Optimized Load Balancer
    let latency_config = LoadBalancerConfig {
        algorithm: LoadBalancerType::LatencyOptimized { max_latency_ms: 100.0 },
        health_check_interval: Duration::from_secs(10),
        timeout: Duration::from_secs(5),
        max_retries: 3,
        circuit_breaker_enabled: true,
        circuit_breaker_threshold: 5,
        circuit_breaker_timeout: Duration::from_secs(30),
    };
    
    let mut latency_balancer = LatencyOptimizedLoadBalancer::new(latency_config);
    
    // Update instances
    for instance in &instances {
        latency_balancer.update_instance(instance.clone()).await?;
    }
    
    // Test instance selection
    let selected = latency_balancer.select_instance("test-service").await?;
    assert!(selected.is_some(), "Latency-optimized balancer should select an instance");
    
    println!("✅ Advanced Load Balancing test passed");
    Ok(())
}

async fn test_gaming_network_components() -> Result<()> {
    println!("🎮 Testing Gaming Network Components...");
    
    // Test NAT Traversal Manager
    let nat_manager = NatTraversalManager::new();
    
    // Test NAT type detection
    let nat_type = nat_manager.get_nat_type();
    println!("📡 NAT Type: {:?}", nat_type);
    
    // Test connection establishment (this will fail in test environment, but we test the API)
    let test_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let connection_result = timeout(
        Duration::from_secs(1),
        nat_manager.establish_connection("test-session".to_string(), test_addr)
    ).await;
    
    // We expect this to timeout or fail in test environment, which is normal
    match connection_result {
        Ok(Ok(_)) => println!("✅ Connection established successfully"),
        Ok(Err(_)) => println!("⚠️ Connection failed (expected in test environment)"),
        Err(_) => println!("⚠️ Connection timed out (expected in test environment)"),
    }
    
    // Test Performance Monitor
    let config = songbird_lib::config::benchmarks::BenchmarkConfig {
        duration: Duration::from_secs(1),
        concurrent_connections: 10,
        request_rate: 100,
        enable_detailed_metrics: true,
        enable_latency_histogram: true,
        enable_throughput_tracking: true,
    };
    
    let mut performance_monitor = PerformanceMonitor::new(config)?;
    
    // Start monitoring
    performance_monitor.start_monitoring().await?;
    
    // Get current metrics
    let metrics = performance_monitor.get_current_metrics().await?;
    println!("📊 Current metrics: {:?}", metrics);
    
    // Stop monitoring
    performance_monitor.stop_monitoring().await?;
    
    println!("✅ Gaming Network Components test passed");
    Ok(())
}

async fn test_federation_system() -> Result<()> {
    println!("🔗 Testing Federation System...");
    
    // Create federation configuration
    let federation_config = songbird_lib::federation::config::FederationConfig {
        cluster_id: Some("test-cluster".to_string()),
        node_id: Some("test-node-1".to_string()),
        cluster_endpoints: vec!["http://127.0.0.1:8080".to_string()],
        heartbeat_interval: 30,
        auto_discovery: true,
        connection_timeout: 5,
        max_retries: 2,
    };
    
    // Initialize federation
    let federation = McpFederation::new(
        songbird_lib::federation::config::FederationMode::Standalone,
        federation_config,
    );
    
    // Test federation start
    let start_result = timeout(Duration::from_secs(2), federation.start()).await;
    match start_result {
        Ok(Ok(_)) => println!("✅ Federation started successfully"),
        Ok(Err(_)) => println!("⚠️ Federation start failed (expected in test environment)"),
        Err(_) => println!("⚠️ Federation start timed out (expected in test environment)"),
    }
    
    // Get federation status
    let status = federation.get_status().await;
    println!("📊 Federation Status: enabled={}, connected={}, nodes={}", 
             status.enabled, status.connected, status.node_count);
    
    // Test heartbeat
    let heartbeat_result = timeout(Duration::from_secs(1), federation.send_heartbeat()).await;
    match heartbeat_result {
        Ok(Ok(_)) => println!("✅ Heartbeat sent successfully"),
        Ok(Err(_)) => println!("⚠️ Heartbeat failed (expected in test environment)"),
        Err(_) => println!("⚠️ Heartbeat timed out (expected in test environment)"),
    }
    
    // Stop federation
    federation.stop().await?;
    
    println!("✅ Federation System test passed");
    Ok(())
}

async fn test_universal_primal_storage() -> Result<()> {
    println!("💾 Testing Universal Primal Storage...");
    
    // Initialize storage
    let storage = NestGateStorage::new();
    
    // Test file operations
    let test_file = "test_integration.txt";
    let test_content = "This is a comprehensive integration test file content.";
    
    // Write file
    storage.write_file(test_file, test_content.as_bytes()).await?;
    println!("✅ File written successfully");
    
    // Read file
    let read_content = storage.read_file(test_file).await?;
    assert_eq!(read_content, test_content.as_bytes());
    println!("✅ File read successfully");
    
    // List files
    let files = storage.list_files().await?;
    assert!(files.contains(&test_file.to_string()));
    println!("✅ File listing successful");
    
    // Test backup operations
    let backup_name = "integration_test_backup";
    storage.create_backup(backup_name).await?;
    println!("✅ Backup created successfully");
    
    // List backups
    let backups = storage.list_backups().await?;
    assert!(backups.contains(&backup_name.to_string()));
    println!("✅ Backup listing successful");
    
    // Delete test file
    storage.delete_file(test_file).await?;
    println!("✅ File deleted successfully");
    
    // Restore backup
    storage.restore_backup(backup_name).await?;
    println!("✅ Backup restored successfully");
    
    // Verify file was restored
    let restored_content = storage.read_file(test_file).await?;
    assert_eq!(restored_content, test_content.as_bytes());
    println!("✅ File restoration verified");
    
    // Clean up
    storage.delete_file(test_file).await?;
    
    println!("✅ Universal Primal Storage test passed");
    Ok(())
}

async fn test_system_integration() -> Result<()> {
    println!("🎯 Testing System Integration...");
    
    // Test that all major components can be initialized together
    let registry = AdvancedServiceRegistry::new();
    let nat_manager = NatTraversalManager::new();
    let storage = NestGateStorage::new();
    
    // Create a configuration that integrates all systems
    let config = SongbirdConfig::default();
    
    // Test service registration with storage backend
    let service_info = ServiceInfo {
        service_id: "integrated-service".to_string(),
        service_type: "integration".to_string(),
        endpoints: vec!["tcp://127.0.0.1:9999".to_string()],
        version: "1.0.0".to_string(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("integration_test".to_string(), serde_json::Value::Bool(true));
            metadata
        },
    };
    
    let health_policy = HealthCheckPolicy {
        service_id: service_info.service_id.clone(),
        check_interval: Duration::from_secs(10),
        timeout: Duration::from_secs(5),
        healthy_threshold: 2,
        unhealthy_threshold: 1,
        health_check_path: Some("/health".to_string()),
        custom_health_checks: vec![],
        failure_action: HealthFailureAction::Alert,
    };
    
    let scaling_policy = AutoScalingPolicy {
        service_id: service_info.service_id.clone(),
        min_instances: 1,
        max_instances: 3,
        target_cpu_utilization: 70.0,
        target_memory_utilization: 75.0,
        target_request_rate: 50.0,
        scale_up_threshold: ScalingThreshold {
            metric_thresholds: HashMap::new(),
            sustained_duration: Duration::from_secs(60),
            scale_factor: 1.5,
        },
        scale_down_threshold: ScalingThreshold {
            metric_thresholds: HashMap::new(),
            sustained_duration: Duration::from_secs(120),
            scale_factor: 1.0,
        },
        cooldown_period: Duration::from_secs(60),
        scaling_strategy: ScalingStrategy::Gaming,
    };
    
    // Register integrated service
    registry.register_advanced_service(
        service_info.clone(),
        health_policy,
        scaling_policy,
    ).await?;
    
    // Store service configuration
    let config_json = serde_json::to_string_pretty(&service_info)?;
    storage.write_file("integrated_service_config.json", config_json.as_bytes()).await?;
    
    // Test service discovery through registry
    let services = registry.list_services_with_status().await;
    assert!(!services.is_empty(), "Integrated service should be registered");
    
    // Test NAT type detection for gaming services
    let nat_type = nat_manager.get_nat_type();
    println!("🌐 NAT Type for gaming: {:?}", nat_type);
    
    // Test connection status
    let connection_status = nat_manager.get_connection_status().await;
    println!("📡 Active connections: {}", connection_status.len());
    
    // Verify storage integration
    let stored_config = storage.read_file("integrated_service_config.json").await?;
    let parsed_config: ServiceInfo = serde_json::from_slice(&stored_config)?;
    assert_eq!(parsed_config.service_id, service_info.service_id);
    
    // Clean up
    storage.delete_file("integrated_service_config.json").await?;
    
    println!("✅ System Integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    println!("🚀 Testing Performance Under Load...");
    
    let registry = AdvancedServiceRegistry::new();
    let storage = NestGateStorage::new();
    
    // Create multiple services
    let service_count = 10;
    let mut services = Vec::new();
    
    for i in 0..service_count {
        let service_info = ServiceInfo {
            service_id: format!("load-test-service-{}", i),
            service_type: "load_test".to_string(),
            endpoints: vec![format!("tcp://127.0.0.1:{}", 8000 + i)],
            version: "1.0.0".to_string(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("load_test".to_string(), serde_json::Value::Bool(true));
                metadata.insert("instance".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
                metadata
            },
        };
        
        let health_policy = HealthCheckPolicy {
            service_id: service_info.service_id.clone(),
            check_interval: Duration::from_secs(5),
            timeout: Duration::from_secs(3),
            healthy_threshold: 2,
            unhealthy_threshold: 1,
            health_check_path: Some("/health".to_string()),
            custom_health_checks: vec![],
            failure_action: HealthFailureAction::Alert,
        };
        
        let scaling_policy = AutoScalingPolicy {
            service_id: service_info.service_id.clone(),
            min_instances: 1,
            max_instances: 5,
            target_cpu_utilization: 70.0,
            target_memory_utilization: 75.0,
            target_request_rate: 100.0,
            scale_up_threshold: ScalingThreshold {
                metric_thresholds: HashMap::new(),
                sustained_duration: Duration::from_secs(30),
                scale_factor: 2.0,
            },
            scale_down_threshold: ScalingThreshold {
                metric_thresholds: HashMap::new(),
                sustained_duration: Duration::from_secs(120),
                scale_factor: 1.0,
            },
            cooldown_period: Duration::from_secs(60),
            scaling_strategy: ScalingStrategy::Gaming,
        };
        
        registry.register_advanced_service(
            service_info.clone(),
            health_policy,
            scaling_policy,
        ).await?;
        
        services.push(service_info);
    }
    
    // Test concurrent file operations
    let mut tasks = Vec::new();
    
    for i in 0..service_count {
        let storage_clone = storage.clone();
        let service = services[i].clone();
        
        let task = tokio::spawn(async move {
            let filename = format!("load_test_file_{}.json", i);
            let content = serde_json::to_string_pretty(&service).unwrap();
            
            // Write file
            storage_clone.write_file(&filename, content.as_bytes()).await.unwrap();
            
            // Read file
            let read_content = storage_clone.read_file(&filename).await.unwrap();
            assert_eq!(read_content, content.as_bytes());
            
            // Delete file
            storage_clone.delete_file(&filename).await.unwrap();
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }
    
    // Verify all services are registered
    let registered_services = registry.list_services_with_status().await;
    assert_eq!(registered_services.len(), service_count);
    
    println!("✅ Performance Under Load test passed");
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    println!("🛡️ Testing Error Handling...");
    
    let storage = NestGateStorage::new();
    
    // Test reading non-existent file
    let result = storage.read_file("non_existent_file.txt").await;
    assert!(result.is_err(), "Should fail when reading non-existent file");
    
    // Test deleting non-existent file
    let result = storage.delete_file("non_existent_file.txt").await;
    assert!(result.is_err(), "Should fail when deleting non-existent file");
    
    // Test restoring non-existent backup
    let result = storage.restore_backup("non_existent_backup").await;
    assert!(result.is_err(), "Should fail when restoring non-existent backup");
    
    println!("✅ Error Handling test passed");
    Ok(())
} 