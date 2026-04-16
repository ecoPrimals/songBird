// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery JSON-RPC Handler
//!
//! Exposes peer discovery and content distribution functionality via JSON-RPC
//! for Dark Forest rendezvous protocol and the seeder/leecher pattern.
//!
//! ## Methods
//! - `discovery.peers` — List discovered peers (supports `family_only`, `capability_filter`)
//! - `discovery.announce` — Announce presence or content availability to the mesh
//! - `discovery.content_peers` — Find seeders for a specific content topic
//! - `discovery.get_peer` — Get specific peer by ID
//!
//! ## Content Distribution
//! `discovery.announce` with a `topic` param stores content announcements in an
//! in-memory registry with TTL-based expiration. Leechers query available content
//! via `discovery.content_peers` to find seeders for specific topics. Topics use
//! the `content:<namespace>` convention (e.g., `content:ludospring:assets`) per
//! `content_distribution_federation.toml`. Manifest hashes use BLAKE3 addressing
//! from `NestGate`'s `ContentManifest`.
//!
//! ## Security Note
//! Peer information includes network addresses. Only expose to trusted consumers.

use crate::error::{IpcError, IpcResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Default TTL for content announcements (10 minutes).
const CONTENT_ANNOUNCEMENT_TTL: Duration = Duration::from_secs(600);

// ============================================================================
// Discovery Handler
// ============================================================================

/// Discovery handler for peer discovery and content distribution operations.
///
/// Maintains both a peer registry (injected from orchestrator) and an in-memory
/// content announcement store for the seeder/leecher pattern defined by
/// `content_distribution_federation.toml`.
pub struct DiscoveryHandler {
    peer_registry: Option<Arc<dyn PeerRegistry>>,
    content_announcements: Arc<RwLock<ContentAnnouncementStore>>,
}

impl DiscoveryHandler {
    /// Create a new discovery handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            peer_registry: None,
            content_announcements: Arc::new(RwLock::new(ContentAnnouncementStore::new())),
        }
    }

    /// Create discovery handler with peer registry
    pub fn with_registry(registry: Arc<dyn PeerRegistry>) -> Self {
        Self {
            peer_registry: Some(registry),
            content_announcements: Arc::new(RwLock::new(ContentAnnouncementStore::new())),
        }
    }

    /// Handle `discovery.peers` JSON-RPC method
    ///
    /// Lists discovered peers, optionally filtered by family affinity or
    /// required capabilities. Used by content distribution federation to
    /// locate same-family seeders with specific capability profiles.
    pub async fn handle_list_peers(&self, params: Value) -> IpcResult<DiscoveryPeersResult> {
        self.handle_list_peers_with(params, || {
            songbird_process_env::var("FAMILY_ID")
                .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        })
        .await
    }

    /// Testable variant with injectable `family_id` resolver.
    async fn handle_list_peers_with<F>(
        &self,
        params: Value,
        resolve_family: F,
    ) -> IpcResult<DiscoveryPeersResult>
    where
        F: FnOnce() -> Result<String, std::env::VarError>,
    {
        let family_only = params.get("family_only").and_then(Value::as_bool).unwrap_or(false);

        let capability_filter: Vec<String> = match params.get("capability_filter") {
            Some(Value::Array(arr)) => {
                arr.iter().filter_map(Value::as_str).map(String::from).collect()
            }
            Some(Value::String(s)) => vec![s.clone()],
            _ => Vec::new(),
        };

        debug!(
            "Discovery: list_peers (family_only={family_only}, cap_filter={:?})",
            capability_filter,
        );

        let mut peers = if let Some(ref registry) = self.peer_registry {
            registry.get_all_peers().await?
        } else {
            Vec::new()
        };

        if family_only {
            let own_family = resolve_family().unwrap_or_default();
            if own_family.is_empty() {
                debug!(
                    "Discovery: family_only requested but no FAMILY_ID set — returning all peers"
                );
            } else {
                debug!("Discovery: family_only filter active (family={own_family})");
                peers.retain(|peer| peer.family_id == own_family);
            }
        }

        if !capability_filter.is_empty() {
            peers.retain(|peer| {
                capability_filter
                    .iter()
                    .all(|required| peer.capabilities.iter().any(|c| c == required))
            });
        }

        let total_count = peers.len();
        info!("Discovery: found {total_count} peers");

        Ok(DiscoveryPeersResult {
            peers,
            total_count,
        })
    }

    /// Handle `discovery.announce` JSON-RPC method
    ///
    /// Announces this node's presence or content availability to the mesh.
    ///
    /// Supports two announcement modes:
    /// - **Presence**: basic peer announcement with `family_id` + `capabilities`
    /// - **Topic**: content distribution announcement with `topic` (e.g.,
    ///   `content:ludospring:assets`) per `content_distribution_federation.toml`.
    ///   Topic announcements are stored in the content announcement registry
    ///   so peers can discover available content via `discovery.peers` with a
    ///   `topic` filter.
    pub async fn handle_announce(&self, params: Value) -> IpcResult<Value> {
        let family_id = params.get("family_id").and_then(Value::as_str).unwrap_or("unknown");
        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();
        let topic = params.get("topic").and_then(Value::as_str);
        let manifest_hash = params.get("manifest_hash").and_then(Value::as_str);
        let seeder_count = params.get("seeder_count").and_then(Value::as_u64);
        let bond_types: Vec<String> = params
            .get("bond_types_accepted")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();
        let node_id = params.get("node_id").and_then(Value::as_str).unwrap_or("unknown");

        if let Some(topic) = topic {
            let announcement = ContentAnnouncement {
                topic: topic.to_string(),
                manifest_hash: manifest_hash.map(String::from),
                family_id: family_id.to_string(),
                node_id: node_id.to_string(),
                seeder_count: seeder_count.unwrap_or(1),
                bond_types_accepted: bond_types.clone(),
                announced_at: Instant::now(),
            };

            let mut store = self.content_announcements.write().await;
            store.gc();
            store.insert(announcement);

            info!(
                "Discovery: content announce stored (topic={}, manifest={}, seeders={}, node={})",
                topic,
                manifest_hash.unwrap_or("none"),
                seeder_count.unwrap_or(1),
                node_id,
            );

            Ok(serde_json::json!({
                "announced": true,
                "mode": "topic",
                "topic": topic,
                "manifest_hash": manifest_hash,
                "seeder_count": seeder_count.unwrap_or(1),
                "bond_types_accepted": bond_types,
                "family_id": family_id,
                "node_id": node_id,
            }))
        } else {
            info!(
                "Discovery: presence announce (family={}, capabilities={})",
                family_id,
                capabilities.len()
            );
            Ok(serde_json::json!({
                "announced": true,
                "mode": "presence",
                "family_id": family_id,
                "capabilities": capabilities,
            }))
        }
    }

    /// Handle `discovery.content_peers` JSON-RPC method
    ///
    /// Returns peers that have announced content for a specific topic.
    /// Used by leechers to find seeders for content distribution.
    ///
    /// Params:
    /// - `topic` (required): content topic to query (e.g., `content:ludospring:assets`)
    /// - `family_only` (optional): restrict to same-family seeders
    /// - `manifest_hash` (optional): filter to a specific manifest version
    pub async fn handle_content_peers(&self, params: Value) -> IpcResult<Value> {
        self.handle_content_peers_with(params, || {
            songbird_process_env::var("FAMILY_ID")
                .or_else(|_| songbird_process_env::var("SONGBIRD_FAMILY_ID"))
        })
        .await
    }

    /// Testable variant with injectable `family_id` resolver.
    async fn handle_content_peers_with<F>(
        &self,
        params: Value,
        resolve_family: F,
    ) -> IpcResult<Value>
    where
        F: FnOnce() -> Result<String, std::env::VarError>,
    {
        let topic = params
            .get("topic")
            .and_then(Value::as_str)
            .ok_or_else(|| IpcError::InvalidParams("missing required 'topic' param".into()))?;
        let family_only = params.get("family_only").and_then(Value::as_bool).unwrap_or(false);
        let manifest_filter = params.get("manifest_hash").and_then(Value::as_str);

        let store = self.content_announcements.read().await;
        let mut announcements: Vec<&ContentAnnouncement> = store.query(topic);

        if let Some(hash) = manifest_filter {
            announcements.retain(|a| a.manifest_hash.as_deref() == Some(hash));
        }

        if family_only {
            let own_family = resolve_family().unwrap_or_default();
            if own_family.is_empty() {
                warn!("Discovery: family_only requested but no FAMILY_ID set");
            } else {
                announcements.retain(|a| a.family_id == own_family);
            }
        }

        let results: Vec<Value> = announcements
            .iter()
            .map(|a| {
                serde_json::json!({
                    "node_id": a.node_id,
                    "family_id": a.family_id,
                    "topic": a.topic,
                    "manifest_hash": a.manifest_hash,
                    "seeder_count": a.seeder_count,
                    "bond_types_accepted": a.bond_types_accepted,
                })
            })
            .collect();

        let total = results.len();
        info!("Discovery: content_peers query (topic={topic}) found {total} seeders");

        Ok(serde_json::json!({
            "seeders": results,
            "total_count": total,
            "topic": topic,
        }))
    }

    /// Handle `discovery.get_peer` JSON-RPC method
    ///
    /// Gets a specific peer by ID.
    pub async fn handle_get_peer(&self, params: Value) -> IpcResult<Option<DiscoveredPeerInfo>> {
        let params: DiscoveryGetPeerParams = serde_json::from_value(params)
            .map_err(|e| IpcError::InvalidParams(format!("Failed to parse params: {e}")))?;

        debug!("Discovery: get_peer (peer_id: {})", params.peer_id);

        if let Some(ref registry) = self.peer_registry {
            let peer = registry.get_peer(&params.peer_id).await?;
            Ok(peer)
        } else {
            Ok(None)
        }
    }
}

