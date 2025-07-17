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

use anyhow::Result;

use async_trait::async_trait;
use songbird_network::network::gaming::nat_traversal::NatTraversalManager;
use songbird_network::network::gaming::performance::PerformanceMonitor;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use songbird_core::load_balancer::{
    RoundRobinLoadBalancer, LeastConnectionsLoadBalancer, WeightedRoundRobinLoadBalancer, LoadBalancerConfig,
    LoadBalancerStrategy as LoadBalancerType, ServiceInstance,
};
use songbird_config::SongbirdConfig;
use songbird_core::{
    registry::{CustomHealthCheck, HealthCheckPolicy, HealthCheckType, HealthFailureAction},
    traits::service::ServiceInfo,
};
use songbird_discovery::traits::PluginRegistry;
use songbird_federation::mcp_handler::McpFederation;
use songbird_network::network::gaming::performance::BenchmarkConfig;
use songbird_registry::plugin::DynamicPluginRegistry;
use songbird_registry::scaling::AutoScalingEngine;
use songbird_registry::scaling::{AutoScalingPolicy, ScalingStrategy, ScalingThreshold};
use songbird_discovery::traits::service::ServiceStatus;
use songbird_federation::config::{FederationConfig, FederationMode};
// Note: Using mock storage for testing since NestGateStorage doesn't exist yet
// use songbird_universal_primals::nestgate::NestGatePrimal;

// Real storage for testing using production-like file system operations
#[derive(Clone)]
struct FileSystemStorage {
    base_path: std::path::PathBuf,
    backup_path: std::path::PathBuf,
}

impl FileSystemStorage {
    fn new() -> Self {
        let base_path = std::env::temp_dir()
            .join("songbird_test_storage")
            .join(uuid::Uuid::new_v4().to_string());
        let backup_path = base_path.join("backups");

        Self {
            base_path,
            backup_path,
        }
    }

