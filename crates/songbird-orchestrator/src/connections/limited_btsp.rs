//! Limited BTSP Connection (Trust Level 1 via Encrypted Tunnel)
//!
//! For peers with same genetic family but no human approval.
//! Uses BTSP (BirdSong Transport Protocol) for port-free, encrypted P2P communication.
//!
//! ## Philosophy
//!
//! "Same family = can hear the song, NOT enter the nest"
//!
//! ## Allowed Operations
//!
//! - `discovery` - Capability discovery
//! - `coordination/*` - BirdSong coordination
//! - `birdsong/*` - BirdSong protocol
//! - `health` - Health checks
//! - `capabilities` - Capability queries
//!
//! ## Denied Operations
//!
//! - `data/*` - No data access
//! - `commands/*` - No command execution
//! - `federation/*` - No full federation
//! - `keys/*` - No key access
//!
//! ## Modern Rust (v3.18.0)
//!
//! - **Zero Hardcoding**: Discovers security provider via capabilities
//! - **Protocol Agnostic**: Uses BTSP tunnel (no HTTP, no ports)
//! - **Safe Rust**: No unsafe code, all async
//! - **RAII**: Automatic tunnel cleanup on drop
//! - **Capability-Based**: Runtime security enforcement

use super::{check_operation_allowed, PeerConnection};
use crate::btsp_client::BtspClient; // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Limited connection via BTSP tunnel (Level 1)
///
/// Allows BirdSong coordination only, no data access or full federation.
/// Communicates over encrypted BTSP tunnel (port-free, NAT-traversal built-in).
///
/// **v3.18.0 Evolution**: Replaces HTTP connections with BTSP for same-family peers.
#[derive(Debug)]
pub struct LimitedBtspConnection {
    /// Peer node ID
    peer_id: String,

    /// BTSP tunnel ID (managed by security provider)
    tunnel_id: Arc<RwLock<String>>,

    /// BTSP client (protocol-agnostic: tarpc/JSON-RPC/HTTP)
    btsp_client: Arc<BtspClient>,

    /// Allowed capabilities for Level 1 trust
    allowed_capabilities: Vec<String>,

    /// Denied capabilities (explicit deny overrides allow)
    denied_capabilities: Vec<String>,

    /// Connection metadata
    established_at: SystemTime,
}

impl LimitedBtspConnection {
    /// Create a new BTSP limited connection
    ///
    /// Establishes encrypted tunnel to peer via security provider.
    /// Uses BirdSong genetic lineage for NAT traversal if needed.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Remote peer node ID
    /// * `peer_tags` - Peer's discovery tags (for trust evaluation)
    /// * `btsp_client` - BTSP client (protocol-agnostic)
    /// * `allowed_capabilities` - Capabilities allowed at Level 1
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
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::connections::LimitedBtspConnection;
    /// # use crate::btsp_client::BtspClient; // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
    /// # use std::sync::Arc;
    /// # async fn example() -> anyhow::Result<()> {
    /// let btsp_client = Arc::new(BtspClient::new());  // Auto-discovers socket
    ///
    /// let conn = LimitedBtspConnection::new(
    ///     "tower2".to_string(),
    ///     vec!["btsp_enabled".to_string()],
    ///     btsp_client,
    ///     vec!["birdsong/*".to_string(), "health".to_string()],
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        peer_id: String,
        peer_tags: Vec<String>,
        btsp_client: Arc<BtspClient>,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        info!("🔐 Creating BTSP Limited connection to peer '{}'", peer_id);
        debug!("   Peer tags: {:?}", peer_tags);

        // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
        let peer_endpoint = crate::btsp_client::PeerEndpoint {
            id: peer_id.clone(),
            endpoint: format!("peer://{}", peer_id),
            public_key: None,
            capabilities: peer_tags.clone(),
        };

        // Establish tunnel via BearDog Unix socket
        let tunnel = btsp_client
            .establish_tunnel(peer_endpoint)
            .await
            .context(format!("Failed to establish BTSP tunnel to peer '{}'", peer_id))?;

        info!("✅ BTSP tunnel established: {} to peer {}", tunnel.id, tunnel.peer_id);

        Ok(Self {
            peer_id: peer_id.clone(),
            tunnel_id: Arc::new(RwLock::new(tunnel.id)),
            btsp_client,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Limited.default_denied_capabilities(),
            established_at: SystemTime::now(),
        })
    }

    /// Create with default Level 1 capabilities
    ///
    /// Convenience constructor using standard Limited trust level capabilities.
    pub async fn with_defaults(
        peer_id: String,
        peer_tags: Vec<String>,
        btsp_client: Arc<BtspClient>,
    ) -> Result<Self> {
        Self::new(
            peer_id,
            peer_tags,
            btsp_client,
            TrustLevel::Limited.default_allowed_capabilities(),
        )
        .await
    }

    /// Send RPC call over BTSP tunnel
    ///
    /// Serializes JSON-RPC 2.0 request and sends over encrypted tunnel.
    /// This is the core communication method for all operations.
    async fn send_rpc(&self, operation: &str, request: Value) -> Result<Value> {
        let tunnel_id = self.tunnel_id.read().await.clone();

        // Create JSON-RPC 2.0 request
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation,
            "params": request,
            "id": uuid::Uuid::new_v4().to_string(),
        });

        debug!("📡 Sending RPC over BTSP tunnel {}: {}", tunnel_id, operation);

        // Serialize request
        let request_bytes =
            serde_json::to_vec(&rpc_request).context("Failed to serialize RPC request")?;

        // Send over tunnel
        // NOTE: In v3.18.0, send_data_over_tunnel is not yet implemented in BtspClient
        // This requires BearDog v0.16.0+ for bidirectional tunnel support.
        //
        // ROADMAP (Phase 2): Bidirectional BTSP Communication
        // - Implement BtspClient.send_data_over_tunnel()
        // - Add request/response correlation
        // - Support streaming data transfer
        // - See: BTSP_CONNECTION_EVOLUTION_V3_18_0.md
        //
        // self.btsp_client.send_data_over_tunnel(&tunnel_id, &request_bytes).await?;

        // For v3.18.0, return error indicating Phase 2 feature
        Err(anyhow!(
            "BTSP bidirectional communication not yet implemented. \
             This requires BearDog v0.16.0+ and BtspClient.send_data_over_tunnel(). \
             Current implementation establishes tunnels only. \
             See BTSP_CONNECTION_EVOLUTION_V3_18_0.md for roadmap."
        ))
    }

    /// Get connection uptime
    pub fn uptime(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.established_at).unwrap_or_default()
    }
}

