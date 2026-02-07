//! BearDog Crypto Client - TRUE PRIMAL Crypto Delegation
//!
//! All cryptographic operations are delegated to BearDog via JSON-RPC.
//! This follows the TRUE PRIMAL pattern where primals only have self-knowledge
//! and discover/use other primal capabilities at runtime.
//!
//! ## Usage via biomeOS Neural API
//!
//! ```rust,ignore
//! let client = BeardogCryptoClient::from_env().await?;
//!
//! // Generate Ed25519 identity for .onion address
//! let keypair = client.ed25519_generate_keypair().await?;
//!
//! // Derive .onion address checksum (SHA3-256)
//! let checksum = client.sha3_256(&data).await?;
//!
//! // Session key exchange (X25519)
//! let session_key = client.x25519_derive_secret(&private, &peer_public).await?;
//!
//! // Encrypt data (ChaCha20-Poly1305)
//! let ciphertext = client.chacha20_poly1305_encrypt(&key, &nonce, &plaintext).await?;
//! ```
//!
//! ## Environment Variables
//!
//! - `BEARDOG_SOCKET`: Direct BearDog socket path (or `tcp:host:port` for TCP)
//! - `CRYPTO_PROVIDER_SOCKET`: biomeOS-wired crypto provider
//! - `NEURAL_API_SOCKET`: biomeOS Neural API for capability routing
//!
//! ## TCP Support (Android/Universal)
//!
//! For platforms without Unix sockets (Android), use TCP transport:
//! ```bash
//! export BEARDOG_SOCKET=tcp:127.0.0.1:9900
//! ```

