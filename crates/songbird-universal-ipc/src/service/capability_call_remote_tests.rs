// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for cross-gate `capability.call` remote dispatch (`remote_dispatch.rs`).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::{CapabilityCallParams, IpcServiceHandler};
use crate::endpoint::NativeEndpoint;
use crate::registry::ServiceRegistry;
use crate::tower_atomic::JsonRpcHandler;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

fn ipc_handler() -> IpcServiceHandler {
    IpcServiceHandler::new(Arc::new(tokio::sync::RwLock::new(ServiceRegistry::new())))
}

fn endpoint_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/jsonrpc")
}

/// Mock remote Songbird JSON-RPC gate (line-delimited TCP).
struct MockRemoteGate {
    port: u16,
    connections: Arc<AtomicUsize>,
    _handle: JoinHandle<()>,
}

impl MockRemoteGate {
    async fn spawn(
        caps_list_response: Option<Value>,
        call_result: Value,
        captured_calls: Option<Arc<Mutex<Vec<Value>>>>,
    ) -> Self {
        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind mock remote gate");
        let port = listener.local_addr().expect("local addr").port();
        let connections = Arc::new(AtomicUsize::new(0));
        let conn_counter = Arc::clone(&connections);

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let caps = caps_list_response.clone();
                let result = call_result.clone();
                let capture = captured_calls.clone();
                let counter = Arc::clone(&conn_counter);

                tokio::spawn(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    let (reader, mut writer) = stream.into_split();
                    let mut buf_reader = BufReader::new(reader);
                    let mut line = String::new();
                    if buf_reader.read_line(&mut line).await.is_err() || line.is_empty() {
                        return;
                    }

                    let Ok(req) = serde_json::from_str::<Value>(line.trim()) else {
                        return;
                    };
                    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
                    let id = req.get("id").cloned().unwrap_or(json!(1));

                    let response = match method {
                        "capabilities.list" => {
                            if let Some(body) = caps {
                                json!({ "jsonrpc": "2.0", "result": body, "id": id })
                            } else {
                                json!({ "jsonrpc": "2.0", "error": { "code": -32000, "message": "no list" }, "id": id })
                            }
                        }
                        "capability.call" => {
                            if let Some(cap) = &capture {
                                cap.lock()
                                    .await
                                    .push(req.get("params").cloned().unwrap_or(Value::Null));
                            }
                            json!({
                                "jsonrpc": "2.0",
                                "result": {
                                    "provider": "remote-mock",
                                    "gate": "remote-mock",
                                    "result": result
                                },
                                "id": id
                            })
                        }
                        _ => json!({
                            "jsonrpc": "2.0",
                            "error": { "code": -32601, "message": format!("unknown: {method}") },
                            "id": id
                        }),
                    };

                    let mut bytes = serde_json::to_vec(&response).expect("serialize response");
                    bytes.push(b'\n');
                    let _ = writer.write_all(&bytes).await;
                });
            }
        });

        Self {
            port,
            connections,
            _handle: handle,
        }
    }

    fn addr(&self) -> SocketAddr {
        format!("127.0.0.1:{}", self.port).parse().expect("parse addr")
    }

    fn connection_count(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }
}

fn sample_call(capability: &str) -> CapabilityCallParams {
    CapabilityCallParams {
        capability: String::from(capability),
        operation: String::from("test.op"),
        params: json!({ "k": 1 }),
        routing: String::from("any"),
    }
}

// ── peer_has_capability ───────────────────────────────────────────────

#[tokio::test]
async fn peer_has_capability_matches_flat_capabilities_array() {
    let h = ipc_handler();
    let mock = MockRemoteGate::spawn(
        Some(json!({ "capabilities": ["echo", "storage"] })),
        json!({}),
        None,
    )
    .await;

    let has = h
        .peer_has_capability(&endpoint_url(mock.port), "echo")
        .await
        .expect("probe should succeed");
    assert!(has);
}

#[tokio::test]
async fn peer_has_capability_matches_provided_capabilities_strings() {
    let h = ipc_handler();
    let mock = MockRemoteGate::spawn(
        Some(json!({ "provided_capabilities": ["crypto", "compute"] })),
        json!({}),
        None,
    )
    .await;

    let has = h
        .peer_has_capability(&endpoint_url(mock.port), "crypto")
        .await
        .expect("probe should succeed");
    assert!(has);
}

#[tokio::test]
async fn peer_has_capability_matches_provided_capabilities_structured() {
    let h = ipc_handler();
    let mock = MockRemoteGate::spawn(
        Some(json!({
            "provided_capabilities": [
                { "type": "content", "version": "1" },
                "legacy-cap"
            ]
        })),
        json!({}),
        None,
    )
    .await;

    assert!(
        h.peer_has_capability(&endpoint_url(mock.port), "content")
            .await
            .expect("structured type match")
    );
    assert!(
        h.peer_has_capability(&endpoint_url(mock.port), "legacy-cap")
            .await
            .expect("structured string match")
    );
}

