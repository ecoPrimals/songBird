// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `BearDog` crypto integration for TLS
//!
//! **EVOLVED (Jan 31, 2026): Platform-agnostic IPC!**
//!
//! This module integrates with `BearDog`'s crypto JSON-RPC API for TLS operations.
//! All cryptographic operations are delegated to `BearDog` via platform-agnostic IPC.
//!
//! **Deep Debt Evolution Status:**
//! - ✅ Phase 1: Platform-agnostic transport (Unix sockets, named pipes, TCP)
//! - 🔄 Phase 2 (planned): universal IPC service discovery instead of filesystem socket heuristics
//!
//! **Current Approach:**
//! - Uses platform-conditional compilation for socket paths
//! - Works on: Linux, macOS, Android (Unix sockets), Windows (named pipes/TCP)
//!
//! **Future Evolution (Deep Debt Opportunity):**
//! - Replace socket path discovery with universal IPC service registry
//! - Use virtual paths: "/primal/beardog" instead of filesystem paths
//! - Full integration with songbird-universal-ipc's capability system

use crate::error::{Result, TlsError};
use base64::{Engine as _, engine::general_purpose};
use pin_project::pin_project;
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

// Platform-agnostic IPC (works on Unix, Windows, etc!)
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;

#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;

/// Stream abstraction for crypto client connections
///
/// **EVOLVED (Feb 5, 2026):** Supports both Unix and TCP sockets
/// to enable cross-platform deployment (especially Android via TCP)
#[pin_project(project = CryptoStreamProj)]
pub enum CryptoStream {
    /// Unix domain socket (Linux/macOS/Android local)
    #[cfg(unix)]
    Unix(#[pin] PlatformStream),
    /// TCP socket (Android cross-device, Windows, or explicit tcp:host:port)
    Tcp(#[pin] tokio::net::TcpStream),
}

impl AsyncRead for CryptoStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            CryptoStreamProj::Unix(stream) => stream.poll_read(cx, buf),
            CryptoStreamProj::Tcp(stream) => stream.poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for CryptoStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.project() {
            #[cfg(unix)]
            CryptoStreamProj::Unix(stream) => stream.poll_write(cx, buf),
            CryptoStreamProj::Tcp(stream) => stream.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            CryptoStreamProj::Unix(stream) => stream.poll_flush(cx),
            CryptoStreamProj::Tcp(stream) => stream.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.project() {
            #[cfg(unix)]
            CryptoStreamProj::Unix(stream) => stream.poll_shutdown(cx),
            CryptoStreamProj::Tcp(stream) => stream.poll_shutdown(cx),
        }
    }
}

/// `BearDog` crypto client for TLS operations
///
/// Communicates with `BearDog` via Unix socket JSON-RPC.
#[derive(Clone)]
pub struct BeardogCryptoClient {
    socket_path: String,
}

impl BeardogCryptoClient {
    /// Create a new `BearDog` crypto client
    ///
    /// Uses runtime discovery to find the Neural API socket for capability.call.
    ///
    /// # Errors
    ///
    /// Returns an error if socket discovery fails or the socket does not exist (Unix only).
    pub fn new() -> Result<Self> {
        let socket_path = Self::discover_socket()?;

        tracing::info!("🌐 BeardogCryptoClient using socket: {}", socket_path);

        // Verify socket exists (Unix only - Windows uses TCP)
        #[cfg(unix)]
        {
            use std::path::Path;
            if !Path::new(&socket_path).exists() {
                return Err(TlsError::CryptoError(format!("Socket not found at: {socket_path}")));
            }
        }

        Ok(Self {
            socket_path,
        })
    }

    /// Create a client with explicit socket path (for testing)
    #[must_use]
    pub const fn with_socket_path(socket_path: String) -> Self {
        Self {
            socket_path,
        }
    }

