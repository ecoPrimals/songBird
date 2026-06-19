// SPDX-License-Identifier: AGPL-3.0-or-later
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

fn peer_by_id<'a>(peers: &'a [Value], node_id: &str) -> &'a Value {
    peers
        .iter()
        .find(|p| p["node_id"].as_str() == Some(node_id))
        .unwrap_or_else(|| panic!("peer {node_id} not found in {peers:?}"))
}

#[tokio::test]
async fn mesh_peers_after_init_returns_bootstrap_info() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_onions": [],
            "bootstrap_peers": [
                { "node_id": "golgi", "address": "10.0.0.10:3492" },
                { "node_id": "sporeGate", "address": "10.0.0.20:3492" }
            ]
        }))
        .await
        .expect("init");

    let response = handler.handle_peers(json!({})).await.expect("peers");
    assert_eq!(response["total"], 2);

    let peers = response["peers"].as_array().expect("peers array");

    let golgi = peer_by_id(peers, "golgi");
    assert_eq!(golgi["path_type"], "direct");
    assert_eq!(golgi["address"], "10.0.0.10:3492");
    assert_eq!(golgi["reachable"], true);

    let spore_gate = peer_by_id(peers, "sporeGate");
    assert_eq!(spore_gate["path_type"], "direct");
    assert_eq!(spore_gate["address"], "10.0.0.20:3492");
    assert_eq!(spore_gate["reachable"], true);
}

#[tokio::test]
async fn mesh_topology_returns_graph_after_init() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_onions": [],
            "bootstrap_peers": [
                { "node_id": "golgi", "address": "10.0.0.10:3492" },
                { "node_id": "sporeGate", "address": "10.0.0.20:3492" }
            ]
        }))
        .await
        .expect("init");

    let response = handler.handle_topology(json!({})).await.expect("topology");

    let nodes = response["nodes"].as_array().expect("nodes");
    assert_eq!(response["node_count"], 3);
    assert!(nodes.iter().any(|n| n["id"] == "flockGate" && n["role"] == "self"));
    assert!(nodes.iter().any(|n| n["id"] == "golgi" && n["role"] == "peer"));
    assert!(nodes.iter().any(|n| n["id"] == "sporeGate" && n["role"] == "peer"));

    let edges = response["edges"].as_array().expect("edges");
    assert_eq!(response["edge_count"], 2);
    assert!(
        edges.iter().any(|e| {
            e["from"] == "flockGate" && e["to"] == "golgi" && e["path_type"] == "direct"
        })
    );
    assert!(edges.iter().any(|e| {
        e["from"] == "flockGate" && e["to"] == "sporeGate" && e["path_type"] == "direct"
    }));
}

#[tokio::test]
async fn mesh_announce_as_relay_true_records_relay() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let before = handler.handle_status(json!({})).await.expect("status before announce");
    assert_eq!(before["relay_capable"], false);

    handler
        .handle_announce(json!({
            "as_relay": true,
            "capabilities": ["relay"]
        }))
        .await
        .expect("announce");

    let after = handler.handle_status(json!({})).await.expect("status after announce");
    assert_eq!(after["relay_capable"], true);
    assert_eq!(after["relay_enabled"], true);
}

#[tokio::test(start_paused = true)]
async fn mesh_health_check_with_stale_peer() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_onions": [],
            "bootstrap_peers": [
                { "node_id": "golgi", "address": "10.0.0.10:3492" }
            ]
        }))
        .await
        .expect("init");

    tokio::time::advance(Duration::from_secs(61)).await;

    {
        let mesh = handler.mesh().await;
        let mesh = mesh.as_ref().expect("mesh");
        mesh.backdate_endpoint_last_seen("golgi", Duration::from_secs(120)).await;
    }

    let health = handler
        .handle_health_check(json!({
            "target_node_ids": ["golgi"]
        }))
        .await
        .expect("health check");
    let results = health["results"].as_array().expect("results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["node_id"], "golgi");
    assert_eq!(results[0]["healthy"], false);

    let mesh = handler.mesh().await;
    let mesh = mesh.as_ref().expect("mesh");
    let paths = mesh.get_all_paths("golgi").await;
    assert_eq!(paths.len(), 1);
    assert!(!paths[0].reachable, "stale peer should be marked unreachable");
}

#[tokio::test]
async fn mesh_init_with_duplicate_node_ids_deduplicates() {
    let handler = MeshHandler::new();

    let init = handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_peers": [
                { "node_id": "golgi", "address": "10.0.0.10:3492" },
                { "node_id": "golgi", "address": "10.0.0.11:3492" }
            ]
        }))
        .await
        .expect("init");

    assert_eq!(init["bootstrap_peers_added"], 1);

    let response = handler.handle_peers(json!({})).await.expect("peers");
    assert_eq!(response["total"], 1);

    let peers = response["peers"].as_array().expect("peers array");
    let golgi = peer_by_id(peers, "golgi");
    assert_eq!(golgi["address"], "10.0.0.11:3492");
}

#[tokio::test(start_paused = true)]
async fn mesh_auto_discover_timeout_returns_empty() {
    let handler = MeshHandler::new();

    handler
        .handle_init(json!({
            "node_id": "flockGate",
            "bootstrap_onions": []
        }))
        .await
        .expect("init");

    let discover = handler
        .handle_auto_discover(json!({
            "timeout_ms": 100,
            "broadcast_port": 15353
        }))
        .await
        .expect("auto_discover");

    assert_eq!(discover["discovered"], 0);
    assert!(discover["peers"].as_array().expect("peers").is_empty());
    assert_eq!(discover["timeout_ms"], 100);
}

#[tokio::test]
async fn mesh_peers_include_offline_flag() {
    let handler = MeshHandler::new();
    let golgi_addr: SocketAddr = "10.0.0.10:3492".parse().expect("golgi addr");
    let spore_addr: SocketAddr = "10.0.0.20:3492".parse().expect("sporeGate addr");

    handler
        .test_init_with_peers(
            "flockGate",
            &[
                (String::from("golgi"), golgi_addr, true),
                (String::from("sporeGate"), spore_addr, false),
            ],
        )
        .await;

    let with_offline =
        handler.handle_peers(json!({ "include_offline": true })).await.expect("peers with offline");
    assert_eq!(with_offline["total"], 2);
    assert_eq!(with_offline["online"], 1);

    let peers = with_offline["peers"].as_array().expect("peers array");
    let offline = peer_by_id(peers, "sporeGate");
    assert_eq!(offline["reachable"], false);

    let online_only = handler.handle_peers(json!({})).await.expect("peers online only");
    assert_eq!(online_only["total"], 1);
    assert_eq!(online_only["online"], 1);

    let online_peers = online_only["peers"].as_array().expect("online peers");
    assert!(online_peers.iter().all(|p| p["node_id"] != "sporeGate"));
    assert!(online_peers.iter().any(|p| p["node_id"] == "golgi"));
}
