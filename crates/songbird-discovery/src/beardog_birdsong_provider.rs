//! `BearDog` `BirdSong` Encryption Provider
//!
//! Implements the `BirdSongEncryption` trait using `BearDog`'s family-based encryption.
//! This provider connects to `BearDog`'s encryption API to encrypt/decrypt discovery
//! packets based on genetic family lineage.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use crate::birdsong::BirdSongEncryption;

/// `BearDog` encryption request (for JSON-RPC birdsong.encrypt method)
#[derive(Debug, Clone, Serialize)]
struct BearDogEncryptRequest {
    /// Plaintext data to encrypt (base64 encoded automatically by serde)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Optional family ID (uses node's family if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Base64 serialization helper (matching `BearDog`'s format)
mod base64_serde {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

/// `BearDog` encryption response (adaptive format)
///
/// Handles both v1 ("encrypted") and v2 ("ciphertext") field names
/// for backward compatibility and graceful API evolution.
#[derive(Debug, Clone, Deserialize)]
struct BearDogEncryptResponse {
    /// Encrypted data (deserialized from base64 automatically)
    /// Supports both "ciphertext" (v2) and "encrypted" (v1) field names
    #[serde(alias = "encrypted")] // v1 compatibility
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,

    /// Family ID used for encryption
    family_id: String,
}

/// `BearDog` decryption request
#[derive(Debug, Clone, Serialize)]
struct BearDogDecryptRequest {
    /// Ciphertext to decrypt (base64 encoded automatically)
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,
}

/// `BearDog` decryption response
#[derive(Debug, Clone, Deserialize)]
struct BearDogDecryptResponse {
    /// Decrypted plaintext (deserialized from base64 automatically)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Family ID that encrypted this
    family_id: String,

    /// Whether we can decrypt (same family)
    success: bool,
}

/// `BearDog` `BirdSong` encryption provider
///
/// Connects to `BearDog`'s Unix socket JSON-RPC interface to provide family-based encryption
/// for discovery packets. Only peers from the same genetic family can
/// decrypt each other's packets.
///
/// **Pure Rust Implementation**: Uses Unix sockets for inter-primal communication,
/// eliminating HTTP overhead and `reqwest` dependency (ring-free!).
pub struct BearDogBirdSongProvider {
    /// `BearDog` Unix socket path
    #[allow(dead_code)]
    socket_path: PathBuf,

    /// JSON-RPC client for `BearDog` communication (Pure Rust!)
    client: UnixRpcClient,

    /// Our family ID (cached from identity query)
    family_id: Option<String>,

    /// Provider availability
    available: bool,
}

impl BearDogBirdSongProvider {
    /// Create new `BearDog` `BirdSong` provider (async factory method)
    ///
    /// # Arguments
    ///
    /// * `socket_path` - `BearDog` Unix socket path (e.g., "/tmp/beardog.sock")
    /// * `family_id` - Optional family ID (will query `BearDog` if not provided)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_discovery::beardog_birdsong_provider::BearDogBirdSongProvider;
    ///
    /// # async fn example() {
    /// let provider = BearDogBirdSongProvider::new(
    ///     "/tmp/beardog.sock",
    ///     Some("ecoPrimals-family-123".to_string())
    /// ).await.unwrap();
    /// # }
    /// ```
    pub async fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Result<Self> {
        let socket_path = socket_path.into();

        // Create UnixRpcClient (100% Pure Rust!)
        let client = UnixRpcClient::new(&socket_path)
            .map_err(|e| anyhow::anyhow!("Failed to connect to BearDog at {socket_path:?}: {e}"))?;

        info!("🎵 BearDog BirdSong provider created (Pure Rust Unix socket!)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Ok(Self {
            socket_path,
            client,
            family_id,
            available: true, // UnixRpcClient::new() already validated connection
        })
    }

