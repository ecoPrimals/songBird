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

    /// Discover network nodes from environment and persisted state.
    ///
    /// Sources (merged, deduplicated by address):
    /// 1. `SONGBIRD_PEERS` env var (comma-separated `node_id@host:port` or bare `host:port`)
    /// 2. Persisted peers from `~/.local/share/songbird/peers.toml` (Wave 106)
    ///
    /// Returns an empty list when no peers are configured (standalone mode).
    /// Full mesh discovery with health probing is handled at the orchestrator
    /// level via `mesh.init` and `BeaconMesh`.
    #[expect(
        clippy::unused_async,
        reason = "async for interface consistency with other discovery methods"
    )]
    pub async fn discover_nodes(&self) -> SongbirdResult<Vec<DiscoveredNode>> {
        let mut nodes: Vec<DiscoveredNode> = Vec::new();
        let mut seen_addresses: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        let peers_env = songbird_process_env::var("SONGBIRD_PEERS").unwrap_or_default();
        for entry in peers_env.split(',').filter(|s| !s.trim().is_empty()) {
            let entry = entry.trim();
            let (node_id, address) = if let Some((nid, addr)) = entry.split_once('@') {
                (nid.to_string(), addr.to_string())
            } else {
                let addr = entry.to_string();
                let nid = addr
                    .split(':')
                    .next()
                    .map_or_else(|| format!("peer-{}", nodes.len()), |ip| format!("peer-{ip}"));
                (nid, addr)
            };
            if seen_addresses.insert(address.clone()) {
                nodes.push(DiscoveredNode {
                    node_id,
                    address,
                    capabilities: vec!["network".to_string(), "mesh".to_string()],
                });
            }
        }

        for (node_id, address) in load_persisted_peers() {
            if seen_addresses.insert(address.clone()) {
                nodes.push(DiscoveredNode {
                    node_id,
                    address,
                    capabilities: vec!["network".to_string(), "mesh".to_string()],
                });
            }
        }

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

/// Load persisted mesh peers from `<data_dir>/peers.toml`.
///
/// Returns `(node_id, address)` pairs. Returns empty vec if file is absent or invalid.
fn load_persisted_peers() -> Vec<(String, String)> {
    let path = songbird_types::defaults::paths::data_dir().join("peers.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };

    #[derive(serde::Deserialize)]
    struct PeersFile {
        #[serde(default)]
        peers: Vec<PeerEntry>,
    }
    #[derive(serde::Deserialize)]
    struct PeerEntry {
        node_id: String,
        address: String,
    }

    let Ok(file) = toml::from_str::<PeersFile>(&content) else {
        return Vec::new();
    };

    file.peers.into_iter().map(|p| (p.node_id, p.address)).collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use songbird_process_env::ScopedEnv;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn network_discovery_new_default() {
        let a = NetworkDiscovery::new();
        let b = NetworkDiscovery;
        let _ = (a, b);
    }

    #[tokio::test]
    async fn discover_nodes_returns_empty_without_peers_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        songbird_process_env::remove_var("SONGBIRD_PEERS");
        let _data = ScopedEnv::new("SONGBIRD_DATA_DIR", "/tmp/songbird-test-nonexistent-dir");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn discover_nodes_parses_songbird_peers_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _env = ScopedEnv::new("SONGBIRD_PEERS", "east@10.0.0.1:7700,west@10.0.0.2:7700");
        let _data = ScopedEnv::new("SONGBIRD_DATA_DIR", "/tmp/songbird-test-nonexistent-dir");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].node_id, "east");
        assert_eq!(result[0].address, "10.0.0.1:7700");
        assert_eq!(result[1].node_id, "west");
        assert_eq!(result[1].address, "10.0.0.2:7700");
    }

    #[tokio::test]
    async fn discover_nodes_bare_addresses_generate_peer_ids() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _env = ScopedEnv::new("SONGBIRD_PEERS", "10.0.0.1:7700,10.0.0.2:7700");
        let _data = ScopedEnv::new("SONGBIRD_DATA_DIR", "/tmp/songbird-test-nonexistent-dir");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].node_id, "peer-10.0.0.1");
        assert_eq!(result[1].node_id, "peer-10.0.0.2");
    }

    #[tokio::test]
    async fn discover_nodes_deduplicates_by_address() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _env = ScopedEnv::new("SONGBIRD_PEERS", "a@10.0.0.1:7700,b@10.0.0.1:7700");
        let _data = ScopedEnv::new("SONGBIRD_DATA_DIR", "/tmp/songbird-test-nonexistent-dir");
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].node_id, "a");
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

    #[test]
    fn load_persisted_peers_no_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _data = ScopedEnv::new("SONGBIRD_DATA_DIR", "/tmp/songbird-test-nonexistent-dir");
        let peers = load_persisted_peers();
        assert!(peers.is_empty());
    }
}
