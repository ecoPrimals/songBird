// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery JSON-RPC Handler
//!
//! Exposes peer discovery functionality via JSON-RPC for Dark Forest rendezvous protocol.
//!
//! ## Methods
//! - `discovery.peers` — List discovered peers (supports `family_only`, `capability_filter`)
//! - `discovery.announce` — Announce presence or content availability to the mesh
//! - `discovery.get_peer` — Get specific peer by ID
//!
//! ## Content Distribution
//! `discovery.announce` supports topic-based announcements for the seeder/leecher pattern
//! defined in `content_distribution_federation.toml`. Topics use the format
//! `content:<namespace>` (e.g., `content:ludospring:assets`).
//!
//! ## Security Note
//! Peer information includes network addresses. Only expose to trusted consumers.

use crate::error::{IpcError, IpcResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

// We'll integrate with songbird-discovery's AnonymousDiscoveryListener
// For now, we'll define the interface that will be connected to the orchestrator

// ============================================================================
// Discovery Handler
// ============================================================================

/// Discovery handler for peer discovery operations
pub struct DiscoveryHandler {
    /// Peer registry (will be injected from orchestrator)
    peer_registry: Option<Arc<dyn PeerRegistry>>,
}

impl DiscoveryHandler {
    /// Create a new discovery handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            peer_registry: None,
        }
    }

    /// Create discovery handler with peer registry
    pub fn with_registry(registry: Arc<dyn PeerRegistry>) -> Self {
        Self {
            peer_registry: Some(registry),
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
    ///   `content:ludospring:assets`) per `content_distribution_federation.toml`
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

        if let Some(topic) = topic {
            info!(
                "Discovery: content announce (topic={}, manifest={}, seeders={})",
                topic,
                manifest_hash.unwrap_or("none"),
                seeder_count.unwrap_or(0)
            );
            Ok(serde_json::json!({
                "announced": true,
                "mode": "topic",
                "topic": topic,
                "manifest_hash": manifest_hash,
                "seeder_count": seeder_count,
                "bond_types_accepted": bond_types,
                "family_id": family_id,
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
        assert!(v["seeder_count"].is_null());
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
}
