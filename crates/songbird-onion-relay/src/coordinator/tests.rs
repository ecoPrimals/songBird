// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::config::{HolePunchConfig, default_stun_servers_fallback};
use super::core::HolePunchCoordinator;
use super::util::{rand_nonce, unix_epoch_millis_u64};
use crate::signaling::PeerInfo;

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
