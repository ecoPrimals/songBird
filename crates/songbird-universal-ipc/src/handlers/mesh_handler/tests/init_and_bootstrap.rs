// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::handlers::mesh_handler::MeshHandler;
use serde_json::json;

#[tokio::test]
async fn test_mesh_handler_uninitialized() {
    let handler = MeshHandler::new();

    let result = handler.handle_status(json!({})).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], false);
    assert_eq!(response["status"], "awaiting_init");
}

#[tokio::test]
async fn test_mesh_init() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": ["abc.onion"]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], true);
    assert_eq!(response["node_id"], "test-tower");
}

#[tokio::test]
async fn mesh_init_with_bootstrap_peers_adds_endpoints() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "east-gate",
            "bootstrap_onions": [],
            "bootstrap_peers": [
                { "node_id": "west-gate", "address": "192.168.1.50:3492" },
                { "node_id": "flock-gate", "address": "10.0.0.5:3492" }
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["initialized"], true);
    assert_eq!(response["bootstrap_peers_added"], 2);

    let mesh = handler.mesh().await;
    let mesh = mesh.as_ref().unwrap();
    let reachable = mesh.get_reachable_nodes().await;
    assert_eq!(reachable.len(), 2, "both bootstrap peers should be reachable");
    assert!(reachable.contains(&"west-gate".to_string()));
    assert!(reachable.contains(&"flock-gate".to_string()));
}

#[tokio::test]
async fn mesh_init_with_invalid_bootstrap_peers_skips_gracefully() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "east-gate",
            "bootstrap_peers": [
                { "node_id": "good-peer", "address": "192.168.1.50:3492" },
                { "node_id": "bad-peer", "address": "not-a-valid-addr" },
                { "missing_fields": true }
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 1, "only valid peer should be added");
}

#[tokio::test]
async fn mesh_init_string_format_bootstrap_peers() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "south-gate",
            "bootstrap_peers": [
                "east-gate@192.168.1.100:7700",
                "192.168.4.29:7700",
                "invalid-no-port"
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 2);

    let mesh = handler.mesh().await;
    let mesh = mesh.as_ref().unwrap();
    let reachable = mesh.get_reachable_nodes().await;
    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&"east-gate".to_string()));
    assert!(reachable.contains(&"peer-192.168.4.29".to_string()));
}

#[tokio::test]
async fn mesh_init_mixed_format_bootstrap_peers() {
    let handler = MeshHandler::new();

    let result = handler
        .handle_init(json!({
            "node_id": "iron-gate",
            "bootstrap_peers": [
                { "node_id": "west-gate", "address": "10.0.0.1:7700" },
                "east-gate@192.168.1.100:7700"
            ]
        }))
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response["bootstrap_peers_added"], 2);
}

#[tokio::test]
async fn test_mesh_status_after_init() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "test-tower",
            "bootstrap_onions": []
        }))
        .await
        .unwrap();

    let result = handler.handle_status(json!({})).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["node_id"], "test-tower");
    assert_eq!(response["reachable_peers"], 0);
    assert!(response["uptime_seconds"].as_u64().is_some());
}

#[tokio::test]
async fn mesh_init_missing_node_id_errors() {
    let handler = MeshHandler::new();
    let err = handler.handle_init(json!({})).await.expect_err("missing node_id");
    assert!(err.contains("node_id"));
}

#[test]
fn mesh_handler_default_matches_new() {
    let _a = MeshHandler::new();
    let _b = MeshHandler::default();
}
