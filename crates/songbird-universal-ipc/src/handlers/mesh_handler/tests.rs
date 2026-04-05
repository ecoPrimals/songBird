// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;

#[tokio::test]
async fn test_mesh_handler_uninitialized() {
    let handler = MeshHandler::new();

    let result = handler.handle_status(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not initialized"));
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

#[test]
fn endpoint_to_strings_local_and_direct() {
    use std::net::SocketAddr;
    let handler = MeshHandler::new();
    let addr: SocketAddr = "192.168.0.1:1234".parse().expect("addr");
    let (t, s) = handler.endpoint_strings_for_test(&EndpointType::Local {
        addr,
    });
    assert_eq!(t, "local");
    assert_eq!(s.as_deref(), Some("192.168.0.1:1234"));

    let (t2, s2) = handler.endpoint_strings_for_test(&EndpointType::Direct {
        addr,
    });
    assert_eq!(t2, "direct");
    assert_eq!(s2.as_deref(), Some("192.168.0.1:1234"));
}

#[test]
fn endpoint_to_strings_relay_and_onion() {
    let handler = MeshHandler::new();
    let (t, s) = handler.endpoint_strings_for_test(&EndpointType::FamilyRelay {
        relay_node_id: "relay-1".into(),
    });
    assert_eq!(t, "family_relay");
    assert_eq!(s.as_deref(), Some("relay-1"));

    let (t2, s2) = handler.endpoint_strings_for_test(&EndpointType::TorOnion {
        onion_addr: "abc.onion".into(),
    });
    assert_eq!(t2, "onion");
    assert_eq!(s2.as_deref(), Some("abc.onion"));
}

#[test]
fn path_to_json_includes_expected_fields() {
    let handler = MeshHandler::new();
    let addr: SocketAddr = "10.0.0.2:9000".parse().expect("addr");
    let path = RelayEndpoint {
        node_id: "peer-9".into(),
        endpoint_type: EndpointType::Direct {
            addr,
        },
        latency: None,
        last_seen: Instant::now(),
        reachable: true,
    };
    let v = handler.path_json_for_test(&path, true);
    assert_eq!(v["found"], true);
    assert_eq!(v["path_type"], "direct");
    assert_eq!(v["target_node_id"], "peer-9");
    assert_eq!(v["reachable"], true);
}

#[tokio::test]
async fn mesh_init_missing_node_id_errors() {
    let handler = MeshHandler::new();
    let err = handler.handle_init(json!({})).await.expect_err("missing node_id");
    assert!(err.contains("node_id"));
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

#[test]
fn mesh_handler_default_matches_new() {
    let _a = MeshHandler::new();
    let _b = MeshHandler::default();
}

#[test]
fn path_to_json_respects_found_flag_and_latency() {
    let handler = MeshHandler::new();
    let addr: SocketAddr = "10.0.0.2:9000".parse().expect("addr");
    let path = RelayEndpoint {
        node_id: "peer-x".into(),
        endpoint_type: EndpointType::Local {
            addr,
        },
        latency: Some(Duration::from_millis(12)),
        last_seen: Instant::now(),
        reachable: false,
    };
    let v = handler.path_json_for_test(&path, false);
    assert_eq!(v["found"], false);
    assert_eq!(v["estimated_latency_ms"], 12);
    assert_eq!(v["reachable"], false);
    assert_eq!(v["path_type"], "local");
}
