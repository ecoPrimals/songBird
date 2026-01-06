//! Integration tests for Unix Socket IPC and Primal Registry
//!
//! These tests verify the complete IPC flow:
//! 1. Server startup and socket creation
//! 2. Client connection and registration
//! 3. Capability-based discovery
//! 4. Multiple concurrent clients
//! 5. Graceful error handling
//!
//! ## Modern Concurrent Testing
//! These tests use atomic readiness flags instead of sleep-based polling,
//! ensuring true concurrency and fast test execution.

use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

// Helper to create a test socket path with temp dir
fn test_socket() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("test-songbird.sock");
    (dir, socket_path)
}

// Helper to start server and wait for readiness (atomic, no sleep!)
async fn start_server_ready(
    socket_path: PathBuf
) -> (tokio::task::JoinHandle<()>, Arc<AtomicBool>) {
    use songbird_orchestrator::ipc::UnixSocketIpcServer;
    
    let server = UnixSocketIpcServer::new(socket_path).await.unwrap();
    let ready_flag = server.readiness_flag();
    
    let handle = tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    // Wait for server to be ready (atomic check, no filesystem polling!)
    assert!(
        UnixSocketIpcServer::wait_ready_flag(&ready_flag, tokio::time::Duration::from_secs(5)).await,
        "Server should become ready within 5 seconds"
    );
    
    (handle, ready_flag)
}

// Helper to send JSON-RPC request and receive response
async fn send_request(
    stream: &mut UnixStream,
    request: serde_json::Value,
) -> Result<serde_json::Value> {
    // Send request
    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    // Read response
    let (reader, _writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    // Parse response
    let response: serde_json::Value = serde_json::from_str(&line)?;
    Ok(response)
}

#[tokio::test]
async fn test_ipc_server_startup() {
    use songbird_orchestrator::ipc::UnixSocketIpcServer;

    let (_dir, socket_path) = test_socket();
    let server = UnixSocketIpcServer::new(socket_path.clone()).await.unwrap();
    
    assert_eq!(server.socket_path(), socket_path);
    assert!(socket_path.exists(), "Socket file should exist");
}

#[tokio::test]
async fn test_primal_registration() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    // Connect as mock BearDog primal
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Register primal
    let request = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "beardog-test",
            "capabilities": ["security", "encryption"],
            "endpoint": "http://localhost:9000"
        },
        "id": 1
    });
    
    let response = send_request(&mut stream, request).await.unwrap();
    
    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["success"].as_bool().unwrap());
    assert_eq!(response["result"]["primal_id"], "beardog-test");
    
    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_capability_discovery() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    // Connect and register
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Register BearDog
    let register_request = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "beardog-test",
            "capabilities": ["security", "encryption", "trust"],
            "endpoint": "http://localhost:9000"
        },
        "id": 1
    });
    
    send_request(&mut stream, register_request).await.unwrap();
    
    // Query for security provider
    let query_request = json!({
        "jsonrpc": "2.0",
        "method": "primal.get_provider",
        "params": {
            "capability": "security"
        },
        "id": 2
    });
    
    let response = send_request(&mut stream, query_request).await.unwrap();
    
    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert_eq!(response["result"]["primal_id"], "beardog-test");
    assert_eq!(response["result"]["capabilities"][0], "security");
    
    server_handle.abort();
}

#[tokio::test]
async fn test_multiple_providers() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    // Connect and register multiple primals
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Register BearDog
    let register_beardog = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "beardog-test",
            "capabilities": ["security", "encryption"],
            "endpoint": "http://localhost:9000"
        },
        "id": 1
    });
    send_request(&mut stream, register_beardog).await.unwrap();
    
    // Register ToadStool
    let register_toadstool = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "toadstool-test",
            "capabilities": ["storage", "blob-storage"],
            "endpoint": "http://localhost:8000"
        },
        "id": 2
    });
    send_request(&mut stream, register_toadstool).await.unwrap();
    
    // Register Gorilla
    let register_gorilla = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "gorilla-test",
            "capabilities": ["compute", "ai-inference"],
            "endpoint": "http://localhost:7000"
        },
        "id": 3
    });
    send_request(&mut stream, register_gorilla).await.unwrap();
    
    // List all primals
    let list_request = json!({
        "jsonrpc": "2.0",
        "method": "primal.list_all",
        "id": 4
    });
    
    let response = send_request(&mut stream, list_request).await.unwrap();
    
    // Verify all primals registered
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 4);
    let primals = response["result"].as_array().unwrap();
    assert_eq!(primals.len(), 3);
    
    // Verify each primal is present
    let primal_ids: Vec<String> = primals
        .iter()
        .map(|p| p["primal_id"].as_str().unwrap().to_string())
        .collect();
    assert!(primal_ids.contains(&"beardog-test".to_string()));
    assert!(primal_ids.contains(&"toadstool-test".to_string()));
    assert!(primal_ids.contains(&"gorilla-test".to_string()));
    
    server_handle.abort();
}

