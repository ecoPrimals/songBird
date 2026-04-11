// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security-provider `BirdSong` encryption
//!
//! Implements the `BirdSongEncryption` trait using the security provider's family-based encryption.
//! This provider connects to the crypto provider's encryption API to encrypt/decrypt discovery
//! packets based on genetic family lineage.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use songbird_universal::UnixRpcClient;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, trace, warn};

use crate::birdsong::BirdSongEncryption;

/// `security provider` encryption request (for JSON-RPC birdsong.encrypt method)
#[derive(Debug, Clone, Serialize)]
struct SecurityProviderEncryptRequest {
    /// Plaintext data to encrypt (base64 encoded automatically by serde)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Optional family ID (uses node's family if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Base64 serialization helper (matching the provider's format)
mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
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

/// Security-provider encryption response (adaptive format)
///
/// Handles both v1 ("encrypted") and v2 ("ciphertext") field names
/// for backward compatibility and graceful API evolution.
#[derive(Debug, Clone, Deserialize)]
struct SecurityProviderEncryptResponse {
    /// Encrypted data (deserialized from base64 automatically)
    /// Supports both "ciphertext" (v2) and "encrypted" (v1) field names
    #[serde(alias = "encrypted")] // v1 compatibility
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,

    /// Family ID used for encryption
    family_id: String,
}

/// Security-provider decryption request
#[derive(Debug, Clone, Serialize)]
struct SecurityProviderDecryptRequest {
    /// Ciphertext to decrypt (base64 encoded automatically)
    #[serde(with = "base64_serde")]
    ciphertext: Vec<u8>,

    /// Family ID for decryption (required by the provider)
    #[serde(skip_serializing_if = "Option::is_none")]
    family_id: Option<String>,
}

/// Security-provider decryption response
#[derive(Debug, Clone, Deserialize)]
struct SecurityProviderDecryptResponse {
    /// Decrypted plaintext (deserialized from base64 automatically)
    #[serde(with = "base64_serde")]
    plaintext: Vec<u8>,

    /// Family ID that encrypted this
    family_id: String,

    /// Whether we can decrypt (same family)
    success: bool,
}

/// Connection type for `security provider` (Unix socket or TCP)
#[derive(Debug, Clone)]
#[allow(
    dead_code,
    reason = "constructed by connect() and pattern-matched in route(); callers wired incrementally"
)]
enum SecurityConnection {
    /// Unix socket path
    Unix(PathBuf),
    /// TCP address (host, port)
    Tcp(String, u16),
}

/// Security-provider `BirdSong` encryption
///
/// Connects to the provider's JSON-RPC interface to provide family-based encryption
/// for discovery packets. Only peers from the same genetic family can
/// decrypt each other's packets.
///
/// **Pure Rust Implementation**: Supports both Unix sockets and TCP connections
/// for cross-platform compatibility (Android uses TCP due to Unix socket restrictions).
///
/// ## Connection Formats
///
/// - Unix socket: `/path/to/security.sock` (or legacy provider-specific path)
/// - TCP socket: `tcp:host:port` (e.g., `tcp:127.0.0.1:9900`)
pub struct SecurityBirdSongProvider {
    /// Connection type (Unix socket or TCP)
    #[allow(
        dead_code,
        reason = "stored for reconnection logic; read when connection routing is wired"
    )]
    connection: SecurityConnection,

    /// Legacy socket path field name (for backward compatibility)
    #[allow(
        dead_code,
        reason = "kept for backward-compat introspection; superseded by `connection`"
    )]
    socket_path: PathBuf,

    /// JSON-RPC client for provider communication (Pure Rust!)
    /// For Unix sockets only - TCP uses direct connection per request
    client: Option<UnixRpcClient>,

    /// TCP endpoint for direct connections (Android support)
    tcp_endpoint: Option<(String, u16)>,

    /// Our family ID (cached from identity query)
    family_id: Option<String>,

    /// Provider availability
    available: bool,
}

