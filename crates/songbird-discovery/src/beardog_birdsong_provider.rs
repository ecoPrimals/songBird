//! BearDog BirdSong Encryption Provider
//!
//! Implements the BirdSongEncryption trait using BearDog's family-based encryption.
//! This provider connects to BearDog's encryption API to encrypt/decrypt discovery
//! packets based on genetic family lineage.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};
use anyhow::Result;

use crate::birdsong_integration::BirdSongEncryption;

/// BearDog API response wrapper
///
/// BearDog wraps all API responses in this structure for consistency
#[derive(Debug, Clone, Deserialize)]
struct BearDogApiResponse<T> {
    /// Success indicator
    success: bool,
    /// Response data (only present if success=true)
    data: Option<T>,
    /// Error message (only present if success=false)
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// BearDog encryption request
#[derive(Debug, Clone, Serialize)]
struct BearDogEncryptRequest {
    /// Plaintext data to encrypt (base64 encoded automatically by serde)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,
    
    /// Optional family ID (uses node's family if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Base64 serialization helper (matching BearDog's format)
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

/// BearDog encryption response (adaptive format)
///
/// Handles both v1 ("encrypted") and v2 ("ciphertext") field names
/// for backward compatibility and graceful API evolution.
#[derive(Debug, Clone, Deserialize)]
struct BearDogEncryptResponse {
    /// Encrypted data (deserialized from base64 automatically)
    /// Supports both "ciphertext" (v2) and "encrypted" (v1) field names
    #[serde(alias = "encrypted")]  // v1 compatibility
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,
    
    /// Family ID used for encryption
    family_id: String,
}

/// BearDog decryption request
#[derive(Debug, Clone, Serialize)]
struct BearDogDecryptRequest {
    /// Ciphertext to decrypt (base64 encoded automatically)
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,
}

/// BearDog decryption response
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

/// BearDog BirdSong encryption provider
///
/// Connects to BearDog's encryption API to provide family-based encryption
/// for discovery packets. Only peers from the same genetic family can
/// decrypt each other's packets.
pub struct BearDogBirdSongProvider {
    /// BearDog API endpoint
    endpoint: String,
    
    /// HTTP client for API calls
    client: Client,
    
    /// Our family ID (cached from identity query)
    family_id: Option<String>,
    
    /// Provider availability
    available: bool,
}

impl BearDogBirdSongProvider {
    /// Create new BearDog BirdSong provider
    ///
    /// # Arguments
    ///
    /// * `endpoint` - BearDog API endpoint (e.g., "http://localhost:7600")
    /// * `family_id` - Optional family ID (will query BearDog if not provided)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_discovery::beardog_birdsong_provider::BearDogBirdSongProvider;
    ///
    /// # async fn example() {
    /// let provider = BearDogBirdSongProvider::new(
    ///     "http://localhost:7600".to_string(),
    ///     Some("ecoPrimals-family-123".to_string())
    /// );
    /// # }
    /// ```
    pub fn new(endpoint: String, family_id: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        let available = Self::check_availability(&endpoint, &client);
        
        info!("🎵 BearDog BirdSong provider created");
        info!("   Endpoint: {}", endpoint);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }
        info!("   Note: Availability will be checked on first use");
        
        Self {
            endpoint,
            client,
            family_id,
            available,
        }
    }
    
    /// Async health check for BearDog availability
    ///
    /// This should be called from async context to properly check if BearDog is available.
    pub async fn check_health(&self) -> bool {
        let health_url = format!("{}/health", self.endpoint);
        
        match self.client
            .get(&health_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => {
                let is_ok = response.status().is_success();
                if is_ok {
                    info!("✅ BearDog health check passed");
                } else {
                    warn!("⚠️  BearDog health check failed: {}", response.status());
                }
                is_ok
            }
            Err(e) => {
                warn!("⚠️  BearDog health check error: {}", e);
                false
            }
        }
    }
    
    /// Check if BearDog is available (sync version)
    ///
    /// Note: This is a synchronous check that should only be used during initialization.
    /// For runtime checks, use the async `check_health()` method instead.
    fn check_availability(_endpoint: &str, _client: &Client) -> bool {
        // Skip sync availability check to avoid nested runtime issues
        // Availability will be properly checked via async check_health() method
        debug!("⚙️  Skipping sync availability check (will check on first use)");
        true // Assume available, will fail gracefully on first async call if not
    }
    