#[tokio::test]
async fn peer_has_capability_returns_false_when_peer_lacks_capability() {
    let h = ipc_handler();
    let mock = MockRemoteGate::spawn(
        Some(json!({ "provided_capabilities": ["other-only"] })),
        json!({}),
        None,
    )
    .await;

    let has = h
        .peer_has_capability(&endpoint_url(mock.port), "missing-cap")
        .await
        .expect("probe should succeed with false");
    assert!(!has);
}

#[tokio::test]
async fn peer_has_capability_probe_failure_returns_err() {
    let h = ipc_handler();
    let err = h
        .peer_has_capability("http://127.0.0.1:1/jsonrpc", "any-cap")
        .await
        .expect_err("unreachable port should fail probe");
    assert!(
        err.contains("probe connect") || err.contains("probe timeout"),
        "unexpected probe error: {err}"
    );
}

#[tokio::test]
async fn peer_has_capability_missing_capabilities_field_returns_err() {
    let h = ipc_handler();
    let mock = MockRemoteGate::spawn(Some(json!({ "note": "empty" })), json!({}), None).await;

    let err = h
        .peer_has_capability(&endpoint_url(mock.port), "cap")
        .await
        .expect_err("missing capabilities should error");
    assert!(err.contains("no provided_capabilities"), "unexpected: {err}");
}

// ── forward_to_remote_gate decision logic ─────────────────────────────

#[tokio::test]
async fn forward_to_remote_gate_errors_when_mesh_uninitialized() {
    let h = ipc_handler();
    let err = h
        .forward_to_remote_gate(&sample_call("remote-cap"))
        .await
        .expect_err("mesh not initialized");
    assert!(err.contains("Mesh not initialized"), "unexpected: {err}");
}

#[tokio::test]
async fn forward_to_remote_gate_errors_when_no_reachable_peers() {
    let h = ipc_handler();
    h.test_init_mesh_with_peers("solo-gate", &[]).await;

    let err = h.forward_to_remote_gate(&sample_call("remote-cap")).await.expect_err("no peers");
    assert!(err.contains("no reachable mesh peers"), "unexpected: {err}");
}

#[tokio::test]
async fn forward_to_remote_gate_skips_unreachable_mesh_peers() {
    let h = ipc_handler();
    let live = MockRemoteGate::spawn(
        Some(json!({ "provided_capabilities": ["target-cap"] })),
        json!({ "status": "ok" }),
        None,
    )
    .await;
    let dead_port: u16 = 9;
    h.test_init_mesh_with_peers(
        "gate-a",
        &[
            (
                String::from("dead-peer"),
                format!("127.0.0.1:{dead_port}").parse().expect("addr"),
                false,
            ),
            (String::from("live-peer"), live.addr(), true),
        ],
    )
    .await;

    let result = h
        .forward_to_remote_gate(&sample_call("target-cap"))
        .await
        .expect("reachable peer should serve capability");
    assert_eq!(result["gate"], "live-peer");
    assert_eq!(result["provider"], "remote:live-peer");
    assert!(live.connection_count() >= 2, "probe + call should hit live peer only");
}

#[tokio::test]
async fn forward_to_remote_gate_iterates_peers_and_prefers_tcp_match() {
    let h = ipc_handler();
    let skip = MockRemoteGate::spawn(
        Some(json!({ "provided_capabilities": ["other-cap"] })),
        json!({ "status": "wrong" }),
        None,
    )
    .await;
    let hit = MockRemoteGate::spawn(
        Some(json!({ "capabilities": ["target-cap"] })),
        json!({ "status": "hit" }),
        None,
    )
    .await;

    h.test_init_mesh_with_peers(
        "gate-b",
        &[
            (String::from("skip-peer"), skip.addr(), true),
            (String::from("hit-peer"), hit.addr(), true),
        ],
    )
    .await;

    let result = h.forward_to_remote_gate(&sample_call("target-cap")).await.expect("remote hit");
    assert_eq!(result["gate"], "hit-peer");
    assert_eq!(result["result"]["result"]["status"], "hit");
    assert!(skip.connection_count() <= 1, "skipped peer should not receive capability.call");
    assert!(hit.connection_count() >= 2, "selected peer should be probed and called");
}

