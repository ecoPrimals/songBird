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
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Chaos and fault injection tests for service registry
//!
//! v3.20.0: Resilience testing for the service registry

use songbird_orchestrator::ipc::ServiceRegistry;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_chaos_concurrent_register_unregister() {
    // Chaos: Primals registering and unregistering concurrently

    let registry = Arc::new(ServiceRegistry::new());

    // Spawn 20 tasks that register and immediately unregister
    let mut handles = vec![];
    for i in 0..20 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            // Register
            let service_id = registry_clone
                .register_service(
                    format!("ChaosPrimal{i}"),
                    vec![format!("chaos{}", i)],
                    format!("/tmp/chaos{i}.sock"),
                    "json-rpc".to_string(),
                    30,
                )
                .await
                .unwrap();

            // Small random delay
            sleep(Duration::from_millis((i * 5) as u64)).await;

            // Unregister
            registry_clone.unregister_service(&service_id).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all chaos to complete
    futures::future::join_all(handles).await;

    // Registry should be empty after all unregistrations
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 0);
}

#[tokio::test]
async fn test_chaos_rapid_capability_queries() {
    // Chaos: Rapid concurrent queries while registry is being updated

    let registry = Arc::new(ServiceRegistry::new());

    // Register initial services
    for i in 0..5 {
        registry
            .register_service(
                format!("Primal{i}"),
                vec!["shared_capability".to_string()],
                format!("/tmp/primal{i}.sock"),
                "json-rpc".to_string(),
                30,
            )
            .await
            .unwrap();
    }

    // Spawn 100 concurrent queries
    let mut query_handles = vec![];
    for _ in 0..100 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            registry_clone.discover_by_capability("shared_capability", None).await.unwrap()
        });
        query_handles.push(handle);
    }

    // All queries should succeed (even under heavy load)
    let results: Vec<_> = futures::future::join_all(query_handles).await;

    for result in results {
        let primals = result.unwrap();
        assert_eq!(primals.len(), 5); // All queries should see all 5 primals
    }
}

#[tokio::test]
async fn test_chaos_health_status_race_conditions() {
    // Chaos: Concurrent health updates and queries

    let registry = Arc::new(ServiceRegistry::new());

    let service_id = registry
        .register_service(
            "RacyPrimal".to_string(),
            vec!["test".to_string()],
            "/tmp/racy.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Spawn 50 tasks that update health concurrently
    let mut update_handles = vec![];
    for i in 0..50 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        let service_id_clone = service_id.clone();
        let handle = tokio::spawn(async move {
            let status = if i % 3 == 0 {
                "healthy"
            } else if i % 3 == 1 {
                "degraded"
            } else {
                "down"
            };
            registry_clone.update_health(&service_id_clone, status.to_string()).await.unwrap();
        });
        update_handles.push(handle);
    }

    // Spawn 50 tasks that query health concurrently
    let mut query_handles = vec![];
    for _ in 0..50 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        let service_id_clone = service_id.clone();
        let handle = tokio::spawn(async move {
            registry_clone.get_service_health(&service_id_clone).await.unwrap()
        });
        query_handles.push(handle);
    }

    // All operations should complete without panics or deadlocks
    futures::future::join_all(update_handles).await;
    let query_results: Vec<_> = futures::future::join_all(query_handles).await;

    // All queries should succeed (even with concurrent updates)
    for result in query_results {
        let (status, _) = result.unwrap();
        // Status should be one of the valid values
        assert!(
            status == "unknown" || status == "healthy" || status == "degraded" || status == "down"
        );
    }
}

#[tokio::test]
async fn test_fault_injection_nonexistent_service_health() {
    // Fault: Query health of non-existent service

    let registry = Arc::new(ServiceRegistry::new());

    let (status, message) = registry.get_service_health("nonexistent-service-12345").await.unwrap();

    assert_eq!(status, "unknown");
    assert!(message.is_some());
    assert!(message.unwrap().contains("not found"));
}

