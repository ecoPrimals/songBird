// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federated BTSP Connection (Trust Level 2 via Encrypted Tunnel)
//!
//! For peers approved by a human for full federation.
//! Uses BTSP (`BirdSong` Transport Protocol) for port-free, encrypted P2P communication.
//!
//! ## Philosophy
//!
//! Human approval grants full federation capabilities.
//!
//! ## Allowed Operations
//!
//! - All Level 1 operations (`BirdSong`, coordination, health)
//! - `federation/*` - Full federation
//! - `data/read` - Read-only data access
//!
//! ## Denied Operations
//!
//! - `data/write` - No data modification
//! - `commands/sensitive` - No sensitive commands
//! - `keys/*` - No key access
//!
//! ## Modern Rust (v3.18.0)
//!
//! - **Zero Hardcoding**: Discovers security provider via capabilities
//! - **Protocol Agnostic**: Uses BTSP tunnel (no HTTP, no ports)
//! - **Safe Rust**: No unsafe code, all async
//! - **RAII**: Automatic tunnel cleanup on drop
//! - **Capability-Based**: Runtime security enforcement

use super::check_operation_allowed;
use crate::btsp_client::BtspClient; // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use songbird_types::{SongbirdError, TrustLevel};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Federated connection via BTSP tunnel (Level 2)
///
/// Allows full federation and read-only data access.
/// Communicates over encrypted BTSP tunnel (port-free, NAT-traversal built-in).
///
/// **v3.18.0 Evolution**: Replaces HTTP connections with BTSP for federated peers.
#[derive(Debug)]
pub struct FederatedBtspConnection {
    /// Peer node ID
    peer_id: String,

    /// BTSP tunnel ID (managed by security provider)
    tunnel_id: Arc<RwLock<String>>,

    /// BTSP client (protocol-agnostic: tarpc/JSON-RPC/HTTP)
    btsp_client: Arc<BtspClient>,

    /// Allowed capabilities for Level 2 trust
    allowed_capabilities: Vec<String>,

    /// Denied capabilities (explicit deny overrides allow)
    denied_capabilities: Vec<String>,

    /// Connection metadata
    established_at: SystemTime,
}

impl FederatedBtspConnection {
    /// Create a new BTSP federated connection
    ///
    /// Establishes encrypted tunnel to peer via security provider.
    /// Uses `BirdSong` genetic lineage for NAT traversal if needed.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Remote peer node ID
    /// * `peer_tags` - Peer's discovery tags (for trust evaluation)
    /// * `btsp_client` - BTSP client (protocol-agnostic)
    /// * `allowed_capabilities` - Capabilities allowed at Level 2
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Connection on success
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Tunnel establishment fails
    /// - Security provider unavailable
    /// - Remote peer unreachable (even with lineage NAT traversal)
    pub async fn new(
        peer_id: String,
        peer_tags: Vec<String>,
        btsp_client: Arc<BtspClient>,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        info!("🔐 Creating BTSP Federated connection to peer '{}'", peer_id);
        debug!("   Peer tags: {:?}", peer_tags);

        // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
        let peer_endpoint = crate::btsp_client::PeerEndpoint {
            id: peer_id.clone(),
            endpoint: format!("peer://{peer_id}"),
            public_key: None,
            capabilities: peer_tags.clone(),
        };

        // Establish tunnel via security provider Unix socket
        let tunnel = btsp_client
            .establish_tunnel(peer_endpoint)
            .await
            .context(format!("Failed to establish BTSP tunnel to peer '{peer_id}'"))?;

        info!("✅ BTSP tunnel established: {} to peer {}", tunnel.id, tunnel.peer_id);

        Ok(Self {
            peer_id: peer_id.clone(),
            tunnel_id: Arc::new(RwLock::new(tunnel.id)),
            btsp_client,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Elevated.default_denied_capabilities(),
            established_at: SystemTime::now(),
        })
    }

    /// Create with default Level 2 capabilities
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn with_defaults(
        peer_id: String,
        peer_tags: Vec<String>,
        btsp_client: Arc<BtspClient>,
    ) -> Result<Self> {
        Self::new(
            peer_id,
            peer_tags,
            btsp_client,
            TrustLevel::Elevated.default_allowed_capabilities(),
        )
        .await
    }

    /// Send RPC call over BTSP tunnel
    async fn send_rpc(&self, operation: &str, request: Value) -> Result<Value> {
        let tunnel_id = self.tunnel_id.read().await.clone();

        // Create JSON-RPC 2.0 request (serialized path reserved for Phase 2 tunnel I/O)
        let _rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation,
            "params": request,
            "id": uuid::Uuid::new_v4().to_string(),
        });

        debug!("📡 Sending RPC over BTSP tunnel {}: {}", tunnel_id, operation);

        // ROADMAP (Phase 2): Bidirectional BTSP Communication
        // See: BTSP_CONNECTION_EVOLUTION_V3_18_0.md
        Err(SongbirdError::not_implemented_with_detail(
            "btsp_bidirectional_rpc",
            "Requires security provider v0.16.0+ and BtspClient.send_data_over_tunnel(); \
             current implementation establishes tunnels only.",
        )
        .into())
    }

    /// Get connection uptime
    #[must_use]
    pub fn uptime(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.established_at).unwrap_or_default()
    }
}

