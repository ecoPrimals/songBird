// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Local CI integration tests: E2E-style flows, chaos, and fault injection **without**
//! external security providers or real network I/O (see `tests/README.md`).
//!
//! - Binds ephemeral TCP with `127.0.0.1:0` only.
//! - Uses production [`songbird_universal_ipc::service::IpcServiceHandler`] and
//!   [`songbird_config::capability_discovery::CapabilityDiscovery`] code paths.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: failures should surface as panics with clear stack traces"
)]

use futures_util::future::join_all;
use serde_json::{Value, json};
use songbird_config::capability_discovery::{
    CapabilityDiscovery, DiscoveryMethod, ServiceEndpoint,
};
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::{
    JsonRpcError, JsonRpcHandler, JsonRpcRequest, JsonRpcResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

/// Line-delimited JSON-RPC over TCP (same framing as orchestrator TCP IPC fallback).
mod jsonrpc_line_tcp {
    use super::*;

    pub async fn spawn_handler_server(
        handler: Arc<IpcServiceHandler>,
    ) -> (tokio::task::JoinHandle<()>, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind 127.0.0.1:0");
        let addr = listener.local_addr().expect("local_addr");

        let join = tokio::spawn(async move {
            loop {
                let accepted = listener.accept().await;
                let Ok((stream, _)) = accepted else {
                    break;
                };
                let handler = Arc::clone(&handler);
                tokio::spawn(async move {
                    handle_one_connection(stream, handler).await;
                });
            }
        });

        (join, addr)
    }

    async fn handle_one_connection(stream: TcpStream, handler: Arc<IpcServiceHandler>) {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            let id = request.id.clone().unwrap_or(Value::Null);
                            if request.jsonrpc == "2.0" {
                                let params = request.params.unwrap_or(Value::Null);
                                match handler.handle(&request.method, params).await {
                                    Ok(result) => JsonRpcResponse::success(result, id),
                                    Err(message) => JsonRpcResponse::error(
                                        JsonRpcError::internal_error(message),
                                        id,
                                    ),
                                }
                            } else {
                                JsonRpcResponse::error(
                                    JsonRpcError {
                                        code: JsonRpcError::INVALID_REQUEST,
                                        message: "Invalid JSON-RPC version (must be 2.0)"
                                            .to_string(),
                                        data: None,
                                    },
                                    id,
                                )
                            }
                        }
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError {
                                code: JsonRpcError::PARSE_ERROR,
                                message: format!("Failed to parse JSON-RPC request: {e}"),
                                data: None,
                            },
                            Value::Null,
                        ),
                    };

                    let mut payload = serde_json::to_vec(&response).expect("serialize response");
                    payload.push(b'\n');
                    if write_half.write_all(&payload).await.is_err() {
                        break;
                    }
                    if write_half.flush().await.is_err() {
                        break;
                    }
                    // Match orchestrator TCP IPC: one request per connection.
                    break;
                }
            }
        }
    }

    pub async fn call(
        addr: SocketAddr,
        method: &str,
        params: Value,
    ) -> Result<JsonRpcResponse, String> {
        let mut stream =
            TcpStream::connect(addr).await.map_err(|e| format!("connect {addr}: {e}"))?;

        let request = JsonRpcRequest::new(method, Some(params), 1);
        let mut payload = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        payload.push(b'\n');
        stream.write_all(&payload).await.map_err(|e| e.to_string())?;
        stream.flush().await.map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.map_err(|e| format!("read_line: {e}"))?;

        serde_json::from_str::<JsonRpcResponse>(&line).map_err(|e| format!("parse response: {e}"))
    }

    pub async fn send_raw_line(addr: SocketAddr, raw: &[u8]) -> Result<String, String> {
        let mut stream = TcpStream::connect(addr).await.map_err(|e| format!("connect: {e}"))?;
        stream.write_all(raw).await.map_err(|e| e.to_string())?;
        if !raw.ends_with(b"\n") {
            stream.write_all(b"\n").await.map_err(|e| e.to_string())?;
        }
        stream.flush().await.map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.map_err(|e| format!("read: {e}"))?;
        Ok(line)
    }
}

fn ipc_handler() -> Arc<IpcServiceHandler> {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    Arc::new(IpcServiceHandler::new(registry))
}

// --- E2E (local, no external deps) -------------------------------------------------

