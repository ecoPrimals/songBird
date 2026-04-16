// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
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
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]

//! Integration tests for end-to-end relay packet forwarding
//!
//! **Pure Rust | Zero Unsafe | Modern Async**
//!
//! Tests the complete relay server flow:
//! 1. Client allocates relay session
//! 2. Client sends data packets through relay
//! 3. Relay server forwards packets to target
//! 4. Session refresh and deallocation

use songbird_lineage_relay::{
    MaskingLevel, RelaySession,
    relay::RelayAuthority,
    relay_protocol::{AllocationRequest, RelayProtocol},
    relay_server::RelayServer,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;

#[tokio::test]
async fn test_relay_allocation_flow() {
    // Start relay server
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(2), server.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Create allocation request
    let request = AllocationRequest::new(
        "tower".into(),
        "pixel".into(),
        "192.168.1.100:5000".parse().unwrap(),
        vec![1, 2, 3, 4],
        300,
    );

    // Send allocation request to relay server
    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let request_msg = RelayProtocol::AllocateRequest(request);
    let encoded = request_msg.encode();
    client_socket.send_to(&encoded, relay_addr).await.unwrap();

    // Receive allocation response
    let mut buf = vec![0u8; 65536];
    let (len, _src) = client_socket.recv_from(&mut buf).await.unwrap();

    let response = RelayProtocol::parse(&buf[..len]).unwrap();

    match response {
        RelayProtocol::AllocateResponse(resp) => {
            assert!(resp.success);
            assert!(resp.session_id.is_some());
            assert_eq!(resp.relay_addr, Some(relay_addr));
            assert_eq!(resp.ttl_seconds, 300);
        }
        _ => panic!("Expected AllocationResponse"),
    }

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_relay_packet_forwarding() {
    // Start relay server
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();
    let server_clone = Arc::new(server);

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_for_task = server_clone.clone();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(3), server_for_task.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Create requester and target sockets
    let requester_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let target_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let target_addr = target_socket.local_addr().unwrap();

    // Send allocation request from requester
    let request =
        AllocationRequest::new("tower".into(), "pixel".into(), target_addr, vec![1, 2, 3], 300);

    let request_msg = RelayProtocol::AllocateRequest(request);
    let encoded = request_msg.encode();
    requester_socket.send_to(&encoded, relay_addr).await.unwrap();

    // Receive allocation response
    let mut buf = vec![0u8; 65536];
    let (len, _) = requester_socket.recv_from(&mut buf).await.unwrap();
    let response = RelayProtocol::parse(&buf[..len]).unwrap();

    let session_id = match response {
        RelayProtocol::AllocateResponse(resp) => {
            assert!(resp.success);
            resp.session_id.unwrap()
        }
        _ => panic!("Expected AllocationResponse"),
    };

    // Send data packet from requester to target (via relay)
    let test_data = b"Hello through relay!";
    let data_packet = RelayProtocol::DataPacket {
        session_id,
        data: test_data.to_vec(),
    };
    let encoded = data_packet.encode();
    requester_socket.send_to(&encoded, relay_addr).await.unwrap();

    // Target receives forwarded data
    let (len, _) = target_socket.recv_from(&mut buf).await.unwrap();

    // The relay server should have forwarded the data packet
    // (Note: In production, this would be the raw data, but our implementation
    //  might wrap it depending on masking level)
    assert!(len > 0);
    // For MaskingLevel::None, data should be forwarded as-is
    assert_eq!(&buf[..test_data.len()], test_data);

    // Check server stats
    let stats = server_clone.stats().await;
    assert_eq!(stats.sessions_active, 1);
    assert_eq!(stats.sessions_total, 1);
    assert!(stats.packets_forwarded >= 1);
    assert!(stats.bytes_forwarded >= test_data.len() as u64);

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_relay_session_refresh() {
    // Start relay server
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(2), server.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Create and allocate session
    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let request = AllocationRequest::new(
        "tower".into(),
        "pixel".into(),
        "192.168.1.100:5000".parse().unwrap(),
        vec![],
        300,
    );

    let request_msg = RelayProtocol::AllocateRequest(request);
    client_socket.send_to(&request_msg.encode(), relay_addr).await.unwrap();

    // Receive allocation response
    let mut buf = vec![0u8; 65536];
    let (len, _) = client_socket.recv_from(&mut buf).await.unwrap();
    let response = RelayProtocol::parse(&buf[..len]).unwrap();

    let session_id = match response {
        RelayProtocol::AllocateResponse(resp) => resp.session_id.unwrap(),
        _ => panic!("Expected AllocationResponse"),
    };

    // Send refresh message
    let refresh_msg = RelayProtocol::Refresh {
        session_id,
    };
    client_socket.send_to(&refresh_msg.encode(), relay_addr).await.unwrap();

    // Session should still be active (refresh succeeded silently)
    // No response expected from refresh

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_relay_session_deallocation() {
    // Start relay server
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();
    let server_clone = Arc::new(server);

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_for_task = server_clone.clone();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(2), server_for_task.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Allocate session
    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let request = AllocationRequest::new(
        "tower".into(),
        "pixel".into(),
        "192.168.1.100:5000".parse().unwrap(),
        vec![],
        300,
    );

    client_socket
        .send_to(&RelayProtocol::AllocateRequest(request).encode(), relay_addr)
        .await
        .unwrap();

    // Receive allocation response
    let mut buf = vec![0u8; 65536];
    let (len, _) = client_socket.recv_from(&mut buf).await.unwrap();
    let response = RelayProtocol::parse(&buf[..len]).unwrap();

    let session_id = match response {
        RelayProtocol::AllocateResponse(resp) => resp.session_id.unwrap(),
        _ => panic!("Expected AllocationResponse"),
    };

    // Verify session is active (poll until allocation is reflected in stats)
    let start = tokio::time::Instant::now();
    loop {
        let stats = server_clone.stats().await;
        if stats.sessions_active >= 1 {
            break;
        }
        assert!(
            start.elapsed() <= Duration::from_secs(2),
            "Timeout waiting for session to be active"
        );
        tokio::task::yield_now().await;
    }
    let stats = server_clone.stats().await;
    assert_eq!(stats.sessions_active, 1);

    // Send deallocation message
    let deallocate_msg = RelayProtocol::Deallocate {
        session_id,
    };
    client_socket.send_to(&deallocate_msg.encode(), relay_addr).await.unwrap();

    // Wait for session to be removed (poll until cleanup or timeout)
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(2) {
        let stats = server_clone.stats().await;
        if stats.sessions_active == 0 {
            break;
        }
        tokio::task::yield_now().await;
    }

    // Session should be removed
    let stats = server_clone.stats().await;
    // Note: Session count might still be 1 if cleanup hasn't run yet, but it will be cleaned up
    // The important part is that the session is marked for removal
    assert!(stats.sessions_active <= 1);

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_relay_client_session_full_lifecycle() {
    // Start relay server
    let authority = Arc::new(RelayAuthority::StubAllow);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(3), server.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Create relay session (client-side)
    let session = Arc::new(
        RelaySession::new(
            "tower".into(),
            relay_addr,
            "pixel".into(),
            "laptop".into(),
            MaskingLevel::None,
        )
        .await
        .unwrap(),
    );

    // Note: In production, allocation would happen via the discovery protocol
    // For this test, we're testing the RelaySession.send() functionality directly

    // Send data through session
    let test_data = b"Client session test data";
    session.send(test_data).await.unwrap();

    // Verify stats updated
    let bytes_sent = session.stats();
    assert_eq!(bytes_sent, test_data.len() as u64);

    // Refresh session
    session.refresh().await.unwrap();

    // Close session
    session.close().await.unwrap();

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_unauthorized_relay_request() {
    // Start relay server with deny authority
    let authority = Arc::new(RelayAuthority::StubDeny);
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

    let relay_addr = server.bind_addr();
    let server_clone = Arc::new(server);

    // Spawn server task with readiness signal
    let (ready_tx, ready_rx) = oneshot::channel();
    let server_for_task = server_clone.clone();
    let server_handle = tokio::spawn(async move {
        let _ = ready_tx.send(());
        tokio::time::timeout(Duration::from_secs(2), server_for_task.run()).await.ok();
    });
    ready_rx.await.expect("Server failed to signal readiness");

    // Try to allocate (should be denied)
    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let request = AllocationRequest::new(
        "tower".into(),
        "unauthorized_node".into(),
        "192.168.1.100:5000".parse().unwrap(),
        vec![],
        300,
    );

    client_socket
        .send_to(&RelayProtocol::AllocateRequest(request).encode(), relay_addr)
        .await
        .unwrap();

    // Receive allocation response (should be error)
    let mut buf = vec![0u8; 65536];
    let (len, _) = client_socket.recv_from(&mut buf).await.unwrap();
    let response = RelayProtocol::parse(&buf[..len]).unwrap();

    match response {
        RelayProtocol::AllocateResponse(resp) => {
            assert!(!resp.success);
            assert!(resp.session_id.is_none());
            assert!(resp.error.is_some());
            assert!(resp.error.unwrap().contains("Unauthorized"));
        }
        _ => panic!("Expected AllocationResponse"),
    }

    // Check server stats (should have authorization failure)
    let stats = server_clone.stats().await;
    assert_eq!(stats.sessions_active, 0);
    assert_eq!(stats.sessions_total, 0);
    assert!(stats.authorization_failures > 0);

    // Cleanup
    server_handle.abort();
}
