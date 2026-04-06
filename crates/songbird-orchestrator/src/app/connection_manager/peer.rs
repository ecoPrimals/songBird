// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Peer registry and lifecycle management

use super::types::PeerMetadata;
use crate::connections::Connection;
use songbird_types::TrustLevel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Peer Registry - Manages peer metadata and lifecycle
///
/// **Responsibilities**:
/// - Store peer metadata
/// - Track rejected peers (audit trail)
/// - Query operations (list, count, get)
/// - Lifecycle management
pub struct PeerRegistry {
    /// Metadata about each peer
    metadata: Arc<RwLock<HashMap<String, PeerMetadata>>>,

    /// Rejected peers (audit trail)
    rejected: Arc<RwLock<HashMap<String, String>>>,
}

impl PeerRegistry {
    /// Create new peer registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(RwLock::new(HashMap::new())),
            rejected: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register peer with metadata
    ///
    /// **Capability-based**: Stores discovered capabilities at runtime
    pub async fn register(
        &self,
        peer_id: String,
        endpoint: String,
        trust_level: TrustLevel,
        discovery_method: String,
        capabilities: Vec<String>,
    ) {
        let metadata = PeerMetadata {
            peer_id: peer_id.clone(),
            endpoint,
            trust_level,
            discovery_method,
            capabilities,
            established_at: std::time::SystemTime::now(),
        };

        self.metadata.write().await.insert(peer_id, metadata);
    }

    /// Mark peer as rejected
    ///
    /// **Audit trail**: Records rejection reason for transparency
    pub async fn reject(&self, peer_id: String, reason: String) {
        self.rejected.write().await.insert(peer_id, reason);
    }

    /// Get metadata for specific peer
    pub async fn get_metadata(&self, peer_id: &str) -> Option<PeerMetadata> {
        self.metadata.read().await.get(peer_id).cloned()
    }

    /// Get all peer metadata
    pub async fn get_all_metadata(&self) -> Vec<PeerMetadata> {
        self.metadata.read().await.values().cloned().collect()
    }

    /// Get peer count
    pub async fn count(&self) -> usize {
        self.metadata.read().await.len()
    }

    /// Get rejected peers
    pub async fn get_rejected(&self) -> HashMap<String, String> {
        self.rejected.read().await.clone()
    }

    /// List connected peers (combines with connection state)
    ///
    /// **Modern pattern**: Read-only access via shared lock
    pub async fn list_connected_peers(
        &self,
        connections: &Arc<RwLock<HashMap<String, Connection>>>,
    ) -> Vec<(String, TrustLevel)> {
        let conns = connections.read().await;
        conns.iter().map(|(id, conn)| (id.clone(), conn.trust_level())).collect()
    }
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