impl SecurityBirdSongProvider {
    /// Create new security-provider `BirdSong` adapter (async factory method)
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Crypto provider socket path. Supports:
    ///   - Unix socket: `/tmp/security.sock` (or capability-discovered path)
    ///   - TCP socket: `tcp:host:port` (e.g., `tcp:127.0.0.1:9900`)
    /// * `family_id` - Optional family ID (will query the provider if not provided)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use songbird_discovery::security_birdsong_provider::SecurityBirdSongProvider;
    ///
    /// # async fn example() {
    /// // Unix socket
    /// let provider = SecurityBirdSongProvider::new(
    ///     "/tmp/security.sock",
    ///     Some("ecoPrimals-family-123".to_string())
    /// ).await.unwrap();
    ///
    /// // TCP socket (Android)
    /// let provider = SecurityBirdSongProvider::new(
    ///     "tcp:127.0.0.1:9900",
    ///     Some("ecoPrimals-family-123".to_string())
    /// ).await.unwrap();
    /// # }
    /// ```
    pub async fn new(socket_path: impl Into<PathBuf>, family_id: Option<String>) -> Result<Self> {
        let socket_path = socket_path.into();
        let path_str = socket_path.to_string_lossy().to_string();

        // Check if this is a TCP connection (tcp:host:port format)
        if let Some(addr) = path_str.strip_prefix("tcp:") {
            // Remove "tcp:" prefix
            let parts: Vec<&str> = addr.rsplitn(2, ':').collect();

            if parts.len() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid TCP address format: {path_str}. Expected tcp:host:port"
                ));
            }

            let port: u16 = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid port in TCP address: {}", parts[0]))?;
            let host = parts[1].to_string();

            info!("🎵 Security-provider BirdSong adapter created (TCP: {}:{})", host, port);
            if let Some(ref fam) = family_id {
                info!("   Family ID: {}", fam);
            }

            return Ok(Self {
                connection: SecurityConnection::Tcp(host.clone(), port),
                socket_path,
                client: None,
                tcp_endpoint: Some((host, port)),
                family_id,
                available: true,
            });
        }

        // Unix socket connection
        let client = UnixRpcClient::new(&socket_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to connect to security provider at {}: {e}",
                socket_path.display()
            )
        })?;

        info!("🎵 Security-provider BirdSong adapter created (Pure Rust Unix socket!)");
        info!("   Socket: {:?}", socket_path);
        if let Some(ref fam) = family_id {
            info!("   Family ID: {}", fam);
        }

        Ok(Self {
            connection: SecurityConnection::Unix(socket_path.clone()),
            socket_path,
            client: Some(client),
            tcp_endpoint: None,
            family_id,
            available: true,
        })
    }

    /// Make a JSON-RPC call via TCP
    ///
    /// Used for Android deployments where Unix sockets are restricted.
    async fn tcp_call<P, R>(&self, method: &str, params: P) -> Result<R, String>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let (host, port) =
            self.tcp_endpoint.as_ref().ok_or_else(|| "TCP endpoint not configured".to_string())?;

        trace!("TCP JSON-RPC call to {}:{} method: {}", host, port, method);

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;

        // Connect via TCP
        let addr = format!("{host}:{port}");
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("Failed to connect to security provider at {addr}: {e}"))?;

        // Send request
        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| format!("Failed to write request: {e}"))?;
        stream.write_all(b"\n").await.map_err(|e| format!("Failed to write delimiter: {e}"))?;
        stream.shutdown().await.map_err(|e| format!("Failed to shutdown write: {e}"))?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| format!("Failed to read response: {e}"))?;

        trace!("TCP received {} bytes", response_bytes.len());

        // Parse JSON-RPC response
        #[derive(Deserialize)]
        struct JsonRpcResponse<T> {
            result: Option<T>,
            error: Option<JsonRpcError>,
        }

        #[derive(Deserialize)]
        struct JsonRpcError {
            code: i32,
            message: String,
        }

        let response: JsonRpcResponse<R> = serde_json::from_slice(&response_bytes)
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        if let Some(error) = response.error {
            return Err(format!("JSON-RPC error {}: {}", error.code, error.message));
        }

        response.result.ok_or_else(|| "Missing result in response".to_string())
    }

    /// Async health check for security-provider availability
    ///
    /// Uses JSON-RPC `health` method for pure Rust health checking.
    pub async fn check_health(&self) -> bool {
        #[derive(Deserialize)]
        struct HealthResponse {
            status: String,
        }

        let result: Result<HealthResponse, _> = if self.tcp_endpoint.is_some() {
            self.tcp_call("health", json!({})).await.map_err(|e| anyhow::anyhow!(e))
        } else if let Some(ref client) = self.client {
            client.call_no_params::<HealthResponse>("health").await
        } else {
            return false;
        };

        match result {
            Ok(response) => {
                let is_ok = response.status == "healthy";
                if is_ok {
                    info!("✅ Security provider health check passed (Pure Rust RPC!)");
                } else {
                    warn!(
                        "⚠️  Security provider health check failed: status = {}",
                        response.status
                    );
                }
                is_ok
            }
            Err(e) => {
                warn!("⚠️  Security provider health check error: {}", e);
                false
            }
        }
    }

    /// Encrypt data using the security provider's family encryption (Pure Rust JSON-RPC!)
    ///
    /// Uses `birdsong.encrypt` JSON-RPC method for inter-primal communication.
    /// Supports both Unix socket and TCP connections.
    async fn encrypt_internal(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let request = SecurityProviderEncryptRequest {
            plaintext: plaintext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("🔒 Security-provider encryption via JSON-RPC (Pure Rust!)");
        debug!("   Plaintext size: {} bytes", plaintext.len());
        debug!("   Family ID: {:?}", self.family_id);

        // Call birdsong.encrypt JSON-RPC method (TCP or Unix)
        let encrypt_response: SecurityProviderEncryptResponse = if self.tcp_endpoint.is_some() {
            self.tcp_call("birdsong.encrypt", &request).await?
        } else if let Some(ref client) = self.client {
            client
                .call("birdsong.encrypt", &request)
                .await
                .map_err(|e| format!("Security provider JSON-RPC encrypt failed: {e}"))?
        } else {
            return Err("No security provider connection available".to_string());
        };

        debug!(
            "🔒 Security provider encrypted {} -> {} bytes (family: {})",
            plaintext.len(),
            encrypt_response.ciphertext.len(),
            encrypt_response.family_id
        );

        Ok(encrypt_response.ciphertext)
    }

    /// Decrypt data using the security provider's family decryption (Pure Rust JSON-RPC!)
    ///
    /// Uses `birdsong.decrypt` JSON-RPC method for inter-primal communication.
    /// Supports both Unix socket and TCP connections.
    async fn decrypt_internal(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let request = SecurityProviderDecryptRequest {
            ciphertext: ciphertext.to_vec(),
            family_id: self.family_id.clone(),
        };

        debug!("🔓 Security-provider decryption via JSON-RPC (Pure Rust!)");
        debug!("   Ciphertext size: {} bytes", ciphertext.len());

        // Call birdsong.decrypt JSON-RPC method (TCP or Unix)
        let decrypt_response: SecurityProviderDecryptResponse = if self.tcp_endpoint.is_some() {
            match self.tcp_call("birdsong.decrypt", &request).await {
                Ok(r) => r,
                Err(e) => {
                    debug!(
                        "🔇 Security provider decrypt RPC error (likely different family): {}",
                        e
                    );
                    return Err(format!("Security provider JSON-RPC decrypt failed: {e}"));
                }
            }
        } else if let Some(ref client) = self.client {
            client.call("birdsong.decrypt", &request).await.map_err(|e| {
                // Different family might return an RPC error - treat as noise
                debug!("🔇 Security provider decrypt RPC error (likely different family): {}", e);
                format!("Security provider JSON-RPC decrypt failed: {e}")
            })?
        } else {
            return Err("No security provider connection available".to_string());
        };

        if !decrypt_response.success {
            // Different family - return None (noise)
            debug!("🔇 Security provider noise: different family ({})", decrypt_response.family_id);
            return Ok(None);
        }

        debug!(
            "🔓 Security provider decrypted {} -> {} bytes (family: {})",
            ciphertext.len(),
            decrypt_response.plaintext.len(),
            decrypt_response.family_id
        );

        Ok(Some(decrypt_response.plaintext))
    }
}

#[async_trait]
impl BirdSongEncryption for SecurityBirdSongProvider {
    fn provider_name(&self) -> String {
        "Security provider".to_string()
    }

    fn family_id(&self) -> Option<String> {
        self.family_id.clone()
    }

    fn is_available(&self) -> bool {
        self.available
    }

    async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }

        self.encrypt_internal(plaintext).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn decrypt_discovery(&self, ciphertext: &[u8]) -> Result<Option<Vec<u8>>, anyhow::Error> {
        if !self.available {
            return Err(anyhow::anyhow!("Security provider not available"));
        }

        self.decrypt_internal(ciphertext).await.map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(test)]
#[path = "security_birdsong_provider_tests.rs"]
mod tests;
