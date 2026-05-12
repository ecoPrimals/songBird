// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::message::StunMessage;
use crate::turn::TurnClient;
use crate::types::StunCredentials;

fn test_credentials() -> Arc<dyn CredentialStore> {
    let mut store = StaticCredentialStore::new();
    store.insert("testuser".to_string(), b"testkey123".to_vec());
    Arc::new(store)
}

#[tokio::test]
async fn server_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = TurnRelayServer::new(addr, test_credentials());
    let stats = server.stats().await;
    assert_eq!(stats.allocations_created, 0);
}

#[tokio::test]
async fn static_credential_store_get_key() {
    let mut store = StaticCredentialStore::new();
    store.insert("user1".to_string(), b"key1".to_vec());
    assert!(store.get_key("user1").is_some());
    assert!(store.get_key("missing").is_none());
}

#[tokio::test]
async fn turn_server_allocate_and_refresh() {
    let creds = test_credentials();
    let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = server.run_with_ready(tx).await;
    });

    let server_addr = rx.await.expect("server ready");

    let client_creds = StunCredentials {
        username: "testuser".to_string(),
        key: b"testkey123".to_vec(),
    };
    let client = TurnClient::new(server_addr, client_creds);

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let alloc = client.allocate(&socket).await.expect("allocate should succeed");
    assert!(alloc.relay_addr.port() > 0);
    assert!(alloc.lifetime_secs > 0);

    let new_lifetime = client.refresh(&socket, 300).await.expect("refresh should succeed");
    assert!(new_lifetime > 0);

    server_handle.abort();
}

#[tokio::test]
async fn turn_server_binding_request() {
    let creds = test_credentials();
    let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = server.run_with_ready(tx).await;
    });

    let server_addr = rx.await.expect("server ready");

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let request = StunMessage::new_binding_request();
    let wire = request.encode();
    socket.send_to(&wire, server_addr).await.unwrap();

    let mut buf = [0u8; 1500];
    let (len, _) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf))
        .await
        .expect("timeout")
        .expect("recv");

    let response = StunMessage::decode(&buf[..len]).expect("decode");
    assert_eq!(response.message_type, MessageType::BindingResponse);
    assert!(response.get_xor_mapped_address().is_some());

    server_handle.abort();
}

#[tokio::test]
async fn turn_server_rejects_unknown_user() {
    let creds = test_credentials();
    let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = server.run_with_ready(tx).await;
    });

    let server_addr = rx.await.expect("server ready");

    let bad_creds = StunCredentials {
        username: "baduser".to_string(),
        key: b"wrongkey".to_vec(),
    };
    let client = TurnClient::new(server_addr, bad_creds);
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let result = client.allocate(&socket).await;
    assert!(result.is_err(), "should reject unknown user");

    server_handle.abort();
}

#[tokio::test]
async fn turn_relay_stats_initial() {
    let stats = TurnRelayStats::default();
    assert_eq!(stats.allocations_created, 0);
    assert_eq!(stats.active_allocations, 0);
    assert_eq!(stats.packets_relayed, 0);
    assert_eq!(stats.bytes_relayed, 0);
    assert_eq!(stats.auth_failures, 0);
    assert!(stats.start_time.is_none());
}
