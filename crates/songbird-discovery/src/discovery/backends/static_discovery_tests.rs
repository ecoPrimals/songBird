// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for static service discovery
//!
//! Tests cover all functionality with edge cases and concurrent access patterns.

use super::*;
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::service::ServiceStatus;
use chrono::Utc;
use std::collections::HashMap;

/// Helper to create a test service
fn create_test_service(id: &str, name: &str) -> ServiceInfo {
    ServiceInfo {
        service_id: id.to_string(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: Some(format!("Test service {name}")),
        endpoints: vec![], // Empty for static discovery tests
        health_check_endpoint: Some("/health".to_string()),
        metadata: HashMap::new(),
        tags: vec![],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        instance_id: format!("{id}-instance"),
        host: format!("{name}.local"),
        port: 8080,
    }
}

#[tokio::test]
async fn test_new_discovery_is_empty() {
    let discovery = StaticServiceDiscovery::new();
    let count = discovery.service_count().await;
    assert_eq!(count, 0, "New discovery should have no services");
}

#[tokio::test]
async fn test_default_discovery_is_empty() {
    let discovery = StaticServiceDiscovery::default();
    let count = discovery.service_count().await;
    assert_eq!(count, 0, "Default discovery should have no services");
}

#[tokio::test]
async fn test_with_services_prepopulates() {
    let services = vec![
        create_test_service("svc1", "service1"),
        create_test_service("svc2", "service2"),
        create_test_service("svc3", "service3"),
    ];

    let discovery = StaticServiceDiscovery::with_services(services).await;
    let count = discovery.service_count().await;
    assert_eq!(count, 3, "Discovery should have 3 prepopulated services");
}

#[tokio::test]
async fn test_register_single_service() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");

    let result = discovery.register(service.clone()).await;
    assert!(result.is_ok(), "Registration should succeed");

    let count = discovery.service_count().await;
    assert_eq!(count, 1, "Should have one service after registration");

    let has_service = discovery.has_service("svc1").await;
    assert!(has_service, "Should find registered service by ID");
}

#[tokio::test]
async fn test_register_multiple_services() {
    let discovery = StaticServiceDiscovery::new();

    for i in 1..=5 {
        let service = create_test_service(&format!("svc{i}"), &format!("service{i}"));
        discovery.register(service).await.unwrap();
    }

    let count = discovery.service_count().await;
    assert_eq!(count, 5, "Should have 5 registered services");
}

#[tokio::test]
async fn test_register_overwrites_existing() {
    let discovery = StaticServiceDiscovery::new();
    let service1 = create_test_service("svc1", "original");
    let mut service2 = create_test_service("svc1", "updated");
    service2.version = "2.0.0".to_string();

    discovery.register(service1).await.unwrap();
    discovery.register(service2).await.unwrap();

    let count = discovery.service_count().await;
    assert_eq!(count, 1, "Duplicate ID should overwrite, not add");

    let all_services = discovery.get_all_services().await;
    assert_eq!(all_services[0].name, "updated", "Should have updated service");
    assert_eq!(all_services[0].version, "2.0.0", "Version should be updated");
}

#[tokio::test]
async fn test_unregister_existing_service() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");

    discovery.register(service).await.unwrap();
    assert_eq!(discovery.service_count().await, 1);

    let result = discovery.unregister("svc1").await;
    assert!(result.is_ok(), "Unregistration should succeed");

    let count = discovery.service_count().await;
    assert_eq!(count, 0, "Service should be removed");

    let has_service = discovery.has_service("svc1").await;
    assert!(!has_service, "Should not find unregistered service");
}

#[tokio::test]
async fn test_unregister_nonexistent_service() {
    let discovery = StaticServiceDiscovery::new();

    let result = discovery.unregister("nonexistent").await;
    assert!(result.is_ok(), "Unregistering nonexistent service should not error");

    let count = discovery.service_count().await;
    assert_eq!(count, 0, "Count should remain 0");
}

#[tokio::test]
async fn test_discover_all_services() {
    let discovery = StaticServiceDiscovery::new();

    for i in 1..=3 {
        let service = create_test_service(&format!("svc{i}"), &format!("service{i}"));
        discovery.register(service).await.unwrap();
    }

    let query = ServiceQuery::new();
    let found = discovery.discover(query).await.unwrap();

    assert_eq!(found.len(), 3, "Should discover all services with empty query");
}