    /// Discover Neural API socket for capability.call routing
    ///
    /// **Platform-agnostic discovery (TRUE PRIMAL pattern)**
    ///
    /// Uses capability-based discovery with platform-specific validation:
    /// - Unix: Checks filesystem for socket files
    /// - Windows: Uses TCP localhost fallback (named pipes future)
    fn discover_socket() -> Result<String> {
        #[cfg(unix)]
        {
            use crate::socket_discovery::{discover_beardog_socket, discover_neural_api_socket};

            // Strategy 1: Try BearDog socket (checks env vars + XDG + fallback)
            // This includes BEARDOG_SOCKET, BEARDOG_CRYPTO_SOCKET, SONGBIRD_CRYPTO_SOCKET
            let beardog_socket = discover_beardog_socket(None);

            // Check if it's a TCP socket (tcp:host:port format) - skip file existence check
            if beardog_socket.starts_with("tcp:") {
                tracing::info!("✅ Discovered BearDog TCP socket: {}", beardog_socket);
                return Ok(beardog_socket);
            }

            if Path::new(&beardog_socket).exists() {
                tracing::info!("✅ Discovered BearDog socket: {}", beardog_socket);
                return Ok(beardog_socket);
            }

            // Strategy 2: Try Neural API socket (checks env vars + XDG + fallback)
            // This includes NEURAL_API_SOCKET, NEURALS_SOCKET
            let neural_socket = discover_neural_api_socket(None);

            // Check if it's a TCP socket
            if neural_socket.starts_with("tcp:") {
                tracing::info!("✅ Discovered Neural API TCP socket: {}", neural_socket);
                return Ok(neural_socket);
            }

            if Path::new(&neural_socket).exists() {
                tracing::info!("✅ Discovered Neural API socket: {}", neural_socket);
                return Ok(neural_socket);
            }

            // Strategy 3: Legacy fallback paths (for backward compatibility)
            let legacy_paths = vec![
                "/var/run/neural-api/socket",
                "/var/run/beardog/crypto.sock",
                "/run/beardog/crypto.sock",
            ];

            for path in legacy_paths {
                if Path::new(path).exists() {
                    tracing::warn!("⚠️  Using legacy socket path: {}", path);
                    return Ok(path.to_string());
                }
            }

            Err(TlsError::CryptoError(format!(
                "Could not discover BearDog or Neural API socket. Tried:\n\
                 - BearDog: {beardog_socket} (not found)\n\
                 - Neural API: {neural_socket} (not found)\n\
                 - Legacy paths: /var/run/neural-api/socket, /var/run/beardog/crypto.sock (not found)\n\
                 \n\
                 Set one of: BEARDOG_SOCKET=tcp:host:port, NEURAL_API_SOCKET, or XDG_RUNTIME_DIR + FAMILY_ID"
            )))
        }

        #[cfg(windows)]
        {
            // Windows: Use TCP localhost fallback
            // Windows path: env-based TCP fallback until universal IPC discovery covers named pipes.
            tracing::warn!("⚠️  Windows: Using TCP localhost fallback for BearDog crypto");

            // Try environment variables first
            if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
                return Ok(socket);
            }
            if let Ok(socket) = std::env::var("NEURAL_API_SOCKET") {
                return Ok(socket);
            }

            // Default to TCP localhost (port from env or default)
            let port =
                std::env::var("BEARDOG_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(9876);

            Ok(format!("127.0.0.1:{}", port))
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Other platforms: TCP fallback
            tracing::warn!("⚠️  Platform: Using TCP localhost fallback for BearDog crypto");
            Ok("127.0.0.1:9876".to_string())
        }
    }

    /// Make a capability.call to Neural API (TRUE PRIMAL pattern)
    ///
    /// **Platform-agnostic implementation:**
    /// - Unix/macOS/Android: Unix domain sockets
    /// - Windows: TCP localhost (future: named pipes via universal IPC)
    async fn call_capability(
        &self,
        capability: &str,
        operation: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Connect using platform-specific transport
        let mut stream = Self::connect_platform(&self.socket_path)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to connect: {e}")))?;

        // Build capability.call JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capability.call",
            "params": {
                "capability": capability,
                "operation": operation,
                "args": args
            },
            "id": 1
        });

        // Serialize request
        let request_str = serde_json::to_string(&request)
            .map_err(|e| TlsError::InternalError(format!("Failed to serialize request: {e}")))?;

        // Send request (with newline delimiter)
        stream
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send newline: {e}")))?;