use crate::error::{OnionError, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest<'a, T: Serialize> {
    jsonrpc: &'static str,
    method: &'a str,
    params: T,
    id: u64,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<T>,
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Transport type for BearDog connection
#[derive(Debug, Clone)]
pub enum BeardogTransport {
    /// Unix socket (Linux/macOS) - default on desktop
    #[cfg(unix)]
    Unix(PathBuf),
    /// TCP socket (Android/universal fallback)
    Tcp(String, u16),
}

/// BearDog crypto client for TRUE PRIMAL delegation
///
/// Delegates all crypto operations to BearDog via JSON-RPC.
/// Supports both Unix sockets (desktop) and TCP (Android/universal).
pub struct BeardogCryptoClient {
    transport: BeardogTransport,
    timeout: Duration,
}

impl BeardogCryptoClient {
    /// Parse transport from connection string
    ///
    /// Formats:
    /// - `tcp:host:port` - TCP connection
    /// - `/path/to/socket` - Unix socket (default)
    fn parse_transport(conn_str: &str) -> Result<BeardogTransport> {
        if conn_str.starts_with("tcp:") {
            // TCP format: tcp:host:port
            let parts: Vec<&str> = conn_str.strip_prefix("tcp:").unwrap().split(':').collect();
            if parts.len() != 2 {
                return Err(OnionError::ConfigError(
                    format!("Invalid TCP format: {}. Use tcp:host:port", conn_str)
                ));
            }
            let host = parts[0].to_string();
            let port: u16 = parts[1].parse().map_err(|_| {
                OnionError::ConfigError(format!("Invalid port: {}", parts[1]))
            })?;
            Ok(BeardogTransport::Tcp(host, port))
        } else {
            // Unix socket path
            #[cfg(unix)]
            {
                Ok(BeardogTransport::Unix(PathBuf::from(conn_str)))
            }
            #[cfg(not(unix))]
            {
                Err(OnionError::ConfigError(
                    "Unix sockets not supported on this platform. Use tcp:host:port".into()
                ))
            }
        }
    }

    /// Create client from environment variables
    ///
    /// Resolution order:
    /// 1. `BEARDOG_SOCKET` - Direct BearDog socket (or tcp:host:port)
    /// 2. `CRYPTO_PROVIDER_SOCKET` - biomeOS-wired provider
    /// 3. XDG fallback paths (Unix only)
    ///
    /// TCP format: `tcp:127.0.0.1:9900`
    pub fn from_env() -> Result<Self> {
        // Try direct BearDog socket (may be tcp:host:port or /path/to/socket)
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            let transport = Self::parse_transport(&socket)?;
            return Ok(Self {
                transport,
                timeout: Duration::from_secs(10),
            });
        }

        // Try biomeOS-wired crypto provider
        if let Ok(socket) = std::env::var("CRYPTO_PROVIDER_SOCKET") {
            let transport = Self::parse_transport(&socket)?;
            return Ok(Self {
                transport,
                timeout: Duration::from_secs(10),
            });
        }

        // XDG runtime fallback (Unix only)
        #[cfg(unix)]
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let family_id = std::env::var("FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .unwrap_or_else(|_| "default".to_string());
            let socket_path = format!("{}/biomeos/beardog-{}.sock", xdg_runtime, family_id);
            if std::path::Path::new(&socket_path).exists() {
                return Ok(Self {
                    transport: BeardogTransport::Unix(PathBuf::from(socket_path)),
                    timeout: Duration::from_secs(10),
                });
            }
        }

        Err(OnionError::ConfigError(
            "No BearDog socket found. Set BEARDOG_SOCKET (tcp:host:port or /path/to/socket)".into()
        ))
    }

    /// Create client with explicit Unix socket path
    #[cfg(unix)]
    pub fn with_socket(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            transport: BeardogTransport::Unix(socket_path.into()),
            timeout: Duration::from_secs(10),
        }
    }

    /// Create client with explicit TCP connection
    pub fn with_tcp(host: impl Into<String>, port: u16) -> Self {
        Self {
            transport: BeardogTransport::Tcp(host.into(), port),
            timeout: Duration::from_secs(10),
        }
    }

    /// Set timeout for RPC calls
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Internal: Send JSON-RPC request over the configured transport
    fn call<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method,
            params,
            id: 1,
        };

        let request_bytes = serde_json::to_vec(&request)?;
        
        // Connect and communicate based on transport type
        let response_line = match &self.transport {
            #[cfg(unix)]
            BeardogTransport::Unix(socket_path) => {
                let mut stream = UnixStream::connect(socket_path).map_err(|e| {
                    OnionError::ConnectionError(format!(
                        "Failed to connect to BearDog Unix socket at {:?}: {}",
                        socket_path, e
                    ))
                })?;

                stream
                    .set_read_timeout(Some(self.timeout))
                    .map_err(|e| OnionError::ConnectionError(format!("Failed to set timeout: {}", e)))?;
                stream
                    .set_write_timeout(Some(self.timeout))
                    .map_err(|e| OnionError::ConnectionError(format!("Failed to set timeout: {}", e)))?;

                // Send request
                stream.write_all(&request_bytes)?;
                stream.write_all(b"\n")?;
                stream.flush()?;

                // Read response
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                reader.read_line(&mut line)?;
                line
            }
            BeardogTransport::Tcp(host, port) => {
                let addr = format!("{}:{}", host, port);
                let mut stream = TcpStream::connect(&addr).map_err(|e| {
                    OnionError::ConnectionError(format!(
                        "Failed to connect to BearDog TCP at {}: {}",
                        addr, e
                    ))
                })?;

                stream
                    .set_read_timeout(Some(self.timeout))
                    .map_err(|e| OnionError::ConnectionError(format!("Failed to set timeout: {}", e)))?;
                stream
                    .set_write_timeout(Some(self.timeout))
                    .map_err(|e| OnionError::ConnectionError(format!("Failed to set timeout: {}", e)))?;

                // Send request
                stream.write_all(&request_bytes)?;
                stream.write_all(b"\n")?;
                stream.flush()?;

                // Read response
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                reader.read_line(&mut line)?;
                line
            }
        };

        let response: JsonRpcResponse<R> = serde_json::from_str(&response_line)?;

        if let Some(error) = response.error {
            return Err(OnionError::RpcError(format!(
                "[{}] {}",
                error.code, error.message
            )));
        }

        response
            .result
            .ok_or_else(|| OnionError::RpcError("No result in response".into()))
    }

    // =========================================================================
    // Ed25519 Operations (Identity Keys)
    // =========================================================================

    /// Generate Ed25519 keypair for .onion identity
    ///
    /// Returns (public_key, secret_key) both as 32-byte arrays
    pub fn ed25519_generate_keypair(&self) -> Result<Ed25519Keypair> {
        #[derive(Serialize)]
        struct Params {}

        #[derive(Deserialize)]
        struct Response {
            public_key: String, // base64
            secret_key: String, // base64
        }

        let response: Response = self.call("crypto.ed25519_generate_keypair", Params {})?;

        let public_key = base64_decode(&response.public_key)?;
        let secret_key = base64_decode(&response.secret_key)?;

        Ok(Ed25519Keypair {
            public_key: public_key.try_into().map_err(|_| {
                OnionError::CryptoError("Invalid public key length".into())
            })?,
            secret_key: secret_key.try_into().map_err(|_| {
                OnionError::CryptoError("Invalid secret key length".into())
            })?,
        })
    }

    /// Sign data with Ed25519
    pub fn ed25519_sign(&self, secret_key: &[u8; 32], message: &[u8]) -> Result<[u8; 64]> {
        #[derive(Serialize)]
        struct Params {
            secret_key: String, // base64
            message: String,    // base64
        }

        #[derive(Deserialize)]
        struct Response {
            signature: String, // base64
        }

        let response: Response = self.call(
            "crypto.sign_ed25519",
            Params {
                secret_key: base64_encode(secret_key),
                message: base64_encode(message),
            },
        )?;

        let signature = base64_decode(&response.signature)?;
        signature.try_into().map_err(|_| {
            OnionError::CryptoError("Invalid signature length".into())
        })
    }

    /// Verify Ed25519 signature
    pub fn ed25519_verify(
        &self,
        public_key: &[u8; 32],
        message: &[u8],
        signature: &[u8; 64],
    ) -> Result<bool> {
        #[derive(Serialize)]
        struct Params {
            public_key: String, // base64
            message: String,    // base64
            signature: String,  // base64
        }

        #[derive(Deserialize)]
        struct Response {
            valid: bool,
        }

        let response: Response = self.call(
            "crypto.verify_ed25519",
            Params {
                public_key: base64_encode(public_key),
                message: base64_encode(message),
                signature: base64_encode(signature),
            },
        )?;

        Ok(response.valid)
    }

    // =========================================================================
    // X25519 Operations (Session Keys)
    // =========================================================================

    /// Generate X25519 ephemeral keypair for session key exchange
    pub fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair> {
        #[derive(Serialize)]
        struct Params {}

        #[derive(Deserialize)]
        struct Response {
            public_key: String, // base64
            secret_key: String, // base64
        }

        let response: Response = self.call("crypto.x25519_generate_ephemeral", Params {})?;

        let public_key = base64_decode(&response.public_key)?;
        let secret_key = base64_decode(&response.secret_key)?;

        Ok(X25519Keypair {
            public_key: public_key.try_into().map_err(|_| {
                OnionError::CryptoError("Invalid public key length".into())
            })?,
            secret_key: secret_key.try_into().map_err(|_| {
                OnionError::CryptoError("Invalid secret key length".into())
            })?,
        })
    }

    /// Derive shared secret via X25519 ECDH
    pub fn x25519_derive_secret(
        &self,
        our_secret: &[u8; 32],
        their_public: &[u8; 32],
    ) -> Result<[u8; 32]> {
        #[derive(Serialize)]
        struct Params {
            secret_key: String, // base64
            public_key: String, // base64
        }

        #[derive(Deserialize)]
        struct Response {
            shared_secret: String, // base64
        }

        let response: Response = self.call(
            "crypto.x25519_derive_secret",
            Params {
                secret_key: base64_encode(our_secret),
                public_key: base64_encode(their_public),
            },
        )?;

        let shared = base64_decode(&response.shared_secret)?;
        shared.try_into().map_err(|_| {
            OnionError::CryptoError("Invalid shared secret length".into())
        })
    }

    // =========================================================================
    // ChaCha20-Poly1305 Operations (Encryption)
    // =========================================================================

    /// Encrypt data with ChaCha20-Poly1305
    pub fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct Params {
            key: String,       // base64
            nonce: String,     // base64
            plaintext: String, // base64
        }

        #[derive(Deserialize)]
        struct Response {
            ciphertext: String, // base64
        }

        let response: Response = self.call(
            "crypto.chacha20_poly1305_encrypt",
            Params {
                key: base64_encode(key),
                nonce: base64_encode(nonce),
                plaintext: base64_encode(plaintext),
            },
        )?;

        base64_decode(&response.ciphertext)
    }

    /// Decrypt data with ChaCha20-Poly1305
    pub fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct Params {
            key: String,        // base64
            nonce: String,      // base64
            ciphertext: String, // base64
        }

        #[derive(Deserialize)]
        struct Response {
            plaintext: String, // base64
        }

        let response: Response = self.call(
            "crypto.chacha20_poly1305_decrypt",
            Params {
                key: base64_encode(key),
                nonce: base64_encode(nonce),
                ciphertext: base64_encode(ciphertext),
            },
        )?;

        base64_decode(&response.plaintext)
    }

    // =========================================================================
    // SHA3-256 Operation (.onion address derivation)
    // =========================================================================

    /// Compute SHA3-256 hash (needed for .onion address checksum)
    ///
    /// NOTE: Requires BearDog to implement `crypto.sha3_256` method.
    /// See: BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md
    pub fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]> {
        #[derive(Serialize)]
        struct Params {
            data: String, // base64
        }

        #[derive(Deserialize)]
        struct Response {
            hash_base64: String, // base64 (BearDog field name)
        }

        let response: Response = self.call(
            "crypto.sha3_256",
            Params {
                data: base64_encode(data),
            },
        )?;

        let hash = base64_decode(&response.hash_base64)?;
        hash.try_into().map_err(|_| {
            OnionError::CryptoError("Invalid hash length".into())
        })
    }

    // =========================================================================
    // HMAC-SHA256 Operations (HKDF)
    // =========================================================================

    /// Compute HMAC-SHA256 (for HKDF key derivation)
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]> {
        #[derive(Serialize)]
        struct Params {
            key: String,  // base64
            data: String, // base64
        }

        #[derive(Deserialize)]
        struct Response {
            mac: String, // base64
        }

        let response: Response = self.call(
            "crypto.hmac_sha256",
            Params {
                key: base64_encode(key),
                data: base64_encode(data),
            },
        )?;

        let mac = base64_decode(&response.mac)?;
        mac.try_into().map_err(|_| {
            OnionError::CryptoError("Invalid MAC length".into())
        })
    }
}

