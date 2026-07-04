// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use crate::discovery_mode::DiscoveryMode;
use crate::federation_config::{FederationConfig, NodeInfo};
use crate::state::{FederationStatus, NodeRegistration, NodeStatus};
use chrono::Utc;

fn sample_registration(node_id: &str, name: &str) -> NodeRegistration {
    NodeRegistration {
        node_id: node_id.to_string(),
        node_name: name.to_string(),
        node_address: String::from("192.168.0.1:8080"),
        endpoints: None,
        cpu_cores: 4,
        memory_gb: 8,
        gpu_model: None,
        storage_gb: None,
        capabilities: vec![],
        status: NodeStatus::Active,
        joined_at: Utc::now(),
        last_heartbeat: Utc::now(),
    }
}

#[test]
fn federation_config_default_serde_roundtrip() {
    let c = FederationConfig::default();
    let json = serde_json::to_string(&c).unwrap();
    let back: FederationConfig = serde_json::from_str(&json).unwrap();
    assert!(!back.enabled);
    assert_eq!(back.heartbeat_interval_secs, 30);
    assert_eq!(back.node_timeout_secs, 60);
}

#[test]
fn federation_config_with_discovery_mode_serializes() {
    let c = FederationConfig {
        discovery_mode: Some(DiscoveryMode::BirdSong),
        ..FederationConfig::default()
    };
    let json = serde_json::to_string(&c).unwrap();
    let back: FederationConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.discovery_mode, Some(DiscoveryMode::BirdSong));
}

#[test]
fn node_info_roundtrip() {
    let n = NodeInfo {
        node_id: "a".into(),
        address: "b".into(),
        status: "c".into(),
    };
    let json = serde_json::to_string(&n).unwrap();
    let back: NodeInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.node_id, "a");
}

#[test]
fn federation_config_default_heartbeat_and_timeout_match_helpers() {
    let c = FederationConfig::default();
    assert_eq!(c.heartbeat_interval_secs, 30);
    assert_eq!(c.node_timeout_secs, 60);
}

#[tokio::test]
async fn discovery_mode_plaintext_without_security_provider() {
    let coord = FederationCoordinator::new().await.unwrap();
    assert!(!coord.has_security_provider().await);
    assert_eq!(coord.discovery_mode().await, DiscoveryMode::Plaintext);
}

#[tokio::test]
async fn effective_discovery_mode_respects_override_and_fallback() {
    let coord = FederationCoordinator::new().await.unwrap();

    let auto = FederationConfig::default();
    assert_eq!(coord.effective_discovery_mode(&auto).await, DiscoveryMode::Plaintext);

    let forced_plain = FederationConfig {
        discovery_mode: Some(DiscoveryMode::Plaintext),
        ..FederationConfig::default()
    };
    assert_eq!(coord.effective_discovery_mode(&forced_plain).await, DiscoveryMode::Plaintext);

    let birdsong_without_security = FederationConfig {
        discovery_mode: Some(DiscoveryMode::BirdSong),
        ..FederationConfig::default()
    };
    assert_eq!(
        coord.effective_discovery_mode(&birdsong_without_security).await,
        DiscoveryMode::Plaintext
    );
}

#[tokio::test]
async fn coordinator_debug_formats_without_panicking() {
    let coord = FederationCoordinator::new().await.unwrap();
    let s = format!("{coord:?}");
    assert!(s.contains("FederationCoordinator"));
}

#[tokio::test]
async fn ingest_peers_from_federation_status_value() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("self-node", "Self");
    let peer = sample_registration("peer-a", "Peer A");

    let status = FederationStatus {
        federation_id: String::from("fed-1"),
        active_nodes: 2,
        nodes: vec![self_reg.clone(), peer.clone()],
        total_cpu_cores: 0,
        total_memory_gb: 0,
        total_storage_gb: 0,
        uptime_seconds: 0,
    };
    let v = serde_json::to_value(&status).unwrap();
    coord.ingest_peers_from_join_response(&v, &self_reg).await;

    let state = coord.state();
    let nodes = state.nodes.read().await;
    assert!(nodes.contains_key("peer-a"));
    assert!(!nodes.contains_key("self-node"));
}

#[tokio::test]
async fn ingest_peers_from_nodes_array_when_status_shape_unknown() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("node-self", "Self");
    let peer = sample_registration("node-peer", "Peer B");

    let v = serde_json::json!({
        "not_federation_status": true,
        "nodes": [ serde_json::to_value(&peer).unwrap() ],
    });
    coord.ingest_peers_from_join_response(&v, &self_reg).await;

    let state = coord.state();
    assert!(state.nodes.read().await.contains_key("node-peer"));
}

#[tokio::test]
async fn ingest_peers_from_peers_array() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("id-self", "Self");
    let peer = sample_registration("id-peer", "Peer C");

    let v = serde_json::json!({
        "peers": [ serde_json::to_value(&peer).unwrap() ],
    });
    coord.ingest_peers_from_join_response(&v, &self_reg).await;

    let state = coord.state();
    assert!(state.nodes.read().await.contains_key("id-peer"));
}

#[tokio::test]
async fn join_federation_errors_without_self_registration() {
    let coord = FederationCoordinator::new().await.unwrap();
    let config = FederationConfig {
        enabled: true,
        self_registration: None,
        ..FederationConfig::default()
    };

    let result = coord.join_federation("192.168.1.1:8080", &config).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("self registration"),
        "Expected 'self registration' in error, got: {err_str}"
    );
}

#[tokio::test]
async fn ingest_peers_skips_self_in_nodes_array() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("self-id", "Me");
    let peer = sample_registration("other-id", "Other");

    let v = serde_json::json!({
        "nodes": [
            serde_json::to_value(&self_reg).unwrap(),
            serde_json::to_value(&peer).unwrap(),
        ],
    });
    coord.ingest_peers_from_join_response(&v, &self_reg).await;

    let state = coord.state();
    let nodes = state.nodes.read().await;
    assert!(!nodes.contains_key("self-id"), "should skip self");
    assert!(nodes.contains_key("other-id"), "should ingest peer");
}

#[tokio::test]
async fn ingest_peers_handles_malformed_entries_gracefully() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("self-x", "Self");
    let good_peer = sample_registration("good-peer", "Good");

    let v = serde_json::json!({
        "nodes": [
            { "invalid": "not a NodeRegistration" },
            serde_json::to_value(&good_peer).unwrap(),
            "just a string",
        ],
    });
    coord.ingest_peers_from_join_response(&v, &self_reg).await;

    let state = coord.state();
    let nodes = state.nodes.read().await;
    assert!(
        nodes.contains_key("good-peer"),
        "valid peer should still be registered despite malformed siblings"
    );
}

#[tokio::test]
async fn ingest_peers_empty_response_no_panic() {
    let coord = FederationCoordinator::new().await.unwrap();
    let self_reg = sample_registration("s", "S");

    coord.ingest_peers_from_join_response(&serde_json::json!({}), &self_reg).await;
    coord.ingest_peers_from_join_response(&serde_json::json!(null), &self_reg).await;
    coord.ingest_peers_from_join_response(&serde_json::json!({"nodes": []}), &self_reg).await;
    coord.ingest_peers_from_join_response(&serde_json::json!({"peers": []}), &self_reg).await;

    let state = coord.state();
    assert_eq!(state.nodes.read().await.len(), 0);
}

#[tokio::test]
async fn discovery_mode_enum_serde_all_variants() {
    for mode in [DiscoveryMode::Plaintext, DiscoveryMode::BirdSong] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: DiscoveryMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, mode);
    }
}
