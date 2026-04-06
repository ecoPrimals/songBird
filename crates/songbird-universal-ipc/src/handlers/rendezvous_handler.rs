// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Rendezvous Handler for JSON-RPC
//!
//! Handles `rendezvous.*` methods for NAT traversal via relay servers.
//!
//! ## Methods
//! - `rendezvous.register` - Register with a rendezvous server
//! - `rendezvous.lookup` - Find peers via rendezvous server
//!
//! ## Architecture
//! Uses trait-based dependency injection (`RendezvousClient` trait) to enable:
//! - Testing with mock implementations
//! - Production with real HTTP client
//! - Runtime configuration
//!
//! ## Evolution Principles
//! - Zero hardcoding: Configurable servers
//! - Mocks isolated: Only in #[cfg(test)]
//! - Capability-based: Trait-based DI
//! - Modern Rust: async/await, Arc, proper error handling

use crate::error::{IpcError, IpcResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RendezvousRegisterParams {
    /// Rendezvous server URL
    pub server: Arc<str>,
    /// Our node ID
    pub node_id: Arc<str>,
    /// Our family ID (for family-scoped discovery)
    pub family_id: Arc<str>,
    /// Public address (from STUN)
    pub public_address: Arc<str>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousRegisterResult {
    /// Registration ID
    pub registration_id: String,
    /// Expiry time (ISO 8601)
    pub expires_at: String,
    /// Rendezvous token for peers
    pub rendezvous_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RendezvousLookupParams {
    /// Rendezvous server URL
    pub server: Arc<str>,
    /// Target node ID or family ID
    pub target: Arc<str>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousLookupResult {
    /// Found peers
    pub peers: Vec<RendezvousPeer>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RendezvousPeer {
    pub node_id: Arc<str>,
    pub family_id: Arc<str>,
    pub public_address: Arc<str>,
    pub rendezvous_token: Arc<str>,
}

// ============================================================================
// Rendezvous Client Trait (Capability-Based)
// ============================================================================

/// Trait for rendezvous client operations (dependency injection)
#[async_trait]
pub trait RendezvousClient: Send + Sync + 'static {
    /// Register with a rendezvous server
    async fn register(
        &self,
        server: &str,
        node_id: &str,
        family_id: &str,
        public_address: &str,
    ) -> Result<RendezvousRegisterResult, String>;

    /// Lookup peers via rendezvous server
    async fn lookup(&self, server: &str, target: &str) -> Result<Vec<RendezvousPeer>, String>;
}

// ============================================================================
// Rendezvous Handler
// ============================================================================

pub struct RendezvousHandler {
    client: Arc<dyn RendezvousClient>,
}

impl RendezvousHandler {
    /// Create new handler with given client
    pub fn new(client: Arc<dyn RendezvousClient>) -> Self {
        Self {
            client,
        }
    }

    /// Handle rendezvous.register
    pub async fn handle_register(&self, params: Value) -> IpcResult<RendezvousRegisterResult> {
        let params: RendezvousRegisterParams =
            serde_json::from_value(params).map_err(|e| IpcError::InvalidParams(e.to_string()))?;

        info!(
            "🌐 Registering with rendezvous server: {} (node: {}, family: {})",
            params.server.as_ref(),
            params.node_id.as_ref(),
            params.family_id.as_ref()
        );

        let result = self
            .client
            .register(
                params.server.as_ref(),
                params.node_id.as_ref(),
                params.family_id.as_ref(),
                params.public_address.as_ref(),
            )
            .await
            .map_err(|e| IpcError::Internal(format!("Rendezvous registration failed: {e}")))?;

        info!("✅ Registered with rendezvous server (registration_id: {})", result.registration_id);

        Ok(result)
    }

    /// Handle rendezvous.lookup
    pub async fn handle_lookup(&self, params: Value) -> IpcResult<RendezvousLookupResult> {
        let params: RendezvousLookupParams =
            serde_json::from_value(params).map_err(|e| IpcError::InvalidParams(e.to_string()))?;

        info!(
            "🔍 Looking up peer via rendezvous server: {} (target: {})",
            params.server.as_ref(),
            params.target.as_ref()
        );

        let peers = self
            .client
            .lookup(params.server.as_ref(), params.target.as_ref())
            .await
            .map_err(|e| IpcError::Internal(format!("Rendezvous lookup failed: {e}")))?;

        info!("✅ Found {} peers via rendezvous", peers.len());

        Ok(RendezvousLookupResult {
            peers,
        })
    }
}

// ============================================================================
// Mock Implementation (Testing Only - Deep Debt Compliant)
// ============================================================================

#[cfg(test)]
mod tests_support {
    use super::{RendezvousClient, RendezvousPeer, RendezvousRegisterResult};
    use async_trait::async_trait;
    use std::sync::Arc;

    pub struct MockRendezvousClient {
        #[allow(clippy::type_complexity)]
        registered: std::sync::RwLock<Vec<(Arc<str>, Arc<str>, Arc<str>, Arc<str>)>>,
    }

    impl Default for MockRendezvousClient {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockRendezvousClient {
        #[must_use]
        pub fn new() -> Self {
            Self {
                registered: std::sync::RwLock::new(Vec::new()),
            }
        }

        pub fn add_peer(&self, node_id: &str, family_id: &str, public_address: &str, token: &str) {
            let mut registered = self.registered.write().unwrap();
            registered.push((
                Arc::from(node_id),
                Arc::from(family_id),
                Arc::from(public_address),
                Arc::from(token),
            ));
        }
    }

    #[async_trait]
    impl RendezvousClient for MockRendezvousClient {
        async fn register(
            &self,
            _server: &str,
            node_id: &str,
            family_id: &str,
            public_address: &str,
        ) -> Result<RendezvousRegisterResult, String> {
            let registration_id = uuid::Uuid::new_v4().to_string();
            let rendezvous_token = format!("token-{}", &registration_id[..8]);

            self.add_peer(node_id, family_id, public_address, &rendezvous_token);

            let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

            Ok(RendezvousRegisterResult {
                registration_id,
                expires_at: expires_at.to_rfc3339(),
                rendezvous_token,
            })
        }

        async fn lookup(&self, _server: &str, target: &str) -> Result<Vec<RendezvousPeer>, String> {
            let peers: Vec<RendezvousPeer> = {
                let registered = self.registered.read().unwrap();
                registered
                    .iter()
                    .filter(|(node_id, family_id, _, _)| {
                        node_id.as_ref() == target || family_id.as_ref() == target
                    })
                    .map(|(node_id, family_id, public_address, token)| RendezvousPeer {
                        node_id: Arc::clone(node_id),
                        family_id: Arc::clone(family_id),
                        public_address: Arc::clone(public_address),
                        rendezvous_token: Arc::clone(token),
                    })
                    .collect()
            };

            Ok(peers)
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::tests_support::MockRendezvousClient;
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_register_success() {
        let client = Arc::new(MockRendezvousClient::new());
        let handler = RendezvousHandler::new(client);

        let params = json!({
            "server": "https://rendezvous.example.com",
            "node_id": "node-alpha",
            "family_id": "nat0",
            "public_address": "203.0.113.45:54321"
        });

        let result = handler.handle_register(params).await.unwrap();

        assert!(!result.registration_id.is_empty());
        assert!(!result.expires_at.is_empty());
        assert!(!result.rendezvous_token.is_empty());
        assert!(result.rendezvous_token.starts_with("token-"));
    }

    #[tokio::test]
    async fn test_register_missing_params() {
        let client = Arc::new(MockRendezvousClient::new());
        let handler = RendezvousHandler::new(client);

        let params = json!({
            "server": "https://rendezvous.example.com"
            // Missing node_id, family_id, public_address
        });

        let result = handler.handle_register(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_lookup_success() {
        let client = Arc::new(MockRendezvousClient::new());

        // Pre-populate with a peer
        client.add_peer("node-gamma", "nat0", "203.0.113.100:5000", "token-abc123");

        let handler = RendezvousHandler::new(client);

        let params = json!({
            "server": "https://rendezvous.example.com",
            "target": "node-gamma"
        });

        let result = handler.handle_lookup(params).await.unwrap();

        assert_eq!(result.peers.len(), 1);
        assert_eq!(result.peers[0].node_id.as_ref(), "node-gamma");
        assert_eq!(result.peers[0].family_id.as_ref(), "nat0");
        assert_eq!(result.peers[0].public_address.as_ref(), "203.0.113.100:5000");
    }

    #[tokio::test]
    async fn test_lookup_by_family_id() {
        let client = Arc::new(MockRendezvousClient::new());

        // Add multiple peers in same family
        client.add_peer("node-alpha", "nat0", "203.0.113.10:5000", "token-1");
        client.add_peer("node-beta", "nat0", "203.0.113.20:5000", "token-2");
        client.add_peer("node-gamma", "nat1", "203.0.113.30:5000", "token-3");

        let handler = RendezvousHandler::new(client);

        let params = json!({
            "server": "https://rendezvous.example.com",
            "target": "nat0"
        });

        let result = handler.handle_lookup(params).await.unwrap();

        assert_eq!(result.peers.len(), 2); // Should find alpha and beta
        assert!(result.peers.iter().any(|p| p.node_id.as_ref() == "node-alpha"));
        assert!(result.peers.iter().any(|p| p.node_id.as_ref() == "node-beta"));
    }

    #[tokio::test]
    async fn test_lookup_not_found() {
        let client = Arc::new(MockRendezvousClient::new());
        let handler = RendezvousHandler::new(client);

        let params = json!({
            "server": "https://rendezvous.example.com",
            "target": "nonexistent-node"
        });

        let result = handler.handle_lookup(params).await.unwrap();

        assert_eq!(result.peers.len(), 0); // No peers found
    }

    #[tokio::test]
    async fn test_register_then_lookup() {
        let client = Arc::new(MockRendezvousClient::new());
        let handler = RendezvousHandler::new(client);

        // Register a peer
        let register_params = json!({
            "server": "https://rendezvous.example.com",
            "node_id": "node-delta",
            "family_id": "nat0",
            "public_address": "203.0.113.50:6000"
        });

        let register_result = handler.handle_register(register_params).await.unwrap();

        // Lookup the peer
        let lookup_params = json!({
            "server": "https://rendezvous.example.com",
            "target": "node-delta"
        });

        let lookup_result = handler.handle_lookup(lookup_params).await.unwrap();

        assert_eq!(lookup_result.peers.len(), 1);
        assert_eq!(lookup_result.peers[0].node_id.as_ref(), "node-delta");
        assert_eq!(
            lookup_result.peers[0].rendezvous_token.as_ref(),
            register_result.rendezvous_token.as_str()
        );
    }
}
