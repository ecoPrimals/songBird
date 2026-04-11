// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Bridge - Connects orchestrator's `AnonymousDiscoveryListener` to IPC
//!
//! This module provides a bridge between the orchestrator's discovery listener
//! and the Discovery Handler's `PeerRegistry` trait.
//!
//! **TRUE PRIMAL**: Runtime discovery, no hardcoding, capability-based.

use crate::error::IpcResult;
use crate::handlers::discovery_handler::{DiscoveredPeerInfo, PeerRegistry};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

// Re-export from songbird-discovery for external use
pub use songbird_discovery::anonymous::AnonymousDiscoveryListener;
pub use songbird_discovery::anonymous::peer::DiscoveredPeer;

/// Bridge between orchestrator's `AnonymousDiscoveryListener` and `PeerRegistry` trait
///
/// This enables the Discovery Handler to access real discovered peers from
/// the UDP beacon listener without tight coupling.
pub struct DiscoveryListenerBridge {
    listener: Arc<AnonymousDiscoveryListener>,
}

impl DiscoveryListenerBridge {
    /// Create a new bridge from an `AnonymousDiscoveryListener`
    pub fn new(listener: Arc<AnonymousDiscoveryListener>) -> Self {
        info!("🌉 Discovery bridge: Connected to Anonymous Discovery Listener");
        Self {
            listener,
        }
    }
}

#[async_trait]
impl PeerRegistry for DiscoveryListenerBridge {
    async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>> {
        debug!("Discovery bridge: Getting all peers");

        // Get peers from the discovery listener
        let peers = self.listener.get_peers().await;

        debug!("Discovery bridge: Found {} peers", peers.len());

        // Convert from DiscoveredPeer to DiscoveredPeerInfo (JSON-RPC compatible)
        let peer_infos = peers.into_iter().map(convert_discovered_peer).collect();

        Ok(peer_infos)
    }

    async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>> {
        debug!("Discovery bridge: Getting peer: {}", peer_id);

        // Try to get by session_id first (v2.x compatibility)
        if let Some(peer) = self.listener.get_peer(peer_id).await {
            return Ok(Some(convert_discovered_peer(peer)));
        }

        // Try to find by node_id (v3.0+)
        let peers = self.listener.get_peers().await;
        for peer in peers {
            if let Some(ref node_id) = peer.node_id
                && node_id == peer_id
            {
                return Ok(Some(convert_discovered_peer(peer)));
            }
        }

        Ok(None)
    }
}

/// Convert from discovery's `DiscoveredPeer` to JSON-RPC `DiscoveredPeerInfo`
fn convert_discovered_peer(peer: DiscoveredPeer) -> DiscoveredPeerInfo {
    // Use node_id if available (v3.0+), otherwise session_id (v2.x)
    let node_id = peer.node_id.clone().unwrap_or_else(|| peer.session_id.clone());

    // Extract family_id from tags if available (Dark Forest protocol)
    let family_id = extract_family_id(&peer).unwrap_or_else(|| "unknown".to_string());

    // Convert SystemTime to ISO 8601 string
    let last_seen = peer
        .last_seen
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
        .map_or_else(|| chrono::Utc::now().to_rfc3339(), |dt| dt.to_rfc3339());

    // Calculate signal quality based on timestamp freshness
    let quality = calculate_quality(&peer);

    DiscoveredPeerInfo {
        node_id,
        family_id,
        address: peer.address.to_string(),
        tcp_port: Some(peer.port),
        capabilities: peer.capabilities,
        last_seen,
        quality: Some(quality),
        node_name: peer.node_name,
        protocols: peer.protocols,
    }
}

/// Extract `family_id` from tags (Dark Forest encrypted lineage)
fn extract_family_id(peer: &DiscoveredPeer) -> Option<String> {
    // Check tags for family_id hint
    peer.tags.as_ref().and_then(|tags| {
        tags.iter()
            .find(|tag| tag.starts_with("family:"))
            .map(|tag| tag.strip_prefix("family:").unwrap_or("unknown").to_string())
    })
}

