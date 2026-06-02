// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery handler types — peer registry trait and JSON-RPC DTOs.

use crate::error::IpcResult;
use serde::{Deserialize, Serialize};

/// Trait for peer registry (implemented by orchestrator bridge and tests).
#[expect(async_fn_in_trait, reason = "native async trait; use PeerRegistrySlot for dispatch")]
pub trait PeerRegistry: Send + Sync {
    /// Get all discovered peers.
    async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>>;

    /// Get a specific peer by ID.
    async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>>;
}

/// Test double for [`PeerRegistry`].
#[cfg(test)]
pub struct MockPeerRegistry {
    pub peers: Vec<DiscoveredPeerInfo>,
}

#[cfg(test)]
impl PeerRegistry for MockPeerRegistry {
    async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>> {
        Ok(self.peers.clone())
    }

    async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>> {
        Ok(self.peers.iter().find(|p| p.node_id == peer_id).cloned())
    }
}

/// Parameters for `discovery.get_peer`.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryGetPeerParams {
    pub peer_id: String,
}

/// Result for `discovery.peers`.
#[derive(Debug, Clone, Serialize)]
pub struct DiscoveryPeersResult {
    pub peers: Vec<DiscoveredPeerInfo>,
    pub total_count: usize,
}

/// Discovered peer information (JSON-RPC compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeerInfo {
    pub node_id: String,
    pub family_id: String,
    pub address: String,
    pub tcp_port: Option<u16>,
    pub capabilities: Vec<String>,
    pub last_seen: String,
    pub quality: Option<f64>,
    pub node_name: Option<String>,
    pub protocols: Vec<String>,
    /// RTT to peer in milliseconds (null if not yet measured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}
