// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Integration tests for capability registration and routing
//!
//! Tests the complete flow:
//! External Provider → Register → Heartbeat → Task Routing → Execution

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;
use songbird_orchestrator::core::registry::types::{
    CapabilityDescriptor, CapabilityRegistrationRequest, HealthStatus,
};
use songbird_orchestrator::core::registry::{CapabilityRegistry, HeartbeatConfig};
use songbird_orchestrator::core::routing::{CapabilityRouter, RoutingDecision, Task};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Create a test capability registry with short timeouts for testing
fn create_test_registry() -> Arc<CapabilityRegistry> {
    let config = HeartbeatConfig {
        interval: Duration::from_millis(100),
        unhealthy_threshold: Duration::from_secs(1),
        removal_threshold: Duration::from_secs(2),
    };
    Arc::new(CapabilityRegistry::with_config(config))
}

/// Create a test registration request
fn create_test_registration_request(provider_id: &str) -> CapabilityRegistrationRequest {
    let mut metadata = HashMap::new();
    metadata.insert("max_concurrent_tasks".to_string(), serde_json::json!(10));

    CapabilityRegistrationRequest {
        provider_id: provider_id.to_string(),
        provider_name: format!("{provider_id} Provider"),
        provider_type: "compute".to_string(),
        version: "1.0.0".to_string(),
        endpoint: format!("http://{provider_id}:9000"),
        capabilities: vec![
            CapabilityDescriptor {
                name: "compute_gpu".to_string(),
                description: "GPU computation".to_string(),
                metadata: HashMap::new(),
            },
            CapabilityDescriptor {
                name: "compute_heavy".to_string(),
                description: "Heavy computation".to_string(),
                metadata: HashMap::new(),
            },
            CapabilityDescriptor {
                name: "ml_training".to_string(),
                description: "ML model training".to_string(),
                metadata: HashMap::new(),
            },
        ],
        workload_endpoint: "/api/v1/workload/execute".to_string(),
        health_endpoint: "/api/v1/health".to_string(),
        metadata,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_provider_registration() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    // Register provider
    let result = registry.register(request.clone()).await;
    assert!(result.is_ok(), "Registration should succeed");
    let registration_id = result.expect("Registration should return ID");
    assert!(!registration_id.is_empty(), "Should return registration ID");

    // Verify provider is registered
    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 1, "Should have one registered provider");
    assert_eq!(providers[0].registration.provider_id, request.provider_id);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_duplicate_registration_fails() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    // First registration should succeed
    let result1 = registry.register(request.clone()).await;
    assert!(result1.is_ok(), "First registration should succeed");

    // Second registration with same ID should fail
    let result2 = registry.register(request).await;
    assert!(result2.is_err(), "Duplicate registration should fail");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_providers() {
    let registry = create_test_registry();

    // Register multiple providers
    for i in 1..=3 {
        let request = create_test_registration_request(&format!("provider-{i}"));
        let result = registry.register(request).await;
        assert!(result.is_ok(), "Registration {i} should succeed");
    }

    // Verify all providers are registered
    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 3, "Should have three registered providers");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_find_providers_by_capability() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    registry.register(request).await.expect("Test registration should succeed");

    // Find providers with specific capability
    let gpu_providers = registry
        .find_providers_with_capability("compute_gpu")
        .await
        .expect("GPU capability query should succeed");
    assert_eq!(gpu_providers.len(), 1, "Should find one GPU provider");

    let heavy_providers = registry
        .find_providers_with_capability("compute_heavy")
        .await
        .expect("Heavy compute capability query should succeed");
    assert_eq!(heavy_providers.len(), 1, "Should find one heavy compute provider");

    // Non-existent capability should return empty
    let no_providers = registry
        .find_providers_with_capability("nonexistent")
        .await
        .expect("Nonexistent capability query should succeed (returns empty)");
    assert_eq!(no_providers.len(), 0, "Should find no providers for nonexistent capability");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_heartbeat_updates() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    let registration_id = registry.register(request.clone()).await.expect("test precondition");

    // Send heartbeat
    let result = registry.update_heartbeat(&request.provider_id, &registration_id, None).await;
    assert!(result.is_ok(), "Heartbeat should succeed");

    // Wrong registration ID should fail
    let result = registry.update_heartbeat(&request.provider_id, "wrong-id", None).await;
    assert!(result.is_err(), "Heartbeat with wrong ID should fail");

    // Unknown provider should fail
    let result = registry.update_heartbeat("unknown-provider", &registration_id, None).await;
    assert!(result.is_err(), "Heartbeat for unknown provider should fail");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_provider_unregistration() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    registry.register(request.clone()).await.expect("test precondition");

    // Verify provider is registered
    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 1);

    // Unregister
    let result = registry.unregister(&request.provider_id).await;
    assert!(result.is_ok(), "Unregistration should succeed");

    // Verify provider is removed
    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 0, "Provider should be removed");

    // Unregister again should fail
    let result = registry.unregister(&request.provider_id).await;
    assert!(result.is_err(), "Unregister should fail for non-existent provider");
}

