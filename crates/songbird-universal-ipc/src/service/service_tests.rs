// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::registry::ServiceRegistry;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::json;
use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_ipc_service_register() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let params = json!({
        "primal_id": "security",
        "capabilities": ["crypto", "btsp"],
        "endpoint": "/tmp/primal-security.sock"
    });

    let result = handler.handle("ipc.register", params).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    assert_eq!(result_value["virtual_endpoint"], "/primal/security");
}

#[tokio::test]
async fn test_ipc_service_resolve() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    // Register first
    let register_params = json!({
        "primal_id": "security",
        "capabilities": ["crypto"],
        "endpoint": "/tmp/primal-security.sock"
    });
    handler.handle("ipc.register", register_params).await.unwrap();

    // Then resolve
    let resolve_params = json!({
        "primal_id": "security"
    });

    let result = handler.handle("ipc.resolve", resolve_params).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    assert_eq!(result_value["virtual_endpoint"], "/primal/security");
    assert!(result_value["native_endpoint"].as_str().unwrap().contains("security"));
}

#[tokio::test]
async fn test_ipc_service_discover() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    // Register service with capability
    let register_params = json!({
        "primal_id": "security",
        "capabilities": ["crypto", "btsp"],
        "endpoint": "/tmp/primal-security.sock"
    });
    handler.handle("ipc.register", register_params).await.unwrap();

    // Discover by capability
    let discover_params = json!({
        "capability": "crypto"
    });

    let result = handler.handle("ipc.discover", discover_params).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    let providers = result_value["providers"].as_array().unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["primal_id"], "security");
}

#[tokio::test]
async fn test_primal_info_introspection() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let result = handler.handle("primal.info", json!({})).await;
    assert!(result.is_ok());

    let info = result.unwrap();
    assert_eq!(info["name"], "songbird");
    assert!(info["version"].is_string());
    assert!(info["capabilities"].is_array());
    assert!(info["capabilities"].as_array().unwrap().contains(&json!("discovery")));
    assert!(info["capabilities"].as_array().unwrap().contains(&json!("stun")));
}

#[tokio::test]
async fn test_primal_capabilities_introspection() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let result = handler.handle("primal.capabilities", json!({})).await;
    assert!(result.is_ok());

    let caps = result.unwrap();
    assert!(caps["capabilities"].is_array());

    let caps_array = caps["capabilities"].as_array().unwrap();
    assert!(!caps_array.is_empty());

    // Verify discovery capability exists with operations
    let discovery_cap = caps_array
        .iter()
        .find(|c| c["name"] == "discovery")
        .expect("discovery capability should exist");

    assert!(discovery_cap["operations"].is_array());
    assert!(discovery_cap["description"].is_string());
}

#[tokio::test]
async fn test_rpc_methods_introspection() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let result = handler.handle("rpc.methods", json!({})).await;
    assert!(result.is_ok());

    let methods = result.unwrap();
    assert!(methods["methods"].is_array());

    let methods_array = methods["methods"].as_array().unwrap();
    assert!(!methods_array.is_empty());

    // Verify introspection methods are listed
    let method_names: Vec<String> =
        methods_array.iter().filter_map(|m| m["name"].as_str()).map(String::from).collect();

    assert!(method_names.contains(&"primal.info".to_string()));
    assert!(method_names.contains(&"primal.capabilities".to_string()));
    assert!(method_names.contains(&"rpc.methods".to_string()));
    assert!(method_names.contains(&"ipc.register".to_string()));
}

#[tokio::test]
async fn test_ipc_service_list() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    // Register multiple services
    for (id, caps) in &[("security", vec!["crypto"]), ("squirrel", vec!["ai"])] {
        let params = json!({
            "primal_id": id,
            "capabilities": caps,
            "endpoint": format!("/tmp/primal-{id}.sock")
        });
        handler.handle("ipc.register", params).await.unwrap();
    }

    // List all
    let result = handler.handle("ipc.list", json!({})).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    let services = result_value["services"].as_array().unwrap();
    assert_eq!(services.len(), 2);
}

#[tokio::test]
async fn test_discover_capabilities() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let result = handler.handle("discover_capabilities", json!({})).await;
    assert!(result.is_ok());

    let caps = result.unwrap();
    assert_eq!(caps["primal"], "songbird");

    let capabilities = caps["capabilities"].as_array().unwrap();
    assert!(!capabilities.is_empty());

    // Verify key capabilities that other primals scan for
    let cap_strs: Vec<&str> = capabilities.iter().filter_map(|c| c.as_str()).collect();
    assert!(cap_strs.contains(&"http.request"), "must advertise http.request");
    assert!(cap_strs.contains(&"secure_http"), "must advertise secure_http");
    assert!(cap_strs.contains(&"discovery.peers"), "must advertise discovery.peers");
    assert!(cap_strs.contains(&"stun.detect"), "must advertise stun capability");
    assert!(cap_strs.contains(&"mesh.status"), "must advertise mesh capability");
    assert!(cap_strs.contains(&"punch.request"), "must advertise punch capability");
}

#[test]
fn ipc_register_params_deserialize_roundtrip() {
    let v = json!({
        "primal_id": "security",
        "capabilities": ["crypto"],
        "endpoint": "/tmp/x.sock"
    });
    let p: RegisterParams = serde_json::from_value(v).expect("RegisterParams");
    assert_eq!(p.primal_id, "security");
    assert_eq!(p.capabilities, vec!["crypto".to_string()]);
    assert_eq!(p.endpoint, "/tmp/x.sock");
}

