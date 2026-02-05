//! Integration tests for STUN server and client
//!
//! Tests the complete server ↔ client interaction.

use songbird_stun::{StunClient, StunServer};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_server_client_loopback_integration() {
    // Start server on random port (port 0 = OS assigns)
    let server_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    
    // Get the actual bound address before spawning
    // (We need to bind to get the actual port)
    let socket = tokio::net::UdpSocket::bind(server_addr).await.unwrap();
    let actual_addr = socket.local_addr().unwrap();
    drop(socket); // Close so server can bind
    
    // Spawn server in background
    let mut server = StunServer::new(actual_addr);
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create client and discover address
    let client = StunClient::new();
    let result = timeout(
        Duration::from_secs(2),
        client.discover_public_address(&actual_addr.to_string())
    ).await;
    
    // Verify discovery worked
    assert!(result.is_ok(), "Discovery timed out");
    let discovered = result.unwrap();
    assert!(discovered.is_ok(), "Discovery failed: {:?}", discovered.err());
    
    let public_addr = discovered.unwrap();
    
    // Should discover loopback address
    assert!(public_addr.ip().is_loopback(), 
            "Expected loopback address, got: {}", public_addr);
    
    // Cleanup: abort server
    server_handle.abort();
}

#[tokio::test]
async fn test_multiple_clients_to_server() {
    // Start server
    let server_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let socket = tokio::net::UdpSocket::bind(server_addr).await.unwrap();
    let actual_addr = socket.local_addr().unwrap();
    drop(socket);
    
    let mut server = StunServer::new(actual_addr);
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
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
    // Start server
    let server_addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let socket = tokio::net::UdpSocket::bind(server_addr).await.unwrap();
    let actual_addr = socket.local_addr().unwrap();
    drop(socket);
    
    let mut server = StunServer::new(actual_addr);
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Send invalid data to server
    let client_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let invalid_data = vec![0u8; 100]; // Invalid STUN message
    
    let send_result = client_socket.send_to(&invalid_data, actual_addr).await;
    assert!(send_result.is_ok());
    
    // Server should continue running (not crash)
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify server still responds to valid requests
    let client = StunClient::new();
    let result = client.discover_public_address(&actual_addr.to_string()).await;
    assert!(result.is_ok(), "Server stopped responding after invalid message");
    
    // Cleanup
    server_handle.abort();
}
