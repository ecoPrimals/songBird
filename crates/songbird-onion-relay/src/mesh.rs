//! Distributed Beacon Mesh
//!
//! Every connected node becomes a potential relay for others.
//! The mesh grows organically - Tor is just bootstrap.
//!
//! ## How It Works
//!
//! 1. First device creates Tor onion (HPC beacon)
//! 2. Second device connects via Tor, exchanges addresses
//! 3. Both can now relay for others
//! 4. Third device can connect via EITHER of the first two
//! 5. Mesh keeps growing, Tor becomes fallback only
//!
//! ## Relay Selection Priority
//!
//! 1. Direct P2P (if hole punch succeeded)
//! 2. Family relay with best latency
//! 3. Any family relay available
//! 4. Tor onion (last resort)

use crate::error::{OnionRelayError, Result};
use crate::signaling::{NatType, PeerInfo, SignalingMessage};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// A relay endpoint (could be direct, family relay, or Tor)
#[derive(Debug, Clone)]
pub struct RelayEndpoint {
    /// Node ID of the relay
    pub node_id: String,
    /// How to reach this relay
    pub endpoint_type: EndpointType,
    /// Last measured latency
    pub latency: Option<Duration>,
    /// Last successful contact
    pub last_seen: Instant,
    /// Is this relay currently reachable?
    pub reachable: bool,
}

/// Type of relay endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointType {
    /// Direct UDP connection (hole punch succeeded)
    Direct {
        addr: SocketAddr,
    },
    /// Relay through another family member
    FamilyRelay {
        relay_node_id: String,
    },
    /// Tor onion service (bootstrap/fallback)
    TorOnion {
        onion_addr: String,
    },
    /// Local network (same LAN)
    Local {
        addr: SocketAddr,
    },
}

impl EndpointType {
    /// Priority for selection (lower = better)
    pub fn priority(&self) -> u8 {
        match self {
            EndpointType::Local {
                ..
            } => 0, // Best: same network
            EndpointType::Direct {
                ..
            } => 1, // Great: direct internet
            EndpointType::FamilyRelay {
                ..
            } => 2, // Good: through family
            EndpointType::TorOnion {
                ..
            } => 3, // Fallback: Tor latency
        }
    }
}

/// Beacon mesh state
pub struct BeaconMesh {
    /// Our node ID
    my_node_id: String,

    /// Known relay endpoints (node_id -> endpoints)
    /// A node might be reachable via multiple paths
    endpoints: RwLock<HashMap<String, Vec<RelayEndpoint>>>,

    /// Our Tor onion address (if we're running one)
    my_onion: RwLock<Option<String>>,

    /// Bootstrap onion addresses from beacon seed
    bootstrap_onions: Vec<String>,

    /// Current best path to each peer
    best_paths: RwLock<HashMap<String, RelayEndpoint>>,
}

impl BeaconMesh {
    /// Create new beacon mesh
    pub fn new(my_node_id: String, bootstrap_onions: Vec<String>) -> Self {
        Self {
            my_node_id,
            endpoints: RwLock::new(HashMap::new()),
            my_onion: RwLock::new(None),
            bootstrap_onions,
            best_paths: RwLock::new(HashMap::new()),
        }
    }

    /// Set our onion address (when we create one)
    pub async fn set_my_onion(&self, onion_addr: String) {
        info!("🧅 Beacon mesh: my onion = {}", &onion_addr[..16.min(onion_addr.len())]);
        *self.my_onion.write().await = Some(onion_addr);
    }

    /// Add a relay endpoint for a peer
    pub async fn add_endpoint(&self, node_id: String, endpoint: RelayEndpoint) {
        info!(
            "📍 Adding endpoint for {}: {:?} (priority {})",
            &node_id[..8.min(node_id.len())],
            endpoint.endpoint_type,
            endpoint.endpoint_type.priority()
        );

        {
            let mut endpoints = self.endpoints.write().await;
            endpoints.entry(node_id.clone()).or_default().push(endpoint.clone());
        }
        // Write lock dropped before acquiring read lock in update_best_path
        self.update_best_path(&node_id).await;
    }

    /// Record successful direct connection
    pub async fn record_direct_connection(
        &self,
        node_id: String,
        addr: SocketAddr,
        latency: Duration,
    ) {
        let endpoint = RelayEndpoint {
            node_id: node_id.clone(),
            endpoint_type: EndpointType::Direct {
                addr,
            },
            latency: Some(latency),
            last_seen: Instant::now(),
            reachable: true,
        };
        self.add_endpoint(node_id, endpoint).await;
    }

