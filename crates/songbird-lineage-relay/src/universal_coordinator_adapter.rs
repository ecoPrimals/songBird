// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Coordinator adapter for lineage relay
//!
//! Makes lineage relay available as a "connectivity" capability

use crate::coordinator::LineageRelayCoordinator;
use crate::error::{LineageRelayError, Result};
use crate::types::NodeId;
use serde::{Deserialize, Serialize};
use songbird_primal_coordination::types::{PrimalRequest, PrimalResponse};
use std::net::SocketAddr;
use std::sync::Arc;

/// Connectivity request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectivityRequest {
    /// Establish connection to peer
    EstablishConnection {
        /// Target [`NodeId`] string as used in coordination RPCs.
        peer_id: String,
        /// Parsed later as [`SocketAddr`] for the first connectivity attempt.
        peer_address: String,
    },
    /// Get relay statistics
    GetRelayStats,
}

/// Connectivity response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectivityResponse {
    /// Direct or relayed path is ready for application traffic.
    ConnectionEstablished {
        /// `"direct"` or `"relayed"` for operator dashboards.
        connection_type: String, // "direct" or "relayed"
    },
    /// Relay statistics
    RelayStats {
        /// Number of live relay sessions tracked by the coordinator.
        active_relays: usize,
        /// Sum of bytes forwarded across all relay sessions.
        total_bytes_relayed: u64,
    },
    /// Operation failed; carries a human-readable reason for the coordinator UI.
    Error(String),
}

/// Universal Coordinator adapter for lineage relay
pub struct LineageRelayAdapter {
    coordinator: Arc<LineageRelayCoordinator>,
}

impl LineageRelayAdapter {
    /// Create new adapter
    #[must_use]
    pub const fn new(coordinator: Arc<LineageRelayCoordinator>) -> Self {
        Self {
            coordinator,
        }
    }

    /// Handle connectivity request from Universal Coordinator
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn handle_request(
        &self,
        request: ConnectivityRequest,
    ) -> Result<ConnectivityResponse> {
        match request {
            ConnectivityRequest::EstablishConnection {
                peer_id,
                peer_address,
            } => {
                let peer = NodeId::from(peer_id);
                let address: SocketAddr = peer_address
                    .parse()
                    .map_err(|e| LineageRelayError::Other(format!("Invalid address: {e}")))?;

                let connection = self.coordinator.establish_connection(peer, address).await?;

                let connection_type = match connection.connection_type() {
                    crate::types::ConnectionType::Direct => "direct",
                    crate::types::ConnectionType::Relayed => "relayed",
                    crate::types::ConnectionType::Upgrading => "upgrading",
                };

                Ok(ConnectivityResponse::ConnectionEstablished {
                    connection_type: connection_type.to_string(),
                })
            }
            ConnectivityRequest::GetRelayStats => {
                let stats = self.coordinator.relay_stats().await;
                let total_bytes: u64 = stats.iter().map(|(_, bytes)| bytes).sum();

                Ok(ConnectivityResponse::RelayStats {
                    active_relays: stats.len(),
                    total_bytes_relayed: total_bytes,
                })
            }
        }
    }
}

/// Wrapper to integrate with Universal Coordinator's `PrimalConnection`
pub struct LineageRelayPrimalConnection {
    adapter: Arc<LineageRelayAdapter>,
}

impl LineageRelayPrimalConnection {
    /// Create new primal connection wrapper
    #[must_use]
    pub const fn new(adapter: Arc<LineageRelayAdapter>) -> Self {
        Self {
            adapter,
        }
    }

    /// Send request (for Universal Coordinator integration)
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn send_request(&self, request: PrimalRequest) -> Result<PrimalResponse> {
        // Extract connectivity request from generic primal request
        // This would be more sophisticated in production
        match serde_json::from_slice::<ConnectivityRequest>(&serde_json::to_vec(&request)?) {
            Ok(conn_request) => {
                let response = self.adapter.handle_request(conn_request).await?;
                Ok(PrimalResponse::Custom(serde_json::to_value(response)?))
            }
            Err(e) => Ok(PrimalResponse::Error(format!("Invalid connectivity request: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birdsong::BirdSongBroadcaster;
    use crate::coordinator::LineageRelayConfig;
    use crate::security::{MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority};

    #[tokio::test]
    async fn test_universal_coordinator_adapter() {
        let lineage_provider = Arc::new(MockLineageProvider::new());
        let crypto =
            Arc::new(MockBirdSongCrypto::new(lineage_provider.clone(), "node-1".to_string()));
        let relay_authority = Arc::new(MockRelayAuthority::new(lineage_provider));

        let broadcaster = Arc::new(
            BirdSongBroadcaster::new(
                crypto,
                NodeId::from("node-1"),
                "127.0.0.1:42600".parse().unwrap(),
                "255.255.255.255:42600".parse().unwrap(),
            )
            .await
            .unwrap(),
        );

        let config = LineageRelayConfig::default();
        let coordinator = Arc::new(
            LineageRelayCoordinator::new(config, broadcaster, relay_authority).await.unwrap(),
        );

        let adapter = LineageRelayAdapter::new(coordinator);

        // Test relay stats request
        let response = adapter.handle_request(ConnectivityRequest::GetRelayStats).await.unwrap();

        match response {
            ConnectivityResponse::RelayStats {
                active_relays,
                ..
            } => {
                assert_eq!(active_relays, 0); // No active relays yet
            }
            _ => panic!("Expected RelayStats response"),
        }
    }
}
