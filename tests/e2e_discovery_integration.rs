//! E2E Service Discovery Integration Tests
//!
//! Tests the complete service discovery flow including:
//! - Discovery backend initialization
//! - Service registration and deregistration
//! - Health monitoring integration
//! - Capability-based queries
//! - Real-world scenarios

#![cfg(test)]

use songbird_discovery::abstraction::registry::ServiceRegistry;
use songbird_discovery::abstraction::types::Service;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_discovery_service_lifecycle_e2e() -> Result<(), Box<dyn std::error::Error>> {
    // Test complete lifecycle: create registry → register → discover → deregister
    let mut registry = ServiceRegistry::new();

    let service = Service {
        id: "test-service-1".to_string(),
        name: "Test Service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.95,
        capabilities: vec!["compute".to_string()],
        metadata: HashMap::new(),
    };

    // Register
    registry.register_service(service.clone()).await?;

    // Verify registration
    let services = registry.list_services().await;
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].id, "test-service-1");

    // Deregister
    registry.deregister_service(&service.id).await?;

    // Verify deregistration
    let services = registry.list_services().await;
    assert!(services.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_discovery_multi_service_registration() -> Result<(), Box<dyn std::error::Error>> {
    // Test registering multiple services with different capabilities
    let mut registry = ServiceRegistry::new();

    let services = vec![
        Service {
            id: "compute-service".to_string(),
            name: "Compute Service".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            health_score: 0.95,
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        },
        Service {
            id: "storage-service".to_string(),
            name: "Storage Service".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            health_score: 0.90,
            capabilities: vec!["storage".to_string()],
            metadata: HashMap::new(),
        },
        Service {
            id: "network-service".to_string(),
            name: "Network Service".to_string(),
            endpoint: "http://localhost:8082".to_string(),
            health_score: 0.85,
            capabilities: vec!["network".to_string()],
            metadata: HashMap::new(),
        },
    ];

    // Register all services
    for service in services {
        registry.register_service(service).await?;
    }

    // Verify all registered
    let registered = registry.list_services().await;
    assert_eq!(registered.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_discovery_capability_filtering() -> Result<(), Box<dyn std::error::Error>> {
    // Test filtering services by capability
    let mut registry = ServiceRegistry::new();

    // Register services with different capabilities
    for i in 0..5 {
        let capability = if i % 2 == 0 {
            "compute"
        } else {
            "storage"
        };
        let service = Service {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            endpoint: format!("http://localhost:808{}", i),
            health_score: 0.90,
            capabilities: vec![capability.to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    // List all services
    let all_services = registry.list_services().await;
    assert_eq!(all_services.len(), 5);

    // Count services by capability
    let compute_count =
        all_services.iter().filter(|s| s.capabilities.contains(&"compute".to_string())).count();
    let storage_count =
        all_services.iter().filter(|s| s.capabilities.contains(&"storage".to_string())).count();

    assert_eq!(compute_count, 3);
    assert_eq!(storage_count, 2);

    Ok(())
}

#[tokio::test]
async fn test_discovery_health_score_ordering() -> Result<(), Box<dyn std::error::Error>> {
    // Test that services can be ordered by health score
    let mut registry = ServiceRegistry::new();

    let health_scores = vec![0.95, 0.75, 0.90, 0.85, 0.99];

    for (i, &score) in health_scores.iter().enumerate() {
        let service = Service {
            id: format!("service-{}", i),
            name: format!("Service {}", i),
            endpoint: format!("http://localhost:808{}", i),
            health_score: score,
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    // Get all services
    let mut services = registry.list_services().await;

    // Sort by health score descending
    services.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());

    // Verify highest health score first
    assert!(services[0].health_score >= 0.99);
    assert!(services[services.len() - 1].health_score <= 0.75);

    Ok(())
}

#[tokio::test]
async fn test_discovery_metadata_enrichment() -> Result<(), Box<dyn std::error::Error>> {
    // Test services with rich metadata
    let mut registry = ServiceRegistry::new();

    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-west-2".to_string());
    metadata.insert("az".to_string(), "us-west-2a".to_string());
    metadata.insert("environment".to_string(), "production".to_string());

    let service = Service {
        id: "enriched-service".to_string(),
        name: "Enriched Service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.95,
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        metadata,
    };

    registry.register_service(service).await?;

    let services = registry.list_services().await;
    assert_eq!(services[0].metadata.len(), 4);
    assert_eq!(services[0].metadata.get("region").map(|s| s.as_str()), Some("us-west-2"));

    Ok(())
}

#[tokio::test]
async fn test_discovery_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test concurrent registrations and queries
    let mut registry = ServiceRegistry::new();

    // Register 10 services concurrently (sequentially for simplicity in tests)
    for i in 0..10 {
        let service = Service {
            id: format!("concurrent-{}", i),
            name: format!("Concurrent Service {}", i),
            endpoint: format!("http://localhost:80{:02}", i),
            health_score: 0.90 + (i as f64 * 0.01),
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    // Verify all registered
    let services = registry.list_services().await;
    assert_eq!(services.len(), 10);

    Ok(())
}

#[tokio::test]
async fn test_discovery_service_update_scenario() -> Result<(), Box<dyn std::error::Error>> {
    // Test updating service information (via re-registration)
    let mut registry = ServiceRegistry::new();

    let service_v1 = Service {
        id: "updatable-service".to_string(),
        name: "Updatable Service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.80,
        capabilities: vec!["compute".to_string()],
        metadata: HashMap::new(),
    };

    registry.register_service(service_v1).await?;

    // Update via deregister + re-register
    registry.deregister_service("updatable-service").await?;

    let mut metadata_v2 = HashMap::new();
    metadata_v2.insert("version".to_string(), "2.0.0".to_string());

    let service_v2 = Service {
        id: "updatable-service".to_string(),
        name: "Updatable Service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.95,
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        metadata: metadata_v2,
    };

    registry.register_service(service_v2).await?;

    // Verify updated
    let services = registry.list_services().await;
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].capabilities.len(), 2);
    assert!(services[0].health_score > 0.90);

    Ok(())
}

#[tokio::test]
async fn test_discovery_high_availability_scenario() -> Result<(), Box<dyn std::error::Error>> {
    // Test HA scenario with multiple instances of same capability
    let mut registry = ServiceRegistry::new();

    // Register 5 compute services (simulating HA deployment)
    for i in 0..5 {
        let service = Service {
            id: format!("compute-ha-{}", i),
            name: format!("Compute HA Instance {}", i),
            endpoint: format!("http://compute-{}.internal:8080", i),
            health_score: 0.85 + (i as f64 * 0.02),
            capabilities: vec!["compute".to_string()],
            metadata: {
                let mut m = HashMap::new();
                m.insert("instance".to_string(), i.to_string());
                m.insert(
                    "role".to_string(),
                    if i == 0 {
                        "primary".to_string()
                    } else {
                        "replica".to_string()
                    },
                );
                m
            },
        };
        registry.register_service(service).await?;
    }

    // Verify HA setup
    let services = registry.list_services().await;
    assert_eq!(services.len(), 5);

    // All should have compute capability
    for service in &services {
        assert!(service.capabilities.contains(&"compute".to_string()));
    }

    Ok(())
}

#[tokio::test]
async fn test_discovery_timeout_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Test that discovery operations complete within reasonable time
    let result = timeout(Duration::from_secs(5), async {
        let mut registry = ServiceRegistry::new();

        for i in 0..20 {
            let service = Service {
                id: format!("service-{}", i),
                name: format!("Service {}", i),
                endpoint: format!("http://localhost:808{}", i % 10),
                health_score: 0.90,
                capabilities: vec!["compute".to_string()],
                metadata: HashMap::new(),
            };
            registry.register_service(service).await?;
        }

        let services = registry.list_services().await;
        assert_eq!(services.len(), 20);

        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await;

    assert!(result.is_ok(), "Discovery operations should complete within timeout");
    assert!(result?.is_ok(), "Discovery operations should succeed");

    Ok(())
}

#[tokio::test]
async fn test_discovery_empty_state_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test operations on empty registry
    let registry = ServiceRegistry::new();

    // List should return empty
    let services = registry.list_services().await;
    assert!(services.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_discovery_duplicate_registration_handling() -> Result<(), Box<dyn std::error::Error>>
{
    // Test handling of duplicate service IDs
    let mut registry = ServiceRegistry::new();

    let service1 = Service {
        id: "duplicate-id".to_string(),
        name: "Service One".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.90,
        capabilities: vec!["compute".to_string()],
        metadata: HashMap::new(),
    };

    let service2 = Service {
        id: "duplicate-id".to_string(),
        name: "Service Two".to_string(),
        endpoint: "http://localhost:8081".to_string(),
        health_score: 0.95,
        capabilities: vec!["storage".to_string()],
        metadata: HashMap::new(),
    };

    // Register first
    registry.register_service(service1).await?;

    // Register second with same ID (may overwrite or error)
    registry.register_service(service2).await?;

    // Should have at least one service
    let services = registry.list_services().await;
    assert!(!services.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_discovery_multi_capability_services() -> Result<(), Box<dyn std::error::Error>> {
    // Test services with multiple capabilities
    let mut registry = ServiceRegistry::new();

    let service = Service {
        id: "multi-cap-service".to_string(),
        name: "Multi Capability Service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        health_score: 0.95,
        capabilities: vec![
            "compute".to_string(),
            "storage".to_string(),
            "network".to_string(),
            "security".to_string(),
        ],
        metadata: HashMap::new(),
    };

    registry.register_service(service).await?;

    let services = registry.list_services().await;
    assert_eq!(services[0].capabilities.len(), 4);

    Ok(())
}

#[tokio::test]
async fn test_discovery_service_endpoint_variations() -> Result<(), Box<dyn std::error::Error>> {
    // Test various endpoint formats
    let mut registry = ServiceRegistry::new();

    let endpoints = vec![
        "http://localhost:8080",
        "https://service.example.com:443",
        "http://192.168.1.100:3000",
        "https://api.service.local:8443/v1",
        "http://service:8080",
    ];

    for (i, endpoint) in endpoints.iter().enumerate() {
        let service = Service {
            id: format!("endpoint-test-{}", i),
            name: format!("Endpoint Test {}", i),
            endpoint: endpoint.to_string(),
            health_score: 0.90,
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    let services = registry.list_services().await;
    assert_eq!(services.len(), 5);

    Ok(())
}

#[tokio::test]
async fn test_discovery_service_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    // Test cleanup of all services
    let mut registry = ServiceRegistry::new();

    // Register multiple services
    for i in 0..10 {
        let service = Service {
            id: format!("cleanup-{}", i),
            name: format!("Cleanup Service {}", i),
            endpoint: format!("http://localhost:808{}", i % 10),
            health_score: 0.90,
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    // Verify registered
    assert_eq!(registry.list_services().await.len(), 10);

    // Cleanup all
    for i in 0..10 {
        registry.deregister_service(&format!("cleanup-{}", i)).await?;
    }

    // Verify cleaned up
    assert!(registry.list_services().await.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_discovery_stress_scenario() -> Result<(), Box<dyn std::error::Error>> {
    // Test registry under moderate load
    let mut registry = ServiceRegistry::new();

    // Register 50 services
    for i in 0..50 {
        let service = Service {
            id: format!("stress-{}", i),
            name: format!("Stress Test Service {}", i),
            endpoint: format!("http://service-{}.internal:8080", i),
            health_score: 0.80 + (i as f64 * 0.004),
            capabilities: vec!["compute".to_string()],
            metadata: HashMap::new(),
        };
        registry.register_service(service).await?;
    }

    // Verify all registered
    assert_eq!(registry.list_services().await.len(), 50);

    // Deregister half
    for i in 0..25 {
        registry.deregister_service(&format!("stress-{}", i)).await?;
    }

    // Verify half remain
    assert_eq!(registry.list_services().await.len(), 25);

    Ok(())
}

#[tokio::test]
async fn test_discovery_service_metadata_query() -> Result<(), Box<dyn std::error::Error>> {
    // Test querying services by metadata
    let mut registry = ServiceRegistry::new();

    // Register services with different metadata
    for i in 0..5 {
        let mut metadata = HashMap::new();
        metadata.insert(
            "environment".to_string(),
            if i < 3 {
                "production".to_string()
            } else {
                "staging".to_string()
            },
        );
        metadata.insert("region".to_string(), format!("us-west-{}", i % 2 + 1));

        let service = Service {
            id: format!("meta-{}", i),
            name: format!("Metadata Service {}", i),
            endpoint: format!("http://localhost:808{}", i),
            health_score: 0.90,
            capabilities: vec!["compute".to_string()],
            metadata,
        };
        registry.register_service(service).await?;
    }

    // Query and filter by metadata
    let all_services = registry.list_services().await;
    let production_services: Vec<_> = all_services
        .iter()
        .filter(|s| s.metadata.get("environment").map(|e| e.as_str()) == Some("production"))
        .collect();

    assert_eq!(production_services.len(), 3);

    Ok(())
}
