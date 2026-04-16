// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::DiscoveryHandler;
use super::content::{ContentAnnouncement, ContentAnnouncementStore};
use super::types::{
    DiscoveredPeerInfo, DiscoveryGetPeerParams, DiscoveryPeersResult, PeerRegistry,
};
use crate::error::{IpcError, IpcResult};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Mock peer registry
// ============================================================================

pub struct MockPeerRegistry {
    pub peers: Vec<DiscoveredPeerInfo>,
}

#[async_trait::async_trait]
impl PeerRegistry for MockPeerRegistry {
    async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>> {
        Ok(self.peers.clone())
    }

    async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>> {
        Ok(self.peers.iter().find(|p| p.node_id == peer_id).cloned())
    }
}

// ============================================================================
// Handler tests
// ============================================================================

#[tokio::test]
async fn test_discovery_handler_creation() {
    let handler = DiscoveryHandler::new();
    assert!(handler.peer_registry.is_none());
}

#[tokio::test]
async fn discovery_handler_default_matches_new() {
    let a = DiscoveryHandler::new();
    let b = DiscoveryHandler::default();
    assert!(a.peer_registry.is_none() && b.peer_registry.is_none());
}

#[tokio::test]
async fn test_handle_list_peers_no_registry() {
    let handler = DiscoveryHandler::new();
    let result = handler.handle_list_peers(json!({})).await;

    assert!(result.is_ok());
    let peers_result = result.unwrap();
    assert_eq!(peers_result.total_count, 0);
    assert!(peers_result.peers.is_empty());
}

