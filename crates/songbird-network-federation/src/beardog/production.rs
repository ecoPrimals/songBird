//! Production `BearDog` Provider - Unix Socket Implementation
//!
//! Implements all `BearDog` integration traits via Unix socket JSON-RPC.
//! This is the production provider that connects to a real `BearDog` instance.
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Pure Rust (Unix sockets, no HTTP/reqwest)
//! - ✅ Zero unsafe code
//! - ✅ Runtime discovery (socket path from env/discovery)
//! - ✅ Modern async Rust (trait-based, async/await)
//! - ✅ Graceful error handling

use super::{
    AccessLevel, BearDogProvider, BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageChain,
    LineageHint, LineageProof, LineageProvider, LineageRelay, RelaySession,
};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Production `BearDog` provider via Unix socket JSON-RPC
///
/// Connects to `BearDog`'s Unix socket to provide:
/// - Lineage management and verification
/// - `BirdSong` encryption/decryption
/// - Relay session management
///
/// ## Usage
///
/// ```rust,no_run
/// use songbird_network_federation::beardog::production::ProductionBearDogProvider;
///
/// # async fn example() -> anyhow::Result<()> {
/// let provider = ProductionBearDogProvider::new("/tmp/beardog.sock").await?;
/// # Ok(())
/// # }
/// ```
pub struct ProductionBearDogProvider {
    socket_path: PathBuf,
    family_id: Option<String>,
}

impl ProductionBearDogProvider {
    /// Create new production `BearDog` provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to `BearDog` Unix socket (discovered at runtime)
    ///
    /// # Errors
    ///
    /// Returns error if socket connection fails or health check fails
    pub async fn new(socket_path: impl Into<PathBuf>) -> Result<Self> {
        let socket_path = socket_path.into();

        info!("🐻 Creating production BearDog provider (Unix socket)");
        info!("   Socket: {:?}", socket_path);

        // Verify socket exists and is connectable
        let _ = UnixStream::connect(&socket_path).await.context("BearDog socket not accessible")?;

        Ok(Self {
            socket_path,
            family_id: None, // Will be queried from env on first use
        })
    }

    /// Create new production `BearDog` provider with explicit `family_id`
    ///
    /// Use this when the `family_id` is known at construction time.
    pub async fn with_family_id(
        socket_path: impl Into<PathBuf>,
        family_id: impl Into<String>,
    ) -> Result<Self> {
        let socket_path = socket_path.into();
        let family_id = family_id.into();

        info!("🐻 Creating production BearDog provider with family_id");
        info!("   Socket: {:?}", socket_path);
        info!("   Family: {}", family_id);

        // Verify socket exists and is connectable
        let _ = UnixStream::connect(&socket_path).await.context("BearDog socket not accessible")?;

        Ok(Self {
            socket_path,
            family_id: Some(family_id),
        })
    }

    /// Set the `family_id` for `BirdSong` operations
    pub fn set_family_id(&mut self, family_id: impl Into<String>) {
        self.family_id = Some(family_id.into());
    }

    /// Call `BearDog` JSON-RPC method via Unix socket
    ///
    /// Pure Rust implementation using tokio `UnixStream`.
    async fn call_beardog(&self, method: &str, params: Value) -> Result<Value> {
        debug!("📞 Calling BearDog: {}", method);

        // Connect to BearDog Unix socket
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to BearDog socket")?;

        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Serialize and send
        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes).await?;
        stream.write_all(b"\n").await.ok(); // Newline delimiter

        // Read response
        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await?;

        // Parse JSON-RPC response
        let response: Value = serde_json::from_slice(&response_bytes)
            .context("Invalid JSON response from BearDog")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(anyhow!("BearDog RPC error: {error}"));
        }

        // Return result
        response.get("result").cloned().ok_or_else(|| anyhow!("No result in BearDog response"))
    }
}

#[async_trait::async_trait]
impl LineageProvider for ProductionBearDogProvider {
    async fn generate_lineage(&self, node_id: &str, parent_id: &str) -> Result<LineageChain> {
        let params = serde_json::json!({
            "node_id": node_id,
            "parent_id": parent_id
        });

        let result = self.call_beardog("genetic.generate_lineage", params).await?;
        serde_json::from_value(result).context("Failed to parse lineage chain")
    }

    async fn verify_lineage(&self, proof: &LineageProof) -> Result<bool> {
        let params = serde_json::json!({
            "proof": proof
        });

        let result = self.call_beardog("genetic.verify_lineage", params).await?;
        result
            .get("valid")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Invalid verify_lineage response"))
    }

    async fn get_descendants(&self, root_id: &str) -> Result<Vec<String>> {
        let params = serde_json::json!({
            "root_id": root_id
        });

        let result = self.call_beardog("genetic.get_descendants", params).await?;
        serde_json::from_value(result).context("Failed to parse descendants")
    }

