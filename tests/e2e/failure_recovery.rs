// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals
//! End-to-End Tests for Failure Detection and Recovery
//!
//! These tests validate that Songbird can detect failures and recover
//! gracefully without cascading failures.

use songbird_test_utils::{fixtures::*, mocks::*};
use songbird_types::HealthStatus;
use songbird_universal::UniversalCapabilityAdapter;
use songbird_universal::test_helpers::register_test_service;
use songbird_universal::capabilities::{SimpleCapabilityRequest, SimpleCapabilityResponse};
use std::time::Duration;

/// Test automatic detection of service failures
#[tokio::test]
async fn test_service_failure_detection() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service("test-compute");
    let service_id = service.id().to_string();

    let _handle = register_test_service(&adapter, service)
        .await
        .expect("Failed to register service");
    // ✅ Event-driven: No sleep needed!

    // Simulate service failure by marking it unhealthy
    adapter
        .update_service_health(&service_id, crate::types::HealthStatus::Unhealthy)
        .await
        .expect("Failed to update health");
    // ✅ Event-driven: No sleep needed!

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

    let service = compute_service("test-compute-recovery");
    let service_id = service.id().to_string();

    let _handle = register_test_service(&adapter, service)
        .await
        .expect("Failed to register service");
    // ✅ Event-driven: No sleep needed!

    // Simulate temporary failure
    adapter
        .update_service_health(&service_id, crate::types::HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");
    // ✅ Event-driven: No sleep needed!

    // Simulate recovery
    adapter
        .update_service_health(&service_id, crate::types::HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");
    // ✅ Event-driven: No sleep needed!

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
    let storage = storage_service("test-storage");
    let compute1 = compute_service("test-compute-1");
    let compute2 = compute_service("test-compute-2-backup");

    let storage_id = storage.id().to_string();
    let compute1_id = compute1.id().to_string();
    let compute2_id = compute2.id().to_string();

    let _h1 = register_test_service(&adapter, storage).await.expect("Failed to register storage");
    let _h2 = register_test_service(&adapter, compute1).await.expect("Failed to register compute1");
    let _h3 = register_test_service(&adapter, compute2).await.expect("Failed to register compute2");
    // ✅ Event-driven: All registered and ready!

    // Simulate storage failure
    adapter
        .update_service_health(&storage_id, crate::types::HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark storage unhealthy");
    // ✅ Event-driven: No sleep needed!

    // Verify compute services are still healthy (not cascading)
    let compute1_health = adapter
        .get_service_health(&compute1_id)
        .await
        .expect("Failed to get compute1 health");

    let compute2_health = adapter
        .get_service_health(&compute2_id)
        .await
        .expect("Failed to get compute2 health");

    assert_eq!(
        compute1_health.status,
        crate::types::HealthStatus::Healthy,
        "Compute1 should remain healthy"
    );

    assert_eq!(
        compute2_health.status,
        crate::types::HealthStatus::Healthy,
        "Compute2 should remain healthy"
    );
}

/// Test circuit breaker activation on repeated failures
#[tokio::test]
async fn test_circuit_breaker_activation() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service("test-compute-cb");
    let service_id = service.id().to_string();

    let _handle = register_test_service(&adapter, service)
        .await
        .expect("Failed to register service");
    // ✅ Event-driven: Ready immediately!

    // Execute multiple failing requests
    for _ in 0..5 {
        let request = crate::capabilities::SimpleCapabilityRequest {
            capability: "compute".to_string(),
            operation: "failing_operation".to_string(),
            parameters: Default::default(),
        };

        let _ = adapter.execute_capability_request(request).await;
    }
    // ✅ Event-driven: No sleep needed after requests!

    // Verify circuit breaker has opened
    let health = adapter
        .get_service_health(&service_id)
        .await
        .expect("Failed to get health");

    assert!(
        health.status == crate::types::HealthStatus::Unhealthy || health.status == crate::types::HealthStatus::Degraded,
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
    let primary = ai_service("test-ai-primary");
    let fallback = ai_service("test-ai-fallback");

    let primary_id = primary.id().to_string();

    let _h1 = register_test_service(&adapter, primary).await.expect("Failed to register primary");
    let _h2 = register_test_service(&adapter, fallback).await.expect("Failed to register fallback");
    // ✅ Event-driven: Both services ready!

    // Make primary unhealthy
    adapter
        .update_service_health(&primary_id, crate::types::HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark primary unhealthy");
    // ✅ Event-driven: Immediate health change!

    // Execute request - should use fallback
    let request = crate::capabilities::SimpleCapabilityRequest {
        capability: "ai_inference".to_string(),
        operation: "infer".to_string(),
        parameters: Default::default(),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Failed to execute request");

    assert!(
        response.success,
        "Request should succeed with fallback service"
    );
}

/// Test partial outage handling
#[tokio::test]
async fn test_partial_outage_handling() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple services of different types
    let compute1 = compute_service("test-compute-partial-1");
    let compute2 = compute_service("test-compute-partial-2");
    let storage = storage_service("test-storage-partial");
    let ai = ai_service("test-ai-partial");

    let compute1_id = compute1.id().to_string();

    let _h1 = register_test_service(&adapter, compute1).await.expect("Register compute1");
    let _h2 = register_test_service(&adapter, compute2).await.expect("Register compute2");
    let _h3 = register_test_service(&adapter, storage).await.expect("Register storage");
    let _h4 = register_test_service(&adapter, ai).await.expect("Register AI");
    // ✅ Event-driven: All services ready!

    // Simulate partial outage (one compute service fails)
    adapter
        .update_service_health(&compute1_id, crate::types::HealthStatus::Unhealthy)
        .await
        .expect("Mark compute1 unhealthy");
    // ✅ Event-driven: Immediate health change!

    // Verify system continues to function
    let compute_request = crate::capabilities::SimpleCapabilityRequest {
        capability: "compute".to_string(),
        operation: "process".to_string(),
        parameters: Default::default(),
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
    let storage_request = crate::capabilities::SimpleCapabilityRequest {
        capability: "storage".to_string(),
        operation: "read".to_string(),
        parameters: Default::default(),
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

    let service = compute_service("test-compute-partition");
    let service_id = service.id().to_string();

    let _handle = register_test_service(&adapter, service)
        .await
        .expect("Failed to register service");
    // ✅ Event-driven: Ready immediately!

    // Simulate network partition
    adapter
        .update_service_health(&service_id, crate::types::HealthStatus::Unreachable)
        .await
        .expect("Failed to mark unreachable");
    // ✅ Event-driven: Immediate state change!

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
        .update_service_health(&service_id, crate::types::HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");
    // ✅ Event-driven: Immediate recovery!

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

    let service = storage_service("test-storage-restart");
    let service_id = service.id().to_string();

    // Register service and store some state
    let _handle = register_test_service(&adapter, service.clone())
        .await
        .expect("Failed to register");
    // ✅ Event-driven: Ready immediately!

    let store_request = crate::capabilities::SimpleCapabilityRequest {
        capability: "storage".to_string(),
        operation: "store".to_string(),
        parameters: vec![
            ("key".to_string(), "test_key".to_string()),
            ("value".to_string(), "test_value".to_string()),
        ]
        .into_iter()
        .collect(),
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
    // ✅ Event-driven: Immediate deregistration!

    let _handle2 = register_test_service(&adapter, service)
        .await
        .expect("Failed to re-register");
    // ✅ Event-driven: Ready immediately after restart!

    // Verify state is accessible after restart
    let retrieve_request = crate::capabilities::SimpleCapabilityRequest {
        capability: "storage".to_string(),
        operation: "retrieve".to_string(),
        parameters: vec![("key".to_string(), "test_key".to_string())]
            .into_iter()
            .collect(),
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

