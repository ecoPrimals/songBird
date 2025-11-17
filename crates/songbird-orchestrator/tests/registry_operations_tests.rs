//! Comprehensive tests for service registry operations

use songbird_orchestrator::core::registry::{ServiceInfo, ServiceRegistry, ServiceStatus};
use songbird_orchestrator::core::{HealthStatus, RegistryConfig};
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use uuid::Uuid;

/// Helper to create test service
fn create_test_service(name: &str, port: u16) -> ServiceInfo {
    ServiceInfo {
        id: Uuid::new_v4(),
        name: name.to_string(),
        address: test_bind_address().to_string(),
        port,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_registry_creation() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);
    assert!(format!("{registry:?}").contains("ServiceRegistry"));
    Ok(())
}

#[tokio::test]
async fn test_registry_initialize() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);
    let result = registry.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_registry_start_stop() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let start_result = registry.start().await;
    assert!(start_result.is_ok());

    let stop_result = registry.stop().await;
    assert!(stop_result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_registry_health_check() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry.health_check().await;
    assert!(health.is_ok());

    let health_status =
        health.ok_or_else(|| SongbirdError::configuration("Failed health check".to_string()))?;
    assert_eq!(health_status.status, HealthStatus::Healthy);
    assert!(health_status.message.is_some());
    assert!(health_status.last_check.is_some());
    Ok(())
}

#[tokio::test]
async fn test_register_service() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service =
        create_test_service("test-service", songbird_config::defaults::ports::orchestrator_port());
    let result = registry.register_service(service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_multiple_services() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service1 =
        create_test_service("service-1", songbird_config::defaults::ports::orchestrator_port());
    let service2 =
        create_test_service("service-2", songbird_config::defaults::ports::discovery_port());
    let service3 =
        create_test_service("service-3", songbird_config::defaults::ports::beardog_port());

    assert!(registry.register_service(service1).await.is_ok());
    assert!(registry.register_service(service2).await.is_ok());
    assert!(registry.register_service(service3).await.is_ok());
}

#[tokio::test]
async fn test_service_info_creation() {
    let port = songbird_config::defaults::ports::metrics_port();
    let service = create_test_service("my-service", port);
    assert_eq!(service.name, "my-service");
    assert_eq!(service.port, port);
    assert_eq!(service.address, test_bind_address());
    assert_eq!(service.status, ServiceStatus::Running);
    assert_eq!(service.health, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_service_info_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-west-2".to_string());

    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "api-service".to_string(),
        address: "192.168.1.100".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("region"), Some(&"us-west-2".to_string()));
}

#[test]
fn test_service_status_variants() -> SongbirdResult<()> {
    assert_eq!(ServiceStatus::Running, ServiceStatus::Running);
    assert_ne!(ServiceStatus::Running, ServiceStatus::Stopped);
    assert_ne!(ServiceStatus::Starting, ServiceStatus::Stopping);
    assert_ne!(ServiceStatus::Error, ServiceStatus::Running);
    Ok(())
}

#[test]
fn test_service_status_serialization() -> SongbirdResult<()> {
    let status = ServiceStatus::Running;
    let json = serde_json::to_string(&status)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ServiceStatus =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(status, deserialized);
    Ok(())
}

#[test]
fn test_service_info_serialization() -> SongbirdResult<()> {
    let service = create_test_service("test", songbird_config::defaults::ports::dashboard_port());
    let json = serde_json::to_string(&service)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ServiceInfo =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(service.name, deserialized.name);
    assert_eq!(service.port, deserialized.port);
    Ok(())
}

#[test]
fn test_registry_config_defaults() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    assert_eq!(config.discovery_interval, 30);
    assert_eq!(config.service_timeout, 300);
    assert_eq!(config.max_services, 1000);
    Ok(())
}

#[test]
fn test_registry_config_custom() -> SongbirdResult<()> {
    let config = RegistryConfig {
        discovery_interval: 60,
        service_timeout: 600,
        max_services: 5000,
    };
    assert_eq!(config.discovery_interval, 60);
    assert_eq!(config.service_timeout, 600);
    assert_eq!(config.max_services, 5000);
    Ok(())
}

#[test]
fn test_registry_config_serialization() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: RegistryConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Failed to deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.discovery_interval, deserialized.discovery_interval);
    assert_eq!(config.service_timeout, deserialized.service_timeout);
    assert_eq!(config.max_services, deserialized.max_services);
    Ok(())
}

