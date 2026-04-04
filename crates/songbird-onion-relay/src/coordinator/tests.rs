// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::config::{HolePunchConfig, default_stun_servers_fallback};
use super::core::HolePunchCoordinator;
use super::types::PunchResult;
use super::util::{rand_nonce, unix_epoch_millis_u64};
use crate::error::OnionRelayError;
use crate::signaling::{PeerInfo, SignalingMessage};
use std::time::Duration;

#[tokio::test]
async fn test_coordinator_creation() {
    let config = HolePunchConfig::default();
    let (coord, _tx, _rx) = HolePunchCoordinator::new("test-node".to_string(), config);
    assert_eq!(coord.my_node_id, "test-node");
}

#[tokio::test]
async fn test_peer_registration() {
    let config = HolePunchConfig::default();
    let (coord, _tx, _rx) = HolePunchCoordinator::new("test-node".to_string(), config);

    let peer = PeerInfo::new("peer-1".to_string(), "1.2.3.4:5678".parse().unwrap());
    coord.register_peer(peer).await;

    assert!(coord.peers.read().await.contains_key("peer-1"));
}

#[test]
fn hole_punch_config_default_has_stun_servers() {
    let c = HolePunchConfig::default();
    assert!(!c.stun_servers.is_empty());
    assert_eq!(c.max_attempts, 20);
    assert!(c.total_timeout >= c.attempt_timeout);
    assert!(c.ack_timeout.as_secs() >= 1);
}

#[test]
fn hole_punch_config_with_stun_servers_override() {
    let c = HolePunchConfig::default().with_stun_servers(vec!["stun.example:3478".into()]);
    assert_eq!(c.stun_servers, vec!["stun.example:3478".to_string()]);
}

#[test]
fn default_stun_servers_fallback_lists_sovereign_servers() {
    let v = default_stun_servers_fallback();
    assert!(!v.is_empty());
    assert!(
        v.iter().any(|s| s.contains("nextcloud") || s.contains("cloudflare")),
        "fallback list should contain sovereign STUN servers"
    );
    assert!(
        !v.iter().any(|s| s.contains("google")),
        "fallback list must not contain Google STUN servers"
    );
}

#[test]
fn unix_epoch_millis_u64_ok() {
    let m = unix_epoch_millis_u64().unwrap();
    assert!(m > 1_000_000_000);
}

#[test]
fn rand_nonce_is_sixteen_bytes() {
    let n = rand_nonce();
    assert_eq!(n.len(), 16);
}

#[tokio::test]
async fn handle_message_register_stores_peer() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let p = PeerInfo::new("remote".into(), "5.5.5.5:5".parse().unwrap());
    let msg = SignalingMessage::Register {
        peer_info: p.clone(),
        encrypted_beacon: None,
    };
    assert!(coord.handle_message(msg).await.is_none());
    assert_eq!(coord.peers.read().await.get("remote").map(|x| x.public_addr), Some(p.public_addr));
}

#[tokio::test]
async fn handle_message_query_returns_peer_or_none() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let p = PeerInfo::new("bob".into(), "6.6.6.6:6".parse().unwrap());
    coord.register_peer(p).await;

    let q = SignalingMessage::Query {
        target_node_id: "bob".into(),
    };
    let resp = coord.handle_message(q).await.expect("query response");
    match resp {
        SignalingMessage::PeerInfoResponse {
            peer_info,
        } => {
            assert!(peer_info.is_some());
        }
        other => panic!("expected PeerInfoResponse, got {other:?}"),
    }

    let q2 = SignalingMessage::Query {
        target_node_id: "missing".into(),
    };
    let resp2 = coord.handle_message(q2).await.expect("query response");
    match resp2 {
        SignalingMessage::PeerInfoResponse {
            peer_info,
        } => assert!(peer_info.is_none()),
        other => panic!("expected PeerInfoResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_message_punch_request_to_other_node_returns_none() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let from = PeerInfo::new("a".into(), "1.1.1.1:1".parse().unwrap());
    let msg = SignalingMessage::PunchRequest {
        from,
        to_node_id: "not-me".into(),
        nonce: [1u8; 16],
    };
    assert!(coord.handle_message(msg).await.is_none());
}

#[tokio::test]
async fn handle_message_punch_request_to_self_without_my_info_returns_none() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("victim".into(), HolePunchConfig::default());
    let from = PeerInfo::new("att".into(), "2.2.2.2:2".parse().unwrap());
    let msg = SignalingMessage::PunchRequest {
        from,
        to_node_id: "victim".into(),
        nonce: [2u8; 16],
    };
    assert!(coord.handle_message(msg).await.is_none());
}

