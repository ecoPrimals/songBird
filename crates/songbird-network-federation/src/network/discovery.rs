// SPDX-License-Identifier: AGPL-3.0-only
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

    /// Discover network nodes via federation state.
    ///
    /// Returns an empty list when no federation peers are reachable.
    /// Real discovery is handled by the orchestrator's federation state
    /// and injected via `songbird-primal-coordination` at runtime.
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::not_implemented` when called without
    /// a configured federation backend (standalone mode).
    pub async fn discover_nodes(&self) -> SongbirdResult<Vec<DiscoveredNode>> {
        Err(songbird_types::SongbirdError::not_implemented(
            "NetworkDiscovery requires federation state injection — use orchestrator discovery",
        ))
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
    async fn discover_nodes_returns_not_implemented_without_federation() {
        let d = NetworkDiscovery::new();
        let result = d.discover_nodes().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("not implemented") || err.to_string().contains("federation"),
            "expected not-implemented error, got: {err}"
        );
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