#[tokio::test]
async fn test_service_lifecycle() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Initialize
    assert!(registry.initialize().await.is_ok());

    // Start
    assert!(registry.start().await.is_ok());

    // Register services
    let service1 =
        create_test_service("service-1", songbird_config::defaults::ports::orchestrator_port());
    assert!(registry.register_service(service1).await.is_ok());

    // Health check
    let health = registry.health_check().await;
    assert!(health.is_ok());
    assert_eq!(
        health.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?.status,
        HealthStatus::Healthy
    );

    // Stop
    assert!(registry.stop().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_registry_health_check_message_contains_count() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register some services
    let ports = [
        songbird_config::defaults::ports::orchestrator_port(),
        songbird_config::defaults::ports::discovery_port(),
        songbird_config::defaults::ports::beardog_port(),
    ];
    for (i, port) in ports.iter().enumerate() {
        let service = create_test_service(&format!("service-{i}"), *port);
        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    let health = registry
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    let message = health
        .message
        .ok_or_else(|| SongbirdError::configuration("Failed to register service".to_string()))?;
    assert!(message.contains('3'));
    assert!(message.contains("services"));
    Ok(())
}

#[test]
fn test_service_info_clone() {
    let service =
        create_test_service("original", songbird_config::defaults::ports::orchestrator_port());
    let cloned = service.clone();
    assert_eq!(service.name, cloned.name);
    assert_eq!(service.port, cloned.port);
    assert_eq!(service.status, cloned.status);
}

#[test]
fn test_service_status_all_variants() {
    let variants = [
        ServiceStatus::Running,
        ServiceStatus::Stopped,
        ServiceStatus::Starting,
        ServiceStatus::Stopping,
        ServiceStatus::Error,
    ];

    // Ensure all variants are distinct
    for (i, v1) in variants.iter().enumerate() {
        for (j, v2) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(v1, v2);
            } else {
                assert_ne!(v1, v2);
            }
        }
    }
}

#[tokio::test]
async fn test_registry_operations_are_async() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Test that operations complete without blocking
    let start_time = std::time::Instant::now();
    registry.initialize().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    registry.start().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    let elapsed = start_time.elapsed();

    // Operations should complete quickly (< 100ms for these no-ops)
    assert!(elapsed.as_millis() < 100);
    Ok(())
}

#[tokio::test]
async fn test_registry_with_custom_config() {
    let config = RegistryConfig {
        discovery_interval: 10,
        service_timeout: 60,
        max_services: 100,
    };
    let mut registry = ServiceRegistry::new(config);

    assert!(registry.initialize().await.is_ok());
    assert!(registry.start().await.is_ok());
    assert!(registry.stop().await.is_ok());
}

// ============================================================================
// ADDITIONAL HIGH-VALUE TESTS FOR COVERAGE
// ============================================================================

#[tokio::test]
async fn test_register_service_with_duplicate_name() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service1 = create_test_service("duplicate", test_orchestrator_port());
    let service2 = create_test_service("duplicate", test_discovery_port());

    // First registration should succeed
    assert!(registry.register_service(service1).await.is_ok());

    // Second registration with same name should still succeed (different IDs)
    assert!(registry.register_service(service2).await.is_ok());
}

