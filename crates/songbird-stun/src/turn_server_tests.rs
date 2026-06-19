// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::message::{MessageType, StunAttribute, StunMessage};
use crate::turn::{TurnClient, encode_xor_peer_address};
use crate::types::StunCredentials;
use bytes::{BufMut, BytesMut};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

fn test_credentials() -> Arc<dyn CredentialStore> {
    let mut store = StaticCredentialStore::new();
    store.insert("testuser".to_string(), b"testkey123".to_vec());
    Arc::new(store)
}

fn client_creds() -> StunCredentials {
    StunCredentials {
        username: "testuser".to_string(),
        key: b"testkey123".to_vec(),
    }
}

async fn start_turn_relay() -> (tokio::task::JoinHandle<()>, SocketAddr, Arc<TurnRelayServer>) {
    let server = Arc::new(TurnRelayServer::new(
        "127.0.0.1:0".parse().expect("parse bind addr"),
        test_credentials(),
    ));
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let server_task = Arc::clone(&server);
    let handle = tokio::spawn(async move {
        let _ = server_task.run_with_ready(ready_tx).await;
    });
    let addr = ready_rx.await.expect("server ready");
    (handle, addr, server)
}

fn parse_stun_error_code(msg: &StunMessage) -> Option<(u16, String)> {
    const ERROR_CODE: u16 = 0x0009;
    msg.attributes.iter().find_map(|attr| {
        if let StunAttribute::Unknown(ERROR_CODE, data) = attr {
            (data.len() >= 4).then(|| {
                let code = u16::from(data[2]) * 100 + u16::from(data[3]);
                let reason = String::from_utf8_lossy(&data[4..]).into_owned();
                (code, reason)
            })
        } else {
            None
        }
    })
}

async fn recv_stun_response(socket: &UdpSocket) -> StunMessage {
    let mut buf = [0u8; 2048];
    let (len, _) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf))
        .await
        .expect("recv timeout")
        .expect("recv failed");
    StunMessage::decode(&buf[..len]).expect("decode response")
}

fn build_send_indication(peer: SocketAddr, payload: &[u8]) -> StunMessage {
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::SendIndication;
    msg.attributes
        .push(StunAttribute::Unknown(0x0012, encode_xor_peer_address(&peer, &msg.transaction_id)));
    msg.attributes.push(StunAttribute::Unknown(0x0013, bytes::Bytes::copy_from_slice(payload)));
    msg
}

fn build_channel_data_frame(channel: u16, payload: &[u8]) -> Vec<u8> {
    let len = u16::try_from(payload.len()).expect("payload fits u16");
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&channel.to_be_bytes());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn build_allocate_with_lifetime(lifetime_secs: u32) -> StunMessage {
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::Allocate;
    msg.attributes.push(StunAttribute::Username("testuser".to_string()));
    let mut transport_attr = BytesMut::with_capacity(4);
    transport_attr.put_u8(17);
    transport_attr.put_u8(0);
    transport_attr.put_u8(0);
    transport_attr.put_u8(0);
    msg.attributes.push(StunAttribute::Unknown(0x0019, transport_attr.freeze()));
    let mut lifetime_attr = BytesMut::with_capacity(4);
    lifetime_attr.put_u32(lifetime_secs);
    msg.attributes.push(StunAttribute::Unknown(0x000D, lifetime_attr.freeze()));
    msg
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

#[tokio::test]
async fn turn_server_create_permission_success() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");
    let peer_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("create_permission should succeed");

    handle.abort();
}

#[tokio::test]
async fn turn_server_channel_bind_after_permission() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");
    let peer_addr: SocketAddr = "127.0.0.1:8888".parse().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("create_permission");
    client.channel_bind(&socket, 0x4000, peer_addr).await.expect("channel_bind should succeed");

    handle.abort();
}

#[tokio::test]
async fn turn_server_refresh_lifetime_zero_releases_allocation() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");
    let stats = server.stats().await;
    assert_eq!(stats.active_allocations, 1);

    let lifetime = client.refresh(&socket, 0).await.expect("refresh release");
    assert_eq!(lifetime, 0);

    let stats = server.stats().await;
    assert_eq!(stats.active_allocations, 0);

    handle.abort();
}

#[tokio::test]
async fn turn_server_duplicate_allocate_returns_error_437() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("first allocate");

    let result = client.allocate(&socket).await;
    assert!(result.is_err(), "duplicate allocate should fail");
    assert!(
        result.unwrap_err().to_string().contains("rejected"),
        "expected Allocate rejection error"
    );

    handle.abort();
}

