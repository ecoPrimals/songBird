#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Integration tests for STUN server and client
//!
//! Tests the complete server ↔ client interaction.

use songbird_stun::{StunClient, StunServer};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;

/// Helper: start a STUN server on an OS-assigned port and return
/// the join handle plus the actual bound address (no sleep needed).
async fn start_stun_server() -> (tokio::task::JoinHandle<()>, std::net::SocketAddr) {
    let (ready_tx, ready_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        let server = StunServer::new("127.0.0.1:0".parse().unwrap());
        let _ = server.run_with_ready(ready_tx).await;
    });

    let addr = ready_rx.await.expect("Server failed to signal readiness");
    (handle, addr)
}

#[tokio::test]
async fn test_server_client_loopback_integration() {
    let (server_handle, actual_addr) = start_stun_server().await;

    // Create client and discover address
    let client = StunClient::new();
    let result =
        timeout(Duration::from_secs(2), client.discover_public_address(&actual_addr.to_string()))
            .await;

    // Verify discovery worked
    assert!(result.is_ok(), "Discovery timed out");
    let discovered = result.unwrap();
    assert!(discovered.is_ok(), "Discovery failed: {:?}", discovered.err());

    let public_addr = discovered.unwrap();

    // Should discover loopback address
    assert!(public_addr.ip().is_loopback(), "Expected loopback address, got: {}", public_addr);

    // Cleanup: abort server
    server_handle.abort();
}

#[tokio::test]
async fn test_multiple_clients_to_server() {
    let (server_handle, actual_addr) = start_stun_server().await;

    // Create multiple clients and send concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let addr = actual_addr.to_string();
        let handle = tokio::spawn(async move {
            let client = StunClient::new();
            client.discover_public_address(&addr).await
        });
        handles.push(handle);
    }

    // Wait for all clients
    let mut successes = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            successes += 1;
        }
    }

    // All clients should succeed
    assert_eq!(successes, 10, "Not all clients succeeded");

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_server_handles_invalid_messages() {
    let (server_handle, actual_addr) = start_stun_server().await;

    // Send invalid data to server
    let client_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let invalid_data = vec![0u8; 100]; // Invalid STUN message

    let send_result = client_socket.send_to(&invalid_data, actual_addr).await;
    assert!(send_result.is_ok());

    // Verify server still responds to valid requests after invalid message
    let client = StunClient::new();
    let result = client.discover_public_address(&actual_addr.to_string()).await;
    assert!(result.is_ok(), "Server stopped responding after invalid message");

    // Cleanup
    server_handle.abort();
}
