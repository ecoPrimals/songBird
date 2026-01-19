//! Federated Connection (Trust Level 2)
//!
//! For peers approved by a human for full federation.
//!
//! ## Philosophy
//!
//! Human approval grants full federation capabilities.
//!
//! ## Allowed Operations
//!
//! - All Level 1 operations (BirdSong, coordination, health)
//! - `federation/*` - Full federation
//! - `data/read` - Read-only data access
//!
//! ## Denied Operations
//!
//! - `data/write` - No data modification
//! - `commands/sensitive` - No sensitive commands
//! - `keys/*` - No key access

use super::{check_operation_allowed, PeerConnection};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Federated connection for human-approved peers (Level 2)
///
/// Allows full federation and read-only data access.
/// **Pure Rust**: Uses Unix socket RPC for peer communication.
pub struct FederatedConnection {
    peer_id: String,
    socket_path: PathBuf,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
    rpc_client: UnixRpcClient,
}

impl FederatedConnection {
    /// Create a new federated connection (Pure Rust Unix socket)
    pub fn new(
        peer_id: String,
        endpoint: String,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        // Convert endpoint to Unix socket path
        let socket_path = std::env::var(format!("{}_SOCKET_PATH", peer_id.to_uppercase()))
            .or_else(|_| std::env::var("PEER_SOCKET_PATH"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(format!("/tmp/{}.sock", peer_id)));

        let rpc_client = UnixRpcClient::new(&socket_path)
            .context(format!("Failed to create RPC client for peer {}", peer_id))?;

        Ok(Self {
            peer_id,
            socket_path,
            allowed_capabilities,
            denied_capabilities: TrustLevel::Elevated.default_denied_capabilities(),
            rpc_client,
        })
    }

    /// Create with default Level 2 capabilities
    pub fn with_defaults(peer_id: String, endpoint: String) -> Result<Self> {
        Self::new(peer_id, endpoint, TrustLevel::Elevated.default_allowed_capabilities())
    }
}

#[async_trait]
impl PeerConnection for FederatedConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Elevated
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
                "🔒 Operation '{}' denied for peer '{}' at trust level 2 (Elevated)",
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

        debug!("✅ Calling federated operation '{}' on peer '{}'", operation, self.peer_id);

        // Make JSON-RPC call (capability restrictions enforced above)
        let result: Value = self.rpc_client.call(operation, &request).await.context(format!(
            "Failed to call operation '{}' on peer '{}'",
            operation, self.peer_id
        ))?;

        debug!("✅ Federated operation '{}' succeeded on peer '{}'", operation, self.peer_id);
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
        debug!("Closing federated connection to peer '{}'", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_allows_level1_plus_federation() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        // Level 1 operations
        assert!(conn.is_operation_allowed("birdsong/sync"));
        assert!(conn.is_operation_allowed("health"));

        // Level 2 operations
        assert!(conn.is_operation_allowed("federation/join"));
        assert!(conn.is_operation_allowed("data/read"));
    }

    #[test]
    fn test_federated_denies_sensitive() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert!(!conn.is_operation_allowed("data/write"));
        assert!(!conn.is_operation_allowed("commands/sensitive"));
        assert!(!conn.is_operation_allowed("keys/access"));
    }

    #[test]
    fn test_trust_level() {
        let conn = FederatedConnection::with_defaults(
            "test_peer".to_string(),
            "http://localhost:8080".to_string(),
        )
        .unwrap();

        assert_eq!(conn.trust_level(), TrustLevel::Elevated);
    }
}
