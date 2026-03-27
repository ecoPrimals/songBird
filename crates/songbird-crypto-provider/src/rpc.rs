// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC wire protocol, routing mode, and semantic method mapping.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tracing::{trace, warn};

use super::{CryptoProvider, CryptoProviderError, Result};

/// Routing mode for crypto operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    /// Call `BearDog` directly (bootstrap / fallback).
    Direct,
    /// Route via Neural API `capability.call` (production default).
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
#[expect(dead_code, reason = "fields consumed via Deserialize")]
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
    #[expect(dead_code, reason = "consumed via Deserialize")]
    data: Option<Value>,
}

impl CryptoProvider {
    #[cfg(unix)]
    async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(path).await
    }

    #[cfg(windows)]
    async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
        PlatformStream::connect(address).await
    }

    /// Call a crypto method, routing through Neural API or direct depending on mode.
    ///
    /// # Errors
    /// Returns an error if the RPC call fails or the response cannot be parsed.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
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
            .map_err(|e| CryptoProviderError::Rpc(format!("Failed to serialize request: {e}")))?;

        trace!(
            "Crypto RPC request ({}): {}",
            if self.mode == RoutingMode::NeuralApi {
                "Neural API"
            } else {
                "Direct"
            },
            request_json
        );

        let mut stream = Self::connect_platform(&self.socket_path).await.map_err(|e| {
            CryptoProviderError::Rpc(format!(
                "Failed to connect to {} at {}: {}",
                if self.mode == RoutingMode::NeuralApi {
                    "Neural API"
                } else {
                    "crypto provider"
                },
                self.socket_path,
                e
            ))
        })?;

        stream
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| CryptoProviderError::Rpc(format!("Failed to send request: {e}")))?;
        stream
            .shutdown()
            .await
            .map_err(|e| CryptoProviderError::Rpc(format!("Failed to shutdown write: {e}")))?;

        let mut response_bytes = Vec::new();
        stream
            .read_to_end(&mut response_bytes)
            .await
            .map_err(|e| CryptoProviderError::Rpc(format!("Failed to read response: {e}")))?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        trace!("Crypto RPC response: {}", response_str);

        let response: JsonRpcResponse = serde_json::from_slice(&response_bytes).map_err(|e| {
            CryptoProviderError::Rpc(format!("Failed to parse response: {e} (raw: {response_str})"))
        })?;

        if let Some(err) = response.error {
            return Err(CryptoProviderError::Rpc(format!(
                "RPC error: {} (code: {})",
                err.message, err.code
            )));
        }

        response.result.ok_or_else(|| CryptoProviderError::Rpc("Null result".to_string()))
    }

    pub fn method_to_capability(method: &str) -> (&'static str, &'static str) {
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
            // Tor protocol (`songbird-tor-protocol`)
            "crypto.ntor.client_init" => ("crypto", "ntor_client_init"),
            "crypto.ntor.client_finish" => ("crypto", "ntor_client_finish"),
            "crypto.kdf.derive" => ("crypto", "kdf_derive"),
            "crypto.cell.encrypt" => ("crypto", "cell_encrypt"),
            "crypto.cell.decrypt" => ("crypto", "cell_decrypt"),
            "crypto.sign.ed25519" => ("crypto", "sign_ed25519"),
            "crypto.verify.ed25519" => ("crypto", "verify_ed25519"),
            "crypto.x25519.generate_ephemeral" => ("crypto", "x25519_generate_ephemeral"),
            "crypto.x25519.derive_secret" => ("crypto", "x25519_derive_secret"),
            "crypto.hash.sha3_256" => ("crypto", "hash_sha3_256"),
            "crypto.hmac.sha256" => ("crypto", "hmac_sha256"),
            "crypto.aead.chacha20_poly1305_encrypt" => ("crypto", "chacha20_poly1305_encrypt"),
            "crypto.aead.chacha20_poly1305_decrypt" => ("crypto", "chacha20_poly1305_decrypt"),
            // Sovereign onion (`songbird-sovereign-onion`)
            "crypto.ed25519.generate_keypair" => ("crypto", "ed25519_generate_keypair"),
            "crypto.ed25519.public_from_secret" => ("crypto", "ed25519_public_from_secret"),
            // NFC genesis (`songbird-nfc`) — legacy BearDog JSON-RPC names
            "crypto.generate_x25519_keypair" => ("crypto", "generate_x25519_keypair"),
            "crypto.x25519_dh" => ("crypto", "x25519_dh"),
            "crypto.generate_random" => ("crypto", "generate_random"),
            "crypto.chacha20poly1305_encrypt" => ("crypto", "chacha20poly1305_encrypt"),
            "crypto.chacha20poly1305_decrypt" => ("crypto", "chacha20poly1305_decrypt"),
            "crypto.ed25519_sign" => ("crypto", "ed25519_sign"),
            "crypto.ed25519_verify" => ("crypto", "ed25519_verify"),
            "crypto.destroy_ephemeral_keys" => ("crypto", "destroy_ephemeral_keys"),
            _ => {
                warn!("Unknown method for capability mapping: {}, using generic operation", method);
                ("crypto", "unknown")
            }
        }
    }

    /// Translate semantic method names to `BearDog` JSON-RPC wire names (direct mode).
    ///
    /// Methods that share the same semantic and wire name pass through the wildcard arm.
    #[must_use]
    pub fn semantic_to_actual(method: &str) -> &str {
        match method {
            "crypto.generate_keypair" | "crypto.x25519.generate_ephemeral" => {
                "crypto.x25519_generate_ephemeral"
            }
            "crypto.ecdh_derive" | "crypto.x25519.derive_secret" => "crypto.x25519_derive_secret",
            "crypto.encrypt_aes_128_gcm" => "crypto.aes128_gcm_encrypt",
            "crypto.decrypt_aes_128_gcm" => "crypto.aes128_gcm_decrypt",
            "crypto.encrypt_aes_256_gcm" => "crypto.aes256_gcm_encrypt",
            "crypto.decrypt_aes_256_gcm" => "crypto.aes256_gcm_decrypt",
            "crypto.encrypt_chacha20_poly1305" | "crypto.aead.chacha20_poly1305_encrypt" => {
                "crypto.chacha20_poly1305_encrypt"
            }
            "crypto.decrypt_chacha20_poly1305" | "crypto.aead.chacha20_poly1305_decrypt" => {
                "crypto.chacha20_poly1305_decrypt"
            }
            "crypto.sign.ed25519" => "crypto.sign_ed25519",
            "crypto.verify.ed25519" => "crypto.verify_ed25519",
            "crypto.hash.sha3_256" => "crypto.sha3_256",
            "crypto.hmac.sha256" => "crypto.hmac_sha256",
            "crypto.ed25519.generate_keypair" => "crypto.ed25519_generate_keypair",
            "crypto.ed25519.public_from_secret" => "crypto.ed25519_public_from_secret",
            _ => method,
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::CryptoProviderError;
    use serde_json::json;

    #[test]
    fn semantic_to_actual_translates_aes_and_x25519_aliases() {
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.encrypt_aes_128_gcm"),
            "crypto.aes128_gcm_encrypt"
        );
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.generate_keypair"),
            "crypto.x25519_generate_ephemeral"
        );
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.ecdh_derive"),
            "crypto.x25519_derive_secret"
        );
    }

    #[test]
    fn semantic_to_actual_passes_through_unknown_methods() {
        assert_eq!(CryptoProvider::semantic_to_actual("crypto.custom.op"), "crypto.custom.op");
    }

    #[test]
    fn method_to_capability_maps_tls_and_tor_methods() {
        assert_eq!(
            CryptoProvider::method_to_capability("tls.derive_handshake_secrets"),
            ("tls_crypto", "derive_handshake_secrets")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.ntor.client_init"),
            ("crypto", "ntor_client_init")
        );
    }

    #[test]
    fn method_to_capability_unknown_falls_back_to_crypto_unknown() {
        assert_eq!(
            CryptoProvider::method_to_capability("not.a.real.method"),
            ("crypto", "unknown")
        );
    }

    #[test]
    fn direct_json_rpc_request_serializes_expected_shape() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: CryptoProvider::semantic_to_actual("crypto.encrypt_aes_256_gcm").to_string(),
            params: json!({ "k": "v" }),
            id: 42,
        };
        let s = serde_json::to_string(&req).unwrap();
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "crypto.aes256_gcm_encrypt");
        assert_eq!(v["id"], 42);
        assert_eq!(v["params"]["k"], "v");
    }

    #[test]
    fn neural_json_rpc_request_wraps_capability_call_params() {
        let (cap, op) = CryptoProvider::method_to_capability("crypto.sha256");
        assert_eq!((cap, op), ("crypto", "sha256"));
        let inner = json!({
            "capability": cap,
            "operation": op,
            "args": json!({ "data": "abc" })
        });
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "capability.call".to_string(),
            params: inner,
            id: 7,
        };
        let s = serde_json::to_string(&req).unwrap();
        let v: Value = s.parse().unwrap();
        assert_eq!(v["method"], "capability.call");
        assert_eq!(v["params"]["capability"], "crypto");
        assert_eq!(v["params"]["operation"], "sha256");
        assert_eq!(v["params"]["args"]["data"], "abc");
    }

    #[test]
    fn json_rpc_response_deserializes_result() {
        let raw = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
        let r: JsonRpcResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(r.jsonrpc, "2.0");
        assert_eq!(r.result.unwrap()["ok"], true);
        assert!(r.error.is_none());
    }

    #[test]
    fn json_rpc_response_deserializes_error() {
        let raw = r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"oops","data":null},"id":2}"#;
        let r: JsonRpcResponse = serde_json::from_str(raw).unwrap();
        let err = r.error.unwrap();
        assert_eq!(err.code, -1);
        assert_eq!(err.message, "oops");
        assert!(r.result.is_none());
    }

    #[test]
    fn crypto_provider_error_display() {
        let e = CryptoProviderError::Rpc("connection failed".to_string());
        assert_eq!(e.to_string(), "RPC error: connection failed");
    }
}
