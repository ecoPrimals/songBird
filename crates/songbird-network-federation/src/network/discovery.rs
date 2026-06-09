// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🔍 Network Discovery
//!
//! **MODERN NETWORK DISCOVERY** ✅

use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;

/// Network discovery service
#[derive(Debug)]
pub struct NetworkDiscovery;

impl Default for NetworkDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDiscovery {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Discover network nodes via environment-configured peer list.
    ///
    /// Reads `SONGBIRD_PEERS` (comma-separated `host:port` entries) and returns them
    /// as discovered nodes. Returns an empty list when no peers are configured
    /// (standalone mode). Full mesh discovery with health probing is handled at the
    /// orchestrator level via `mesh.init` and `BeaconMesh`.
    #[expect(
        clippy::unused_async,
        reason = "async for interface consistency with other discovery methods"
    )]
    pub async fn discover_nodes(&self) -> SongbirdResult<Vec<DiscoveredNode>> {
        let peers_env = songbird_process_env::var("SONGBIRD_PEERS").unwrap_or_default();
        if peers_env.is_empty() {
            return Ok(Vec::new());
        }

        let nodes: Vec<DiscoveredNode> = peers_env
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .enumerate()
            .map(|(i, addr)| DiscoveredNode {
                node_id: format!("peer-{i}"),
                address: addr.trim().to_string(),
                capabilities: vec!["network".to_string()],
            })
            .collect();

        Ok(nodes)
    }
}

/// Discovered network node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNode {
    pub node_id: String,
    pub address: String,
    pub capabilities: Vec<String>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn network_discovery_new_default() {
        let a = NetworkDiscovery::new();
        let b = NetworkDiscovery;
        let _ = (a, b);
    }

    #[tokio::test]
    async fn discover_nodes_returns_empty_without_peers_env() {
        use songbird_process_env::ScopedEnv;
        songbird_process_env::remove_var("SONGBIRD_PEERS");
        let _guard = ScopedEnv::new("__DISCOVERY_LOCK_PLACEHOLDER", "1");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn discover_nodes_parses_songbird_peers_env() {
        use songbird_process_env::ScopedEnv;
        let _env = ScopedEnv::new("SONGBIRD_PEERS", "10.0.0.1:7700,10.0.0.2:7700");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].address, "10.0.0.1:7700");
        assert_eq!(result[1].address, "10.0.0.2:7700");
    }

    #[test]
    fn discovered_node_serde_roundtrip() {
        let n = DiscoveredNode {
            node_id: "n1".to_string(),
            address: "10.0.0.1:8080".to_string(),
            capabilities: vec!["a".to_string(), "b".to_string()],
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: DiscoveredNode = serde_json::from_str(&json).unwrap();
        assert_eq!(n.node_id, back.node_id);
        assert_eq!(n.capabilities, back.capabilities);
    }

    #[test]
    fn discovered_node_empty_capabilities() {
        let n = DiscoveredNode {
            node_id: "x".to_string(),
            address: "h".to_string(),
            capabilities: vec![],
        };
        assert!(n.capabilities.is_empty());
    }
}
