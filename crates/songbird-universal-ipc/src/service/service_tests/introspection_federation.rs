// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::registry::ServiceRegistry;
use crate::service::*;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::json;
use songbird_network_federation::state::{FederationState, NodeRegistration, NodeStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_primal_info_introspection() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
async fn health_liveness_returns_healthy_status_only() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let v = handler.handle("health.liveness", json!({})).await.expect("liveness");
    assert_eq!(v, json!({ "status": "alive" }));
}

#[tokio::test]
async fn capabilities_list_returns_wire_standard_envelope() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let v = handler.handle("capabilities.list", json!({})).await.expect("caps");
    assert_eq!(v["primal"].as_str().unwrap(), "songbird");
    assert!(v["version"].as_str().is_some(), "version must be present");
    let methods = v["methods"].as_array().expect("methods must be a JSON array");
    assert!(methods.iter().any(|m| m == "health.liveness"), "must include health.liveness");
    assert!(methods.iter().any(|m| m == "identity.get"), "must include identity.get");
}

#[tokio::test]
async fn identity_get_returns_wire_standard_response() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let v = handler.handle("identity.get", json!({})).await.expect("identity.get");
    assert_eq!(v["primal"].as_str().unwrap(), "songbird");
    assert_eq!(v["domain"].as_str().unwrap(), "network");
    assert_eq!(v["license"].as_str().unwrap(), "AGPL-3.0-or-later");
    let methods = v["methods"].as_array().expect("methods must be present");
    assert!(!methods.is_empty());
    assert!(methods.iter().any(|m| m == "identity.get"));
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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