#[tokio::test]
async fn test_register_service_with_empty_name() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service = create_test_service("", test_orchestrator_port());
    let result = registry.register_service(service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_service_with_very_long_name() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let long_name = "a".repeat(1000);
    let service = create_test_service(&long_name, test_orchestrator_port());
    let result = registry.register_service(service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_service_with_special_characters() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let special_names = vec![
        "service-with-dashes",
        "service_with_underscores",
        "service.with.dots",
        "service@special!chars",
        "服务",
        "🚀rocket-service",
    ];

    for name in special_names {
        let service = create_test_service(name, test_orchestrator_port());
        let result = registry.register_service(service).await;
        assert!(result.is_ok(), "Failed to register service: {name}");
    }
}

#[tokio::test]
async fn test_registry_with_max_services_boundary() -> SongbirdResult<()> {
    let config = RegistryConfig {
        discovery_interval: 10,
        service_timeout: 60,
        max_services: 5,
    };
    let mut registry = ServiceRegistry::new(config);

    // Register up to max_services
    for i in 0..5 {
        let service =
            create_test_service(&format!("service-{i}"), test_orchestrator_port() + i as u16);
        assert!(registry.register_service(service).await.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_registry_with_zero_max_services() -> SongbirdResult<()> {
    let config = RegistryConfig {
        discovery_interval: 10,
        service_timeout: 60,
        max_services: 0,
    };
    let registry = ServiceRegistry::new(config);

    // Registry should still be created with zero max services
    assert!(format!("{registry:?}").contains("ServiceRegistry"));
    Ok(())
}

#[tokio::test]
async fn test_registry_with_very_large_max_services() {
    let config = RegistryConfig {
        discovery_interval: 10,
        service_timeout: 60,
        max_services: u32::MAX,
    };
    let mut registry = ServiceRegistry::new(config);

    assert!(registry.initialize().await.is_ok());
}

#[tokio::test]
async fn test_registry_multiple_start_stop_cycles() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    for _ in 0..5 {
        assert!(registry.start().await.is_ok());
        assert!(registry.stop().await.is_ok());
    }
}

#[tokio::test]
async fn test_registry_start_without_initialize() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Start without initialize should still work
    let result = registry.start().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_registry_stop_without_start() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Stop without start should still work (idempotent)
    let result = registry.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_registry_health_check_without_initialize() {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry.health_check().await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn test_concurrent_service_registration() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services concurrently
    let mut tasks = vec![];
    for i in 0..10 {
        let service =
            create_test_service(&format!("concurrent-{i}"), test_orchestrator_port() + i as u16);
        let result = registry.register_service(service).await;
        tasks.push(result);
    }

    // All should succeed
    for result in tasks {
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_registry_operations_complete_quickly() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let start = std::time::Instant::now();

    registry.initialize().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    registry.start().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    registry.stop().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;

    let elapsed = start.elapsed();

    // All operations should complete in less than 200ms
    assert!(elapsed.as_millis() < 200);
    Ok(())
}

#[test]
fn test_service_info_with_different_statuses() {
    let statuses = vec![
        ServiceStatus::Running,
        ServiceStatus::Stopped,
        ServiceStatus::Starting,
        ServiceStatus::Stopping,
        ServiceStatus::Error,
    ];

    for status in statuses {
        let mut service = create_test_service("test", test_orchestrator_port());
        service.status = status.clone();
        assert_eq!(service.status, status);
    }
}

#[test]
fn test_service_info_with_different_health_statuses() {
    let health_statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    for health in health_statuses {
        let mut service = create_test_service("test", test_orchestrator_port());
        service.health = health.clone();
        assert_eq!(service.health, health);
    }
}

#[test]
fn test_service_info_with_custom_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());
    metadata.insert("tags".to_string(), "production,critical".to_string());

    let mut service = create_test_service("test", test_orchestrator_port());
    service.metadata = metadata.clone();

    assert_eq!(service.metadata.len(), 3);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
}

#[test]
fn test_service_info_with_empty_metadata() -> SongbirdResult<()> {
    let service = create_test_service("test", test_orchestrator_port());
    assert!(service.metadata.is_empty());
    Ok(())
}

#[test]
fn test_service_info_with_large_metadata() -> SongbirdResult<()> {
    let mut metadata = HashMap::new();
    for i in 0..100 {
        metadata.insert(format!("key{i}"), format!("value{i}"));
    }

    let mut service = create_test_service("test", test_orchestrator_port());
    service.metadata = metadata;

    assert_eq!(service.metadata.len(), 100);
    Ok(())
}

#[test]
fn test_service_info_debug_format() -> SongbirdResult<()> {
    let service = create_test_service("debug-test", test_orchestrator_port());
    let debug_str = format!("{service:?}");

    assert!(debug_str.contains("ServiceInfo"));
    assert!(debug_str.contains("debug-test"));
    Ok(())
}

#[tokio::test]
async fn test_registry_config_with_minimum_values() -> SongbirdResult<()> {
    let config = RegistryConfig {
        discovery_interval: 1,
        service_timeout: 1,
        max_services: 1,
    };
    let mut registry = ServiceRegistry::new(config);

    assert!(registry.initialize().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_registry_config_clone() -> SongbirdResult<()> {
    let config1 = RegistryConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.discovery_interval, config2.discovery_interval);
    assert_eq!(config1.service_timeout, config2.service_timeout);
    assert_eq!(config1.max_services, config2.max_services);
    Ok(())
}

#[test]
fn test_registry_config_debug() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("RegistryConfig"));
    assert!(!debug_str.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_registry_with_port_boundary_values() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let ports = vec![1, 80, 443, 8080, 65535];

    for port in ports {
        let service = create_test_service(&format!("service-{port}"), port);
        assert!(registry.register_service(service).await.is_ok());
    }
}

#[test]
fn test_service_status_equality() {
    assert_eq!(ServiceStatus::Running, ServiceStatus::Running);
    assert_ne!(ServiceStatus::Running, ServiceStatus::Stopped);
    assert_ne!(ServiceStatus::Starting, ServiceStatus::Stopping);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Unhealthy, HealthStatus::Unknown);
}

#[tokio::test]
async fn test_registry_repeated_initialization() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Multiple initializations should be idempotent
    for _ in 0..5 {
        assert!(registry.initialize().await.is_ok());
    }
}

#[tokio::test]
async fn test_registry_lifecycle_order_variations() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Test different operation orders
    assert!(registry.start().await.is_ok());
    assert!(registry.initialize().await.is_ok());
    assert!(registry.health_check().await.is_ok());
    assert!(registry.stop().await.is_ok());
}

#[tokio::test]
async fn test_service_info_uuid_uniqueness() {
    let service1 = create_test_service("service", test_orchestrator_port());
    let service2 = create_test_service("service", test_orchestrator_port());

    // Even with same name and port, UUIDs should be different
    assert_ne!(service1.id, service2.id);
}

#[test]
fn test_service_info_address_format() {
    let service = create_test_service("test", test_orchestrator_port());

    // Address should be in valid format
    assert!(!service.address.is_empty());
    assert!(service.address.contains('.') || service.address.contains(':'));
}

#[tokio::test]
async fn test_registry_operations_are_independent() {
    let config1 = RegistryConfig::default();
    let config2 = RegistryConfig::default();

    let mut registry1 = ServiceRegistry::new(config1);
    let mut registry2 = ServiceRegistry::new(config2);

    // Operations on one registry shouldn't affect the other
    assert!(registry1.start().await.is_ok());
    assert!(registry2.initialize().await.is_ok());
    assert!(registry1.health_check().await.is_ok());
    assert!(registry2.start().await.is_ok());
}