#[tokio::test]
async fn test_handle_list_peers_with_mock_registry() {
    let mock_peers = vec![
        DiscoveredPeerInfo {
            node_id: "node-alpha".to_string(),
            family_id: "nat0".to_string(),
            address: "192.168.1.100:2300".to_string(),
            tcp_port: Some(8081),
            capabilities: vec!["crypto".to_string(), "tls".to_string()],
            last_seen: "2026-01-29T00:00:00Z".to_string(),
            quality: Some(0.95),
            node_name: Some("alpha-tower".to_string()),
            protocols: vec!["birdsong".to_string()],
        },
        DiscoveredPeerInfo {
            node_id: "node-beta".to_string(),
            family_id: "nat0".to_string(),
            address: "192.168.1.101:2300".to_string(),
            tcp_port: Some(8082),
            capabilities: vec!["crypto".to_string()],
            last_seen: "2026-01-29T00:01:00Z".to_string(),
            quality: Some(0.88),
            node_name: Some("beta-tower".to_string()),
            protocols: vec!["birdsong".to_string()],
        },
    ];

    let registry = Arc::new(MockPeerRegistry {
        peers: mock_peers.clone(),
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result = handler.handle_list_peers(json!({})).await;

    assert!(result.is_ok());
    let peers_result = result.unwrap();
    assert_eq!(peers_result.total_count, 2);
    assert_eq!(peers_result.peers[0].node_id, "node-alpha");
    assert_eq!(peers_result.peers[1].node_id, "node-beta");
}

#[tokio::test]
async fn test_handle_get_peer_by_id() {
    let mock_peers = vec![DiscoveredPeerInfo {
        node_id: "node-gamma".to_string(),
        family_id: "nat0".to_string(),
        address: "192.0.2.10:2300".to_string(),
        tcp_port: Some(8082),
        capabilities: vec!["crypto".to_string(), "tls".to_string()],
        last_seen: "2026-01-29T02:26:00Z".to_string(),
        quality: Some(0.95),
        node_name: Some("gamma-tower".to_string()),
        protocols: vec!["birdsong".to_string(), "tarpc".to_string()],
    }];

    let registry = Arc::new(MockPeerRegistry {
        peers: mock_peers,
    });
    let handler = DiscoveryHandler::with_registry(registry);

    let params = json!({"peer_id": "node-gamma"});
    let peer = handler.handle_get_peer(params).await.unwrap().unwrap();
    assert_eq!(peer.node_id, "node-gamma");
    assert_eq!(peer.tcp_port, Some(8082));

    let params = json!({"peer_id": "node-nonexistent"});
    assert!(handler.handle_get_peer(params).await.unwrap().is_none());
}

#[tokio::test]
async fn test_get_peer_params_parsing() {
    let params = json!({"peer_id": "node-test"});
    let parsed: DiscoveryGetPeerParams = serde_json::from_value(params).expect("Should parse");
    assert_eq!(parsed.peer_id, "node-test");
}

#[tokio::test]
async fn test_discovered_peer_info_serialization() {
    let peer = DiscoveredPeerInfo {
        node_id: "test-node".to_string(),
        family_id: "test-family".to_string(),
        address: "127.0.0.1:2300".to_string(),
        tcp_port: Some(8080),
        capabilities: vec!["test".to_string()],
        last_seen: "2026-01-29T00:00:00Z".to_string(),
        quality: Some(0.99),
        node_name: Some("test-tower".to_string()),
        protocols: vec!["test-protocol".to_string()],
    };

    let json = serde_json::to_value(&peer).expect("Should serialize");
    assert_eq!(json["node_id"], "test-node");
    assert_eq!(json["family_id"], "test-family");
    assert_eq!(json["tcp_port"], 8080);
}

#[test]
fn discovery_peers_result_serialization_shape() {
    let r = DiscoveryPeersResult {
        peers: vec![],
        total_count: 0,
    };
    let v = serde_json::to_value(&r).expect("json");
    assert_eq!(v["total_count"], 0);
    assert!(v["peers"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn handle_get_peer_malformed_params_errors() {
    let handler = DiscoveryHandler::new();
    let err =
        handler.handle_get_peer(json!({ "peer_id": 12345 })).await.expect_err("type mismatch");
    assert!(matches!(err, IpcError::InvalidParams(_)));
}

#[tokio::test]
async fn handle_get_peer_no_registry_returns_none_when_valid() {
    let handler = DiscoveryHandler::new();
    let r = handler.handle_get_peer(json!({ "peer_id": "any-id" })).await.expect("ok");
    assert!(r.is_none());
}

#[tokio::test]
async fn handle_announce_null_family_defaults_unknown() {
    let handler = DiscoveryHandler::new();
    let v = handler
        .handle_announce(json!({ "family_id": null, "capabilities": [] }))
        .await
        .expect("announce");
    assert_eq!(v["family_id"], "unknown");
    assert_eq!(v["announced"], true);
    assert_eq!(v["mode"], "presence");
}

#[tokio::test]
async fn handle_announce_capabilities_skips_non_strings() {
    let handler = DiscoveryHandler::new();
    let v = handler
        .handle_announce(json!({
            "family_id": "fam",
            "capabilities": ["a", 2, "b", null, "c"]
        }))
        .await
        .expect("announce");
    let caps = v["capabilities"].as_array().expect("caps array");
    let strs: Vec<&str> = caps.iter().filter_map(|x| x.as_str()).collect();
    assert_eq!(strs, vec!["a", "b", "c"]);
}

#[tokio::test]
async fn handle_announce_topic_mode_content_distribution() {
    let handler = DiscoveryHandler::new();
    let v = handler
        .handle_announce(json!({
            "family_id": "nat0",
            "node_id": "seeder-alpha",
            "topic": "content:ludospring:assets",
            "manifest_hash": "blake3:abc123",
            "seeder_count": 3,
            "bond_types_accepted": ["data_bond", "compute_bond"]
        }))
        .await
        .expect("topic announce");
    assert_eq!(v["announced"], true);
    assert_eq!(v["mode"], "topic");
    assert_eq!(v["topic"], "content:ludospring:assets");
    assert_eq!(v["manifest_hash"], "blake3:abc123");
    assert_eq!(v["seeder_count"], 3);
    assert_eq!(v["node_id"], "seeder-alpha");
    assert_eq!(v["bond_types_accepted"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn handle_announce_topic_minimal_fields() {
    let handler = DiscoveryHandler::new();
    let v = handler
        .handle_announce(json!({ "topic": "content:docs" }))
        .await
        .expect("minimal topic announce");
    assert_eq!(v["mode"], "topic");
    assert_eq!(v["topic"], "content:docs");
    assert!(v["manifest_hash"].is_null());
    assert_eq!(v["seeder_count"], 1, "defaults to 1 when omitted");
    assert_eq!(v["bond_types_accepted"].as_array().unwrap().len(), 0);
}

// ============================================================================
// Capability / family filtering tests
// ============================================================================

fn peer(id: &str, family: &str, caps: &[&str]) -> DiscoveredPeerInfo {
    DiscoveredPeerInfo {
        node_id: id.to_string(),
        family_id: family.to_string(),
        address: "10.0.0.1:3492".to_string(),
        tcp_port: None,
        capabilities: caps.iter().map(|s| (*s).to_string()).collect(),
        last_seen: "2026-04-16T00:00:00Z".to_string(),
        quality: None,
        node_name: None,
        protocols: vec![],
    }
}

#[tokio::test]
async fn list_peers_capability_filter_narrows_results() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![
            peer("seeder-1", "nat0", &["content_seeder", "crypto"]),
            peer("plain-node", "nat0", &["crypto"]),
        ],
    });
    let handler = DiscoveryHandler::with_registry(registry);

    let result = handler
        .handle_list_peers(json!({ "capability_filter": ["content_seeder"] }))
        .await
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.peers[0].node_id, "seeder-1");
}

#[tokio::test]
async fn list_peers_no_filter_returns_all() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![peer("a", "fam", &[]), peer("b", "fam", &[])],
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result = handler.handle_list_peers(json!({})).await.unwrap();
    assert_eq!(result.total_count, 2);
}

#[tokio::test]
async fn list_peers_capability_filter_string_form() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![
            peer("storage-node", "nat0", &["storage"]),
            peer("compute-node", "nat0", &["compute"]),
        ],
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result =
        handler.handle_list_peers(json!({ "capability_filter": "storage" })).await.unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.peers[0].node_id, "storage-node");
}

#[tokio::test]
async fn list_peers_multiple_capability_filters_require_all() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![
            peer("full", "fam", &["content_seeder", "crypto", "tls"]),
            peer("partial", "fam", &["content_seeder"]),
        ],
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result = handler
        .handle_list_peers(json!({ "capability_filter": ["content_seeder", "tls"] }))
        .await
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.peers[0].node_id, "full");
}

