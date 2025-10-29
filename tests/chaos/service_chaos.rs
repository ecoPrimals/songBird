//! Chaos Engineering Tests - Service Failures
//!
//! These tests validate system resilience under random service failures
//! and adversarial conditions.

use songbird_test_utils::{fixtures::*, mocks::*, chaos_engineering::*};
use songbird_types::{CapabilityRequest, HealthStatus};
use songbird_universal::UniversalCapabilityAdapter;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

/// Test random single service failures during load
#[tokio::test]
async fn test_random_service_failures_under_load() {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register multiple services
    let mut services = vec![];
    for i in 0..5 {
        let mut service = compute_service_fixture();
        service.id = format!("compute_{}", i);
        adapter.register_service(service.clone()).await.expect("Register failed");
        services.push(service);
    }

    sleep(Duration::from_millis(200)).await;

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

        sleep(Duration::from_millis(10)).await;
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

    // Register 10 services
    let mut services = vec![];
    for i in 0..10 {
        let mut service = compute_service_fixture();
        service.id = format!("compute_{}", i);
        adapter.register_service(service.clone()).await.expect("Register failed");
        services.push(service);
    }

    sleep(Duration::from_millis(200)).await;

    // Fail 5 services simultaneously
    for i in 0..5 {
        adapter
            .update_service_health(&services[i].id, HealthStatus::Unhealthy)
            .await
            .expect("Failed to update health");
    }

    sleep(Duration::from_millis(100)).await;

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

    // Register services
    let service = compute_service_fixture();
    adapter
        .register_service(service.clone())
        .await
        .expect("Register failed");

    sleep(Duration::from_millis(100)).await;

    // Simulate coordinator stress
    for _ in 0..100 {
        tokio::spawn({
            let adapter = adapter.clone();
            async move {
                let _ = adapter.discover_capability_providers("compute").await;
            }
        });
    }

    sleep(Duration::from_millis(500)).await;

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

    let service = compute_service_fixture();
    let service_id = service.id.clone();
    
    adapter
        .register_service(service)
        .await
        .expect("Register failed");

    sleep(Duration::from_millis(100)).await;

    // Rapidly cycle between healthy and unhealthy
    for _ in 0..20 {
        adapter
            .update_service_health(&service_id, HealthStatus::Unhealthy)
            .await
            .expect("Failed to mark unhealthy");

        sleep(Duration::from_millis(10)).await;

        adapter
            .update_service_health(&service_id, HealthStatus::Healthy)
            .await
            .expect("Failed to mark healthy");

        sleep(Duration::from_millis(10)).await;
    }

    // System should stabilize
    sleep(Duration::from_millis(100)).await;

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

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Register failed");

    sleep(Duration::from_millis(100)).await;

    // Simulate transient failure
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    sleep(Duration::from_millis(50)).await;

    // Auto-recovery
    adapter
        .update_service_health(&service_id, HealthStatus::Healthy)
        .await
        .expect("Failed to mark healthy");

    sleep(Duration::from_millis(50)).await;

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

    // Register primary and backup
    let primary = compute_service_fixture();
    let primary_id = primary.id.clone();
    
    let mut backup = compute_service_fixture();
    backup.id = format!("{}_backup", backup.id);

    adapter.register_service(primary).await.expect("Register primary");
    adapter.register_service(backup).await.expect("Register backup");

    sleep(Duration::from_millis(100)).await;

    // Permanently fail primary
    adapter
        .update_service_health(&primary_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    // Deregister to simulate permanent failure
    adapter
        .deregister_service(&primary_id)
        .await
        .expect("Failed to deregister");

    sleep(Duration::from_millis(100)).await;

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

    adapter
        .register_service(service)
        .await
        .expect("Register failed");

    sleep(Duration::from_millis(100)).await;

    // Simulate gradual degradation
    adapter
        .update_service_health(&service_id, HealthStatus::Degraded)
        .await
        .expect("Failed to mark degraded");

    sleep(Duration::from_millis(200)).await;

    // Then complete failure
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    sleep(Duration::from_millis(100)).await;

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

    // Register only one critical service
    let service = storage_service_fixture();
    let service_id = service.id.clone();

    adapter
        .register_service(service)
        .await
        .expect("Register failed");

    sleep(Duration::from_millis(100)).await;

    // Fail the critical service
    adapter
        .update_service_health(&service_id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    sleep(Duration::from_millis(50)).await;

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

    // Register core and optional services
    let core = storage_service_fixture();
    let optional = ai_service_fixture();

    adapter.register_service(core).await.expect("Register core");
    adapter.register_service(optional.clone()).await.expect("Register optional");

    sleep(Duration::from_millis(100)).await;

    // Fail optional service
    adapter
        .update_service_health(&optional.id, HealthStatus::Unhealthy)
        .await
        .expect("Failed to mark unhealthy");

    sleep(Duration::from_millis(50)).await;

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

