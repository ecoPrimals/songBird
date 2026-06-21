// SPDX-License-Identifier: AGPL-3.0-or-later
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
                    crate::types::ConnectionType::TurnRelayed => "turn_relayed",
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
    /// Extracts `ConnectivityRequest` from the `params` field of a
    /// `PrimalRequest::Custom` variant; returns `PrimalResponse::Error`
    /// for non-Custom variants or unparseable params.
    ///
    /// # Errors
    ///
    /// Returns error if the underlying connectivity handler fails.
    pub async fn send_request(&self, request: PrimalRequest) -> Result<PrimalResponse> {
        let params = match request {
            PrimalRequest::Custom {
                params,
                ..
            } => params,
            other => {
                return Ok(PrimalResponse::Error(format!(
                    "Invalid connectivity request: expected Custom variant, got {:?}",
                    std::mem::discriminant(&other)
                )));
            }
        };

        match serde_json::from_value::<ConnectivityRequest>(params) {
            Ok(conn_request) => {
                let response = self.adapter.handle_request(conn_request).await?;
                Ok(PrimalResponse::Custom(serde_json::to_value(response)?))
            }
            Err(e) => Ok(PrimalResponse::Error(format!("Invalid connectivity request: {e}"))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::ip_constant)]
mod tests {
    use super::*;
    use crate::birdsong::BirdSongBroadcaster;
    use crate::coordinator::LineageRelayConfig;
    use crate::relay::RelayAuthority;
    use crate::security::{
        BirdSongCrypto, MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority,
    };

    async fn make_adapter() -> LineageRelayAdapter {
        let lineage_provider = Arc::new(MockLineageProvider::new());
        let crypto = Arc::new(BirdSongCrypto::from(MockBirdSongCrypto::new(
            lineage_provider.clone(),
            "node-1".to_string(),
        )));
        let relay_authority =
            Arc::new(RelayAuthority::from(MockRelayAuthority::new(lineage_provider)));

        let broadcaster = Arc::new(
            BirdSongBroadcaster::new(
                crypto,
                NodeId::from("node-1"),
                "127.0.0.1:0".parse().unwrap(),
                "127.0.0.1:0".parse().unwrap(),
            )
            .await
            .unwrap(),
        );

        let config = LineageRelayConfig::default();
        let coordinator = Arc::new(
            LineageRelayCoordinator::new(config, broadcaster, relay_authority).await.unwrap(),
        );

        LineageRelayAdapter::new(coordinator)
    }

    #[tokio::test]
    async fn test_relay_stats_request() {
        let adapter = make_adapter().await;
        let response = adapter.handle_request(ConnectivityRequest::GetRelayStats).await.unwrap();

        match response {
            ConnectivityResponse::RelayStats {
                active_relays,
                total_bytes_relayed,
            } => {
                assert_eq!(active_relays, 0);
                assert_eq!(total_bytes_relayed, 0);
            }
            _ => panic!("Expected RelayStats response"),
        }
    }

    #[tokio::test]
    async fn test_establish_connection_invalid_address() {
        let adapter = make_adapter().await;
        let result = adapter
            .handle_request(ConnectivityRequest::EstablishConnection {
                peer_id: "peer-42".into(),
                peer_address: "not-a-valid-address".into(),
            })
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid address"),
            "Expected 'Invalid address' in error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_establish_connection_invalid_port() {
        let adapter = make_adapter().await;
        let result = adapter
            .handle_request(ConnectivityRequest::EstablishConnection {
                peer_id: "peer-99".into(),
                peer_address: "192.168.1.1:99999".into(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_connection_send_request_invalid_json() {
        let adapter = make_adapter().await;
        let primal_conn = LineageRelayPrimalConnection::new(Arc::new(adapter));

        let bad_request = PrimalRequest::Custom {
            operation: "unknown_op".into(),
            params: serde_json::json!({"unknown_field": "not a ConnectivityRequest"}),
        };
        let response = primal_conn.send_request(bad_request).await.unwrap();

        match response {
            PrimalResponse::Error(msg) => {
                assert!(
                    msg.contains("Invalid connectivity request"),
                    "Expected parse error, got: {msg}"
                );
            }
            _ => panic!("Expected PrimalResponse::Error for invalid request"),
        }
    }

    #[tokio::test]
    async fn test_primal_connection_send_relay_stats() {
        let adapter = make_adapter().await;
        let primal_conn = LineageRelayPrimalConnection::new(Arc::new(adapter));

        let request = PrimalRequest::Custom {
            operation: "connectivity".into(),
            params: serde_json::to_value(ConnectivityRequest::GetRelayStats).unwrap(),
        };
        let response = primal_conn.send_request(request).await.unwrap();

        match response {
            PrimalResponse::Custom(val) => {
                let resp: ConnectivityResponse = serde_json::from_value(val).unwrap();
                match resp {
                    ConnectivityResponse::RelayStats {
                        active_relays,
                        total_bytes_relayed,
                    } => {
                        assert_eq!(active_relays, 0);
                        assert_eq!(total_bytes_relayed, 0);
                    }
                    _ => panic!("Expected RelayStats in Custom response"),
                }
            }
            _ => panic!("Expected PrimalResponse::Custom"),
        }
    }

    #[test]
    fn connectivity_request_serde_roundtrip() {
        let req = ConnectivityRequest::EstablishConnection {
            peer_id: "node-abc".into(),
            peer_address: "10.0.0.1:8080".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ConnectivityRequest = serde_json::from_str(&json).unwrap();
        match deserialized {
            ConnectivityRequest::EstablishConnection {
                peer_id,
                peer_address,
            } => {
                assert_eq!(peer_id, "node-abc");
                assert_eq!(peer_address, "10.0.0.1:8080");
            }
            ConnectivityRequest::GetRelayStats => panic!("Expected EstablishConnection"),
        }
    }

    #[test]
    fn connectivity_response_serde_roundtrip() {
        let resp = ConnectivityResponse::ConnectionEstablished {
            connection_type: "direct".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ConnectivityResponse = serde_json::from_str(&json).unwrap();
        match deserialized {
            ConnectivityResponse::ConnectionEstablished {
                connection_type,
            } => {
                assert_eq!(connection_type, "direct");
            }
            _ => panic!("Expected ConnectionEstablished"),
        }
    }

    #[test]
    fn connectivity_response_error_serde() {
        let resp = ConnectivityResponse::Error("something broke".into());
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ConnectivityResponse = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(deserialized, ConnectivityResponse::Error(msg) if msg == "something broke")
        );
    }
}