#[tokio::test]
async fn e2e_local_capability_discovery_via_environment_only() {
    let discovery =
        CapabilityDiscovery::with_methods_env_reader(vec![DiscoveryMethod::Environment], |key| {
            if key == "STORAGE_ENDPOINT" {
                Ok("http://127.0.0.1:9".to_string())
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });

    let providers: Vec<ServiceEndpoint> = discovery
        .find_providers_by_capability("storage")
        .await
        .expect("environment discovery should find injected provider");

    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].id, "storage-provider-env");
    assert!(providers[0].url.contains("127.0.0.1"));
    assert!(providers[0].capabilities.contains(&"storage".to_string()));
}

#[tokio::test]
async fn e2e_jsonrpc_health_triad_over_local_tcp() {
    let handler = ipc_handler();
    let (server, addr) = jsonrpc_line_tcp::spawn_handler_server(Arc::clone(&handler)).await;

    for method in ["health.liveness", "health.readiness", "health.check"] {
        let resp = jsonrpc_line_tcp::call(addr, method, json!({}))
            .await
            .unwrap_or_else(|e| panic!("{method}: {e}"));
        assert!(resp.error.is_none(), "{method}: {:?}", resp.error);
        let result = resp.result.expect("result");
        assert!(
            result.get("status").and_then(|v| v.as_str()).is_some(),
            "{method}: expected status string, got {result}"
        );
    }

    server.abort();
    let _ = server.await;
}

#[tokio::test]
async fn e2e_service_registration_and_discovery_lifecycle() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(Arc::clone(&registry)));

    let reg = json!({
        "primal_id": "e2e-svc-alpha",
        "capabilities": ["e2e.test.cap"],
        "endpoint": "/tmp/e2e-svc-alpha.sock"
    });
    let r1 = handler.handle("ipc.register", reg).await.expect("register");
    assert!(r1.get("virtual_endpoint").is_some());

    let disc = json!({ "capability": "e2e.test.cap" });
    let r2 = handler.handle("ipc.discover", disc).await.expect("discover");
    let providers = r2["providers"].as_array().expect("providers array");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["primal_id"], "e2e-svc-alpha");

    let r3 = handler.handle("ipc.list", json!({})).await.expect("list");
    let services = r3["services"].as_array().expect("services");
    assert_eq!(services.len(), 1);
}

// --- Chaos -----------------------------------------------------------------------

#[tokio::test]
async fn chaos_connection_drop_after_successful_jsonrpc_tcp_call() {
    let handler = ipc_handler();
    let (server, addr) = jsonrpc_line_tcp::spawn_handler_server(Arc::clone(&handler)).await;

    jsonrpc_line_tcp::call(addr, "health.liveness", json!({}))
        .await
        .expect("first call should succeed while server is up");

    server.abort();
    let _ = server.await;

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let err = jsonrpc_line_tcp::call(addr, "health.liveness", json!({}))
        .await
        .expect_err("expected connection failure after server dropped");
    assert!(
        err.contains("connect") || err.contains("Connection refused") || err.contains("refused"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn chaos_concurrent_ipc_registration_stress_100() {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    let handler = Arc::new(IpcServiceHandler::new(Arc::clone(&registry)));

    let tasks: Vec<_> = (0_u32..100)
        .map(|i| {
            let h = Arc::clone(&handler);
            tokio::spawn(async move {
                let params = json!({
                    "primal_id": format!("stress-{i}"),
                    "capabilities": ["stress.test"],
                    "endpoint": format!("/tmp/stress-{i}.sock")
                });
                h.handle("ipc.register", params).await
            })
        })
        .collect();

    let results = join_all(tasks).await;
    for res in results {
        let inner = res.expect("join");
        assert!(inner.is_ok(), "{inner:?}");
    }

    let listed = handler.handle("ipc.list", json!({})).await.expect("list");
    let n = listed["services"].as_array().expect("services").len();
    assert_eq!(n, 100);
}

// --- Fault injection -------------------------------------------------------------

#[tokio::test]
async fn fault_malformed_jsonrpc_line_returns_parse_error_not_panic() {
    let handler = ipc_handler();
    let (server, addr) = jsonrpc_line_tcp::spawn_handler_server(Arc::clone(&handler)).await;

    let line = jsonrpc_line_tcp::send_raw_line(addr, b"{{{not-json-at-all")
        .await
        .expect("server should still accept TCP and return a line");

    let resp: JsonRpcResponse =
        serde_json::from_str(line.trim()).expect("response must be valid JSON");
    assert!(resp.result.is_none());
    let err = resp.error.expect("error object");
    assert_eq!(err.code, JsonRpcError::PARSE_ERROR);

    server.abort();
    let _ = server.await;
}
