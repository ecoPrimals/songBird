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

/// BearDog RPC client
#[derive(Debug)]
pub struct BearDogClient {
    socket_path: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl BearDogClient {
    /// Create a new BearDog client
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
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

    /// Call BearDog RPC method
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        trace!("→ BearDog RPC: {} (id={})", method, id);

        // Connect to BearDog
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| Error::BearDogRpc(format!("Failed to connect to BearDog at {}: {}", self.socket_path, e)))?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        let response: JsonRpcResponse = serde_json::from_slice(&buffer)
            .map_err(|e| Error::BearDogRpc(format!("Failed to parse BearDog response: {}", e)))?;

        trace!("← BearDog RPC: {} result (id={})", method, response.id);

        // Check for errors
        if let Some(error) = response.error {
            return Err(Error::BearDogRpc(format!("{} (code: {})", error.message, error.code)));
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
        let client = BearDogClient::new("/tmp/beardog.sock");
        assert_eq!(client.socket_path, "/tmp/beardog.sock");
    }

    #[test]
    fn test_request_id_increment() {
        let client = BearDogClient::new("/tmp/beardog.sock");
        let id1 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let id2 = client.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(id2, id1 + 1);
    }
}

