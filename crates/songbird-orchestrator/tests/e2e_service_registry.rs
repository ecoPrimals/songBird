// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! E2E tests for service registry
//!
//! v3.20.0: Full workflow tests for primal registration and discovery

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use songbird_orchestrator::ipc::ServiceRegistry;
use std::sync::Arc;

#[tokio::test]
async fn test_e2e_full_registration_discovery_workflow() {
    // Scenario: Multiple primals register, then discover each other

    let registry = Arc::new(ServiceRegistry::new());

    // Step 1: BearDog registers with encryption capability
    let beardog_id = registry
        .register_service(
            "BearDog".to_string(),
            vec!["encryption".to_string(), "identity".to_string()],
            "/run/user/1000/beardog-nat0.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    assert!(beardog_id.starts_with("beardog-"));

    // Step 2: ToadStool registers with compute capability
    let toadstool_id = registry
        .register_service(
            "ToadStool".to_string(),
            vec!["compute".to_string(), "execution".to_string()],
            "/run/user/1000/toadstool-nat0.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    assert!(toadstool_id.starts_with("toadstool-"));

    // Step 3: NestGate registers with storage capability
    let nestgate_id = registry
        .register_service(
            "NestGate".to_string(),
            vec!["storage".to_string(), "persistence".to_string()],
            "/run/user/1000/nestgate-nat0.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    assert!(nestgate_id.starts_with("nestgate-"));

    // Step 4: biomeOS discovers encryption provider (should find BearDog)
    let encryption_providers =
        registry.discover_by_capability("encryption", Some("json-rpc")).await.unwrap();

    assert_eq!(encryption_providers.len(), 1);
    assert_eq!(encryption_providers[0].primal_name, "BearDog");
    assert_eq!(encryption_providers[0].service_id, beardog_id);
    assert!(encryption_providers[0].capabilities.contains(&"encryption".to_string()));

    // Step 5: biomeOS discovers storage provider (should find NestGate)
    let storage_providers = registry.discover_by_capability("storage", None).await.unwrap();

    assert_eq!(storage_providers.len(), 1);
    assert_eq!(storage_providers[0].primal_name, "NestGate");
    assert_eq!(storage_providers[0].service_id, nestgate_id);

    // Step 6: petalTongue discovers all primals (wildcard)
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();

    assert_eq!(all_primals.len(), 3);

    // Verify all primals are present
    let names: Vec<String> = all_primals.iter().map(|p| p.primal_name.clone()).collect();
    assert!(names.contains(&"BearDog".to_string()));
    assert!(names.contains(&"ToadStool".to_string()));
    assert!(names.contains(&"NestGate".to_string()));

    // Step 7: Check health of a specific service
    let (health_status, _) = registry.get_service_health(&beardog_id).await.unwrap();
    assert_eq!(health_status, "unknown"); // Initial state

    // Step 8: Simulate health check update
    registry.update_health(&beardog_id, "healthy".to_string()).await.unwrap();
    let (health_status, _) = registry.get_service_health(&beardog_id).await.unwrap();
    assert_eq!(health_status, "healthy");

    // Step 9: Service unregisters (cleanup)
    registry.unregister_service(&beardog_id).await.unwrap();
    let encryption_providers = registry.discover_by_capability("encryption", None).await.unwrap();
    assert_eq!(encryption_providers.len(), 0); // BearDog no longer discoverable
}

#[tokio::test]
async fn test_e2e_concurrent_registrations() {
    // Scenario: Multiple primals register concurrently (stress test for Arc<RwLock>)

    let registry = Arc::new(ServiceRegistry::new());

    // Spawn 10 concurrent registration tasks
    let mut handles = vec![];
    for i in 0..10 {
        let registry_clone: Arc<ServiceRegistry> = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            registry_clone
                .register_service(
                    format!("Primal{i}"),
                    vec![format!("capability{}", i)],
                    format!("/tmp/primal{i}.sock"),
                    "json-rpc".to_string(),
                    30,
                )
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // Wait for all registrations to complete
    let service_ids: Vec<String> =
        futures::future::join_all(handles).await.into_iter().map(|r| r.unwrap()).collect();

    // Verify all 10 services registered
    assert_eq!(service_ids.len(), 10);

    // Verify wildcard discovery returns all 10
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 10);
}

#[tokio::test]
async fn test_e2e_re_registration_updates_capabilities() {
    // Scenario: A primal re-registers with updated capabilities

    let registry = Arc::new(ServiceRegistry::new());

    // Initial registration
    let service_id = registry
        .register_service(
            "DynamicPrimal".to_string(),
            vec!["capability_v1".to_string()],
            "/tmp/dynamic.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Discover by v1 capability
    let v1_primals = registry.discover_by_capability("capability_v1", None).await.unwrap();
    assert_eq!(v1_primals.len(), 1);

    // Re-register with updated capabilities
    let service_id_updated = registry
        .register_service(
            "DynamicPrimal".to_string(),
            vec!["capability_v1".to_string(), "capability_v2".to_string()], // Added v2
            "/tmp/dynamic.sock".to_string(),                                // Same endpoint
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Service ID should be the same (update, not new registration)
    assert_eq!(service_id, service_id_updated);

    // Discover by v2 capability (should now be found)
    let v2_primals = registry.discover_by_capability("capability_v2", None).await.unwrap();
    assert_eq!(v2_primals.len(), 1);
    assert_eq!(v2_primals[0].service_id, service_id);
    assert!(v2_primals[0].capabilities.contains(&"capability_v2".to_string()));
}

#[tokio::test]
async fn test_e2e_protocol_filtering() {
    // Scenario: Multiple primals with different protocols

    let registry = Arc::new(ServiceRegistry::new());

    // Register JSON-RPC primal
    registry
        .register_service(
            "JsonRpcPrimal".to_string(),
            vec!["encryption".to_string()],
            "/tmp/jsonrpc.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Register tarpc primal
    registry
        .register_service(
            "TarpcPrimal".to_string(),
            vec!["encryption".to_string()],
            "tcp://localhost:8091".to_string(),
            "tarpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Register HTTP primal
    registry
        .register_service(
            "HttpPrimal".to_string(),
            vec!["encryption".to_string()],
            "http://localhost:8080".to_string(),
            "http".to_string(),
            30,
        )
        .await
        .unwrap();

    // Discover all encryption providers (no filter)
    let all_encryption = registry.discover_by_capability("encryption", None).await.unwrap();
    assert_eq!(all_encryption.len(), 3);

    // Discover only JSON-RPC encryption providers
    let jsonrpc_only =
        registry.discover_by_capability("encryption", Some("json-rpc")).await.unwrap();
    assert_eq!(jsonrpc_only.len(), 1);
    assert_eq!(jsonrpc_only[0].primal_name, "JsonRpcPrimal");

    // Discover only tarpc encryption providers
    let tarpc_only = registry.discover_by_capability("encryption", Some("tarpc")).await.unwrap();
    assert_eq!(tarpc_only.len(), 1);
    assert_eq!(tarpc_only[0].primal_name, "TarpcPrimal");
}

#[tokio::test]
async fn test_e2e_health_status_lifecycle() {
    // Scenario: Health status changes over time

    let registry = Arc::new(ServiceRegistry::new());

    let service_id = registry
        .register_service(
            "HealthyPrimal".to_string(),
            vec!["test".to_string()],
            "/tmp/healthy.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Initial health: unknown
    let (status, _) = registry.get_service_health(&service_id).await.unwrap();
    assert_eq!(status, "unknown");

    // Simulate health check: healthy
    registry.update_health(&service_id, "healthy".to_string()).await.unwrap();
    let (status, _) = registry.get_service_health(&service_id).await.unwrap();
    assert_eq!(status, "healthy");

    // Simulate health check: degraded
    registry.update_health(&service_id, "degraded".to_string()).await.unwrap();
    let (status, _) = registry.get_service_health(&service_id).await.unwrap();
    assert_eq!(status, "degraded");

    // Simulate health check: down
    registry.update_health(&service_id, "down".to_string()).await.unwrap();
    let (status, _) = registry.get_service_health(&service_id).await.unwrap();
    assert_eq!(status, "down");

    // Discovery should still return the service (even if down)
    // This allows monitoring tools to see "down" services
    let primals = registry.discover_by_capability("test", None).await.unwrap();
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].health_status, "down");
}

#[tokio::test]
async fn test_e2e_wildcard_discovery_returns_all() {
    // Scenario: petalTongue needs to discover ALL primals for visualization

    let registry = Arc::new(ServiceRegistry::new());

    // Register diverse primals
    registry
        .register_service(
            "BearDog".to_string(),
            vec!["encryption".to_string()],
            "/tmp/beardog.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry
        .register_service(
            "ToadStool".to_string(),
            vec!["compute".to_string()],
            "/tmp/toadstool.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry
        .register_service(
            "NestGate".to_string(),
            vec!["storage".to_string()],
            "/tmp/nestgate.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry
        .register_service(
            "Squirrel".to_string(),
            vec!["ai_coordination".to_string()],
            "/tmp/squirrel.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry
        .register_service(
            "biomeOS".to_string(),
            vec!["orchestration".to_string()],
            "/tmp/biomeos.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();
    registry
        .register_service(
            "petalTongue".to_string(),
            vec!["visualization".to_string()],
            "/tmp/petaltongue.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Wildcard discovery
    let all_primals = registry.discover_by_capability("*", None).await.unwrap();
    assert_eq!(all_primals.len(), 6);

    // Verify all expected primals are present
    let names: Vec<String> = all_primals.iter().map(|p| p.primal_name.clone()).collect();
    assert!(names.contains(&"BearDog".to_string()));
    assert!(names.contains(&"ToadStool".to_string()));
    assert!(names.contains(&"NestGate".to_string()));
    assert!(names.contains(&"Squirrel".to_string()));
    assert!(names.contains(&"biomeOS".to_string()));
    assert!(names.contains(&"petalTongue".to_string()));
}
