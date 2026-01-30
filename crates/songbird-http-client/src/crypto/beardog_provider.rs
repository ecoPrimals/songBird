//! BearDog Provider for CryptoCapability
//!
//! Implements `CryptoCapability` trait using BearDog via JSON-RPC 2.0
//! over Unix sockets.
//!
//! ## Socket Discovery
//!
//! **XDG-Compliant | Zero Hardcoding | Runtime Discovery**
//!
//! Uses intelligent socket discovery via `socket_discovery` module:
//! 1. Environment variables (e.g., `$BEARDOG_SOCKET`)
//! 2. XDG Runtime Dir (`/run/user/$UID/biomeos/`)
//! 3. Legacy `/tmp` paths (fallback)
//!
//! This enables automated Tower Atomic deployment via biomeOS Neural API.

use async_trait::async_trait;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// Platform-agnostic IPC transport
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
use tracing::{debug, info, trace, warn};

use super::capability::{CryptoCapability, TlsApplicationSecrets, TlsHandshakeSecrets};
use super::socket_discovery;
use crate::error::{Error, Result};

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
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

/// Routing mode for BearDog provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoutingMode {
    /// Direct RPC to BearDog (testing)
    Direct,
    /// Route through Neural API with capability.call (production)
    NeuralApi,
}

/// BearDog implementation of CryptoCapability
///
/// Communicates with BearDog via JSON-RPC 2.0 over Unix sockets.
/// All cryptographic operations are delegated to BearDog.
///
/// Supports dual-mode routing:
/// - Direct mode: Direct RPC calls to BearDog
/// - Neural API mode: Uses capability.call for semantic routing
#[derive(Debug)]
pub struct BearDogProvider {
    socket_path: String,
    request_id: AtomicU64,
    mode: RoutingMode,
}

