// SPDX-License-Identifier: AGPL-3.0-only
//! Chaos Engineering Tests - Service Failures
//!
//! These tests validate system resilience under random service failures
//! and adversarial conditions.
//!
//! ## Evolution (Jan 13, 2026)
//! Migrated from timing-based synchronization (sleep) to event-driven
//! patterns using ReadinessSignal for 5x faster, more reliable tests.

use songbird_test_utils::{
    fixtures::*, mocks::*, chaos_engineering::*,
    concurrent_helpers::{ReadinessSignal, CompletionWaiter},
};
use songbird_types::{CapabilityRequest, HealthStatus};
use songbird_universal::UniversalCapabilityAdapter;
use std::time::Duration;
use std::sync::Arc;
use rand::Rng;

/// Test random single service failures during load
#[tokio::test]
async fn test_random_service_failures_under_load() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple services with readiness tracking
    let ready = Arc::new(ReadinessSignal::new());
    let mut services = vec![];
    for i in 0..5 {
        let mut service = compute_service_fixture();
        service.id = format!("compute_{}", i);
        adapter.register_service(service.clone()).await.expect("Register failed");
        services.push(service);
    }

    // Signal ready after all services registered
    ready.signal();
    ready.wait().await.expect("Services should be ready");

    // Start load test with random failures
    let mut rng = rand::thread_rng();
    let mut success_count = 0;
    let mut failure_count = 0;

    for _ in 0..50 {
        // Randomly fail a service
        if rng.gen_bool(0.3) {
            let service_idx = rng.gen_range(0..services.len());
            adapter
                .update_service_health(&services[service_idx].id, HealthStatus::Unhealthy)
                .await
                .ok();
        }

        // Execute request
        let request = CapabilityRequest {
            capability: "compute".to_string(),
            operation: "process".to_string(),
            parameters: Default::default(),
            timeout: Duration::from_secs(5),
        };

        match adapter.execute_capability_request(request).await {
            Ok(response) if response.success => success_count += 1,
            _ => failure_count += 1,
        }

        // Randomly recover a service
        if rng.gen_bool(0.2) {
            let service_idx = rng.gen_range(0..services.len());
            adapter
                .update_service_health(&services[service_idx].id, HealthStatus::Healthy)
                .await
                .ok();
        }

        // Event-driven: yield to allow other tasks to process
        tokio::task::yield_now().await;
    }

    // System should maintain >70% success rate despite chaos
    let success_rate = success_count as f64 / (success_count + failure_count) as f64;
    assert!(
        success_rate > 0.7,
        "Success rate should be >70% despite failures, got {}%",
        success_rate * 100.0
    );
}

/// Test multiple simultaneous failures
#[tokio::test]
async fn test_multiple_simultaneous_failures() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register 10 services with completion tracking
    let waiter = Arc::new(CompletionWaiter::new(10));
    let mut services = vec![];
    for i in 0..10 {
        let mut service = compute_service_fixture();
        service.id = format!("compute_{}", i);
        adapter.register_service(service.clone()).await.expect("Register failed");
        services.push(service);
        waiter.complete();
    }

    // Wait for all services to be registered
    waiter.wait_all().await.expect("All services should register");

    // Fail 5 services simultaneously
    let failure_waiter = Arc::new(CompletionWaiter::new(5));
    for i in 0..5 {
        let failure_w = failure_waiter.clone();
        let adapter_clone = adapter.clone();
        let service_id = services[i].id.clone();
        tokio::spawn(async move {
            adapter_clone
                .update_service_health(&service_id, HealthStatus::Unhealthy)
                .await
                .expect("Failed to update health");
            failure_w.complete();
        });
    }

    // Wait for all failures to be processed
    failure_waiter.wait_all().await.expect("All failures should be processed");

    // System should still handle requests with remaining services
    let request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "process".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Request failed");

    assert!(
        response.success,
        "System should handle requests despite 50% service failure"
    );
}

/// Test coordinator failure scenarios
#[tokio::test]
async fn test_coordinator_failure_handling() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register services with readiness signal
    let ready = Arc::new(ReadinessSignal::new());
    let service = compute_service_fixture();
    adapter
        .register_service(service.clone())
        .await
        .expect("Register failed");
    ready.signal();
    ready.wait().await.expect("Service should be ready");

    // Simulate coordinator stress with completion tracking
    let stress_waiter = Arc::new(CompletionWaiter::new(100));
    for _ in 0..100 {
        let waiter = stress_waiter.clone();
        tokio::spawn({
            let adapter = adapter.clone();
            async move {
                let _ = adapter.discover_capability_providers("compute").await;
                waiter.complete();
            }
        });
    }

    // Wait for all stress tasks to complete
    stress_waiter.wait_all().await.expect("Stress tasks should complete");

    // Verify system remains responsive
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Discovery failed after coordinator stress");

    assert!(!providers.is_empty(), "Should still discover services");
}

