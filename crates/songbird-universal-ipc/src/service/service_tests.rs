// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]
#![allow(clippy::unchecked_time_subtraction, reason = "test time arithmetic")]

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

/// Validates the full `ipc.resolve` → `TransportEndpoint` wire format
/// that sourDough's `IpcClient` consumes. This is the Phase 2 M1 gate test.
///
/// Wire contract:
/// - `endpoint.transport` discriminant tag (internally tagged enum via `#[serde(tag = "transport")]`)
/// - UDS variant: `{ "transport": "uds", "path": "..." }`
/// - TCP variant: `{ "transport": "tcp", "host": "...", "port": N }`
/// - MeshRelay variant: `{ "transport": "mesh_relay", "peer_id": "...", "capability": "..." }`
///
/// sourDough callers use `"native": true` to force the direct endpoint (no relay).
/// Default mode (`"virtual": true`) returns `mesh_relay` when relay is available.
#[tokio::test]
async fn ipc_resolve_returns_transport_endpoint_json_sourdough_wire_compat() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    // Register a UDS-based primal
    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto.sign", "crypto.verify", "security"],
                "endpoint": "/run/user/1000/biomeos/beardog.sock"
            }),
        )
        .await
        .unwrap();

    // Register a TCP-based primal
    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "skunkbat",
                "capabilities": ["observability", "metrics"],
                "endpoint": "tcp://127.0.0.1:9090"
            }),
        )
        .await
        .unwrap();

    // --- Resolve UDS primal by capability with native=true (bypass relay) ---
    let uds_result = handler
        .handle("ipc.resolve", json!({ "capability": "crypto.sign", "native": true }))
        .await
        .expect("resolve UDS primal by capability (native)");

    let ep = &uds_result["endpoint"];
    assert_eq!(ep["transport"], "uds", "UDS endpoint must have transport='uds'");
    assert_eq!(
        ep["path"], "/run/user/1000/biomeos/beardog.sock",
        "UDS path must match registered socket"
    );
    assert!(ep.get("host").is_none(), "UDS variant must not have 'host'");
    assert!(ep.get("port").is_none(), "UDS variant must not have 'port'");

    // Validate envelope fields that sourDough also reads
    assert_eq!(uds_result["relay"], false);
    assert!(uds_result["capabilities"].as_array().unwrap().iter().any(|c| c == "crypto.sign"));
    assert!(uds_result["native_endpoint"].as_str().unwrap().contains("beardog.sock"));

    // --- Resolve TCP primal by primal_id with native=true ---
    let tcp_result = handler
        .handle("ipc.resolve", json!({ "primal_id": "skunkbat", "native": true }))
        .await
        .expect("resolve TCP primal by primal_id (native)");

    let ep = &tcp_result["endpoint"];
    assert_eq!(ep["transport"], "tcp", "TCP endpoint must have transport='tcp'");
    assert_eq!(ep["host"], "127.0.0.1");
    assert_eq!(ep["port"], 9090);
    assert!(ep.get("path").is_none(), "TCP variant must not have 'path'");
    assert!(ep.get("peer_id").is_none(), "TCP variant must not have 'peer_id'");

    // --- Resolve with default virtual mode (relay) ---
    let relay_result = handler
        .handle("ipc.resolve", json!({ "capability": "crypto.sign" }))
        .await
        .expect("resolve with relay (default virtual mode)");

    let ep = &relay_result["endpoint"];
    // When relay is available, endpoint becomes mesh_relay
    assert_eq!(
        ep["transport"], "mesh_relay",
        "Virtual mode should return mesh_relay when relay is active"
    );
    assert_eq!(ep["peer_id"], "beardog");
    assert_eq!(ep["capability"], "crypto.sign");
    assert_eq!(relay_result["relay"], true);
    // native_endpoint is always present regardless of relay
    assert!(relay_result["native_endpoint"].as_str().unwrap().contains("beardog.sock"));

    // --- Resolve by `name` alias with native ---
    let name_result = handler
        .handle("ipc.resolve", json!({ "name": "beardog", "native": true }))
        .await
        .expect("resolve by name alias");
    assert_eq!(name_result["endpoint"]["transport"], "uds");
    assert_eq!(name_result["endpoint"]["path"], "/run/user/1000/biomeos/beardog.sock");

    // --- Verify `ipc.resolve_by_name` method alias ---
    let alias_result = handler
        .handle("ipc.resolve_by_name", json!({ "name": "skunkbat", "native": true }))
        .await
        .expect("ipc.resolve_by_name method alias");
    assert_eq!(alias_result["endpoint"]["transport"], "tcp");
    assert_eq!(alias_result["endpoint"]["port"], 9090);
}

