//! BearDog RPC client for crypto operations
//!
//! Communicates with BearDog via JSON-RPC 2.0 over Unix sockets.

use crate::error::{Error, Result};
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, trace};

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// BearDog RPC client (routes through Neural API for capability translation)
#[derive(Debug)]
pub struct BearDogClient {
    neural_api_socket: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl BearDogClient {
    /// Create a new BearDog client that routes through Neural API
    /// 
    /// This client uses Neural API's capability translation to convert
    /// semantic capability names (e.g., "crypto.generate_keypair") to
    /// actual provider method names (e.g., "x25519_generate_ephemeral").
    /// 
    /// # Arguments
    /// * `neural_api_socket` - Path to Neural API socket (e.g., "/tmp/neural-api-nat0.sock")
    pub fn new(neural_api_socket: impl Into<String>) -> Self {
        Self {
            neural_api_socket: neural_api_socket.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Create from environment variable (fallback to default)
    pub fn from_env() -> Self {
        let socket = std::env::var("NEURAL_API_SOCKET")
            .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
        Self::new(socket)
    }

    /// Generate x25519 keypair
    pub async fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        debug!("🔑 Generating x25519 keypair via BearDog");
        
        let result = self.call("crypto.generate_keypair", json!({
            "algorithm": "x25519"
        })).await?;

        let public_key = result["public_key"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing public_key".to_string()))?;
        let private_key = result["private_key"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing private_key".to_string()))?;

        let public_key = BASE64_STANDARD.decode(public_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid public_key base64: {}", e)))?;
        let private_key = BASE64_STANDARD.decode(private_key)
            .map_err(|e| Error::BearDogRpc(format!("Invalid private_key base64: {}", e)))?;

        Ok((public_key, private_key))
    }

    /// Perform ECDH key exchange
    pub async fn ecdh_derive(&self, private_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
        debug!("🔐 Performing ECDH via BearDog");
        
        let result = self.call("crypto.ecdh_derive", json!({
            "private_key": BASE64_STANDARD.encode(private_key),
            "public_key": BASE64_STANDARD.encode(public_key)
        })).await?;

        let shared_secret = result["shared_secret"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing shared_secret".to_string()))?;

        BASE64_STANDARD.decode(shared_secret)
            .map_err(|e| Error::BearDogRpc(format!("Invalid shared_secret base64: {}", e)))
    }

    /// Derive TLS session secrets
    pub async fn tls_derive_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
    ) -> Result<TlsSecrets> {
        debug!("🔒 Deriving TLS secrets via BearDog");
        
        let result = self.call("tls.derive_secrets", json!({
            "shared_secret": BASE64_STANDARD.encode(shared_secret),
            "client_random": BASE64_STANDARD.encode(client_random),
            "server_random": BASE64_STANDARD.encode(server_random)
        })).await?;

        Ok(TlsSecrets {
            client_write_key: BASE64_STANDARD.decode(
                result["client_write_key"].as_str()
                    .ok_or_else(|| Error::BearDogRpc("Missing client_write_key".to_string()))?
            ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_key: {}", e)))?,
            
            server_write_key: BASE64_STANDARD.decode(
                result["server_write_key"].as_str()
                    .ok_or_else(|| Error::BearDogRpc("Missing server_write_key".to_string()))?
            ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_key: {}", e)))?,
            
            client_write_iv: BASE64_STANDARD.decode(
                result["client_write_iv"].as_str()
                    .ok_or_else(|| Error::BearDogRpc("Missing client_write_iv".to_string()))?
            ).map_err(|e| Error::BearDogRpc(format!("Invalid client_write_iv: {}", e)))?,
            
            server_write_iv: BASE64_STANDARD.decode(
                result["server_write_iv"].as_str()
                    .ok_or_else(|| Error::BearDogRpc("Missing server_write_iv".to_string()))?
            ).map_err(|e| Error::BearDogRpc(format!("Invalid server_write_iv: {}", e)))?,
        })
    }

    /// Encrypt data with ChaCha20-Poly1305
    pub async fn encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes via BearDog", plaintext.len());
        
        let result = self.call("crypto.encrypt", json!({
            "algorithm": "chacha20-poly1305",
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "plaintext": BASE64_STANDARD.encode(plaintext),
            "aad": BASE64_STANDARD.encode(aad)
        })).await?;

        let ciphertext = result["ciphertext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing ciphertext".to_string()))?;

        BASE64_STANDARD.decode(ciphertext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid ciphertext base64: {}", e)))
    }

    /// Decrypt data with ChaCha20-Poly1305
    pub async fn decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        trace!("🔓 Decrypting {} bytes via BearDog", ciphertext.len());
        
        let result = self.call("crypto.decrypt", json!({
            "algorithm": "chacha20-poly1305",
            "key": BASE64_STANDARD.encode(key),
            "nonce": BASE64_STANDARD.encode(nonce),
            "ciphertext": BASE64_STANDARD.encode(ciphertext),
            "aad": BASE64_STANDARD.encode(aad)
        })).await?;

        let plaintext = result["plaintext"]
            .as_str()
            .ok_or_else(|| Error::BearDogRpc("Missing plaintext".to_string()))?;

        BASE64_STANDARD.decode(plaintext)
            .map_err(|e| Error::BearDogRpc(format!("Invalid plaintext base64: {}", e)))
    }

    /// Call BearDog capability via Neural API translation
    /// 
    /// This method sends a `capability.call` request to Neural API, which:
    /// 1. Translates the semantic capability (e.g., "crypto.generate_keypair")
    /// 2. Looks up the actual provider (BearDog)
    /// 3. Routes the call to the actual method (e.g., "x25519_generate_ephemeral")
    /// 4. Returns the result transparently
    async fn call(&self, capability: &str, args: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // Build capability.call request for Neural API
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "capability.call".to_string(),
            params: json!({
                "capability": capability,
                "args": args
            }),
            id,
        };

        trace!("→ Neural API capability.call: {} (id={})", capability, id);

        // Connect to Neural API
        let mut stream = UnixStream::connect(&self.neural_api_socket)
            .await
            .map_err(|e| Error::BearDogRpc(format!(
                "Failed to connect to Neural API at {}: {}", 
                self.neural_api_socket, e
            )))?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        let response: JsonRpcResponse = serde_json::from_slice(&buffer)
            .map_err(|e| Error::BearDogRpc(format!("Failed to parse Neural API response: {}", e)))?;

        trace!("← Neural API result for {} (id={})", capability, response.id);

        // Check for errors
        if let Some(error) = response.error {
            return Err(Error::BearDogRpc(format!(
                "Neural API error for {}: {} (code: {})", 
                capability, error.message, error.code
            )));
        }

        response.result.ok_or_else(|| Error::BearDogRpc("Missing result in response".to_string()))
    }
}

/// TLS session secrets
#[derive(Debug)]
pub struct TlsSecrets {
    pub client_write_key: Vec<u8>,
    pub server_write_key: Vec<u8>,
    pub client_write_iv: Vec<u8>,
    pub server_write_iv: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beardog_client_creation() {
        let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
        assert_eq!(client.neural_api_socket, "/tmp/neural-api-nat0.sock");
    }

    #[test]
    fn test_request_id_increment() {
        let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
        let id1 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let id2 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(id2, id1 + 1);
    }
    
    #[test]
    fn test_from_env_default() {
        // When NEURAL_API_SOCKET is not set, should use default
        std::env::remove_var("NEURAL_API_SOCKET");
        let client = BearDogClient::from_env();
        assert_eq!(client.neural_api_socket, "/tmp/neural-api-nat0.sock");
    }
    
    #[test]
    fn test_from_env_custom() {
        // When NEURAL_API_SOCKET is set, should use it
        std::env::set_var("NEURAL_API_SOCKET", "/custom/path.sock");
        let client = BearDogClient::from_env();
        assert_eq!(client.neural_api_socket, "/custom/path.sock");
        std::env::remove_var("NEURAL_API_SOCKET");
    }
}

