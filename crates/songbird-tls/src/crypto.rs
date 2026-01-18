//! BearDog crypto integration for TLS
//!
//! This module integrates with BearDog's crypto JSON-RPC API for TLS operations.
//! All cryptographic operations are delegated to BearDog via Unix sockets.

use crate::error::{Result, TlsError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// BearDog crypto client for TLS operations
///
/// Communicates with BearDog via Unix socket JSON-RPC.
#[derive(Clone)]
pub struct BeardogCryptoClient {
    socket_path: String,
}

impl BeardogCryptoClient {
    /// Create a new BearDog crypto client
    ///
    /// Uses runtime discovery to find the BearDog socket.
    pub async fn new() -> Result<Self> {
        let socket_path = Self::discover_socket()?;
        
        // Verify socket exists
        if !Path::new(&socket_path).exists() {
            return Err(TlsError::CryptoError(format!(
                "BearDog socket not found at: {}",
                socket_path
            )));
        }
        
        Ok(Self { socket_path })
    }

    /// Create a client with explicit socket path (for testing)
    pub fn with_socket_path(socket_path: String) -> Self {
        Self { socket_path }
    }

    /// Discover BearDog crypto socket
    ///
    /// Uses capability-based discovery (no hardcoding!)
    fn discover_socket() -> Result<String> {
        // Strategy 1: Environment variable (highest priority)
        if let Ok(path) = std::env::var("SONGBIRD_CRYPTO_SOCKET") {
            return Ok(path);
        }

        // Strategy 2: BearDog-specific environment variable
        if let Ok(path) = std::env::var("BEARDOG_CRYPTO_SOCKET") {
            return Ok(path);
        }

        // Strategy 3: Default paths
        let default_paths = vec![
            "/tmp/beardog-crypto.sock",
            "/var/run/beardog/crypto.sock",
            "/run/beardog/crypto.sock",
        ];

        for path in default_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Strategy 4: Search /tmp for any crypto socket
        if let Ok(entries) = std::fs::read_dir("/tmp") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().contains("crypto") && name.to_string_lossy().ends_with(".sock") {
                        return Ok(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Err(TlsError::CryptoError(
            "Could not discover BearDog crypto socket".to_string(),
        ))
    }

    /// Make a JSON-RPC call to BearDog
    async fn call_jsonrpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        // Connect to Unix socket
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to connect to BearDog: {}", e)))?;

        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Serialize request
        let request_str = serde_json::to_string(&request)
            .map_err(|e| TlsError::InternalError(format!("Failed to serialize request: {}", e)))?;

        // Send request (with newline delimiter)
        stream
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send request: {}", e)))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send newline: {}", e)))?;

        // Read response
        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to read response: {}", e)))?;

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| TlsError::CryptoError(format!("Failed to parse response: {}", e)))?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(TlsError::CryptoError(format!(
                "BearDog error: {} (code {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            TlsError::CryptoError("BearDog response missing result field".to_string())
        })
    }

    /// Generate X25519 ephemeral keypair
    ///
    /// Returns: (public_key, secret_key) as raw bytes
    pub async fn x25519_generate_ephemeral(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let params = serde_json::json!({
            "purpose": "tls_handshake"
        });

        let result = self.call_jsonrpc("crypto.x25519_generate_ephemeral", params).await?;

        // Extract public_key and secret_key (base64 encoded)
        let public_key_b64 = result["public_key"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing public_key in response".to_string()))?;
        let secret_key_b64 = result["secret_key"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing secret_key in response".to_string()))?;

        let public_key = base64::decode(public_key_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode public_key: {}", e)))?;
        let secret_key = base64::decode(secret_key_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode secret_key: {}", e)))?;

        Ok((public_key, secret_key))
    }

    /// Derive X25519 shared secret
    ///
    /// Returns: shared_secret as raw bytes
    pub async fn x25519_derive_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "our_secret": base64::encode(our_secret),
            "their_public": base64::encode(their_public)
        });

        let result = self.call_jsonrpc("crypto.x25519_derive_secret", params).await?;

        let shared_secret_b64 = result["shared_secret"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing shared_secret in response".to_string()))?;

        let shared_secret = base64::decode(shared_secret_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode shared_secret: {}", e)))?;

        Ok(shared_secret)
    }

    /// Encrypt with ChaCha20-Poly1305 (AEAD)
    ///
    /// BearDog generates the nonce.
    /// Returns: (ciphertext, nonce, tag)
    pub async fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let mut params = serde_json::json!({
            "plaintext": base64::encode(plaintext),
            "key": base64::encode(key)
        });

        if let Some(aad_data) = aad {
            params["aad"] = serde_json::json!(base64::encode(aad_data));
        }

        let result = self.call_jsonrpc("crypto.chacha20_poly1305_encrypt", params).await?;

        let ciphertext_b64 = result["ciphertext"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing ciphertext in response".to_string()))?;
        let nonce_b64 = result["nonce"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing nonce in response".to_string()))?;
        let tag_b64 = result["tag"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing tag in response".to_string()))?;

        let ciphertext = base64::decode(ciphertext_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode ciphertext: {}", e)))?;
        let nonce = base64::decode(nonce_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode nonce: {}", e)))?;
        let tag = base64::decode(tag_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode tag: {}", e)))?;

        Ok((ciphertext, nonce, tag))
    }

    /// Decrypt with ChaCha20-Poly1305 (AEAD)
    ///
    /// Returns: plaintext
    pub async fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let mut params = serde_json::json!({
            "ciphertext": base64::encode(ciphertext),
            "key": base64::encode(key),
            "nonce": base64::encode(nonce),
            "tag": base64::encode(tag)
        });

        if let Some(aad_data) = aad {
            params["aad"] = serde_json::json!(base64::encode(aad_data));
        }

        let result = self.call_jsonrpc("crypto.chacha20_poly1305_decrypt", params).await?;

        let plaintext_b64 = result["plaintext"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing plaintext in response".to_string()))?;

        let plaintext = base64::decode(plaintext_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode plaintext: {}", e)))?;

        Ok(plaintext)
    }

    /// Sign with Ed25519
    ///
    /// Returns: signature (64 bytes)
    pub async fn ed25519_sign(&self, message: &[u8], key_id: &str) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "message": base64::encode(message),
            "key_id": key_id,
            "purpose": "certificate_signing"
        });

        let result = self.call_jsonrpc("crypto.sign_ed25519", params).await?;

        let signature_b64 = result["signature"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing signature in response".to_string()))?;

        let signature = base64::decode(signature_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode signature: {}", e)))?;

        Ok(signature)
    }

    /// HMAC-SHA256
    ///
    /// Returns: MAC (32 bytes)
    pub async fn hmac_sha256(&self, message: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "message": base64::encode(message),
            "key": base64::encode(key)
        });

        let result = self.call_jsonrpc("crypto.hmac_sha256", params).await?;

        let mac_b64 = result["mac"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing mac in response".to_string()))?;

        let mac = base64::decode(mac_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode mac: {}", e)))?;

        Ok(mac)
    }
}

// JSON-RPC types
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_socket_env_var() {
        // Set environment variable
        std::env::set_var("SONGBIRD_CRYPTO_SOCKET", "/tmp/test.sock");
        
        let result = BeardogCryptoClient::discover_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test.sock");
        
        // Cleanup
        std::env::remove_var("SONGBIRD_CRYPTO_SOCKET");
    }

    #[test]
    fn test_with_socket_path() {
        let client = BeardogCryptoClient::with_socket_path("/tmp/custom.sock".to_string());
        assert_eq!(client.socket_path, "/tmp/custom.sock");
    }

    // Note: Integration tests with live BearDog are in tests/ directory
}