impl BearDogProvider {
    /// Create new BearDog provider with explicit socket path (Direct mode)
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
            mode: RoutingMode::Direct,
        }
    }

    /// Create from environment (supports both Direct and Neural API modes)
    ///
    /// **XDG-Compliant Socket Discovery** (Jan 28, 2026):
    /// - Priority 1: Environment variables (`$BEARDOG_SOCKET`, `$NEURAL_API_SOCKET`)
    /// - Priority 2: XDG Runtime Dir (`/run/user/$UID/biomeos/beardog-$FAMILY_ID.sock`)
    /// - Priority 3: Legacy `/tmp` paths (fallback with warning)
    ///
    /// Modes (via `$BEARDOG_MODE`):
    /// - "neural" (default): Connects to Neural API for capability.call routing
    /// - "direct": Connects directly to BearDog (testing only)
    ///
    /// This enables automated Tower Atomic deployment via biomeOS Neural API
    /// while maintaining backward compatibility.
    pub fn from_env() -> Self {
        let mode = std::env::var("BEARDOG_MODE").unwrap_or_else(|_| "neural".to_string());

        match mode.as_str() {
            "direct" => {
                // Direct mode: Discover BearDog socket with XDG fallback
                let socket = socket_discovery::discover_beardog_socket();
                info!("🔧 BearDog provider: DIRECT mode → {}", socket);
                Self {
                    socket_path: socket,
                    request_id: AtomicU64::new(1),
                    mode: RoutingMode::Direct,
                }
            }
            _ => {
                // Neural API mode (TRUE PRIMAL pattern): Discover Neural API socket
                let socket = socket_discovery::discover_neural_api_socket();
                info!("🌐 BearDog provider: NEURAL API mode (capability.call) → {}", socket);
                Self {
                    socket_path: socket,
                    request_id: AtomicU64::new(1),
                    mode: RoutingMode::NeuralApi,
                }
            }
        }
    }

    /// Get the socket path (for diagnostics/debugging)
    #[allow(dead_code)]
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Platform-agnostic connection helper
    ///
    /// - Unix/macOS/Android: Unix domain sockets
    /// - Windows: TCP localhost
    #[cfg(unix)]
    async fn connect_platform_static(path: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(path).await
    }

    #[cfg(windows)]
    async fn connect_platform_static(address: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(address).await
    }

    #[cfg(not(any(unix, windows)))]
    async fn connect_platform_static(address: &str) -> std::io::Result<tokio::net::TcpStream> {
        tokio::net::TcpStream::connect(address).await
    }

    /// Make JSON-RPC call to BearDog (or via Neural API)
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = match self.mode {
            RoutingMode::Direct => {
                // Direct mode: Map semantic names to BearDog's actual method names
                let actual_method = self.semantic_to_actual(method);
                JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: actual_method.to_string(),
                    params,
                    id,
                }
            }
            RoutingMode::NeuralApi => {
                // Neural API mode: Use capability.call for semantic routing
                let (capability, operation) = self.method_to_capability(method);

                trace!("🌐 Neural API: capability.call({}, {})", capability, operation);

                JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: "capability.call".to_string(),
                    params: json!({
                        "capability": capability,
                        "operation": operation,
                        "args": params
                    }),
                    id,
                }
            }
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| Error::BearDogRpc(format!("Failed to serialize request: {}", e)))?;

        trace!(
            "BearDog RPC request ({}): {}",
            if self.mode == RoutingMode::NeuralApi {
                "Neural API"
            } else {
                "Direct"
            },
            request_json
        );

        // Connect to BearDog (platform-agnostic)
        let mut stream = Self::connect_platform_static(&self.socket_path).await.map_err(|e| {
            Error::BearDogRpc(format!(
                "Failed to connect to BearDog at {}: {}",
                self.socket_path, e
            ))
        })?;

        // Send request
        stream
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| Error::BearDogRpc(format!("Failed to send request: {}", e)))?;
        stream
            .shutdown()
            .await
            .map_err(|e| Error::BearDogRpc(format!("Failed to shutdown write: {}", e)))?;

        // Read response
        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| Error::BearDogRpc(format!("Failed to read response: {}", e)))?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        trace!("BearDog RPC response: {}", response_str);

        let response: JsonRpcResponse = serde_json::from_slice(&response_bytes).map_err(|e| {
            Error::BearDogRpc(format!("Failed to parse response: {} (raw: {})", e, response_str))
        })?;

        // Handle errors
        if let Some(err) = response.error {
            return Err(Error::BearDogRpc(format!(
                "BearDog error: {} (code: {})",
                err.message, err.code
            )));
        }

        response.result.ok_or_else(|| Error::BearDogRpc("BearDog returned null result".to_string()))
    }

    /// Map semantic method names to (capability, operation) pairs for Neural API
    ///
    /// This mapping enables TRUE PRIMAL loose coupling via capability.call.
    /// Neural API will translate these semantic names to actual BearDog methods
    /// using the graph in tower_atomic_bootstrap.toml.
    fn method_to_capability(&self, method: &str) -> (&'static str, &'static str) {
        match method {
            // Key exchange
            "crypto.generate_keypair" => ("crypto", "generate_keypair"),
            "crypto.ecdh_derive" => ("crypto", "derive_secret"),

            // AEAD Encryption - keep specific algorithm for Neural API semantic translation
            "crypto.encrypt_aes_128_gcm" => ("crypto", "encrypt_aes_128_gcm"),
            "crypto.encrypt_aes_256_gcm" => ("crypto", "encrypt_aes_256_gcm"),
            "crypto.encrypt_chacha20_poly1305" => ("crypto", "encrypt_chacha20_poly1305"),

            // AEAD Decryption - keep specific algorithm for Neural API semantic translation
            "crypto.decrypt_aes_128_gcm" => ("crypto", "decrypt_aes_128_gcm"),
            "crypto.decrypt_aes_256_gcm" => ("crypto", "decrypt_aes_256_gcm"),
            "crypto.decrypt_chacha20_poly1305" => ("crypto", "decrypt_chacha20_poly1305"),

            // Hashing
            "crypto.sha256" => ("crypto", "sha256"),
            "crypto.sha384" => ("crypto", "sha384"),
            "crypto.hash_for_cipher" => ("crypto", "hash_for_cipher"),

            // HKDF
            "crypto.hkdf_extract" => ("crypto", "hkdf_extract"),
            "crypto.hkdf_expand" => ("crypto", "hkdf_expand"),

            // TLS specific - keep full method names for correct routing
            "tls.derive_handshake_secrets" => ("tls_crypto", "derive_handshake_secrets"),
            "tls.derive_application_secrets" => ("tls_crypto", "derive_application_secrets"),
            "tls.compute_finished_verify_data" => ("tls_crypto", "compute_finished_verify_data"),

            // Default: unknown operation
            _ => {
                warn!("Unknown method for capability mapping: {}, using generic operation", method);
                ("crypto", "unknown")
            }
        }
    }

    /// Map semantic method names to BearDog's actual method names (Direct mode only)
    fn semantic_to_actual<'a>(&self, method: &'a str) -> &'a str {
        match method {
            // Key exchange
            "crypto.generate_keypair" => "x25519_generate_ephemeral",
            "crypto.ecdh_derive" => "x25519_diffie_hellman",

            // AEAD
            "crypto.encrypt_aes_128_gcm" => "crypto.aes128_gcm_encrypt",
            "crypto.decrypt_aes_128_gcm" => "crypto.aes128_gcm_decrypt",
            "crypto.encrypt_aes_256_gcm" => "crypto.aes256_gcm_encrypt",
            "crypto.decrypt_aes_256_gcm" => "crypto.aes256_gcm_decrypt",
            "crypto.encrypt_chacha20_poly1305" => "crypto.chacha20_poly1305_encrypt",
            "crypto.decrypt_chacha20_poly1305" => "crypto.chacha20_poly1305_decrypt",

            // Hashing
            "crypto.sha256" => "crypto.sha256",
            "crypto.sha384" => "crypto.sha384",

            // HKDF
            "crypto.hkdf_extract" => "crypto.hkdf_extract",
            "crypto.hkdf_expand" => "crypto.hkdf_expand",

            // TLS specific
            "tls.derive_handshake_secrets" => "tls.derive_handshake_secrets",
            "tls.derive_application_secrets" => "tls.derive_application_secrets",
            "tls.compute_finished_verify_data" => "tls.compute_finished_verify_data",

            // Default: pass through
            _ => method,
        }
    }
}

