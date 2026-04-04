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

use super::{CryptoProvider, Result, RpcError};

/// Routing mode for crypto operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingMode {
    /// Call security provider directly (bootstrap / fallback).
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

fn truncate_for_error(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… ({} bytes total)", &s[..end], s.len())
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

        let request_json = serde_json::to_string(&request).map_err(RpcError::RequestSerialize)?;

        trace!(
            "Crypto RPC request ({}): {}",
            if self.mode == RoutingMode::NeuralApi {
                "Neural API"
            } else {
                "Direct"
            },
            request_json
        );

        let target = if self.mode == RoutingMode::NeuralApi {
            "Neural API"
        } else {
            "crypto provider"
        };
        let mut stream = Self::connect_platform(&self.socket_path).await.map_err(|source| {
            RpcError::Connect {
                target,
                path: self.socket_path.clone(),
                source,
            }
        })?;

        stream.write_all(request_json.as_bytes()).await.map_err(RpcError::SendRequest)?;
        stream.shutdown().await.map_err(RpcError::ShutdownWrite)?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).await.map_err(RpcError::ReadResponse)?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        trace!("Crypto RPC response: {}", response_str);

        let raw_preview = truncate_for_error(&response_str, 512);
        let response: JsonRpcResponse =
            serde_json::from_slice(&response_bytes).map_err(|source| RpcError::ResponseParse {
                raw_preview,
                source,
            })?;

        if let Some(err) = response.error {
            return Err(RpcError::Remote {
                code: err.code,
                message: err.message,
            }
            .into());
        }

        response.result.ok_or_else(|| RpcError::NullResult.into())
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
            "crypto.aead.chacha20_poly1305_encrypt" | "crypto.aead_encrypt" => {
                ("crypto", "chacha20_poly1305_encrypt")
            }
            "crypto.aead.chacha20_poly1305_decrypt" | "crypto.aead_decrypt" => {
                ("crypto", "chacha20_poly1305_decrypt")
            }
            // Sovereign onion (`songbird-sovereign-onion`)
            "crypto.ed25519.generate_keypair" => ("crypto", "ed25519_generate_keypair"),
            "crypto.ed25519.public_from_secret" => ("crypto", "ed25519_public_from_secret"),
            // NFC genesis (`songbird-nfc`) — legacy security provider JSON-RPC names
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

    /// Translate semantic method names to `security provider` JSON-RPC wire names (direct mode).
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
            "crypto.encrypt_chacha20_poly1305"
            | "crypto.aead.chacha20_poly1305_encrypt"
            | "crypto.aead_encrypt" => "crypto.chacha20_poly1305_encrypt",
            "crypto.decrypt_chacha20_poly1305"
            | "crypto.aead.chacha20_poly1305_decrypt"
            | "crypto.aead_decrypt" => "crypto.chacha20_poly1305_decrypt",
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
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test harness I/O and tokio::spawn")]
mod tests {
    use super::*;
    use crate::{CryptoProviderError, RpcError};
    use serde_json::json;

