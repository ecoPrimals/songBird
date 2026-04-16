// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Peer Connection Handler for JSON-RPC
//!
//! Handles `peer.connect` method for direct peer-to-peer connections using UDP hole punching.
//!
//! ## Method
//! - `peer.connect` - Initiate direct connection to peer (hole punching)
//!
//! ## Architecture
//! Uses enum dispatch ([`PeerConnector`](crate::handlers::udp_peer_connector::PeerConnector)) for production UDP and test doubles.

use crate::error::{IpcError, IpcResult};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

pub use super::peer_types::{PeerChannel, PeerConnectParams, PeerConnectResult};

use super::udp_peer_connector::PeerConnector;

// ============================================================================
// Peer Handler
// ============================================================================

pub struct PeerHandler {
    connector: Arc<PeerConnector>,
}

impl PeerHandler {
    /// Create new handler with given connector
    #[must_use]
    pub fn new(connector: Arc<PeerConnector>) -> Self {
        Self {
            connector,
        }
    }

    /// Handle peer.connect
    pub async fn handle_connect(&self, params: Value) -> IpcResult<PeerConnectResult> {
        let params: PeerConnectParams =
            serde_json::from_value(params).map_err(|e| IpcError::InvalidParams(e.to_string()))?;

        info!(
            "🔗 Initiating peer connection to: {} (binding: {:?}, token: {:?})",
            params.target_address, params.our_binding, params.rendezvous_token
        );

        let result = self
            .connector
            .connect(
                &params.target_address,
                params.our_binding.as_deref(),
                params.rendezvous_token.as_deref(),
            )
            .await
            .map_err(|e| IpcError::Internal(format!("Peer connection failed: {e}")))?;

        match result.state.as_str() {
            "connected" => {
                info!("✅ Peer connected successfully (connection_id: {})", result.connection_id);
            }
            "connecting" => {
                info!("🔄 Peer connection in progress (connection_id: {})", result.connection_id);
            }
            "failed" => {
                warn!("❌ Peer connection failed (connection_id: {})", result.connection_id);
            }
            _ => warn!("⚠️  Unknown connection state: {}", result.state),
        }

        Ok(result)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::handlers::udp_peer_connector::MockPeerConnector;
    use serde_json::json;

    #[tokio::test]
    async fn test_connect_success() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());
        assert!(result.channel.is_some());

        let channel = result.channel.unwrap();
        assert_eq!(channel.remote_address, "203.0.113.45:54321");
        assert_eq!(channel.protocol, "udp");
        assert!(channel.latency_ms.is_some());
    }

    #[tokio::test]
    async fn test_connect_with_binding() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321",
            "our_binding": "0.0.0.0:5000"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        let channel = result.channel.unwrap();
        assert_eq!(channel.local_address, "0.0.0.0:5000");
    }

    #[tokio::test]
    async fn test_connect_with_rendezvous_token() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321",
            "rendezvous_token": "token-abc123"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());
    }

    #[tokio::test]
    async fn test_connect_failure() {
        let inner = MockPeerConnector::new();
        inner.set_should_succeed(false);
        let connector = Arc::new(PeerConnector::Mock(inner));

        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.45:54321"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "failed");
        assert!(result.channel.is_none());
    }

    #[tokio::test]
    async fn test_connect_missing_params() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            // Missing target_address
        });

        let result = handler.handle_connect(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_all_params() {
        let connector = Arc::new(PeerConnector::Mock(MockPeerConnector::new()));
        let handler = PeerHandler::new(connector);

        let params = json!({
            "target_address": "203.0.113.100:6000",
            "our_binding": "192.168.1.10:5000",
            "rendezvous_token": "token-xyz789"
        });

        let result = handler.handle_connect(params).await.unwrap();

        assert_eq!(result.state, "connected");
        assert!(!result.connection_id.is_empty());

        let channel = result.channel.unwrap();
        assert_eq!(channel.local_address, "192.168.1.10:5000");
        assert_eq!(channel.remote_address, "203.0.113.100:6000");
    }

    #[test]
    fn peer_connect_params_json_roundtrip() {
        let p = PeerConnectParams {
            target_address: "198.51.100.2:4000".into(),
            our_binding: Some("0.0.0.0:5000".into()),
            rendezvous_token: Some("tok".into()),
        };
        let v = serde_json::to_value(&p).expect("serialize");
        let back: PeerConnectParams = serde_json::from_value(v).expect("deserialize");
        assert_eq!(back.target_address, p.target_address);
        assert_eq!(back.our_binding, p.our_binding);
        assert_eq!(back.rendezvous_token, p.rendezvous_token);
    }

    #[tokio::test]
    async fn connect_propagates_connector_error() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::ErrorSim));
        let err = handler
            .handle_connect(json!({ "target_address": "127.0.0.1:1" }))
            .await
            .expect_err("connector error");
        match err {
            IpcError::Internal(msg) => assert!(msg.contains("simulated")),
            _ => panic!("expected Internal error"),
        }
    }

    #[tokio::test]
    async fn connect_invalid_params_type_errors() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let err = handler
            .handle_connect(json!({ "target_address": 12345 }))
            .await
            .expect_err("type mismatch");
        assert!(matches!(err, IpcError::InvalidParams(_)));
    }

    #[test]
    fn peer_connect_result_serialization_shape() {
        let r = PeerConnectResult {
            connection_id: "cid".into(),
            state: "connecting".into(),
            channel: None,
        };
        let v = serde_json::to_value(&r).expect("json");
        assert_eq!(v["state"], "connecting");
        assert!(v["channel"].is_null());
    }

    #[test]
    fn peer_channel_serialization_roundtrip() {
        let c = PeerChannel {
            local_address: "a".into(),
            remote_address: "b".into(),
            protocol: "udp".into(),
            latency_ms: Some(1),
        };
        let v = serde_json::to_value(&c).expect("ser");
        assert_eq!(v["protocol"], "udp");
    }

    #[tokio::test]
    async fn connect_accepts_optional_null_fields() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let params = json!({
            "target_address": "198.51.100.1:4000",
            "our_binding": null,
            "rendezvous_token": null
        });
        let r = handler.handle_connect(params).await.expect("ok");
        assert_eq!(r.state, "connected");
    }

    #[tokio::test]
    async fn connect_unknown_state_still_returns_ok() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Weird));
        let r = handler.handle_connect(json!({ "target_address": "1.1.1.1:1" })).await.expect("ok");
        assert_eq!(r.state, "negotiating");
    }

    #[tokio::test]
    async fn target_address_accepts_ipv6_bracket_form() {
        let handler = PeerHandler::new(Arc::new(PeerConnector::Mock(MockPeerConnector::new())));
        let r = handler
            .handle_connect(json!({ "target_address": "[2001:db8::1]:5000" }))
            .await
            .expect("ok");
        assert_eq!(r.channel.expect("ch").remote_address, "[2001:db8::1]:5000");
    }
}
