//! End-to-End Tests for Multi-Service Coordination
//!
//! These tests validate that multiple services can coordinate effectively
//! through the Songbird orchestration layer.

use songbird_test_utils::{fixtures::*, mocks::*};
use songbird_types::{CapabilityRequest, CapabilityResponse, HealthStatus};
use songbird_universal::UniversalCapabilityAdapter;
use std::time::Duration;
use tokio::time::sleep;

/// Test that compute and storage services can coordinate on a workflow
#[tokio::test]
async fn test_compute_storage_workflow() {
    // Setup environment with compute and storage services
    let env = create_healthy_environment();
    
    // Create adapter
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register compute service
    let compute_service = compute_service_fixture();
    adapter.register_service(compute_service.clone()).await
        .expect("Failed to register compute service");

    // Register storage service
    let storage_service = storage_service_fixture();
    adapter.register_service(storage_service.clone()).await
        .expect("Failed to register storage service");

    // Wait for service registration to propagate
    sleep(Duration::from_millis(100)).await;

    // Discover compute providers
    let compute_providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover compute providers");

    assert!(
        !compute_providers.is_empty(),
        "Should discover at least one compute provider"
    );

    // Discover storage providers
    let storage_providers = adapter
        .discover_capability_providers("storage")
        .await
        .expect("Failed to discover storage providers");

    assert!(
        !storage_providers.is_empty(),
        "Should discover at least one storage provider"
    );

    // Execute compute task that requires storage
    let compute_request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "process_data".to_string(),
        parameters: vec![("storage_provider".to_string(), storage_providers[0].id.clone())]
            .into_iter()
            .collect(),
        timeout: Duration::from_secs(30),
    };

    let compute_response = adapter
        .execute_capability_request(compute_request)
        .await
        .expect("Failed to execute compute request");

    assert!(
        compute_response.success,
        "Compute request should succeed: {:?}",
        compute_response.error
    );

    // Verify storage was accessed
    let storage_request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "get_access_log".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let storage_response = adapter
        .execute_capability_request(storage_request)
        .await
        .expect("Failed to execute storage request");

    assert!(
        storage_response.success,
        "Storage access log request should succeed"
    );
}

/// Test that AI and security services can coordinate for authenticated inference
#[tokio::test]
async fn test_ai_security_coordination() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register AI service
    let ai_service = ai_service_fixture();
    adapter.register_service(ai_service.clone()).await
        .expect("Failed to register AI service");

    // Register security service
    let security_service = security_service_fixture();
    adapter.register_service(security_service.clone()).await
        .expect("Failed to register security service");

    sleep(Duration::from_millis(100)).await;

    // First, authenticate
    let auth_request = CapabilityRequest {
        capability: "authentication".to_string(),
        operation: "authenticate".to_string(),
        parameters: vec![
            ("username".to_string(), "test_user".to_string()),
            ("password".to_string(), "test_pass".to_string()),
        ]
        .into_iter()
        .collect(),
        timeout: Duration::from_secs(10),
    };

    let auth_response = adapter
        .execute_capability_request(auth_request)
        .await
        .expect("Failed to authenticate");

    assert!(auth_response.success, "Authentication should succeed");

    let token = auth_response
        .data
        .get("token")
        .expect("Should receive auth token");

    // Then, use token for AI inference
    let inference_request = CapabilityRequest {
        capability: "ai_inference".to_string(),
        operation: "infer".to_string(),
        parameters: vec![
            ("auth_token".to_string(), token.clone()),
            ("model".to_string(), "test_model".to_string()),
            ("input".to_string(), "test input data".to_string()),
        ]
        .into_iter()
        .collect(),
        timeout: Duration::from_secs(30),
    };

    let inference_response = adapter
        .execute_capability_request(inference_request)
        .await
        .expect("Failed to execute inference");

    assert!(
        inference_response.success,
        "Authenticated inference should succeed"
    );
}

/// Test complete service mesh with all service types
#[tokio::test]
async fn test_complete_service_mesh() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register all service types
    let services = vec![
        compute_service_fixture(),
        storage_service_fixture(),
        ai_service_fixture(),
        security_service_fixture(),
    ];

    for service in &services {
        adapter
            .register_service(service.clone())
            .await
            .expect("Failed to register service");
    }

    sleep(Duration::from_millis(200)).await;

    // Discover all capabilities
    let capabilities = vec!["compute", "storage", "ai_inference", "authentication"];

    for capability in capabilities {
        let providers = adapter
            .discover_capability_providers(capability)
            .await
            .expect(&format!("Failed to discover {} providers", capability));

        assert!(
            !providers.is_empty(),
            "Should discover at least one {} provider",
            capability
        );
    }

    // Verify health status of all services
    let health_statuses = adapter
        .get_all_service_health()
        .await
        .expect("Failed to get health statuses");

    assert_eq!(
        health_statuses.len(),
        services.len(),
        "Should have health status for all services"
    );

    for (service, health) in services.iter().zip(health_statuses.iter()) {
        assert_eq!(
            health.status,
            HealthStatus::Healthy,
            "Service {} should be healthy",
            service.id
        );
    }
}

