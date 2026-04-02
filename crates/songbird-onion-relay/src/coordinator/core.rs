// SPDX-License-Identifier: AGPL-3.0-only
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