#[tokio::test(start_paused = true)]
async fn test_heartbeat_timeout() {
    let registry = create_test_registry();
    let request = create_test_registration_request("test-provider-1");

    let registration_id = registry.register(request.clone()).await.expect("test precondition");

    registry
        .update_heartbeat(&request.provider_id, &registration_id, None)
        .await
        .expect("test precondition");

    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 1);

    let registry_clone = registry.clone();
    registry_clone.start_health_monitor();

    // Advance virtual time past unhealthy threshold (1s) but before removal (2s)
    tokio::time::advance(Duration::from_millis(1200)).await;
    tokio::task::yield_now().await;

    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 1, "Provider should still be registered");
    assert!(
        providers.iter().any(|p| p.health.status == HealthStatus::Unhealthy),
        "Provider should be unhealthy"
    );

    // Advance past removal threshold
    tokio::time::advance(Duration::from_millis(1500)).await;
    tokio::task::yield_now().await;

    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 0, "Provider should be removed after timeout");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_routing_with_external_provider() {
    let registry = create_test_registry();
    let federation_state = Arc::new(FederationState::new("test".to_string()));
    let service_registry = Arc::new(FederatedServiceRegistry::new());

    // Create router with capability registry
    let router = CapabilityRouter::with_capability_registry(
        federation_state,
        service_registry,
        registry.clone(),
    );

    // Register a GPU provider
    let request = create_test_registration_request("gpu-provider-1");
    registry.register(request).await.expect("test precondition");

    // Create a GPU task
    let task = Task::builder("ml_training").with_gpu().build();

    // Route the task
    let decision = router.route_task(&task).await;
    assert!(decision.is_ok(), "Routing should succeed");

    // Should route to external provider
    match decision.expect("test precondition") {
        RoutingDecision::RouteToExternalProvider {
            provider_id,
            ..
        } => {
            assert_eq!(provider_id, "gpu-provider-1");
        }
        other => panic!("Expected RouteToExternalProvider, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_routing_falls_back_without_provider() {
    let registry = create_test_registry();
    let federation_state = Arc::new(FederationState::new("test".to_string()));
    let service_registry = Arc::new(FederatedServiceRegistry::new());

    // Create router with empty registry (no providers registered)
    let router =
        CapabilityRouter::with_capability_registry(federation_state, service_registry, registry);

    // Create a GPU task
    let task = Task::builder("ml_training").with_gpu().build();

    // Route the task - should fall back to capability resolver
    let decision = router.route_task(&task).await;

    // Should either find a capability provider or fail gracefully
    match decision {
        Ok(RoutingDecision::RouteToCapability {
            ..
        }) => {
            // Fallback to static capability provider (if configured)
        }
        Err(_) => {
            // No provider available (expected when registry and resolver are empty)
        }
        Ok(other) => panic!("Unexpected routing decision: {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn test_provider_selection_prefers_healthy() {
    let registry = create_test_registry();

    let request1 = create_test_registration_request("provider-1");
    let request2 = create_test_registration_request("provider-2");

    let reg_id_1 = registry.register(request1.clone()).await.expect("test precondition");
    let _reg_id_2 = registry.register(request2.clone()).await.expect("test precondition");

    let registry_clone = registry.clone();
    registry_clone.start_health_monitor();

    // Both should be available initially
    let providers = registry
        .find_providers_with_capability("compute_gpu")
        .await
        .expect("should find expected value");
    assert_eq!(providers.len(), 2, "Should find both providers");

    // Keep provider-1 alive, let provider-2 expire
    registry
        .update_heartbeat(&request1.provider_id, &reg_id_1, None)
        .await
        .expect("test precondition");

    // Advance past unhealthy threshold — provider-2 has no heartbeat
    tokio::time::advance(Duration::from_millis(1200)).await;
    tokio::task::yield_now().await;

    // Refresh provider-1's heartbeat after the time advance
    registry
        .update_heartbeat(&request1.provider_id, &reg_id_1, None)
        .await
        .expect("test precondition");

    let providers = registry
        .find_providers_with_capability("compute_gpu")
        .await
        .expect("should find expected value");

    assert_eq!(providers.len(), 1, "Only healthy provider should be returned");
    assert_eq!(
        providers[0].registration.provider_id, "provider-1",
        "Should include the healthy provider-1"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_registrations() {
    let registry = create_test_registry();

    // Register multiple providers concurrently
    let mut handles = vec![];
    for i in 1..=10 {
        let registry_clone = registry.clone();
        let handle = tokio::spawn(async move {
            let request = create_test_registration_request(&format!("provider-{i}"));
            registry_clone.register(request).await
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        let result = handle.await.expect("test precondition");
        assert!(result.is_ok(), "Concurrent registration should succeed");
    }

    // Verify all providers are registered
    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 10, "Should have 10 registered providers");
}

#[tokio::test(start_paused = true)]
async fn test_registry_health_monitor_lifecycle() {
    let registry = create_test_registry();

    let request = create_test_registration_request("test-provider");
    let reg_id = registry.register(request.clone()).await.expect("test precondition");

    let registry_clone = registry.clone();
    registry_clone.start_health_monitor();

    // Send periodic heartbeats, advancing time in small steps
    for _ in 0..5 {
        tokio::time::advance(Duration::from_millis(200)).await;
        tokio::task::yield_now().await;
        registry
            .update_heartbeat(&request.provider_id, &reg_id, None)
            .await
            .expect("test precondition");
    }

    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 1, "Provider should remain registered with heartbeats");

    // Stop sending heartbeats, advance past removal threshold
    tokio::time::advance(Duration::from_millis(2500)).await;
    tokio::task::yield_now().await;

    let providers = registry.list_providers().await;
    assert_eq!(providers.len(), 0, "Provider should be removed after timeout");
}