#[async_trait]
impl CryptoCapability for BearDogProvider {
    fn name(&self) -> &str {
        "BearDog"
    }

    async fn is_available(&self) -> bool {
        // Try a simple operation
        match self
            .call(
                "crypto.sha256",
                json!({
                    "data": BASE64_STANDARD.encode(b"ping")
                }),
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                warn!("BearDog availability check failed: {}", e);
                false
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // Key Exchange
    // ═══════════════════════════════════════════════════════════════════

    async fn generate_x25519_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let result = self.call("crypto.generate_keypair", json!({})).await?;

        let public_b64 = result
            .get("public_key")
            .or_else(|| result.get("public"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing public_key in response".to_string()))?;

        let private_b64 = result
            .get("secret_key") // BearDog's standard field name
            .or_else(|| result.get("private_key"))
            .or_else(|| result.get("secret"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::BearDogRpc("Missing secret_key/private_key in response".to_string())
            })?;

        let public = BASE64_STANDARD
            .decode(public_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 public key: {}", e)))?;
        let private = BASE64_STANDARD
            .decode(private_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 private key: {}", e)))?;

        debug!(
            "Generated X25519 keypair: {} bytes public, {} bytes private",
            public.len(),
            private.len()
        );

        Ok((public, private))
    }

    async fn derive_x25519_shared_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.ecdh_derive",
                json!({
                    "our_secret": BASE64_STANDARD.encode(our_secret),
                    "their_public": BASE64_STANDARD.encode(their_public)
                }),
            )
            .await?;

        let shared_b64 = result
            .get("shared_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing shared_secret in response".to_string()))?;

        let shared = BASE64_STANDARD
            .decode(shared_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 shared secret: {}", e)))?;

        debug!("Derived shared secret: {} bytes", shared.len());

        Ok(shared)
    }

    // ═══════════════════════════════════════════════════════════════════
    // AEAD Encryption
    // ═══════════════════════════════════════════════════════════════════

    async fn aes128_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.encrypt_aes_128_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_ciphertext(&result)
    }

    async fn aes128_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_aes_128_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_plaintext(&result)
    }

    async fn aes256_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.encrypt_aes_256_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_ciphertext(&result)
    }