    /// Record successful family relay path
    pub async fn record_relay_path(
        &self,
        target_node_id: String,
        via_node_id: String,
        latency: Duration,
    ) {
        let endpoint = RelayEndpoint {
            node_id: target_node_id.clone(),
            endpoint_type: EndpointType::FamilyRelay {
                relay_node_id: via_node_id,
            },
            latency: Some(latency),
            last_seen: Instant::now(),
            reachable: true,
        };
        self.add_endpoint(target_node_id, endpoint).await;
    }

    /// Get best path to a peer
    pub async fn get_best_path(&self, node_id: &str) -> Option<RelayEndpoint> {
        self.best_paths.read().await.get(node_id).cloned()
    }

    /// Get all known paths to a peer (for fallback)
    pub async fn get_all_paths(&self, node_id: &str) -> Vec<RelayEndpoint> {
        self.endpoints.read().await.get(node_id).cloned().unwrap_or_default()
    }

    /// Find best relay to connect to a new peer
    pub async fn find_relay_for(&self, target_node_id: &str) -> Option<RelayEndpoint> {
        // 1. Check if we have direct path
        if let Some(path) = self.get_best_path(target_node_id).await {
            return Some(path);
        }

        // 2. Find a connected peer that might relay for us
        let endpoints = self.endpoints.read().await;
        let mut best_relay: Option<(String, &RelayEndpoint)> = None;

        for (node_id, eps) in endpoints.iter() {
            if node_id == target_node_id {
                continue;
            }

            for ep in eps {
                if !ep.reachable {
                    continue;
                }

                // Prefer lower priority (better connection type)
                if let Some((_, best_ep)) = &best_relay {
                    if ep.endpoint_type.priority() < best_ep.endpoint_type.priority() {
                        best_relay = Some((node_id.clone(), ep));
                    } else if ep.endpoint_type.priority() == best_ep.endpoint_type.priority() {
                        // Same priority - prefer lower latency
                        if let (Some(new_lat), Some(best_lat)) = (ep.latency, best_ep.latency) {
                            if new_lat < best_lat {
                                best_relay = Some((node_id.clone(), ep));
                            }
                        }
                    }
                } else {
                    best_relay = Some((node_id.clone(), ep));
                }
            }
        }

        if let Some((relay_node_id, _)) = best_relay {
            return Some(RelayEndpoint {
                node_id: target_node_id.to_string(),
                endpoint_type: EndpointType::FamilyRelay {
                    relay_node_id,
                },
                latency: None,
                last_seen: Instant::now(),
                reachable: true,
            });
        }

        // 3. Fall back to Tor bootstrap
        if !self.bootstrap_onions.is_empty() {
            return Some(RelayEndpoint {
                node_id: target_node_id.to_string(),
                endpoint_type: EndpointType::TorOnion {
                    onion_addr: self.bootstrap_onions[0].clone(),
                },
                latency: None,
                last_seen: Instant::now(),
                reachable: true,
            });
        }

        None
    }

    /// Get list of nodes we can relay for others
    pub async fn get_reachable_nodes(&self) -> Vec<String> {
        self.endpoints
            .read()
            .await
            .iter()
            .filter(|(_, eps)| eps.iter().any(|e| e.reachable))
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }

    /// Announce ourselves as relay to the mesh
    pub async fn announce_as_relay(&self) -> SignalingMessage {
        let reachable = self.get_reachable_nodes().await;
        let my_onion = self.my_onion.read().await.clone();

        info!(
            "📢 Announcing as relay: can reach {} nodes, onion: {:?}",
            reachable.len(),
            my_onion.as_ref().map(|o| &o[..16.min(o.len())])
        );

        SignalingMessage::Register {
            peer_info: PeerInfo {
                node_id: self.my_node_id.clone(),
                public_addr: "0.0.0.0:0".parse().unwrap(), // Will be filled by STUN
                local_addr: None,
                nat_type: NatType::Unknown,
                timestamp: SystemTime::now(),
                capabilities: vec!["relay".to_string(), format!("can_reach:{}", reachable.len())],
            },
            encrypted_beacon: my_onion,
        }
    }