#[tokio::test]
async fn forward_to_remote_gate_tcp_failure_without_turn_returns_error() {
    let h = ipc_handler();
    let dead: SocketAddr = "127.0.0.1:1".parse().expect("addr");
    h.test_init_mesh_with_peers("gate-c", &[(String::from("dead-peer"), dead, true)]).await;

    let err = h
        .forward_to_remote_gate(&sample_call("any-cap"))
        .await
        .expect_err("TCP + TURN should fail");
    assert!(
        err.contains("No local or remote provider") && err.contains("TURN relay"),
        "unexpected: {err}"
    );
    assert!(err.contains("tried 1 mesh peers"), "unexpected: {err}");
}

#[tokio::test]
async fn forward_to_remote_gate_tries_tcp_when_capability_probe_fails() {
    let h = ipc_handler();
    let mock =
        MockRemoteGate::spawn(None, json!({ "via": "probe-failed-still-called" }), None).await;

    h.test_init_mesh_with_peers("gate-d", &[(String::from("try-peer"), mock.addr(), true)]).await;

    let result = h
        .forward_to_remote_gate(&sample_call("target-cap"))
        .await
        .expect("probe failure should not block TCP attempt");
    assert_eq!(result["gate"], "try-peer");
    assert_eq!(result["result"]["result"]["via"], "probe-failed-still-called");
}

// ── remote dispatch error handling & request shape ────────────────────

#[tokio::test]
async fn forward_to_remote_tcp_adds_routing_local_to_prevent_loops() {
    let h = ipc_handler();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let mock = MockRemoteGate::spawn(
        Some(json!({ "capabilities": ["loop-cap"] })),
        json!({ "ok": true }),
        Some(Arc::clone(&captured)),
    )
    .await;

    let call = sample_call("loop-cap");
    h.forward_to_remote_tcp(&endpoint_url(mock.port), &call).await.expect("forward");

    let params = captured.lock().await;
    assert_eq!(params.len(), 1);
    assert_eq!(params[0]["routing"], "local");
    assert_eq!(params[0]["capability"], "loop-cap");
    assert_eq!(params[0]["operation"], "test.op");
}

#[tokio::test]
async fn capability_call_invalid_params_returns_deserialization_error() {
    let h = ipc_handler();
    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": 42,
                "operation": ["not", "a", "string"]
            }),
        )
        .await
        .expect_err("invalid params");
    assert!(err.contains("Invalid params"), "unexpected: {err}");
}

#[tokio::test]
async fn capability_call_provider_without_connectable_socket_errors() {
    let registry = Arc::new(tokio::sync::RwLock::new(ServiceRegistry::new()));
    registry
        .write()
        .await
        .register(
            "inprocess-only",
            NativeEndpoint::InProcess(42),
            vec![String::from("wasm-cap")],
            None,
            None,
        )
        .await
        .expect("register inprocess provider");

    let h = IpcServiceHandler::new(registry);
    let err = h
        .handle(
            "capability.call",
            json!({
                "capability": "wasm-cap",
                "operation": "run",
                "params": {},
                "routing": "local"
            }),
        )
        .await
        .expect_err("no connectable socket");
    assert!(
        err.contains("no connectable socket") && err.contains("inprocess-only"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn capability_call_end_to_end_remote_dispatch_via_mesh() {
    let h = ipc_handler();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let mock = MockRemoteGate::spawn(
        Some(json!({ "provided_capabilities": ["enroll"] })),
        json!({ "enrolled": true }),
        Some(Arc::clone(&captured)),
    )
    .await;

    h.test_init_mesh_with_peers("enroll-gate", &[(String::from("remote-gate"), mock.addr(), true)])
        .await;

    let result = h
        .handle(
            "capability.call",
            json!({
                "capability": "enroll",
                "operation": "enroll.submit",
                "params": { "node": "child-1" }
            }),
        )
        .await
        .expect("remote capability.call");

    assert_eq!(result["gate"], "remote-gate");
    assert_eq!(result["provider"], "remote:remote-gate");
    assert_eq!(result["result"]["result"]["enrolled"], true);

    let params = captured.lock().await;
    assert_eq!(params.len(), 1);
    assert_eq!(params[0]["routing"], "local");
}

// ── mesh.announce UDS routing parity ──────────────────────────────────

#[tokio::test]
async fn mesh_announce_routes_via_ipc_handler() {
    let h = ipc_handler();
    h.handle("mesh.init", json!({ "node_id": format!("announce-{}", Uuid::new_v4()) }))
        .await
        .expect("mesh.init");

    let result = h.handle("mesh.announce", json!({ "as_relay": true })).await;

    match result {
        Ok(response) => {
            assert!(response.is_object(), "response should be object");
            assert_eq!(response["announced"], true);
        }
        Err(err) => panic!("mesh.announce must route through dispatch, got: {err}"),
    }
}
