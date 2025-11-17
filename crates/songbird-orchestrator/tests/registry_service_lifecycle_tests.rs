//! Service registry lifecycle tests
//!
//! Tests for service registration, health checking, and lifecycle management

use songbird_orchestrator::core::registry::ServiceStatus;
use songbird_orchestrator::core::{HealthStatus, RegistryConfig, ServiceInfo, ServiceRegistry};
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_registry_creation() {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    // Registry should be created successfully
    assert_eq!(registry.get_services().len(), 0, "New registry should be empty");
}

#[tokio::test]
async fn test_registry_initialization() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let result = registry.initialize().await;
    assert!(result.is_ok(), "Registry initialization should succeed");
}

#[tokio::test]
async fn test_registry_start_stop() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Start registry
    let start_result = registry.start().await;
    assert!(start_result.is_ok(), "Registry start should succeed");

    // Stop registry
    let stop_result = registry.stop().await;
    assert!(stop_result.is_ok(), "Registry stop should succeed");
}

#[tokio::test]
async fn test_service_registration() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "Test Service".to_string(),
        address: "localhost".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // Register service
    let result = registry.register_service(service.clone()).await;
    assert!(result.is_ok(), "Service registration should succeed");

    // Verify service is registered
    let services = registry.get_services();
    assert!(services.contains_key(&service_id), "Service should be in registry");

    let retrieved = services
        .get(&service_id)
        .or_else(|_| SongbirdError::configuration("Failed to register service".to_string()))?;
    assert_eq!(retrieved.name, "Test Service");
    assert_eq!(retrieved.address, test_bind_address());
    assert_eq!(retrieved.port, 8080);
    Ok(())
}

#[tokio::test]
async fn test_service_unregistration() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "Test Service".to_string(),
        address: "localhost".to_string(),
        port: 8081,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // Register service
    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    // Unregister service
    let result = registry.unregister_service(&service_id).await;
    assert!(result.is_ok(), "Service unregistration should succeed");

    // Verify service is no longer registered
    let services = registry.get_services();
    assert!(!services.contains_key(&service_id), "Service should be removed from registry");
    Ok(())
}

#[tokio::test]
async fn test_multiple_service_registrations() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register multiple services
    for i in 0..5 {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Service {}", i),
            address: "localhost".to_string(),
            port: 8000 + i,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    // Verify all services are registered
    let services = registry.get_services();
    assert_eq!(services.len(), 5, "Should have 5 registered services");
    Ok(())
}

#[tokio::test]
async fn test_service_health_check() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry.health_check().await;
    assert!(health.is_ok(), "Health check should succeed");

    let health_status =
        health.ok_or_else(|_| SongbirdError::configuration("Failed health check".to_string()))?;
    assert_eq!(health_status.status, HealthStatus::Healthy);
    Ok(())
}

#[tokio::test]
async fn test_service_metadata_storage() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("environment".to_string(), "production".to_string());

    let service = ServiceInfo {
        id: service_id,
        name: "Metadata Service".to_string(),
        address: "localhost".to_string(),
        port: 9000,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata,
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    // Retrieve and verify metadata
    let services = registry.get_services();
    let retrieved = services
        .get(&service_id)
        .or_else(|_| SongbirdError::configuration("Failed to register service".to_string()))?;

    assert_eq!(retrieved.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(retrieved.metadata.get("environment"), Some(&"production".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_service_status_variations() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let statuses = vec![
        ServiceStatus::Running,
        ServiceStatus::Stopped,
        ServiceStatus::Starting,
        ServiceStatus::Stopping,
        ServiceStatus::Error,
    ];

    for (i, status) in statuses.into_iter().enumerate() {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Status Test {}", i),
            address: "localhost".to_string(),
            port: 9100 + i as u16,
            status,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let result = registry.register_service(service).await;
        assert!(result.is_ok(), "Should register service with any valid status");
    }

    assert_eq!(registry.get_services().len(), 5, "All status types should be registerable");
}

#[tokio::test]
async fn test_registry_get_services_immutable() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "Test".to_string(),
        address: "localhost".to_string(),
        port: 9200,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    // Get services returns immutable reference
    let services = registry.get_services();
    assert_eq!(services.len(), 1);

    // Should still be able to access registry after getting services
    assert_eq!(registry.get_services().len(), 1);
    Ok(())
}
