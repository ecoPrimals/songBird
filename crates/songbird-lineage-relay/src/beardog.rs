//! `BearDog` `BirdSong` Provider - Production & Test Implementations
//!
//! Production implementation connects to `BearDog` via Unix socket JSON-RPC.
//! Test mocks allow testing lineage relay without `BearDog` running.
//!
//! ## Deep Debt Compliance
//!
//! - ✅ Modern async Rust (trait-based, async/await)
//! - ✅ Zero unsafe code
//! - ✅ Runtime discovery (no hardcoded paths)
//! - ✅ Mocks isolated to #[cfg(test)]
//! - ✅ Pure Rust (Unix sockets, not HTTP)

use crate::birdsong::{BirdSongCrypto, LineageHint};
use crate::error::Result;
use crate::types::NodeId;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info};

// Only import RwLock in test context
#[cfg(test)]
use tokio::sync::RwLock;

/// Production `BearDog` `BirdSong` Provider
///
/// Connects to `BearDog` via Unix socket JSON-RPC to provide lineage-based
/// encryption for relay broadcasts. Only family members with lineage proofs
/// can decrypt messages.
///
/// ## Deep Debt Principles
///
/// - Runtime discovery (socket path via env or discovery)
/// - Zero unsafe code (pure Rust async)
/// - Trait-based (implements `BirdSongCrypto`)
/// - Graceful error handling
pub struct BearDogBirdSongProvider {
    socket_path: PathBuf,
    family_id: Option<String>,
}

impl BearDogBirdSongProvider {
    /// Create new `BearDog` `BirdSong` provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - `BearDog` Unix socket path (discovered at runtime)
    /// * `family_id` - Optional family ID for validation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_lineage_relay::beardog::BearDogBirdSongProvider;
    ///
    /// # async fn example() {
    /// let provider = BearDogBirdSongProvider::new(
    ///     "/tmp/beardog.sock",
    ///     Some("ecoPrimals-family-123".to_string())
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Self {
        let socket_path = socket_path.into();

        info!("🐻 BearDog BirdSong provider created (Unix socket)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Self {
            socket_path,
            family_id,
        }
    }

    /// Call `BearDog` JSON-RPC method via Unix socket
    ///
    /// Pure Rust implementation using tokio `UnixStream`.
    async fn call_beardog(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Connect to BearDog Unix socket
        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to connect to BearDog at {:?}: {}",
                self.socket_path, e
            ))
        })?;

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
                "Failed to write to BearDog: {e}"
            ))
        })?;
        stream.write_all(b"\n").await.ok(); // Newline delimiter

        // Read response
        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Failed to read from BearDog: {e}"
            ))
        })?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_slice(&response_bytes)?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(crate::error::LineageRelayError::BirdSongError(format!(
                "BearDog RPC error: {error}"
            )));
        }

        // Return result
        response.get("result").cloned().ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(
                "No result in BearDog response".to_string(),
            )
        })
    }
}