#[tokio::test]
async fn list_peers_family_only_filters_by_env() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![
            peer("same-fam", "nat0", &["storage"]),
            peer("other-fam", "other", &["storage"]),
        ],
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result = handler
        .handle_list_peers_with(json!({ "family_only": true }), || Ok("nat0".to_string()))
        .await
        .unwrap();
    assert_eq!(result.total_count, 1);
    assert_eq!(result.peers[0].node_id, "same-fam");
}

#[tokio::test]
async fn list_peers_family_only_no_env_returns_all() {
    let registry = Arc::new(MockPeerRegistry {
        peers: vec![peer("a", "fam1", &[]), peer("b", "fam2", &[])],
    });
    let handler = DiscoveryHandler::with_registry(registry);
    let result = handler
        .handle_list_peers_with(json!({ "family_only": true }), || {
            Err(std::env::VarError::NotPresent)
        })
        .await
        .unwrap();
    assert_eq!(result.total_count, 2, "no FAMILY_ID means no filtering");
}

// ============================================================================
// Content distribution tests
// ============================================================================

#[tokio::test]
async fn announce_topic_stores_in_content_registry() {
    let handler = DiscoveryHandler::new();
    handler
        .handle_announce(json!({
            "topic": "content:ludospring:assets",
            "node_id": "seeder-1",
            "family_id": "nat0",
            "manifest_hash": "blake3:deadbeef",
            "seeder_count": 2,
        }))
        .await
        .expect("announce");

    let store = handler.content_announcements.read().await;
    assert_eq!(store.len(), 1);
    let results = store.query("content:ludospring:assets");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].node_id, "seeder-1");
    assert_eq!(results[0].manifest_hash.as_deref(), Some("blake3:deadbeef"));
}

#[tokio::test]
async fn announce_topic_updates_existing_entry() {
    let handler = DiscoveryHandler::new();
    handler
        .handle_announce(
            json!({ "topic": "content:docs", "node_id": "node-a", "manifest_hash": "blake3:v1" }),
        )
        .await
        .unwrap();
    handler
        .handle_announce(
            json!({ "topic": "content:docs", "node_id": "node-a", "manifest_hash": "blake3:v2" }),
        )
        .await
        .unwrap();

    let store = handler.content_announcements.read().await;
    assert_eq!(store.len(), 1, "same (topic, node_id) key should update");
    assert_eq!(store.query("content:docs")[0].manifest_hash.as_deref(), Some("blake3:v2"));
}

