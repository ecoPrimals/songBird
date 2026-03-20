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

#[derive(Debug, Deserialize)]
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
pub struct MockPeerConnector {
    // Simulate success/failure behavior
    should_succeed: std::sync::RwLock<bool>,
}

#[cfg(test)]
impl Default for MockPeerConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
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

#[cfg(test)]
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
            // Simulate successful connection
            let local_address = our_binding
                .map_or_else(|| "0.0.0.0:0".to_string(), std::string::ToString::to_string);

            Ok(PeerConnectResult {
                connection_id,
                state: "connected".to_string(),
                channel: Some(PeerChannel {
                    local_address,
                    remote_address: target_address.to_string(),
                    protocol: "udp".to_string(),
                    latency_ms: Some(25), // Simulated latency
                }),
            })
        } else {
            // Simulate failed connection
            Ok(PeerConnectResult {
                connection_id,
                state: "failed".to_string(),
                channel: None,
            })
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
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
}
