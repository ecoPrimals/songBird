//! End-to-End Tests for Failure Detection and Recovery
//!
//! These tests validate that Songbird can detect failures and recover
//! gracefully without cascading failures.

use songbird_test_utils::{fixtures::*, mocks::*};
use songbird_types::{CapabilityRequest, HealthStatus};
use songbird_universal::UniversalCapabilityAdapter;
use std::time::Duration;
use tokio::time::sleep;

/// Test automatic detection of service failures
#[tokio::test]
async fn test_service_failure_detection() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Failed to register service");

    sleep(Duration::from_millis(100)).await;

    // Simulate service failure by marking it unhealthy
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to update health");

    sleep(Duration::from_millis(50)).await;

    // Verify failure is detected
    let health = adapter
        .get_service_health(&service_id)
        .await
        .expect("Failed to get health status");

    assert_eq!(
        health.status,
        HealthStatus::Unhealthy,
        "Service failure should be detected"
    );

    // Verify service is not included in discovery
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert!(
        !providers.iter().any(|p| p.id == service_id),
        "Unhealthy service should not be discovered"
    );
}

/// Test automatic service recovery
#[tokio::test]
async fn test_automatic_service_recovery() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Failed to register service");

    sleep(Duration::from_millis(100)).await;

    // Simulate temporary failure
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Simulate recovery
    adapter
        .update_service_health(&service_id, HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");

    sleep(Duration::from_millis(50)).await;

    // Verify service is back in rotation
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover providers");

    assert!(
        providers.iter().any(|p| p.id == service_id),
        "Recovered service should be discoverable"
    );
}

/// Test prevention of cascading failures
#[tokio::test]
async fn test_cascading_failure_prevention() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple services with dependencies
    let storage = storage_service_fixture();
    let compute1 = compute_service_fixture();
    let mut compute2 = compute_service_fixture();
    compute2.id = format!("{}_backup", compute2.id);

    adapter.register_service(storage.clone()).await.expect("Failed to register storage");
    adapter.register_service(compute1.clone()).await.expect("Failed to register compute1");
    adapter.register_service(compute2.clone()).await.expect("Failed to register compute2");

    sleep(Duration::from_millis(100)).await;

    // Simulate storage failure
    adapter
        .update_service_health(&storage.id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark storage unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Verify compute services are still healthy (not cascading)
    let compute1_health = adapter
        .get_service_health(&compute1.id)
        .await
        .expect("Failed to get compute1 health");

    let compute2_health = adapter
        .get_service_health(&compute2.id)
        .await
        .expect("Failed to get compute2 health");

    assert_eq!(
        compute1_health.status,
        HealthStatus::Healthy,
        "Compute1 should remain healthy"
    );

    assert_eq!(
        compute2_health.status,
        HealthStatus::Healthy,
        "Compute2 should remain healthy"
    );
}

/// Test circuit breaker activation on repeated failures
#[tokio::test]
async fn test_circuit_breaker_activation() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    adapter
        .register_service(service.clone())
        .await
        .expect("Failed to register service");

    sleep(Duration::from_millis(100)).await;

    // Execute multiple failing requests
    for _ in 0..5 {
        let request = CapabilityRequest {
            capability: "compute".to_string(),
            operation: "failing_operation".to_string(),
            parameters: Default::default(),
            timeout: Duration::from_secs(1),
        };

        let _ = adapter.execute_capability_request(request).await;
    }

    sleep(Duration::from_millis(100)).await;

    // Verify circuit breaker has opened
    let health = adapter
        .get_service_health(&service.id)
        .await
        .expect("Failed to get health");

    assert!(
        health.status == HealthStatus::Unhealthy || health.status == HealthStatus::Degraded,
        "Circuit breaker should have activated"
    );
}