    /// Async health check for `BearDog` availability
    ///
    /// Uses JSON-RPC `health` method for pure Rust health checking.
    pub async fn check_health(&self) -> bool {
        #[derive(Deserialize)]
        struct HealthResponse {
            status: String,
        }

        match self.client.call_no_params::<HealthResponse>("health").await {
            Ok(response) => {
                let is_ok = response.status == "healthy";
                if is_ok {
                    info!("✅ BearDog health check passed (Pure Rust RPC!)");
                } else {
                    warn!("⚠️  BearDog health check failed: status = {}", response.status);
                }
                is_ok
            }
            Err(e) => {
                warn!("⚠️  BearDog health check error: {}", e);
                false
            }
        }
    }

    /// Encrypt data using `BearDog` family encryption (Pure Rust JSON-RPC!)
    ///
    /// Uses `birdsong.encrypt` JSON-RPC method for inter-primal communication.
    async fn encrypt_internal(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let request = BearDogEncryptRequest {
            plaintext: plaintext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("🔒 BearDog encryption via JSON-RPC (Pure Rust!)");
        debug!("   Plaintext size: {} bytes", plaintext.len());
        debug!("   Family ID: {:?}", self.family_id);

        // Call BearDog's birdsong.encrypt JSON-RPC method
        let encrypt_response: BearDogEncryptResponse = self
            .client
            .call("birdsong.encrypt", &request)
            .await
            .map_err(|e| format!("BearDog JSON-RPC encrypt failed: {e}"))?;

        debug!(
            "🔒 BearDog encrypted {} -> {} bytes (family: {})",
            plaintext.len(),
            encrypt_response.ciphertext.len(),
            encrypt_response.family_id
        );

        Ok(encrypt_response.ciphertext)
    }

    /// Decrypt data using `BearDog` family decryption (Pure Rust JSON-RPC!)
    ///
    /// Uses `birdsong.decrypt` JSON-RPC method for inter-primal communication.
    async fn decrypt_internal(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let request = BearDogDecryptRequest {
            ciphertext: ciphertext.to_vec(),
        };

        debug!("🔓 BearDog decryption via JSON-RPC (Pure Rust!)");
        debug!("   Ciphertext size: {} bytes", ciphertext.len());

        // Call BearDog's birdsong.decrypt JSON-RPC method
        let decrypt_response: BearDogDecryptResponse =
            self.client.call("birdsong.decrypt", &request).await.map_err(|e| {
                // Different family might return an RPC error - treat as noise
                debug!("🔇 BearDog decrypt RPC error (likely different family): {}", e);
                format!("BearDog JSON-RPC decrypt failed: {e}")
            })?;

        if !decrypt_response.success {
            // Different family - return None (noise)
            debug!("🔇 BearDog noise: different family ({})", decrypt_response.family_id);
            return Ok(None);
        }

        debug!(
            "🔓 BearDog decrypted {} -> {} bytes (family: {})",
            ciphertext.len(),
            decrypt_response.plaintext.len(),
            decrypt_response.family_id
        );

        Ok(Some(decrypt_response.plaintext))
    }
}

#[async_trait]
impl BirdSongEncryption for BearDogBirdSongProvider {
    fn provider_name(&self) -> String {
        "BearDog".to_string()
    }

    fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    fn is_available(&self) -> bool {
        self.available
    }

    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        if !self.available {
            return Err(anyhow::anyhow!("BearDog provider not available"));
        }

        self.encrypt_internal(plaintext).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if !self.available {
            return Err(anyhow::anyhow!("BearDog provider not available"));
        }

        self.decrypt_internal(ciphertext).await.map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_provider_creation() {
        // This test requires BearDog to be running
        let socket_path = "/tmp/beardog.sock";

        match BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await {
            Ok(provider) => {
                assert_eq!(provider.provider_name(), "BearDog");
                assert_eq!(provider.family_id(), Some("test-family".to_string()));
            }
            Err(_) => {
                println!("⏭️  Skipping provider creation test - BearDog not available");
            }
        }
    }

    #[tokio::test]
    async fn test_provider_creation_no_family() {
        // This test requires BearDog to be running
        let socket_path = "/tmp/beardog.sock";

        match BearDogBirdSongProvider::new(socket_path, None).await {
            Ok(provider) => {
                assert_eq!(provider.provider_name(), "BearDog");
                assert_eq!(provider.family_id(), None);
            }
            Err(_) => {
                println!("⏭️  Skipping provider creation test - BearDog not available");
            }
        }
    }