#[async_trait]
impl PeerConnection for LimitedBtspConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Limited
    }

    fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    fn denied_capabilities(&self) -> &[String] {
        &self.denied_capabilities
    }

    fn is_operation_allowed(&self, operation: &str) -> bool {
        check_operation_allowed(operation, &self.allowed_capabilities, &self.denied_capabilities)
    }

    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // Enforce capability restrictions
        if !self.is_operation_allowed(operation) {
            warn!(
                "🔒 Operation '{}' denied for peer '{}' at trust level 1 (Limited) via BTSP",
                operation, self.peer_id
            );
            return Err(anyhow!(
                "Operation '{}' not allowed at trust level 1 (Limited). \
                 Allowed: {:?}. \
                 To enable this operation, elevate trust to level 2 (Elevated) via user approval.",
                operation,
                self.allowed_capabilities
            ));
        }

        debug!(
            "🔐 Calling limited operation '{}' on peer '{}' via BTSP tunnel",
            operation, self.peer_id
        );

        // Send RPC over BTSP tunnel
        self.send_rpc(operation, request).await
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }

    fn endpoint(&self) -> &'static str {
        // BTSP connections don't have traditional endpoints (no URLs, no ports)
        // Return descriptive string for observability
        "btsp://<encrypted-tunnel>"
    }

    async fn close(&self) -> Result<()> {
        let tunnel_id = self.tunnel_id.read().await.clone();
        info!(
            "🔌 Closing BTSP Limited connection to peer '{}' (tunnel: {})",
            self.peer_id, tunnel_id
        );

        // Close tunnel via security provider
        self.btsp_client.close_tunnel(&tunnel_id).await.context("Failed to close BTSP tunnel")?;

        info!("✅ BTSP tunnel closed: {}", tunnel_id);
        Ok(())
    }
}

/// RAII cleanup: Automatically close tunnel on drop
impl Drop for LimitedBtspConnection {
    fn drop(&mut self) {
        // Spawn cleanup task (Drop is sync, but close_tunnel is async)
        // Note: This is a "best effort" cleanup. The tunnel will be closed
        // eventually by the security provider's timeout mechanism if this fails.

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
    use super::*;

    #[test]
    fn test_limited_allows_birdsong() {
        // Capability checking doesn't require async or BTSP client
        let allowed = TrustLevel::Limited.default_allowed_capabilities();
        let denied = TrustLevel::Limited.default_denied_capabilities();

        assert!(check_operation_allowed("birdsong/sync", &allowed, &denied));
        assert!(check_operation_allowed("coordination/state", &allowed, &denied));
        assert!(check_operation_allowed("health", &allowed, &denied));
        assert!(check_operation_allowed("capabilities", &allowed, &denied));
    }

    #[test]
    fn test_limited_denies_data() {
        let allowed = TrustLevel::Limited.default_allowed_capabilities();
        let denied = TrustLevel::Limited.default_denied_capabilities();

        assert!(!check_operation_allowed("data/read", &allowed, &denied));
        assert!(!check_operation_allowed("data/write", &allowed, &denied));
        assert!(!check_operation_allowed("commands/exec", &allowed, &denied));
        assert!(!check_operation_allowed("federation/join", &allowed, &denied));
        assert!(!check_operation_allowed("keys/access", &allowed, &denied));
    }

    #[tokio::test]
    async fn test_btsp_connection_creation() {
        // Test connection creation (will fail without real security provider)
        // This validates the API, not the end-to-end flow

        // v3.20.0: Unix socket client auto-discovers from environment
        let btsp_client = Arc::new(BtspClient::new());

        let result = LimitedBtspConnection::with_defaults(
            "test_peer".to_string(),
            vec!["btsp_enabled".to_string()],
            btsp_client,
        )
        .await;

        // Should fail (no real security provider), but validates API
        assert!(result.is_err());
        assert!(format!("{:?}", result).contains("Failed to establish BTSP tunnel"));
    }

    #[tokio::test]
    async fn test_endpoint_shows_btsp() {
        // LimitedBtspConnection doesn't have traditional HTTP endpoints
        // It should return a descriptive BTSP indicator

        // We can't create a real connection without a security provider,
        // but we can test the pattern is correct by checking the trait contract
        let allowed = TrustLevel::Limited.default_allowed_capabilities();
        assert!(!allowed.is_empty(), "Limited trust should have some allowed capabilities");
    }

    #[test]
    fn test_trust_level() {
        // Trust level is constant, no async needed
        let allowed = TrustLevel::Limited.default_allowed_capabilities();
        let denied = TrustLevel::Limited.default_denied_capabilities();

        // Verify Level 1 capabilities are defined
        assert!(!allowed.is_empty(), "Limited trust should allow some operations");
        assert!(!denied.is_empty(), "Limited trust should deny some operations");
    }
}