impl Default for DiscoveryHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Peer Registry Trait
// ============================================================================

/// Trait for peer registry (implemented by orchestrator)
#[async_trait::async_trait]
pub trait PeerRegistry: Send + Sync {
    /// Get all discovered peers
    async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>>;

    /// Get a specific peer by ID
    async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>>;
}

// ============================================================================
// Types
// ============================================================================

/// Parameters for `discovery.get_peer`
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryGetPeerParams {
    /// Peer ID (`node_id` or `session_id`)
    pub peer_id: String,
}

/// Result for `discovery.peers`
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryPeersResult {
    /// Discovered peers
    pub peers: Vec<DiscoveredPeerInfo>,

    /// Total count
    pub total_count: usize,
}

/// Discovered peer information (JSON-RPC compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeerInfo {
    /// Peer's node ID
    pub node_id: String,

    /// Peer's family ID (genetic lineage)
    pub family_id: String,

    /// IP:port from beacon
    pub address: String,

    /// TCP gateway port (if advertised)
    pub tcp_port: Option<u16>,

    /// Capabilities advertised
    pub capabilities: Vec<String>,

    /// Last seen timestamp (ISO 8601)
    pub last_seen: String,

    /// Signal quality / latency (0.0-1.0)
    pub quality: Option<f64>,

    /// Node name (human-readable)
    pub node_name: Option<String>,

    /// Protocols supported
    pub protocols: Vec<String>,
}