#[tokio::test]
async fn test_unregister_primal() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Register
    let register = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "test-primal",
            "capabilities": ["test-capability"],
            "endpoint": "http://localhost:5000"
        },
        "id": 1
    });
    send_request(&mut stream, register).await.unwrap();
    
    // Verify registered
    let list1 = json!({
        "jsonrpc": "2.0",
        "method": "primal.list_all",
        "id": 2
    });
    let response1 = send_request(&mut stream, list1).await.unwrap();
    assert_eq!(response1["result"].as_array().unwrap().len(), 1);
    
    // Unregister
    let unregister = json!({
        "jsonrpc": "2.0",
        "method": "primal.unregister",
        "params": {
            "primal_id": "test-primal"
        },
        "id": 3
    });
    let response2 = send_request(&mut stream, unregister).await.unwrap();
    assert!(response2["result"]["success"].as_bool().unwrap());
    
    // Verify unregistered
    let list2 = json!({
        "jsonrpc": "2.0",
        "method": "primal.list_all",
        "id": 4
    });
    let response3 = send_request(&mut stream, list2).await.unwrap();
    assert_eq!(response3["result"].as_array().unwrap().len(), 0);
    
    server_handle.abort();
}

#[tokio::test]
async fn test_health_and_ping() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Ping
    let ping = json!({
        "jsonrpc": "2.0",
        "method": "primal.ping",
        "id": 1
    });
    let response1 = send_request(&mut stream, ping).await.unwrap();
    assert_eq!(response1["jsonrpc"], "2.0");
    assert_eq!(response1["result"]["pong"], true);
    
    // Health
    let health = json!({
        "jsonrpc": "2.0",
        "method": "primal.health",
        "id": 2
    });
    let response2 = send_request(&mut stream, health).await.unwrap();
    assert_eq!(response2["result"]["status"], "healthy");
    assert_eq!(response2["result"]["registered_primals"], 0);
    
    server_handle.abort();
}

#[tokio::test]
async fn test_invalid_method() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Call non-existent method
    let request = json!({
        "jsonrpc": "2.0",
        "method": "primal.nonexistent",
        "id": 1
    });
    
    let response = send_request(&mut stream, request).await.unwrap();
    
    // Verify error
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND
    
    server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_connections() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    // Spawn multiple concurrent clients
    let mut handles = vec![];
    
    for i in 0..5 {
        let socket_path = socket_path.clone();
        let handle = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&socket_path).await.unwrap();
            
            // Register primal
            let register = json!({
                "jsonrpc": "2.0",
                "method": "primal.register",
                "params": {
                    "primal_id": format!("primal-{}", i),
                    "capabilities": [format!("capability-{}", i)],
                    "endpoint": format!("http://localhost:{}", 8000 + i)
                },
                "id": 1
            });
            
            let response = send_request(&mut stream, register).await.unwrap();
            assert!(response["result"]["success"].as_bool().unwrap());
        });
        
        handles.push(handle);
    }
    
    // Wait for all clients
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all registered
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let list = json!({
        "jsonrpc": "2.0",
        "method": "primal.list_all",
        "id": 1
    });
    let response = send_request(&mut stream, list).await.unwrap();
    assert_eq!(response["result"].as_array().unwrap().len(), 5);
    
    server_handle.abort();
}

#[tokio::test]
async fn test_list_providers_for_capability() {
    let (_dir, socket_path) = test_socket();
    
    // Start server and wait for readiness (atomic, no sleep!)
    let (server_handle, _ready_flag) = start_server_ready(socket_path.clone()).await;
    
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    
    // Register two primals with "security" capability
    let register1 = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "beardog-1",
            "capabilities": ["security", "encryption"],
            "endpoint": "http://localhost:9001"
        },
        "id": 1
    });
    send_request(&mut stream, register1).await.unwrap();
    
    let register2 = json!({
        "jsonrpc": "2.0",
        "method": "primal.register",
        "params": {
            "primal_id": "beardog-2",
            "capabilities": ["security", "trust"],
            "endpoint": "http://localhost:9002"
        },
        "id": 2
    });
    send_request(&mut stream, register2).await.unwrap();
    
    // List all providers for "security"
    let list = json!({
        "jsonrpc": "2.0",
        "method": "primal.list_providers",
        "params": {
            "capability": "security"
        },
        "id": 3
    });
    
    let response = send_request(&mut stream, list).await.unwrap();
    
    // Verify both providers returned
    assert_eq!(response["result"].as_array().unwrap().len(), 2);
    
    server_handle.abort();
}