    /// Handle relay request from another peer
    pub async fn handle_relay_request(
        &self,
        from_node_id: &str,
        to_node_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        // Check if we can reach the target
        if let Some(path) = self.get_best_path(to_node_id).await {
            info!(
                "🔄 Relaying {} bytes from {} to {} via {:?}",
                data.len(),
                &from_node_id[..8.min(from_node_id.len())],
                &to_node_id[..8.min(to_node_id.len())],
                path.endpoint_type
            );

            // Forward the data (actual implementation would use the path)
            Ok(())
        } else {
            warn!("❌ Cannot relay to {}: no path known", &to_node_id[..8.min(to_node_id.len())]);
            Err(OnionRelayError::PeerNotFound(to_node_id.to_string()))
        }
    }

    /// Periodic health check - update reachability
    pub async fn health_check(&self) {
        let now = Instant::now();
        let mut endpoints = self.endpoints.write().await;

        for (node_id, eps) in endpoints.iter_mut() {
            for ep in eps.iter_mut() {
                // Mark as unreachable if not seen in 60 seconds
                if now.duration_since(ep.last_seen) > Duration::from_secs(60) && ep.reachable {
                    debug!("📴 {} endpoint via {:?} marked unreachable", node_id, ep.endpoint_type);
                    ep.reachable = false;
                }
            }
        }
    }

    // --- Private methods ---

    async fn update_best_path(&self, node_id: &str) {
        let endpoints = self.endpoints.read().await;

        if let Some(eps) = endpoints.get(node_id) {
            // Find best reachable endpoint
            let best = eps
                .iter()
                .filter(|e| e.reachable)
                .min_by_key(|e| {
                    // Sort by priority, then latency
                    let priority = e.endpoint_type.priority() as u32 * 10000;
                    let latency = e.latency.map(|l| l.as_millis() as u32).unwrap_or(5000);
                    priority + latency
                })
                .cloned();

            drop(endpoints);

            if let Some(ep) = best {
                self.best_paths.write().await.insert(node_id.to_string(), ep);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mesh_creation() {
        let mesh = BeaconMesh::new("tower".to_string(), vec!["abc123.onion".to_string()]);

        assert_eq!(mesh.my_node_id, "tower");
        assert_eq!(mesh.bootstrap_onions.len(), 1);
    }

    #[tokio::test]
    async fn test_endpoint_priority() {
        assert!(
            EndpointType::Local {
                addr: "127.0.0.1:1234".parse().unwrap()
            }
            .priority()
                < EndpointType::Direct {
                    addr: "1.2.3.4:1234".parse().unwrap()
                }
                .priority()
        );

        assert!(
            EndpointType::Direct {
                addr: "1.2.3.4:1234".parse().unwrap()
            }
            .priority()
                < EndpointType::FamilyRelay {
                    relay_node_id: "relay".to_string()
                }
                .priority()
        );

        assert!(
            EndpointType::FamilyRelay {
                relay_node_id: "relay".to_string()
            }
            .priority()
                < EndpointType::TorOnion {
                    onion_addr: "abc.onion".to_string()
                }
                .priority()
        );
    }

    #[tokio::test]
    async fn test_add_and_find_path() {
        let mesh = BeaconMesh::new("tower".to_string(), vec![]);

        // Add direct endpoint for pixel
        mesh.record_direct_connection(
            "pixel".to_string(),
            "1.2.3.4:5678".parse().unwrap(),
            Duration::from_millis(50),
        )
        .await;

        // Should find it
        let path = mesh.get_best_path("pixel").await;
        assert!(path.is_some());
        assert!(matches!(path.unwrap().endpoint_type, EndpointType::Direct { .. }));
    }

    #[tokio::test]
    async fn test_relay_fallback() {
        let mesh = BeaconMesh::new("laptop".to_string(), vec!["bootstrap.onion".to_string()]);

        // Add connection to tower (direct)
        mesh.record_direct_connection(
            "tower".to_string(),
            "1.2.3.4:5678".parse().unwrap(),
            Duration::from_millis(30),
        )
        .await;

        // Ask for path to unknown "phone"
        let path = mesh.find_relay_for("phone").await;
        assert!(path.is_some());

        // Should suggest tower as relay (since it's connected)
        // OR fall back to Tor bootstrap
        let ep = path.unwrap();
        assert!(
            matches!(ep.endpoint_type, EndpointType::FamilyRelay { .. })
                || matches!(ep.endpoint_type, EndpointType::TorOnion { .. })
        );
    }
}