#[async_trait]
impl BirdSongCrypto for BearDogBirdSongProvider {
    async fn encrypt_for_lineage(&self, message: &[u8], hint: LineageHint) -> Result<Vec<u8>> {
        debug!("🔒 Encrypting for lineage via BearDog (hint: {:?})", hint);

        // Encode message as base64 for JSON-RPC
        use base64::{engine::general_purpose, Engine as _};
        let plaintext_b64 = general_purpose::STANDARD.encode(message);

        // Build request params
        let params = serde_json::json!({
            "plaintext": plaintext_b64,
            "family_id": self.family_id,
            "lineage_hint": format!("{:?}", hint) // Serialized for BearDog
        });

        // Call birdsong.encrypt
        let result = self.call_beardog("birdsong.encrypt", params).await?;

        // Extract ciphertext
        let ciphertext_b64 = result
            .get("ciphertext")
            .or_else(|| result.get("encrypted")) // v1 compatibility
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::LineageRelayError::BirdSongError(
                    "No ciphertext in BearDog encrypt response".to_string(),
                )
            })?;

        let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!(
                "Invalid base64 ciphertext: {e}"
            ))
        })?;

        debug!("✅ Encrypted {} → {} bytes", message.len(), ciphertext.len());
        Ok(ciphertext)
    }

    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>> {
        debug!("🔓 Decrypting BirdSong from {:?}", sender);

        // Encode ciphertext as base64 for JSON-RPC
        use base64::{engine::general_purpose, Engine as _};
        let ciphertext_b64 = general_purpose::STANDARD.encode(encrypted);

        // Build request params
        let params = serde_json::json!({
            "ciphertext": ciphertext_b64,
            "sender_node_id": sender.0
        });

        // Call birdsong.decrypt
        let result = if let Ok(r) = self.call_beardog("birdsong.decrypt", params).await {
            r
        } else {
            // Decryption failure might just mean different family (noise)
            debug!("🔇 BearDog decrypt failed - likely different family (noise)");
            return Ok(None);
        };

        // Check success flag
        let success = result.get("success").and_then(serde_json::Value::as_bool).unwrap_or(false);

        if !success {
            debug!("🔇 BearDog decrypt: different family (noise)");
            return Ok(None);
        }

        // Extract plaintext
        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::LineageRelayError::BirdSongError(
                "No plaintext in BearDog decrypt response".to_string(),
            )
        })?;

        let plaintext = general_purpose::STANDARD.decode(plaintext_b64).map_err(|e| {
            crate::error::LineageRelayError::BirdSongError(format!("Invalid base64 plaintext: {e}"))
        })?;

        debug!("✅ Decrypted {} bytes from family", plaintext.len());
        Ok(Some(plaintext))
    }
}

// ═══════════════════════════════════════════════════════════════════
// TEST MOCKS - Isolated under #[cfg(test)]
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
pub struct MockLineageProvider {
    /// Lineage graph: `node_id` → `parent_id`
    lineages: Arc<RwLock<HashMap<String, String>>>,
    /// Descendants: `ancestor_id` → list of `descendant_ids`
    descendants: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[cfg(test)]
impl MockLineageProvider {
    /// Create new mock lineage provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            lineages: Arc::new(RwLock::new(HashMap::new())),
            descendants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add lineage relationship (for testing)
    pub async fn add_lineage(&self, child: &str, parent: &str) {
        self.lineages.write().await.insert(child.to_string(), parent.to_string());

        // Add to descendants
        let mut descendants = self.descendants.write().await;
        descendants.entry(parent.to_string()).or_insert_with(Vec::new).push(child.to_string());
    }

    /// Check if node2 is an ancestor of node1
    pub async fn is_ancestor(&self, node: &str, potential_ancestor: &str) -> bool {
        let lineages = self.lineages.read().await;
        let mut current = node.to_string();

        // Walk up the lineage chain
        while let Some(parent) = lineages.get(&current) {
            if parent == potential_ancestor {
                return true;
            }
            current = parent.clone();
        }

        false
    }