/// Validates `capability.resolve` also returns `TransportEndpoint` JSON.
/// sourDough's `IpcClient` can use either `ipc.resolve` or `capability.resolve`.
#[tokio::test]
async fn capability_resolve_returns_transport_endpoint_json() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto.sign"],
                "endpoint": "/run/user/1000/biomeos/beardog.sock"
            }),
        )
        .await
        .unwrap();

    let result = handler
        .handle("capability.resolve", json!({ "capability": "crypto.sign" }))
        .await
        .expect("capability.resolve");

    let ep = &result["endpoint"];
    assert_eq!(ep["transport"], "uds");
    assert_eq!(ep["path"], "/run/user/1000/biomeos/beardog.sock");
    assert_eq!(result["primal_id"], "beardog");
}

/// Validates that `ipc.register` returns `TransportEndpoint` in its response
/// so primals get immediate confirmation of their registered transport.
#[tokio::test]
async fn ipc_register_returns_transport_endpoint_in_response() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    // UDS registration
    let uds_reg = handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "beardog",
                "capabilities": ["crypto"],
                "endpoint": "/run/user/1000/biomeos/beardog.sock"
            }),
        )
        .await
        .unwrap();

    let t = &uds_reg["transport"];
    assert_eq!(t["transport"], "uds");
    assert_eq!(t["path"], "/run/user/1000/biomeos/beardog.sock");

    // TCP registration
    let tcp_reg = handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "skunkbat",
                "capabilities": ["observability"],
                "endpoint": "tcp://127.0.0.1:9090"
            }),
        )
        .await
        .unwrap();

    let t = &tcp_reg["transport"];
    assert_eq!(t["transport"], "tcp");
    assert_eq!(t["host"], "127.0.0.1");
    assert_eq!(t["port"], 9090);
}

/// When a capability isn't registered locally but a mesh peer advertises it,
/// `ipc.resolve` should return a `MeshRelay` transport endpoint pointing to
/// the remote peer — enabling transparent cross-gate routing (Wave 107 M1).
#[tokio::test]
async fn ipc_resolve_falls_back_to_mesh_peer_when_local_absent() {
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    // Inject a remote peer's capabilities into the mesh handler
    {
        let mesh_handler = handler.mesh_handler();
        let mut caps = mesh_handler.peer_capabilities.write().await;
        caps.insert(
            "iron-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec![
                    "linalg".to_string(),
                    "linalg.svd".to_string(),
                    "compute".to_string(),
                ],
                last_seen: Instant::now(),
            },
        );
    }

    // Resolve a capability that only exists on the remote peer
    let result = handler
        .handle("ipc.resolve", json!({ "capability": "linalg.svd" }))
        .await
        .expect("should resolve via mesh fallback");

    let ep = &result["endpoint"];
    assert_eq!(ep["transport"], "mesh_relay", "must return mesh_relay transport");
    assert_eq!(ep["peer_id"], "iron-gate", "must point to the peer with the capability");
    assert_eq!(ep["capability"], "linalg.svd");
    assert_eq!(result["relay"], true, "relay flag must be true for mesh resolution");
    assert_eq!(result["native_endpoint"], "mesh://iron-gate");
    assert!(
        result["capabilities"].as_array().unwrap().iter().any(|c| c == "linalg.svd"),
        "capabilities must include the resolved capability"
    );
}

/// `capability.resolve` also falls back to mesh peers when no local provider exists.
#[tokio::test]
async fn capability_resolve_falls_back_to_mesh_peer() {
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    {
        let mesh_handler = handler.mesh_handler();
        let mut caps = mesh_handler.peer_capabilities.write().await;
        caps.insert(
            "south-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["crypto.sign".to_string(), "security".to_string()],
                last_seen: Instant::now(),
            },
        );
    }

    let result = handler
        .handle("capability.resolve", json!({ "capability": "crypto.sign" }))
        .await
        .expect("should resolve via mesh fallback");

    let ep = &result["endpoint"];
    assert_eq!(ep["transport"], "mesh_relay");
    assert_eq!(ep["peer_id"], "south-gate");
    assert_eq!(ep["capability"], "crypto.sign");
    assert_eq!(result["primal_id"], "remote:south-gate");
}

