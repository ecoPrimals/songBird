//! Full Trust Connection (Trust Level 3)
//!
//! For peers with human entropy added (SoloKey, Phone HSM).
//!
//! ## Philosophy
//!
//! Human entropy enables all operations including sensitive ones.
//!
//! ## Allowed Operations
//!
//! - `*` - Everything (no restrictions)
//!
//! ## Denied Operations
//!
//! - None

use super::PeerConnection;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use songbird_types::TrustLevel;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tracing::debug;

/// Full trust connection for peers with human entropy (Level 3)
///
/// Allows all operations with no restrictions.
/// **Pure Rust**: Uses Unix socket RPC for peer communication.
pub struct FullTrustConnection {
    peer_id: String,
    socket_path: PathBuf,
    allowed_capabilities: Vec<String>,
    rpc_client: UnixRpcClient,
}

impl FullTrustConnection {
    /// Create a new full trust connection (Pure Rust Unix socket)
    pub fn new(peer_id: String, endpoint: String) -> Result<Self> {
        // Convert endpoint to Unix socket path
        let socket_path = std::env::var(format!("{}_SOCKET_PATH", peer_id.to_uppercase()))
            .or_else(|_| std::env::var("PEER_SOCKET_PATH"))
            .map_or_else(|_| PathBuf::from(format!("/tmp/{}.sock", peer_id)), PathBuf::from);

        let rpc_client = UnixRpcClient::new(&socket_path)
            .context(format!("Failed to create RPC client for peer {}", peer_id))?;

        Ok(Self {
            peer_id,
            socket_path,
            allowed_capabilities: vec!["*".to_string()],
            rpc_client,
        })
    }
}

#[async_trait]
impl PeerConnection for FullTrustConnection {
    fn trust_level(&self) -> TrustLevel {
        TrustLevel::Highest
    }

    fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    fn denied_capabilities(&self) -> &[String] {
        &[] // Nothing denied at highest trust
    }

    fn is_operation_allowed(&self, _operation: &str) -> bool {
        true // Everything allowed
    }

    async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        debug!(
            "🔓 Calling full-trust operation '{}' on peer '{}' via RPC (Level 3)",
            operation, self.peer_id
        );

        // Make JSON-RPC call (no capability restrictions, full trust!)
        let result: Value = self.rpc_client.call(operation, &request).await.context(format!(
            "Failed to call operation '{}' on peer '{}'",
            operation, self.peer_id
        ))?;

        debug!("✅ Full-trust operation '{}' succeeded on peer '{}'", operation, self.peer_id);
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
        debug!("Closing full-trust connection to peer '{}'", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_trust_allows_everything() {
        let conn =
            FullTrustConnection::new("test_peer".to_string(), "http://localhost:8080".to_string())
                .unwrap();

        // Everything allowed
        assert!(conn.is_operation_allowed("data/read"));
        assert!(conn.is_operation_allowed("data/write"));
        assert!(conn.is_operation_allowed("commands/sensitive"));
        assert!(conn.is_operation_allowed("keys/access"));
        assert!(conn.is_operation_allowed("anything/at/all"));
    }

    #[test]
    fn test_trust_level() {
        let conn =
            FullTrustConnection::new("test_peer".to_string(), "http://localhost:8080".to_string())
                .unwrap();

        assert_eq!(conn.trust_level(), TrustLevel::Highest);
    }
}
