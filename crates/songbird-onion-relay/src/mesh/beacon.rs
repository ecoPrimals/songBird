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
        let mut endpoints = self.endpoints.write().await;

        for (node_id, eps) in endpoints.iter_mut() {
            for ep in eps.iter_mut() {
                if now.duration_since(ep.last_seen) > Duration::from_secs(60) && ep.reachable {
                    debug!("📴 {} endpoint via {:?} marked unreachable", node_id, ep.endpoint_type);
                    ep.reachable = false;
                }
            }
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

            if let Some(ep) = best {
                self.best_paths.write().await.insert(node_id.to_string(), ep);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::super::types::{EndpointType, RelayEndpoint};
    use super::*;
    use crate::signaling::SignalingMessage;
    use std::time::{Duration, Instant};

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

        mesh.record_direct_connection(
            "pixel".to_string(),
            "1.2.3.4:5678".parse().unwrap(),
            Duration::from_millis(50),
        )
        .await;

        let path = mesh.get_best_path("pixel").await;
        assert!(path.is_some());
        assert!(matches!(path.unwrap().endpoint_type, EndpointType::Direct { .. }));
    }

    #[tokio::test]
    async fn test_relay_fallback() {
        let mesh = BeaconMesh::new("laptop".to_string(), vec!["bootstrap.onion".to_string()]);

        mesh.record_direct_connection(
            "tower".to_string(),
            "1.2.3.4:5678".parse().unwrap(),
            Duration::from_millis(30),
        )
        .await;

        let path = mesh.find_relay_for("phone").await;
        assert!(path.is_some());

        let ep = path.unwrap();
        assert!(
            matches!(ep.endpoint_type, EndpointType::FamilyRelay { .. })
                || matches!(ep.endpoint_type, EndpointType::TorOnion { .. })
        );
    }

    #[tokio::test]
    async fn set_my_onion_and_announce_register_shape() {
        let mesh = BeaconMesh::new("me".into(), vec![]);
        mesh.set_my_onion("abcd1234efgh5678ijkl9012mnop3456qrst7890uvwx.onion".into()).await;
        let msg = mesh.announce_as_relay().await;
        match msg {
            SignalingMessage::Register {
                peer_info,
                encrypted_beacon,
            } => {
                assert_eq!(peer_info.node_id, "me");
                assert!(peer_info.capabilities.iter().any(|c| c.starts_with("can_reach:")));
                assert_eq!(
                    encrypted_beacon,
                    Some("abcd1234efgh5678ijkl9012mnop3456qrst7890uvwx.onion".into())
                );
            }
            other => panic!("expected Register, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_all_paths_and_best_prefers_lower_priority() {
        let mesh = BeaconMesh::new("hub".into(), vec![]);
        let addr = "10.0.0.5:9000".parse().unwrap();
        mesh.record_direct_connection("peer".into(), addr, Duration::from_millis(10)).await;
        mesh.record_relay_path("peer".into(), "via".into(), Duration::from_millis(5)).await;

        let paths = mesh.get_all_paths("peer").await;
        assert_eq!(paths.len(), 2, "both endpoints recorded");

        let best = mesh.get_best_path("peer").await.expect("best path");
        assert!(
            matches!(best.endpoint_type, EndpointType::Direct { .. }),
            "direct should beat family relay: {:?}",
            best.endpoint_type
        );
    }

    #[tokio::test]
    async fn find_relay_for_unknown_peer_without_bootstrap_returns_none() {
        let mesh = BeaconMesh::new("solo".into(), vec![]);
        assert!(mesh.find_relay_for("nobody").await.is_none(), "no relays and no bootstrap → None");
    }

    #[tokio::test]
    async fn find_relay_for_prefers_reachable_lower_priority_endpoint() {
        let mesh = BeaconMesh::new("me".into(), vec![]);
        mesh.record_relay_path("target".into(), "r1".into(), Duration::from_millis(100)).await;
        mesh.record_direct_connection(
            "helper".into(),
            "1.1.1.1:1".parse().unwrap(),
            Duration::from_millis(20),
        )
        .await;

        let path = mesh.find_relay_for("target").await.expect("helper or bootstrap path");
        assert!(
            matches!(path.endpoint_type, EndpointType::FamilyRelay { .. }),
            "expected family relay toward target, got {:?}",
            path.endpoint_type
        );
    }

    #[tokio::test]
    async fn handle_relay_request_ok_and_peer_not_found() {
        let mesh = BeaconMesh::new("relay".into(), vec![]);
        mesh.record_direct_connection("dest".into(), "8.8.8.8:53".parse().unwrap(), Duration::ZERO)
            .await;

        mesh.handle_relay_request("src", "dest", vec![1, 2, 3]).await.expect("path to dest exists");

        let err = mesh.handle_relay_request("src", "missing", vec![]).await.expect_err("no path");
        assert!(matches!(err, crate::OnionRelayError::PeerNotFound(_)));
    }

    #[tokio::test(start_paused = true)]
    async fn health_check_marks_stale_unreachable() {
        let mesh = BeaconMesh::new("n".into(), vec![]);
        let ep = RelayEndpoint {
            node_id: "p".into(),
            endpoint_type: EndpointType::Direct {
                addr: "1.1.1.1:1".parse().unwrap(),
            },
            latency: None,
            last_seen: Instant::now()
                .checked_sub(Duration::from_secs(120))
                .expect("instant far enough after epoch for subtraction"),
            reachable: true,
        };
        {
            let mut map = mesh.endpoints.write().await;
            map.insert("p".into(), vec![ep.clone()]);
        }

        mesh.health_check().await;

        let eps = mesh.get_all_paths("p").await;
        assert_eq!(eps.len(), 1);
        assert!(!eps[0].reachable, "endpoint older than 60s should be marked unreachable");
    }

    #[tokio::test]
    async fn get_reachable_nodes_filters_unreachable() {
        let mesh = BeaconMesh::new("n".into(), vec![]);
        let mut ep = RelayEndpoint {
            node_id: "up".into(),
            endpoint_type: EndpointType::Direct {
                addr: "2.2.2.2:2".parse().unwrap(),
            },
            latency: Some(Duration::from_millis(1)),
            last_seen: Instant::now(),
            reachable: true,
        };
        {
            let mut map = mesh.endpoints.write().await;
            map.insert("up".into(), vec![ep.clone()]);
            ep.reachable = false;
            map.insert("down".into(), vec![ep]);
        }

        let nodes = mesh.get_reachable_nodes().await;
        assert_eq!(nodes, vec!["up".to_string()]);
    }

    #[tokio::test]
    async fn endpoint_type_priority_ordering() {
        assert_eq!(
            EndpointType::Local {
                addr: "127.0.0.1:1".parse().unwrap()
            }
            .priority(),
            0
        );
        assert_eq!(
            EndpointType::Direct {
                addr: "1.1.1.1:1".parse().unwrap()
            }
            .priority(),
            1
        );
        assert_eq!(
            EndpointType::FamilyRelay {
                relay_node_id: "r".into()
            }
            .priority(),
            2
        );
        assert_eq!(
            EndpointType::TorOnion {
                onion_addr: "x.onion".into()
            }
            .priority(),
            3
        );
    }

    #[tokio::test]
    async fn find_relay_for_skips_unreachable_candidate_endpoints() {
        let mesh = BeaconMesh::new("me".into(), vec![]);
        mesh.add_endpoint(
            "fast_but_down".into(),
            RelayEndpoint {
                node_id: "fast_but_down".into(),
                endpoint_type: EndpointType::Direct {
                    addr: "9.9.9.9:9".parse().unwrap(),
                },
                latency: Some(Duration::from_millis(1)),
                last_seen: Instant::now(),
                reachable: false,
            },
        )
        .await;
        mesh.record_direct_connection(
            "slow_but_up".into(),
            "8.8.8.8:8".parse().unwrap(),
            Duration::from_millis(80),
        )
        .await;

        let path = mesh.find_relay_for("stranger").await.expect("reachable helper exists");
        assert!(
            matches!(
                &path.endpoint_type,
                EndpointType::FamilyRelay {
                    relay_node_id
                } if relay_node_id == "slow_but_up"
            ),
            "unreachable endpoints must not be selected as relay helpers: {:?}",
            path.endpoint_type
        );
    }

    #[tokio::test]
    async fn find_relay_for_prefers_lower_latency_when_priority_matches() {
        let mesh = BeaconMesh::new("me".into(), vec![]);
        mesh.record_direct_connection(
            "higher_latency".into(),
            "1.1.1.1:1".parse().unwrap(),
            Duration::from_millis(90),
        )
        .await;
        mesh.record_direct_connection(
            "lower_latency".into(),
            "2.2.2.2:2".parse().unwrap(),
            Duration::from_millis(12),
        )
        .await;

        let path = mesh.find_relay_for("unknown_peer").await.expect("two helpers registered");
        assert!(
            matches!(
                &path.endpoint_type,
                EndpointType::FamilyRelay {
                    relay_node_id
                } if relay_node_id == "lower_latency"
            ),
            "expected lower-latency direct path to win tie-break: {:?}",
            path.endpoint_type
        );
    }

    #[tokio::test]
    async fn find_relay_for_returns_best_path_when_target_has_known_route() {
        let mesh = BeaconMesh::new("me".into(), vec!["boot.onion".into()]);
        mesh.record_direct_connection(
            "pixel".into(),
            "10.0.0.1:9000".parse().unwrap(),
            Duration::from_millis(5),
        )
        .await;

        let direct = mesh.find_relay_for("pixel").await.expect("direct path registered");
        assert!(
            matches!(direct.endpoint_type, EndpointType::Direct { .. }),
            "should return stored best path, got {:?}",
            direct.endpoint_type
        );
    }

    #[tokio::test]
    async fn add_endpoint_family_then_direct_updates_best_to_direct() {
        let mesh = BeaconMesh::new("hub".into(), vec![]);
        mesh.record_relay_path("peer".into(), "via".into(), Duration::from_millis(2)).await;
        mesh.record_direct_connection(
            "peer".into(),
            "192.0.2.1:1".parse().unwrap(),
            Duration::from_millis(40),
        )
        .await;

        let best = mesh.get_best_path("peer").await.expect("best exists");
        assert!(
            matches!(best.endpoint_type, EndpointType::Direct { .. }),
            "direct should replace family relay in best-path table: {:?}",
            best.endpoint_type
        );
    }

    #[tokio::test]
    async fn handle_relay_request_fails_when_only_unreachable_endpoints_exist() {
        let mesh = BeaconMesh::new("relay".into(), vec![]);
        mesh.add_endpoint(
            "gone".into(),
            RelayEndpoint {
                node_id: "gone".into(),
                endpoint_type: EndpointType::Direct {
                    addr: "198.51.100.1:1".parse().unwrap(),
                },
                latency: None,
                last_seen: Instant::now(),
                reachable: false,
            },
        )
        .await;

        assert!(
            mesh.get_best_path("gone").await.is_none(),
            "unreachable-only endpoints must not populate best_paths"
        );

        let err =
            mesh.handle_relay_request("src", "gone", vec![9]).await.expect_err("no reachable path");
        assert!(matches!(err, crate::OnionRelayError::PeerNotFound(_)));
    }

    #[tokio::test]
    async fn get_all_paths_returns_empty_for_unknown_peer() {
        let mesh = BeaconMesh::new("solo".into(), vec![]);
        assert!(mesh.get_all_paths("nope").await.is_empty());
    }
}
