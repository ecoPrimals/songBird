// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! [`HolePunchCoordinator`] construction, peer bookkeeping, and signaling dispatch.

use crate::signaling::{PeerInfo, SignalingMessage};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};

use super::config::HolePunchConfig;

/// Hole punch coordinator
///
/// Manages the hole punch process using a signaling channel
pub struct HolePunchCoordinator {
    /// Our node ID
    pub(crate) my_node_id: String,
    /// Our STUN-discovered info
    pub(crate) my_info: RwLock<Option<PeerInfo>>,
    /// Known peers from signaling
    pub(crate) peers: RwLock<HashMap<String, PeerInfo>>,
    /// Configuration
    pub(crate) config: HolePunchConfig,
    /// Channel to send signaling messages
    pub(crate) signal_tx: mpsc::Sender<SignalingMessage>,
    /// Channel to receive signaling messages
    pub(crate) signal_rx: RwLock<Option<mpsc::Receiver<SignalingMessage>>>,
}

impl HolePunchCoordinator {
    /// Builds a coordinator plus inbound/outbound signaling channels wired to it.
    #[must_use]
    pub fn new(
        my_node_id: String,
        config: HolePunchConfig,
    ) -> (Self, mpsc::Sender<SignalingMessage>, mpsc::Receiver<SignalingMessage>) {
        let (outbound_tx, outbound_rx) = mpsc::channel(100);
        let (inbound_tx, inbound_rx) = mpsc::channel(100);

        let coordinator = Self {
            my_node_id,
            my_info: RwLock::new(None),
            peers: RwLock::new(HashMap::new()),
            config,
            signal_tx: outbound_tx,
            signal_rx: RwLock::new(Some(inbound_rx)),
        };

        (coordinator, inbound_tx, outbound_rx)
    }

    /// Stores or replaces [`PeerInfo`] learned from the rendezvous channel.
    pub async fn register_peer(&self, peer_info: PeerInfo) {
        tracing::info!("📝 Registered peer: {} at {}", peer_info.node_id, peer_info.public_addr);
        self.peers.write().await.insert(peer_info.node_id.clone(), peer_info);
    }

    /// Returns the number of currently registered peers.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Dispatches rendezvous messages (register, query, punch, relay, etc.).
    pub async fn handle_message(&self, msg: SignalingMessage) -> Option<SignalingMessage> {
        match msg {
            SignalingMessage::Register {
                peer_info,
                ..
            } => {
                self.register_peer(peer_info).await;
                None
            }
            SignalingMessage::Query {
                target_node_id,
            } => {
                let peer_info = self.peers.read().await.get(&target_node_id).cloned();
                Some(SignalingMessage::PeerInfoResponse {
                    peer_info,
                })
            }
            SignalingMessage::PunchRequest {
                from,
                to_node_id,
                nonce,
            } => {
                if to_node_id == self.my_node_id {
                    // We're the target - send ack with coordinated time
                    self.register_peer(from.clone()).await;

                    let my_info = self.my_info.read().await.clone();
                    if let Some(info) = my_info {
                        // Start in 100ms to allow network propagation
                        let start_at_ms = u64::try_from(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis(),
                        )
                        .unwrap_or(0)
                            + 100;

                        return Some(SignalingMessage::PunchAck {
                            from: info,
                            nonce,
                            start_at_ms,
                        });
                    }
                }
                None
            }
            SignalingMessage::Heartbeat {
                node_id,
            } => {
                // Update timestamp for peer
                if let Some(peer) = self.peers.write().await.get_mut(&node_id) {
                    peer.timestamp = SystemTime::now();
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::signaling::{NatType, PeerInfo};
    use std::net::SocketAddr;

    fn test_peer(id: &str, addr: &str) -> PeerInfo {
        PeerInfo {
            node_id: id.to_string(),
            public_addr: addr.parse::<SocketAddr>().unwrap(),
            local_addr: None,
            nat_type: NatType::Unknown,
            timestamp: SystemTime::now(),
            capabilities: vec!["relay".to_string()],
        }
    }

    #[tokio::test]
    async fn new_creates_coordinator_with_empty_peers() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("node-a".into(), HolePunchConfig::default());
        assert_eq!(coord.my_node_id, "node-a");
        assert_eq!(coord.peer_count().await, 0);
        assert!(coord.my_info.read().await.is_none());
    }

    #[tokio::test]
    async fn register_peer_adds_to_registry() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        let peer = test_peer("peer-1", "192.0.2.1:4000");
        coord.register_peer(peer).await;
        assert_eq!(coord.peer_count().await, 1);
        let map = coord.peers.read().await;
        assert!(map.contains_key("peer-1"));
    }

    #[tokio::test]
    async fn register_peer_replaces_existing() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        coord.register_peer(test_peer("peer-1", "192.0.2.1:4000")).await;
        coord.register_peer(test_peer("peer-1", "192.0.2.1:5000")).await;
        assert_eq!(coord.peer_count().await, 1);
        let map = coord.peers.read().await;
        assert_eq!(map["peer-1"].public_addr.port(), 5000);
    }

    #[tokio::test]
    async fn handle_register_message_stores_peer() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        let msg = SignalingMessage::Register {
            peer_info: test_peer("peer-a", "10.0.0.1:3000"),
            encrypted_beacon: None,
        };
        let reply = coord.handle_message(msg).await;
        assert!(reply.is_none(), "Register returns no reply");
        assert_eq!(coord.peer_count().await, 1);
    }

