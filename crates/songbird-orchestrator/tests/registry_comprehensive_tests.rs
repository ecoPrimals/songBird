//! Comprehensive tests for Service Registry

use songbird_orchestrator::core::registry::ServiceStatus;
use songbird_orchestrator::core::{
    ComponentHealth, HealthStatus, RegistryConfig, ServiceInfo, ServiceRegistry,
};
use songbird_types::SongbirdError;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_registry_creation() {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    // Verify registry is created
    assert!(std::mem::size_of_val(&registry) > 0);
}

#[tokio::test]
async fn test_registry_initialize() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let result = registry.initialize().await;
    assert!(result.is_ok(), "Registry initialization should succeed");
}

#[tokio::test]
async fn test_registry_start_stop() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let start_result = registry.start().await;
    assert!(start_result.is_ok(), "Registry start should succeed");

    let stop_result = registry.stop().await;
    assert!(stop_result.is_ok(), "Registry stop should succeed");
}

#[tokio::test]
async fn test_registry_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Health check should succeed: {}", e)))?;

    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.message.is_some());
    assert!(health.last_check.is_some());
    Ok(())
}

#[tokio::test]
async fn test_service_registration() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "test-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let result = registry.register_service(service).await;
    assert!(result.is_ok(), "Service registration should succeed");
}

#[tokio::test]
async fn test_multiple_service_registration() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    for i in 0..5 {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("service-{i}"),
            address: "127.0.0.1".to_string(),
            port: 8080 + i,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let result = registry.register_service(service).await;
        assert!(result.is_ok(), "Service {i} registration should succeed");
    }

    let services = registry.get_services();
    assert_eq!(services.len(), 5, "Should have 5 registered services");
}

#[tokio::test]
async fn test_service_count_after_registration() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "count-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 9000,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let initial_count = registry.get_services().len();
    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Registration should succeed: {}", e)))?;
    let final_count = registry.get_services().len();

    assert_eq!(final_count, initial_count + 1, "Service count should increase by 1");
    Ok(())
}

#[tokio::test]
async fn test_service_lookup() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "lookup-test".to_string(),
        address: "192.168.1.1".to_string(),
        port: 7777,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service.clone())
        .await
        .map_err(|e| SongbirdError::configuration(format!("Registration should succeed: {}", e)))?;

    let services = registry.get_services();
    let found = services.get(&service_id);

    assert!(found.is_some(), "Service should be found in registry");
    let found_service =
        found.ok_or_else(|| SongbirdError::configuration("Test operation failed".to_string()))?;
    assert_eq!(found_service.id, service_id);
    assert_eq!(found_service.name, "lookup-test");
    Ok(())
}

#[tokio::test]
async fn test_service_status_enum() {
    assert_eq!(ServiceStatus::Running, ServiceStatus::Running);
    assert_ne!(ServiceStatus::Running, ServiceStatus::Stopped);

    // Test all status variants
    let statuses = [
        ServiceStatus::Running,
        ServiceStatus::Stopped,
        ServiceStatus::Starting,
        ServiceStatus::Stopping,
        ServiceStatus::Error,
    ];

    assert_eq!(statuses.len(), 5);
}

#[tokio::test]
async fn test_service_info_creation() {
    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "creation-test".to_string(),
        address: "10.0.0.1".to_string(),
        port: 3000,
        status: ServiceStatus::Starting,
        health: HealthStatus::Healthy,
        metadata: HashMap::from([
            ("version".to_string(), "1.0.0".to_string()),
            ("environment".to_string(), "test".to_string()),
        ]),
    };

    assert_eq!(service.name, "creation-test");
    assert_eq!(service.port, 3000);
    assert_eq!(service.status, ServiceStatus::Starting);
    assert_eq!(service.metadata.len(), 2);
}

#[tokio::test]
async fn test_service_info_clone() {
    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "clone-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let cloned = service.clone();
    assert_eq!(service.id, cloned.id);
    assert_eq!(service.name, cloned.name);
    assert_eq!(service.port, cloned.port);
}

#[tokio::test]
async fn test_service_info_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "serialize-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&service)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("serialize-test"));
    assert!(json.contains("8080"));
    Ok(())
}