/// When a capability exists locally, mesh fallback is NOT used (local always wins).
#[tokio::test]
async fn ipc_resolve_prefers_local_over_mesh_peer() {
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = IpcServiceHandler::new_isolated(registry.clone());

    // Register locally with a path that won't collide with any running process
    let test_sock = format!("/tmp/songbird-test-{}/local-primal.sock", std::process::id());
    handler
        .handle(
            "ipc.register",
            json!({
                "primal_id": "local-primal",
                "capabilities": ["security"],
                "endpoint": test_sock
            }),
        )
        .await
        .unwrap();

    // Also inject the same capability as available on a remote peer
    {
        let mesh_handler = handler.mesh_handler();
        let mut caps = mesh_handler.peer_capabilities.write().await;
        caps.insert(
            "remote-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["security".to_string()],
                last_seen: Instant::now(),
            },
        );
    }

    let result = handler
        .handle("ipc.resolve", json!({ "capability": "security", "native": true }))
        .await
        .expect("should resolve locally");

    let ep = &result["endpoint"];
    assert_eq!(ep["transport"], "uds", "local provider must win over mesh peer");
    assert!(
        ep["path"].as_str().unwrap().contains("local-primal.sock"),
        "must point to local socket"
    );
}

/// `find_peer_with_capability` ignores expired entries.
#[tokio::test]
async fn find_peer_with_capability_ignores_expired() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::{Duration, Instant};

    let handler = MeshHandler::new();

    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            "stale-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["stale-cap".to_string()],
                last_seen: Instant::now() - Duration::from_secs(700),
            },
        );
        caps.insert(
            "fresh-gate".to_string(),
            PeerCapabilityEntry {
                capabilities: vec!["fresh-cap".to_string()],
                last_seen: Instant::now(),
            },
        );
    }

    assert!(handler.find_peer_with_capability("stale-cap").await.is_none());

    let found = handler.find_peer_with_capability("fresh-cap").await;
    assert!(found.is_some());
    let (peer, caps) = found.unwrap();
    assert_eq!(peer, "fresh-gate");
    assert!(caps.contains(&"fresh-cap".to_string()));
}

/// `find_peer_with_capability` selects the peer with the lowest-cost path when
/// multiple peers provide the same capability (overlay preferred over direct).
#[tokio::test]
async fn find_peer_with_capability_prefers_lower_cost_path() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use songbird_onion_relay::mesh::{BeaconMesh, EndpointType, RelayEndpoint};
    use std::time::{Duration, Instant};

    let handler = MeshHandler::new();

    // Initialize mesh with two peers
    let mesh = BeaconMesh::new(String::from("local-gate"), vec![]);

    // gate-a: direct (priority 1), high latency
    mesh.record_direct_connection(
        String::from("gate-a"),
        "203.0.113.1:7700".parse().unwrap(),
        Duration::from_millis(100),
    )
    .await;

    // gate-b: overlay (priority 0), low latency
    mesh.record_overlay_connection(
        String::from("gate-b"),
        "10.13.37.5:7700".parse().unwrap(),
        "wireguard",
        Duration::from_millis(2),
    )
    .await;

    *handler.mesh.write().await = Some(std::sync::Arc::new(mesh));

    // Both peers advertise the same capability
    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            String::from("gate-a"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("shared-cap")],
                last_seen: Instant::now(),
            },
        );
        caps.insert(
            String::from("gate-b"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("shared-cap")],
                last_seen: Instant::now(),
            },
        );
    }

    // Should prefer gate-b (overlay, lower cost)
    let found = handler.find_peer_with_capability("shared-cap").await;
    assert!(found.is_some());
    let (peer, _) = found.unwrap();
    assert_eq!(peer, "gate-b", "should prefer overlay peer with lower cost");
}

/// `find_peer_with_capability` returns the single holder even without mesh initialized.
#[tokio::test]
async fn find_peer_with_capability_single_holder_no_mesh() {
    use crate::handlers::mesh_handler::MeshHandler;
    use crate::handlers::mesh_handler::capability_propagation::PeerCapabilityEntry;
    use std::time::Instant;

    let handler = MeshHandler::new();

    {
        let mut caps = handler.peer_capabilities.write().await;
        caps.insert(
            String::from("only-gate"),
            PeerCapabilityEntry {
                capabilities: vec![String::from("unique-cap")],
                last_seen: Instant::now(),
            },
        );
    }

    let found = handler.find_peer_with_capability("unique-cap").await;
    assert!(found.is_some());
    let (peer, _) = found.unwrap();
    assert_eq!(peer, "only-gate");
}
