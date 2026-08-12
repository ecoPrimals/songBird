// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Security (crypto) provider integration for TLS
//!
//! **EVOLVED (Jan 31, 2026): Platform-agnostic IPC!**
//!
//! This module integrates with the security provider's crypto JSON-RPC API for TLS operations.
//! All cryptographic operations are delegated to the crypto provider via platform-agnostic IPC.
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
//! - Use virtual paths: "/primal/security" instead of filesystem paths
//! - Full integration with songbird-universal-ipc's capability system

use crate::error::{Result, TlsError};
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;
use songbird_types::IpcStream;
#[cfg(unix)]
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Stream abstraction for crypto client connections — backed by [`IpcStream`].
pub type CryptoStream = IpcStream;

/// TLS crypto client backed by the security (crypto) provider
///
/// Communicates with the provider via Unix socket JSON-RPC.
///
/// Supports two modes:
/// - **Neural API** (default): Wraps calls in `capability.call` envelope for biomeOS routing
/// - **Direct**: Calls bearDog semantic methods directly (required when `BEARDOG_MODE=direct`)
#[derive(Clone)]
pub struct SecurityTlsCryptoClient {
    socket_path: String,
    /// When true, uses direct bearDog semantic methods instead of `capability.call`.
    direct_mode: bool,
}

impl SecurityTlsCryptoClient {
    /// Create a new TLS crypto client for the discovered security provider socket
    ///
    /// Auto-detects mode from `SECURITY_PROVIDER_MODE` or `BEARDOG_MODE` env vars.
    /// When set to `"direct"`, calls bearDog semantic methods directly (no `capability.call`).
    ///
    /// # Errors
    ///
    /// Returns an error if socket discovery fails or the socket does not exist (Unix only).
    pub fn new() -> Result<Self> {
        let direct_mode = songbird_process_env::var("SECURITY_PROVIDER_MODE")
            .or_else(|_| {
                songbird_process_env::var("BEARDOG_MODE").inspect(|_| {
                    songbird_types::defaults::legacy_env::warn_if_legacy_primal_env("BEARDOG_MODE");
                })
            })
            .map(|v| v.eq_ignore_ascii_case("direct"))
            .unwrap_or(false);

        let socket_path = Self::discover_socket()?;

        if direct_mode {
            tracing::info!(
                socket = %socket_path,
                "SecurityTlsCryptoClient: DIRECT mode (semantic methods)"
            );
        } else {
            tracing::info!(
                socket = %socket_path,
                "SecurityTlsCryptoClient: Neural API mode (capability.call)"
            );
        }

        // Verify socket exists (Unix only - Windows uses TCP)
        #[cfg(unix)]
        {
            if !Path::new(&socket_path).exists() {
                return Err(TlsError::CryptoError(format!("Socket not found at: {socket_path}")));
            }
        }

        Ok(Self {
            socket_path,
            direct_mode,
        })
    }

    /// Create a client in direct mode targeting bearDog's signing socket.
    ///
    /// Calls semantic methods (`crypto.sign_ed25519`, `crypto.x25519_generate_ephemeral`,
    /// etc.) directly — no `capability.call` wrapping.
    #[must_use]
    pub fn new_direct(socket_path: String) -> Self {
        tracing::info!(
            socket = %socket_path,
            "SecurityTlsCryptoClient: DIRECT mode (explicit)"
        );
        Self {
            socket_path,
            direct_mode: true,
        }
    }

    /// Create a client with explicit socket path (for testing)
    #[must_use]
    pub const fn with_socket_path(socket_path: String) -> Self {
        Self {
            socket_path,
            direct_mode: false,
        }
    }

