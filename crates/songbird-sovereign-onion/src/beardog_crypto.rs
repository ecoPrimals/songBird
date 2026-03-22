// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Neural API / BearDog crypto delegation via [`songbird_crypto_provider::CryptoProvider`]
//!
//! All cryptographic operations are routed through `CryptoProvider::from_env()` (Neural API by
//! default; set `BEARDOG_MODE=direct` for bootstrap). See `songbird-crypto-provider` for
//! environment variables and socket discovery.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let client = BeardogCryptoClient::from_env();
//! let keypair = client.ed25519_generate_keypair().await?;
//! ```

use crate::error::{OnionError, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use songbird_crypto_provider::{CryptoProvider, RoutingMode};

/// `BearDog` / Neural API crypto client for TRUE PRIMAL delegation
#[derive(Clone, Debug)]
pub struct BeardogCryptoClient {
    provider: CryptoProvider,
}

impl BeardogCryptoClient {
    /// Create client using [`CryptoProvider::from_env`] (Neural API socket by default).
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            provider: CryptoProvider::from_env(),
        }
    }

    /// Wrap an existing provider (tests or custom wiring).
    #[must_use]
    pub fn from_provider(provider: CryptoProvider) -> Self {
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
                "BearDog ed25519_public_from_secret: invalid public key length".into(),
            )
        })
    }

    // =========================================================================
    // X25519 Operations (Session Keys)
    // =========================================================================

    /// Generate X25519 ephemeral keypair for session key exchange
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
    use super::*;

    #[tokio::test]
    async fn from_provider_unreachable_socket_errors_on_call() {
        let client = BeardogCryptoClient::from_provider(CryptoProvider::new(
            "/tmp/songbird-sovereign-onion-no-such.sock",
        ));
        let r = client.ed25519_generate_keypair().await;
        assert!(r.is_err());
    }
}