#[tokio::test]
async fn handle_message_punch_request_to_self_with_my_info_returns_ack() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("victim".into(), HolePunchConfig::default());
    *coord.my_info.write().await =
        Some(PeerInfo::new("victim".into(), "3.3.3.3:3".parse().unwrap()));

    let from = PeerInfo::new("peer".into(), "4.4.4.4:4".parse().unwrap());
    let nonce = [7u8; 16];
    let msg = SignalingMessage::PunchRequest {
        from: from.clone(),
        to_node_id: "victim".into(),
        nonce,
    };
    let ack = coord.handle_message(msg).await.expect("punch ack");
    match ack {
        SignalingMessage::PunchAck {
            from: ack_from,
            nonce: n,
            start_at_ms,
        } => {
            assert_eq!(ack_from.node_id, "victim");
            assert_eq!(n, nonce);
            assert!(start_at_ms > 0, "coordinated start time should be set");
        }
        other => panic!("expected PunchAck, got {other:?}"),
    }
}

#[tokio::test]
async fn handle_message_heartbeat_updates_timestamp() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let mut peer = PeerInfo::new("beat".into(), "9.9.9.9:9".parse().unwrap());
    peer.timestamp = std::time::SystemTime::UNIX_EPOCH;
    coord.register_peer(peer).await;

    let hb = SignalingMessage::Heartbeat {
        node_id: "beat".into(),
    };
    assert!(coord.handle_message(hb).await.is_none());

    let ts = coord.peers.read().await.get("beat").expect("peer").timestamp;
    assert!(ts > std::time::SystemTime::UNIX_EPOCH, "heartbeat should refresh timestamp");
}

#[tokio::test]
async fn handle_message_unknown_variant_returns_none() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let msg = SignalingMessage::Error {
        code: 1,
        message: "x".into(),
    };
    assert!(coord.handle_message(msg).await.is_none());
}

#[tokio::test]
async fn punch_to_peer_errors_without_my_info() {
    let config = HolePunchConfig::default();
    let (coord, _in_tx, _out_rx) = HolePunchCoordinator::new("me".into(), config);
    coord.register_peer(PeerInfo::new("p".into(), "8.8.8.8:8".parse().unwrap())).await;

    let err = coord.punch_to_peer("p").await.expect_err("must discover address first");
    assert!(matches!(err, crate::OnionRelayError::Other(_)), "expected Other, got {err:?}");
}

#[tokio::test]
async fn punch_to_peer_errors_when_peer_missing() {
    let config = HolePunchConfig::default();
    let (coord, _in_tx, _out_rx) = HolePunchCoordinator::new("me".into(), config);
    *coord.my_info.write().await = Some(PeerInfo::new("me".into(), "127.0.0.1:1".parse().unwrap()));

    let err = coord.punch_to_peer("ghost").await.expect_err("unknown peer");
    assert!(matches!(err, crate::OnionRelayError::PeerNotFound(_)));
}

