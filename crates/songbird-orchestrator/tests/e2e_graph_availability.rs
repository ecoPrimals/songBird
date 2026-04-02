// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! End-to-end tests for graph availability checking
//!
//! These tests verify the full workflow from graph creation to availability
//! checking and alternative suggestion, using real service registry integration.

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
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

use songbird_orchestrator::graph::{
    AvailabilityChecker, Graph, GraphMetadata, GraphNode, NodeAvailabilityStatus,
};
use songbird_orchestrator::ipc::registry::ServiceRegistry;
use std::sync::Arc;

fn create_test_graph(nodes: Vec<GraphNode>) -> Graph {
    Graph::new(
        "e2e-test".to_string(),
        "E2E Test Graph".to_string(),
        nodes,
        vec![],
        GraphMetadata::default(),
    )
}

fn create_test_node(id: &str, capability: &str, protocol: Option<&str>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        primal_name: None,
        capability: capability.to_string(),
        inputs: vec![],
        outputs: vec![],
        config: serde_json::json!({}),
        preferred_protocol: protocol.map(std::string::ToString::to_string),
        timeout_secs: None,
    }
}

#[tokio::test]
async fn test_e2e_availability_workflow() {
    // Step 1: Set up service registry with multiple primals
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    // Register 3 primals with different capabilities and health statuses
    let beardog_id = registry
        .register_service(
            "BearDog".to_string(),
            vec!["encryption".to_string(), "identity".to_string()],
            "/run/user/1000/beardog.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let nestgate_id = registry
        .register_service(
            "NestGate".to_string(),
            vec!["storage".to_string()],
            "/run/user/1000/nestgate.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let toadstool_id = registry
        .register_service(
            "ToadStool".to_string(),
            vec!["compute".to_string()],
            "/run/user/1000/toadstool.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Mark BearDog and NestGate as healthy, ToadStool as degraded
    registry.update_health(&beardog_id, "healthy".to_string()).await.unwrap();
    registry.update_health(&nestgate_id, "healthy".to_string()).await.unwrap();
    registry.update_health(&toadstool_id, "degraded".to_string()).await.unwrap();

    // Step 2: Create a graph needing encryption, storage, and compute
    let graph = create_test_graph(vec![
        create_test_node("encrypt", "encryption", Some("json-rpc")),
        create_test_node("store", "storage", Some("json-rpc")),
        create_test_node("compute", "compute", Some("json-rpc")),
    ]);

    // Step 3: Check availability
    let report = checker.check_availability(&graph).await.unwrap();

    // Verify results
    assert_eq!(report.summary.total_nodes, 3);
    assert_eq!(report.available.len(), 2); // encrypt + store
    assert_eq!(report.degraded.len(), 1); // compute
    assert_eq!(report.unavailable.len(), 0);

    // Check specific node statuses
    let encrypt_status = report.details.get("encrypt").unwrap();
    assert_eq!(encrypt_status.status, NodeAvailabilityStatus::Available);
    assert_eq!(encrypt_status.primal, Some("BearDog".to_string()));

    let store_status = report.details.get("store").unwrap();
    assert_eq!(store_status.status, NodeAvailabilityStatus::Available);
    assert_eq!(store_status.primal, Some("NestGate".to_string()));

    let compute_status = report.details.get("compute").unwrap();
    assert_eq!(compute_status.status, NodeAvailabilityStatus::Degraded);
    assert_eq!(compute_status.primal, Some("ToadStool".to_string()));
}

#[tokio::test]
async fn test_e2e_alternatives_workflow() {
    // Step 1: Set up service registry with multiple encryption providers
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    // Register 3 encryption providers with different protocols and health
    let beardog_id = registry
        .register_service(
            "BearDog".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/beardog.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let fastcrypto_id = registry
        .register_service(
            "FastCrypto".to_string(),
            vec!["encryption".to_string()],
            "tcp://localhost:5000".to_string(),
            "tarpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let slowcrypto_id = registry
        .register_service(
            "SlowCrypto".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/slowcrypto.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Set different health statuses
    registry.update_health(&beardog_id, "healthy".to_string()).await.unwrap();
    registry.update_health(&fastcrypto_id, "healthy".to_string()).await.unwrap();
    registry.update_health(&slowcrypto_id, "degraded".to_string()).await.unwrap();

    // Step 2: Request alternatives for an encryption node preferring json-rpc
    let node = create_test_node("encrypt", "encryption", Some("json-rpc"));
    let suggestions = checker.suggest_alternatives(&node).await.unwrap();

    // Step 3: Verify alternatives are ranked correctly
    assert_eq!(suggestions.alternatives.len(), 3);

    // Rank 1 should be BearDog (healthy + json-rpc protocol match)
    assert_eq!(suggestions.alternatives[0].rank, 1);
    assert_eq!(suggestions.alternatives[0].primal_name, "BearDog");
    assert_eq!(suggestions.alternatives[0].health_status, "healthy");
    assert_eq!(suggestions.alternatives[0].protocol, "json-rpc");
    assert_eq!(suggestions.alternatives[0].compatibility_score, 100);

    // Rank 2 should be SlowCrypto (degraded + json-rpc protocol match)
    assert_eq!(suggestions.alternatives[1].rank, 2);
    assert_eq!(suggestions.alternatives[1].primal_name, "SlowCrypto");
    assert_eq!(suggestions.alternatives[1].health_status, "degraded");
    assert_eq!(suggestions.alternatives[1].protocol, "json-rpc");

    // Rank 3 should be FastCrypto (healthy but protocol mismatch)
    assert_eq!(suggestions.alternatives[2].rank, 3);
    assert_eq!(suggestions.alternatives[2].primal_name, "FastCrypto");
    assert_eq!(suggestions.alternatives[2].health_status, "healthy");
    assert_eq!(suggestions.alternatives[2].protocol, "tarpc");
    assert!(
        suggestions.alternatives[2].compatibility_score
            < suggestions.alternatives[0].compatibility_score
    );

    // Verify recommendation is BearDog
    assert!(suggestions.recommendation.is_some());
    let recommendation = suggestions.recommendation.unwrap();
    assert!(recommendation.service_id.starts_with("beardog-"));
}

#[tokio::test]
async fn test_e2e_real_registry_integration() {
    // This test simulates a real-world scenario where primals register themselves
    // and a graph needs to be validated before execution

    // Step 1: Primals register themselves on startup
    let registry = Arc::new(ServiceRegistry::new());

    // Simulate primal startup registrations
    let beardog_id = registry
        .register_service(
            "BearDog".to_string(),
            vec!["encryption".to_string(), "identity".to_string(), "trust".to_string()],
            "/run/user/1000/beardog-nat0-node-alpha.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let nestgate_id = registry
        .register_service(
            "NestGate".to_string(),
            vec!["storage".to_string(), "persistence".to_string()],
            "/run/user/1000/nestgate-nat0-node-alpha.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let toadstool_id = registry
        .register_service(
            "ToadStool".to_string(),
            vec!["compute".to_string(), "execution".to_string()],
            "/run/user/1000/toadstool-nat0-node-alpha.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    // Simulate health checks (all healthy after startup)
    for service_id in [&beardog_id, &nestgate_id, &toadstool_id] {
        registry.update_health(service_id, "healthy".to_string()).await.unwrap();
    }

    // Step 2: biomeOS creates a graph for data processing
    let graph = create_test_graph(vec![
        create_test_node("decrypt-input", "encryption", Some("json-rpc")),
        create_test_node("process-data", "compute", Some("json-rpc")),
        create_test_node("store-result", "storage", Some("json-rpc")),
        create_test_node("verify-identity", "identity", Some("json-rpc")),
    ]);

    // Step 3: Check if all required capabilities are available
    let checker = AvailabilityChecker::new(registry.clone());
    let report = checker.check_availability(&graph).await.unwrap();

    // Verify 100% availability
    assert_eq!(report.summary.total_nodes, 4);
    assert_eq!(report.summary.available_nodes, 4);
    assert_eq!(report.summary.availability_percent, 100.0);
    assert!(report.unavailable.is_empty());
    assert!(report.unhealthy.is_empty());
    assert!(report.degraded.is_empty());

    // Verify each node has a primal assigned
    for node_id in ["decrypt-input", "process-data", "store-result", "verify-identity"] {
        let node_status = report.details.get(node_id).unwrap();
        assert_eq!(node_status.status, NodeAvailabilityStatus::Available);
        assert!(node_status.primal.is_some());
        assert!(node_status.service_id.is_some());
        assert!(node_status.endpoint.is_some());
    }

    // Step 4: Simulate a primal going down
    registry.update_health(&toadstool_id, "down".to_string()).await.unwrap();

    // Step 5: Re-check availability
    let report2 = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report2.summary.available_nodes, 3); // Only 3 now
    assert_eq!(report2.unhealthy.len(), 1); // process-data node is unhealthy
    assert!(report2.unhealthy.contains(&"process-data".to_string()));

    // Step 6: Get alternatives for the unhealthy node
    let process_node = create_test_node("process-data", "compute", Some("json-rpc"));
    let alternatives = checker.suggest_alternatives(&process_node).await.unwrap();

    // Should still find ToadStool (even though unhealthy) and show status
    assert_eq!(alternatives.alternatives.len(), 1);
    assert_eq!(alternatives.alternatives[0].primal_name, "ToadStool");
    assert_eq!(alternatives.alternatives[0].health_status, "down");
    // Score: down=0 + json-rpc protocol=40 + timestamp=10 = 50
    assert_eq!(alternatives.alternatives[0].compatibility_score, 50);
}