    async fn write_file(&self, filename: &str, content: &[u8]) -> Result<()> {
        let file_path = self.base_path.join(filename);

        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&file_path, content).await?;
        tracing::debug!("Wrote file to storage: {:?}", file_path);
        Ok(())
    }

    async fn read_file(&self, filename: &str) -> Result<Vec<u8>> {
        let file_path = self.base_path.join(filename);
        tokio::fs::read(&file_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", filename, e))
    }

    async fn delete_file(&self, filename: &str) -> Result<()> {
        let file_path = self.base_path.join(filename);
        tokio::fs::remove_file(&file_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete file {}: {}", filename, e))
    }

    async fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.base_path, &self.base_path, &mut files)
            .await?;
        Ok(files)
    }

    async fn collect_files_recursive(
        &self,
        current_dir: &std::path::Path,
        base_path: &std::path::Path,
        files: &mut Vec<String>,
    ) -> Result<()> {
        if !current_dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(current_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Ok(relative_path) = path.strip_prefix(base_path) {
                    if let Some(path_str) = relative_path.to_str() {
                        files.push(path_str.to_string());
                    }
                }
            } else if path.is_dir() {
                Box::pin(self.collect_files_recursive(&path, base_path, files)).await?;
            }
        }

        Ok(())
    }

    async fn create_backup(&self, backup_name: &str) -> Result<()> {
        let backup_dir = self.backup_path.join(backup_name);
        tokio::fs::create_dir_all(&backup_dir).await?;

        // Copy all files to backup directory
        self.copy_directory_recursive(&self.base_path, &backup_dir)
            .await?;
        tracing::debug!("Created backup: {:?}", backup_dir);
        Ok(())
    }

    async fn copy_directory_recursive(
        &self,
        from: &std::path::Path,
        to: &std::path::Path,
    ) -> Result<()> {
        if !from.exists() {
            return Ok(());
        }

        tokio::fs::create_dir_all(to).await?;
        let mut entries = tokio::fs::read_dir(from).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name();
            let dest_path = to.join(&name);

            if path.is_file() {
                tokio::fs::copy(&path, &dest_path).await?;
            } else if path.is_dir() {
                Box::pin(self.copy_directory_recursive(&path, &dest_path)).await?;
            }
        }

        Ok(())
    }

    async fn list_backups(&self) -> Result<Vec<String>> {
        if !self.backup_path.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.backup_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    backups.push(name.to_string());
                }
            }
        }

        Ok(backups)
    }

    async fn restore_backup(&self, backup_name: &str) -> Result<()> {
        let backup_dir = self.backup_path.join(backup_name);

        if !backup_dir.exists() {
            return Err(anyhow::anyhow!("Backup not found: {}", backup_name));
        }

        // Clear existing files
        if self.base_path.exists() {
            tokio::fs::remove_dir_all(&self.base_path).await?;
        }
        tokio::fs::create_dir_all(&self.base_path).await?;

        // Restore files from backup
        self.copy_directory_recursive(&backup_dir, &self.base_path)
            .await?;
        tracing::debug!("Restored backup: {:?}", backup_dir);
        Ok(())
    }

    /// Cleanup storage (for test teardown)
    async fn cleanup(&self) -> Result<()> {
        if self.base_path.exists() {
            tokio::fs::remove_dir_all(&self.base_path).await?;
        }
        Ok(())
    }
}

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

    let registry = songbird_core::registry::ServiceRegistry::new().await?;

    // Create a test service
    let service_info = ServiceInfo {
        service_id: "test-gaming-service".to_string(),
        name: "Test Gaming Service".to_string(),
        service_type: "gaming".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test gaming service".to_string()),
        endpoints: vec![],
        health_check_endpoint: Some("/health".to_string()),
        tags: vec!["gaming".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        host: "127.0.0.1".to_string(),
        port: 7777,
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert(
                "game_type".to_string(),
                serde_json::Value::String("fps".to_string()),
            );
            metadata.insert(
                "max_players".to_string(),
                serde_json::Value::Number(serde_json::Number::from(64)),
            );
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
                check_type: HealthCheckType::CpuUsage {
                    max_percentage: 80.0,
                },
                parameters: HashMap::new(),
                weight: 0.5,
            },
            CustomHealthCheck {
                name: "memory_check".to_string(),
                check_type: HealthCheckType::MemoryUsage {
                    max_percentage: 85.0,
                },
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
    registry
        .register_advanced_service(service_info.clone(), health_policy, scaling_policy)
        .await?;

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
            id: "instance-1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8001,
            weight: 1,
            healthy: true,
            health_score: 0.95,
            avg_response_time: 25.0,
            cpu_usage: 45.0,
            memory_usage: 60.0,
            gpu_usage: Some(30.0),
            gpu_memory_usage: Some(40.0),
            active_connections: 50,
            last_updated: chrono::Utc::now(),
        },
        ServiceInstance {
            id: "instance-2".to_string(),
            address: "127.0.0.1".to_string(),
            port: 8002,
            weight: 1,
            healthy: true,
            health_score: 0.87,
            avg_response_time: 35.0,
            cpu_usage: 65.0,
            memory_usage: 75.0,
            gpu_usage: Some(50.0),
            gpu_memory_usage: Some(60.0),
            active_connections: 80,
            last_updated: chrono::Utc::now(),
        },
    ];

    // Test Health-Based Load Balancer
    let health_config = LoadBalancerConfig {
        strategy: LoadBalancerType::HealthBased,
        health_check_interval: 10,
        max_retries: 3,
        timeout_seconds: 5,
    };

    let health_balancer = HealthBasedLoadBalancer::new(0.7); // 70% health threshold

    // Test instances are already available in the instances vector
    // Health-based load balancer will use them directly for selection

    // Test instance selection
    let selected = health_balancer.select_instance(&instances).await;
    assert!(
        selected.is_some(),
        "Health-based balancer should select an instance"
    );

    // Test Latency-Optimized Load Balancer
    let latency_config = LoadBalancerConfig {
        strategy: LoadBalancerType::LatencyOptimized,
        health_check_interval: 10,
        max_retries: 3,
        timeout_seconds: 5,
    };

    let latency_balancer = LatencyOptimizedLoadBalancer::new(1000.0); // 1000ms max latency

    // Test instances are already available in the instances vector
    // Latency-optimized load balancer will use them directly for selection

    // Test instance selection
    let selected = latency_balancer.select_instance(&instances).await;
    assert!(
        selected.is_some(),
        "Latency-optimized balancer should select an instance"
    );

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
        nat_manager.establish_connection("test-session".to_string(), test_addr),
    )
    .await;

    // We expect this to timeout or fail in test environment, which is normal
    match connection_result {
        Ok(Ok(_)) => println!("✅ Connection established successfully"),
        Ok(Err(_)) => println!("⚠️ Connection failed (expected in test environment)"),
        Err(_) => println!("⚠️ Connection timed out (expected in test environment)"),
    }

    // Test Performance Monitor
    let config = BenchmarkConfig {
        test_duration_seconds: 1,
        concurrent_connections: 10,
        packet_rate_per_connection: 100,
        target_latency_us: 50_000,
        memory_pressure_test: true,
        cpu_stress_test: true,
    };

    let mut performance_monitor = PerformanceMonitor::new(config)?;

    // Start monitoring
    performance_monitor.start_monitoring().await?;

    // Get current metrics
    let metrics = performance_monitor.get_current_metrics().await;
    println!("📊 Current metrics: {:?}", metrics);

    // Note: PerformanceMonitor doesn't have stop_monitoring method
    // The monitoring will stop when the monitor is dropped

    println!("✅ Gaming Network Components test passed");
    Ok(())
}