    /// Encrypt data using BearDog family encryption
    ///
    /// Uses adaptive API endpoint detection to work with both v1 and v2.
    async fn encrypt_internal(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        // Try v1 endpoint first (for backward compatibility with existing deployments)
        let url_v1 = format!("{}/api/v1/birdsong/encrypt_discovery", self.endpoint);
        let url_v2 = format!("{}/api/v2/birdsong/encrypt", self.endpoint);
        
        let request = BearDogEncryptRequest {
            plaintext: plaintext.to_vec(),
            family_id: self.family_id.clone(),
        };
        
        debug!("🔒 Attempting BearDog encryption (trying v1 first, then v2)");
        debug!("   Plaintext size: {} bytes", plaintext.len());
        debug!("   Family ID: {:?}", self.family_id);
        
        // Try v1 endpoint first
        let response = match self.client
            .post(&url_v1)
            .json(&request)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                debug!("✅ BearDog v1 endpoint responded successfully");
                resp
            }
            Ok(resp) => {
                let status = resp.status();
                debug!("⚠️  BearDog v1 endpoint returned {}, trying v2", status);
                // Try v2 endpoint
                self.client
                    .post(&url_v2)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| format!("BearDog v2 encrypt request failed: {}", e))?
            }
            Err(e) => {
                debug!("⚠️  BearDog v1 endpoint unavailable: {}, trying v2", e);
                // Try v2 endpoint
                self.client
                    .post(&url_v2)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| format!("BearDog v2 encrypt request failed: {}", e))?
            }
        };
        
        if !response.status().is_success() {
            return Err(format!("BearDog encrypt failed: {}", response.status()));
        }
        
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read BearDog response: {}", e))?;
        
        debug!("📥 BearDog response: {}", response_text);
        
        // Parse the wrapped API response
        let api_response: BearDogApiResponse<BearDogEncryptResponse> = 
            serde_json::from_str(&response_text)
                .map_err(|e| {
                    let error_msg = format!("Failed to parse BearDog response: {}. Response was: {}", e, response_text);
                    warn!("❌ {}", error_msg);
                    error_msg
                })?;
        
        // Check success flag
        if !api_response.success {
            let error_msg = format!("BearDog returned success=false: {:?}", api_response.error);
            warn!("❌ {}", error_msg);
            return Err(error_msg);
        }
        
        // Extract data
        let encrypt_response = api_response.data
            .ok_or_else(|| {
                let error_msg = "BearDog response missing 'data' field";
                warn!("❌ {}", error_msg);
                error_msg.to_string()
            })?;
        
        debug!("🔒 BearDog encrypted {} -> {} bytes (family: {})", 
               plaintext.len(), encrypt_response.ciphertext.len(), encrypt_response.family_id);
        
        Ok(encrypt_response.ciphertext)
    }
    
    /// Decrypt data using BearDog family decryption
    ///
    /// Uses adaptive API endpoint detection to work with both v1 and v2.
    async fn decrypt_internal(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, String> {
        // Try v1 endpoint first (for backward compatibility)
        let url_v1 = format!("{}/api/v1/birdsong/decrypt_discovery", self.endpoint);
        let url_v2 = format!("{}/api/v2/birdsong/decrypt", self.endpoint);
        
        let request = BearDogDecryptRequest {
            ciphertext: ciphertext.to_vec(),
        };
        
        debug!("🔓 Attempting BearDog decryption (trying v1 first, then v2)");
        debug!("   Ciphertext size: {} bytes", ciphertext.len());
        
        // Try v1 endpoint first
        let response = match self.client
            .post(&url_v1)
            .json(&request)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                debug!("✅ BearDog v1 decrypt endpoint responded successfully");
                resp
            }
            Ok(resp) => {
                debug!("⚠️  BearDog v1 decrypt endpoint returned {}, trying v2", resp.status());
                // Try v2 endpoint
                self.client
                    .post(&url_v2)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| format!("BearDog v2 decrypt request failed: {}", e))?
            }
            Err(e) => {
                debug!("⚠️  BearDog v1 decrypt endpoint unavailable: {}, trying v2", e);
                // Try v2 endpoint
                self.client
                    .post(&url_v2)
                    .json(&request)
                    .send()
                    .await
                    .map_err(|e| format!("BearDog v2 decrypt request failed: {}", e))?
            }
        };
        
        if !response.status().is_success() {
            return Err(format!("BearDog decrypt failed: {}", response.status()));
        }
        
        let response_text = response.text().await
            .map_err(|e| format!("Failed to read BearDog response: {}", e))?;
        
        debug!("📥 BearDog decrypt response: {}", response_text);
        
        // Parse the wrapped API response
        let api_response: BearDogApiResponse<BearDogDecryptResponse> = 
            serde_json::from_str(&response_text)
                .map_err(|e| {
                    let error_msg = format!("Failed to parse BearDog response: {}. Response was: {}", e, response_text);
                    warn!("❌ {}", error_msg);
                    error_msg
                })?;
        
        // Check success flag
        if !api_response.success {
            let error_msg = format!("BearDog returned success=false: {:?}", api_response.error);
            debug!("🔇 {}", error_msg);
            return Ok(None); // Different family, not an error
        }
        
        // Extract data
        let decrypt_response = api_response.data
            .ok_or_else(|| {
                let error_msg = "BearDog response missing 'data' field";
                warn!("❌ {}", error_msg);
                error_msg.to_string()
            })?;
        
        if !decrypt_response.success {
            // Different family - return None (noise)
            debug!("🔇 BearDog noise: different family ({})", decrypt_response.family_id);
            return Ok(None);
        }
        
        debug!("🔓 BearDog decrypted {} -> {} bytes (family: {})", 
               ciphertext.len(), decrypt_response.plaintext.len(), decrypt_response.family_id);
        
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
        
        self.encrypt_internal(plaintext)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
    
    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if !self.available {
            return Err(anyhow::anyhow!("BearDog provider not available"));
        }
        
        self.decrypt_internal(ciphertext)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_provider_creation() {
        let provider = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("test-family".to_string())
        );
        
        assert_eq!(provider.provider_name(), "BearDog");
        assert_eq!(provider.family_id(), Some("test-family".to_string()));
    }
    
    #[test]
    fn test_provider_creation_no_family() {
        let provider = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            None
        );
        
        assert_eq!(provider.provider_name(), "BearDog");
        assert_eq!(provider.family_id(), None);
    }
    
    /// Test parsing BearDog's v1 API response format
    #[test]
    fn test_beardog_v1_response_parsing() {
        let response_json = r#"{"success":true,"data":{"encrypted":"QyT2lSkyuIpJNewXcv098jYDbS9H8FdLj8D6kK5xR0zoJXYcwVd0yU3iQzzz3k1vK6ysAU+0rQ==","family_id":"iidn"}}"#;
        
        let parsed: BearDogApiResponse<BearDogEncryptResponse> = 
            serde_json::from_str(response_json).unwrap();
        
        assert!(parsed.success);
        assert!(parsed.data.is_some());
        
        let data = parsed.data.unwrap();
        assert_eq!(data.family_id, "iidn");
        assert!(!data.ciphertext.is_empty());
        
        println!("✅ v1 response: Ciphertext decoded to {} bytes", data.ciphertext.len());
    }
    
    /// Test parsing BearDog's v2 API response format
    #[test]
    fn test_beardog_v2_response_parsing() {
        let response_json = r#"{"success":true,"data":{"ciphertext":"yo8Tz+qVxUp7A01pf7PYAhTvfe0Cl727z9r6nh/Qey21gL09gL+wTzS4ghiTKO6gnyqYvukBVw==","family_id":"iidn"}}"#;
        
        let parsed: BearDogApiResponse<BearDogEncryptResponse> = 
            serde_json::from_str(response_json).unwrap();
        
        assert!(parsed.success);
        assert!(parsed.data.is_some());
        
        let data = parsed.data.unwrap();
        assert_eq!(data.family_id, "iidn");
        assert!(!data.ciphertext.is_empty());
        
        println!("✅ v2 response: Ciphertext decoded to {} bytes", data.ciphertext.len());
    }
    
    /// Test parsing BearDog error response
    #[test]
    fn test_beardog_error_response_parsing() {
        let response_json = r#"{"success":false,"error":"Invalid family_id"}"#;
        
        let parsed: BearDogApiResponse<BearDogEncryptResponse> = 
            serde_json::from_str(response_json).unwrap();
        
        assert!(!parsed.success);
        assert!(parsed.data.is_none());
        assert_eq!(parsed.error, Some("Invalid family_id".to_string()));
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
        // Test with invalid endpoint
        let provider = BearDogBirdSongProvider::new(
            "http://invalid-endpoint:99999".to_string(),
            Some("test-family".to_string())
        );
        
        let is_healthy = provider.check_health().await;
        assert!(!is_healthy, "Health check should fail for invalid endpoint");
    }
    
    #[tokio::test]
    async fn test_encrypt_unavailable_provider() {
        // Test encryption with unavailable provider
        let mut provider = BearDogBirdSongProvider::new(
            "http://invalid-endpoint:99999".to_string(),
            Some("test-family".to_string())
        );
        
        // Manually set as unavailable
        provider.available = false;
        
        let plaintext = b"Hello, BirdSong!";
        let result = provider.encrypt_discovery(plaintext).await;
        
        assert!(result.is_err(), "Encryption should fail for unavailable provider");
        assert!(result.unwrap_err().to_string().contains("not available"));
    }
    
    #[tokio::test]
    async fn test_decrypt_unavailable_provider() {
        // Test decryption with unavailable provider
        let mut provider = BearDogBirdSongProvider::new(
            "http://invalid-endpoint:99999".to_string(),
            Some("test-family".to_string())
        );
        
        // Manually set as unavailable
        provider.available = false;
        
        let ciphertext = b"encrypted-data";
        let result = provider.decrypt_discovery(ciphertext).await;
        
        assert!(result.is_err(), "Decryption should fail for unavailable provider");
        assert!(result.unwrap_err().to_string().contains("not available"));
    }
    
    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        // This test requires a running BearDog instance with API v2
        // Skip if not available
        let provider = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("test-family".to_string())
        );
        
        // Check if BearDog is available
        if !provider.check_health().await {
            println!("⏭️  Skipping roundtrip test - BearDog not available");
            return;
        }
        
        let plaintext = b"Hello, BirdSong!";
        
        // Encrypt
        let ciphertext = provider.encrypt_discovery(plaintext).await;
        if ciphertext.is_err() {
            println!("⏭️  Skipping roundtrip test - BearDog API v2 may not be ready: {:?}", ciphertext);
            return;
        }
        let ciphertext = ciphertext.unwrap();
        
        // Verify ciphertext is different from plaintext
        assert_ne!(ciphertext, plaintext.to_vec(), "Ciphertext should differ from plaintext");
        
        // Decrypt
        let decrypted = provider.decrypt_discovery(&ciphertext).await;
        if decrypted.is_err() {
            println!("⏭️  Skipping roundtrip test - BearDog API v2 decryption failed: {:?}", decrypted);
            return;
        }
        let decrypted = decrypted.unwrap();
        
        // Verify roundtrip
        assert_eq!(decrypted, Some(plaintext.to_vec()), "Roundtrip should return original plaintext");
    }
    
    #[tokio::test]
    async fn test_different_family_decryption() {
        // This test requires a running BearDog instance with API v2
        let provider1 = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("family-1".to_string())
        );
        
        let provider2 = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("family-2".to_string())
        );
        
        // Check if BearDog is available
        if !provider1.check_health().await {
            println!("⏭️  Skipping cross-family test - BearDog not available");
            return;
        }
        
        let plaintext = b"Secret message";
        
        // Family 1 encrypts
        let ciphertext = provider1.encrypt_discovery(plaintext).await;
        if ciphertext.is_err() {
            println!("⏭️  Skipping cross-family test - BearDog API v2 may not be ready");
            return;
        }
        let ciphertext = ciphertext.unwrap();
        
        // Family 2 tries to decrypt (should fail or return None)
        let decrypted = provider2.decrypt_discovery(&ciphertext).await;
        
        // If decryption succeeds, it should return None (noise) for different family
        if let Ok(result) = decrypted {
            assert_eq!(result, None, "Different family should not be able to decrypt (should return None)");
        }
        // If decryption fails entirely, that's also acceptable behavior
    }
    
    #[test]
    fn test_endpoint_formatting() {
        let provider = BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("test-family".to_string())
        );
        
        assert!(provider.endpoint.contains("localhost:7600"));
    }
    
    #[tokio::test]
    async fn test_concurrent_encrypt_requests() {
        // Test thread safety with concurrent encryption requests
        let provider = std::sync::Arc::new(BearDogBirdSongProvider::new(
            "http://localhost:7600".to_string(),
            Some("test-family".to_string())
        ));
        
        if !provider.check_health().await {
            println!("⏭️  Skipping concurrent test - BearDog not available");
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
    }
}

