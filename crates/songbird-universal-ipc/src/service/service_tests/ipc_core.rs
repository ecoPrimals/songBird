// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::registry::ServiceRegistry;
use crate::service::*;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_ipc_service_register() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
async fn test_ipc_service_list() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    let handler = IpcServiceHandler::new_isolated(registry.clone());

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
    assert_eq!(r.primal_id.as_deref(), Some("a"));

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
            socket: Some("/run/q".into()),
            virtual_endpoint: "/primal/q".into(),
            native_endpoint: "unix:///run/q".into(),
            capabilities: vec![],
            signature: None,
            signed_payload: None,
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
        transport: None,
        signature: None,
        signed_payload: None,
    };
    let v = serde_json::to_value(&reg).expect("RegisterResult json");
    assert_eq!(v["virtual_endpoint"], "/primal/x");

    let res = ResolveResult {
        socket: None,
        virtual_endpoint: "/primal/x".into(),
        native_endpoint: "native".into(),
        endpoint: TransportEndpoint::Uds {
            path: "/tmp/x.sock".into(),
        },
        capabilities: vec!["c".into()],
        relay: false,
        relay_socket: None,
        signature: None,
        signed_payload: None,
    };
    let v2 = serde_json::to_value(&res).expect("ResolveResult json");
    assert_eq!(v2["capabilities"][0], "c");
    assert_eq!(v2["endpoint"]["transport"], "uds");
}

#[tokio::test]
async fn ipc_resolve_errors_when_primal_not_registered() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let err = handler
        .handle("ipc.resolve", json!({ "primal_id": "ghost" }))
        .await
        .expect_err("not registered");
    assert!(err.contains("not found") || err.contains("Not found") || err.contains("found"));
}

#[tokio::test]
async fn unknown_rpc_method_returns_error() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());
    let err = handler.handle("no.such.method", json!({})).await.expect_err("unknown method");
    assert!(
        err.contains("unknown JSON-RPC method") || err.contains("Unknown method"),
        "unexpected error: {err}"
    );
}
