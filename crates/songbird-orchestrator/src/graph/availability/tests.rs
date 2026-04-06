// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
use std::sync::Arc;

use super::*;
use crate::graph::types::{Graph, GraphMetadata, GraphNode};
use crate::ipc::registry::ServiceRegistry;
use crate::ipc::types::PrimalEndpoint;

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
async fn test_all_available() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    registry
        .register_service(
            "security provider".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/security-provider.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption", None)],
        vec![],
        GraphMetadata::default(),
    );

    let report = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report.available.len(), 1);
    assert_eq!(report.unavailable.len(), 0);
    assert_eq!(report.summary.availability_percent, 100.0);
}

#[tokio::test]
async fn test_some_unavailable() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    registry
        .register_service(
            "security provider".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/security-provider.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![
            create_test_node("node-1", "encryption", None),
            create_test_node("node-2", "storage", None),
        ],
        vec![],
        GraphMetadata::default(),
    );

    let report = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report.available.len(), 1);
    assert_eq!(report.unavailable.len(), 1);
    assert!(report.unavailable.contains(&"node-2".to_string()));
    assert_eq!(report.summary.availability_percent, 50.0);
}

#[tokio::test]
async fn test_no_primals_registered() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption", None)],
        vec![],
        GraphMetadata::default(),
    );

    let report = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report.available.len(), 0);
    assert_eq!(report.unavailable.len(), 1);
    assert_eq!(report.summary.availability_percent, 0.0);
}

#[tokio::test]
async fn test_protocol_filtering() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    registry
        .register_service(
            "FastPrimal".to_string(),
            vec!["encryption".to_string()],
            "tcp://localhost:5000".to_string(),
            "tarpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let node = create_test_node("node-1", "encryption", Some("json-rpc"));

    let report = checker.check_node_availability(&node).await.unwrap();
    assert_eq!(report.status, NodeAvailabilityStatus::Available);
    assert_eq!(report.protocol, Some("tarpc".to_string()));
}

#[tokio::test]
async fn test_suggest_alternatives_ranking() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    let healthy_service_id = registry
        .register_service(
            "HealthyPrimal".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/healthy.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let degraded_service_id = registry
        .register_service(
            "DegradedPrimal".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/degraded.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    registry.update_health(&healthy_service_id, "healthy".to_string()).await.unwrap();

    registry.update_health(&degraded_service_id, "degraded".to_string()).await.unwrap();

    let node = create_test_node("node-1", "encryption", Some("json-rpc"));
    let suggestions = checker.suggest_alternatives(&node).await.unwrap();

    assert_eq!(suggestions.alternatives.len(), 2);
    assert_eq!(suggestions.alternatives[0].rank, 1);
    assert!(suggestions.alternatives[0].primal_name.contains("Healthy"));
    assert!(
        suggestions.alternatives[0].compatibility_score
            > suggestions.alternatives[1].compatibility_score
    );
}

#[tokio::test]
async fn test_unhealthy_primal() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    let service_id = registry
        .register_service(
            "UnhealthyPrimal".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/unhealthy.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    registry.update_health(&service_id, "down".to_string()).await.unwrap();

    let node = create_test_node("node-1", "encryption", None);
    let report = checker.check_node_availability(&node).await.unwrap();

    assert_eq!(report.status, NodeAvailabilityStatus::Unhealthy);
    assert_eq!(report.health_status, Some("down".to_string()));
}

#[tokio::test]
async fn test_degraded_primal() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    let service_id = registry
        .register_service(
            "DegradedPrimal".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/degraded.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    registry.update_health(&service_id, "degraded".to_string()).await.unwrap();

    let node = create_test_node("node-1", "encryption", None);
    let report = checker.check_node_availability(&node).await.unwrap();

    assert_eq!(report.status, NodeAvailabilityStatus::Degraded);
    assert_eq!(report.health_status, Some("degraded".to_string()));
}

#[tokio::test]
async fn test_health_status_changes() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());

    let service_id = registry
        .register_service(
            "FlakeyPrimal".to_string(),
            vec!["encryption".to_string()],
            "/run/user/1000/flakey.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .unwrap();

    let graph = Graph::new(
        "test".to_string(),
        "Test".to_string(),
        vec![create_test_node("node-1", "encryption", None)],
        vec![],
        GraphMetadata::default(),
    );

    let report1 = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report1.summary.availability_percent, 100.0);

    registry.update_health(&service_id, "healthy".to_string()).await.unwrap();
    let report2 = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report2.summary.availability_percent, 100.0);

    registry.update_health(&service_id, "degraded".to_string()).await.unwrap();
    let report3 = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report3.available.len(), 0);
    assert_eq!(report3.degraded.len(), 1);

    registry.update_health(&service_id, "down".to_string()).await.unwrap();
    let report4 = checker.check_availability(&graph).await.unwrap();
    assert_eq!(report4.available.len(), 0);
    assert_eq!(report4.unhealthy.len(), 1);
    assert_eq!(report4.summary.availability_percent, 0.0);
}

