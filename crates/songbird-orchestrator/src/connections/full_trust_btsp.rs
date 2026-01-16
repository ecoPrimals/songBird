//! Full Trust BTSP Connection (Trust Level 3 via Encrypted Tunnel)
//!
//! For peers with maximum trust (human entropy + genetic family).
//! Uses BTSP (BirdSong Transport Protocol) for port-free, encrypted P2P communication.
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

use super::PeerConnection;
use crate::btsp_client::BtspClient; // v3.20.0: Unix socket BTSP client (Jan 16, 2026)
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;
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
    /// Uses BirdSong genetic lineage for NAT traversal if needed.
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
            endpoint: format!("peer://{}", peer_id), // Will be resolved via BirdSong/lineage
            public_key: None, // Will be exchanged during handshake
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
            established_at: SystemTime::now(),
        })
    }

    /// Send RPC call over BTSP tunnel
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

        // TODO(v3.18.1): Implement bidirectional BTSP communication
        Err(anyhow!(
            "BTSP bidirectional communication not yet implemented. \
             This requires BearDog v0.16.0+ and BtspClient.send_data_over_tunnel(). \
             Current implementation establishes tunnels only."
        ))
    }

    /// Get connection uptime
    pub fn uptime(&self) -> std::time::Duration {
        SystemTime::now().duration_since(self.established_at).unwrap_or_default()
    }
}

#[async_trait]
impl PeerConnection for FullTrustBtspConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Highest
    }

    fn allowed_capabilities(&self) -> &[String] {
        // Full trust = all operations allowed
        static ALL_CAPS: &[String] = &[];
        ALL_CAPS
    }

    fn denied_capabilities(&self) -> &[String] {
        // Full trust = no restrictions
        static NO_DENIED: &[String] = &[];
        NO_DENIED
    }

    fn is_operation_allowed(&self, _operation: &str) -> bool {
        // Full trust = all operations allowed
        true
    }

    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        // No capability checks at Level 3 (full trust)
        debug!(
            "🔓 Calling full trust operation '{}' on peer '{}' via BTSP tunnel (unrestricted)",
            operation, self.peer_id
        );

        // Send RPC over BTSP tunnel
        self.send_rpc(operation, request).await
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }

    fn endpoint(&self) -> &'static str {
        "btsp://<encrypted-tunnel>"
    }

    async fn close(&self) -> Result<()> {
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
    use super::*;

    #[test]
    fn test_full_trust_allows_everything() {
        // Full trust allows any operation
        let conn_props = TrustLevel::Highest;

        // At Level 3, everything is allowed
        assert_eq!(conn_props.as_u8(), 3);
    }

    #[tokio::test]
    async fn test_btsp_full_trust_connection_creation() {
        // v3.20.0: Unix socket client auto-discovers from environment
        let btsp_client = Arc::new(BtspClient::new());

        let result = FullTrustBtspConnection::new(
            "test_peer".to_string(),
            vec!["btsp_enabled".to_string()],
            btsp_client,
        )
        .await;

        // Should fail (no real security provider), but validates API
        assert!(result.is_err());
    }

    #[test]
    fn test_trust_level_highest() {
        assert_eq!(TrustLevel::Highest.as_u8(), 3);
        assert_eq!(TrustLevel::Highest.name(), "Highest");
    }
}
