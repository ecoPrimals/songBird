//! BearDog RPC communication
//!
//! JSON-RPC call implementation with dual-mode support (Direct and Neural API).

use super::core::{BearDogClient, BearDogMode};
use super::types::{JsonRpcRequest, JsonRpcResponse};
use crate::error::{Error, Result};
use serde_json::{json, Value};
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Platform-agnostic IPC transport
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, trace};

impl BearDogClient {
    /// Map semantic capability names to actual BearDog method names
    ///
    /// **DEPRECATED**: This mapping is only used in Direct mode for backward compatibility.
    /// In production (Neural API mode), semantic translation is handled by Neural API's
    /// capability registry, allowing BearDog to evolve its API independently.
    ///
    /// (Used only in Direct mode)
    #[deprecated(
        since = "0.2.0",
        note = "Use Neural API's capability.call for semantic routing in production. Direct mode is for testing only."
    )]
    fn semantic_to_actual(&self, capability: &str) -> Result<&'static str> {
        Ok(match capability {
            // Crypto operations - map to BearDog's actual method names
            "crypto.generate_keypair" => "crypto.x25519_generate_ephemeral",
            "crypto.ecdh_derive" => "crypto.x25519_derive_secret",
            "crypto.encrypt" => "crypto.chacha20_poly1305_encrypt",
            "crypto.decrypt" => "crypto.chacha20_poly1305_decrypt",
            "crypto.encrypt_aes_128_gcm" => "crypto.aes128_gcm_encrypt",
            "crypto.decrypt_aes_128_gcm" => "crypto.aes128_gcm_decrypt",
            "crypto.encrypt_aes_256_gcm" => "crypto.aes256_gcm_encrypt",
            "crypto.decrypt_aes_256_gcm" => "crypto.aes256_gcm_decrypt",
            "crypto.sha256" => "crypto.sha256",
            "crypto.sha384" => "crypto.sha384",
            "crypto.hkdf_extract" => "crypto.hkdf_extract",
            "crypto.hkdf_expand" => "crypto.hkdf_expand",

            // TLS key derivation - already correct
            "tls.derive_handshake_secrets" => "tls.derive_handshake_secrets",
            "tls.derive_application_secrets" => "tls.derive_application_secrets",
            "tls.compute_finished_verify_data" => "tls.compute_finished_verify_data",

            _ => {
                return Err(Error::BearDogRpc(format!(
                    "Unknown capability: {}. Add mapping to semantic_to_actual()",
                    capability
                )))
            }
        })
    }

    /// Platform-agnostic connection helper
    ///
    /// - Unix/macOS/Android: Unix domain sockets
    /// - Windows: TCP localhost
    #[cfg(unix)]
    async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(path).await
    }

    #[cfg(windows)]
    async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(address).await
    }

    #[cfg(not(any(unix, windows)))]
    async fn connect_platform(address: &str) -> std::io::Result<tokio::net::TcpStream> {
        tokio::net::TcpStream::connect(address).await
    }

    /// Make an RPC call to BearDog
    ///
    /// In Direct mode: Calls BearDog directly using actual method names
    /// In Neural API mode: Routes through Neural API for capability translation
    pub(super) async fn call(&self, capability: &str, args: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        match &self.mode {
            BearDogMode::Direct {
                socket_path,
            } => self.call_direct(socket_path, capability, args, id).await,
            BearDogMode::NeuralApi {
                socket_path,
            } => self.call_neural_api(socket_path, capability, args, id).await,
        }
    }

    /// Direct RPC to BearDog (testing, simple deployments)
    async fn call_direct(
        &self,
        socket_path: &str,
        capability: &str,
        args: Value,
        id: u64,
    ) -> Result<Value> {
        // Note: Direct mode is deprecated for production use
        #[allow(deprecated)]
        let method = self.semantic_to_actual(capability)?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: args,
            id,
        };

        trace!("→ BearDog direct RPC: {} (id={})", method, id);

        // Connect to BearDog directly (platform-agnostic)
        let mut stream = Self::connect_platform(socket_path).await.map_err(|e| {
            Error::BearDogRpc(format!("Failed to connect to BearDog at {}: {}", socket_path, e))
        })?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Shutdown write to signal we're done
        stream.shutdown().await?;

        // Read response
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        let response: JsonRpcResponse = serde_json::from_slice(&buffer)
            .map_err(|e| Error::BearDogRpc(format!("Invalid JSON response: {}", e)))?;

        if let Some(error) = response.error {
            return Err(Error::BearDogRpc(format!(
                "BearDog error: {} (code: {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| Error::BearDogRpc("No result in response".to_string()))
    }

    /// TRUE PRIMAL: Route through Neural API for semantic capability resolution
    async fn call_neural_api(
        &self,
        socket_path: &str,
        capability: &str,
        args: Value,
        id: u64,
    ) -> Result<Value> {
        // Split semantic name into capability + operation
        // e.g., "crypto.generate_keypair" → capability:"crypto", operation:"generate_keypair"
        let parts: Vec<&str> = capability.split('.').collect();
        let (cap, op) = if parts.len() >= 2 {
            (parts[0], parts[1..].join("."))
        } else {
            ("crypto", capability.to_string())
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "capability.call".to_string(),
            params: json!({
                "capability": cap,
                "operation": op,
                "args": args
            }),
            id,
        };

        trace!("→ Neural API capability.call: {}.{} (id={})", cap, op, id);

        // Connect to Neural API (platform-agnostic)
        let mut stream = Self::connect_platform(socket_path).await.map_err(|e| {
            Error::BearDogRpc(format!("Failed to connect to Neural API at {}: {}", socket_path, e))
        })?;

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Shutdown write to signal we're done
        stream.shutdown().await?;

        // Read response with JSON-aware reading (Neural API keeps socket open)
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 4096];
        let read_timeout = Duration::from_millis(100);

        loop {
            match timeout(read_timeout, stream.read(&mut temp_buf)).await {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    // Check for complete JSON
                    if let Ok(s) = std::str::from_utf8(&buffer) {
                        if serde_json::from_str::<Value>(s).is_ok() {
                            break; // Complete JSON received!
                        }
                    }
                }
                Ok(Err(e)) => return Err(Error::BearDogRpc(format!("Socket read error: {}", e))),
                Err(_) => {
                    // Timeout - check if we have valid JSON
                    if !buffer.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&buffer) {
                            if serde_json::from_str::<Value>(s).is_ok() {
                                break;
                            }
                        }
                    }
                    return Err(Error::BearDogRpc("Timeout reading from Neural API".to_string()));
                }
            }
        }

        // Log raw response for debugging
        if let Ok(response_str) = std::str::from_utf8(&buffer) {
            trace!(
                "← Raw Neural API response ({} bytes): {}",
                buffer.len(),
                if response_str.len() > 500 {
                    format!("{}... (truncated)", &response_str[..500])
                } else {
                    response_str.to_string()
                }
            );
        }

        let response: JsonRpcResponse = serde_json::from_slice(&buffer).map_err(|e| {
            error!("❌ Failed to parse Neural API response for {}: {}", capability, e);
            if let Ok(response_str) = std::str::from_utf8(&buffer) {
                error!("   Raw response: {}", response_str);
            }
            Error::BearDogRpc(format!("Failed to parse Neural API response: {}", e))
        })?;

        let id_str = response.id.map(|id| id.to_string()).unwrap_or_else(|| "null".to_string());
        trace!("← Neural API result for {} (id={})", capability, id_str);

        // Check for errors
        if let Some(error) = response.error {
            error!(
                "❌ Neural API error for {}: {} (code: {})",
                capability, error.message, error.code
            );
            return Err(Error::BearDogRpc(format!(
                "Neural API error for {}: {} (code: {})",
                capability, error.message, error.code
            )));
        }

        debug!("✅ Neural API call successful: {}", capability);
        response.result.ok_or_else(|| {
            error!("❌ Missing result in Neural API response for {}", capability);
            Error::BearDogRpc("Missing result in response".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_mapping() {
        let client = BearDogClient::new_direct("/tmp/test.sock");
        #[allow(deprecated)]
        {
            assert_eq!(
                client.semantic_to_actual("crypto.generate_keypair").unwrap(),
                "crypto.x25519_generate_ephemeral"
            );
            assert_eq!(
                client.semantic_to_actual("crypto.ecdh_derive").unwrap(),
                "crypto.x25519_derive_secret"
            );
        }
    }

    #[test]
    fn test_semantic_mapping_unknown() {
        let client = BearDogClient::new_direct("/tmp/test.sock");
        #[allow(deprecated)]
        let result = client.semantic_to_actual("unknown.method");
        assert!(result.is_err());
    }
}