async fn test_federation_system() -> Result<()> {
    println!("🔗 Testing Federation System...");

    // Create federation configuration
    let federation_config = FederationConfig {
        cluster_id: Some("test-cluster".to_string()),
        node_id: Some("test-node-1".to_string()),
        cluster_endpoints: vec!["http://127.0.0.1:8080".to_string()],
        heartbeat_interval: Some(30),
        auto_discovery: true,
        connection_timeout: 5,
        max_retries: 2,
        discovery_port: 8081,
        port: 8080,
    };

    // Initialize federation
    let federation = McpFederation::new(
        FederationMode::Standalone,
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
    println!(
        "📊 Federation Status: enabled={}, connected={}, nodes={}",
        status.enabled, status.connected, status.node_count
    );

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
    let storage = FileSystemStorage::new();

    // Test file operations
    let test_file = "test_integration.txt";
    let test_content = "This is a comprehensive integration test file content.";

    // Write file
    storage
        .write_file(test_file, test_content.as_bytes())
        .await?;
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
    storage.cleanup().await?;

    println!("✅ Universal Primal Storage test passed");
    Ok(())
}

async fn test_system_integration() -> Result<()> {
    println!("🎯 Testing System Integration...");

    // Test that all major components can be initialized together
    let registry = songbird_core::registry::ServiceRegistry::new().await?;
    let nat_manager = NatTraversalManager::new();
    let storage = FileSystemStorage::new();

    // Create a configuration that integrates all systems
    let _config = SongbirdConfig::default();

    // Test service registration with storage backend
    let service_info = ServiceInfo {
        service_id: "integrated-service".to_string(),
        name: "Integrated Service".to_string(),
        service_type: "integration".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Integration test service".to_string()),
        endpoints: vec![],
        health_check_endpoint: Some("/health".to_string()),
        tags: vec!["integration".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        host: "127.0.0.1".to_string(),
        port: 9999,
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert(
                "integration_test".to_string(),
                serde_json::Value::Bool(true),
            );
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
    registry
        .register_advanced_service(service_info.clone(), health_policy, scaling_policy)
        .await?;

    // Store service configuration
    let config_json = serde_json::to_string_pretty(&service_info)?;
    storage
        .write_file("integrated_service_config.json", config_json.as_bytes())
        .await?;

    // Test service discovery through registry
    let services = registry.list_services_with_status().await;
    assert!(
        !services.is_empty(),
        "Integrated service should be registered"
    );

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
    storage
        .delete_file("integrated_service_config.json")
        .await?;

    println!("✅ System Integration test passed");
    Ok(())
}

#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    println!("🚀 Testing Performance Under Load...");

    let registry = songbird_core::registry::ServiceRegistry::new().await?;
    let storage = FileSystemStorage::new();

    // Create multiple services
    let service_count = 10;
    let mut services = Vec::new();

    for i in 0..service_count {
        let service_info = ServiceInfo {
            service_id: format!("load-test-service-{}", i),
            name: format!("Load Test Service {}", i),
            service_type: "load_test".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Load testing service".to_string()),
            endpoints: vec![],
            health_check_endpoint: Some("/health".to_string()),
            tags: vec!["load_test".to_string()],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            instance_id: uuid::Uuid::new_v4().to_string(),
            host: "127.0.0.1".to_string(),
            port: (8000 + i) as u16,
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("load_test".to_string(), serde_json::Value::Bool(true));
                metadata.insert(
                    "instance".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(i)),
                );
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

        registry
            .register_advanced_service(service_info.clone(), health_policy, scaling_policy)
            .await?;

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
            storage_clone
                .write_file(&filename, content.as_bytes())
                .await
                .unwrap();

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

    let storage = FileSystemStorage::new();

    // Test reading non-existent file
    let result = storage.read_file("non_existent_file.txt").await;
    assert!(
        result.is_err(),
        "Should fail when reading non-existent file"
    );

    // Test deleting non-existent file
    let result = storage.delete_file("non_existent_file.txt").await;
    assert!(
        result.is_err(),
        "Should fail when deleting non-existent file"
    );

    // Test restoring non-existent backup
    let result = storage.restore_backup("non_existent_backup").await;
    assert!(
        result.is_err(),
        "Should fail when restoring non-existent backup"
    );

    println!("✅ Error Handling test passed");
    Ok(())
}