// =============================================================================
// Supporting Types
// =============================================================================

/// Ed25519 keypair for identity/signing
#[derive(Debug, Clone)]
pub struct Ed25519Keypair {
    /// Ed25519 public key (32 bytes)
    pub public_key: [u8; 32],
    /// Ed25519 secret key (32 bytes)
    pub secret_key: [u8; 32],
}

/// X25519 keypair for key exchange
#[derive(Debug, Clone)]
pub struct X25519Keypair {
    /// X25519 public key (32 bytes)
    pub public_key: [u8; 32],
    /// X25519 secret key (32 bytes)
    pub secret_key: [u8; 32],
}

// =============================================================================
// Helpers
// =============================================================================

fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD
        .decode(s)
        .map_err(|e| OnionError::CryptoError(format!("Base64 decode error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_from_env_no_socket() {
        // Clear env vars
        std::env::remove_var("BEARDOG_SOCKET");
        std::env::remove_var("CRYPTO_PROVIDER_SOCKET");
        
        // Should fail without socket
        let result = BeardogCryptoClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    #[cfg(unix)]
    fn test_client_with_socket() {
        let client = BeardogCryptoClient::with_socket("/tmp/test-beardog.sock");
        match &client.transport {
            BeardogTransport::Unix(path) => {
                assert_eq!(path.to_str().unwrap(), "/tmp/test-beardog.sock");
            }
            _ => panic!("Expected Unix transport"),
        }
    }

    #[test]
    fn test_client_with_tcp() {
        let client = BeardogCryptoClient::with_tcp("127.0.0.1", 9900);
        match &client.transport {
            BeardogTransport::Tcp(host, port) => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(*port, 9900);
            }
            #[cfg(unix)]
            _ => panic!("Expected TCP transport"),
        }
    }

    #[test]
    fn test_parse_tcp_transport() {
        let transport = BeardogCryptoClient::parse_transport("tcp:127.0.0.1:9900").unwrap();
        match transport {
            BeardogTransport::Tcp(host, port) => {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 9900);
            }
            #[cfg(unix)]
            _ => panic!("Expected TCP transport"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_unix_transport() {
        let transport = BeardogCryptoClient::parse_transport("/tmp/beardog.sock").unwrap();
        match transport {
            BeardogTransport::Unix(path) => {
                assert_eq!(path.to_str().unwrap(), "/tmp/beardog.sock");
            }
            _ => panic!("Expected Unix transport"),
        }
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, BearDog!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}