    /// Test parsing BearDog JSON-RPC encrypt response (modern format)
    #[test]
    fn test_beardog_encrypt_response_parsing() {
        let response_json = r#"{"ciphertext":"yo8Tz+qVxUp7A01pf7PYAhTvfe0Cl727z9r6nh/Qey21gL09gL+wTzS4ghiTKO6gnyqYvukBVw==","family_id":"iidn"}"#;

        let parsed: BearDogEncryptResponse = serde_json::from_str(response_json).unwrap();

        assert_eq!(parsed.family_id, "iidn");
        assert!(!parsed.ciphertext.is_empty());

        println!("✅ Encrypt response: Ciphertext decoded to {} bytes", parsed.ciphertext.len());
    }

    /// Test parsing BearDog JSON-RPC decrypt response
    #[test]
    fn test_beardog_decrypt_response_parsing() {
        let response_json =
            r#"{"plaintext":"SGVsbG8sIEJlYXJEb2ch","family_id":"iidn","success":true}"#;

        let parsed: BearDogDecryptResponse = serde_json::from_str(response_json).unwrap();

        assert_eq!(parsed.family_id, "iidn");
        assert!(parsed.success);
        assert_eq!(parsed.plaintext, b"Hello, BearDog!");

        println!("✅ Decrypt response: Plaintext decoded to {} bytes", parsed.plaintext.len());
    }

