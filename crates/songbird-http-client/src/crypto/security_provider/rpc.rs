// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC wire protocol, routing mode, and semantic method mapping for `SecurityCryptoProvider`.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use songbird_types::defaults::timeouts::DEFAULT_SECURITY_RPC_TIMEOUT;
use std::sync::atomic::Ordering;
use tokio::io::AsyncWriteExt;
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tracing::trace;

use super::SecurityCryptoProvider;
use crate::error::{Error, Result};

/// Routing mode for the security / crypto provider
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    Direct,
    NeuralApi,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
#[allow(
    dead_code,
    reason = "deserialized from JSON-RPC 2.0 wire; fields accessed by dispatch logic"
)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(
        dead_code,
        reason = "deserialized from JSON-RPC error; available for structured error detail"
    )]
    data: Option<Value>,
}

impl SecurityCryptoProvider {
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

    pub(super) async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = match self.mode {
            RoutingMode::Direct => {
                let actual_method = Self::semantic_to_actual(method);
                JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: actual_method.to_string(),
                    params,
                    id,
                }
            }
            RoutingMode::NeuralApi => {
                let (capability, operation) = Self::method_to_capability(method);

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
            .map_err(|e| Error::SecurityProviderRpc(format!("Failed to serialize request: {e}")))?;

        trace!(
            "Security provider RPC request ({}): {}",
            if self.mode == RoutingMode::NeuralApi {
                "Neural API"
            } else {
                "Direct"
            },
            request_json
        );

        let mut stream = Self::connect_platform_static(&self.socket_path).await.map_err(|e| {
            Error::SecurityProviderRpc(format!(
                "Failed to connect to security provider at {}: {}",
                self.socket_path, e
            ))
        })?;

        stream
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| Error::SecurityProviderRpc(format!("Failed to send request: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| Error::SecurityProviderRpc(format!("Failed to send newline: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| Error::SecurityProviderRpc(format!("Failed to flush: {e}")))?;

        // JSON-aware chunked read — server may keep socket open (no EOF).
        let response_bytes =
            crate::io_util::read_json_response(&mut stream, DEFAULT_SECURITY_RPC_TIMEOUT)
                .await
                .map_err(|e| Error::SecurityProviderRpc(format!("Security provider: {e}")))?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        trace!("Security provider RPC response: {}", response_str);

        let response: JsonRpcResponse = serde_json::from_slice(&response_bytes).map_err(|e| {
            Error::SecurityProviderRpc(format!(
                "Failed to parse response: {e} (raw: {response_str})"
            ))
        })?;

        if let Some(err) = response.error {
            return Err(Error::SecurityProviderRpc(format!(
                "Security provider error: {} (code: {})",
                err.message, err.code
            )));
        }

        response.result.ok_or_else(|| {
            Error::SecurityProviderRpc("Security provider returned null result".to_string())
        })
    }

    pub(crate) fn method_to_capability(method: &str) -> (&'static str, &'static str) {
        use tracing::warn;

        match method {
            "crypto.generate_keypair" => ("crypto", "generate_keypair"),
            "crypto.ecdh_derive" => ("crypto", "derive_secret"),
            "crypto.encrypt_aes_128_gcm" => ("crypto", "encrypt_aes_128_gcm"),
            "crypto.encrypt_aes_256_gcm" => ("crypto", "encrypt_aes_256_gcm"),
            "crypto.encrypt_chacha20_poly1305" => ("crypto", "encrypt_chacha20_poly1305"),
            "crypto.decrypt_aes_128_gcm" => ("crypto", "decrypt_aes_128_gcm"),
            "crypto.decrypt_aes_256_gcm" => ("crypto", "decrypt_aes_256_gcm"),
            "crypto.decrypt_chacha20_poly1305" => ("crypto", "decrypt_chacha20_poly1305"),
            "crypto.sha256" => ("crypto", "sha256"),
            "crypto.sha384" => ("crypto", "sha384"),
            "crypto.hash_for_cipher" => ("crypto", "hash_for_cipher"),
            "crypto.hkdf_extract" => ("crypto", "hkdf_extract"),
            "crypto.hkdf_expand" => ("crypto", "hkdf_expand"),
            "tls.derive_handshake_secrets" => ("tls_crypto", "derive_handshake_secrets"),
            "tls.derive_application_secrets" => ("tls_crypto", "derive_application_secrets"),
            "tls.compute_finished_verify_data" => ("tls_crypto", "compute_finished_verify_data"),
            _ => {
                warn!("Unknown method for capability mapping: {}, using generic operation", method);
                ("crypto", "unknown")
            }
        }
    }

    pub(crate) fn semantic_to_actual(method: &str) -> &str {
        match method {
            "crypto.generate_keypair" => "crypto.x25519_generate_ephemeral",
            "crypto.ecdh_derive" => "crypto.x25519_derive_secret",
            "crypto.encrypt_aes_128_gcm" => "crypto.aes128_gcm_encrypt",
            "crypto.decrypt_aes_128_gcm" => "crypto.aes128_gcm_decrypt",
            "crypto.encrypt_aes_256_gcm" => "crypto.aes256_gcm_encrypt",
            "crypto.decrypt_aes_256_gcm" => "crypto.aes256_gcm_decrypt",
            "crypto.encrypt_chacha20_poly1305" => "crypto.chacha20_poly1305_encrypt",
            "crypto.decrypt_chacha20_poly1305" => "crypto.chacha20_poly1305_decrypt",
            "crypto.sha256" => "crypto.sha256",
            "crypto.sha384" => "crypto.sha384",
            "crypto.hkdf_extract" => "crypto.hkdf_extract",
            "crypto.hkdf_expand" => "crypto.hkdf_expand",
            "tls.derive_handshake_secrets" => "tls.derive_handshake_secrets",
            "tls.derive_application_secrets" => "tls.derive_application_secrets",
            "tls.compute_finished_verify_data" => "tls.compute_finished_verify_data",
            _ => method,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::SecurityCryptoProvider;

    #[test]
    fn semantic_to_actual_maps_crypto_aliases() {
        assert_eq!(
            SecurityCryptoProvider::semantic_to_actual("crypto.generate_keypair"),
            "crypto.x25519_generate_ephemeral"
        );
        assert_eq!(
            SecurityCryptoProvider::semantic_to_actual("crypto.ecdh_derive"),
            "crypto.x25519_derive_secret"
        );
        assert_eq!(
            SecurityCryptoProvider::semantic_to_actual("crypto.encrypt_aes_128_gcm"),
            "crypto.aes128_gcm_encrypt"
        );
        assert_eq!(
            SecurityCryptoProvider::semantic_to_actual("tls.compute_finished_verify_data"),
            "tls.compute_finished_verify_data"
        );
    }

    #[test]
    fn semantic_to_actual_passes_through_unknown_methods() {
        assert_eq!(
            SecurityCryptoProvider::semantic_to_actual("custom.namespace.op"),
            "custom.namespace.op"
        );
    }

    #[test]
    fn method_to_capability_known_tls_methods() {
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("tls.derive_handshake_secrets"),
            ("tls_crypto", "derive_handshake_secrets")
        );
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("tls.derive_application_secrets"),
            ("tls_crypto", "derive_application_secrets")
        );
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("tls.compute_finished_verify_data"),
            ("tls_crypto", "compute_finished_verify_data")
        );
    }

    #[test]
    fn method_to_capability_known_crypto_methods() {
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("crypto.sha256"),
            ("crypto", "sha256")
        );
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("crypto.encrypt_chacha20_poly1305"),
            ("crypto", "encrypt_chacha20_poly1305")
        );
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("crypto.decrypt_aes_256_gcm"),
            ("crypto", "decrypt_aes_256_gcm")
        );
    }

    #[test]
    fn method_to_capability_unknown_falls_back_to_generic() {
        assert_eq!(
            SecurityCryptoProvider::method_to_capability("totally.unknown.method"),
            ("crypto", "unknown")
        );
    }
}
