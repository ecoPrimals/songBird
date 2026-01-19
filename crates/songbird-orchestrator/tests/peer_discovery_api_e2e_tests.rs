// End-to-End Tests for Peer Discovery API (v3.8.0)
//
// These tests verify the complete flow of peer discovery from
// ConnectionManager through IPC to external clients.

use anyhow::Result;
use serde_json::{json, Value};
use songbird_orchestrator::app::connection_manager::{ConnectionManager, PeerMetadata};
use songbird_orchestrator::ipc::UnixSocketIpcServer;
use songbird_types::TrustLevel;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Helper to create a temporary socket path
fn test_socket() -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("test-discovery.sock");
    (dir, socket_path)
}

/// Helper to send a JSON-RPC request and receive response
async fn send_request(stream: &mut UnixStream, request: Value) -> Result<Value> {
    let request_str = serde_json::to_string(&request)?;

    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let (reader, _writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    Ok(serde_json::from_str(&line)?)
}

/// Start IPC server with ConnectionManager and wait for readiness
async fn start_server_with_manager(
    socket_path: std::path::PathBuf,
) -> (tokio::task::JoinHandle<Result<()>>, Arc<ConnectionManager>) {
    // Remove any existing socket
    if socket_path.exists() {
        let _ = tokio::fs::remove_file(&socket_path).await;
    }

    // Create connection manager
    let connection_manager = Arc::new(ConnectionManager::new());

    // Create IPC server
    let mut server = UnixSocketIpcServer::new(socket_path).await.unwrap();
    server.set_connection_manager(Arc::clone(&connection_manager));

    let ready_flag = server.readiness_flag();

    // Spawn server
    let handle = tokio::spawn(async move { server.start().await });

    // Wait for readiness
    UnixSocketIpcServer::wait_ready_flag(&ready_flag, Duration::from_secs(5)).await;

    (handle, connection_manager)
}

#[tokio::test]
async fn test_e2e_list_peers_empty() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, _manager) = start_server_with_manager(socket_path.clone()).await;

    // Connect as client
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    // Request peer list
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.list_peers",
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_null());
    assert_eq!(response["result"]["total"], 0);
    assert!(response["result"]["peers"].is_array());
    assert_eq!(response["result"]["peers"].as_array().unwrap().len(), 0);

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_list_peers_with_data() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    // Add some peers
    manager
        .establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    manager
        .establish_connection(
            "tower2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator", "federation-member"].iter().map(|s| s.to_string()).collect(),
            vec![], // peer_tags
            TrustLevel::Elevated,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Connect as client
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    // Request peer list
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.list_peers",
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_null());
    assert_eq!(response["result"]["total"], 2);

    let peers = response["result"]["peers"].as_array().unwrap();
    assert_eq!(peers.len(), 2);

    // Verify peer data
    let peer_ids: Vec<_> = peers.iter().map(|p| p["peer_id"].as_str().unwrap()).collect();
    assert!(peer_ids.contains(&"tower1"));
    assert!(peer_ids.contains(&"tower2"));

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_peer_count() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    // Initially 0 peers
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.peer_count",
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["count"], 0);

    // Add a peer
    manager
        .establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Check count again
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.peer_count",
        "id": 2
    });

    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["count"], 1);

    // Add another peer
    manager
        .establish_connection(
            "tower2".to_string(),
            "https://192.168.1.101:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Check count one more time
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.peer_count",
        "id": 3
    });

    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["count"], 2);

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_rejected_peers() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    // Reject some peers
    use songbird_orchestrator::trust::peer_trust::PeerTrustDecision;

    let decision1 = PeerTrustDecision::Reject {
        reason: "no_genetic_lineage".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue1".to_string(),
            "https://192.168.1.200:8080".to_string(),
            vec![],
            &decision1,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    let decision2 = PeerTrustDecision::Reject {
        reason: "different_family".to_string(),
        trust_level: "none".to_string(),
    };

    manager
        .handle_trust_decision(
            "rogue2".to_string(),
            "https://192.168.1.201:8080".to_string(),
            vec![],
            &decision2,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Connect and query rejected peers
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.rejected_peers",
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_null());
    assert_eq!(response["result"]["total"], 2);

    let rejected = response["result"]["rejected"].as_array().unwrap();
    assert_eq!(rejected.len(), 2);

    // Verify rejection reasons
    let peer_ids: Vec<_> = rejected.iter().map(|p| p["peer_id"].as_str().unwrap()).collect();
    assert!(peer_ids.contains(&"rogue1"));
    assert!(peer_ids.contains(&"rogue2"));

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_peer_ping_success() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    // Add a peer to ping
    manager
        .establish_connection(
            "tower1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Connect and ping the peer
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "peer.ping",
        "params": {"target": "tower1"},
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_null());
    assert_eq!(response["result"]["pong"], true);
    assert_eq!(response["result"]["peer_id"], "tower1");
    assert_eq!(response["result"]["endpoint"], "https://192.168.1.100:8080");
    assert!(response["result"]["latency_ms"].is_number());
    assert_eq!(response["result"]["trust_level"], 1); // TrustLevel::Limited

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_peer_ping_not_found() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, _manager) = start_server_with_manager(socket_path.clone()).await;

    // Connect and try to ping non-existent peer
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "peer.ping",
        "params": {"target": "non_existent"},
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_null());
    assert!(response["error"].is_object());
    assert!(response["error"]["message"].as_str().unwrap().contains("not found"));

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_concurrent_clients() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    // Add some peers
    for i in 1..=5 {
        manager
            .establish_connection(
                format!("peer{}", i),
                format!("https://192.168.1.{}:8080", 100 + i),
                vec!["orchestrator".to_string()],
                vec![], // peer_tags
                TrustLevel::Limited,
                "udp_multicast".to_string(),
            )
            .await
            .unwrap();
    }

    // Spawn multiple concurrent clients
    let mut handles = vec![];

    for i in 0..10 {
        let path = socket_path.clone();
        let handle = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&path).await.unwrap();

            let request = json!({
                "jsonrpc": "2.0",
                "method": "discovery.list_peers",
                "id": i
            });

            let response = send_request(&mut stream, request).await.unwrap();

            assert_eq!(response["id"], i);
            assert_eq!(response["result"]["total"], 5);
        });
        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        handle.await.unwrap();
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_method_not_found() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, _manager) = start_server_with_manager(socket_path.clone()).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.non_existent_method",
        "id": 1
    });

    let response = send_request(&mut stream, request).await.unwrap();

    // Verify error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_null());
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // Method not found

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_invalid_json() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, _manager) = start_server_with_manager(socket_path.clone()).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    // Send invalid JSON
    stream.write_all(b"this is not json\n").await.unwrap();
    stream.flush().await.unwrap();

    let (reader, _writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    let response: Value = serde_json::from_str(&line).unwrap();

    // Verify parse error
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // Parse error

    server_handle.abort();
}

#[tokio::test]
async fn test_e2e_sequential_requests() {
    let (_dir, socket_path) = test_socket();
    let (server_handle, manager) = start_server_with_manager(socket_path.clone()).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();

    // Request 1: Check initial count
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.peer_count",
        "id": 1
    });
    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["count"], 0);

    // Add a peer
    manager
        .establish_connection(
            "peer1".to_string(),
            "https://192.168.1.100:8080".to_string(),
            vec!["orchestrator".to_string()],
            vec![], // peer_tags
            TrustLevel::Limited,
            "udp_multicast".to_string(),
        )
        .await
        .unwrap();

    // Request 2: List peers
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.list_peers",
        "id": 2
    });
    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["total"], 1);

    // Request 3: Ping peer
    let request = json!({
        "jsonrpc": "2.0",
        "method": "peer.ping",
        "params": {"target": "peer1"},
        "id": 3
    });
    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["pong"], true);

    // Request 4: Check final count
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.peer_count",
        "id": 4
    });
    let response = send_request(&mut stream, request).await.unwrap();
    assert_eq!(response["result"]["count"], 1);

    server_handle.abort();
}