    #[cfg(unix)]
    fn discover_socket_unix() -> Result<String> {
        use crate::socket_discovery::{discover_neural_api_socket, discover_security_socket};

        let security_socket = discover_security_socket(None);

        if security_socket.starts_with("tcp:") {
            tracing::info!("✅ Discovered security provider TCP socket: {}", security_socket);
            return Ok(security_socket);
        }

        if Path::new(&security_socket).exists() {
            tracing::info!("✅ Discovered security provider socket: {}", security_socket);
            return Ok(security_socket);
        }

        let neural_socket = discover_neural_api_socket(None);

        if neural_socket.starts_with("tcp:") {
            tracing::info!("✅ Discovered Neural API TCP socket: {}", neural_socket);
            return Ok(neural_socket);
        }

        if Path::new(&neural_socket).exists() {
            tracing::info!("✅ Discovered Neural API socket: {}", neural_socket);
            return Ok(neural_socket);
        }

        // Env-first and XDG before fixed legacy paths (last resort warns).
        if let Ok(p) = songbird_process_env::var("SECURITY_PROVIDER_SOCKET")
            && !p.starts_with("tcp:")
            && Path::new(&p).exists()
        {
            tracing::info!("Using SECURITY_PROVIDER_SOCKET path (exists on disk): {}", p);
            return Ok(p);
        }

        if let Ok(xdg) = songbird_process_env::var("XDG_RUNTIME_DIR") {
            let biome_dir = songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR;
            let sec = std::path::PathBuf::from(&xdg).join(biome_dir).join("security.sock");
            if sec.exists() {
                tracing::info!("Using XDG biomeOS security socket: {}", sec.display());
                return Ok(sec.to_string_lossy().into_owned());
            }
            for name in ["neural-api.sock", "ai.sock"] {
                let p = std::path::PathBuf::from(&xdg).join(biome_dir).join(name);
                if p.exists() {
                    tracing::info!("Using XDG biomeOS neural socket: {}", p.display());
                    return Ok(p.to_string_lossy().into_owned());
                }
            }
        }

        if let Ok(p) = songbird_process_env::var("BIOMEOS_SOCKET")
            && !p.starts_with("tcp:")
            && Path::new(&p).exists()
        {
            tracing::info!("Using BIOMEOS_SOCKET: {}", p);
            return Ok(p);
        }

        let sys_runtime = songbird_types::constants::BIOMEOS_SYSTEM_RUNTIME_DIR;
        let legacy_security = format!("{sys_runtime}/security.sock");
        let legacy_run = format!(
            "/run/{}/security.sock",
            songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR
        );
        let legacy_paths: [(&str, &str); 3] = [
            (&legacy_security, "SECURITY_PROVIDER_SOCKET or XDG_RUNTIME_DIR/biomeos/security.sock"),
            (&legacy_run, "SECURITY_PROVIDER_SOCKET or XDG_RUNTIME_DIR/biomeos/security.sock"),
            ("/var/run/neural-api/socket", "BIOMEOS_SOCKET or XDG discovery for neural-api"),
        ];

        for (path, migrate_hint) in legacy_paths {
            if Path::new(path).exists() {
                tracing::warn!("using legacy socket path: {path} ({migrate_hint})");
                return Ok(path.to_string());
            }
        }

        Err(TlsError::CryptoError(format!(
            "Could not discover security provider or Neural API socket. Tried:\n\
             - Security: {security_socket} (not found)\n\
             - Neural API: {neural_socket} (not found)\n\
             - SECURITY_PROVIDER_SOCKET, XDG_RUNTIME_DIR/biomeos/{{security,neural-api,ai}}.sock, BIOMEOS_SOCKET\n\
             - Legacy: /var/run/biomeos/security.sock, /run/biomeos/security.sock, /var/run/neural-api/socket (not found)\n\
             \n\
             Set one of: SECURITY_PROVIDER_SOCKET, NEURAL_API_SOCKET, BIOMEOS_SOCKET, or XDG_RUNTIME_DIR"
        )))
    }