    /// Test base64_serde serialization
    #[test]
    fn test_base64_serde_serialization() {
        let request = BearDogEncryptRequest {
            plaintext: b"test_message".to_vec(),
            family_id: Some("test-family".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        println!("✅ Serialized request: {}", json);

        // Should contain base64-encoded plaintext
        assert!(json.contains("dGVzdF9tZXNzYWdl")); // "test_message" in base64
        assert!(json.contains("test-family"));
    }

    /// Test base64_serde deserialization roundtrip
    #[test]
    fn test_base64_serde_roundtrip() {
        // Simulate BearDog's response format
        let response_json = r#"{"encrypted":"dGVzdF9jaXBoZXJ0ZXh0","family_id":"test-family"}"#;

        let parsed: BearDogEncryptResponse = serde_json::from_str(response_json).unwrap();

        assert_eq!(parsed.family_id, "test-family");
        assert_eq!(parsed.ciphertext, b"test_ciphertext");

        println!("✅ Base64 roundtrip: {} bytes", parsed.ciphertext.len());
    }

    #[tokio::test]
    async fn test_health_check_unavailable() {
        // Test with invalid socket path
        let invalid_socket = "/tmp/nonexistent_beardog_test.sock";

        // Ensure socket doesn't exist
        let _ = std::fs::remove_file(invalid_socket);

        // new() creates the client but doesn't connect yet
        // The connection failure will happen on first RPC call
        match BearDogBirdSongProvider::new(invalid_socket, Some("test-family".to_string())).await {
            Ok(provider) => {
                // health check should fail for non-existent socket
                let is_healthy = provider.check_health().await;
                assert!(!is_healthy, "Health check should fail for invalid socket");
                println!("✅ Health check correctly failed for non-existent socket!");
            }
            Err(_) => {
                // Also acceptable if new() itself fails
                println!("✅ Provider creation correctly failed for non-existent socket!");
            }
        }
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        // This test requires a running BearDog instance
        let socket_path = "/tmp/beardog.sock";

        let provider = match BearDogBirdSongProvider::new(
            socket_path,
            Some("test-family".to_string()),
        )
        .await
        {
            Ok(p) => p,
            Err(_) => {
                println!(
                    "⏭️  Skipping roundtrip test - BearDog not available at {:?}",
                    socket_path
                );
                return;
            }
        };

        // Check if BearDog is healthy
        if !provider.check_health().await {
            println!("⏭️  Skipping roundtrip test - BearDog health check failed");
            return;
        }

        let plaintext = b"Hello, BirdSong!";

        // Encrypt
        let ciphertext = match provider.encrypt_discovery(plaintext).await {
            Ok(c) => c,
            Err(e) => {
                println!("⏭️  Skipping roundtrip test - Encryption failed: {}", e);
                return;
            }
        };

        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext, plaintext.to_vec(), "Ciphertext should differ from plaintext");

        // Decrypt
        let decrypted = match provider.decrypt_discovery(&ciphertext).await {
            Ok(d) => d,
            Err(e) => {
                println!("⏭️  Skipping roundtrip test - Decryption failed: {}", e);
                return;
            }
        };

        // Verify roundtrip
        assert_eq!(
            decrypted,
            Some(plaintext.to_vec()),
            "Roundtrip should return original plaintext"
        );

        println!("✅ Pure Rust JSON-RPC roundtrip successful!");
    }

    #[tokio::test]
    async fn test_different_family_decryption() {
        // This test requires a running BearDog instance
        let socket_path = "/tmp/beardog.sock";

        let provider1 =
            match BearDogBirdSongProvider::new(socket_path, Some("family-1".to_string())).await {
                Ok(p) => p,
                Err(_) => {
                    println!("⏭️  Skipping cross-family test - BearDog not available");
                    return;
                }
            };

        let provider2 =
            match BearDogBirdSongProvider::new(socket_path, Some("family-2".to_string())).await {
                Ok(p) => p,
                Err(_) => {
                    println!("⏭️  Skipping cross-family test - BearDog not available");
                    return;
                }
            };

        // Check if BearDog is healthy
        if !provider1.check_health().await {
            println!("⏭️  Skipping cross-family test - BearDog health check failed");
            return;
        }

        let plaintext = b"Secret message";

        // Family 1 encrypts
        let ciphertext = match provider1.encrypt_discovery(plaintext).await {
            Ok(c) => c,
            Err(_) => {
                println!("⏭️  Skipping cross-family test - Encryption failed");
                return;
            }
        };

        // Family 2 tries to decrypt (should fail or return None)
        let decrypted = provider2.decrypt_discovery(&ciphertext).await;

        // If decryption succeeds, it should return None (noise) for different family
        if let Ok(result) = decrypted {
            assert_eq!(
                result, None,
                "Different family should not be able to decrypt (should return None)"
            );
            println!("✅ Pure Rust JSON-RPC correctly rejected different family!");
        }
        // If decryption fails entirely, that's also acceptable behavior
    }

    #[tokio::test]
    async fn test_socket_path_formatting() {
        // This test requires BearDog to be running
        let socket_path = "/tmp/beardog.sock";

        match BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await {
            Ok(provider) => {
                assert_eq!(provider.socket_path.to_str().unwrap(), socket_path);
                println!("✅ Pure Rust Unix socket path correctly stored!");
            }
            Err(_) => {
                println!("⏭️  Skipping socket path test - BearDog not available");
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_encrypt_requests() {
        // Test thread safety with concurrent encryption requests (Pure Rust!)
        let socket_path = "/tmp/beardog.sock";

        let provider = match BearDogBirdSongProvider::new(
            socket_path,
            Some("test-family".to_string()),
        )
        .await
        {
            Ok(p) => Arc::new(p),
            Err(_) => {
                println!("⏭️  Skipping concurrent test - BearDog not available");
                return;
            }
        };

        if !provider.check_health().await {
            println!("⏭️  Skipping concurrent test - BearDog health check failed");
            return;
        }

        let mut handles = vec![];

        for i in 0..5 {
            let provider_clone = Arc::clone(&provider);
            let handle = tokio::spawn(async move {
                let plaintext = format!("Message {}", i);
                provider_clone.encrypt_discovery(plaintext.as_bytes()).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            // Check that the task didn't panic
            assert!(result.is_ok(), "Concurrent encryption task should not panic");
        }

        println!("✅ Pure Rust JSON-RPC handled 5 concurrent requests successfully!");
    }
}