    /// Check if node2 is a descendant of node1
    pub async fn is_descendant(&self, node: &str, potential_descendant: &str) -> bool {
        self.is_ancestor(potential_descendant, node).await
    }
}

#[cfg(test)]
impl Default for MockLineageProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
/// Mock `BirdSong` crypto (for testing)
pub struct MockBirdSongCrypto {
    lineage_provider: Arc<MockLineageProvider>,
    my_id: String,
}

#[cfg(test)]
impl MockBirdSongCrypto {
    /// Create new mock crypto
    #[must_use]
    pub fn new(lineage_provider: Arc<MockLineageProvider>, my_id: String) -> Self {
        Self {
            lineage_provider,
            my_id,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl BirdSongCrypto for MockBirdSongCrypto {
    async fn encrypt_for_lineage(&self, message: &[u8], _hint: LineageHint) -> Result<Vec<u8>> {
        // Mock: prepend "LINEAGE:" to indicate lineage-encrypted
        let mut encrypted = b"LINEAGE:".to_vec();
        encrypted.extend_from_slice(message);
        Ok(encrypted)
    }

    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>> {
        // Check if we're in sender's lineage
        let can_decrypt = self.lineage_provider.is_ancestor(&self.my_id, &sender.0).await
            || self.lineage_provider.is_descendant(&self.my_id, &sender.0).await;

        if can_decrypt && encrypted.starts_with(b"LINEAGE:") {
            Ok(Some(encrypted[8..].to_vec()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
/// Mock relay authority (for testing)
pub struct MockRelayAuthority {
    lineage_provider: Arc<MockLineageProvider>,
}

#[cfg(test)]
impl MockRelayAuthority {
    /// Create new mock relay authority
    #[must_use]
    pub fn new(lineage_provider: Arc<MockLineageProvider>) -> Self {
        Self {
            lineage_provider,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl RelayAuthority for MockRelayAuthority {
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        // Check if relay_node is ancestor of requester
        let authorized = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(RelayAuthorization {
            relay_node: relay_node.clone(),
            requester: requester.clone(),
            authorized,
            masking_level: if authorized {
                MaskingLevel::Masked
            } else {
                MaskingLevel::FullVisibility
            },
            ttl_seconds: 300,
            issued_at: SystemTime::now(),
            audit_token: format!("mock_token_{}", uuid::Uuid::new_v4()),
        })
    }

    async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        // Simple masking: masked for descendants
        let is_ancestor = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(if is_ancestor {
            MaskingLevel::Masked
        } else {
            MaskingLevel::FullVisibility
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_beardog_provider_creation() {
        let provider =
            BearDogBirdSongProvider::new("/tmp/beardog.sock", Some("test-family".to_string()));

        assert_eq!(provider.socket_path.to_str().unwrap(), "/tmp/beardog.sock");
        assert_eq!(provider.family_id, Some("test-family".to_string()));
    }

    #[tokio::test]
    async fn test_mock_lineage_provider() {
        let provider = MockLineageProvider::new();

        // Create lineage: child → parent → grandparent
        provider.add_lineage("child", "parent").await;
        provider.add_lineage("parent", "grandparent").await;

        assert!(provider.is_ancestor("child", "parent").await);
        assert!(provider.is_ancestor("child", "grandparent").await);
        assert!(!provider.is_ancestor("parent", "child").await);

        assert!(provider.is_descendant("parent", "child").await);
        assert!(provider.is_descendant("grandparent", "child").await);
    }

    #[tokio::test]
    async fn test_mock_birdsong_crypto() {
        let provider = Arc::new(MockLineageProvider::new());
        provider.add_lineage("child", "parent").await;

        let crypto = MockBirdSongCrypto::new(provider.clone(), "parent".to_string());

        let message = b"test message";
        let encrypted =
            crypto.encrypt_for_lineage(message, LineageHint::DirectAncestors).await.unwrap();

        // Parent should be able to decrypt child's message
        let decrypted = crypto.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();
        assert_eq!(decrypted, Some(message.to_vec()));

        // Unrelated node cannot decrypt
        let crypto_unrelated = MockBirdSongCrypto::new(provider, "unrelated".to_string());
        let decrypted_unrelated =
            crypto_unrelated.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();
        assert_eq!(decrypted_unrelated, None);
    }

    #[tokio::test]
    async fn test_mock_relay_authority() {
        let provider = Arc::new(MockLineageProvider::new());
        provider.add_lineage("child", "parent").await;

        let authority = MockRelayAuthority::new(provider);

        // Parent should be authorized to relay for child
        let auth = authority
            .authorize_relay(&NodeId::from("parent"), &NodeId::from("child"))
            .await
            .unwrap();
        assert!(auth.authorized);

        // Child should NOT be authorized to relay for parent
        let auth = authority
            .authorize_relay(&NodeId::from("child"), &NodeId::from("parent"))
            .await
            .unwrap();
        assert!(!auth.authorized);
    }
}