    /// Discover the crypto provider socket.
    ///
    /// In Neural API mode this is the biomeOS neural-api socket;
    /// in direct mode this is bearDog's signing socket.
    ///
    /// **Platform-agnostic discovery (TRUE PRIMAL pattern)**
    ///
    /// - Unix: Checks filesystem for socket files
    /// - Windows: Uses TCP localhost fallback (named pipes future)
    #[allow(
        clippy::unnecessary_wraps,
        reason = "returns Err on unix; consistent signature across platforms"
    )]
    fn discover_socket() -> Result<String> {
        #[cfg(unix)]
        {
            Self::discover_socket_unix()
        }

        #[cfg(windows)]
        {
            tracing::warn!("⚠️  Windows: Using TCP localhost fallback for security provider");

            if let Ok(socket) = songbird_process_env::var("CRYPTO_PROVIDER_SOCKET") {
                return Ok(socket);
            }
            if let Ok(socket) = songbird_process_env::var("SECURITY_PROVIDER_SOCKET") {
                return Ok(socket);
            }
            if let Ok(socket) = songbird_process_env::var("SECURITY_SOCKET") {
                return Ok(socket);
            }
            if let Ok(socket) = songbird_process_env::var("BEARDOG_SOCKET") {
                tracing::warn!(
                    "BEARDOG_SOCKET is deprecated — migrate to SECURITY_PROVIDER_SOCKET, SECURITY_SOCKET, or CRYPTO_PROVIDER_SOCKET; prefer CAPABILITY_SECURITY_ENDPOINT (capability-first)"
                );
                return Ok(socket);
            }
            if let Ok(socket) = songbird_process_env::var("NEURAL_API_SOCKET") {
                return Ok(socket);
            }

            let port = songbird_process_env::var("CRYPTO_PROVIDER_PORT")
                .or_else(|_| songbird_process_env::var("SECURITY_PROVIDER_PORT"))
                .or_else(|_| songbird_process_env::var("SECURITY_PORT"))
                .or_else(|_| {
                    tracing::warn!(
                        "BEARDOG_PORT is deprecated — migrate to SECURITY_PROVIDER_PORT, SECURITY_PORT, or CRYPTO_PROVIDER_PORT; prefer CAPABILITY_* for capability-first configuration"
                    );
                    songbird_process_env::var("BEARDOG_PORT")
                })
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(songbird_types::defaults::ports::DEFAULT_CRYPTO_TRANSPORT_PORT);

            Ok(format!("{}:{port}", songbird_types::constants::LOCALHOST))
        }

        #[cfg(not(any(unix, windows)))]
        {
            tracing::warn!("⚠️  Platform: Using TCP localhost fallback for security provider");
            let port = songbird_process_env::var("CRYPTO_PROVIDER_PORT")
                .or_else(|_| songbird_process_env::var("SECURITY_PROVIDER_PORT"))
                .or_else(|_| songbird_process_env::var("SECURITY_PORT"))
                .or_else(|_| {
                    tracing::warn!(
                        "BEARDOG_PORT is deprecated — migrate to SECURITY_PROVIDER_PORT, SECURITY_PORT, or CRYPTO_PROVIDER_PORT; prefer CAPABILITY_* for capability-first configuration"
                    );
                    songbird_process_env::var("BEARDOG_PORT")
                })
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(songbird_types::defaults::ports::DEFAULT_CRYPTO_TRANSPORT_PORT);
            Ok(format!("{}:{port}", songbird_types::constants::LOCALHOST))
        }
    }

    /// Make a crypto call — auto-routes based on mode.
    ///
    /// - **Neural API mode**: Wraps in `capability.call` envelope for biomeOS routing.
    /// - **Direct mode**: Calls bearDog semantic methods directly (e.g. `crypto.sign_ed25519`).
    async fn call_capability(
        &self,
        capability: &str,
        operation: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        if self.direct_mode {
            let method = Self::map_to_direct_method(capability, operation);
            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": args,
                "id": 1
            });
            self.send_request(request, &format!("Direct: {method}")).await
        } else {
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
            self.send_request(request, "Capability call").await
        }
    }

    /// Map (capability, operation) to bearDog's direct semantic method name.
    ///
    /// These mappings match bearDog's `capability_registry.toml` crypto domain.
    fn map_to_direct_method(_capability: &str, operation: &str) -> &'static str {
        match operation {
            "generate_keypair" => "crypto.x25519_generate_ephemeral",
            "derive_secret" => "crypto.x25519_derive_secret",
            "encrypt" => "crypto.chacha20_poly1305_encrypt",
            "decrypt" => "crypto.chacha20_poly1305_decrypt",
            "sign" | "sign_ed25519" => "crypto.sign_ed25519",
            "verify" | "verify_ed25519" => "crypto.verify_ed25519",
            "hash_sha3_256" => "crypto.hash_sha3_256",
            "derive_handshake_secrets" => "tls.derive_handshake_secrets",
            "derive_application_secrets" => "tls.derive_application_secrets",
            "compute_finished_verify_data" => "tls.compute_finished_verify_data",
            _ => "crypto.hmac_sha256",
        }
    }

    /// Send a JSON-RPC request over the platform transport and return the result.
    async fn send_request(
        &self,
        request: serde_json::Value,
        error_context: &str,
    ) -> Result<serde_json::Value> {
        let mut stream = Self::connect_platform(&self.socket_path)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to connect: {e}")))?;

        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| TlsError::InternalError(format!("Failed to serialize request: {e}")))?;

        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to send delimiter: {e}")))?;

        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| TlsError::CryptoError(format!("Failed to read response: {e}")))?;

        let response: JsonRpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| TlsError::CryptoError(format!("Failed to parse response: {e}")))?;

        if let Some(error) = response.error {
            return Err(TlsError::CryptoError(format!(
                "{error_context} error: {} (code {})",
                error.message, error.code
            )));
        }

        response.result.ok_or_else(|| {
            TlsError::CryptoError(format!("{error_context} response missing result field"))
        })
    }

    /// Platform-agnostic connection helper.
    ///
    /// Supports `tcp:host:port` for explicit TCP (Android cross-device, Windows)
    /// and filesystem paths for Unix domain sockets via [`IpcStream`].
    async fn connect_platform(path: &str) -> std::io::Result<CryptoStream> {
        if let Some(addr) = path.strip_prefix("tcp:") {
            tracing::debug!("Connecting to TCP socket: {addr}");
            let stream = tokio::net::TcpStream::connect(addr).await?;
            Ok(CryptoStream::Tcp(stream))
        } else {
            tracing::debug!("Connecting to IPC endpoint: {path}");
            IpcStream::connect(path).await
        }
    }

    /// Make a JSON-RPC call with a direct method name (legacy/testing)
    #[allow(
        dead_code,
        reason = "used by crypto unit tests; not referenced from production call sites"
    )]
    async fn call_jsonrpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        self.send_request(request, "security provider").await
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
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing public_key in response")))?;
        let secret_key_b64 = result["secret_key"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing secret_key in response")))?;

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
            TlsError::CryptoError(String::from("Missing shared_secret in response"))
        })?;

        let shared_secret = general_purpose::STANDARD
            .decode(shared_secret_b64)
            .map_err(|e| TlsError::CryptoError(format!("Failed to decode shared_secret: {e}")))?;

        Ok(shared_secret)
    }

    /// Encrypt with ChaCha20-Poly1305 (AEAD)
    ///
    /// The crypto provider generates the nonce.
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
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing ciphertext in response")))?;
        let nonce_b64 = result["nonce"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing nonce in response")))?;
        let tag_b64 = result["tag"]
            .as_str()
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing tag in response")))?;

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
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing plaintext in response")))?;

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
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing signature in response")))?;

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
            .ok_or_else(|| TlsError::CryptoError(String::from("Missing mac in response")))?;

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
#[expect(dead_code, reason = "deserialized from external data")]
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
#[path = "crypto_tests.rs"]
mod tests;