/// Test dynamic service discovery as services come online
#[tokio::test]
async fn test_dynamic_service_discovery() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Initially, no services
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert!(
        providers.is_empty(),
        "Should have no providers initially"
    );

    // Add first compute service
    let compute1 = compute_service_fixture();
    adapter
        .register_service(compute1.clone())
        .await
        .expect("Failed to register first compute service");

    sleep(Duration::from_millis(100)).await;

    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert_eq!(providers.len(), 1, "Should have one provider");

    // Add second compute service
    let mut compute2 = compute_service_fixture();
    compute2.id = format!("{}_2", compute2.id);
    adapter
        .register_service(compute2.clone())
        .await
        .expect("Failed to register second compute service");

    sleep(Duration::from_millis(100)).await;

    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert_eq!(providers.len(), 2, "Should have two providers");

    // Remove first service
    adapter
        .deregister_service(&compute1.id)
        .await
        .expect("Failed to deregister first service");

    sleep(Duration::from_millis(100)).await;

    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert_eq!(providers.len(), 1, "Should have one provider after deregistration");
    assert_eq!(providers[0].id, compute2.id, "Should be the second service");
}

/// Test service registration and health check propagation
#[tokio::test]
async fn test_service_registration_flow() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    // Register service
    adapter
        .register_service(service.clone())
        .await
        .expect("Failed to register service");

    // Verify service is discoverable
    sleep(Duration::from_millis(100)).await;

    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert!(
        providers.iter().any(|p| p.id == service_id),
        "Service should be discoverable"
    );

    // Verify health status is tracked
    let health = adapter
        .get_service_health(&service_id)
        .await
        .expect("Failed to get service health");

    assert_eq!(
        health.status,
        HealthStatus::Healthy,
        "Service should be healthy"
    );

    // Update health status
    adapter
        .update_service_health(&service_id, HealthStatus::Degraded)
        .await
        .expect("Failed to update health status");

    sleep(Duration::from_millis(50)).await;

    let health = adapter
        .get_service_health(&service_id)
        .await
        .expect("Failed to get updated health");

    assert_eq!(
        health.status,
        HealthStatus::Degraded,
        "Health status should be updated"
    );
}

/// Test load balancing across multiple providers
#[tokio::test]
async fn test_load_balancing() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple compute providers
    for i in 0..3 {
        let mut service = compute_service_fixture();
        service.id = format!("compute_service_{}", i);
        adapter
            .register_service(service)
            .await
            .expect("Failed to register service");
    }

    sleep(Duration::from_millis(200)).await;

    // Execute multiple requests and track which providers handle them
    let mut provider_usage = std::collections::HashMap::new();

    for _ in 0..30 {
        let request = CapabilityRequest {
            capability: "compute".to_string(),
            operation: "process".to_string(),
            parameters: Default::default(),
            timeout: Duration::from_secs(10),
        };

        let response = adapter
            .execute_capability_request(request)
            .await
            .expect("Failed to execute request");

        if let Some(provider_id) = response.data.get("provider_id") {
            *provider_usage.entry(provider_id.clone()).or_insert(0) += 1;
        }
    }

    // Verify requests are distributed across providers
    assert!(
        provider_usage.len() >= 2,
        "Requests should be distributed across multiple providers"
    );

    // Each provider should handle some requests (allowing for variance)
    for (provider, count) in &provider_usage {
        assert!(
            *count > 0,
            "Provider {} should handle at least some requests",
            provider
        );
    }
}

/// Test service failover when a provider becomes unhealthy
#[tokio::test]
async fn test_service_failover() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register two compute providers
    let service1 = compute_service_fixture();
    let service1_id = service1.id.clone();
    adapter
        .register_service(service1)
        .await
        .expect("Failed to register service 1");

    let mut service2 = compute_service_fixture();
    service2.id = format!("{}_backup", service2.id);
    let service2_id = service2.id.clone();
    adapter
        .register_service(service2)
        .await
        .expect("Failed to register service 2");

    sleep(Duration::from_millis(100)).await;

    // Mark first service as unhealthy
    adapter
        .update_service_health(&service1_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark service as unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Execute request - should automatically fail over to healthy service
    let request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "process".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Failed to execute request");

    assert!(
        response.success,
        "Request should succeed despite one unhealthy service"
    );

    if let Some(provider_id) = response.data.get("provider_id") {
        assert_eq!(
            provider_id, &service2_id,
            "Request should be routed to healthy service"
        );
    }
}

/// Test multi-tier architecture with service dependencies
#[tokio::test]
async fn test_multi_tier_architecture() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Tier 1: Storage (foundation)
    let storage = storage_service_fixture();
    adapter
        .register_service(storage)
        .await
        .expect("Failed to register storage");

    // Tier 2: Compute (depends on storage)
    let compute = compute_service_fixture();
    adapter
        .register_service(compute)
        .await
        .expect("Failed to register compute");

    // Tier 3: AI (depends on compute and storage)
    let ai = ai_service_fixture();
    adapter
        .register_service(ai)
        .await
        .expect("Failed to register AI");

    sleep(Duration::from_millis(200)).await;

    // Execute AI request that cascades through tiers
    let request = CapabilityRequest {
        capability: "ai_inference".to_string(),
        operation: "complex_analysis".to_string(),
        parameters: vec![
            ("use_storage".to_string(), "true".to_string()),
            ("use_compute".to_string(), "true".to_string()),
        ]
        .into_iter()
        .collect(),
        timeout: Duration::from_secs(60),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Failed to execute multi-tier request");

    assert!(
        response.success,
        "Multi-tier request should succeed"
    );

    // Verify all tiers were involved
    assert!(
        response.data.contains_key("storage_accessed"),
        "Should have accessed storage tier"
    );
    assert!(
        response.data.contains_key("compute_used"),
        "Should have used compute tier"
    );
    assert!(
        response.data.contains_key("ai_inference_completed"),
        "Should have completed AI inference"
    );
}