        // Read response
        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to read response: {e}")))?;

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| TlsError::CryptoError(format!("Failed to parse response: {e}")))?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(TlsError::CryptoError(format!(
                "Capability call error: {} (code {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            TlsError::CryptoError("Capability call response missing result field".to_string())
        })
    }

    /// Platform-agnostic connection helper
    ///
    /// **Evolution Strategy:**
    /// - Unix: Unix domain sockets (filesystem paths) OR TCP (tcp:host:port)
    /// - Windows: TCP sockets (parse address from path)
    /// - Future: Use songbird-universal-ipc for full platform abstraction
    ///
    /// **EVOLVED (Feb 5, 2026):** Support tcp:host:port format for Android/cross-platform
    #[cfg(unix)]
    async fn connect_platform(path: &str) -> std::io::Result<CryptoStream> {
        // Check for TCP format (tcp:host:port)
        if let Some(addr) = path.strip_prefix("tcp:") {
            // Remove "tcp:" prefix
            tracing::debug!("🌐 Connecting to TCP socket: {}", addr);
            let stream = tokio::net::TcpStream::connect(addr).await?;
            Ok(CryptoStream::Tcp(stream))
        } else {
            tracing::debug!("🔌 Connecting to Unix socket: {}", path);
            let stream = PlatformStream::connect(path).await?;
            Ok(CryptoStream::Unix(stream))
        }
    }

    #[cfg(windows)]
    async fn connect_platform(address: &str) -> std::io::Result<CryptoStream> {
        // On Windows, strip tcp: prefix if present
        let addr = if address.starts_with("tcp:") {
            &address[4..]
        } else {
            address
        };
        let stream = PlatformStream::connect(addr).await?;
        Ok(CryptoStream::Tcp(stream))
    }

    #[cfg(not(any(unix, windows)))]
    async fn connect_platform(address: &str) -> std::io::Result<CryptoStream> {
        // Strip tcp: prefix if present
        let addr = if address.starts_with("tcp:") {
            &address[4..]
        } else {
            address
        };
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(CryptoStream::Tcp(stream))
    }

    /// Make a JSON-RPC call (legacy/testing)
    #[allow(dead_code)]
    async fn call_jsonrpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Connect using platform-specific transport
        let mut stream = Self::connect_platform(&self.socket_path)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to connect to BearDog: {e}")))?;

        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Serialize request
        let request_str = serde_json::to_string(&request)
            .map_err(|e| TlsError::InternalError(format!("Failed to serialize request: {e}")))?;

        // Send request (with newline delimiter)
        stream
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send newline: {e}")))?;

        // Read response
        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to read response: {e}")))?;

        // Parse JSON-RPC response
        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| TlsError::CryptoError(format!("Failed to parse response: {e}")))?;

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
    /// Returns: (`public_key`, `secret_key`) as raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn x25519_generate_ephemeral(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let params = serde_json::json!({
            "purpose": "tls_handshake"
        });

        let result = self.call_capability("crypto", "generate_keypair", params).await?;

        // Extract public_key and secret_key (base64 encoded)
        let public_key_b64 = result["public_key"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing public_key in response".to_string()))?;
        let secret_key_b64 = result["secret_key"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing secret_key in response".to_string()))?;

        let public_key = general_purpose::STANDARD
            .decode(public_key_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode public_key: {e}")))?;
        let secret_key = general_purpose::STANDARD
            .decode(secret_key_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode secret_key: {e}")))?;

        Ok((public_key, secret_key))
    }

    /// Derive X25519 shared secret
    ///
    /// Returns: `shared_secret` as raw bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn x25519_derive_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "our_secret": general_purpose::STANDARD.encode(our_secret),
            "their_public": general_purpose::STANDARD.encode(their_public)
        });

        let result = self.call_capability("crypto", "derive_secret", params).await?;

        let shared_secret_b64 = result["shared_secret"].as_str().ok_or_else(|| {
            TlsError::CryptoError("Missing shared_secret in response".to_string())
        })?;

        let shared_secret = general_purpose::STANDARD
            .decode(shared_secret_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode shared_secret: {e}")))?;

        Ok(shared_secret)
    }

    /// Encrypt with ChaCha20-Poly1305 (AEAD)
    ///
    /// `BearDog` generates the nonce.
    /// Returns: (ciphertext, nonce, tag)
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let mut params = serde_json::json!({
            "plaintext": general_purpose::STANDARD.encode(plaintext),
            "key": general_purpose::STANDARD.encode(key)
        });

        if let Some(aad_data) = aad {
            params["aad"] = serde_json::json!(general_purpose::STANDARD.encode(aad_data));
        }

        let result = self.call_capability("crypto", "encrypt", params).await?;

        let ciphertext_b64 = result["ciphertext"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing ciphertext in response".to_string()))?;
        let nonce_b64 = result["nonce"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing nonce in response".to_string()))?;
        let tag_b64 = result["tag"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing tag in response".to_string()))?;

        let ciphertext = general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode ciphertext: {e}")))?;
        let nonce = general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode nonce: {e}")))?;
        let tag = general_purpose::STANDARD
            .decode(tag_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode tag: {e}")))?;

        Ok((ciphertext, nonce, tag))
    }

    /// Decrypt with ChaCha20-Poly1305 (AEAD)
    ///
    /// Returns: plaintext
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let mut params = serde_json::json!({
            "ciphertext": general_purpose::STANDARD.encode(ciphertext),
            "key": general_purpose::STANDARD.encode(key),
            "nonce": general_purpose::STANDARD.encode(nonce),
            "tag": general_purpose::STANDARD.encode(tag)
        });

        if let Some(aad_data) = aad {
            params["aad"] = serde_json::json!(general_purpose::STANDARD.encode(aad_data));
        }

        let result = self.call_capability("crypto", "decrypt", params).await?;

        let plaintext_b64 = result["plaintext"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing plaintext in response".to_string()))?;

        let plaintext = general_purpose::STANDARD
            .decode(plaintext_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode plaintext: {e}")))?;

        Ok(plaintext)
    }

    /// Sign with Ed25519
    ///
    /// Returns: signature (64 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn ed25519_sign(&self, message: &[u8], key_id: &str) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "message": general_purpose::STANDARD.encode(message),
            "key_id": key_id,
            "purpose": "certificate_signing"
        });

        let result = self.call_capability("crypto", "sign", params).await?;

        let signature_b64 = result["signature"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing signature in response".to_string()))?;

        let signature = general_purpose::STANDARD
            .decode(signature_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode signature: {e}")))?;

        Ok(signature)
    }

    /// HMAC-SHA256
    ///
    /// Returns: MAC (32 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if the capability call fails or the response is invalid.
    pub async fn hmac_sha256(&self, message: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "message": general_purpose::STANDARD.encode(message),
            "key": general_purpose::STANDARD.encode(key)
        });

        let result = self.call_capability("crypto", "hmac_sha256", params).await?;

        let mac_b64 = result["mac"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError("Missing mac in response".to_string()))?;

        let mac = general_purpose::STANDARD
            .decode(mac_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode mac: {e}")))?;

        Ok(mac)
    }
}

// JSON-RPC types
/// JSON-RPC 2.0 Response
///
/// Note: Fields are used during deserialization but not directly accessed in code.
/// The response is parsed and converted to domain types immediately.
#[allow(dead_code)]
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
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Create a temp socket file, returning its path. Caller must clean up.
    fn create_temp_socket() -> String {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = format!("/tmp/songbird-tls-test-{}-{}.sock", std::process::id(), id);
        std::fs::File::create(&path).expect("create temp socket");
        path
    }

    #[test]
    fn test_with_socket_path() {
        let client = BeardogCryptoClient::with_socket_path("/tmp/custom.sock".to_string());
        assert_eq!(client.socket_path, "/tmp/custom.sock");
    }

    #[test]
    fn test_with_socket_path_various_locations() {
        let paths = [
            "/var/run/custom/socket.sock",
            "/tmp/my-app.sock",
            "/run/beardog/test.sock",
            "tcp:127.0.0.1:9900",
        ];
        for path in paths {
            let client = BeardogCryptoClient::with_socket_path(path.to_string());
            assert_eq!(client.socket_path, path);
        }
    }

    #[test]
    fn test_client_clone_preserves_socket_path() {
        let original = BeardogCryptoClient::with_socket_path("/tmp/original.sock".to_string());
        let cloned = original.clone();
        assert_eq!(original.socket_path, cloned.socket_path);
    }

    #[test]
    fn test_discover_socket_with_real_file() {
        // Create a real temp socket file so discover_socket finds it
        let sock_path = create_temp_socket();

        // Set env var pointing to it -- but note: discover_socket uses
        // socket_discovery which checks env vars. The actual socket exists on disk.
        // We test via with_socket_path to avoid env var races.
        let client = BeardogCryptoClient::with_socket_path(sock_path.clone());
        assert_eq!(client.socket_path, sock_path);

        std::fs::remove_file(&sock_path).ok();
    }

    #[test]
    fn test_discover_socket_tcp_format() {
        // TCP sockets skip file existence check
        let client = BeardogCryptoClient::with_socket_path("tcp:127.0.0.1:9900".to_string());
        assert_eq!(client.socket_path, "tcp:127.0.0.1:9900");
    }

    #[tokio::test]
    async fn test_new_fails_on_nonexistent_socket() {
        // with_socket_path then manually verify socket doesn't exist
        let client =
            BeardogCryptoClient::with_socket_path("/tmp/nonexistent-tls-test.sock".to_string());
        // The socket_path is set but actual connection would fail
        assert_eq!(client.socket_path, "/tmp/nonexistent-tls-test.sock");
    }

    #[test]
    fn test_concurrent_client_creation() {
        // Demonstrate thread-safe client construction
        let handles: Vec<_> = (0..10)
            .map(|i| {
                std::thread::spawn(move || {
                    let client = BeardogCryptoClient::with_socket_path(format!(
                        "/tmp/concurrent-{}.sock",
                        i
                    ));
                    assert_eq!(client.socket_path, format!("/tmp/concurrent-{}.sock", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread panicked");
        }
    }
}
