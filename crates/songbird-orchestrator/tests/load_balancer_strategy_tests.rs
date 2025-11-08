//! Load balancer strategy and routing tests
//!
//! Tests for load balancing strategies, service selection,
//! health-based routing, and failure handling

use songbird_orchestrator::core::registry::ServiceStatus;
use songbird_orchestrator::core::{HealthStatus, RegistryConfig, ServiceInfo, ServiceRegistry};
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_registry_with_multiple_services() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register multiple services
    for i in 0..10 {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Service-{}", i),
            address: "localhost".to_string(),
            port: 8000 + i,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();
    assert_eq!(services.len(), 10, "Should have 10 registered services");

    // Verify all services are healthy
    let healthy_count =
        services.values().filter(|s| matches!(s.health, HealthStatus::Healthy)).count();
    assert_eq!(healthy_count, 10, "All services should be healthy");
    Ok(())
}

#[tokio::test]
async fn test_registry_mixed_health_statuses() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let health_statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    for (i, health) in health_statuses.into_iter().enumerate() {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Health-Test-{}", i),
            address: "localhost".to_string(),
            port: 9000 + i as u16,
            status: ServiceStatus::Running,
            health,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();
    assert_eq!(services.len(), 4, "Should have 4 services with different health states");

    // Count each health status
    let healthy = services.values().filter(|s| matches!(s.health, HealthStatus::Healthy)).count();
    let degraded = services.values().filter(|s| matches!(s.health, HealthStatus::Degraded)).count();
    let unhealthy =
        services.values().filter(|s| matches!(s.health, HealthStatus::Unhealthy)).count();

    assert_eq!(healthy, 1, "Should have 1 healthy service");
    assert_eq!(degraded, 1, "Should have 1 degraded service");
    assert_eq!(unhealthy, 1, "Should have 1 unhealthy service");
    Ok(())
}

#[tokio::test]
async fn test_registry_service_filtering_by_status() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services with different statuses
    let statuses = vec![
        (ServiceStatus::Running, 3),
        (ServiceStatus::Stopped, 2),
        (ServiceStatus::Starting, 1),
    ];

    for (status, count) in statuses {
        for i in 0..count {
            let service = ServiceInfo {
                id: Uuid::new_v4(),
                name: format!("{:?}-{}", status, i),
                address: "localhost".to_string(),
                port: 10000 + i,
                status: status.clone(),
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };
            registry.register_service(service).await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "TODO: Replace with proper error handling: {}",
                    e
                ))
            })?;
        }
    }

    let services = registry.get_services();

    // Count running services
    let running_count =
        services.values().filter(|s| matches!(s.status, ServiceStatus::Running)).count();
    assert_eq!(running_count, 3, "Should have 3 running services");

    // Count stopped services
    let stopped_count =
        services.values().filter(|s| matches!(s.status, ServiceStatus::Stopped)).count();
    assert_eq!(stopped_count, 2, "Should have 2 stopped services");
    Ok(())
}

#[tokio::test]
async fn test_registry_service_port_uniqueness() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services on different ports
    let ports: Vec<u16> = (8000..8010).collect();

    for port in ports {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Port-{}", port),
            address: "localhost".to_string(),
            port,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();

    // Verify all unique ports
    let mut ports_seen = std::collections::HashSet::new();
    for service in services.values() {
        ports_seen.insert(service.port);
    }

    assert_eq!(ports_seen.len(), 10, "All services should have unique ports");
    Ok(())
}

#[tokio::test]
async fn test_registry_service_address_variations() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let addresses = vec![
        test_bind_address(),
        test_bind_address(),
        "example.com".to_string(),
        "192.168.1.1".to_string(),
    ];

    for (i, address) in addresses.iter().enumerate() {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Address-{}", i),
            address: address.to_string(),
            port: 8080,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();
    assert_eq!(services.len(), 4, "Should register services with different addresses");

    // Verify address diversity
    // Note: localhost and 127.0.0.1 may be normalized to the same address
    let unique_addresses: std::collections::HashSet<_> =
        services.values().map(|s| s.address.as_str()).collect();
    assert!(
        unique_addresses.len() >= 3,
        "Should have at least 3 unique addresses (localhost and 127.0.0.1 may be normalized)"
    );
    Ok(())
}

#[tokio::test]
async fn test_registry_bulk_operations() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Bulk register
    let mut service_ids = Vec::new();
    for i in 0..50 {
        let id = Uuid::new_v4();
        let service = ServiceInfo {
            id,
            name: format!("Bulk-{}", i),
            address: "localhost".to_string(),
            port: 8000 + (i % 100) as u16,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
        service_ids.push(id);
    }

    assert_eq!(registry.get_services().len(), 50, "Should have 50 services");

    // Bulk unregister half
    for id in service_ids.iter().take(25) {
        registry.unregister_service(id).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    assert_eq!(registry.get_services().len(), 25, "Should have 25 services remaining");
    Ok(())
}

#[tokio::test]
async fn test_registry_service_name_patterns() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let name_patterns = vec!["web-server", "api-gateway", "database", "cache", "queue"];

    for pattern in name_patterns {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: pattern.to_string(),
            address: "localhost".to_string(),
            port: 8080,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();

    // Verify we can find services by name pattern
    let web_services: Vec<_> = services.values().filter(|s| s.name.contains("web")).collect();
    assert_eq!(web_services.len(), 1, "Should find web service");

    let api_services: Vec<_> = services.values().filter(|s| s.name.contains("api")).collect();
    assert_eq!(api_services.len(), 1, "Should find API service");
    Ok(())
}

#[tokio::test]
async fn test_registry_metadata_queries() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services with metadata
    for i in 0..5 {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), format!("1.{}.0", i));
        metadata.insert(
            "environment".to_string(),
            if i % 2 == 0 {
                "prod"
            } else {
                "dev"
            }
            .to_string(),
        );

        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Meta-Service-{}", i),
            address: "localhost".to_string(),
            port: 8000 + i as u16,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata,
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }

    let services = registry.get_services();

    // Query by environment
    let prod_services: Vec<_> = services
        .values()
        .filter(|s| s.metadata.get("environment") == Some(&"prod".to_string()))
        .collect();
    assert_eq!(prod_services.len(), 3, "Should find 3 prod services");

    let dev_services: Vec<_> = services
        .values()
        .filter(|s| s.metadata.get("environment") == Some(&"dev".to_string()))
        .collect();
    assert_eq!(dev_services.len(), 2, "Should find 2 dev services");
}

#[tokio::test]
async fn test_registry_concurrent_modifications() {
    let config = RegistryConfig::default();
    let registry = std::sync::Arc::new(tokio::sync::Mutex::new(ServiceRegistry::new(config)));

    // Spawn concurrent registration tasks
    let mut handles = vec![];

    for i in 0..10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            let service = ServiceInfo {
                id: Uuid::new_v4(),
                name: format!("Concurrent-{}", i),
                address: "localhost".to_string(),
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

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.is_ok(), "Concurrent registration should succeed");
    }

    let reg = registry.lock().await;
    assert_eq!(reg.get_services().len(), 10, "All concurrent registrations should succeed");
}
