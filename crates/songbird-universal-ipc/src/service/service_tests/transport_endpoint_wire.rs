// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::registry::ServiceRegistry;
use crate::service::*;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

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
                "endpoint": "/tmp/songbird-test-isolated/beardog.sock"
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
        ep["path"], "/tmp/songbird-test-isolated/beardog.sock",
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
    assert_eq!(name_result["endpoint"]["path"], "/tmp/songbird-test-isolated/beardog.sock");

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
                "endpoint": "/tmp/songbird-test-isolated/beardog.sock"
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
    assert_eq!(ep["path"], "/tmp/songbird-test-isolated/beardog.sock");
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
                "endpoint": "/tmp/songbird-test-isolated/beardog.sock"
            }),
        )
        .await
        .unwrap();

    let t = &uds_reg["transport"];
    assert_eq!(t["transport"], "uds");
    assert_eq!(t["path"], "/tmp/songbird-test-isolated/beardog.sock");

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
