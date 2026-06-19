// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Full Trust Connection (Trust Level 3)
//!
//! For peers with human entropy added (`SoloKey`, Phone HSM).
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

use anyhow::{Context, Result};
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
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new(peer_id: String, _endpoint: String) -> Result<Self> {
        // Convert endpoint to Unix socket path
        let socket_path =
            songbird_process_env::var(format!("{}_SOCKET_PATH", peer_id.to_uppercase()))
                .or_else(|_| songbird_process_env::var("PEER_SOCKET_PATH"))
                .map_or_else(
                    |_| crate::env_config::peer_fallback_socket_path(&peer_id),
                    PathBuf::from,
                );

        let rpc_client = UnixRpcClient::new(&socket_path)
            .context(format!("Failed to create RPC client for peer {peer_id}"))?;

        Ok(Self {
            peer_id,
            socket_path,
            allowed_capabilities: vec![String::from("*")],
            rpc_client,
        })
    }
}

impl FullTrustConnection {
    pub fn trust_level(&self) -> TrustLevel {
        TrustLevel::Highest
    }

    pub fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    pub fn denied_capabilities(&self) -> &[String] {
        &[] // Nothing denied at highest trust
    }

    pub fn is_operation_allowed(&self, _operation: &str) -> bool {
        true // Everything allowed
    }

    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
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

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn endpoint(&self) -> &str {
        // Return socket path as string for compatibility
        self.socket_path.to_str().unwrap_or(&self.peer_id)
    }

    pub async fn close(&self) -> Result<()> {
        debug!("Closing full-trust connection to peer '{}'", self.peer_id);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_full_trust_allows_everything() {
        let conn = FullTrustConnection::new(
            String::from("test_peer"),
            String::from("http://localhost:8080"),
        )
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
        let conn = FullTrustConnection::new(
            String::from("test_peer"),
            String::from("http://localhost:8080"),
        )
        .unwrap();

        assert_eq!(conn.trust_level(), TrustLevel::Highest);
    }
}
