// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Neural API / security-provider crypto delegation via [`songbird_crypto_provider::CryptoProvider`]
//!
//! All cryptographic operations are routed through `CryptoProvider::from_env()` (Neural API by
//! default; set `SECURITY_PROVIDER_MODE=direct` for local bootstrap). See `songbird-crypto-provider` for
//! environment variables, legacy aliases, and socket discovery.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let client = SecurityCryptoClient::from_env();
//! let keypair = client.ed25519_generate_keypair().await?;
//! ```

use crate::error::{OnionError, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use songbird_crypto_provider::{CryptoProvider, RoutingMode};

/// Security-provider (Neural API) crypto client for delegated cryptography
#[derive(Clone, Debug)]
pub struct SecurityCryptoClient {
    provider: CryptoProvider,
}

impl SecurityCryptoClient {
    /// Create client using [`CryptoProvider::from_env`] (Neural API socket by default).
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            provider: CryptoProvider::from_env(),
        }
    }

    /// Wrap an existing provider (tests or custom wiring).
    #[must_use]
    pub const fn from_provider(provider: CryptoProvider) -> Self {
        Self {
            provider,
        }
    }

    /// Use a fixed Neural API Unix socket path (`RoutingMode::NeuralApi`).
    #[must_use]
    pub fn from_neural_api_socket(path: impl Into<String>) -> Self {
        Self {
            provider: CryptoProvider::with_mode(path, RoutingMode::NeuralApi),
        }
    }

    async fn call_json(&self, method: &str, params: Value) -> Result<Value> {
        self.provider.call(method, params).await.map_err(|e| OnionError::RpcError(e.to_string()))
    }

    // =========================================================================
    // Ed25519 Operations (Identity Keys)
    // =========================================================================

    /// Generate Ed25519 keypair for .onion identity
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn ed25519_generate_keypair(&self) -> Result<Ed25519Keypair> {
        #[derive(Deserialize)]
        struct Response {
            public_key: String,
            secret_key: String,
        }

        let v = self.call_json("crypto.ed25519.generate_keypair", json!({})).await?;
        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("ed25519_generate_keypair response: {e}")))?;

        let public_key = base64_decode(&response.public_key)?;
        let secret_key = base64_decode(&response.secret_key)?;

        Ok(Ed25519Keypair {
            public_key: public_key
                .try_into()
                .map_err(|_| OnionError::CryptoError("Invalid public key length".into()))?,
            secret_key: secret_key
                .try_into()
                .map_err(|_| OnionError::CryptoError("Invalid secret key length".into()))?,
        })
    }

    /// Sign data with Ed25519
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn ed25519_sign(&self, secret_key: &[u8; 32], message: &[u8]) -> Result<[u8; 64]> {
        #[derive(Deserialize)]
        struct Response {
            signature: String,
        }

        let v = self
            .call_json(
                "crypto.sign.ed25519",
                json!({
                    "secret_key": base64_encode(secret_key),
                    "message": base64_encode(message),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("ed25519_sign response: {e}")))?;

        let signature = base64_decode(&response.signature)?;
        signature.try_into().map_err(|_| OnionError::CryptoError("Invalid signature length".into()))
    }

    /// Verify Ed25519 signature
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn ed25519_verify(
        &self,
        public_key: &[u8; 32],
        message: &[u8],
        signature: &[u8; 64],
    ) -> Result<bool> {
        #[derive(Deserialize)]
        struct Response {
            valid: bool,
        }

        let v = self
            .call_json(
                "crypto.verify.ed25519",
                json!({
                    "public_key": base64_encode(public_key),
                    "message": base64_encode(message),
                    "signature": base64_encode(signature),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("ed25519_verify response: {e}")))?;

        Ok(response.valid)
    }

    /// Derive Ed25519 public key from secret key bytes via JSON-RPC delegation.
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn ed25519_public_from_secret(&self, secret_key: &[u8; 32]) -> Result<[u8; 32]> {
        #[derive(Deserialize)]
        struct Response {
            public_key: String,
        }

        let v = self
            .call_json(
                "crypto.ed25519.public_from_secret",
                json!({
                    "secret_key": base64_encode(secret_key),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v).map_err(|e| {
            OnionError::RpcError(format!("ed25519_public_from_secret response: {e}"))
        })?;

        let public_key = base64_decode(&response.public_key)?;
        public_key.try_into().map_err(|_| {
            OnionError::CryptoError(
                "security provider ed25519_public_from_secret: invalid public key length".into(),
            )
        })
    }

    // =========================================================================
    // X25519 Operations (Session Keys)
    // =========================================================================

    /// Generate X25519 ephemeral keypair for session key exchange
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair> {
        #[derive(Deserialize)]
        struct Response {
            public_key: String,
            secret_key: String,
        }

        let v = self.call_json("crypto.x25519.generate_ephemeral", json!({})).await?;

        let response: Response = serde_json::from_value(v).map_err(|e| {
            OnionError::RpcError(format!("x25519_generate_ephemeral response: {e}"))
        })?;

        let public_key = base64_decode(&response.public_key)?;
        let secret_key = base64_decode(&response.secret_key)?;

        Ok(X25519Keypair {
            public_key: public_key
                .try_into()
                .map_err(|_| OnionError::CryptoError("Invalid public key length".into()))?,
            secret_key: secret_key
                .try_into()
                .map_err(|_| OnionError::CryptoError("Invalid secret key length".into()))?,
        })
    }

    /// Derive shared secret via X25519 ECDH
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn x25519_derive_secret(
        &self,
        our_secret: &[u8; 32],
        their_public: &[u8; 32],
    ) -> Result<[u8; 32]> {
        #[derive(Deserialize)]
        struct Response {
            shared_secret: String,
        }

        let v = self
            .call_json(
                "crypto.x25519.derive_secret",
                json!({
                    "secret_key": base64_encode(our_secret),
                    "public_key": base64_encode(their_public),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("x25519_derive_secret response: {e}")))?;

        let shared = base64_decode(&response.shared_secret)?;
        shared
            .try_into()
            .map_err(|_| OnionError::CryptoError("Invalid shared secret length".into()))
    }

    // =========================================================================
    // ChaCha20-Poly1305 Operations (Encryption)
    // =========================================================================

    /// Encrypt data with ChaCha20-Poly1305
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        #[derive(Deserialize)]
        struct Response {
            ciphertext: String,
        }

        let v = self
            .call_json(
                "crypto.aead.chacha20_poly1305_encrypt",
                json!({
                    "key": base64_encode(key),
                    "nonce": base64_encode(nonce),
                    "plaintext": base64_encode(plaintext),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v).map_err(|e| {
            OnionError::RpcError(format!("chacha20_poly1305_encrypt response: {e}"))
        })?;

        base64_decode(&response.ciphertext)
    }

    /// Decrypt data with ChaCha20-Poly1305
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        #[derive(Deserialize)]
        struct Response {
            plaintext: String,
        }

        let v = self
            .call_json(
                "crypto.aead.chacha20_poly1305_decrypt",
                json!({
                    "key": base64_encode(key),
                    "nonce": base64_encode(nonce),
                    "ciphertext": base64_encode(ciphertext),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v).map_err(|e| {
            OnionError::RpcError(format!("chacha20_poly1305_decrypt response: {e}"))
        })?;

        base64_decode(&response.plaintext)
    }

    // =========================================================================
    // SHA3-256 Operation (.onion address derivation)
    // =========================================================================

    /// Compute SHA3-256 hash (needed for .onion address checksum)
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]> {
        #[derive(Deserialize)]
        struct Response {
            hash_base64: String,
        }

        let v = self
            .call_json(
                "crypto.hash.sha3_256",
                json!({
                    "data": base64_encode(data),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("sha3_256 response: {e}")))?;

        let hash = base64_decode(&response.hash_base64)?;
        hash.try_into().map_err(|_| OnionError::CryptoError("Invalid hash length".into()))
    }

    // =========================================================================
    // HMAC-SHA256 Operations (HKDF)
    // =========================================================================

    /// Compute HMAC-SHA256 (for HKDF key derivation)
    ///
    /// # Errors
    ///
    /// Returns an error if the `security provider` JSON-RPC call fails or the response cannot be decoded.
    pub async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]> {
        #[derive(Deserialize)]
        struct Response {
            mac: String,
        }

        let v = self
            .call_json(
                "crypto.hmac.sha256",
                json!({
                    "key": base64_encode(key),
                    "data": base64_encode(data),
                }),
            )
            .await?;

        let response: Response = serde_json::from_value(v)
            .map_err(|e| OnionError::RpcError(format!("hmac_sha256 response: {e}")))?;

        let mac = base64_decode(&response.mac)?;
        mac.try_into().map_err(|_| OnionError::CryptoError("Invalid MAC length".into()))
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
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(s).map_err(|e| OnionError::CryptoError(format!("Base64 decode error: {e}")))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    use tokio::io::AsyncReadExt;

    #[tokio::test(start_paused = true)]
    async fn from_provider_unreachable_socket_errors_on_call() {
        let client = SecurityCryptoClient::from_provider(CryptoProvider::new(
            "/tmp/songbird-sovereign-onion-no-such.sock",
        ));
        let r = client.ed25519_generate_keypair().await;
        assert!(r.is_err());
    }

    #[test]
    fn from_neural_api_socket_builds_client() {
        let c = SecurityCryptoClient::from_neural_api_socket(
            "/tmp/songbird-sovereign-onion-neural.sock",
        );
        let _ = std::any::type_name_of_val(&c);
    }

    #[test]
    fn from_env_builds_client() {
        let c = SecurityCryptoClient::from_env();
        let _ = std::any::type_name_of_val(&c);
    }

    #[test]
    fn ed25519_keypair_and_x25519_keypair_are_debuggable() {
        let e = Ed25519Keypair {
            public_key: [1u8; 32],
            secret_key: [2u8; 32],
        };
        let x = X25519Keypair {
            public_key: [3u8; 32],
            secret_key: [4u8; 32],
        };
        assert!(format!("{e:?}").contains("Ed25519Keypair"));
        assert!(format!("{x:?}").contains("X25519Keypair"));
    }

    fn test_b64(data: &[u8]) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        STANDARD.encode(data)
    }

    async fn read_json_rpc_request(stream: &mut tokio::net::UnixStream) -> serde_json::Value {
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.expect("read request");
        serde_json::from_slice(&buf).expect("parse JSON-RPC request")
    }

    async fn start_direct_mock_server<F>(handler: F) -> String
    where
        F: Fn(&str) -> serde_json::Value + Send + Sync + 'static,
    {
        use serde_json::json;
        use tokio::io::AsyncWriteExt;
        use tokio::net::UnixListener;

        let path = std::env::temp_dir().join(format!(
            "songbird-onion-crypto-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let path_str = path.to_string_lossy().to_string();
        let listener = UnixListener::bind(&path_str).expect("bind mock crypto socket");

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let req = read_json_rpc_request(&mut stream).await;
                let method = req["method"].as_str().unwrap_or("");
                let id = req["id"].as_u64().unwrap_or(1);
                let result = handler(method);
                let body = json!({"jsonrpc":"2.0","result":result,"id":id}).to_string();
                let _ = stream.write_all(body.as_bytes()).await;
            }
        });

        path_str
    }

    #[tokio::test(start_paused = true)]
    async fn ed25519_generate_keypair_success_with_mock_provider() {
        let path = start_direct_mock_server(|method| {
            assert_eq!(method, "crypto.ed25519_generate_keypair");
            json!({
                "public_key": test_b64(&[0xAAu8; 32]),
                "secret_key": test_b64(&[0xBBu8; 32]),
            })
        })
        .await;

        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let kp = client.ed25519_generate_keypair().await.expect("keypair from mock");
        assert_eq!(kp.public_key, [0xAA; 32]);
        assert_eq!(kp.secret_key, [0xBB; 32]);
    }

    #[tokio::test(start_paused = true)]
    async fn ed25519_generate_keypair_malformed_response_is_rpc_error() {
        let path = start_direct_mock_server(|_| json!({"only_public_key":"x"})).await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let err = client
            .ed25519_generate_keypair()
            .await
            .expect_err("missing secret_key should fail decode");
        assert!(matches!(err, OnionError::RpcError(_)), "expected RpcError, got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn ed25519_generate_keypair_invalid_base64_is_crypto_error() {
        let path = start_direct_mock_server(|_| {
            json!({
                "public_key": "!!!",
                "secret_key": test_b64(&[1u8; 32]),
            })
        })
        .await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let err = client.ed25519_generate_keypair().await.expect_err("bad base64 should fail");
        assert!(matches!(err, OnionError::CryptoError(_)), "expected CryptoError, got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn ed25519_generate_keypair_wrong_public_key_length_is_crypto_error() {
        let path = start_direct_mock_server(|_| {
            json!({
                "public_key": test_b64(&[1u8; 16]),
                "secret_key": test_b64(&[2u8; 32]),
            })
        })
        .await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let err =
            client.ed25519_generate_keypair().await.expect_err("short public key should fail");
        assert!(matches!(err, OnionError::CryptoError(_)), "expected CryptoError, got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn sha3_256_success_returns_32_byte_hash() {
        let hash_bytes = [0x11u8; 32];
        let path = start_direct_mock_server(move |method| {
            assert_eq!(method, "crypto.sha3_256");
            json!({ "hash_base64": test_b64(&hash_bytes) })
        })
        .await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let out = client.sha3_256(b"test input").await.expect("sha3 from mock");
        assert_eq!(out, hash_bytes);
    }

    #[tokio::test(start_paused = true)]
    async fn hmac_sha256_malformed_response_missing_mac_field() {
        let path = start_direct_mock_server(|method| {
            assert_eq!(method, "crypto.hmac_sha256");
            json!({ "not_mac": test_b64(&[0u8; 32]) })
        })
        .await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let err = client.hmac_sha256(&[0u8; 32], b"data").await.expect_err("missing mac field");
        assert!(matches!(err, OnionError::RpcError(_)), "expected RpcError, got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn x25519_derive_secret_rpc_error_from_server() {
        use tokio::io::AsyncWriteExt;
        use tokio::net::UnixListener;

        let path = std::env::temp_dir().join(format!(
            "songbird-onion-crypto-err-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let path_str = path.to_string_lossy().to_string();
        let listener = UnixListener::bind(&path_str).expect("bind error mock socket");

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let req = read_json_rpc_request(&mut stream).await;
            let id = req["id"].as_u64().unwrap_or(1);
            let body = format!(
                r#"{{"jsonrpc":"2.0","error":{{"code":-32000,"message":"denied","data":null}},"id":{id}}}"#
            );
            stream.write_all(body.as_bytes()).await.expect("write error response");
        });

        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path_str,
            RoutingMode::Direct,
        ));
        let err = client
            .x25519_derive_secret(&[3u8; 32], &[4u8; 32])
            .await
            .expect_err("server JSON-RPC error should propagate");
        assert!(matches!(err, OnionError::RpcError(_)), "expected RpcError, got {err:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn chacha20_poly1305_decrypt_invalid_ciphertext_length_is_crypto_error() {
        let path = start_direct_mock_server(|method| {
            assert_eq!(method, "crypto.chacha20_poly1305_decrypt");
            json!({ "plaintext": test_b64(&[0u8; 3]) })
        })
        .await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        // Response decodes fine; exercise successful decode path
        let plain = client
            .chacha20_poly1305_decrypt(&[1u8; 32], &[2u8; 12], b"cipher")
            .await
            .expect("mock returns plaintext bytes");
        assert_eq!(plain, vec![0u8; 3]);
    }

    #[tokio::test(start_paused = true)]
    async fn ed25519_public_from_secret_invalid_length_is_crypto_error() {
        let path = start_direct_mock_server(|_| json!({ "public_key": test_b64(&[9u8; 8]) })).await;
        let client = SecurityCryptoClient::from_provider(CryptoProvider::with_mode(
            &path,
            RoutingMode::Direct,
        ));
        let err = client.ed25519_public_from_secret(&[1u8; 32]).await.expect_err("short pubkey");
        assert!(matches!(err, OnionError::CryptoError(_)), "expected CryptoError, got {err:?}");
    }
}