#[tokio::test(start_paused = true)]
async fn punch_to_peer_signaling_timeout_when_no_ack() {
    let config = HolePunchConfig {
        max_attempts: 1,
        attempt_timeout: Duration::from_millis(0),
        packet_interval: Duration::ZERO,
        ack_timeout: Duration::from_millis(0),
        ..Default::default()
    };

    let (coord, _in_tx, mut out_rx) = HolePunchCoordinator::new("me".into(), config);
    *coord.my_info.write().await = Some(PeerInfo::new("me".into(), "127.0.0.1:2".parse().unwrap()));
    coord.register_peer(PeerInfo::new("peer".into(), "127.0.0.1:3".parse().unwrap())).await;

    let err = coord.punch_to_peer("peer").await.expect_err("no ack");
    assert!(
        matches!(err, crate::OnionRelayError::SignalingTimeout),
        "expected SignalingTimeout, got {err:?}"
    );
    let _drain = out_rx.recv().await;
}

#[tokio::test]
async fn punch_to_peer_relay_fallback_when_udp_unanswered() {
    let config = HolePunchConfig {
        max_attempts: 2,
        attempt_timeout: Duration::from_millis(0),
        packet_interval: Duration::ZERO,
        ack_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let (coord, inbound_tx, mut outbound_rx) = HolePunchCoordinator::new("me".into(), config);
    *coord.my_info.write().await = Some(PeerInfo::new("me".into(), "127.0.0.1:2".parse().unwrap()));
    coord.register_peer(PeerInfo::new("peer".into(), "127.0.0.1:9".parse().unwrap())).await;

    let helper = tokio::spawn(async move {
        let msg = outbound_rx.recv().await.expect("outbound punch request");
        let nonce = match msg {
            SignalingMessage::PunchRequest {
                nonce,
                ..
            } => nonce,
            other => panic!("expected PunchRequest, got {other:?}"),
        };
        let start_at_ms = unix_epoch_millis_u64().unwrap().saturating_add(100);
        inbound_tx
            .send(SignalingMessage::PunchAck {
                from: PeerInfo::new("peer".into(), "127.0.0.1:9".parse().unwrap()),
                nonce,
                start_at_ms,
            })
            .await
            .expect("deliver ack");
    });

    let result = coord.punch_to_peer("peer").await.expect("punch completes");
    helper.await.expect("helper join");

    assert!(
        matches!(
            result,
            PunchResult::Relay {
                attempts: 2
            }
        ),
        "expected relay fallback when peer UDP is silent, got {result:?}"
    );
}

#[tokio::test]
async fn discover_public_address_fails_when_stun_server_list_empty() {
    let config = HolePunchConfig {
        stun_servers: vec![],
        ..HolePunchConfig::default()
    };
    let (coord, _in_tx, _out_rx) = HolePunchCoordinator::new("me".into(), config);
    let err = coord
        .discover_public_address()
        .await
        .expect_err("no STUN servers means discovery cannot succeed");
    assert!(matches!(err, OnionRelayError::StunFailed(_)), "expected StunFailed, got {err:?}");
}

#[tokio::test]
async fn handle_message_heartbeat_for_unknown_peer_is_noop() {
    let (coord, _in_tx, _out_rx) =
        HolePunchCoordinator::new("me".into(), HolePunchConfig::default());
    let hb = SignalingMessage::Heartbeat {
        node_id: "not_registered".into(),
    };
    assert!(coord.handle_message(hb).await.is_none());
    assert!(coord.peers.read().await.get("not_registered").is_none());
}

#[tokio::test]
async fn punch_to_peer_fails_when_outbound_signaling_channel_closed() {
    let config = HolePunchConfig::default();
    let (coord, inbound_tx, outbound_rx) = HolePunchCoordinator::new("me".into(), config);
    drop(outbound_rx);
    *coord.my_info.write().await = Some(PeerInfo::new("me".into(), "127.0.0.1:1".parse().unwrap()));
    coord.register_peer(PeerInfo::new("peer".into(), "127.0.0.1:2".parse().unwrap())).await;

    let err = coord.punch_to_peer("peer").await.expect_err("signal_tx closed");
    assert!(
        matches!(err, OnionRelayError::Transport(_)),
        "expected Transport when signaling send fails, got {err:?}"
    );
    drop(inbound_tx);
}
