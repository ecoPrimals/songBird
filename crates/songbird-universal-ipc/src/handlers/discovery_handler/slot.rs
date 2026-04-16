// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Concrete peer-registry dispatch (replaces `Arc<dyn PeerRegistry>` for native async traits).

use std::sync::Arc;

use crate::error::IpcResult;
use crate::handlers::discovery_bridge::DiscoveryListenerBridge;
use crate::handlers::discovery_handler::types::{DiscoveredPeerInfo, PeerRegistry};

/// Wired peer registry for [`super::DiscoveryHandler`] (enum dispatch, not `dyn`).
pub enum PeerRegistrySlot {
    /// Orchestrator discovery listener bridge.
    Bridge(Arc<DiscoveryListenerBridge>),
    /// Unit tests.
    #[cfg(test)]
    Mock(Arc<super::types::MockPeerRegistry>),
}

impl PeerRegistrySlot {
    pub(crate) async fn get_all_peers(&self) -> IpcResult<Vec<DiscoveredPeerInfo>> {
        match self {
            Self::Bridge(b) => <DiscoveryListenerBridge as PeerRegistry>::get_all_peers(b).await,
            #[cfg(test)]
            Self::Mock(m) => {
                <super::types::MockPeerRegistry as PeerRegistry>::get_all_peers(m).await
            }
        }
    }

    pub(crate) async fn get_peer(&self, peer_id: &str) -> IpcResult<Option<DiscoveredPeerInfo>> {
        match self {
            Self::Bridge(b) => {
                <DiscoveryListenerBridge as PeerRegistry>::get_peer(b, peer_id).await
            }
            #[cfg(test)]
            Self::Mock(m) => {
                <super::types::MockPeerRegistry as PeerRegistry>::get_peer(m, peer_id).await
            }
        }
    }
}