impl FederatedBtspConnection {
    pub fn trust_level(&self) -> TrustLevel {
        TrustLevel::Elevated
    }

    pub fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    pub fn denied_capabilities(&self) -> &[String] {
        &self.denied_capabilities
    }

    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        check_operation_allowed(operation, &self.allowed_capabilities, &self.denied_capabilities)
    }

    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // Enforce capability restrictions
        if !self.is_operation_allowed(operation) {
            warn!(
                "🔒 Operation '{}' denied for peer '{}' at trust level 2 (Elevated) via BTSP",
                operation, self.peer_id
            );
            return Err(anyhow!(
                "Operation '{}' not allowed at trust level 2 (Elevated). \
                 Allowed: {:?}. \
                 To enable sensitive operations, elevate trust to level 3 (Highest) via human entropy.",
                operation,
                self.allowed_capabilities
            ));
        }

        debug!(
            "🔐 Calling federated operation '{}' on peer '{}' via BTSP tunnel",
            operation, self.peer_id
        );

        // Send RPC over BTSP tunnel
        self.send_rpc(operation, request).await
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn endpoint(&self) -> &'static str {
        "btsp://<encrypted-tunnel>"
    }

    pub async fn close(&self) -> Result<()> {
        let tunnel_id = self.tunnel_id.read().await.clone();
        info!(
            "🔌 Closing BTSP Federated connection to peer '{}' (tunnel: {})",
            self.peer_id, tunnel_id
        );

        self.btsp_client.close_tunnel(&tunnel_id).await.context("Failed to close BTSP tunnel")?;

        info!("✅ BTSP tunnel closed: {}", tunnel_id);
        Ok(())
    }
}

/// RAII cleanup: Automatically close tunnel on drop
impl Drop for FederatedBtspConnection {
    fn drop(&mut self) {
        let tunnel_id = self.tunnel_id.clone();
        let btsp_client = Arc::clone(&self.btsp_client);
        let peer_id = self.peer_id.clone();

        tokio::spawn(async move {
            let id = tunnel_id.read().await.clone();
            if let Err(e) = btsp_client.close_tunnel(&id).await {
                warn!("⚠️ Failed to close BTSP tunnel for peer '{}' during drop: {}", peer_id, e);
            } else {
                debug!("✅ BTSP tunnel cleanup complete for peer '{}'", peer_id);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_federated_allows_level1_plus_federation() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        let denied = TrustLevel::Elevated.default_denied_capabilities();

        assert!(check_operation_allowed("birdsong/sync", &allowed, &denied));
        assert!(check_operation_allowed("health", &allowed, &denied));
        assert!(check_operation_allowed("federation/join", &allowed, &denied));
        assert!(check_operation_allowed("data/read", &allowed, &denied));
    }

    #[test]
    fn test_federated_denies_sensitive() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        let denied = TrustLevel::Elevated.default_denied_capabilities();

        assert!(!check_operation_allowed("data/write", &allowed, &denied));
        assert!(!check_operation_allowed("commands/sensitive", &allowed, &denied));
        assert!(!check_operation_allowed("keys/access", &allowed, &denied));
    }

    #[test]
    fn test_elevated_trust_level_identity() {
        assert_eq!(TrustLevel::Elevated.as_u8(), 2);
        assert_eq!(TrustLevel::Elevated.name(), "elevated");
        assert!(!TrustLevel::Elevated.description().is_empty());
    }

    #[test]
    fn test_keys_wildcard_denies_nested_key_ops() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        let denied = TrustLevel::Elevated.default_denied_capabilities();

        assert!(!check_operation_allowed("keys/export", &allowed, &denied));
        assert!(!check_operation_allowed("keys/", &allowed, &denied));
    }

    #[test]
    fn test_federation_wildcard_allows_subpaths() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        let denied = TrustLevel::Elevated.default_denied_capabilities();

        assert!(check_operation_allowed("federation/status/sync", &allowed, &denied));
    }

    #[test]
    fn test_data_read_allowed_but_not_write() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        let denied = TrustLevel::Elevated.default_denied_capabilities();

        assert!(check_operation_allowed("data/read", &allowed, &denied));
        assert!(!check_operation_allowed("data/write", &allowed, &denied));
    }

    #[test]
    fn test_explicit_deny_overrides_federation_style_allow() {
        let allowed = vec!["federation/*".to_string(), "data/write".to_string()];
        let denied = vec!["federation/block".to_string()];

        assert!(check_operation_allowed("federation/ok", &allowed, &denied));
        assert!(!check_operation_allowed("federation/block", &allowed, &denied));
    }

    #[test]
    fn test_elevated_default_lists_include_federation_and_discovery() {
        let allowed = TrustLevel::Elevated.default_allowed_capabilities();
        assert!(allowed.iter().any(|c| c == "federation/*"));
        assert!(allowed.iter().any(|c| c == "discovery"));
    }

    #[test]
    fn test_commands_sensitive_denied_when_commands_wildcard_allowed() {
        let allowed = vec!["commands/*".to_string()];
        let denied = vec!["commands/sensitive".to_string()];

        assert!(check_operation_allowed("commands/ok", &allowed, &denied));
        assert!(!check_operation_allowed("commands/sensitive", &allowed, &denied));
    }
}
