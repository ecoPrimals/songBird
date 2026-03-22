// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use serde_json::json;

#[tokio::test]
async fn test_ipc_service_register() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    let params = json!({
        "primal_id": "beardog",
        "capabilities": ["crypto", "btsp"],
        "endpoint": "/tmp/primal-beardog.sock"
    });

    let result = handler.handle("ipc.register", params).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
}

#[tokio::test]
async fn test_ipc_service_resolve() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    // Register first
    let register_params = json!({
        "primal_id": "beardog",
        "capabilities": ["crypto"],
        "endpoint": "/tmp/primal-beardog.sock"
    });
    handler.handle("ipc.register", register_params).await.unwrap();

    // Then resolve
    let resolve_params = json!({
        "primal_id": "beardog"
    });

    let result = handler.handle("ipc.resolve", resolve_params).await;
    assert!(result.is_ok());

    let result_value = result.unwrap();
    assert_eq!(result_value["virtual_endpoint"], "/primal/beardog");
    assert!(result_value["native_endpoint"].as_str().unwrap().contains("beardog"));
}

#[tokio::test]
async fn test_ipc_service_discover() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());

    // Register service with capability
    let register_params = json!({
        "primal_id": "beardog",
        "capabilities": ["crypto", "btsp"],
        "endpoint": "/tmp/primal-beardog.sock"
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
    assert_eq!(providers[0]["primal_id"], "beardog");
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
    for (id, caps) in &[("beardog", vec!["crypto"]), ("squirrel", vec!["ai"])] {
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
        "primal_id": "beardog",
        "capabilities": ["crypto"],
        "endpoint": "/tmp/x.sock"
    });
    let p: RegisterParams = serde_json::from_value(v).expect("RegisterParams");
    assert_eq!(p.primal_id, "beardog");
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
    assert!(err.contains("Unknown method"));
}

#[tokio::test]
async fn ipc_register_tcp_localhost_parses_port() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let params = json!({
        "primal_id": "tcp-primal",
        "capabilities": ["x"],
        "endpoint": "127.0.0.1:9555"
    });
    let v = handler.handle("ipc.register", params).await.expect("register");
    assert_eq!(v["virtual_endpoint"], "/primal/tcp-primal");
}

#[tokio::test]
async fn ipc_register_rejects_invalid_endpoint_format() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let params = json!({
        "primal_id": "bad",
        "capabilities": [],
        "endpoint": "not-a-socket-or-tcp"
    });
    let err = handler.handle("ipc.register", params).await.expect_err("bad endpoint");
    assert!(err.contains("Invalid endpoint") || err.contains("endpoint"));
}

#[tokio::test]
async fn ipc_register_rejects_bad_tcp_port() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let params = json!({
        "primal_id": "bad",
        "capabilities": [],
        "endpoint": "127.0.0.1:notaport"
    });
    let err = handler.handle("ipc.register", params).await.expect_err("bad port");
    assert!(err.contains("Invalid TCP port") || err.contains("port"));
}

#[tokio::test]
async fn ipc_register_invalid_json_params() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let err = handler.handle("ipc.register", json!("not-an-object")).await.expect_err("params");
    assert!(err.contains("Invalid params"));
}

#[tokio::test]
async fn http_get_missing_url_errors() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let err = handler.handle("http.get", json!({})).await.expect_err("url");
    assert!(err.contains("Missing 'url'"));
}

#[tokio::test]
async fn http_post_missing_body_errors() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let err = handler
        .handle("http.post", json!({ "url": "http://127.0.0.1:9/" }))
        .await
        .expect_err("body");
    assert!(err.contains("body"));
}

#[tokio::test]
async fn health_returns_service_count() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "h1",
                "capabilities": [],
                "endpoint": "/tmp/h1.sock"
            }),
        )
        .await
        .expect("reg");
    let v = handler.handle("health", json!({})).await.expect("health");
    assert_eq!(v["status"], "healthy");
    assert_eq!(v["services"], 1);
}

#[tokio::test]
async fn health_liveness_returns_minimal_payload() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let v = handler.handle("health.liveness", json!({})).await.expect("live");
    assert_eq!(v, json!({ "status": "healthy" }));
}

#[tokio::test]
async fn identity_uses_injected_family_id_env() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::with_family_id_env(registry, |_| Ok("family-xyz".into()));
    let v = handler.handle("identity", json!({})).await.expect("id");
    assert_eq!(v["family_id"], "family-xyz");
}

#[tokio::test]
async fn capabilities_list_returns_string_array() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    let a = handler.handle("capabilities.list", json!({})).await.expect("caps");
    let arr = a.as_array().expect("array");
    let strings: Vec<&str> = arr.iter().filter_map(|x| x.as_str()).collect();
    assert!(strings.contains(&"network.discovery"));
    assert!(strings.contains(&"ipc.jsonrpc"));
    assert_eq!(strings.len(), crate::introspection::SONGBIRD_CAPABILITY_STRINGS.len());
}

#[tokio::test]
async fn ipc_discover_empty_when_no_match() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new(registry.clone());
    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "only-stun",
                "capabilities": ["stun"],
                "endpoint": "/tmp/o.sock"
            }),
        )
        .await
        .expect("reg");
    let v = handler
        .handle("ipc.discover", json!({ "capability": "nonexistent-cap" }))
        .await
        .expect("disc");
    assert_eq!(v["providers"].as_array().map(Vec::len), Some(0));
}