    #[tokio::test]
    async fn handle_query_returns_known_peer() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        coord.register_peer(test_peer("peer-x", "10.0.0.2:5000")).await;

        let reply = coord
            .handle_message(SignalingMessage::Query {
                target_node_id: "peer-x".into(),
            })
            .await;
        match reply {
            Some(SignalingMessage::PeerInfoResponse {
                peer_info,
            }) => {
                let info = peer_info.unwrap();
                assert_eq!(info.node_id, "peer-x");
            }
            other => panic!("expected PeerInfoResponse, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_query_returns_none_for_unknown_peer() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        let reply = coord
            .handle_message(SignalingMessage::Query {
                target_node_id: "ghost".into(),
            })
            .await;
        match reply {
            Some(SignalingMessage::PeerInfoResponse {
                peer_info,
            }) => {
                assert!(peer_info.is_none());
            }
            other => panic!("expected PeerInfoResponse with None, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_heartbeat_updates_timestamp() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        let old_peer = test_peer("peer-h", "10.0.0.3:6000");
        coord.register_peer(old_peer).await;

        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        let reply = coord
            .handle_message(SignalingMessage::Heartbeat {
                node_id: "peer-h".into(),
            })
            .await;
        assert!(reply.is_none(), "Heartbeat returns no reply");
    }

    #[tokio::test]
    async fn handle_punch_request_for_self_returns_ack_when_info_set() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self-node".into(), HolePunchConfig::default());

        let my_info = test_peer("self-node", "203.0.113.1:7000");
        *coord.my_info.write().await = Some(my_info);

        let test_nonce = [7u8; 16];
        let reply = coord
            .handle_message(SignalingMessage::PunchRequest {
                from: test_peer("initiator", "198.51.100.1:8000"),
                to_node_id: "self-node".into(),
                nonce: test_nonce,
            })
            .await;
        match reply {
            Some(SignalingMessage::PunchAck {
                from,
                nonce,
                start_at_ms,
            }) => {
                assert_eq!(from.node_id, "self-node");
                assert_eq!(nonce, test_nonce);
                assert!(start_at_ms > 0);
            }
            other => panic!("expected PunchAck, got {other:?}"),
        }
        assert_eq!(coord.peer_count().await, 1, "initiator should be registered");
    }

    #[tokio::test]
    async fn handle_punch_request_for_other_returns_none() {
        let (coord, _in_tx, _out_rx) =
            HolePunchCoordinator::new("self".into(), HolePunchConfig::default());
        let reply = coord
            .handle_message(SignalingMessage::PunchRequest {
                from: test_peer("initiator", "198.51.100.1:8000"),
                to_node_id: "someone-else".into(),
                nonce: [1u8; 16],
            })
            .await;
        assert!(reply.is_none());
    }
}