#[tokio::test]
async fn turn_server_duplicate_allocate_error_code_437_on_wire() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let creds = client_creds();
    let client = TurnClient::new(server_addr, creds.clone());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("first allocate");

    let request = {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::Allocate;
        msg.attributes.push(StunAttribute::Username("testuser".to_string()));
        let mut transport_attr = BytesMut::with_capacity(4);
        transport_attr.put_u8(17);
        transport_attr.put_u8(0);
        transport_attr.put_u8(0);
        transport_attr.put_u8(0);
        msg.attributes.push(StunAttribute::Unknown(0x0019, transport_attr.freeze()));
        msg
    };
    let wire = request.encode_authenticated(&creds.key);
    socket.send_to(&wire, server_addr).await.unwrap();

    let response = recv_stun_response(&socket).await;
    assert_eq!(response.message_type, MessageType::AllocateError);
    let (code, reason) = parse_stun_error_code(&response).expect("ERROR-CODE attribute");
    assert_eq!(code, 437);
    assert!(reason.contains("Allocation mismatch"));

    handle.abort();
}

#[tokio::test]
async fn turn_server_allocate_missing_username_returns_401() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::Allocate;
    let mut transport_attr = BytesMut::with_capacity(4);
    transport_attr.put_u8(17);
    transport_attr.put_u8(0);
    transport_attr.put_u8(0);
    transport_attr.put_u8(0);
    msg.attributes.push(StunAttribute::Unknown(0x0019, transport_attr.freeze()));
    let wire = msg.encode();
    socket.send_to(&wire, server_addr).await.unwrap();

    let response = recv_stun_response(&socket).await;
    assert_eq!(response.message_type, MessageType::AllocateError);
    let (code, reason) = parse_stun_error_code(&response).expect("ERROR-CODE attribute");
    assert_eq!(code, 401);
    assert_eq!(reason, "Unauthorized");

    handle.abort();
}

#[tokio::test]
async fn turn_server_error_response_format_unauthorized() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::Allocate;
    msg.attributes.push(StunAttribute::Username("nobody".to_string()));
    let wire = msg.encode();
    socket.send_to(&wire, server_addr).await.unwrap();

    let response = recv_stun_response(&socket).await;
    assert_eq!(response.message_type, MessageType::AllocateError);
    assert_eq!(response.transaction_id, msg.transaction_id);

    let (code, reason) = parse_stun_error_code(&response).expect("ERROR-CODE");
    assert_eq!(code, 401);
    assert_eq!(reason, "Unauthorized");

    if let StunAttribute::Unknown(0x0009, data) = &response.attributes[0] {
        assert_eq!(data[0], 0);
        assert_eq!(data[1], 0);
        assert_eq!(data[2], 4);
        assert_eq!(data[3], 1);
        assert_eq!(&data[4..], b"Unauthorized");
    } else {
        panic!("expected ERROR-CODE attribute");
    }

    handle.abort();
}

#[tokio::test]
async fn turn_server_stats_update_after_allocate() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let before = server.stats().await;
    assert_eq!(before.allocations_created, 0);
    assert_eq!(before.active_allocations, 0);

    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    client.allocate(&socket).await.expect("allocate");

    let after = server.stats().await;
    assert_eq!(after.allocations_created, 1);
    assert_eq!(after.active_allocations, 1);
    assert!(after.uptime_seconds <= 5);

    handle.abort();
}

#[tokio::test]
async fn turn_server_refresh_unauthorized_returns_401() {
    let (handle, server_addr, _server) = start_turn_relay().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::Refresh;
    let wire = msg.encode();
    socket.send_to(&wire, server_addr).await.unwrap();

    let response = recv_stun_response(&socket).await;
    assert_eq!(response.message_type, MessageType::RefreshError);
    let (code, reason) = parse_stun_error_code(&response).expect("ERROR-CODE");
    assert_eq!(code, 401);
    assert_eq!(reason, "Unauthorized");

    handle.abort();
}

#[tokio::test]
async fn turn_server_auth_failure_increments_stats() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::Allocate;
    msg.attributes.push(StunAttribute::Username("baduser".to_string()));
    let wire = msg.encode();
    socket.send_to(&wire, server_addr).await.unwrap();
    let _ = recv_stun_response(&socket).await;

    let stats = server.stats().await;
    assert_eq!(stats.auth_failures, 1);

    handle.abort();
}

#[tokio::test]
async fn static_credential_store_len_and_is_empty() {
    let mut store = StaticCredentialStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    store.insert("a".to_string(), b"k".to_vec());
    assert!(!store.is_empty());
    assert_eq!(store.len(), 1);
}