    type TestResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
    fn direct_json_rpc_request_serializes_expected_shape() -> TestResult {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: CryptoProvider::semantic_to_actual("crypto.encrypt_aes_256_gcm").to_string(),
            params: json!({ "k": "v" }),
            id: 42,
        };
        let s = serde_json::to_string(&req)?;
        let v: Value = serde_json::from_str(&s)?;
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "crypto.aes256_gcm_encrypt");
        assert_eq!(v["id"], 42);
        assert_eq!(v["params"]["k"], "v");
        Ok(())
    }

    #[test]
    fn neural_json_rpc_request_wraps_capability_call_params() -> TestResult {
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
        let s = serde_json::to_string(&req)?;
        let v: Value = serde_json::from_str(&s)?;
        assert_eq!(v["method"], "capability.call");
        assert_eq!(v["params"]["capability"], "crypto");
        assert_eq!(v["params"]["operation"], "sha256");
        assert_eq!(v["params"]["args"]["data"], "abc");
        Ok(())
    }

    #[test]
    fn json_rpc_response_deserializes_result() -> TestResult {
        let raw = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
        let r: JsonRpcResponse = serde_json::from_str(raw)?;
        assert_eq!(r.jsonrpc, "2.0");
        let result = r.result.as_ref().ok_or_else(|| {
            Box::<dyn std::error::Error + Send + Sync>::from(std::io::Error::other(
                "expected result field",
            ))
        })?;
        assert_eq!(result["ok"], true);
        assert!(r.error.is_none());
        Ok(())
    }

    #[test]
    fn json_rpc_response_deserializes_error() -> TestResult {
        let raw = r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"oops","data":null},"id":2}"#;
        let r: JsonRpcResponse = serde_json::from_str(raw)?;
        let err = r.error.as_ref().ok_or_else(|| {
            Box::<dyn std::error::Error + Send + Sync>::from(std::io::Error::other(
                "expected error field",
            ))
        })?;
        assert_eq!(err.code, -1);
        assert_eq!(err.message, "oops");
        assert!(r.result.is_none());
        Ok(())
    }

    #[test]
    fn crypto_provider_error_display() {
        let e = CryptoProviderError::from(RpcError::NullResult);
        assert_eq!(e.to_string(), "JSON-RPC response contained null result");
    }

    #[test]
    fn semantic_to_actual_maps_chacha_and_ed25519_aliases() {
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.encrypt_chacha20_poly1305"),
            "crypto.chacha20_poly1305_encrypt"
        );
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.aead_encrypt"),
            "crypto.chacha20_poly1305_encrypt"
        );
        assert_eq!(
            CryptoProvider::semantic_to_actual("crypto.ed25519.generate_keypair"),
            "crypto.ed25519_generate_keypair"
        );
    }

    #[test]
    fn method_to_capability_maps_nfc_legacy_and_aead_aliases() {
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.generate_x25519_keypair"),
            ("crypto", "generate_x25519_keypair")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.aead_encrypt"),
            ("crypto", "chacha20_poly1305_encrypt")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.aead_decrypt"),
            ("crypto", "chacha20_poly1305_decrypt")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.hkdf_expand"),
            ("crypto", "hkdf_expand")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.cell.encrypt"),
            ("crypto", "cell_encrypt")
        );
    }

    #[test]
    fn method_to_capability_maps_application_tls_and_hash_for_cipher() {
        assert_eq!(
            CryptoProvider::method_to_capability("tls.derive_application_secrets"),
            ("tls_crypto", "derive_application_secrets")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.hash_for_cipher"),
            ("crypto", "hash_for_cipher")
        );
        assert_eq!(
            CryptoProvider::method_to_capability("crypto.ntor.client_finish"),
            ("crypto", "ntor_client_finish")
        );
    }

    #[cfg(unix)]
    mod unix_call {
        use super::*;
        use serde_json::json;
        use std::sync::{Arc, Mutex};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixListener;

        async fn read_json_rpc_request(
            stream: &mut tokio::net::UnixStream,
        ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>
        {
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await?;
            Ok(serde_json::from_slice(&buf)?)
        }

        #[tokio::test(start_paused = true)]
        async fn call_direct_mode_sends_translated_wire_method_and_returns_result() -> TestResult {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let req = read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                assert_eq!(
                    req["method"], "crypto.aes256_gcm_encrypt",
                    "direct mode should translate semantic method to security provider wire name"
                );
                let id = req["id"].as_u64().expect("id");
                let body = format!(r#"{{"jsonrpc":"2.0","result":{{"digest":"abc"}},"id":{id}}}"#);
                stream.write_all(body.as_bytes()).await.expect("write response");
            });

            let provider = CryptoProvider::with_mode(&path_str, RoutingMode::Direct);
            let result = provider.call("crypto.encrypt_aes_256_gcm", json!({ "k": "v" })).await;
            let val = match result {
                Ok(v) => v,
                Err(e) => return Err(format!("expected Ok, got {e}").into()),
            };
            assert_eq!(val["digest"], "abc", "result payload should match server");
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn call_neural_mode_wraps_capability_call() -> TestResult {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let req = read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                assert_eq!(req["method"], "capability.call");
                let params = &req["params"];
                assert_eq!(params["capability"], "crypto");
                assert_eq!(params["operation"], "sha256");
                assert_eq!(params["args"]["data"], json!([]));
                let id = req["id"].as_u64().expect("id");
                let body = format!(r#"{{"jsonrpc":"2.0","result":{{"ok":true}},"id":{id}}}"#);
                stream.write_all(body.as_bytes()).await.expect("write response");
            });

            let provider = CryptoProvider::with_mode(&path_str, RoutingMode::NeuralApi);
            let result = provider.call("crypto.sha256", json!({ "data": [] })).await;
            let val = match result {
                Ok(v) => v,
                Err(e) => return Err(format!("expected Ok, got {e}").into()),
            };
            assert_eq!(val["ok"], true);
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn call_returns_rpc_error_when_server_returns_jsonrpc_error() -> TestResult {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let req = read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                let id = req["id"].as_u64().expect("id");
                let body = format!(
                    r#"{{"jsonrpc":"2.0","error":{{"code":-32000,"message":"denied","data":null}},"id":{id}}}"#
                );
                stream.write_all(body.as_bytes()).await.expect("write response");
            });

            let provider = CryptoProvider::new(&path_str);
            let result = provider.call("crypto.sha256", json!({})).await;
            let err = match result {
                Err(e) => e,
                Ok(v) => return Err(format!("expected Rpc error, got Ok({v})").into()),
            };
            match err {
                CryptoProviderError::Rpc(RpcError::Remote {
                    code,
                    message,
                }) => {
                    assert_eq!(code, -32000);
                    assert!(
                        message.contains("denied"),
                        "message should include server error details, got {message:?}"
                    );
                }
                e => return Err(format!("expected Rpc Remote error, got {e:?}").into()),
            }
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn call_returns_error_when_result_is_null() -> TestResult {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let req = read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                let id = req["id"].as_u64().expect("id");
                let body = format!(r#"{{"jsonrpc":"2.0","result":null,"id":{id}}}"#);
                stream.write_all(body.as_bytes()).await.expect("write response");
            });

            let provider = CryptoProvider::new(&path_str);
            let result = provider.call("crypto.sha256", json!({})).await;
            let err = match result {
                Err(e) => e,
                Ok(v) => return Err(format!("expected error for null result, got Ok({v})").into()),
            };
            match err {
                CryptoProviderError::Rpc(RpcError::NullResult) => {}
                e => return Err(format!("expected Rpc NullResult, got {e:?}").into()),
            }
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn call_returns_error_when_response_is_not_json() -> TestResult {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let _req = read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                stream.write_all(b"not-json").await.expect("write garbage");
            });

            let provider = CryptoProvider::new(&path_str);
            let result = provider.call("crypto.sha256", json!({})).await;
            let err = match result {
                Err(e) => e,
                Ok(v) => return Err(format!("expected parse error, got Ok({v})").into()),
            };
            match err {
                CryptoProviderError::Rpc(RpcError::ResponseParse {
                    raw_preview,
                    ..
                }) => {
                    assert!(
                        raw_preview.contains("not-json"),
                        "raw preview should include server bytes, got {raw_preview:?}"
                    );
                }
                e => return Err(format!("expected Rpc ResponseParse, got {e:?}").into()),
            }
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn call_connection_refused_reports_neural_or_crypto_in_message() -> TestResult {
            let provider =
                CryptoProvider::with_mode("/nonexistent/path/to.sock", RoutingMode::Direct);
            let result = provider.call("crypto.sha256", json!({})).await;
            let err = match result {
                Err(e) => e,
                Ok(v) => return Err(format!("expected connect failure, got Ok({v})").into()),
            };
            match err {
                CryptoProviderError::Rpc(RpcError::Connect {
                    target,
                    path,
                    ..
                }) => {
                    assert_eq!(target, "crypto provider");
                    assert_eq!(path, "/nonexistent/path/to.sock");
                }
                e => return Err(format!("expected Rpc Connect, got {e:?}").into()),
            }

            let neural =
                CryptoProvider::with_mode("/nonexistent/neural.sock", RoutingMode::NeuralApi);
            let result = neural.call("crypto.sha256", json!({})).await;
            let err = match result {
                Err(e) => e,
                Ok(v) => return Err(format!("expected neural connect failure, got Ok({v})").into()),
            };
            match err {
                CryptoProviderError::Rpc(RpcError::Connect {
                    target,
                    path,
                    ..
                }) => {
                    assert_eq!(target, "Neural API");
                    assert_eq!(path, "/nonexistent/neural.sock");
                }
                e => return Err(format!("expected Rpc Connect, got {e:?}").into()),
            }
            Ok(())
        }

        #[tokio::test(start_paused = true)]
        async fn sequential_calls_use_incrementing_json_rpc_ids() {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join("sock");
            let path_str = path.to_string_lossy().to_string();
            let listener = UnixListener::bind(&path_str).expect("bind listener");

            let seen = Arc::new(Mutex::new(Vec::new()));
            let seen_clone = Arc::clone(&seen);

            tokio::spawn(async move {
                for _ in 0..2 {
                    let (mut stream, _) = listener.accept().await.expect("accept");
                    let req =
                        read_json_rpc_request(&mut stream).await.expect("read_json_rpc_request");
                    let id = req["id"].as_u64().expect("id");
                    seen_clone.lock().expect("lock").push(id);
                    let body = format!(r#"{{"jsonrpc":"2.0","result":{{}},"id":{id}}}"#);
                    stream.write_all(body.as_bytes()).await.expect("write");
                }
            });

            let provider = CryptoProvider::new(&path_str);
            let r1 = provider.call("crypto.sha256", json!({})).await;
            assert!(r1.is_ok(), "first call: {:?}", r1);
            let r2 = provider.call("crypto.sha256", json!({})).await;
            assert!(r2.is_ok(), "second call: {:?}", r2);

            let ids = seen.lock().expect("lock");
            assert_eq!(&*ids, &[1, 2], "fetch_add should yield 1 then 2 for first two calls");
        }
    }
}
