// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate
)]

//! E2E Unix Socket Validation Tests
//!
//! End-to-end tests that validate actual Unix socket behavior for upstream issues.
//! These tests spin up a real Unix socket server and test with actual connections.

use serde_json::{Value, json};
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{RwLock, oneshot};
use tokio::time::{Duration, timeout};

// Global mutex to serialize environment variable tests
static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Helper: Start a test Unix socket server with readiness signaling.
///
/// Returns a join handle and a oneshot receiver that resolves when the
/// server has bound its socket and is ready to accept connections.
async fn start_test_server(
    socket_path: &str,
) -> (tokio::task::JoinHandle<()>, oneshot::Receiver<()>) {
    let socket_path = socket_path.to_string();
    let (ready_tx, ready_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        // Remove existing socket
        let _ = std::fs::remove_file(&socket_path);

        // Create listener
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");

        // Signal readiness — socket is bound
        let _ = ready_tx.send(());

        // Create handler
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        let handler = IpcServiceHandler::new(registry);

        // Accept ONE connection for testing
        if let Ok((stream, _)) = listener.accept().await {
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            // Handle multiple requests on persistent connection
            loop {
                line.clear();

                match reader.read_line(&mut line).await {
                    Ok(0) => break, // Client closed
                    Ok(_) => {
                        // Parse request
                        if let Ok(request) = serde_json::from_str::<Value>(&line) {
                            let method = request["method"].as_str().unwrap_or("");
                            let params = request.get("params").cloned().unwrap_or(json!({}));
                            let id = request["id"].clone();

                            // Handle request
                            let result = handler.handle(method, params).await;

                            // Build response
                            let response = match result {
                                Ok(res) => json!({
                                    "jsonrpc": "2.0",
                                    "result": res,
                                    "id": id
                                }),
                                Err(err) => json!({
                                    "jsonrpc": "2.0",
                                    "error": {"code": -32603, "message": err},
                                    "id": id
                                }),
                            };

                            // Send response
                            let response_str = serde_json::to_string(&response).unwrap();
                            let _ = write_half.write_all(response_str.as_bytes()).await;
                            let _ = write_half.write_all(b"\n").await;
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        // Cleanup
        let _ = std::fs::remove_file(&socket_path);
    });

    (handle, ready_rx)
}

// ============================================================================
// E2E TESTS - Issue 1: Standard Methods via Unix Socket
// ============================================================================

#[tokio::test]
async fn test_e2e_health_via_unix_socket() {
    let socket_path = "/tmp/songbird-test-health.sock";

    // Start server and wait for readiness signal
    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    // Connect client
    let mut stream = timeout(Duration::from_secs(5), UnixStream::connect(socket_path))
        .await
        .expect("Timeout connecting")
        .expect("Failed to connect");

    // Send health request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "health",
        "params": {},
        "id": 1
    });

    stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();

    // Read response (with timeout to handle persistent connection)
    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();

    timeout(Duration::from_secs(1), reader.read_line(&mut response))
        .await
        .expect("Timeout reading response")
        .expect("Failed to read response");

    // Parse and verify
    let response: Value = serde_json::from_str(&response).expect("Invalid JSON");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "healthy");
    assert_eq!(response["result"]["primal"], "songbird");
    assert!(response["result"]["uptime_seconds"].is_number());
    assert_eq!(response["id"], 1);

    // Close connection
    drop(stream);

    // Cleanup
    server_handle.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_identity_via_unix_socket() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let socket_path = "/tmp/songbird-test-identity.sock";

    // Clean slate
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("NODE_FAMILY_ID");

    // Set test environment variable
    songbird_process_env::set_var("FAMILY_ID", "test_e2e_family");

    // Start server and wait for readiness signal
    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    // Connect and request
    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    let request = json!({
        "jsonrpc": "2.0",
        "method": "identity",
        "params": {},
        "id": 2
    });

    stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();

    // Read response
    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    timeout(Duration::from_secs(1), reader.read_line(&mut response)).await.unwrap().unwrap();

    let response: Value = serde_json::from_str(&response).unwrap();

    assert_eq!(response["result"]["primal"], "songbird");
    assert_eq!(response["result"]["family_id"], "test_e2e_family");
    assert!(response["result"]["capabilities"].is_array());

    // Cleanup
    drop(stream);
    server_handle.abort();
    songbird_process_env::remove_var("FAMILY_ID");
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_persistent_connection_multiple_requests() {
    let socket_path = "/tmp/songbird-test-persistent.sock";

    // Start server and wait for readiness signal
    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    // Connect once
    let mut stream = UnixStream::connect(socket_path).await.expect("Failed to connect");

    // Send 3 requests on same connection
    for i in 1..=3 {
        let method = match i {
            1 => "health",
            2 => "identity",
            _ => "rpc.discover",
        };

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": i
        });

        stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
        stream.write_all(b"\n").await.unwrap();

        // Read response
        let mut reader = BufReader::new(&mut stream);
        let mut response = String::new();
        timeout(Duration::from_secs(1), reader.read_line(&mut response)).await.unwrap().unwrap();

        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["id"], i);
        assert!(response["result"].is_object());
    }

    // Cleanup
    drop(stream);
    server_handle.abort();
    let _ = std::fs::remove_file(socket_path);
}

