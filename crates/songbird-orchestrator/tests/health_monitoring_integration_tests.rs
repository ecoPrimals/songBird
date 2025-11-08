//! Health monitoring and status tracking tests
//!
//! Tests for health check operations, status transitions,
//! and monitoring integration

use songbird_orchestrator::core::registry::ServiceStatus;
use songbird_orchestrator::core::{
    ComponentHealth, HealthStatus, RegistryConfig, ServiceInfo, ServiceRegistry,
};
use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_registry_health_check_basic() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry.health_check().await;
    assert!(health.is_ok(), "Health check should succeed");

    let component_health = health.ok_or_else(|_| {
        SongbirdError::configuration("Failed health check".to_string())
    })?;
    assert_eq!(component_health.status, HealthStatus::Healthy);
    assert!(component_health.message.is_some());
    Ok(())
}

#[tokio::test]
async fn test_registry_health_with_services() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register some services
    for i in 0..5 {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("Health-Service-{}", i),
            address: "localhost".to_string(),
            port: 8000 + i,
            status: ServiceStatus::Running,
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("Failed to register service".to_string())
        })?;
    }

    let health = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;
    assert_eq!(health.status, HealthStatus::Healthy);

    // Health message should mention service count
    if let Some(msg) = &health.message {
        assert!(msg.contains("5"), "Health message should mention 5 services");
    }
    Ok(())
}

#[tokio::test]
async fn test_service_status_transitions() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let service_id = Uuid::new_v4();

    // Start with Starting status
    let service = ServiceInfo {
        id: service_id,
        name: "Transitioning Service".to_string(),
        address: "localhost".to_string(),
        port: 8080,
        status: ServiceStatus::Starting,
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };

    registry.register_service(service).await.map_err(|e| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;

    // Verify initial status
    let services = registry.get_services();
    let service = services.get(&service_id).or_else(|_| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;
    assert_eq!(service.status, ServiceStatus::Starting);
    assert_eq!(service.health, HealthStatus::Unknown);
    Ok(())
}

#[tokio::test]
async fn test_health_status_variations() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    let health_transitions = vec![
        (HealthStatus::Healthy, ServiceStatus::Running),
        (HealthStatus::Degraded, ServiceStatus::Running),
        (HealthStatus::Unhealthy, ServiceStatus::Error),
        (HealthStatus::Unknown, ServiceStatus::Starting),
    ];

    for (health, status) in health_transitions {
        let service = ServiceInfo {
            id: Uuid::new_v4(),
            name: format!("{:?}-{:?}", health, status),
            address: "localhost".to_string(),
            port: 9000,
            status,
            health,
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.map_err(|e| {
            SongbirdError::configuration("Failed to register service".to_string())
        })?;
    }

    assert_eq!(
        registry.get_services().len(),
        4,
        "Should have 4 services with different health/status combinations"
    );
    Ok(())
}

#[tokio::test]
async fn test_component_health_timestamp() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    let health = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed health check".to_string())
    })?;

    // Verify timestamp is present and reasonable
    assert!(health.last_check.is_some(), "Health check should have timestamp");

    if let Some(timestamp) = health.last_check {
        let now = chrono::Utc::now().timestamp() as u64;
        assert!(timestamp <= now, "Timestamp should not be in the future");
        assert!(timestamp >= now - 60, "Timestamp should be recent (within last minute)");
    }
    Ok(())
}

#[tokio::test]
async fn test_health_check_repeatability() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let registry = ServiceRegistry::new(config);

    // Perform multiple health checks
    for _ in 0..10 {
        let health = registry.health_check().await;
        assert!(health.is_ok(), "Health check should always succeed");

        let component_health = health.ok_or_else(|_| {
            SongbirdError::configuration("Failed health check".to_string())
        })?;
        assert_eq!(component_health.status, HealthStatus::Healthy);
    }
    Ok(())
}

