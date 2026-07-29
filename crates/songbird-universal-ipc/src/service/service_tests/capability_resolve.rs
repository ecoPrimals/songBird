// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::registry::ServiceRegistry;
use crate::service::*;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn ipc_resolve_by_capability_returns_provider() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "security-provider",
                "capabilities": ["crypto.sign", "crypto.verify"],
                "endpoint": "/tmp/security.sock"
            }),
        )
        .await
        .unwrap();

    let result = handler
        .handle("ipc.resolve", json!({ "capability": "crypto.sign" }))
        .await
        .expect("resolve by capability");
    assert!(result["native_endpoint"].as_str().unwrap().contains("security"));
    assert!(result["capabilities"].as_array().unwrap().iter().any(|c| c == "crypto.sign"));
}

#[tokio::test]
async fn ipc_resolve_by_capability_unknown_errors() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let err = handler
        .handle("ipc.resolve", json!({ "capability": "no.such.cap" }))
        .await
        .expect_err("no provider");
    assert!(err.contains("No provider found"), "unexpected: {err}");
}

#[tokio::test]
async fn ipc_resolve_missing_both_params_errors() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let err = handler.handle("ipc.resolve", json!({})).await.expect_err("missing params");
    assert!(err.contains("primal_id") && err.contains("capability"), "unexpected: {err}");
}

#[tokio::test]
async fn ipc_resolve_capability_preferred_over_primal_id() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "storage-primal",
                "capabilities": ["storage.put", "storage.get"],
                "endpoint": "/tmp/storage.sock"
            }),
        )
        .await
        .unwrap();

    let result = handler
        .handle(
            "ipc.resolve",
            json!({ "primal_id": "storage-primal", "capability": "storage.put" }),
        )
        .await
        .expect("both params — capability takes precedence");
    assert!(result["native_endpoint"].as_str().unwrap().contains("storage"));
}

#[tokio::test]
async fn ipc_resolve_capability_falls_back_to_primal_name() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto.sign", "security"],
                "endpoint": "/tmp/beardog.sock"
            }),
        )
        .await
        .unwrap();

    // "beardog" is not a capability token, but IS a primal name — fallback kicks in
    let result = handler
        .handle("ipc.resolve", json!({ "capability": "beardog" }))
        .await
        .expect("capability 'beardog' not found, but fallback to primal name succeeds");
    assert_eq!(result["native_endpoint"].as_str().unwrap(), "unix:///tmp/beardog.sock");
}

#[tokio::test]
async fn ipc_resolve_name_alias_for_primal_id() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto.sign"],
                "endpoint": "/tmp/beardog.sock"
            }),
        )
        .await
        .unwrap();

    // `name` is a serde alias for `primal_id`
    let result = handler
        .handle("ipc.resolve", json!({ "name": "beardog" }))
        .await
        .expect("name alias should work like primal_id");
    assert_eq!(result["native_endpoint"].as_str().unwrap(), "unix:///tmp/beardog.sock");
}

#[tokio::test]
async fn ipc_resolve_by_name_method_alias() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto.sign"],
                "endpoint": "/tmp/beardog.sock"
            }),
        )
        .await
        .unwrap();

    // `ipc.resolve_by_name` is a normalization alias for `ipc.resolve`
    let result = handler
        .handle("ipc.resolve_by_name", json!({ "name": "beardog" }))
        .await
        .expect("ipc.resolve_by_name should route to ipc.resolve handler");
    assert_eq!(result["native_endpoint"].as_str().unwrap(), "unix:///tmp/beardog.sock");
}

#[tokio::test]
async fn discovery_peers_returns_mesh_bootstrap_peers() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry);

    let init_result = handler
        .handle(
            "mesh.init",
            json!({
                "node_id": "east-gate",
                "bootstrap_peers": [
                    { "node_id": "iron-gate", "address": "192.168.1.238:7700" },
                    { "node_id": "west-gate", "address": "10.0.0.5:3492" }
                ]
            }),
        )
        .await
        .unwrap();
    assert_eq!(init_result["initialized"], true);
    assert_eq!(init_result["bootstrap_peers_added"], 2);

    let peers_result = handler.handle("discovery.peers", json!({})).await.unwrap();
    let peers = peers_result["peers"].as_array().unwrap();

    assert_eq!(
        peers_result["total_count"].as_u64().unwrap(),
        2,
        "discovery.peers should return mesh bootstrap peers"
    );

    let node_ids: Vec<&str> = peers.iter().filter_map(|p| p["node_id"].as_str()).collect();
    assert!(node_ids.contains(&"iron-gate"), "should contain iron-gate");
    assert!(node_ids.contains(&"west-gate"), "should contain west-gate");

    let iron = peers.iter().find(|p| p["node_id"] == "iron-gate").unwrap();
    assert_eq!(iron["address"].as_str().unwrap(), "192.168.1.238:7700");
    assert_eq!(iron["tcp_port"].as_u64().unwrap(), 7700);
}