#[tokio::test]
async fn turn_server_send_indication_relays_to_permitted_peer() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");

    let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = peer_socket.local_addr().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("permission");

    let payload = b"relay-via-send-indication";
    let indication = build_send_indication(peer_addr, payload);
    socket.send_to(&indication.encode(), server_addr).await.expect("send indication");

    let mut buf = [0u8; 2048];
    let (len, from) = tokio::time::timeout(Duration::from_secs(2), peer_socket.recv_from(&mut buf))
        .await
        .expect("peer recv timeout")
        .expect("peer should receive relayed payload");
    assert_eq!(&buf[..len], payload);
    assert!(from.ip().is_loopback(), "relay should originate from loopback, got {from}");

    let stats = server.stats().await;
    assert!(stats.packets_relayed >= 1, "send indication should increment packets_relayed");
    assert!(
        stats.bytes_relayed >= payload.len() as u64,
        "bytes_relayed should include payload size"
    );

    handle.abort();
}

#[tokio::test]
async fn turn_server_send_indication_dropped_without_permission() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");

    let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = peer_socket.local_addr().unwrap();
    // No create_permission — peer is unpermitted.

    let indication = build_send_indication(peer_addr, b"should-not-arrive");
    socket.send_to(&indication.encode(), server_addr).await.unwrap();

    let recv_result =
        tokio::time::timeout(Duration::from_millis(300), peer_socket.recv_from(&mut [0u8; 256]))
            .await;
    assert!(recv_result.is_err(), "unpermitted peer should not receive relayed data");

    let stats = server.stats().await;
    assert_eq!(
        stats.packets_relayed, 0,
        "dropped send indication should not increment relay stats"
    );

    handle.abort();
}

#[tokio::test]
async fn turn_server_send_indication_missing_data_is_ignored() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");

    let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = peer_socket.local_addr().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("permission");

    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::SendIndication;
    msg.attributes.push(StunAttribute::Unknown(
        0x0012,
        encode_xor_peer_address(&peer_addr, &msg.transaction_id),
    ));
    socket.send_to(&msg.encode(), server_addr).await.unwrap();

    let recv_result =
        tokio::time::timeout(Duration::from_millis(300), peer_socket.recv_from(&mut [0u8; 256]))
            .await;
    assert!(recv_result.is_err(), "SendIndication without DATA should not relay");

    let stats = server.stats().await;
    assert_eq!(stats.packets_relayed, 0);

    handle.abort();
}

#[tokio::test]
async fn turn_server_channel_data_relays_to_bound_peer() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");

    let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = peer_socket.local_addr().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("permission");
    client.channel_bind(&socket, 0x4000, peer_addr).await.expect("channel bind");

    let payload = b"channel-data-payload";
    let frame = build_channel_data_frame(0x4000, payload);
    socket.send_to(&frame, server_addr).await.expect("send channel data");

    let mut buf = [0u8; 2048];
    let (len, _) = tokio::time::timeout(Duration::from_secs(2), peer_socket.recv_from(&mut buf))
        .await
        .expect("peer recv timeout")
        .expect("peer should receive channel data relay");
    assert_eq!(&buf[..len], payload);

    let stats = server.stats().await;
    assert!(stats.packets_relayed >= 1);
    assert!(stats.bytes_relayed >= payload.len() as u64);

    handle.abort();
}

#[tokio::test]
async fn turn_server_channel_data_dropped_for_unbound_channel() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let client = TurnClient::new(server_addr, client_creds());
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    client.allocate(&socket).await.expect("allocate");

    let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let peer_addr = peer_socket.local_addr().unwrap();
    client.create_permission(&socket, peer_addr).await.expect("permission");
    // Channel 0x4000 not bound.

    let frame = build_channel_data_frame(0x4000, b"orphan-channel");
    socket.send_to(&frame, server_addr).await.unwrap();

    let recv_result =
        tokio::time::timeout(Duration::from_millis(300), peer_socket.recv_from(&mut [0u8; 256]))
            .await;
    assert!(recv_result.is_err(), "unbound channel should not relay data");

    let stats = server.stats().await;
    assert_eq!(stats.packets_relayed, 0);

    handle.abort();
}

#[tokio::test]
async fn turn_server_cleanup_loop_removes_expired_allocation() {
    let (handle, server_addr, server) = start_turn_relay().await;
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let alloc_msg = build_allocate_with_lifetime(2);
    socket.send_to(&alloc_msg.encode(), server_addr).await.unwrap();
    let _response = recv_stun_response(&socket).await;

    let stats = server.stats().await;
    assert_eq!(stats.active_allocations, 1, "short-lived allocation should be active");

    // cleanup_loop uses std::time::Instant (wall clock), so advance real time past
    // the 2s lifetime plus the 30s cleanup interval.
    tokio::time::sleep(Duration::from_secs(33)).await;

    let stats = server.stats().await;
    assert_eq!(
        stats.active_allocations, 0,
        "cleanup_loop should remove allocation after expiry + cleanup interval"
    );

    handle.abort();
}