// ============================================================================
// Content Distribution Types
// ============================================================================

/// A content availability announcement from a seeder node.
///
/// Stored in the in-memory `ContentAnnouncementStore` when a peer calls
/// `discovery.announce` with a `topic` param. Queried via `discovery.content_peers`.
#[derive(Debug, Clone)]
pub struct ContentAnnouncement {
    /// Content topic (e.g., `content:ludospring:assets`)
    pub topic: String,
    /// BLAKE3 manifest hash (from `NestGate` `ContentManifest`)
    pub manifest_hash: Option<String>,
    /// Announcing peer's family ID
    pub family_id: String,
    /// Announcing peer's node ID
    pub node_id: String,
    /// Number of seeders the announcer knows about
    pub seeder_count: u64,
    /// Bond types accepted for this content
    pub bond_types_accepted: Vec<String>,
    /// When this announcement was received
    announced_at: Instant,
}

/// In-memory store for content announcements with TTL-based expiration.
///
/// Keyed by `(topic, node_id)` so a node can update its announcement for
/// a given topic by re-announcing.
#[derive(Debug)]
struct ContentAnnouncementStore {
    entries: HashMap<(String, String), ContentAnnouncement>,
    ttl: Duration,
}

impl ContentAnnouncementStore {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: CONTENT_ANNOUNCEMENT_TTL,
        }
    }

    #[cfg(test)]
    fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            ttl,
        }
    }

    fn insert(&mut self, announcement: ContentAnnouncement) {
        let key = (announcement.topic.clone(), announcement.node_id.clone());
        self.entries.insert(key, announcement);
    }

    fn query(&self, topic: &str) -> Vec<&ContentAnnouncement> {
        let now = Instant::now();
        self.entries
            .values()
            .filter(|a| a.topic == topic && now.duration_since(a.announced_at) < self.ttl)
            .collect()
    }

    /// Remove expired entries.
    fn gc(&mut self) {
        let now = Instant::now();
        let ttl = self.ttl;
        self.entries.retain(|_, a| now.duration_since(a.announced_at) < ttl);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

// ============================================================================
// Mock Registry for Testing
// ============================================================================

#[cfg(test)]
mod tests_support {
    use super::{DiscoveredPeerInfo, IpcResult, PeerRegistry};

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
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::tests_support::MockPeerRegistry;
    use super::*;
    use crate::error::IpcError;
    use serde_json::json;

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
        assert_eq!(peers_result.peers.len(), 2);
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
            peers: mock_peers.clone(),
        });
        let handler = DiscoveryHandler::with_registry(registry);

        // Test found peer
        let params = json!({"peer_id": "node-gamma"});
        let result = handler.handle_get_peer(params).await;

        assert!(result.is_ok());
        let peer = result.unwrap();
        assert!(peer.is_some());
        let peer = peer.unwrap();
        assert_eq!(peer.node_id, "node-gamma");
        assert_eq!(peer.tcp_port, Some(8082));

        // Test not found peer
        let params = json!({"peer_id": "node-nonexistent"});
        let result = handler.handle_get_peer(params).await;

        assert!(result.is_ok());
        let peer = result.unwrap();
        assert!(peer.is_none());
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
        let bonds = v["bond_types_accepted"].as_array().unwrap();
        assert_eq!(bonds.len(), 2);
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

    #[tokio::test]
    async fn list_peers_capability_filter_narrows_results() {
        let mock_peers = vec![
            DiscoveredPeerInfo {
                node_id: "seeder-1".to_string(),
                family_id: "nat0".to_string(),
                address: "10.0.0.1:2300".to_string(),
                tcp_port: Some(8080),
                capabilities: vec!["content_seeder".to_string(), "crypto".to_string()],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: Some(0.9),
                node_name: None,
                protocols: vec!["birdsong".to_string()],
            },
            DiscoveredPeerInfo {
                node_id: "plain-node".to_string(),
                family_id: "nat0".to_string(),
                address: "10.0.0.2:2300".to_string(),
                tcp_port: Some(8080),
                capabilities: vec!["crypto".to_string()],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: Some(0.8),
                node_name: None,
                protocols: vec!["birdsong".to_string()],
            },
        ];

        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
        });
        let handler = DiscoveryHandler::with_registry(registry);

        let result = handler
            .handle_list_peers(json!({
                "capability_filter": ["content_seeder"]
            }))
            .await
            .unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.peers[0].node_id, "seeder-1");
    }

    #[tokio::test]
    async fn list_peers_no_filter_returns_all() {
        let mock_peers = vec![
            DiscoveredPeerInfo {
                node_id: "a".to_string(),
                family_id: "fam".to_string(),
                address: "10.0.0.1:2300".to_string(),
                tcp_port: None,
                capabilities: vec![],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: None,
                node_name: None,
                protocols: vec![],
            },
            DiscoveredPeerInfo {
                node_id: "b".to_string(),
                family_id: "fam".to_string(),
                address: "10.0.0.2:2300".to_string(),
                tcp_port: None,
                capabilities: vec![],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: None,
                node_name: None,
                protocols: vec![],
            },
        ];

        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
        });
        let handler = DiscoveryHandler::with_registry(registry);
        let result = handler.handle_list_peers(json!({})).await.unwrap();
        assert_eq!(result.total_count, 2);
    }

    #[tokio::test]
    async fn list_peers_capability_filter_string_form() {
        let mock_peers = vec![
            DiscoveredPeerInfo {
                node_id: "storage-node".to_string(),
                family_id: "nat0".to_string(),
                address: "10.0.0.1:2300".to_string(),
                tcp_port: Some(8080),
                capabilities: vec!["storage".to_string()],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: Some(0.9),
                node_name: None,
                protocols: vec![],
            },
            DiscoveredPeerInfo {
                node_id: "compute-node".to_string(),
                family_id: "nat0".to_string(),
                address: "10.0.0.2:2300".to_string(),
                tcp_port: None,
                capabilities: vec!["compute".to_string()],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: None,
                node_name: None,
                protocols: vec![],
            },
        ];

        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
        });
        let handler = DiscoveryHandler::with_registry(registry);

        // Federation graph passes capability_filter as a bare string
        let result = handler
            .handle_list_peers(json!({
                "capability_filter": "storage"
            }))
            .await
            .unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.peers[0].node_id, "storage-node");
    }

    #[tokio::test]
    async fn list_peers_multiple_capability_filters_require_all() {
        let mock_peers = vec![
            DiscoveredPeerInfo {
                node_id: "full".to_string(),
                family_id: "fam".to_string(),
                address: "10.0.0.1:2300".to_string(),
                tcp_port: None,
                capabilities: vec![
                    "content_seeder".to_string(),
                    "crypto".to_string(),
                    "tls".to_string(),
                ],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: None,
                node_name: None,
                protocols: vec![],
            },
            DiscoveredPeerInfo {
                node_id: "partial".to_string(),
                family_id: "fam".to_string(),
                address: "10.0.0.2:2300".to_string(),
                tcp_port: None,
                capabilities: vec!["content_seeder".to_string()],
                last_seen: "2026-04-15T00:00:00Z".to_string(),
                quality: None,
                node_name: None,
                protocols: vec![],
            },
        ];

        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
        });
        let handler = DiscoveryHandler::with_registry(registry);
        let result = handler
            .handle_list_peers(json!({
                "capability_filter": ["content_seeder", "tls"]
            }))
            .await
            .unwrap();

        assert_eq!(result.total_count, 1);
        assert_eq!(result.peers[0].node_id, "full");
    }

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
    async fn list_peers_family_only_filters_by_env() {
        let mock_peers =
            vec![peer("same-fam", "nat0", &["storage"]), peer("other-fam", "other", &["storage"])];
        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
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
        let mock_peers = vec![peer("a", "fam1", &[]), peer("b", "fam2", &[])];
        let registry = Arc::new(MockPeerRegistry {
            peers: mock_peers,
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

    // ========================================================================
    // Content distribution tests
    // ========================================================================

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
            .handle_announce(json!({
                "topic": "content:docs",
                "node_id": "node-a",
                "manifest_hash": "blake3:v1",
            }))
            .await
            .unwrap();

        handler
            .handle_announce(json!({
                "topic": "content:docs",
                "node_id": "node-a",
                "manifest_hash": "blake3:v2",
            }))
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
            .handle_announce(json!({
                "family_id": "nat0",
                "capabilities": ["crypto"],
            }))
            .await
            .unwrap();

        let store = handler.content_announcements.read().await;
        assert_eq!(store.len(), 0, "presence mode should not create content entries");
    }

    #[tokio::test]
    async fn content_peers_returns_matching_seeders() {
        let handler = DiscoveryHandler::new();

        handler
            .handle_announce(json!({
                "topic": "content:assets",
                "node_id": "seeder-1",
                "family_id": "nat0",
                "manifest_hash": "blake3:aaa",
            }))
            .await
            .unwrap();

        handler
            .handle_announce(json!({
                "topic": "content:assets",
                "node_id": "seeder-2",
                "family_id": "nat0",
                "manifest_hash": "blake3:aaa",
            }))
            .await
            .unwrap();

        handler
            .handle_announce(json!({
                "topic": "content:other",
                "node_id": "seeder-3",
                "family_id": "nat0",
            }))
            .await
            .unwrap();

        let result =
            handler.handle_content_peers(json!({ "topic": "content:assets" })).await.unwrap();

        assert_eq!(result["total_count"], 2);
        assert_eq!(result["topic"], "content:assets");
        let seeders = result["seeders"].as_array().unwrap();
        assert_eq!(seeders.len(), 2);
    }

    #[tokio::test]
    async fn content_peers_manifest_hash_filter() {
        let handler = DiscoveryHandler::new();

        handler
            .handle_announce(json!({
                "topic": "content:data",
                "node_id": "s1",
                "manifest_hash": "blake3:v1",
            }))
            .await
            .unwrap();

        handler
            .handle_announce(json!({
                "topic": "content:data",
                "node_id": "s2",
                "manifest_hash": "blake3:v2",
            }))
            .await
            .unwrap();

        let result = handler
            .handle_content_peers(json!({
                "topic": "content:data",
                "manifest_hash": "blake3:v2",
            }))
            .await
            .unwrap();

        assert_eq!(result["total_count"], 1);
        assert_eq!(result["seeders"][0]["node_id"], "s2");
    }

    #[tokio::test]
    async fn content_peers_family_only_filter() {
        let handler = DiscoveryHandler::new();

        handler
            .handle_announce(json!({
                "topic": "content:shared",
                "node_id": "same-fam-seeder",
                "family_id": "nat0",
            }))
            .await
            .unwrap();

        handler
            .handle_announce(json!({
                "topic": "content:shared",
                "node_id": "other-fam-seeder",
                "family_id": "other-family",
            }))
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

        handler
            .handle_announce(json!({
                "topic": "content:stuff",
                "node_id": "s1",
            }))
            .await
            .unwrap();

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
}
