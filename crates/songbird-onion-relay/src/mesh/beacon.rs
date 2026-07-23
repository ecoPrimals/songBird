// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Beacon mesh state machine: endpoint table, best-path selection, relay announcements, health.

use crate::error::{OnionRelayError, Result};
use crate::signaling::{NatType, PeerInfo, SignalingMessage};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{EndpointType, RelayEndpoint};

/// Public socket placeholder for relay registration until STUN overwrites reachability.
const RELAY_REGISTER_PUBLIC_ADDR: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));

/// Tracks relay endpoints and best paths for mesh connectivity.
pub struct BeaconMesh {
    /// Our node ID
    pub(super) my_node_id: String,

    /// Known relay endpoints (`node_id` -> endpoints)
    pub(super) endpoints: RwLock<HashMap<String, Vec<RelayEndpoint>>>,

    /// Our Tor onion address (if we're running one)
    my_onion: RwLock<Option<String>>,

    /// Bootstrap onion addresses from beacon seed
    pub(super) bootstrap_onions: Vec<String>,

    /// Current best path to each peer
    best_paths: RwLock<HashMap<String, RelayEndpoint>>,
}

impl BeaconMesh {
    /// Create new beacon mesh
    #[must_use]
    pub fn new(my_node_id: String, bootstrap_onions: Vec<String>) -> Self {
        Self {
            my_node_id,
            endpoints: RwLock::new(HashMap::new()),
            my_onion: RwLock::new(None),
            bootstrap_onions,
            best_paths: RwLock::new(HashMap::new()),
        }
    }

    /// Our node ID.
    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.my_node_id
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

    /// Record successful overlay (WireGuard/VPN) connection with measured latency
    pub async fn record_overlay_connection(
        &self,
        node_id: String,
        addr: SocketAddr,
        overlay_name: &str,
        latency: Duration,
    ) {
        let endpoint = RelayEndpoint {
            node_id: node_id.clone(),
            endpoint_type: EndpointType::Overlay {
                addr,
                overlay_name: String::from(overlay_name),
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
        if let Some(path) = self.get_best_path(target_node_id).await {
            return Some(path);
        }

        let endpoints_snapshot = {
            let guard = self.endpoints.read().await;
            guard.clone()
        };
        let relay_node_id = {
            let mut best_relay: Option<(String, &RelayEndpoint)> = None;

            for (node_id, eps) in &endpoints_snapshot {
                if node_id == target_node_id {
                    continue;
                }

                for ep in eps {
                    if !ep.reachable {
                        continue;
                    }

                    if let Some((_, best_ep)) = &best_relay {
                        if ep.endpoint_type.priority() < best_ep.endpoint_type.priority() {
                            best_relay = Some((node_id.clone(), ep));
                        } else if ep.endpoint_type.priority() == best_ep.endpoint_type.priority()
                            && let (Some(new_lat), Some(best_lat)) = (ep.latency, best_ep.latency)
                            && new_lat < best_lat
                        {
                            best_relay = Some((node_id.clone(), ep));
                        }
                    } else {
                        best_relay = Some((node_id.clone(), ep));
                    }
                }
            }

            best_relay.map(|(id, _)| id)
        };

        if let Some(relay_node_id) = relay_node_id {
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

    /// Get all known node IDs (reachable and unreachable).
    pub async fn get_known_nodes(&self) -> Vec<String> {
        self.endpoints.read().await.keys().cloned().collect()
    }

    /// Remove a peer from the mesh (all endpoints and best-path cache).
    ///
    /// Returns `true` if the peer existed and was removed.
    pub async fn remove_peer(&self, node_id: &str) -> bool {
        let removed_endpoints = self.endpoints.write().await.remove(node_id).is_some();
        let removed_best = self.best_paths.write().await.remove(node_id).is_some();
        if removed_endpoints || removed_best {
            info!(peer = node_id, "Removed peer from mesh");
        }
        removed_endpoints || removed_best
    }

    /// Backdate `last_seen` for health-check simulation and integration tests.
    pub async fn backdate_endpoint_last_seen(&self, node_id: &str, age: Duration) {
        let mut endpoints = self.endpoints.write().await;
        if let Some(eps) = endpoints.get_mut(node_id) {
            for ep in eps.iter_mut() {
                if let Some(stale) = ep.last_seen.checked_sub(age) {
                    ep.last_seen = stale;
                }
            }
        }
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
                public_addr: RELAY_REGISTER_PUBLIC_ADDR,
                local_addr: None,
                nat_type: NatType::Unknown,
                timestamp: SystemTime::now(),
                capabilities: vec![String::from("relay"), format!("can_reach:{}", reachable.len())],
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
        if let Some(path) = self.get_best_path(to_node_id).await {
            info!(
                "🔄 Relaying {} bytes from {} to {} via {:?}",
                data.len(),
                &from_node_id[..8.min(from_node_id.len())],
                &to_node_id[..8.min(to_node_id.len())],
                path.endpoint_type
            );

            Ok(())
        } else {
            warn!("❌ Cannot relay to {}: no path known", &to_node_id[..8.min(to_node_id.len())]);
            Err(OnionRelayError::PeerNotFound(to_node_id.to_string()))
        }
    }

    /// Periodic health check - update reachability
    pub async fn health_check(&self) {
        let now = Instant::now();
        let mut affected = Vec::new();

        {
            let mut endpoints = self.endpoints.write().await;

            for (node_id, eps) in endpoints.iter_mut() {
                for ep in eps.iter_mut() {
                    if now.duration_since(ep.last_seen) > Duration::from_secs(60) && ep.reachable {
                        debug!(
                            "📴 {} endpoint via {:?} marked unreachable",
                            node_id, ep.endpoint_type
                        );
                        ep.reachable = false;
                        affected.push(node_id.clone());
                    }
                }
            }
        }

        for node_id in affected {
            self.update_best_path(&node_id).await;
        }
    }

    async fn update_best_path(&self, node_id: &str) {
        let endpoints = self.endpoints.read().await;

        if let Some(eps) = endpoints.get(node_id) {
            let best = eps
                .iter()
                .filter(|e| e.reachable)
                .min_by_key(|e| {
                    let priority = u32::from(e.endpoint_type.priority()) * 10000;
                    let latency =
                        e.latency.map_or(5000, |l| u32::try_from(l.as_millis()).unwrap_or(5000));
                    priority + latency
                })
                .cloned();

            drop(endpoints);

            let mut best_paths = self.best_paths.write().await;
            if let Some(ep) = best {
                best_paths.insert(node_id.to_string(), ep);
            } else {
                best_paths.remove(node_id);
            }
        }
    }
}

#[cfg(test)]
#[path = "beacon_tests.rs"]
mod tests;