/// Test graceful degradation when services fail
#[tokio::test]
async fn test_graceful_degradation() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register primary and fallback services
    let primary = ai_service_fixture();
    let mut fallback = ai_service_fixture();
    fallback.id = format!("{}_fallback", fallback.id);
    // Mark fallback as lower priority
    fallback.priority = Some(2);

    adapter.register_service(primary.clone()).await.expect("Failed to register primary");
    adapter.register_service(fallback.clone()).await.expect("Failed to register fallback");

    sleep(Duration::from_millis(100)).await;

    // Make primary unhealthy
    adapter
        .update_service_health(&primary.id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark primary unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Execute request - should use fallback
    let request = CapabilityRequest {
        capability: "ai_inference".to_string(),
        operation: "infer".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Failed to execute request");

    assert!(
        response.success,
        "Request should succeed with fallback service"
    );

    if let Some(provider_id) = response.data.get("provider_id") {
        assert_eq!(
            provider_id, &fallback.id,
            "Should use fallback service"
        );
    }
}

/// Test partial outage handling
#[tokio::test]
async fn test_partial_outage_handling() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple services of different types
    let compute1 = compute_service_fixture();
    let compute2 = {
        let mut c = compute_service_fixture();
        c.id = format!("{}_2", c.id);
        c
    };
    let storage = storage_service_fixture();
    let ai = ai_service_fixture();

    adapter.register_service(compute1.clone()).await.expect("Register compute1");
    adapter.register_service(compute2.clone()).await.expect("Register compute2");
    adapter.register_service(storage.clone()).await.expect("Register storage");
    adapter.register_service(ai.clone()).await.expect("Register AI");

    sleep(Duration::from_millis(100)).await;

    // Simulate partial outage (one compute service fails)
    adapter
        .update_service_health(&compute1.id, HealthStatus::Unhealthy)
        .await
        .expect("Mark compute1 unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Verify system continues to function
    let compute_request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "process".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let compute_response = adapter
        .execute_capability_request(compute_request)
        .await
        .expect("Compute request failed");

    assert!(
        compute_response.success,
        "Compute should still work with backup"
    );

    // Verify other services unaffected
    let storage_request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "read".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let storage_response = adapter
        .execute_capability_request(storage_request)
        .await
        .expect("Storage request failed");

    assert!(
        storage_response.success,
        "Storage should be unaffected by compute outage"
    );
}

/// Test network partition recovery
#[tokio::test]
async fn test_network_partition_recovery() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Failed to register service");

    sleep(Duration::from_millis(100)).await;

    // Simulate network partition
    adapter
        .update_service_health(&service_id, HealthStatus::Unreachable)
        .await
        .expect("Failed to mark unreachable");

    sleep(Duration::from_millis(50)).await;

    // Verify service is not used
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover");

    assert!(
        !providers.iter().any(|p| p.id == service_id),
        "Unreachable service should not be discovered"
    );

    // Simulate partition healing
    adapter
        .update_service_health(&service_id, HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");

    sleep(Duration::from_millis(50)).await;

    // Verify service is back
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Failed to discover");

    assert!(
        providers.iter().any(|p| p.id == service_id),
        "Service should be rediscovered after partition heals"
    );
}

/// Test state reconstruction after service restart
#[tokio::test]
async fn test_state_reconstruction() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = storage_service_fixture();
    let service_id = service.id.clone();

    // Register service and store some state
    adapter
        .register_service(service.clone())
        .await
        .expect("Failed to register");

    sleep(Duration::from_millis(100)).await;

    let store_request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "store".to_string(),
        parameters: vec![
            ("key".to_string(), "test_key".to_string()),
            ("value".to_string(), "test_value".to_string()),
        ]
        .into_iter()
        .collect(),
        timeout: Duration::from_secs(10),
    };

    adapter
        .execute_capability_request(store_request)
        .await
        .expect("Failed to store data");

    // Simulate service restart
    adapter
        .deregister_service(&service_id)
        .await
        .expect("Failed to deregister");

    sleep(Duration::from_millis(100)).await;

    adapter
        .register_service(service)
        .await
        .expect("Failed to re-register");

    sleep(Duration::from_millis(100)).await;

    // Verify state is accessible after restart
    let retrieve_request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "retrieve".to_string(),
        parameters: vec![("key".to_string(), "test_key".to_string())]
            .into_iter()
            .collect(),
        timeout: Duration::from_secs(10),
    };

    let response = adapter
        .execute_capability_request(retrieve_request)
        .await
        .expect("Failed to retrieve");

    assert!(response.success, "Should retrieve stored data after restart");
    assert_eq!(
        response.data.get("value"),
        Some(&"test_value".to_string()),
        "Should have correct value"
    );
}

