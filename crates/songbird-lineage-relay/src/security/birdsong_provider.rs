// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::Result;
use crate::types::{LineageHint, NodeId};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};

use songbird_types::IpcStream;

/// Production `BirdSong` crypto client via the security provider (Unix socket JSON-RPC)
///
/// Connects to the capability-discovered security provider to provide lineage-based
/// encryption for relay broadcasts. Only family members with lineage proofs
/// can decrypt messages.
///
/// ## Deep Debt Principles
///
/// - Runtime discovery (socket path via env or discovery)
/// - Zero unsafe code (pure Rust async)
/// - Enum dispatch into this client (`BirdSongCrypto::Security`)
/// - Graceful error handling
#[derive(Clone, Debug)]
pub struct SecurityBirdSongProvider {
    socket_path: PathBuf,
    family_id: Option<String>,
}

impl SecurityBirdSongProvider {
    /// Create a new `BirdSong` provider for the given security-provider socket
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Security provider Unix socket path (discovered at runtime)
    /// * `family_id` - Optional family ID for validation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_lineage_relay::security::SecurityBirdSongProvider;
    ///
    /// # async fn example() {
    /// let provider = SecurityBirdSongProvider::new(
    ///     "/tmp/security.sock",
    ///     Some(String::from("ecoPrimals-family-123"))
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Self {
        let socket_path = socket_path.into();

        info!("Security-provider BirdSong client created (Unix socket)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Self {
            socket_path,
            family_id,
        }
    }

    /// Test-only accessors (see `security_tests` module; submodules could read fields inline).
    #[cfg(test)]
    pub(crate) fn test_socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Test-only accessor for configured family id.
    #[cfg(test)]
    pub(crate) fn test_family_id(&self) -> Option<&String> {
        self.family_id.as_ref()
    }

    async fn connect_ipc(path: &std::path::Path) -> Result<IpcStream> {
        let path_str = path.to_string_lossy();
        IpcStream::connect(&path_str).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to connect to security provider at {}: {e}",
                path.display(),
            ))
        })
    }

    /// Call security-provider JSON-RPC method via IPC socket
    ///
    /// On Unix: connects to a Unix domain socket.
    /// On Windows: connects via TCP localhost (port read from socket path file).
    async fn call_security_rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut stream = Self::connect_ipc(&self.socket_path).await?;

        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Serialize and send
        let request_bytes = serde_json::to_vec(&request)?;

        stream.write_all(&request_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to write to security provider: {e}"
            ))
        })?;
        stream.write_all(b"\n").await.ok(); // Newline delimiter

        // Read response
        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to read from security provider: {e}"
            ))
        })?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(crate::error::LineageRelayError::BirdSongError(format!(
                "Security provider RPC error: {error}"
            )));
        }

        // Return result
        response.get("result").cloned().ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(String::from(
                "No result in security provider response",
            ))
        })
    }
}

impl SecurityBirdSongProvider {
    /// Encrypt message for lineage via the security provider.
    pub async fn encrypt_for_lineage(&self, message: &[u8], hint: LineageHint) -> Result<Vec<u8>> {
        debug!("🔒 Encrypting for lineage via security provider (hint: {:?})", hint);

        // Encode message as base64 for JSON-RPC
        use base64::{Engine as _, engine::general_purpose};
        let plaintext_b64 = general_purpose::STANDARD.encode(message);

        // Build request params
        let params = serde_json::json!({
            "plaintext": plaintext_b64,
            "family_id": self.family_id,
            "lineage_hint": format!("{:?}", hint) // Serialized for security provider
        });

        // Call birdsong.encrypt
        let result = self.call_security_rpc("birdsong.encrypt", params).await?;

        // Extract ciphertext
        let ciphertext_b64 = result
            .get("ciphertext")
            .or_else(|| result.get("encrypted")) // v1 compatibility
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::LineageRelayError::BirdSongError(String::from(
                    "No ciphertext in security provider encrypt response",
                ))
            })?;

        let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Invalid base64 ciphertext: {e}"
            ))
        })?;

        debug!("✅ Encrypted {} → {} bytes", message.len(), ciphertext.len());
        Ok(ciphertext)
    }

    /// Decrypt `BirdSong` ciphertext via the security provider.
    pub async fn decrypt_birdsong(
        &self,
        encrypted: &[u8],
        sender: &NodeId,
    ) -> Result<Option<Vec<u8>>> {
        debug!("🔓 Decrypting BirdSong from {:?}", sender);

        // Encode ciphertext as base64 for JSON-RPC
        use base64::{Engine as _, engine::general_purpose};
        let ciphertext_b64 = general_purpose::STANDARD.encode(encrypted);

        // Build request params
        let params = serde_json::json!({
            "ciphertext": ciphertext_b64,
            "sender_node_id": sender.0
        });

        // Call birdsong.decrypt
        let Ok(result) = self.call_security_rpc("birdsong.decrypt", params).await else {
            // Decryption failure might just mean different family (noise)
            debug!("🔇 Security provider decrypt failed - likely different family (noise)");
            return Ok(None);
        };

        // Check success flag
        let success = result.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);

        if !success {
            debug!("🔇 Security provider decrypt: different family (noise)");
            return Ok(None);
        }

        // Extract plaintext
        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(String::from(
                "No plaintext in security provider decrypt response",
            ))
        })?;

        let plaintext = general_purpose::STANDARD.decode(plaintext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!("Invalid base64 plaintext: {e}"))
        })?;

        debug!("✅ Decrypted {} bytes from family", plaintext.len());
        Ok(Some(plaintext))
    }
}
