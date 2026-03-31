// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Peer Connection Handler for JSON-RPC
//!
//! Handles `peer.connect` method for direct peer-to-peer connections using UDP hole punching.
//!
//! ## Method
//! - `peer.connect` - Initiate direct connection to peer (hole punching)
//!
//! ## Architecture
//! Uses trait-based dependency injection (`PeerConnector` trait) to enable:
//! - Testing with mock implementations
//! - Production with real UDP hole punching
//! - Integration with STUN bindings
//!
//! ## Evolution Principles
//! - Zero hardcoding: Configurable timeouts and retry logic
//! - Mocks isolated: Only in #[cfg(test)]
//! - Capability-based: Trait-based DI
//! - Modern Rust: async/await, Arc, proper error handling

use crate::error::{IpcError, IpcResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerConnectParams {
    /// Target peer address (IP:port)
    pub target_address: String,
    /// Our STUN binding (for symmetric NAT, optional)
    pub our_binding: Option<String>,
    /// Rendezvous token (if using rendezvous, optional)
    pub rendezvous_token: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PeerConnectResult {
    /// Connection ID
    pub connection_id: String,
    /// Connection state
    pub state: String, // "connecting", "connected", "failed"
    /// Established channel info (if connected)
    pub channel: Option<PeerChannel>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PeerChannel {
    /// Local endpoint
    pub local_address: String,
    /// Remote endpoint
    pub remote_address: String,
    /// Protocol (udp/tcp)
    pub protocol: String,
    /// Latency (ms, if measured)
    pub latency_ms: Option<u64>,
}

// ============================================================================
// Peer Connector Trait (Capability-Based)
// ============================================================================

/// Trait for peer connection operations (dependency injection)
#[async_trait]
pub trait PeerConnector: Send + Sync + 'static {
    /// Initiate connection to peer (UDP hole punching)
    async fn connect(
        &self,
        target_address: &str,
        our_binding: Option<&str>,
        rendezvous_token: Option<&str>,
    ) -> Result<PeerConnectResult, String>;
}

// ============================================================================
// Peer Handler
// ============================================================================

pub struct PeerHandler {
    connector: Arc<dyn PeerConnector>,
}

impl PeerHandler {
    /// Create new handler with given connector
    pub fn new(connector: Arc<dyn PeerConnector>) -> Self {
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
// Mock Implementation (Testing Only - Deep Debt Compliant)
// ============================================================================

#[cfg(test)]
mod tests_support {
    use super::{PeerChannel, PeerConnectResult, PeerConnector};
    use async_trait::async_trait;

    pub struct MockPeerConnector {
        should_succeed: std::sync::RwLock<bool>,
    }

    impl Default for MockPeerConnector {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockPeerConnector {
        #[must_use]
        pub fn new() -> Self {
            Self {
                should_succeed: std::sync::RwLock::new(true),
            }
        }

        pub fn set_should_succeed(&self, succeed: bool) {
            *self.should_succeed.write().unwrap() = succeed;
        }
    }

    #[async_trait]
    impl PeerConnector for MockPeerConnector {
        async fn connect(
            &self,
            target_address: &str,
            our_binding: Option<&str>,
            _rendezvous_token: Option<&str>,
        ) -> Result<PeerConnectResult, String> {
            let should_succeed = *self.should_succeed.read().unwrap();

            let connection_id = uuid::Uuid::new_v4().to_string();

            if should_succeed {
                let local_address = our_binding
                    .map_or_else(|| "0.0.0.0:0".to_string(), std::string::ToString::to_string);

                Ok(PeerConnectResult {
                    connection_id,
                    state: "connected".to_string(),
                    channel: Some(PeerChannel {
                        local_address,
                        remote_address: target_address.to_string(),
                        protocol: "udp".to_string(),
                        latency_ms: Some(25),
                    }),
                })
            } else {
                Ok(PeerConnectResult {
                    connection_id,
                    state: "failed".to_string(),
                    channel: None,
                })
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::tests_support::MockPeerConnector;
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_connect_success() {
        let connector = Arc::new(MockPeerConnector::new());
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
        let connector = Arc::new(MockPeerConnector::new());
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
        let connector = Arc::new(MockPeerConnector::new());
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
        let connector = Arc::new(MockPeerConnector::new());
        connector.set_should_succeed(false);

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
        let connector = Arc::new(MockPeerConnector::new());
        let handler = PeerHandler::new(connector);

        let params = json!({
            // Missing target_address
        });

        let result = handler.handle_connect(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_all_params() {
        let connector = Arc::new(MockPeerConnector::new());
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

    struct ErrorPeerConnector;

    #[async_trait]
    impl PeerConnector for ErrorPeerConnector {
        async fn connect(
            &self,
            _target_address: &str,
            _our_binding: Option<&str>,
            _rendezvous_token: Option<&str>,
        ) -> Result<PeerConnectResult, String> {
            Err("simulated transport failure".to_string())
        }
    }

    #[tokio::test]
    async fn connect_propagates_connector_error() {
        let handler = PeerHandler::new(Arc::new(ErrorPeerConnector));
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
        let handler = PeerHandler::new(Arc::new(MockPeerConnector::new()));
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
        let handler = PeerHandler::new(Arc::new(MockPeerConnector::new()));
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
        struct WeirdConnector;

        #[async_trait]
        impl PeerConnector for WeirdConnector {
            async fn connect(
                &self,
                _target: &str,
                _b: Option<&str>,
                _t: Option<&str>,
            ) -> Result<PeerConnectResult, String> {
                Ok(PeerConnectResult {
                    connection_id: "x".into(),
                    state: "negotiating".into(),
                    channel: None,
                })
            }
        }

        let handler = PeerHandler::new(Arc::new(WeirdConnector));
        let r = handler.handle_connect(json!({ "target_address": "1.1.1.1:1" })).await.expect("ok");
        assert_eq!(r.state, "negotiating");
    }

    #[tokio::test]
    async fn target_address_accepts_ipv6_bracket_form() {
        let handler = PeerHandler::new(Arc::new(MockPeerConnector::new()));
        let r = handler
            .handle_connect(json!({ "target_address": "[2001:db8::1]:5000" }))
            .await
            .expect("ok");
        assert_eq!(r.channel.expect("ch").remote_address, "[2001:db8::1]:5000");
    }
}