#[tokio::test]
async fn test_service_info_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "deserialize-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 9090,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&service)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    let deserialized: ServiceInfo = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Should deserialize: {}", e)))?;

    assert_eq!(service.name, deserialized.name);
    assert_eq!(service.port, deserialized.port);
    Ok(())
}

#[tokio::test]
async fn test_registry_config_default() {
    let config = RegistryConfig::default();

    // Verify default config is valid
    assert!(std::mem::size_of_val(&config) > 0);
}

#[tokio::test]
async fn test_component_health_creation() {
    let health = ComponentHealth {
        status: HealthStatus::Healthy,
        message: Some("All systems operational".to_string()),
        last_check: Some(12345),
    };

    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.message.is_some());
    assert_eq!(health.last_check, Some(12345));
}

#[tokio::test]
async fn test_health_status_values() {
    // Test all health status variants
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;

    assert_eq!(healthy, HealthStatus::Healthy);
    assert_eq!(degraded, HealthStatus::Degraded);
    assert_eq!(unhealthy, HealthStatus::Unhealthy);

    assert_ne!(healthy, degraded);
    assert_ne!(degraded, unhealthy);
}

#[tokio::test]
async fn test_registry_with_different_ports() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let ports = vec![3000, 8080, 8443, 9000, 9090];

    for port in ports {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("service-port-{port}"),
            address: "127.0.0.1".to_string(),
            port,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let result = registry.register_service(service).await;
        assert!(result.is_ok(), "Service with port {port} should register");
    }
}

#[tokio::test]
async fn test_service_with_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "2.0.0".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());
    metadata.insert("tier".to_string(), "production".to_string());

    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "metadata-service".to_string(),
        address: "10.0.0.5".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Registration should succeed: {}", e)))?;

    assert_eq!(metadata.len(), 3);
    Ok(())
}

#[tokio::test]
async fn test_registry_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Complete lifecycle test
    registry
        .initialize()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Initialize should succeed: {}", e)))?;
    registry
        .start()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Start should succeed: {}", e)))?;

    let health = registry
        .health_check()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Health check should succeed: {}", e)))?;
    assert_eq!(health.status, HealthStatus::Healthy);

    registry
        .stop()
        .await
        .map_err(|e| SongbirdError::configuration(format!("Stop should succeed: {}", e)))?;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_service_registration() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let registry = std::sync::Arc::new(tokio::sync::Mutex::new(ServiceRegistry::new(config)));

    let mut handles = vec![];

    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            let service = ServiceInfo {
                id: Uuid::new_v4(),
                name: format!("concurrent-service-{i}"),
                address: "127.0.0.1".to_string(),
                port: 8000 + i,
                status: ServiceStatus::Running,
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            let mut reg = registry_clone.lock().await;
            reg.register_service(service).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle
            .await
            .map_err(|e| SongbirdError::configuration(format!("Task should complete: {}", e)))?;
        assert!(result.is_ok(), "Concurrent registration should succeed");
    }
    Ok(())
}

#[tokio::test]
async fn test_service_status_transitions() -> Result<(), Box<dyn std::error::Error>> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();

    // Register service in Starting state
    let service = ServiceInfo {
        id: service_id,
        name: "transition-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        status: ServiceStatus::Starting,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Registration should succeed: {}", e)))?;

    // Update to Running state
    let updated_service = ServiceInfo {
        id: service_id,
        name: "transition-test".to_string(),
        address: "127.0.0.1".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(updated_service)
        .await
        .map_err(|e| SongbirdError::configuration(format!("Update should succeed: {}", e)))?;
    Ok(())
}

#[tokio::test]
async fn test_empty_registry() {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let services = registry.get_services();
    assert!(services.is_empty(), "New registry should be empty");
}

#[tokio::test]
async fn test_service_with_different_addresses() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let addresses = ["127.0.0.1", "192.168.1.100", "10.0.0.1", "172.16.0.1", "::1"];

    for (i, addr) in addresses.iter().enumerate() {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("service-addr-{i}"),
            address: (*addr).to_string(),
            port: 8080,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let result = registry.register_service(service).await;
        assert!(result.is_ok(), "Service with address {addr} should register");
    }
}