#[tokio::test]
async fn test_discover_by_name() {
    let discovery = StaticServiceDiscovery::new();

    let service1 = create_test_service("svc1", "alpha-service");
    let service2 = create_test_service("svc2", "beta-service");
    let service3 = create_test_service("svc3", "alpha-service");

    discovery.register(service1).await.unwrap();
    discovery.register(service2).await.unwrap();
    discovery.register(service3).await.unwrap();

    let query = ServiceQuery {
        name: Some("alpha-service".to_string()),
        ..Default::default()
    };

    let found = discovery.discover(query).await.unwrap();
    assert_eq!(found.len(), 2, "Should find 2 services named 'alpha-service'");

    for service in &found {
        assert_eq!(service.name, "alpha-service");
    }
}

#[tokio::test]
async fn test_discover_nonexistent_name() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");
    discovery.register(service).await.unwrap();

    let query = ServiceQuery {
        name: Some("nonexistent".to_string()),
        ..Default::default()
    };

    let found = discovery.discover(query).await.unwrap();
    assert_eq!(found.len(), 0, "Should find no services with nonexistent name");
}

#[tokio::test]
async fn test_list_all_services() {
    let discovery = StaticServiceDiscovery::new();

    for i in 1..=4 {
        let service = create_test_service(&format!("svc{i}"), &format!("service{i}"));
        discovery.register(service).await.unwrap();
    }

    let all = discovery.list_all().await.unwrap();
    assert_eq!(all.len(), 4, "list_all should return all services");
}

#[tokio::test]
async fn test_exists_for_registered_service() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");

    discovery.register(service).await.unwrap();

    let exists = discovery.exists("svc1").await.unwrap();
    assert!(exists, "Registered service should exist");
}

#[tokio::test]
async fn test_exists_for_nonexistent_service() {
    let discovery = StaticServiceDiscovery::new();

    let exists = discovery.exists("nonexistent").await.unwrap();
    assert!(!exists, "Nonexistent service should not exist");
}

#[tokio::test]
async fn test_is_registered() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");

    let registered_before = discovery.is_registered("svc1").await.unwrap();
    assert!(!registered_before, "Service should not be registered initially");

    discovery.register(service).await.unwrap();

    let registered_after = discovery.is_registered("svc1").await.unwrap();
    assert!(registered_after, "Service should be registered after registration");
}

#[tokio::test]
async fn test_update_health_succeeds() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");
    discovery.register(service).await.unwrap();

    let result = discovery.update_health("svc1", ServiceHealthStatus::Healthy).await;
    assert!(result.is_ok(), "Health update should succeed");
}

#[tokio::test]
async fn test_update_health_nonexistent_service() {
    let discovery = StaticServiceDiscovery::new();

    let result = discovery.update_health("nonexistent", ServiceHealthStatus::Unhealthy).await;
    assert!(result.is_ok(), "Health update should not error for nonexistent service");
}

#[tokio::test]
async fn test_update_metadata() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");
    discovery.register(service).await.unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());

    let result = discovery.update_metadata("svc1", metadata.clone()).await;
    assert!(result.is_ok(), "Metadata update should succeed");

    // Verify metadata was updated
    let services = discovery.get_all_services().await;
    let service = &services[0];
    assert_eq!(
        service.metadata.get("key1"),
        Some(&serde_json::Value::String("value1".to_string()))
    );
    assert_eq!(
        service.metadata.get("key2"),
        Some(&serde_json::Value::String("value2".to_string()))
    );
}

#[tokio::test]
async fn test_update_metadata_nonexistent_service() {
    let discovery = StaticServiceDiscovery::new();
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());

    let result = discovery.update_metadata("nonexistent", metadata).await;
    assert!(result.is_ok(), "Metadata update should not error for nonexistent service");
}

#[tokio::test]
async fn test_update_metadata_extends_existing() {
    let discovery = StaticServiceDiscovery::new();
    let mut service = create_test_service("svc1", "test-service");
    service.metadata.insert("existing".to_string(), serde_json::Value::String("old".to_string()));
    discovery.register(service).await.unwrap();

    let mut new_metadata = HashMap::new();
    new_metadata.insert("new_key".to_string(), "new_value".to_string());

    discovery.update_metadata("svc1", new_metadata).await.unwrap();

    let services = discovery.get_all_services().await;
    let service = &services[0];

    // Should have both old and new metadata
    assert!(service.metadata.contains_key("existing"));
    assert!(service.metadata.contains_key("new_key"));
}