#[tokio::test]
async fn test_fault_injection_unregister_nonexistent_service() {
    // Fault: Unregister a service that doesn't exist

    let registry = Arc::new(ServiceRegistry::new());

    // Should not panic or error - graceful handling
    let result = registry.unregister_service("nonexistent-service").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fault_injection_empty_capabilities() {
    // Fault: Register service with empty capabilities

    let registry = Arc::new(ServiceRegistry::new());

    let service_id = registry
        .register_service(
            "NoCapsPrimal".to_string(),
            vec![], // Empty capabilities
            "/tmp/nocaps.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Should still be registered (service_id should start with lowercase primal name)
    assert!(service_id.starts_with("nocapsprimal-"));

    // But won't be discoverable by any specific capability
    let primals = registry.discover_by_capability("any_capability", None).await.unwrap();
    assert_eq!(primals.len(), 0);

    // Should be discoverable by wildcard
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 1);
}

#[tokio::test]
async fn test_fault_injection_duplicate_endpoint_different_primal() {
    // Fault: Two different primals try to register with same endpoint

    let registry = Arc::new(ServiceRegistry::new());

    // Primal A registers first
    let service_id_a = registry
        .register_service(
            "PrimalA".to_string(),
            vec!["capability_a".to_string()],
            "/tmp/shared.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Primal B tries to register with same endpoint
    let service_id_b = registry
        .register_service(
            "PrimalB".to_string(),
            vec!["capability_b".to_string()],
            "/tmp/shared.sock".to_string(), // Same endpoint!
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Should be treated as update (same service_id)
    assert_eq!(service_id_a, service_id_b);

    // Only one service should exist
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 1);

    // Should have updated capabilities
    assert!(all_primals[0].capabilities.contains(&"capability_b".to_string()));
}

#[tokio::test]
async fn test_chaos_massive_concurrent_operations() {
    // Chaos: Mix of all operations happening concurrently

    let registry = Arc::new(ServiceRegistry::new());

    let mut handles = vec![];

    // 20 registrations
    for i in 0..20 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            registry_clone
                .register_service(
                    format!("Primal{i}"),
                    vec![format!("cap{}", i % 5)], // 5 different capabilities
                    format!("/tmp/primal{i}.sock"),
                    "json-rpc".to_string(),
                    30,
                )
                .await
                .unwrap();
        }));
    }

    // 50 capability queries
    for i in 0..50 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            registry_clone.discover_by_capability(&format!("cap{}", i % 5), None).await.unwrap();
        }));
    }

    // 30 wildcard queries
    for _ in 0..30 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            registry_clone.discover_by_capability("*", None).await.unwrap();
        }));
    }

    // 20 health queries
    for i in 0..20 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            // Query might fail if service doesn't exist yet, that's OK
            let _ = registry_clone.get_service_health(&format!("primal-{i}")).await;
        }));
    }

    // All operations should complete without panics or deadlocks
    futures::future::join_all(handles).await;

    // Registry should be in a consistent state
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 20);
}

#[tokio::test]
async fn test_fault_injection_extreme_capability_names() {
    // Fault: Edge cases in capability names

    let registry = Arc::new(ServiceRegistry::new());

    // Very long capability name
    let long_cap = "a".repeat(1000);
    registry
        .register_service(
            "LongCapPrimal".to_string(),
            vec![long_cap.clone()],
            "/tmp/longcap.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Should be discoverable
    let primals = registry.discover_by_capability(&long_cap, None).await.unwrap();
    assert_eq!(primals.len(), 1);

    // Special characters in capability
    registry
        .register_service(
            "SpecialCapPrimal".to_string(),
            vec!["capability-with-dashes".to_string(), "capability.with.dots".to_string()],
            "/tmp/specialcap.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let primals = registry.discover_by_capability("capability-with-dashes", None).await.unwrap();
    assert_eq!(primals.len(), 1);
}
