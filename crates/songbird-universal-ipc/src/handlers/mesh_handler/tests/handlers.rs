// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::handlers::mesh_handler::MeshHandler;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn node_id_async_returns_shared_arc_without_extra_allocation() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "tower-node-abc",
            "bootstrap_onions": []
        }))
        .await
        .expect("mesh init");

    let id_a = handler.node_id_async().await;
    let id_b = handler.node_id_async().await;
    assert_eq!(id_a.as_ref(), "tower-node-abc");
    assert!(Arc::ptr_eq(&id_a, &id_b));
}

#[test]
fn node_id_blocking_accessor_returns_arc_str() {
    let handler = MeshHandler::with_mesh(
        songbird_onion_relay::mesh::BeaconMesh::new(String::from("sync-node"), vec![]),
        "sync-node",
    );
    let id = handler.node_id();
    assert_eq!(id.as_ref(), "sync-node");
}

#[tokio::test]
async fn test_mesh_find_path_not_found() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_find_path(json!({
            "target_node_id": "unknown-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["found"], false);
    assert_eq!(response["reason"], "peer_not_discovered");
}

#[tokio::test]
async fn test_mesh_find_path_with_bootstrap() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": ["bootstrap.onion"]
        }))
        .await
        .unwrap();

    let result = handler
        .handle_find_path(json!({
            "target_node_id": "remote-peer"
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["found"], true);
    assert_eq!(response["path_type"], "onion");
}

#[tokio::test]
async fn test_mesh_announce() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_announce(json!({
            "as_relay": true,
            "capabilities": ["relay", "stun"]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["announced"], true);
}

#[tokio::test]
async fn test_mesh_peers_empty() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler.handle_peers(json!({})).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["total"], 0);
    assert!(response["peers"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_mesh_auto_discover() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_auto_discover(json!({
            "timeout_ms": 100,
            "broadcast_port": 15353
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["discovered"], 0);
    assert!(response["peers"].as_array().unwrap().is_empty());
    assert_eq!(response["broadcast_port"], 15353);
}

#[tokio::test]
async fn test_mesh_auto_discover_requires_init() {
    let handler = MeshHandler::new();

    let result = handler.handle_auto_discover(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not initialized"));
}

#[tokio::test]
async fn mesh_topology_requires_init() {
    let handler = MeshHandler::new();
    let err = handler.handle_topology(json!({})).await.expect_err("topology");
    assert!(err.contains("not initialized"), "unexpected: {err}");
}

#[tokio::test]
async fn test_mesh_health_check() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler
        .handle_health_check(json!({
            "target_node_ids": ["unknown-peer"],
            "timeout_ms": 1000
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["all_healthy"], false);
}

#[tokio::test]
async fn mesh_find_path_missing_target_errors() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "tower",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let err = handler.handle_find_path(json!({})).await.expect_err("missing target");
    assert!(err.contains("target_node_id"));
}

#[tokio::test]
async fn mesh_announce_as_relay_false_short_circuits() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "tower",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let v = handler.handle_announce(json!({ "as_relay": false })).await.expect("announce response");
    assert_eq!(v["announced"], false);
}

#[tokio::test]
async fn mesh_probe_latency_requires_init() {
    let handler = MeshHandler::new();
    let result = handler.handle_probe_latency(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Mesh not initialized"));
}

#[tokio::test]
async fn mesh_probe_latency_with_no_peers_returns_empty() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let result = handler.handle_probe_latency(json!({ "timeout_ms": 1000 })).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["probed"], 0);
    assert_eq!(response["total_peers"], 0);
    assert_eq!(response["results"].as_array().expect("results array").len(), 0);
}

#[tokio::test]
async fn mesh_probe_latency_skips_non_tcp_endpoints() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-relay-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    // Add a peer with a relay endpoint (no direct TCP address)
    {
        let mesh = handler.mesh.read().await;
        let mesh = mesh.as_ref().unwrap();
        mesh.record_relay_path(
            "relay-peer".to_string(),
            "via-relay".to_string(),
            Duration::from_millis(50),
        )
        .await;
    }

    let result = handler.handle_probe_latency(json!({})).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let results = response["results"].as_array().expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["status"], "skipped");
    assert_eq!(results[0]["reason"], "no_tcp_endpoint");
}

#[tokio::test]
async fn mesh_probe_latency_attempts_unreachable_peer() {
    let handler = MeshHandler::new();
    handler
        .handle_init(json!({
            "node_id": "probe-unreachable-test",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    // Add a peer with a direct TCP endpoint that won't be reachable
    {
        let mesh = handler.mesh.read().await;
        let mesh = mesh.as_ref().unwrap();
        mesh.record_direct_connection(
            "unreachable-peer".to_string(),
            "192.0.2.1:1".parse().unwrap(), // RFC 5737 documentation address
            Duration::from_millis(999),
        )
        .await;
    }

    let result = handler.handle_probe_latency(json!({ "timeout_ms": 200 })).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let results = response["results"].as_array().expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["node_id"], "unreachable-peer");
    assert_eq!(results[0]["status"], "error");
    assert!(
        results[0]["error"].as_str().unwrap().contains("timeout")
            || results[0]["error"].as_str().unwrap().contains("failed")
    );
}
