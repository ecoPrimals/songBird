//! Limited Connection (Trust Level 1)
//!
//! For peers with same genetic family but no human approval.
//!
//! ## Philosophy
//!
//! "Same family = can hear the song, NOT enter the nest"
//!
//! ## Allowed Operations
//!
//! - `discovery` - Capability discovery
//! - `coordination/*` - `BirdSong` coordination
//! - `birdsong/*` - `BirdSong` protocol
//! - `health` - Health checks
//! - `capabilities` - Capability queries
//!
//! ## Denied Operations
//!
//! - `data/*` - No data access
//! - `commands/*` - No command execution
//! - `federation/*` - No full federation
//! - `keys/*` - No key access

use super::{check_operation_allowed, PeerConnection};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Limited connection for same-family peers (Level 1)
///
/// Allows `BirdSong` coordination only, no data access or full federation.
/// **Pure Rust**: Uses Unix socket RPC for peer communication.
pub struct LimitedConnection {
    peer_id: String,
    socket_path: PathBuf,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
    rpc_client: UnixRpcClient,
}

impl LimitedConnection {
    /// Create a new limited connection
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Peer node ID
    /// * `endpoint` - Peer endpoint URL
    /// * `allowed_capabilities` - Capabilities allowed at this level
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use songbird_orchestrator::connections::LimitedConnection;
    /// # async fn example() -> anyhow::Result<()> {
    /// let conn = LimitedConnection::new(
    ///     "tower2".to_string(),
    ///     "https://192.168.1.100:8080".to_string(),
    ///     vec!["birdsong/*".to_string(), "health".to_string()],
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        peer_id: String,
        endpoint: String,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        // Convert endpoint to Unix socket path
        let socket_path = std::env::var(format!("{}_SOCKET_PATH", peer_id.to_uppercase()))
            .or_else(|_| std::env::var("PEER_SOCKET_PATH"))
            .map_or_else(|_| PathBuf::from(format!("/tmp/{peer_id}.sock")), PathBuf::from);

        let rpc_client = UnixRpcClient::new(&socket_path)
            .context(format!("Failed to create RPC client for peer {peer_id}"))?;

        Ok(Self {
            peer_id,
            socket_path,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Limited.default_denied_capabilities(),
            rpc_client,
        })
    }

    /// Create with default Level 1 capabilities
    pub fn with_defaults(peer_id: String, endpoint: String) -> Result<Self> {
        Self::new(peer_id, endpoint, TrustLevel::Limited.default_allowed_capabilities())
    }
}

#[async_trait]
impl PeerConnection for LimitedConnection {
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
                "🔒 Operation '{}' denied for peer '{}' at trust level 1 (Limited)",
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
            "🎵 Calling limited operation '{}' on peer '{}' (BirdSong)",
            operation, self.peer_id
        );

        // Make JSON-RPC call (capability restrictions enforced above)
        let result: Value = self.rpc_client.call(operation, &request).await.context(format!(
            "Failed to call operation '{}' on peer '{}'",
            operation, self.peer_id
        ))?;

        debug!("✅ Limited operation '{}' succeeded on peer '{}'", operation, self.peer_id);
        Ok(result)
    }

    fn peer_id(&self) -> &str {
        &self.peer_id
    }

    fn endpoint(&self) -> &str {
        // Return socket path as string for compatibility
        self.socket_path.to_str().unwrap_or(&self.peer_id)
    }

    async fn close(&self) -> Result<()> {
        debug!("Closing limited connection to peer '{}'", self.peer_id);
        // HTTP client cleanup happens automatically
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limited_allows_birdsong() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert!(conn.is_operation_allowed("birdsong/sync"));
        assert!(conn.is_operation_allowed("coordination/state"));
        assert!(conn.is_operation_allowed("health"));
        assert!(conn.is_operation_allowed("capabilities"));
    }

    #[test]
    fn test_limited_denies_data() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert!(!conn.is_operation_allowed("data/read"));
        assert!(!conn.is_operation_allowed("data/write"));
        assert!(!conn.is_operation_allowed("commands/exec"));
        assert!(!conn.is_operation_allowed("federation/join"));
        assert!(!conn.is_operation_allowed("keys/access"));
    }

    #[test]
    fn test_trust_level() {
        let conn = LimitedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert_eq!(conn.trust_level(), TrustLevel::Limited);
    }
}