#[tokio::test]
async fn announce_presence_does_not_store_content() {
    let handler = DiscoveryHandler::new();
    handler
        .handle_announce(json!({ "family_id": "nat0", "capabilities": ["crypto"] }))
        .await
        .unwrap();
    let store = handler.content_announcements.read().await;
    assert_eq!(store.len(), 0, "presence mode should not create content entries");
}

#[tokio::test]
async fn content_peers_returns_matching_seeders() {
    let handler = DiscoveryHandler::new();
    for (topic, nid) in [
        ("content:assets", "seeder-1"),
        ("content:assets", "seeder-2"),
        ("content:other", "seeder-3"),
    ] {
        handler
            .handle_announce(json!({ "topic": topic, "node_id": nid, "family_id": "nat0", "manifest_hash": "blake3:aaa" }))
            .await
            .unwrap();
    }
    let result = handler.handle_content_peers(json!({ "topic": "content:assets" })).await.unwrap();
    assert_eq!(result["total_count"], 2);
    assert_eq!(result["seeders"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn content_peers_manifest_hash_filter() {
    let handler = DiscoveryHandler::new();
    handler
        .handle_announce(
            json!({ "topic": "content:data", "node_id": "s1", "manifest_hash": "blake3:v1" }),
        )
        .await
        .unwrap();
    handler
        .handle_announce(
            json!({ "topic": "content:data", "node_id": "s2", "manifest_hash": "blake3:v2" }),
        )
        .await
        .unwrap();

    let result = handler
        .handle_content_peers(json!({ "topic": "content:data", "manifest_hash": "blake3:v2" }))
        .await
        .unwrap();
    assert_eq!(result["total_count"], 1);
    assert_eq!(result["seeders"][0]["node_id"], "s2");
}

#[tokio::test]
async fn content_peers_family_only_filter() {
    let handler = DiscoveryHandler::new();
    handler
        .handle_announce(
            json!({ "topic": "content:shared", "node_id": "same-fam-seeder", "family_id": "nat0" }),
        )
        .await
        .unwrap();
    handler
        .handle_announce(json!({ "topic": "content:shared", "node_id": "other-fam-seeder", "family_id": "other-family" }))
        .await
        .unwrap();

    let result = handler
        .handle_content_peers_with(
            json!({ "topic": "content:shared", "family_only": true }),
            || Ok("nat0".to_string()),
        )
        .await
        .unwrap();
    assert_eq!(result["total_count"], 1);
    assert_eq!(result["seeders"][0]["node_id"], "same-fam-seeder");
}

#[tokio::test]
async fn content_peers_requires_topic() {
    let handler = DiscoveryHandler::new();
    let err = handler.handle_content_peers(json!({})).await.expect_err("missing topic");
    assert!(matches!(err, IpcError::InvalidParams(_)));
}

#[tokio::test]
async fn content_peers_empty_topic_returns_none() {
    let handler = DiscoveryHandler::new();
    handler.handle_announce(json!({ "topic": "content:stuff", "node_id": "s1" })).await.unwrap();
    let result =
        handler.handle_content_peers(json!({ "topic": "content:nonexistent" })).await.unwrap();
    assert_eq!(result["total_count"], 0);
    assert!(result["seeders"].as_array().unwrap().is_empty());
}

#[test]
fn content_announcement_store_gc_removes_expired() {
    let mut store = ContentAnnouncementStore::with_ttl(Duration::from_millis(0));
    store.insert(ContentAnnouncement {
        topic: "t".into(),
        manifest_hash: None,
        family_id: "f".into(),
        node_id: "n".into(),
        seeder_count: 1,
        bond_types_accepted: vec![],
        announced_at: Instant::now() - Duration::from_secs(1),
    });
    assert_eq!(store.len(), 1);
    store.gc();
    assert_eq!(store.len(), 0);
}

#[test]
fn content_announcement_store_query_excludes_expired() {
    let mut store = ContentAnnouncementStore::with_ttl(Duration::from_millis(0));
    store.insert(ContentAnnouncement {
        topic: "content:old".into(),
        manifest_hash: None,
        family_id: "f".into(),
        node_id: "n".into(),
        seeder_count: 1,
        bond_types_accepted: vec![],
        announced_at: Instant::now() - Duration::from_secs(1),
    });
    assert!(store.query("content:old").is_empty(), "expired entries filtered out");
}
