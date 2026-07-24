// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection Manager - Domain-driven modular architecture
//!
//! Manages peer connections with trust-based capability enforcement.
//!
//! **Architecture** (v3.21.0 - Jan 19, 2026):
//! - `peer`: Peer metadata and lifecycle management
//! - `trust`: Trust evaluation and establishment
//! - `btsp`: BTSP client integration and connection creation
//! - `types`: Domain types and serialization

use anyhow::Result;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export public types
pub use btsp::BtspConnectionFactory;
pub use peer::PeerRegistry;
pub use trust::TrustEvaluator;
pub use types::{PeerMetadata, systemtime_as_secs};

use crate::btsp_client::BtspClient;
use crate::connections::Connection;
use crate::trust::peer_trust::PeerTrustDecision;

mod btsp;
mod peer;
mod trust;
mod types;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;

/// Connection Manager - Coordinates peer connections with progressive trust
///
/// **Modern Design** (v3.21.0):
/// - Domain-driven module organization
/// - Each module < 300 lines
/// - Clear separation of concerns
/// - Easy to test in isolation
pub struct ConnectionManager {
    /// Active connections by `peer_id`
    connections: Arc<RwLock<HashMap<String, Connection>>>,

    /// Peer registry (metadata, lifecycle)
    peer_registry: PeerRegistry,

    /// Trust evaluator
    trust_evaluator: TrustEvaluator,

    /// BTSP connection factory
    btsp_factory: BtspConnectionFactory,
}

impl ConnectionManager {
    /// Create a new connection manager
    ///
    /// **Modern Rust** (v3.21.0):
    /// - Composed from domain modules
    /// - Clear responsibility boundaries
    /// - Zero blocking calls
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            peer_registry: PeerRegistry::new(),
            trust_evaluator: TrustEvaluator::new(),
            btsp_factory: BtspConnectionFactory::new(),
        }
    }

    /// Get BTSP client (lazily initialized)
    #[expect(dead_code, reason = "wires when BTSP peer connections go live")]
    async fn btsp_client(&self) -> Result<Arc<BtspClient>> {
        self.btsp_factory.get_or_init_client().await
    }

    /// Handle trust decision for discovered peer
    ///
    /// **v3.21.0**: Refactored to domain modules
    /// **Capability-based**: Discovers peer capabilities at runtime
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn handle_trust_decision(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        trust_decision: &PeerTrustDecision,
        discovery_method: String,
    ) -> Result<()> {
        self.trust_evaluator
            .handle_decision(
                peer_id,
                endpoint,
                capabilities,
                peer_tags,
                trust_decision,
                discovery_method,
                &self.connections,
                &self.peer_registry,
                &self.btsp_factory,
            )
            .await
    }

    /// Establish connection at specified trust level
    ///
    /// **v3.21.0**: Refactored to domain modules
    /// **Modern pattern**: Delegates to trust evaluator
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn establish_connection(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        peer_tags: Vec<String>,
        trust_level: TrustLevel,
        discovery_method: String,
    ) -> Result<()> {
        self.trust_evaluator
            .establish_connection(
                peer_id,
                endpoint,
                trust_level,
                discovery_method,
                capabilities,
                peer_tags,
                &self.connections,
                &self.peer_registry,
                &self.btsp_factory,
            )
            .await
    }

    /// Call peer with operation
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn call_peer(&self, peer_id: &str, operation: &str, request: Value) -> Result<Value> {
        let connections = self.connections.read().await;
        let connection = connections
            .get(peer_id)
            .ok_or_else(|| anyhow::anyhow!("Peer not connected: {peer_id}"))?;

        connection.call(operation, request).await
    }

    /// Get connection trust level
    pub async fn get_connection(&self, peer_id: &str) -> Option<TrustLevel> {
        let connections = self.connections.read().await;
        connections.get(peer_id).map(super::super::connections::Connection::trust_level)
    }

    /// List all connected peers
    pub async fn list_peers(&self) -> Vec<(String, TrustLevel)> {
        self.peer_registry.list_connected_peers(&self.connections).await
    }

    /// Get peer metadata
    pub async fn get_peer_metadata(&self, peer_id: &str) -> Option<PeerMetadata> {
        self.peer_registry.get_metadata(peer_id).await
    }

    /// Get all peer metadata
    pub async fn get_all_peers(&self) -> Vec<PeerMetadata> {
        self.peer_registry.get_all_metadata().await
    }

    /// Get peer count
    pub async fn get_peer_count(&self) -> usize {
        self.peer_registry.count().await
    }

    /// Get rejected peers
    pub async fn get_rejected_peers(&self) -> HashMap<String, String> {
        self.peer_registry.get_rejected().await
    }

    /// Close connection to peer
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn close_connection(&self, peer_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(peer_id);
        Ok(())
    }

    /// Get connection statistics by trust level
    pub async fn connection_stats(&self) -> HashMap<TrustLevel, usize> {
        let connections = self.connections.read().await;
        let mut stats = HashMap::new();

        for connection in connections.values() {
            *stats.entry(connection.trust_level()).or_insert(0) += 1;
        }

        stats
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