/// Calculate signal quality based on timestamp freshness
fn calculate_quality(peer: &DiscoveredPeer) -> f64 {
    let now = std::time::SystemTime::now();

    now.duration_since(peer.last_seen).map_or(0.99, |elapsed| {
        let seconds = elapsed.as_secs();

        // Quality degrades with time
        if seconds < 10 {
            0.99 // Very fresh
        } else if seconds < 30 {
            0.95 // Fresh
        } else if seconds < 60 {
            0.90 // Recent
        } else if seconds < 300 {
            0.80 // Aging
        } else {
            0.50 // Stale
        }
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_convert_discovered_peer_v3() {
        let peer = DiscoveredPeer {
            session_id: "session-123".to_string(),
            node_id: Some("node-alpha".to_string()),
            node_name: Some("alpha-tower".to_string()),
            endpoints: None,
            capabilities: vec!["crypto".to_string(), "tls".to_string()],
            tags: Some(vec!["family:nat0".to_string()]),
            timestamp: Some(1706500000),
            identity_attestations: None,
            protocols: vec!["birdsong".to_string()],
            port: 8081,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 2300),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        let info = convert_discovered_peer(peer);

        assert_eq!(info.node_id, "node-alpha");
        assert_eq!(info.family_id, "nat0");
        assert_eq!(info.address, "192.168.1.100:2300");
        assert_eq!(info.tcp_port, Some(8081));
        assert!(info.quality.is_some());
        assert!(info.quality.unwrap() > 0.9); // Fresh peer
    }

    #[test]
    fn test_convert_discovered_peer_v2_fallback() {
        let peer = DiscoveredPeer {
            session_id: "session-456".to_string(),
            node_id: None, // v2.x - no node_id
            node_name: None,
            endpoints: None,
            capabilities: vec!["crypto".to_string()],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec!["http".to_string()],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)), 2300),
            last_seen: SystemTime::now(),
            version: "2.0".to_string(),
        };

        let info = convert_discovered_peer(peer);

        assert_eq!(info.node_id, "session-456"); // Falls back to session_id
        assert_eq!(info.family_id, "unknown"); // No family info
        assert_eq!(info.address, "192.168.1.101:2300");
    }

    #[test]
    fn test_extract_family_id() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: Some(vec!["family:nat0".to_string(), "other:value".to_string()]),
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        let family_id = extract_family_id(&peer);
        assert_eq!(family_id, Some("nat0".to_string()));
    }

    #[test]
    fn extract_family_id_tag_family_prefix_only_yields_empty_string() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: Some(vec!["family:".to_string()]),
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        assert_eq!(extract_family_id(&peer), Some(String::new()));
    }

    #[test]
    fn test_extract_family_id_no_tags() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        let family_id = extract_family_id(&peer);
        assert_eq!(family_id, None);
    }

    #[test]
    fn test_calculate_quality_fresh() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
            last_seen: SystemTime::now(), // Fresh!
            version: "3.0".to_string(),
        };

        let quality = calculate_quality(&peer);
        assert!(quality > 0.95); // Should be very high
    }

    #[test]
    fn calculate_quality_age_buckets() {
        let mk = |secs_ago: u64| -> DiscoveredPeer {
            DiscoveredPeer {
                session_id: "test".to_string(),
                node_id: None,
                node_name: None,
                endpoints: None,
                capabilities: vec![],
                tags: None,
                timestamp: None,
                identity_attestations: None,
                protocols: vec![],
                port: 8080,
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
                last_seen: SystemTime::now() - Duration::from_secs(secs_ago),
                version: "3.0".to_string(),
            }
        };

        assert!(calculate_quality(&mk(5)) > 0.98);
        assert!((0.94..0.96).contains(&calculate_quality(&mk(15))));
        assert!((0.89..0.91).contains(&calculate_quality(&mk(45))));
        assert!((0.79..0.81).contains(&calculate_quality(&mk(120))));
        assert!(calculate_quality(&mk(400)) < 0.55);
    }

    #[test]
    fn test_calculate_quality_stale() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2300),
            last_seen: SystemTime::now() - Duration::from_secs(600), // 10 minutes ago
            version: "3.0".to_string(),
        };

        let quality = calculate_quality(&peer);
        assert!(quality < 0.6); // Should be lower
    }
}
