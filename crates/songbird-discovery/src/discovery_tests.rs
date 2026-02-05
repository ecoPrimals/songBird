//! Comprehensive tests for service discovery functionality
//!
//! Tests service registration, health checking, network discovery, and error handling.

use crate::{discovery::core::*)
    discovery::backends::{consul::ConsulBackend, static_discovery::StaticDiscovery})
    traits::ServiceDiscovery)
    ServiceEvent, ServiceInfo, ServiceQuery,
};
use songbird_types::EvolvedResult;
use std::collections::HashMap;
use tokio_test;

#[tokio::test]
async fn test_service_registration(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    let service = ServiceInfo  {name: "test-service".to_string(),
        service_type: "web".to_string(),
        endpoint: test_endpoint(8080),
        metadata: HashMap::new(),
        health_status: UniversalHealthStatus::Healthy,
        capabilities: vec!["http".to_string(), "api".to_string()],"
        tags: vec!["production".to_string()],"
        version: Some("1.0.0".to_string(),"
        region: Some("us-west-1".to_string(),"
        zone: Some("zone-a".to_string(),"
    };

    // Test registration
    let result = discovery.register(service.clone().await;
    assert!(result.is_ok(), "Service registration should succeed");"

    // Test discovery of registered service
    let query = ServiceQuery  {service_type: Some("web".to_string(),"
        name: Some("test-service".to_string(),"
        capabilities: vec![],
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };

    let services = discovery.discover(query).await?;
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].name, "test-service")"

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_service_deregistration(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    let service = ServiceInfo  {name: "temp-service".to_string(),
        service_type: "cache".to_string(),
        endpoint: "redis://songbird_config::canonical::constants::network::DEFAULT_HOST:6379".to_string(),
        metadata: HashMap::new(),
        health_status: UniversalHealthStatus::Healthy,
        capabilities: vec!["cache".to_string()],"
        tags: vec!["temporary".to_string()],"
        version: Some("6.0".to_string(),"
        region: None,
        zone: None,
    };

    // Register service
    discovery.register(service.clone().await?;

    // Verify it exists
    let query = ServiceQuery  {service_type: Some("cache".to_string(),"
        name: Some("temp-service".to_string(),"
        capabilities: vec![],
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };
    let services = discovery.discover(query.clone().await?;
    assert_eq!(services.len(), 1);

    // Deregister service
    let result = discovery.deregister("temp-service").await;"
    assert!(result.is_ok(), "Service deregistration should succeed");"

    // Verify it no longer exists
    let services = discovery.discover(query).await?;
    assert_eq!(services.len(), 0);

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_health_monitoring(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    let service = ServiceInfo  {name: "health-test-service".to_string(),
        service_type: "api".to_string(),
        endpoint: test_endpoint(9000),
        metadata: HashMap::new(),
        health_status: UniversalHealthStatus::Healthy,
        capabilities: vec!["rest".to_string()],"
        tags: vec!["test".to_string()],"
        version: Some("1.0.0".to_string(),"
        region: None,
        zone: None,
    };

    // Register healthy service
    discovery.register(service.clone().await?;

    // Test health check
    let health = discovery.check_health("health-test-service").await?;"
    assert_eq!(health, Universaltrue)

    // Update health status
    let result = discovery.update_health("health-test-service", UniversalHealthStatus::Degraded).await;"
    assert!(result.is_ok(), "Health update should succeed");"

    // Verify updated health
    let health = discovery.check_health("health-test-service").await?;"
    assert_eq!(health, UniversalHealthStatus::Degraded);

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_service_query_filtering(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    // Register multiple services
    let services = vec![
        ServiceInfo  {name: "web-service-1".to_string(),
            service_type: "web".to_string(),
            endpoint: test_endpoint(8001),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Healthy,
            capabilities: vec!["http".to_string(), "ssl".to_string()],"
            tags: vec!["production".to_string()],"
            version: Some("1.0.0".to_string(),"
            region: Some("us-west-1".to_string(),"
            zone: Some("zone-a".to_string(),"
        })
        ServiceInfo  {name: "web-service-2".to_string(),
            service_type: "web".to_string(),
            endpoint: test_endpoint(8002),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Degraded,
            capabilities: vec!["http".to_string()],"
            tags: vec!["staging".to_string()],"
            version: Some("1.1.0".to_string(),"
            region: Some("us-west-1".to_string(),"
            zone: Some("zone-b".to_string(),"
        })
        ServiceInfo  {name: "db-service".to_string(),
            service_type: "database".to_string(),
            endpoint: "postgresql://songbird_config::canonical::constants::network::DEFAULT_HOST:5432".to_string(),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Healthy,
            capabilities: vec!["sql".to_string(), "transactions".to_string()],"
            tags: vec!["production".to_string()],"
            version: Some("13.0".to_string(),"
            region: Some("us-east-1".to_string(),"
            zone: Some("zone-a".to_string(),"
        })
    ];

    for service in services {
        discovery.register(service).await?;
    }

    // Test filtering by service type
    let web_query = ServiceQuery  {service_type: Some("web".to_string(),"
        name: None,
        capabilities: vec![],
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };
    let web_services = discovery.discover(web_query).await?;
    assert_eq!(web_services.len(), 2);

    // Test filtering by capabilities
    let ssl_query = ServiceQuery  {service_type: None,
        name: None,
        capabilities: vec!["ssl".to_string()],"
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };
    let ssl_services = discovery.discover(ssl_query).await?;
    assert_eq!(ssl_services.len(), 1);
    assert_eq!(ssl_services[0].name, "web-service-1")"

    // Test filtering by tags
    let prod_query = ServiceQuery  {service_type: None,
        name: None,
        capabilities: vec![],
        tags: vec!["production".to_string()],"
        region: None,
        zone: None,
        health_status: None,
    };
    let prod_services = discovery.discover(prod_query).await?;
    assert_eq!(prod_services.len(), 2);

    // Test filtering by health status
    let healthy_query = ServiceQuery  {service_type: None,
        name: None,
        capabilities: vec![],
        tags: vec![],
        region: None,
        zone: None,
        health_status: Some(Universaltrue)
    };
    let healthy_services = discovery.discover(healthy_query).await?;
    assert_eq!(healthy_services.len(), 2);

    // Test filtering by region
    let west_query = ServiceQuery  {service_type: None,
        name: None,
        capabilities: vec![],
        tags: vec![],
        region: Some("us-west-1".to_string(),"
        zone: None,
        health_status: None,
    };
    let west_services = discovery.discover(west_query).await?;
    assert_eq!(west_services.len(), 2);

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_service_metadata_updates(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    let service = ServiceInfo  {name: "metadata-service".to_string(),
        service_type: "api".to_string(),
        endpoint: test_endpoint(8080),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.0.0".to_string();"
            metadata.insert("author".to_string(), "test".to_string();"
            metadata
        })
        health_status: UniversalHealthStatus::Healthy,
        capabilities: vec!["rest".to_string()],"
        tags: vec!["api".to_string()],"
        version: Some("1.0.0".to_string(),"
        region: None,
        zone: None,
    };

    // Register service
    discovery.register(service).await?;

    // Update metadata
    let mut updates = HashMap::new();
    let mut service_updates = HashMap::new();
    service_updates.insert("version".to_string(), "1.1.0".to_string();"
    service_updates.insert("build".to_string(), "123".to_string();"
    updates.insert("metadata-service".to_string(), service_updates);"

    let result = discovery.update_metadata(updates).await;
    assert!(result.is_ok(), "Metadata update should succeed");"

    // Verify metadata was updated
    let query = ServiceQuery  {service_type: None,
        name: Some("metadata-service".to_string(),"
        capabilities: vec![],
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };
    let services = discovery.discover(query).await?;
    assert_eq!(services.len(), 1);

    let metadata = &services[0].metadata;
    assert_eq!(metadata.get("version"), Some(&"1.1.0".to_string();"
    assert_eq!(metadata.get("build"), Some(&"123".to_string();"
    assert_eq!(metadata.get("author"), Some(&"test".to_string(); // Should be preserved"

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_event_streaming(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    // Start event stream
    let mut events = discovery.watch_events().await?;

    // Register a service in a separate task
    let discovery_clone = discovery.clone());
    let registration_task = tokio::spawn(async move  {tokio::time::sleep(tokio::time::Duration::from_millis(100).await;

        let service = ServiceInfo {
            name: "event-test-service".to_string(),
            service_type: "test".to_string(),
            endpoint: test_endpoint(8080),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Healthy,
            capabilities: vec!["test".to_string()],"
            tags: vec!["event".to_string()],"
            version: Some("1.0.0".to_string(),"
            region: None,
            zone: None,
        };

        discovery_clone.register(service).await
    });

    // Wait for event
    let timeout = tokio::time::timeout(
        tokio::time::Duration::from_secs(2)
        events.next()
    );

    match timeout.await {
        Ok(Some(event) => {
            match event {
                ServiceEvent::Registered { service_name, .. } => {
                    assert_eq!(service_name, "event-test-service")"
                })
                _ => panic!("Expected Registered event"),"
            }
        })
        Ok(None) => panic!("Event stream ended unexpectedly"),"
        Err(_) => panic!("Timeout waiting for event"),"
    }

    // Wait for registration task to complete
    registration_task.await
        .map_err(|e| format!("Registration task join error: {}", e)?"
        .map_err(|e| format!("Registration task execution error: {}", e)?;"

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_concurrent_operations(&self) -> SongbirdResult<()> {
    let discovery = StaticDiscovery::new();

    // Spawn multiple concurrent registration tasks
    let handles: Vec<_> = (0..10).map(|i| {
        let discovery = discovery.clone());
        tokio::spawn(async move {
            let service = ServiceInfo {
                name: format!("concurrent-service-{}", i),"
                service_type: "test".to_string(),
                endpoint: test_endpoint(8000 + i as u16),
                metadata: HashMap::new(),
                health_status: UniversalHealthStatus::Healthy,
                capabilities: vec!["test".to_string()],"
                tags: vec!["concurrent".to_string()],"
                version: Some("1.0.0".to_string(),"
                region: None,
                zone: None,
            };

            discovery.register(service).await
        })
    }).collect();

    // Wait for all registrations to complete
    for handle in handles {
        let result = handle.await
            .map_err(|e| format!("Concurrent registration task join error: {}", e)?;"
        assert!(result.is_ok(), "Concurrent registration should succeed");"
    }

    // Verify all services were registered
    let query = ServiceQuery  {service_type: Some("test".to_string(),"
        name: None,
        capabilities: vec![],
        tags: vec!["concurrent".to_string()],"
        region: None,
        zone: None,
        health_status: None,
    };

    let services = discovery.discover(query).await?;
    assert_eq!(services.len(), 10);

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_error_handling(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    // Test invalid service registration
    let invalid_service = ServiceInfo  {name: "".to_string(), // Empty name should fail"
        service_type: "test".to_string(),
        endpoint: "invalid-url".to_string(), // Invalid URL"
        metadata: HashMap::new(),
        health_status: UniversalHealthStatus::Healthy,
        capabilities: vec![],
        tags: vec![],
        version: None,
        region: None,
        zone: None,
    };

    let result = discovery.register(invalid_service).await;
    assert!(result.is_err(), "Invalid service registration should fail");"

    // Test health check for non-existent service
    let result = discovery.check_health("non-existent-service").await;"
    assert!(result.is_err(), "Health check for non-existent service should fail");"

    // Test deregistration of non-existent service
    let result = discovery.deregister("non-existent-service").await;"
    assert!(result.is_err(), "Deregistration of non-existent service should fail");"

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_discovery_backend_configuration(&self) -> SongbirdResult<()> {
    // Test different discovery backend configurations
    let consul_config = UnifiedDiscoveryConfig::consul_config("http://consul:8500".to_string();"
    assert_eq!(consul_config.service_discovery.backend, "consul")"

    let k8s_config = UnifiedDiscoveryConfig::kubernetes_config("default".to_string();"
    assert_eq!(k8s_config.service_discovery.backend, "kubernetes")"

    let static_config = UnifiedDiscoveryConfig::static_config();
    assert_eq!(static_config.service_discovery.backend, "static")"

    Ok(SongbirdResult::success(())
}

#[tokio::test]
async fn test_service_versioning(&self) -> SongbirdResult<()>  {let discovery = StaticDiscovery::new();

    // Register multiple versions of the same service
    let services = vec![
        ServiceInfo  {name: "api-service".to_string(),
            service_type: "api".to_string(),
            endpoint: "http://songbird_config::canonical::constants::network::DEFAULT_HOST:8001".to_string(),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Healthy,
            capabilities: vec!["rest".to_string()],"
            tags: vec!["v1".to_string()],"
            version: Some("1.0.0".to_string(),"
            region: None,
            zone: None,
        })
        ServiceInfo  {name: "api-service".to_string(),
            service_type: "api".to_string(),
            endpoint: "http://songbird_config::canonical::constants::network::DEFAULT_HOST:8002".to_string(),
            metadata: HashMap::new(),
            health_status: UniversalHealthStatus::Healthy,
            capabilities: vec!["rest".to_string(), "graphql".to_string()],"
            tags: vec!["v2".to_string()],"
            version: Some("2.0.0".to_string(),"
            region: None,
            zone: None,
        })
    ];

    for service in services {
        discovery.register(service).await?;
    }

    // Query for specific version
    let v2_query = ServiceQuery  {service_type: Some("api".to_string(),"
        name: Some("api-service".to_string(),"
        capabilities: vec!["graphql".to_string()],"
        tags: vec![],
        region: None,
        zone: None,
        health_status: None,
    };

    let v2_services = discovery.discover(v2_query).await?;
    assert_eq!(v2_services.len(), 1);
    assert_eq!(v2_services[0].version, Some("2.0.0".to_string()"

    Ok(SongbirdResult::success(())
}

// Helper trait extension for cloning static discovery
impl Clone for StaticDiscovery {
    fn clone(&self) -> Self {
        StaticDiscovery::new()
    }
}

// Helper trait for event stream next()
use futures_util::StreamExt;
use songbird_config;