    async fn aes256_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_aes_256_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_plaintext(&result)
    }

    async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.encrypt_chacha20_poly1305",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_ciphertext(&result)
    }

    async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_chacha20_poly1305",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        self.extract_plaintext(&result)
    }

    // ═══════════════════════════════════════════════════════════════════
    // Hashing
    // ═══════════════════════════════════════════════════════════════════

    async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha256",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing hash in response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 hash: {}", e)))
    }

    async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha384",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing hash in response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 hash: {}", e)))
    }

    async fn hash_for_cipher(&self, data: &[u8], cipher_suite: u16) -> Result<Vec<u8>> {
        debug!(
            "🔐 hash_for_cipher: cipher_suite=0x{:04x}, data={} bytes",
            cipher_suite,
            data.len()
        );

        let result = self
            .call(
                "crypto.hash_for_cipher",
                json!({
                    "data": BASE64_STANDARD.encode(data),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing hash in response".to_string()))?;

        let hash = BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 hash: {}", e)))?;

        // Log the result for debugging
        let algorithm = result.get("algorithm").and_then(|v| v.as_str()).unwrap_or("unknown");
        debug!("  → algorithm={}, hash_length={} bytes", algorithm, hash.len());

        Ok(hash)
    }

    // ═══════════════════════════════════════════════════════════════════
    // Key Derivation
    // ═══════════════════════════════════════════════════════════════════

    async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_extract",
                json!({
                    "salt": BASE64_STANDARD.encode(salt),
                    "ikm": BASE64_STANDARD.encode(ikm)
                }),
            )
            .await?;

        let prk_b64 = result
            .get("prk")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing prk in response".to_string()))?;

        BASE64_STANDARD
            .decode(prk_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 prk: {}", e)))
    }

    async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_expand",
                json!({
                    "prk": BASE64_STANDARD.encode(prk),
                    "info": BASE64_STANDARD.encode(info),
                    "length": length
                }),
            )
            .await?;

        let okm_b64 = result
            .get("okm")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing okm in response".to_string()))?;

        BASE64_STANDARD
            .decode(okm_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 okm: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════════════
    // TLS 1.3 Specific
    // ═══════════════════════════════════════════════════════════════════

    async fn tls_derive_handshake_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsHandshakeSecrets> {
        info!("🔑 Deriving TLS 1.3 handshake secrets (RFC 8446 Section 7.1)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!("  → transcript_hash: {} bytes", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);

        let result = self
            .call(
                "tls.derive_handshake_secrets",
                json!({
                    "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
                    "client_random": BASE64_STANDARD.encode(client_random),
                    "server_random": BASE64_STANDARD.encode(server_random),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        // BearDog returns field names matching RFC 8446 terminology:
        // - client_handshake_secret (not client_handshake_traffic_secret)
        // - client_write_key (not client_key)
        // - etc.
        Ok(TlsHandshakeSecrets {
            client_handshake_secret: self.extract_b64_field(&result, "client_handshake_secret")?,
            server_handshake_secret: self.extract_b64_field(&result, "server_handshake_secret")?,
            client_write_key: self.extract_b64_field(&result, "client_write_key")?,
            client_write_iv: self.extract_b64_field(&result, "client_write_iv")?,
            server_write_key: self.extract_b64_field(&result, "server_write_key")?,
            server_write_iv: self.extract_b64_field(&result, "server_write_iv")?,
            // BearDog now returns the raw handshake_secret (needed for application secrets derivation)
            // This is the intermediate value in RFC 8446 key schedule, NOT the traffic secrets
            handshake_secret: self.extract_b64_field(&result, "handshake_secret")?,
        })
    }

    async fn tls_derive_application_secrets(
        &self,
        handshake_secret: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsApplicationSecrets> {
        info!("🔑 Deriving TLS 1.3 application secrets (cipher suite: 0x{:04x})", cipher_suite);
        let result = self
            .call(
                "tls.derive_application_secrets",
                json!({
                    "handshake_secret": BASE64_STANDARD.encode(handshake_secret),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        // BearDog returns shorter field names per RFC 8446 terminology:
        // - client_application_secret (not client_application_traffic_secret)
        Ok(TlsApplicationSecrets {
            client_traffic_secret: self.extract_b64_field(&result, "client_application_secret")?,
            server_traffic_secret: self.extract_b64_field(&result, "server_application_secret")?,
            client_write_key: self.extract_b64_field(&result, "client_write_key")?,
            client_write_iv: self.extract_b64_field(&result, "client_write_iv")?,
            server_write_key: self.extract_b64_field(&result, "server_write_key")?,
            server_write_iv: self.extract_b64_field(&result, "server_write_iv")?,
        })
    }

    async fn tls_compute_finished_verify_data(
        &self,
        base_key: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<Vec<u8>> {
        debug!(
            "🔐 tls_compute_finished_verify_data: cipher=0x{:04x}, hash={} bytes",
            cipher_suite,
            transcript_hash.len()
        );

        let result = self
            .call(
                "tls.compute_finished_verify_data",
                json!({
                    "base_key": BASE64_STANDARD.encode(base_key),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        self.extract_b64_field(&result, "verify_data")
    }
}

impl BearDogProvider {
    /// Extract ciphertext from response
    fn extract_ciphertext(&self, result: &Value) -> Result<Vec<u8>> {
        let ct_b64 = result
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing ciphertext in response".to_string()))?;

        BASE64_STANDARD
            .decode(ct_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 ciphertext: {}", e)))
    }

    /// Extract plaintext from response
    fn extract_plaintext(&self, result: &Value) -> Result<Vec<u8>> {
        let pt_b64 = result
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc("Missing plaintext in response".to_string()))?;

        BASE64_STANDARD
            .decode(pt_b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 plaintext: {}", e)))
    }

    /// Extract base64-encoded field from response
    fn extract_b64_field(&self, result: &Value, field: &str) -> Result<Vec<u8>> {
        let b64 = result
            .get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::BearDogRpc(format!("Missing {} in response", field)))?;

        BASE64_STANDARD
            .decode(b64)
            .map_err(|e| Error::BearDogRpc(format!("Invalid base64 {}: {}", field, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = BearDogProvider::new("/tmp/beardog.sock");
        assert_eq!(provider.name(), "BearDog");
        assert_eq!(provider.socket_path(), "/tmp/beardog.sock");
        assert_eq!(provider.mode, RoutingMode::Direct);
    }

    #[test]
    fn test_semantic_mapping() {
        let provider = BearDogProvider::new("/tmp/beardog.sock");

        assert_eq!(
            provider.semantic_to_actual("crypto.generate_keypair"),
            "x25519_generate_ephemeral"
        );
        assert_eq!(provider.semantic_to_actual("crypto.ecdh_derive"), "x25519_diffie_hellman");
    }

    #[test]
    fn test_capability_mapping() {
        let provider = BearDogProvider::new("/tmp/beardog.sock");

        assert_eq!(
            provider.method_to_capability("crypto.generate_keypair"),
            ("crypto", "generate_keypair")
        );
        assert_eq!(
            provider.method_to_capability("crypto.ecdh_derive"),
            ("crypto", "derive_secret")
        );
        assert_eq!(provider.method_to_capability("crypto.sha256"), ("crypto", "sha256"));
        // Fixed: Keep full method names for Neural API semantic translation
        assert_eq!(
            provider.method_to_capability("tls.derive_handshake_secrets"),
            ("tls_crypto", "derive_handshake_secrets")
        );
    }

    #[test]
    fn test_neural_api_mode() {
        std::env::set_var("BEARDOG_MODE", "neural");
        std::env::set_var("NEURAL_API_SOCKET", "/tmp/neural-api.sock");

        let provider = BearDogProvider::from_env();
        assert_eq!(provider.mode, RoutingMode::NeuralApi);
        assert_eq!(provider.socket_path(), "/tmp/neural-api.sock");
    }

    #[test]
    fn test_direct_mode() {
        std::env::set_var("BEARDOG_MODE", "direct");
        std::env::set_var("BEARDOG_SOCKET", "/tmp/beardog.sock");

        let provider = BearDogProvider::from_env();
        assert_eq!(provider.mode, RoutingMode::Direct);
        assert_eq!(provider.socket_path(), "/tmp/beardog.sock");
    }
}