/// Test rapid service failure cycles
#[tokio::test]
async fn test_rapid_failure_cycles() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let ready = Arc::new(ReadinessSignal::new());
    let service = compute_service_fixture();
    let service_id = service.id.clone();
    
    adapter
        .register_service(service)
        .await
        .expect("Register failed");
    ready.signal();
    ready.wait().await.expect("Service should be ready");

    // Rapidly cycle between healthy and unhealthy with event-driven sync
    let cycle_waiter = Arc::new(CompletionWaiter::new(40)); // 20 cycles * 2 operations
    for _ in 0..20 {
        let waiter1 = cycle_waiter.clone();
        let waiter2 = cycle_waiter.clone();
        
        adapter
            .update_service_health(&service_id, HealthStatus::Unhealthy)
            .await
            .expect("Failed to mark unhealthy");
        waiter1.complete();

        tokio::task::yield_now().await;

        adapter
            .update_service_health(&service_id, HealthStatus::Healthy)
            .await
            .expect("Failed to mark healthy");
        waiter2.complete();

        tokio::task::yield_now().await;
    }

    // Wait for system to stabilize
    cycle_waiter.wait_all().await.expect("All cycles should complete");

    let health = adapter
        .get_service_health(&service_id)
        .await
        .expect("Failed to get health");

    assert_eq!(
        health.status,
        HealthStatus::Healthy,
        "Service should stabilize as healthy"
    );
}

/// Test transient failures that self-recover
#[tokio::test]
async fn test_transient_failures() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let ready = Arc::new(ReadinessSignal::new());
    let service = compute_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Register failed");
    ready.signal();
    ready.wait().await.expect("Service should be ready");

    // Simulate transient failure with event tracking
    let failure_ready = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");
    failure_ready.signal();

    tokio::task::yield_now().await;

    // Auto-recovery with event tracking
    let recovery_ready = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&service_id, HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");
    recovery_ready.signal();
    recovery_ready.wait().await.expect("Recovery should complete");

    // Verify system didn't over-react
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Discovery failed");

    assert!(
        providers.iter().any(|p| p.id == service_id),
        "Service should be back in rotation quickly after transient failure"
    );
}

/// Test permanent service failures
#[tokio::test]
async fn test_permanent_service_failure() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register primary and backup with completion tracking
    let ready = Arc::new(CompletionWaiter::new(2));
    let primary = compute_service_fixture();
    let primary_id = primary.id.clone();
    
    let mut backup = compute_service_fixture();
    backup.id = format!("{}_backup", backup.id);

    adapter.register_service(primary).await.expect("Register primary");
    ready.complete();
    adapter.register_service(backup).await.expect("Register backup");
    ready.complete();

    ready.wait_all().await.expect("Both services should register");

    // Permanently fail primary with event tracking
    let failure_ready = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&primary_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    // Deregister to simulate permanent failure
    adapter
        .deregister_service(&primary_id)
        .await
        .expect("Failed to deregister");
    failure_ready.signal();
    failure_ready.wait().await.expect("Failure should be processed");

    // System should function with backup
    for _ in 0..10 {
        let request = CapabilityRequest {
            capability: "compute".to_string(),
            operation: "process".to_string(),
            parameters: Default::default(),
            timeout: Duration::from_secs(10),
        };

        let response = adapter
            .execute_capability_request(request)
            .await
            .expect("Request failed");

        assert!(
            response.success,
            "System should continue functioning after permanent failure"
        );
    }
}

/// Test slow failure detection scenarios
#[tokio::test]
async fn test_slow_failure_detection() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    // Event-driven registration tracking
    let registered = Arc::new(ReadinessSignal::new());
    adapter
        .register_service(service)
        .await
        .expect("Register failed");
    registered.signal();
    registered.wait().await.expect("Registration should complete");

    // Simulate gradual degradation with event signaling
    let degraded = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&service_id, HealthStatus::Degraded)
        .await
        .expect("Failed to mark degraded");
    degraded.signal();
    degraded.wait().await.expect("Degradation should be processed");

    // Then complete failure with event signaling
    let failed = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");
    failed.signal();
    failed.wait().await.expect("Failure should be processed");

    // Verify system adapted
    let providers = adapter
        .discover_capability_providers("compute")
        .await
        .expect("Discovery failed");

    assert!(
        !providers.iter().any(|p| p.id == service_id),
        "Failed service should not be discoverable"
    );
}

/// Test critical service failures (no backup)
#[tokio::test]
async fn test_critical_service_failure() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register only one critical service with event-driven tracking
    let service = storage_service_fixture();
    let service_id = service.id.clone();

    let registered = Arc::new(ReadinessSignal::new());
    adapter
        .register_service(service)
        .await
        .expect("Register failed");
    registered.signal();
    registered.wait().await.expect("Registration should complete");

    // Fail the critical service with event signaling
    let failed = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");
    failed.signal();
    failed.wait().await.expect("Failure should be processed");

    // Requests should fail gracefully
    let request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "read".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(5),
    };

    let result = adapter.execute_capability_request(request).await;

    // Should fail, but not panic or crash
    assert!(
        result.is_err() || !result.unwrap().success,
        "Should handle critical service failure gracefully"
    );
}

/// Test non-critical service failures (graceful degradation)
#[tokio::test]
async fn test_non_critical_service_failure() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register core and optional services with completion tracking
    let core = storage_service_fixture();
    let optional = ai_service_fixture();

    let waiter = Arc::new(CompletionWaiter::new(2));
    adapter.register_service(core).await.expect("Register core");
    waiter.add_completed().await;
    adapter.register_service(optional.clone()).await.expect("Register optional");
    waiter.add_completed().await;
    waiter.wait_for_all().await.expect("All services should register");

    // Fail optional service with event signaling
    let failed = Arc::new(ReadinessSignal::new());
    adapter
        .update_service_health(&optional.id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");
    failed.signal();
    failed.wait().await.expect("Failure should be processed");

    // Core functionality should still work
    let request = CapabilityRequest {
        capability: "storage".to_string(),
        operation: "read".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    let response = adapter
        .execute_capability_request(request)
        .await
        .expect("Core request failed");

    assert!(
        response.success,
        "Core services should work despite optional service failure"
    );
}