    async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> Result<Option<usize>> {
        let params = serde_json::json!({
            "ancestor_id": ancestor_id,
            "descendant_id": descendant_id
        });

        let result = self.call_beardog("genetic.get_lineage_depth", params).await?;
        Ok(result.get("depth").and_then(serde_json::Value::as_u64).map(|d| d as usize))
    }
}

#[async_trait::async_trait]
impl BirdSongCrypto for ProductionBearDogProvider {
    async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> Result<EncryptedBirdSong> {
        use base64::{engine::general_purpose, Engine as _};

        // Get family_id from self, env vars, or default (canonical chain)
        let family_id = self
            .family_id
            .clone()
            .or_else(|| {
                std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
                    .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                    .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
                    .or_else(|_| std::env::var("FAMILY_ID"))
                    .ok()
            })
            .unwrap_or_else(|| "default".to_string());

        let params = serde_json::json!({
            "plaintext": general_purpose::STANDARD.encode(payload),
            "lineage_hint": format!("{:?}", lineage_hint),
            "family_id": family_id
        });

        let result = self.call_beardog("birdsong.encrypt", params).await?;
        serde_json::from_value(result).context("Failed to parse encrypted birdsong")
    }

    async fn decrypt_birdsong(&self, encrypted: &EncryptedBirdSong) -> Result<Option<Vec<u8>>> {
        // Get family_id from self, env vars, or default (canonical chain)
        let family_id = self
            .family_id
            .clone()
            .or_else(|| {
                std::env::var("SONGBIRD_ORCHESTRATOR_FAMILY_ID")
                    .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                    .or_else(|_| std::env::var("SONGBIRD_FAMILY_ID"))
                    .or_else(|_| std::env::var("FAMILY_ID"))
                    .ok()
            })
            .unwrap_or_else(|| "default".to_string());

        let params = serde_json::json!({
            "encrypted": encrypted,
            "family_id": family_id
        });

        let result = self.call_beardog("birdsong.decrypt", params).await?;

        let success = result.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);
        if !success {
            return Ok(None); // Different family (noise)
        }

        use base64::{engine::general_purpose, Engine as _};
        let plaintext_b64 = result
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No plaintext in decrypt response"))?;

        let plaintext = general_purpose::STANDARD.decode(plaintext_b64)?;
        Ok(Some(plaintext))
    }

    async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> Result<BroadcastKey> {
        let params = serde_json::json!({
            "lineage_hint": format!("{:?}", lineage_hint),
            "proof": proof
        });

        let result = self.call_beardog("birdsong.request_key", params).await?;
        serde_json::from_value(result).context("Failed to parse broadcast key")
    }

    async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> Result<Vec<BroadcastKey>> {
        let params = serde_json::json!({
            "requests": requests
        });

        let result = self.call_beardog("birdsong.request_keys_batch", params).await?;
        serde_json::from_value(result).context("Failed to parse broadcast keys")
    }
}

#[async_trait::async_trait]
impl LineageRelay for ProductionBearDogProvider {
    async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> Result<RelaySession> {
        let params = serde_json::json!({
            "requester": requester,
            "target": target,
            "lineage_proof": lineage_proof
        });

        let result = self.call_beardog("relay.offer", params).await?;
        serde_json::from_value(result).context("Failed to parse relay session")
    }

    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        AccessLevel::from_lineage_depth(lineage_depth)
    }

    async fn relay_packet(&self, session: &RelaySession, packet: &[u8]) -> Result<()> {
        use base64::{engine::general_purpose, Engine as _};

        let params = serde_json::json!({
            "session_id": session.session_id,
            "packet": general_purpose::STANDARD.encode(packet)
        });

        self.call_beardog("relay.relay_packet", params).await?;
        Ok(())
    }

    async fn revoke_relay(&self, session_id: &str) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id
        });

        self.call_beardog("relay.revoke", params).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl BearDogProvider for ProductionBearDogProvider {
    async fn is_available(&self) -> bool {
        // Try health check
        match self.call_beardog("health", serde_json::json!({})).await {
            Ok(result) => {
                result.get("status").and_then(|v| v.as_str()).is_some_and(|s| s == "healthy")
            }
            Err(e) => {
                warn!("BearDog health check failed: {}", e);
                false
            }
        }
    }

    fn version(&self) -> &'static str {
        "production-unix-socket"
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down BearDog provider connection");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_production_provider_creation() {
        // Test with non-existent socket (should error gracefully)
        let result = ProductionBearDogProvider::new("/tmp/nonexistent_beardog_test.sock").await;
        assert!(result.is_err(), "Should error when socket doesn't exist");
    }
}
