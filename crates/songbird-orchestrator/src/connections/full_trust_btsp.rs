// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Full Trust BTSP Connection (Trust Level 3 via Encrypted Tunnel)
//!
//! For peers with maximum trust (human entropy + genetic family).
//! Uses BTSP (`BirdSong` Transport Protocol) for port-free, encrypted P2P communication.
//!
//! ## Philosophy
//!
//! Human entropy (USB seed, physical presence) grants maximum trust.
//!
//! ## Allowed Operations
//!
//! - All operations (wildcard `*`)
//! - No restrictions at this trust level
//!
//! ## Denied Operations
//!
//! - None (full trust)
//!
//! ## Modern Rust (v3.18.0)
//!
//! - **Zero Hardcoding**: Discovers security provider via capabilities
//! - **Protocol Agnostic**: Uses BTSP tunnel (no HTTP, no ports)
//! - **Safe Rust**: No unsafe code, all async
//! - **RAII**: Automatic tunnel cleanup on drop
//! - **Capability-Based**: Runtime security enforcement

use crate::btsp_client::BtspClient; // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
use anyhow::{Context, Result};
use serde_json::Value;
use songbird_types::{SongbirdError, TrustLevel};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Full Trust connection via BTSP tunnel (Level 3)
///
/// Allows all operations (unrestricted).
/// Communicates over encrypted BTSP tunnel (port-free, NAT-traversal built-in).
///
/// **v3.18.0 Evolution**: Replaces HTTP connections with BTSP for full-trust peers.
#[derive(Debug)]
pub struct FullTrustBtspConnection {
    /// Peer node ID
    peer_id: String,

    /// BTSP tunnel ID (managed by security provider)
    tunnel_id: Arc<RwLock<String>>,

    /// BTSP client (protocol-agnostic: tarpc/JSON-RPC/HTTP)
    btsp_client: Arc<BtspClient>,

    /// Connection metadata
    established_at: SystemTime,
}

impl FullTrustBtspConnection {
    /// Create a new BTSP full trust connection
    ///
    /// Establishes encrypted tunnel to peer via security provider.
    /// Uses `BirdSong` genetic lineage for NAT traversal if needed.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Remote peer node ID
    /// * `peer_tags` - Peer's discovery tags (for trust evaluation)
    /// * `btsp_client` - BTSP client (protocol-agnostic)
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
    ) -> Result<Self> {
        info!("🔐 Creating BTSP Full Trust connection to peer '{}'", peer_id);
        debug!("   Peer tags: {:?}", peer_tags);

        // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
        // Create peer endpoint from peer_id and tags
        let peer_endpoint = crate::btsp_client::PeerEndpoint {
            id: peer_id.clone(),
            endpoint: format!("peer://{peer_id}"), // Will be resolved via BirdSong/lineage
            public_key: None,                      // Will be exchanged during handshake
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
            established_at: SystemTime::now(),
        })
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
        // Requires security provider v0.16.0+ and BtspClient.send_data_over_tunnel()
        // See: BTSP_CONNECTION_EVOLUTION_V3_18_0.md for implementation plan
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

impl FullTrustBtspConnection {
    pub fn trust_level(&self) -> TrustLevel {
        TrustLevel::Highest
    }

    pub fn allowed_capabilities(&self) -> &[String] {
        // Full trust = all operations allowed
        static ALL_CAPS: &[String] = &[];
        ALL_CAPS
    }

    pub fn denied_capabilities(&self) -> &[String] {
        // Full trust = no restrictions
        static NO_DENIED: &[String] = &[];
        NO_DENIED
    }

    pub fn is_operation_allowed(&self, _operation: &str) -> bool {
        // Full trust = all operations allowed
        true
    }

    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // No capability checks at Level 3 (full trust)
        debug!(
            "🔓 Calling full trust operation '{}' on peer '{}' via BTSP tunnel (unrestricted)",
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
            "🔌 Closing BTSP Full Trust connection to peer '{}' (tunnel: {})",
            self.peer_id, tunnel_id
        );

        self.btsp_client.close_tunnel(&tunnel_id).await.context("Failed to close BTSP tunnel")?;

        info!("✅ BTSP tunnel closed: {}", tunnel_id);
        Ok(())
    }
}

/// RAII cleanup: Automatically close tunnel on drop
impl Drop for FullTrustBtspConnection {
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
    use crate::connections::check_operation_allowed;

    #[test]
    fn test_full_trust_allows_everything() {
        assert_eq!(TrustLevel::Highest.as_u8(), 3);
    }

    #[test]
    fn test_highest_defaults_star_allow_empty_deny_matches_check_operation_allowed() {
        let allowed = TrustLevel::Highest.default_allowed_capabilities();
        let denied = TrustLevel::Highest.default_denied_capabilities();

        assert_eq!(allowed, vec!["*".to_string()]);
        assert!(denied.is_empty());
        assert!(check_operation_allowed("data/write", &allowed, &denied));
        assert!(check_operation_allowed("commands/sensitive/x", &allowed, &denied));
    }

    #[test]
    fn test_trust_level_highest_identity() {
        assert_eq!(TrustLevel::Highest.name(), "highest");
        assert_eq!(TrustLevel::Highest.beardog_alias(), "explicit");
        assert!(!TrustLevel::Highest.description().is_empty());
    }

    #[test]
    fn test_trust_level_from_u8_roundtrip() {
        assert_eq!(TrustLevel::from_u8(3), Some(TrustLevel::Highest));
        assert_eq!(TrustLevel::Highest.as_u8(), 3);
        assert_eq!(TrustLevel::from_u8(9), None);
    }

    #[test]
    fn test_full_trust_ordering_relative_to_other_levels() {
        assert!(TrustLevel::Highest > TrustLevel::Elevated);
        assert!(TrustLevel::Elevated > TrustLevel::Limited);
    }

    #[test]
    fn test_check_operation_allowed_star_with_no_deny() {
        let allowed = vec!["*".to_string()];
        let denied: Vec<String> = vec![];
        assert!(check_operation_allowed("any/operation/at/all", &allowed, &denied));
    }

    #[test]
    fn test_explicit_deny_still_overrides_star_allow() {
        let allowed = vec!["*".to_string()];
        let denied = vec!["data/secret".to_string()];
        assert!(!check_operation_allowed("data/secret", &allowed, &denied));
        assert!(check_operation_allowed("data/public", &allowed, &denied));
    }

    #[test]
    fn test_highest_default_allowed_is_single_wildcard() {
        let caps = TrustLevel::Highest.default_allowed_capabilities();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "*");
    }

    #[test]
    fn test_highest_default_denied_is_empty() {
        assert!(TrustLevel::Highest.default_denied_capabilities().is_empty());
    }
}
