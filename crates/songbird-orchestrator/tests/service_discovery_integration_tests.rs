//! Service discovery integration and query tests
//!
//! Tests for service discovery operations, querying, and integration scenarios

use songbird_orchestrator::core::registry::ServiceStatus;
use songbird_orchestrator::core::{HealthStatus, RegistryConfig, ServiceInfo, ServiceRegistry};
use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_bind_address;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_service_discovery_by_id() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "Discoverable Service".to_string(),
        address: "localhost".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service.clone())
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    // Discover by ID
    let services = registry.get_services();
    let found = services.get(&service_id);

    assert!(found.is_some(), "Should find service by ID");
    assert_eq!(found?.name, "Discoverable Service");
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_multiple_criteria() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services with different characteristics
    let services_data = vec![
        (
            "web-1",
            test_bind_address(),
            songbird_config::defaults::ports::orchestrator_port(),
            ServiceStatus::Running,
            HealthStatus::Healthy,
        ),
        (
            "web-2",
            test_bind_address(),
            songbird_config::defaults::ports::discovery_port(),
            ServiceStatus::Running,
            HealthStatus::Healthy,
        ),
        (
            "api-1",
            test_bind_address(),
            songbird_config::defaults::ports::metrics_port(),
            ServiceStatus::Running,
            HealthStatus::Degraded,
        ),
        ("db-1", test_bind_address(), 5432, ServiceStatus::Starting, HealthStatus::Unknown),
    ];

    for (name, addr, port, status, health) in services_data {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            address: addr.to_string(),
            port,
            status,
            health,
            metadata: HashMap::new(),
        };
        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    let all_services = registry.get_services();

    // Query by status
    let running: Vec<_> =
        all_services.values().filter(|s| matches!(s.status, ServiceStatus::Running)).collect();
    assert_eq!(running.len(), 3, "Should find 3 running services");

    // Query by health
    let healthy: Vec<_> =
        all_services.values().filter(|s| matches!(s.health, HealthStatus::Healthy)).collect();
    assert_eq!(healthy.len(), 2, "Should find 2 healthy services");

    // Query by name pattern
    let web_services: Vec<_> =
        all_services.values().filter(|s| s.name.starts_with("web")).collect();
    assert_eq!(web_services.len(), 2, "Should find 2 web services");
}

#[tokio::test]
async fn test_service_discovery_empty_registry() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let services = registry.get_services();
    assert_eq!(services.len(), 0, "Empty registry should return no services");
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_after_unregister() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();
    let service = ServiceInfo {
        id: service_id,
        name: "Temporary Service".to_string(),
        address: "localhost".to_string(),
        port: songbird_config::defaults::ports::orchestrator_port(),
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    registry
        .unregister_service(&service_id)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    let services = registry.get_services();
    assert!(!services.contains_key(&service_id), "Unregistered service should not be discoverable");
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_by_port() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let target_port = 9999;
    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "Port Service".to_string(),
        address: "localhost".to_string(),
        port: target_port,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    registry
        .register_service(service)
        .await
        .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;

    let services = registry.get_services();
    let found: Vec<_> = services.values().filter(|s| s.port == target_port).collect();

    assert_eq!(found.len(), 1, "Should find service by port");
    assert_eq!(found[0].port, target_port);
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_by_address() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let addresses = vec![test_bind_address(), "example.com".to_string()];

    for addr in &addresses {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Service-{}", addr),
            address: addr.to_string(),
            port: songbird_config::defaults::ports::orchestrator_port(),
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    let services = registry.get_services();

    // Find by specific address
    let localhost_services: Vec<_> =
        services.values().filter(|s| s.address == test_bind_address()).collect();
    assert_eq!(localhost_services.len(), 1);

    let example_services: Vec<_> =
        services.values().filter(|s| s.address == "example.com").collect();
    assert_eq!(example_services.len(), 1);
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_with_metadata_filter() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services with metadata
    for i in 0..5 {
        let mut metadata = HashMap::new();
        metadata.insert(
            "environment".to_string(),
            if i < 3 {
                "prod"
            } else {
                "dev"
            }
            .to_string(),
        );
        metadata.insert(
            "region".to_string(),
            format!(
                "us-{}",
                if i % 2 == 0 {
                    "west"
                } else {
                    "east"
                }
            ),
        );

        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Service-{}", i),
            address: "localhost".to_string(),
            port: 8000 + i as u16,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata,
        };
        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    let services = registry.get_services();

    // Filter by environment
    let prod_services: Vec<_> = services
        .values()
        .filter(|s| s.metadata.get("environment") == Some(&"prod".to_string()))
        .collect();
    assert_eq!(prod_services.len(), 3);

    // Filter by region
    let west_services: Vec<_> = services
        .values()
        .filter(|s| s.metadata.get("region").map(|r| r.contains("west")).unwrap_or(false))
        .collect();
    assert_eq!(west_services.len(), 3);
}

#[tokio::test]
async fn test_service_discovery_ordering() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let mut service_ids = Vec::new();
    for i in 0..5 {
        let id = Uuid::new_v4();
        let service = ServiceInfo {
            id,
            name: format!("Service-{}", i),
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
        service_ids.push(id);
    }

    let services = registry.get_services();

    // Verify all services are present
    for id in &service_ids {
        assert!(services.contains_key(id), "Service should be discoverable");
    }
    Ok(())
}

#[tokio::test]
async fn test_service_discovery_complex_query() {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register diverse services
    let test_data = vec![
        (
            "web-frontend",
            songbird_config::defaults::ports::orchestrator_port(),
            ServiceStatus::Running,
            HealthStatus::Healthy,
            "prod",
        ),
        (
            "web-backend",
            songbird_config::defaults::ports::discovery_port(),
            ServiceStatus::Running,
            HealthStatus::Healthy,
            "prod",
        ),
        (
            "api-gateway",
            songbird_config::defaults::ports::metrics_port(),
            ServiceStatus::Running,
            HealthStatus::Degraded,
            "prod",
        ),
        ("cache-server", 6379, ServiceStatus::Running, HealthStatus::Healthy, "prod"),
        ("test-service", 8888, ServiceStatus::Starting, HealthStatus::Unknown, "dev"),
    ];

    for (name, port, status, health, env) in test_data {
        let mut metadata = HashMap::new();
        metadata.insert("environment".to_string(), env.to_string());

        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            address: "localhost".to_string(),
            port,
            status,
            health,
            metadata,
        };
        registry
            .register_service(service)
            .await
            .map_err(|e| SongbirdError::configuration("Failed to register service".to_string()))?;
    }

    let services = registry.get_services();

    // Complex query: prod + running + healthy
    let prod_healthy_running: Vec<_> = services
        .values()
        .filter(|s| {
            s.metadata.get("environment") == Some(&"prod".to_string())
                && matches!(s.status, ServiceStatus::Running)
                && matches!(s.health, HealthStatus::Healthy)
        })
        .collect();

    assert_eq!(prod_healthy_running.len(), 3, "Should find 3 services matching all criteria");
}