#[test]
fn ipc_resolve_and_discover_params_deserialize() {
    let r: ResolveParams = serde_json::from_value(json!({"primal_id": "a"})).expect("resolve");
    assert_eq!(r.primal_id, "a");

    let d: DiscoverParams = serde_json::from_value(json!({"capability": "stun"})).expect("disc");
    assert_eq!(d.capability, "stun");
}

#[test]
fn ipc_list_and_provider_serialization_shapes() {
    let list = ListResult {
        services: vec![ServiceInfo {
            primal_id: "p".into(),
            virtual_endpoint: "/primal/p".into(),
            capabilities: vec!["c".into()],
        }],
    };
    let v = serde_json::to_value(&list).expect("list json");
    assert_eq!(v["services"][0]["primal_id"], "p");

    let dr = DiscoverResult {
        providers: vec![ProviderInfo {
            primal_id: "q".into(),
            virtual_endpoint: "/primal/q".into(),
            native_endpoint: "unix:///run/q".into(),
            capabilities: vec![],
        }],
    };
    let v2 = serde_json::to_value(&dr).expect("discover result");
    assert_eq!(v2["providers"][0]["native_endpoint"], "unix:///run/q");
}

#[test]
fn register_and_resolve_result_serialization() {
    let reg = RegisterResult {
        virtual_endpoint: "/primal/x".into(),
        registered_at: "t0".into(),
    };
    let v = serde_json::to_value(&reg).expect("RegisterResult json");
    assert_eq!(v["virtual_endpoint"], "/primal/x");

    let res = ResolveResult {
        virtual_endpoint: "/primal/x".into(),
        native_endpoint: "native".into(),
        capabilities: vec!["c".into()],
    };
    let v2 = serde_json::to_value(&res).expect("ResolveResult json");
    assert_eq!(v2["capabilities"][0], "c");
}

#[tokio::test]
async fn ipc_resolve_errors_when_primal_not_registered() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let err = handler
        .handle("ipc.resolve", json!({ "primal_id": "ghost" }))
        .await
        .expect_err("not registered");
    assert!(err.contains("not found") || err.contains("Not found") || err.contains("found"));
}

#[tokio::test]
async fn unknown_rpc_method_returns_error() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let err = handler.handle("no.such.method", json!({})).await.expect_err("unknown method");
    assert!(
        err.contains("unknown JSON-RPC method") || err.contains("Unknown method"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn health_liveness_returns_healthy_status_only() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let v = handler.handle("health.liveness", json!({})).await.expect("liveness");
    assert_eq!(v, json!({ "status": "healthy" }));
}

#[tokio::test]
async fn capabilities_list_returns_expected_tokens() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let v = handler.handle("capabilities.list", json!({})).await.expect("caps");
    let arr = v.as_array().expect("capabilities.list must return a JSON array");
    let strings: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
    for expected in crate::introspection::SONGBIRD_CAPABILITY_STRINGS {
        assert!(strings.contains(expected), "missing capability token {expected}");
    }
    assert_eq!(strings.len(), crate::introspection::SONGBIRD_CAPABILITY_STRINGS.len());
}

#[test]
fn federation_response_types_serialize_expected_shape() {
    let peers = FederationPeersResponse {
        peers: vec!["a".into(), "b".into()],
        total_count: 2,
        federation_enabled: true,
    };
    let v = serde_json::to_value(&peers).expect("FederationPeersResponse json");
    assert_eq!(v["peers"], json!(["a", "b"]));
    assert_eq!(v["total_count"], json!(2));
    assert_eq!(v["federation_enabled"], json!(true));
    assert!(v.get("comment").is_none());

    let status = FederationStatusResponse {
        enabled: true,
        active_connections: 3,
    };
    let s = serde_json::to_value(&status).expect("FederationStatusResponse json");
    assert_eq!(s["enabled"], json!(true));
    assert_eq!(s["active_connections"], json!(3));
    assert!(s.get("comment").is_none());
}

#[tokio::test]
async fn federation_peers_and_status_without_state_match_empty_defaults() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let p = handler.handle("federation.peers", json!({})).await.expect("peers");
    assert_eq!(p["peers"], json!([]));
    assert_eq!(p["total_count"], json!(0));
    assert_eq!(p["federation_enabled"], json!(false));

    let st = handler.handle("federation.status", json!({})).await.expect("status");
    assert_eq!(st["enabled"], json!(false));
    assert_eq!(st["active_connections"], json!(0));
}

#[tokio::test]
async fn federation_peers_and_status_reflect_federation_state() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let federation = Arc::new(FederationState::new("ipc-test".into()));
    let now = chrono::Utc::now();
    federation
        .register_node(NodeRegistration {
            node_id: "tower-1".into(),
            node_name: "Tower".into(),
            node_address: "127.0.0.1:1".into(),
            endpoints: None,
            cpu_cores: 0,
            memory_gb: 0,
            gpu_model: None,
            storage_gb: None,
            capabilities: vec![],
            status: NodeStatus::Active,
            joined_at: now,
            last_heartbeat: now,
        })
        .await;

    let handler =
        IpcServiceHandler::with_federation_state(registry.clone(), Arc::clone(&federation));

    let p = handler.handle("songbird.federation.peers", json!({})).await.expect("peers");
    assert_eq!(p["peers"], json!(["tower-1"]));
    assert_eq!(p["total_count"], json!(1));
    assert_eq!(p["federation_enabled"], json!(true));

    let st = handler.handle("songbird.federation.status", json!({})).await.expect("status");
    assert_eq!(st["enabled"], json!(true));
    assert_eq!(st["active_connections"], json!(1));
}
