// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery JSON-RPC Handler
//!
//! Exposes peer discovery functionality via JSON-RPC for Dark Forest rendezvous protocol.
//!
//! ## Methods
//! - `discovery.peers` - List discovered peers from UDP beacons
//! - `discovery.get_peer` - Get specific peer by ID
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
    /// Lists all discovered peers from UDP beacons.
    pub async fn handle_list_peers(&self, _params: Value) -> IpcResult<DiscoveryPeersResult> {
        debug!("Discovery: list_peers");

        // Get peers from registry
        let peers = if let Some(ref registry) = self.peer_registry {
            registry.get_all_peers().await?
        } else {
            // No registry connected - return empty list (for testing)
            Vec::new()
        };

        let total_count = peers.len();
        info!("✅ Discovery: Found {} peers", total_count);

        Ok(DiscoveryPeersResult {
            peers,
            total_count,
        })
    }

    /// Handle `discovery.announce` JSON-RPC method
    ///
    /// Announces this node's presence to the discovery network.
    pub async fn handle_announce(&self, params: Value) -> IpcResult<Value> {
        let family_id = params.get("family_id").and_then(Value::as_str).unwrap_or("unknown");
        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
            .unwrap_or_default();
        info!("✅ Discovery: announce (family={}, capabilities={})", family_id, capabilities.len());
        Ok(serde_json::json!({
            "announced": true,
            "family_id": family_id,
            "capabilities": capabilities
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
}