#[tokio::test]
async fn test_clear_removes_all_services() {
    let discovery = StaticServiceDiscovery::new();

    for i in 1..=5 {
        let service = create_test_service(&format!("svc{i}"), &format!("service{i}"));
        discovery.register(service).await.unwrap();
    }

    assert_eq!(discovery.service_count().await, 5);

    discovery.clear().await;

    assert_eq!(discovery.service_count().await, 0, "Clear should remove all services");

    let all = discovery.get_all_services().await;
    assert!(all.is_empty(), "Should have no services after clear");
}

#[tokio::test]
async fn test_get_all_services_returns_clones() {
    let discovery = StaticServiceDiscovery::new();
    let service = create_test_service("svc1", "test-service");
    discovery.register(service).await.unwrap();

    let services1 = discovery.get_all_services().await;
    let services2 = discovery.get_all_services().await;

    assert_eq!(services1.len(), services2.len());
    assert_eq!(services1[0].service_id, services2[0].service_id);
}

#[tokio::test]
async fn test_concurrent_registration() {
    let discovery = Arc::new(StaticServiceDiscovery::new());
    let mut handles = vec![];

    // Spawn 10 concurrent registration tasks
    for i in 0..10 {
        let discovery_clone = Arc::clone(&discovery);
        let handle = tokio::spawn(async move {
            let service = create_test_service(&format!("svc{i}"), &format!("service{i}"));
            discovery_clone.register(service).await
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    let count = discovery.service_count().await;
    assert_eq!(count, 10, "All concurrent registrations should succeed");
}

#[tokio::test]
async fn test_concurrent_reads() {
    let services =
        vec![create_test_service("svc1", "service1"), create_test_service("svc2", "service2")];
    let discovery = Arc::new(StaticServiceDiscovery::with_services(services).await);
    let mut handles = vec![];

    // Spawn 20 concurrent read tasks
    for _ in 0..20 {
        let discovery_clone = Arc::clone(&discovery);
        let handle = tokio::spawn(async move {
            let count = discovery_clone.service_count().await;
            let all = discovery_clone.get_all_services().await;
            (count, all.len())
        });
        handles.push(handle);
    }

    // All reads should succeed and return consistent results
    for handle in handles {
        let (count, all_len) = handle.await.unwrap();
        assert_eq!(count, 2);
        assert_eq!(all_len, 2);
    }
}

#[tokio::test]
async fn test_watch_returns_empty_stream() {
    let discovery = StaticServiceDiscovery::new();
    let query = ServiceQuery::new();

    let result = discovery.watch(query).await;
    assert!(result.is_ok(), "Watch should return Ok with empty stream");
}

#[tokio::test]
async fn test_as_any_returns_self() {
    let discovery = StaticServiceDiscovery::new();
    let any = discovery.as_any();
    assert!(any.is::<StaticServiceDiscovery>(), "as_any should return self type");
}

#[tokio::test]
async fn test_empty_discovery_operations() {
    let discovery = StaticServiceDiscovery::new();

    // All operations should work on empty discovery
    assert_eq!(discovery.service_count().await, 0);
    assert!(discovery.get_all_services().await.is_empty());
    assert!(!discovery.has_service("any").await);

    let query = ServiceQuery::new();
    let found = discovery.discover(query).await.unwrap();
    assert!(found.is_empty());

    let all = discovery.list_all().await.unwrap();
    assert!(all.is_empty());
}

#[tokio::test]
async fn test_service_lifecycle() {
    let discovery = StaticServiceDiscovery::new();
    let service_id = "lifecycle-test";
    let service = create_test_service(service_id, "test-service");

    // 1. Initially not present
    assert!(!discovery.has_service(service_id).await);
    assert!(!discovery.is_registered(service_id).await.unwrap());

    // 2. Register
    discovery.register(service).await.unwrap();
    assert!(discovery.has_service(service_id).await);
    assert!(discovery.is_registered(service_id).await.unwrap());

    // 3. Update metadata
    let mut metadata = HashMap::new();
    metadata.insert("status".to_string(), "active".to_string());
    discovery.update_metadata(service_id, metadata).await.unwrap();

    // 4. Update health
    discovery.update_health(service_id, ServiceHealthStatus::Healthy).await.unwrap();

    // 5. Unregister
    discovery.unregister(service_id).await.unwrap();
    assert!(!discovery.has_service(service_id).await);
    assert!(!discovery.is_registered(service_id).await.unwrap());
}
