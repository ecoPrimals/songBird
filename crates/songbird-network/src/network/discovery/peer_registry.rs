//! Peer registry for managing discovered peers

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::types::{DiscoveredPeer, DiscoveryConfig, PeerType};
use songbird_errors::Result;
use songbird_universal_primals::PrimalCapability;

/// Peer registry for managing discovered peers
#[derive(Clone)]
pub struct PeerRegistry {
    peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
    peer_capabilities: Arc<RwLock<HashMap<String, Vec<PrimalCapability>>>>,
    last_seen: Arc<RwLock<HashMap<String, Instant>>>,
    config: DiscoveryConfig,
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerRegistry {
    /// Create new peer registry
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_capabilities: Arc::new(RwLock::new(HashMap::new())),
            last_seen: Arc::new(RwLock::new(HashMap::new())),
            config: DiscoveryConfig::default(),
        }
    }

    /// Create peer registry with configuration
    pub fn with_config(config: DiscoveryConfig) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_capabilities: Arc::new(RwLock::new(HashMap::new())),
            last_seen: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a discovered peer
    pub async fn register_peer(
        &self,
        peer: DiscoveredPeer,
        capabilities: Vec<PrimalCapability>,
    ) -> Result<()> {
        let peer_id = peer.peer_id.clone();

        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_id.clone(), peer);
        }

        {
            let mut caps = self.peer_capabilities.write().await;
            caps.insert(peer_id.clone(), capabilities);
        }

        {
            let mut last_seen = self.last_seen.write().await;
            last_seen.insert(peer_id.clone(), Instant::now());
        }

        debug!("Registered peer: {}", peer_id);
        Ok(())
    }

    /// Update peer last seen time
    pub async fn update_peer_last_seen(&self, peer_id: &str) -> Result<()> {
        let mut last_seen = self.last_seen.write().await;

        if last_seen.contains_key(peer_id) {
            last_seen.insert(peer_id.to_string(), Instant::now());

            // Also update the peer's last_seen field
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(peer_id) {
                peer.update_last_seen();
            }

            debug!("Updated last seen for peer: {}", peer_id);
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::network_error(format!(
                "Peer not found: {peer_id}"
            )))
        }
    }

    /// Get peer by ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<DiscoveredPeer> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// Get peer capabilities
    pub async fn get_peer_capabilities(&self, peer_id: &str) -> Option<Vec<PrimalCapability>> {
        let capabilities = self.peer_capabilities.read().await;
        capabilities.get(peer_id).cloned()
    }

    /// Get all registered peers
    pub async fn get_all_peers(&self) -> HashMap<String, DiscoveredPeer> {
        self.peers.read().await.clone()
    }

    /// Get peers by type
    pub async fn get_peers_by_type(&self, peer_type: PeerType) -> Vec<DiscoveredPeer> {
        let peers = self.peers.read().await;
        peers
            .values()
            .filter(|peer| {
                std::mem::discriminant(&peer.peer_type) == std::mem::discriminant(&peer_type)
            })
            .cloned()
            .collect()
    }

    /// Remove peer
    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        {
            let mut peers = self.peers.write().await;
            peers.remove(peer_id);
        }

        {
            let mut capabilities = self.peer_capabilities.write().await;
            capabilities.remove(peer_id);
        }

        {
            let mut last_seen = self.last_seen.write().await;
            last_seen.remove(peer_id);
        }

        debug!("Removed peer: {}", peer_id);
        Ok(())
    }

    /// Cleanup expired peers
    pub async fn cleanup_expired_peers(&self) {
        let mut expired_peers = Vec::new();

        {
            let last_seen = self.last_seen.read().await;
            let now = Instant::now();

            for (peer_id, last_seen_time) in last_seen.iter() {
                if now.duration_since(*last_seen_time) > self.config.peer_timeout {
                    expired_peers.push(peer_id.clone());
                }
            }
        }

        for peer_id in expired_peers {
            if let Err(e) = self.remove_peer(&peer_id).await {
                debug!("Failed to remove expired peer {}: {}", peer_id, e);
            } else {
                info!("Removed expired peer: {}", peer_id);
            }
        }
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }

    /// Check if peer exists
    pub async fn has_peer(&self, peer_id: &str) -> bool {
        let peers = self.peers.read().await;
        peers.contains_key(peer_id)
    }

    /// Get active peers (recently seen)
    pub async fn get_active_peers(&self) -> Vec<DiscoveredPeer> {
        let peers = self.peers.read().await;
        let last_seen = self.last_seen.read().await;
        let now = Instant::now();
        let timeout = self.config.peer_timeout;

        peers
            .values()
            .filter(|peer| {
                if let Some(last_seen_time) = last_seen.get(&peer.peer_id) {
                    now.duration_since(*last_seen_time) <= timeout
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Get peer statistics
    pub async fn get_peer_statistics(&self) -> PeerStatistics {
        let peers = self.peers.read().await;
        let last_seen = self.last_seen.read().await;
        let now = Instant::now();
        let timeout = self.config.peer_timeout;

        let total_peers = peers.len();
        let active_peers = peers
            .keys()
            .filter(|peer_id| {
                if let Some(last_seen_time) = last_seen.get(*peer_id) {
                    now.duration_since(*last_seen_time) <= timeout
                } else {
                    false
                }
            })
            .count();

        let mut peer_types = HashMap::new();
        for peer in peers.values() {
            let type_name = format!("{:?}", peer.peer_type);
            *peer_types.entry(type_name).or_insert(0) += 1;
        }

        PeerStatistics {
            total_peers,
            active_peers,
            expired_peers: total_peers - active_peers,
            peer_types,
        }
    }

    /// Clear all peers
    pub async fn clear_all_peers(&self) {
        {
            let mut peers = self.peers.write().await;
            peers.clear();
        }

        {
            let mut capabilities = self.peer_capabilities.write().await;
            capabilities.clear();
        }

        {
            let mut last_seen = self.last_seen.write().await;
            last_seen.clear();
        }

        info!("Cleared all registered peers");
    }

    /// Get peer last seen time
    pub async fn get_peer_last_seen(&self, peer_id: &str) -> Option<Instant> {
        let last_seen = self.last_seen.read().await;
        last_seen.get(peer_id).copied()
    }

    /// Update peer capabilities
    pub async fn update_peer_capabilities(
        &self,
        peer_id: &str,
        capabilities: Vec<PrimalCapability>,
    ) -> Result<()> {
        let mut caps = self.peer_capabilities.write().await;

        if caps.contains_key(peer_id) {
            caps.insert(peer_id.to_string(), capabilities);
            debug!("Updated capabilities for peer: {}", peer_id);
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::network_error(format!(
                "Peer not found: {peer_id}"
            )))
        }
    }

    /// Get gaming-optimized peers
    pub async fn get_gaming_peers(&self) -> Vec<(DiscoveredPeer, Vec<PrimalCapability>)> {
        let peers = self.peers.read().await;
        let capabilities = self.peer_capabilities.read().await;

        peers
            .values()
            .filter_map(|peer| {
                if let Some(caps) = capabilities.get(&peer.peer_id) {
                    // Check if peer has gaming optimization
                    let has_gaming = caps.iter().any(|cap| {
                        if let PrimalCapability::Custom { name, properties } = cap {
                            name == "Gaming"
                                && properties
                                    .iter()
                                    .any(|(k, v)| k == "optimized" && v == "true")
                        } else {
                            false
                        }
                    });

                    if has_gaming {
                        Some((peer.clone(), caps.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get best peers by latency
    pub async fn get_best_peers(
        &self,
        limit: usize,
    ) -> Vec<(DiscoveredPeer, Vec<PrimalCapability>)> {
        let peers = self.peers.read().await;
        let capabilities = self.peer_capabilities.read().await;

        let mut peer_caps: Vec<_> = peers
            .values()
            .filter_map(|peer| {
                capabilities
                    .get(&peer.peer_id)
                    .map(|caps| (peer.clone(), caps.clone()))
            })
            .collect();

        // Sort by latency (ascending)
        peer_caps.sort_by_key(|(_, caps)| {
            // Extract latency from NetworkConnectivity capability
            caps.iter()
                .find_map(|cap| {
                    if let PrimalCapability::Custom { name, properties } = cap {
                        if name == "NetworkConnectivity" {
                            properties
                                .iter()
                                .find(|(k, _)| k == "latency_ms")
                                .and_then(|(_, v)| v.parse::<f32>().ok())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(f32::MAX) as u32 // Default to high latency if not found
        });

        peer_caps.truncate(limit);
        peer_caps
    }
}

/// Peer registry statistics
#[derive(Debug, Clone)]
pub struct PeerStatistics {
    pub total_peers: usize,
    pub active_peers: usize,
    pub expired_peers: usize,
    pub peer_types: HashMap<String, usize>,
}
