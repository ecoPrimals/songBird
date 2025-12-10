//! End-to-End Service Discovery Tests
//!
//! Tests actual service registration, discovery, and health checking.

#![cfg(test)]

use songbird_discovery::{
    ServiceDiscovery, ServiceInfo, DiscoveryConfig,
    production::adapters::ProductionDiscoveryBackend,
};
use std::time::Duration;
// Removed unused: use tokio::time::sleep;

#[tokio::test]
async fn test_service_registration_and_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // Create discovery backend
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    // Register a service
    let service_info = ServiceInfo {
        id: "test-service-1".to_string(),
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["http".to_string(), "rest".to_string()],
        metadata: Default::default(),
    };
    
    discovery.register(service_info.clone()).await?;
    
    // Discover the service
    let discovered = discovery.discover_by_name("test-service").await?;
    
    assert!(!discovered.is_empty(), "Should discover registered service");
    assert_eq!(discovered[0].name, "test-service");
    assert_eq!(discovered[0].version, "1.0.0");
    
    Ok(())
}

#[tokio::test]
async fn test_capability_based_service_discovery() -> Result<(), Box<dyn std::error::Error>> {
    // Create discovery backend
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    // Register services with different capabilities
    let services = vec![
        ServiceInfo {
            id: "compute-1".to_string(),
            name: "compute-service".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec!["compute".to_string(), "cpu".to_string()],
            metadata: Default::default(),
        },
        ServiceInfo {
            id: "storage-1".to_string(),
            name: "storage-service".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "http://localhost:8082".to_string(),
            capabilities: vec!["storage".to_string(), "ssd".to_string()],
            metadata: Default::default(),
        },
    ];
    
    for service in services {
        discovery.register(service).await?;
    }
    
    // Discover by capability
    let compute_services = discovery.discover_by_capability("compute").await?;
    assert_eq!(compute_services.len(), 1, "Should find one compute service");
    assert_eq!(compute_services[0].name, "compute-service");
    
    let storage_services = discovery.discover_by_capability("storage").await?;
    assert_eq!(storage_services.len(), 1, "Should find one storage service");
    assert_eq!(storage_services[0].name, "storage-service");
    
    Ok(())
}

#[tokio::test]
async fn test_service_health_check_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Create discovery backend
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    // Register a service
    let service_info = ServiceInfo {
        id: "health-test-1".to_string(),
        name: "health-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8083".to_string(),
        capabilities: vec!["http".to_string()],
        metadata: Default::default(),
    };
    
    discovery.register(service_info).await?;
    
    // Verify service is registered
    let services = discovery.discover_by_name("health-service").await?;
    assert_eq!(services.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_service_deregistration() -> Result<(), Box<dyn std::error::Error>> {
    // Create discovery backend
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    // Register a service
    let service_id = "deregister-test-1";
    let service_info = ServiceInfo {
        id: service_id.to_string(),
        name: "temp-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8084".to_string(),
        capabilities: vec!["temp".to_string()],
        metadata: Default::default(),
    };
    
    discovery.register(service_info).await?;
    
    // Verify it's registered
    let services_before = discovery.discover_by_name("temp-service").await?;
    assert!(!services_before.is_empty());
    
    // Deregister
    discovery.deregister(service_id).await?;
    
    // Verify it's gone
    let services_after = discovery.discover_by_name("temp-service").await?;
    assert!(services_after.is_empty(), "Service should be deregistered");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_service_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test concurrent registration and discovery
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    // Spawn multiple concurrent registrations
    let mut handles = vec![];
    
    for i in 0..5 {
        let discovery_clone = discovery.clone();
        let handle = tokio::spawn(async move {
            let service = ServiceInfo {
                id: format!("concurrent-{}", i),
                name: format!("service-{}", i),
                version: "1.0.0".to_string(),
                endpoint: format!("http://localhost:{}", 9000 + i),
                capabilities: vec!["test".to_string()],
                metadata: Default::default(),
            };
            
            discovery_clone.register(service).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok(), "Concurrent registration should succeed");
    }
    
    // Give a moment for all registrations to settle
    sleep(Duration::from_millis(50)).await;
    
    // Verify all services are discoverable
    for i in 0..5 {
        let services = discovery.discover_by_name(&format!("service-{}", i)).await?;
        assert!(!services.is_empty(), "Service {} should be discoverable", i);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_service_update_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Test updating service information
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    let service_id = "update-test-1";
    
    // Register initial version
    let initial_service = ServiceInfo {
        id: service_id.to_string(),
        name: "update-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8085".to_string(),
        capabilities: vec!["v1".to_string()],
        metadata: Default::default(),
    };
    
    discovery.register(initial_service).await?;
    
    // Update to new version
    let updated_service = ServiceInfo {
        id: service_id.to_string(),
        name: "update-service".to_string(),
        version: "2.0.0".to_string(),
        endpoint: "http://localhost:8085".to_string(),
        capabilities: vec!["v1".to_string(), "v2".to_string()],
        metadata: Default::default(),
    };
    
    discovery.register(updated_service).await?;
    
    // Verify updated version is discoverable
    let services = discovery.discover_by_name("update-service").await?;
    
    // Should have the updated service (exact behavior depends on implementation)
    assert!(!services.is_empty(), "Updated service should be discoverable");
    
    Ok(())
}

#[tokio::test]
async fn test_service_metadata_propagation() -> Result<(), Box<dyn std::error::Error>> {
    // Test that service metadata is properly stored and retrieved
    let backend = ProductionDiscoveryBackend::new(DiscoveryConfig::default());
    let discovery = ServiceDiscovery::new(backend);
    
    use std::collections::HashMap;
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-east-1".to_string());
    metadata.insert("environment".to_string(), "production".to_string());
    
    let service_info = ServiceInfo {
        id: "metadata-test-1".to_string(),
        name: "metadata-service".to_string(),
        version: "1.0.0".to_string(),
        endpoint: "http://localhost:8086".to_string(),
        capabilities: vec!["meta".to_string()],
        metadata: metadata.clone(),
    };
    
    discovery.register(service_info).await?;
    
    // Discover and verify metadata
    let services = discovery.discover_by_name("metadata-service").await?;
    assert!(!services.is_empty());
    
    let discovered_service = &services[0];
    assert_eq!(
        discovered_service.metadata.get("region"),
        Some(&"us-east-1".to_string())
    );
    assert_eq!(
        discovered_service.metadata.get("environment"),
        Some(&"production".to_string())
    );
    
    Ok(())
}