// ============================================================================
// E2E TESTS - Issue 2: family_id Priority Validation
// ============================================================================

#[tokio::test]
async fn test_e2e_family_id_priority_family_id_first() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let socket_path = "/tmp/songbird-test-fam1.sock";

    // Clean slate
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("NODE_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");

    // Canonical priority: SONGBIRD_ORCHESTRATOR_FAMILY_ID > BIOMEOS_FAMILY_ID
    // > SONGBIRD_FAMILY_ID > FAMILY_ID > NODE_FAMILY_ID
    // Set lower-priority vars — SONGBIRD_FAMILY_ID should win over FAMILY_ID
    songbird_process_env::set_var("FAMILY_ID", "lowest");
    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "winner");
    songbird_process_env::set_var("NODE_FAMILY_ID", "third");

    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"identity\",\"id\":1}\n").await.unwrap();

    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    timeout(Duration::from_secs(1), reader.read_line(&mut response)).await.unwrap().unwrap();

    let response: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["result"]["family_id"], "winner");

    // Cleanup
    drop(stream);
    server_handle.abort();
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("NODE_FAMILY_ID");
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_e2e_family_id_default() {
    let _guard = ENV_TEST_LOCK.lock().unwrap();
    let socket_path = "/tmp/songbird-test-default.sock";

    // Ensure no env vars set — canonical default is "default"
    songbird_process_env::remove_var("FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    songbird_process_env::remove_var("NODE_FAMILY_ID");
    songbird_process_env::remove_var("SONGBIRD_ORCHESTRATOR_FAMILY_ID");
    songbird_process_env::remove_var("BIOMEOS_FAMILY_ID");

    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    stream.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"identity\",\"id\":1}\n").await.unwrap();

    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    timeout(Duration::from_secs(1), reader.read_line(&mut response)).await.unwrap().unwrap();

    let response: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["result"]["family_id"], "default");

    // Cleanup
    drop(stream);
    server_handle.abort();
    let _ = std::fs::remove_file(socket_path);
}

// ============================================================================
// E2E TESTS - Persistent Connection Behavior (Issue 1 Clarification)
// ============================================================================

#[tokio::test]
async fn test_e2e_connection_stays_open_after_response() {
    let socket_path = "/tmp/songbird-test-persistent2.sock";

    let (server_handle, ready_rx) = start_test_server(socket_path).await;
    ready_rx.await.expect("Server failed to signal readiness");

    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // Send first request
    write_half.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"id\":1}\n").await.unwrap();

    let mut response1 = String::new();
    timeout(Duration::from_secs(1), reader.read_line(&mut response1)).await.unwrap().unwrap();

    assert!(response1.contains("\"result\""));

    // Wait a bit - connection should still be open
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send second request on SAME connection
    write_half
        .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"identity\",\"id\":2}\n")
        .await
        .unwrap();

    let mut response2 = String::new();
    timeout(Duration::from_secs(1), reader.read_line(&mut response2)).await.unwrap().unwrap();

    assert!(response2.contains("\"result\""));
    let res2: Value = serde_json::from_str(&response2).unwrap();
    assert_eq!(res2["id"], 2);

    // Cleanup (stream auto-drops at end of scope)
    server_handle.abort();
    let _ = std::fs::remove_file(socket_path);
}
