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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]

//! End-to-end tests for UDP multicast discovery
//!
//! Tests the full discovery flow with multicast and known peers

use songbird_discovery::anonymous::{
    AnonymousDiscoveryBroadcaster, AnonymousDiscoveryListener, AnonymousDiscoveryMessage,
    DiscoveredPeer, TransportEndpointMessage,
};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

/// Initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multicast_discovery_loopback() {
    init_tracing();

    // Test broadcaster and listener creation with multicast
    let capabilities = vec!["compute".to_string(), "storage".to_string()];
    let protocols = vec!["https".to_string()];
    let multicast_addr: SocketAddr = "224.0.0.251:2301".parse().unwrap();

    let _broadcaster = AnonymousDiscoveryBroadcaster::new(
        capabilities.clone(),
        protocols.clone(),
        8080,
        vec![multicast_addr],
        1, // Broadcast every second
    );

    let listener = AnonymousDiscoveryListener::new(2301, 60);
    assert!(
        listener.get_peers().await.is_empty(),
        "peer registry starts empty before any UDP traffic"
    );

    // Wire-format parity: what the broadcaster would emit validates and round-trips.
    let msg = AnonymousDiscoveryMessage::new(capabilities, protocols, 8080);
    msg.validate().expect("v2.1 discovery message should validate");
    let bytes = msg.to_bytes().expect("serialize");
    let roundtrip = AnonymousDiscoveryMessage::from_bytes(&bytes).expect("deserialize");
    assert_eq!(roundtrip.capabilities, msg.capabilities);
    assert_eq!(roundtrip.protocols, msg.protocols);

    let mut invalid = AnonymousDiscoveryMessage::new(vec!["x".into()], vec!["https".into()], 8080);
    invalid.capabilities.clear();
    assert!(invalid.validate().is_err(), "validation error path: empty capabilities");

    // Stale-peer TTL uses listener timeout semantics (60s): ancient last_seen is stale.
    let stale_peer = DiscoveredPeer {
        session_id: "sess".into(),
        node_id: None,
        node_name: None,
        endpoints: None,
        capabilities: vec!["compute".into()],
        tags: None,
        timestamp: None,
        identity_attestations: None,
        protocols: vec!["https".into()],
        port: 8080,
        address: "127.0.0.1:2301".parse().unwrap(),
        last_seen: SystemTime::UNIX_EPOCH,
        version: "2.1".into(),
    };
    assert!(stale_peer.is_stale(60));

    // Bind + multicast join path: listener runs until cancelled; timeout means setup succeeded.
    let outcome =
        tokio::time::timeout(Duration::from_millis(500), listener.start_listening()).await;
    match outcome {
        Ok(Err(e)) => panic!("listener failed to bind or join multicast: {e}"),
        Err(_elapsed) => {}
        Ok(Ok(())) => unreachable!("start_listening does not return after successful setup"),
    }
}

#[tokio::test]
async fn test_known_peers_discovery() {
    init_tracing();

    // Test that broadcaster supports known peers configuration
    let capabilities = vec!["orchestration".to_string()];
    let protocols = vec!["https".to_string()];
    let multicast_addr: SocketAddr = "224.0.0.251:2302".parse().unwrap();
    let known_peer1: SocketAddr = "192.168.1.100:2302".parse().unwrap();
    let known_peer2: SocketAddr = "192.168.1.101:2302".parse().unwrap();

    let _broadcaster = AnonymousDiscoveryBroadcaster::new(
        capabilities.clone(),
        protocols.clone(),
        8080,
        vec![multicast_addr],
        30,
    )
    .with_known_peers(vec![known_peer1, known_peer2]);

    let msg = AnonymousDiscoveryMessage::new(capabilities, protocols, 8080);
    msg.validate().expect("message matching broadcaster config should validate");
}

#[tokio::test]
async fn test_v3_multicast_discovery() {
    init_tracing();

    // Test v3.0 protocol with node identity and multiple endpoints
    let node_id = "test-node-123".to_string();
    let node_name = "test-tower".to_string();
    let endpoints = vec![TransportEndpointMessage {
        interface_type: "ethernet".to_string(),
        address: "192.168.1.100:8080".to_string(),
        protocols: vec!["https".to_string()],
        preference: 100,
    }];
    let capabilities = vec!["compute".to_string()];
    let multicast_addr: SocketAddr = "224.0.0.251:2303".parse().unwrap();

    let _broadcaster = AnonymousDiscoveryBroadcaster::new_v3(
        node_id.clone(),
        node_name.clone(),
        endpoints.clone(),
        capabilities.clone(),
        vec![multicast_addr],
        30,
    );

    let msg = AnonymousDiscoveryMessage::new_v3(node_id, node_name, endpoints, capabilities);
    msg.validate().expect("v3.0 discovery message should validate");
    assert_eq!(msg.version, "3.0");
    assert!(msg.node_id.is_some());
}

#[tokio::test]
async fn test_listener_multicast_join() {
    init_tracing();

    // Test that listener properly configures multicast
    let listener = AnonymousDiscoveryListener::new(2304, 60);
    assert!(listener.get_peers().await.is_empty());

    let outcome =
        tokio::time::timeout(Duration::from_millis(500), listener.start_listening()).await;
    match outcome {
        Ok(Err(e)) => panic!("multicast listener failed to start: {e}"),
        Err(_elapsed) => {}
        Ok(Ok(())) => unreachable!(),
    }
}

#[tokio::test]
async fn test_listener_broadcast_fallback() {
    init_tracing();

    // Test broadcast-only mode (no multicast)
    let listener = AnonymousDiscoveryListener::new_broadcast_only(2305, 60);
    assert!(listener.get_peers().await.is_empty());
}

#[tokio::test]
async fn test_hybrid_discovery_strategy() {
    init_tracing();

    // Test hybrid strategy: multicast + known peers + broadcast
    let capabilities = vec!["gpu-compute".to_string()];
    let protocols = vec!["https".to_string(), "tarpc-tls".to_string()];

    // Multicast address
    let multicast_addr: SocketAddr = "224.0.0.251:2306".parse().unwrap();

    // Broadcast address (fallback)
    let broadcast_addr: SocketAddr = "255.255.255.255:2306".parse().unwrap();

    // Known peers
    let known_peer: SocketAddr = "192.168.1.150:2306".parse().unwrap();

    let _broadcaster = AnonymousDiscoveryBroadcaster::new(
        capabilities.clone(),
        protocols.clone(),
        8080,
        vec![multicast_addr, broadcast_addr], // Both multicast and broadcast
        30,
    )
    .with_known_peers(vec![known_peer]);

    let msg = AnonymousDiscoveryMessage::new(capabilities, protocols, 8080);
    msg.validate().expect("hybrid broadcaster config should produce a valid wire message");
}
