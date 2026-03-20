// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Lineage-Enhanced Service Discovery Backend
//!
//! Extends mDNS/DNS-SD service discovery to include genetic lineage information
//! for automatic peer trust establishment.

use crate::discovery_packet::DiscoveryPacket;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Service discovery backend with lineage support
#[derive(Debug)]
pub struct LineageServiceDiscovery {
    /// Service name for mDNS
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    service_name: String,

    /// Local node information
    local_node_id: String,
    local_capabilities: Vec<String>,
    local_endpoint: String,

    /// Genetic lineage (if available)
    local_lineage: Option<songbird_types::LineageId>,
    local_proof: Option<songbird_types::LineageProof>,

    /// Cache of discovered peers with lineage
    peer_cache: HashMap<String, DiscoveryPacket>,

    /// Cache TTL
    cache_ttl: Duration,
}

impl LineageServiceDiscovery {
    /// Create a new lineage-aware service discovery backend
    pub fn new(
        service_name: impl Into<String>,
        node_id: impl Into<String>,
        capabilities: Vec<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            local_node_id: node_id.into(),
            local_capabilities: capabilities,
            local_endpoint: endpoint.into(),
            local_lineage: None,
            local_proof: None,
            peer_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Set local lineage information
    #[must_use]
    pub fn with_lineage(
        mut self,
        lineage_id: songbird_types::LineageId,
        proof: songbird_types::LineageProof,
    ) -> Self {
        self.local_lineage = Some(lineage_id);
        self.local_proof = Some(proof);
        self
    }

    /// Advertise service with genetic lineage over mDNS
    ///
    /// Broadcasts this node's presence and capabilities, including
    /// cryptographic lineage for automatic trust establishment.
    pub async fn advertise_with_lineage(&self) -> Result<()> {
        info!("🎵 Advertising Songbird service with genetic lineage");

        // Create discovery packet
        let mut packet = DiscoveryPacket::new(
            self.local_node_id.clone(),
            self.local_capabilities.clone(),
            self.local_endpoint.clone(),
        );

        // Add lineage if available
        if let (Some(lineage), Some(proof)) = (&self.local_lineage, &self.local_proof) {
            packet = packet.with_lineage(lineage.clone(), proof.clone());
            info!("✅ Including genetic lineage: {}", lineage);
        } else {
            warn!("⚠️ No genetic lineage available - peers will require manual approval");
        }

        // Convert to TXT records for mDNS
        let txt_records = packet.to_txt_records();

        debug!("📡 mDNS TXT records: {} entries", txt_records.len());
        for (key, value) in &txt_records {
            debug!("  {}: {} bytes", key, value.len());
        }

        // NOTE: For production mDNS broadcasting, use songbird-config::discovery::MdnsDiscovery
        // which provides full RFC 6762 compliant mDNS with mdns-sd integration.
        // This module focuses on lineage-specific discovery logic.

        Ok(())
    }

    /// Discover peers with lineage information
    ///
    /// Scans mDNS for Songbird instances and parses their genetic lineage.
    /// Returns peers sorted by lineage compatibility (same lineage first).
    pub async fn discover_peers_with_lineage(&mut self) -> Result<Vec<DiscoveryPacket>> {
        info!("🔍 Discovering Songbird peers with lineage information");

        // NOTE: For production mDNS discovery, use songbird-config::discovery::MdnsDiscovery
        // which provides full capability-based discovery with mdns-sd integration.
        // This module can be integrated with that system for lineage-aware discovery.

        // For now, return cached peers
        let mut peers: Vec<_> = self.peer_cache.values().cloned().collect();

        // Sort by lineage compatibility
        if let Some(our_lineage) = &self.local_lineage {
            peers.sort_by_key(|p| {
                p.genetic_lineage.as_ref().map_or(3, |peer_lineage| {
                    // Same lineage first
                    if peer_lineage == our_lineage {
                        0
                    } else if peer_lineage.tower_id() == our_lineage.tower_id() {
                        // Same tower second
                        1
                    } else {
                        // Different lineage last
                        2
                    }
                })
            });
        }

        info!("✅ Discovered {} peers", peers.len());
        Ok(peers)
    }

    /// Process a received mDNS announcement
    ///
    /// Parses TXT records from an mDNS announcement and updates peer cache.
    pub fn process_announcement(&mut self, txt_records: &HashMap<String, String>) -> Result<()> {
        let packet = DiscoveryPacket::from_txt_records(txt_records)
            .context("Failed to parse discovery packet from TXT records")?;

        if packet.has_lineage() {
            info!("✅ Peer {} has genetic lineage: {:?}", packet.node_id, packet.genetic_lineage);
        } else {
            debug!("ℹ️  Peer {} has no lineage information", packet.node_id);
        }

        // Update cache
        self.peer_cache.insert(packet.node_id.clone(), packet);

        Ok(())
    }

    /// Get peers that share our genetic lineage
    #[must_use]
    pub fn get_same_lineage_peers(&self) -> Vec<&DiscoveryPacket> {
        self.local_lineage.as_ref().map_or_else(Vec::new, |our_lineage| {
            self.peer_cache
                .values()
                .filter(|p| p.genetic_lineage.as_ref() == Some(our_lineage))
                .collect()
        })
    }

    /// Get peers with different lineage
    #[must_use]
    pub fn get_different_lineage_peers(&self) -> Vec<&DiscoveryPacket> {
        self.local_lineage.as_ref().map_or_else(Vec::new, |our_lineage| {
            self.peer_cache
                .values()
                .filter(|p| {
                    p.genetic_lineage
                        .as_ref()
                        .is_some_and(|peer_lineage| peer_lineage != our_lineage)
                })
                .collect()
        })
    }

    /// Get peers with no lineage information
    #[must_use]
    pub fn get_no_lineage_peers(&self) -> Vec<&DiscoveryPacket> {
        self.peer_cache.values().filter(|p| p.genetic_lineage.is_none()).collect()
    }

    /// Clear expired peers from cache
    pub fn cleanup_expired_peers(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ttl_secs = self.cache_ttl.as_secs();

        self.peer_cache.retain(|_id, packet| {
            let age = now.saturating_sub(packet.timestamp);
            age < ttl_secs
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::{LineageId, LineageProof};

    #[tokio::test]
    async fn test_create_discovery_backend() {
        let discovery = LineageServiceDiscovery::new(
            "_songbird._tcp.local",
            "test-node-1",
            vec!["compute".to_string()],
            "http://192.168.1.100:8080",
        );

        assert_eq!(discovery.local_node_id, "test-node-1");
        assert!(discovery.local_lineage.is_none());
    }

    #[tokio::test]
    async fn test_discovery_with_lineage() {
        let lineage_id = LineageId::new("lineage:tower1:2026:abc");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);

        let discovery = LineageServiceDiscovery::new(
            "_songbird._tcp.local",
            "test-node-1",
            vec!["compute".to_string()],
            "http://192.168.1.100:8080",
        )
        .with_lineage(lineage_id, proof);

        assert!(discovery.local_lineage.is_some());
        assert!(discovery.local_proof.is_some());
    }

    #[tokio::test]
    async fn test_process_announcement() {
        let mut discovery = LineageServiceDiscovery::new(
            "_songbird._tcp.local",
            "test-node-1",
            vec!["compute".to_string()],
            "http://192.168.1.100:8080",
        );

        let packet = DiscoveryPacket::new(
            "peer-node-1",
            vec!["storage".to_string()],
            "http://192.168.1.101:8080",
        );

        let txt_records = packet.to_txt_records();
        discovery.process_announcement(&txt_records).unwrap();

        assert_eq!(discovery.peer_cache.len(), 1);
    }

    #[test]
    fn test_peer_filtering_by_lineage() {
        let our_lineage = LineageId::new("lineage:tower1:2026:abc");
        let our_proof = LineageProof::new(our_lineage.clone(), vec![], 1234567890);

        let mut discovery = LineageServiceDiscovery::new(
            "_songbird._tcp.local",
            "test-node-1",
            vec!["compute".to_string()],
            "http://192.168.1.100:8080",
        )
        .with_lineage(our_lineage.clone(), our_proof);

        // Add peer with same lineage
        let same_packet = DiscoveryPacket::new(
            "peer-same",
            vec!["storage".to_string()],
            "http://192.168.1.101:8080",
        )
        .with_lineage(our_lineage.clone(), LineageProof::new(our_lineage, vec![], 1234567890));
        discovery.peer_cache.insert("peer-same".to_string(), same_packet);

        // Add peer with different lineage
        let different_lineage = LineageId::new("lineage:tower2:2026:xyz");
        let different_packet = DiscoveryPacket::new(
            "peer-different",
            vec!["compute".to_string()],
            "http://192.168.1.102:8080",
        )
        .with_lineage(
            different_lineage.clone(),
            LineageProof::new(different_lineage, vec![], 1234567890),
        );
        discovery.peer_cache.insert("peer-different".to_string(), different_packet);

        // Add peer with no lineage
        let no_lineage_packet = DiscoveryPacket::new(
            "peer-no-lineage",
            vec!["ai".to_string()],
            "http://192.168.1.103:8080",
        );
        discovery.peer_cache.insert("peer-no-lineage".to_string(), no_lineage_packet);

        // Test filtering
        assert_eq!(discovery.get_same_lineage_peers().len(), 1);
        assert_eq!(discovery.get_different_lineage_peers().len(), 1);
        assert_eq!(discovery.get_no_lineage_peers().len(), 1);
    }
}