fn sample_primal(health: &str, protocol: &str) -> PrimalEndpoint {
    PrimalEndpoint {
        service_id: "sid-test".to_string(),
        primal_name: "primal-test".to_string(),
        capabilities: vec!["encryption".to_string()],
        endpoint: "/tmp/test.sock".to_string(),
        protocol: protocol.to_string(),
        last_health_check: "2020-01-01T00:00:00Z".to_string(),
        health_status: health.to_string(),
    }
}

#[test]
fn compatibility_score_healthy_beats_unknown_when_protocol_matches() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", Some("json-rpc"));
    let healthy = sample_primal("healthy", "json-rpc");
    let unknown = sample_primal("unknown", "json-rpc");
    let s_healthy = checker.calculate_compatibility_score(&node, &healthy);
    let s_unknown = checker.calculate_compatibility_score(&node, &unknown);
    assert!(s_healthy > s_unknown);
    assert_eq!(s_healthy, 100);
    assert_eq!(s_unknown, 95);
}

#[test]
fn compatibility_score_exact_protocol_beats_json_rpc_fallback() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", Some("tarpc"));
    let exact = sample_primal("healthy", "tarpc");
    let universal = sample_primal("healthy", "json-rpc");
    assert!(
        checker.calculate_compatibility_score(&node, &exact)
            > checker.calculate_compatibility_score(&node, &universal)
    );
}

#[test]
fn compatibility_score_down_health_zeroes_health_points() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let down = sample_primal("down", "json-rpc");
    assert_eq!(checker.calculate_compatibility_score(&node, &down), 30);
}

#[test]
fn suggestion_reason_notes_degraded_health() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let primal = sample_primal("degraded", "json-rpc");
    let reason = checker.generate_suggestion_reason(&node, &primal, 42);
    assert!(reason.contains("degraded"));
    assert!(reason.contains("score: 42"));
}

#[tokio::test]
async fn empty_graph_reports_zero_availability_percent() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let graph = Graph::new(
        "empty".to_string(),
        "Empty".to_string(),
        vec![],
        vec![],
        GraphMetadata::default(),
    );
    let report = checker.check_availability(&graph).await.expect("empty graph check");
    assert_eq!(report.summary.total_nodes, 0);
    assert_eq!(report.summary.available_nodes, 0);
    assert_eq!(report.summary.availability_percent, 0.0);
}

#[test]
fn compatibility_score_no_recency_when_last_health_check_empty() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let mut p = sample_primal("healthy", "json-rpc");
    p.last_health_check = String::new();
    assert_eq!(checker.calculate_compatibility_score(&node, &p), 70);
}

#[test]
fn suggestion_reason_lists_healthy_for_healthy_primal() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let primal = sample_primal("healthy", "json-rpc");
    let reason = checker.generate_suggestion_reason(&node, &primal, 99);
    assert!(reason.contains("healthy"));
    assert!(reason.contains("capability 'encryption'"));
    assert!(reason.contains("score: 99"));
}

#[test]
fn node_availability_status_serde_roundtrip() {
    let s = serde_json::to_string(&NodeAvailabilityStatus::Unavailable).expect("serialize");
    assert_eq!(s, "\"unavailable\"");
    let v: NodeAvailabilityStatus = serde_json::from_str(&s).expect("deserialize");
    assert_eq!(v, NodeAvailabilityStatus::Unavailable);
}

#[test]
fn suggestion_reason_labels_unknown_health_as_unhealthy() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let mut p = sample_primal("weird_state", "json-rpc");
    let reason = checker.generate_suggestion_reason(&node, &p, 7);
    assert!(reason.contains("unhealthy"), "{}", reason);

    p.health_status = "healthy".to_string();
    let ok = checker.generate_suggestion_reason(&node, &p, 100);
    assert!(ok.contains("healthy"));
}

#[test]
fn compatibility_score_nonstandard_health_gets_zero_health_points() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry);
    let node = create_test_node("n1", "encryption", None);
    let p = sample_primal("custom-unknown-state", "json-rpc");
    assert_eq!(checker.calculate_compatibility_score(&node, &p), 30);
}

#[tokio::test]
async fn availability_summary_partial_nodes() {
    let registry = Arc::new(ServiceRegistry::new());
    let checker = AvailabilityChecker::new(registry.clone());
    registry
        .register_service(
            "A".to_string(),
            vec!["c1".to_string()],
            "/a.sock".to_string(),
            "json-rpc".to_string(),
            30,
        )
        .await
        .expect("register");
    let graph = Graph::new(
        "g".to_string(),
        "G".to_string(),
        vec![create_test_node("x", "c1", None), create_test_node("y", "missing-cap", None)],
        vec![],
        GraphMetadata::default(),
    );
    let report = checker.check_availability(&graph).await.expect("check");
    assert_eq!(report.summary.total_nodes, 2);
    assert_eq!(report.summary.available_nodes, 1);
    assert!((report.summary.availability_percent - 50.0).abs() < f64::EPSILON);
}
