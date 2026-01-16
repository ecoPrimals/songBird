//! BTSP Unix Socket Integration Tests
//!
//! Tests the Unix socket-based BTSP client with a real BearDog server.
//!
//! **Status**: Scaffolding ready for Week 2
//! **Blocked By**: Requires BearDog Unix socket server running
//! **Run With**: `cargo test --test btsp_unix_socket_integration -- --ignored`

mod helpers;

use helpers::{BearDogMock, temp_unix_socket_path, cleanup_socket};
use songbird_orchestrator::btsp_client::{BtspClient, PeerEndpoint, Direction, TunnelState};
use std::time::Duration;

#[tokio::test]
#[ignore = "Week 2: Requires BearDog Unix socket server"]
async fn test_btsp_tunnel_establishment() {
    // Setup: Start BearDog mock
    let socket_path = temp_unix_socket_path("beardog");
    let mut mock = BearDogMock::new(&socket_path);
    mock.start().await.unwrap();
    
    // Spawn mock handler
    tokio::spawn(async move {
        loop {
            if let Err(e) = mock.handle_connection().await {
                eprintln!("Mock error: {}", e);
                break;
            }
        }
    });
    
    // Give mock time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test: Create BTSP client with mock socket
    std::env::set_var("BEARDOG_SOCKET", &socket_path);
    let client = BtspClient::new();
    
    // Test: Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer".to_string(),
        endpoint: format!("unix://{}", socket_path),
        public_key: None,
        capabilities: vec!["btsp_enabled".to_string()],
    };
    
    let tunnel = client.establish_tunnel(peer).await.unwrap();
    assert_eq!(tunnel.state, TunnelState::Established);
    assert_eq!(tunnel.peer_id, "test-peer");
    
    // Cleanup
    cleanup_socket(&socket_path);
    std::env::remove_var("BEARDOG_SOCKET");
}

#[tokio::test]
#[ignore = "Week 2: Requires BearDog Unix socket server"]
async fn test_btsp_tunnel_encrypt_decrypt() {
    // Setup
    let socket_path = temp_unix_socket_path("beardog-crypt");
    let mut mock = BearDogMock::new(&socket_path);
    mock.start().await.unwrap();
    
    tokio::spawn(async move {
        loop {
            if let Err(e) = mock.handle_connection().await {
                break;
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    std::env::set_var("BEARDOG_SOCKET", &socket_path);
    let client = BtspClient::new();
    
    let peer = PeerEndpoint {
        id: "test-peer".to_string(),
        endpoint: format!("unix://{}", socket_path),
        public_key: None,
        capabilities: vec!["btsp_enabled".to_string()],
    };
    
    let tunnel = client.establish_tunnel(peer).await.unwrap();
    
    // Test: Encrypt data
    let plaintext = b"Hello BTSP!";
    let ciphertext = client
        .tunnel_encrypt(&tunnel, plaintext, Direction::Egress)
        .await
        .unwrap();
    
    // Note: Mock returns same data, real BearDog would encrypt
    assert!(!ciphertext.is_empty());
    
    // Test: Decrypt data
    let decrypted = client.tunnel_decrypt(&tunnel, &ciphertext).await.unwrap();
    assert_eq!(decrypted, plaintext);
    
    // Test: Close tunnel
    client.tunnel_close(&tunnel).await.unwrap();
    
    // Cleanup
    cleanup_socket(&socket_path);
    std::env::remove_var("BEARDOG_SOCKET");
}

#[tokio::test]
#[ignore = "Week 2: Requires BearDog Unix socket server"]
async fn test_btsp_tunnel_status() {
    // Setup
    let socket_path = temp_unix_socket_path("beardog-status");
    let mut mock = BearDogMock::new(&socket_path);
    mock.start().await.unwrap();
    
    tokio::spawn(async move {
        loop {
            if let Err(e) = mock.handle_connection().await {
                break;
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    std::env::set_var("BEARDOG_SOCKET", &socket_path);
    let client = BtspClient::new();
    
    let peer = PeerEndpoint {
        id: "test-peer".to_string(),
        endpoint: format!("unix://{}", socket_path),
        public_key: None,
        capabilities: vec!["btsp_enabled".to_string()],
    };
    
    let tunnel = client.establish_tunnel(peer).await.unwrap();
    
    // Test: Get tunnel status
    let status = client.tunnel_status(&tunnel).await.unwrap();
    assert!(status.get("tunnel_id").is_some());
    
    // Cleanup
    client.tunnel_close(&tunnel).await.unwrap();
    cleanup_socket(&socket_path);
    std::env::remove_var("BEARDOG_SOCKET");
}

#[tokio::test]
#[ignore = "Week 2: Requires BearDog Unix socket server"]
async fn test_btsp_ping() {
    // Setup
    let socket_path = temp_unix_socket_path("beardog-ping");
    let mut mock = BearDogMock::new(&socket_path);
    mock.start().await.unwrap();
    
    tokio::spawn(async move {
        loop {
            if let Err(e) = mock.handle_connection().await {
                break;
            }
        }
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    std::env::set_var("BEARDOG_SOCKET", &socket_path);
    let client = BtspClient::new();
    
    // Test: Ping BearDog
    let response = client.ping().await.unwrap();
    assert_eq!(response["result"]["primal"], "beardog");
    
    // Cleanup
    cleanup_socket(&socket_path);
    std::env::remove_var("BEARDOG_SOCKET");
}