#[tokio::test]
async fn test_registry_lifecycle_health() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Health before initialization
    let health1 = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed health check".to_string())
    })?;
    assert_eq!(health1.status, HealthStatus::Healthy);

    // Initialize
    registry.initialize().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;

    // Health after initialization
    let health2 = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to initialize orchestrator".to_string())
    })?;
    assert_eq!(health2.status, HealthStatus::Healthy);

    // Start
    registry.start().await.map_err(|e| {
        SongbirdError::configuration("Failed to start orchestrator".to_string())
    })?;

    // Health after start
    let health3 = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to start orchestrator".to_string())
    })?;
    assert_eq!(health3.status, HealthStatus::Healthy);

    // Stop
    registry.stop().await.map_err(|e| {
        SongbirdError::configuration("Failed to start orchestrator".to_string())
    })?;

    // Health after stop should still work
    let health4 = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to stop orchestrator".to_string())
    })?;
    assert_eq!(health4.status, HealthStatus::Healthy);
    Ok(())
}

#[tokio::test]
async fn test_service_health_distribution() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register services with various health statuses
    let health_distribution =
        vec![(HealthStatus::Healthy, 5), (HealthStatus::Degraded, 3), (HealthStatus::Unhealthy, 2)];

    for (health, count) in health_distribution {
        for i in 0..count {
            let service = ServiceInfo {
                id: Uuid::new_v4(),
                name: format!("{:?}-{}", health, i),
                address: "localhost".to_string(),
                port: 8000 + i,
                status: ServiceStatus::Running,
                health: health.clone(),
                metadata: HashMap::new(),
            };
            registry.register_service(service).await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "Error: {}",
                    e
                ))
            })?;
        }
    }

    let services = registry.get_services();

    // Count health statuses
    let healthy = services.values().filter(|s| matches!(s.health, HealthStatus::Healthy)).count();
    let degraded = services.values().filter(|s| matches!(s.health, HealthStatus::Degraded)).count();
    let unhealthy =
        services.values().filter(|s| matches!(s.health, HealthStatus::Unhealthy)).count();

    assert_eq!(healthy, 5);
    assert_eq!(degraded, 3);
    assert_eq!(unhealthy, 2);

    // Registry overall health should still be healthy
    let registry_health = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed health check".to_string())
    })?;
    assert_eq!(registry_health.status, HealthStatus::Healthy);
    Ok(())
}

#[tokio::test]
async fn test_health_monitoring_metadata() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Register service with health metadata
    let mut metadata = HashMap::new();
    metadata.insert("last_health_check".to_string(), "2025-10-31T20:00:00Z".to_string());
    metadata.insert("health_score".to_string(), "95".to_string());

    let service = ServiceInfo {
        id: Uuid::new_v4(),
        name: "Monitored Service".to_string(),
        address: "localhost".to_string(),
        port: 8080,
        status: ServiceStatus::Running,
        health: HealthStatus::Healthy,
        metadata,
    };

    let service_id = service.id;
    registry.register_service(service).await.map_err(|e| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;

    // Retrieve and verify metadata
    let services = registry.get_services();
    let service = services.get(&service_id).or_else(|_| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;

    assert_eq!(service.metadata.get("health_score"), Some(&"95".to_string()));
    assert!(service.metadata.contains_key("last_health_check"));
    Ok(())
}

#[tokio::test]
async fn test_component_health_message_content() -> SongbirdResult<()> {
    let config = RegistryConfig::default();
    let mut registry = ServiceRegistry::new(config);

    // Empty registry
    let health_empty = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed health check".to_string())
    })?;
    assert!(health_empty.message.is_some());
    if let Some(msg) = &health_empty.message {
        assert!(msg.contains("0"), "Should mention 0 services");
    }

    // Add services
    for i in 0..3 {
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
            SongbirdError::configuration("Failed to register service".to_string())
        })?;
    }

    // With services
    let health_with_services = registry.health_check().await.map_err(|e| {
        SongbirdError::configuration("Failed to register service".to_string())
    })?;
    assert!(health_with_services.message.is_some());
    if let Some(msg) = &health_with_services.message {
        assert!(msg.contains("3"), "Should mention 3 services");
    }
    Ok(())
}
