// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
#![allow(async_fn_in_trait, reason = "native async trait methods; futures are Send in impls")]

//! Cryptographic operations for Tor protocol
//!
//! - **`security provider` / Neural API**: Key operations delegated via IPC (`songbird-crypto-provider`)
//! - **SHA3-256**: Pure Rust for local operations (onion address checksums, descriptor IDs)

pub mod sha3;

pub use songbird_crypto_provider::{CryptoProvider, CryptoProviderError, RoutingMode};

use crate::error::{Error, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::json;

fn map_crypto_err(e: &CryptoProviderError) -> Error {
    Error::Crypto(e.to_string())
}

/// Tor-protocol crypto operations routed through [`CryptoProvider`].
pub trait TorProtocolCrypto {
    /// Initialize client-side ntor handshake
    ///
    /// Returns (`client_public_key`, `handshake_state`) for CREATE2 payload.
    async fn tor_ntor_client_init(
        &self,
        node_id: &[u8; 20],
        node_onion_key: &[u8; 32],
    ) -> Result<NtorClientInit>;

    /// Complete client-side ntor handshake with server's response
    async fn tor_ntor_client_finish(
        &self,
        state_id: &str,
        server_public: &[u8; 32],
        auth_tag: &[u8; 32],
    ) -> Result<KeyMaterial>;

    /// Tor-specific Key Derivation Function
    async fn tor_kdf(&self, key_seed: &[u8; 32], info: &[u8], length: usize) -> Result<Vec<u8>>;

    /// Encrypt Tor cell data with `ChaCha20`
    async fn tor_cell_encrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt Tor cell data with `ChaCha20`
    async fn tor_cell_decrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>>;

    /// Encrypt with AES-128-CTR (for onion layer encryption)
    async fn aes_128_ctr_encrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>>;

    /// Decrypt with AES-128-CTR (for onion layer decryption)
    async fn aes_128_ctr_decrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>>;

    /// Sign data with Ed25519
    async fn ed25519_sign(&self, secret_key_id: &str, data: &[u8]) -> Result<[u8; 64]>;

    /// Verify Ed25519 signature
    async fn ed25519_verify(
        &self,
        public_key: &[u8; 32],
        data: &[u8],
        signature: &[u8; 64],
    ) -> Result<bool>;

    /// Generate ephemeral X25519 keypair
    async fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair>;

    /// Derive shared secret (ECDH)
    async fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8; 32],
        their_public_key: &[u8; 32],
    ) -> Result<[u8; 32]>;

    /// Hash with SHA3-256
    async fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]>;

    /// HMAC-SHA256 (required for Tor ntor handshake)
    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]>;

    /// Encrypt with `ChaCha20Poly1305`
    async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;

    /// Decrypt with `ChaCha20Poly1305`
    async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>>;
}

impl TorProtocolCrypto for CryptoProvider {
    async fn tor_ntor_client_init(
        &self,
        node_id: &[u8; 20],
        node_onion_key: &[u8; 32],
    ) -> Result<NtorClientInit> {
        let result = self
            .call(
                "crypto.ntor.client_init",
                json!({
                    "node_id": BASE64.encode(node_id),
                    "node_onion_key": BASE64.encode(node_onion_key)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let client_public_b64 =
            result.get("client_public").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto(String::from("Missing client_public in ntor_client_init response"))
            })?;

        let state_id = result.get("state_id").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing state_id in ntor_client_init response"))
        })?;

        let client_public_bytes = BASE64
            .decode(client_public_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode client_public: {e}")))?;

        let mut client_public = [0u8; 32];
        if client_public_bytes.len() >= 32 {
            client_public.copy_from_slice(&client_public_bytes[..32]);
        }

        Ok(NtorClientInit {
            client_public,
            state_id: state_id.to_string(),
        })
    }

    async fn tor_ntor_client_finish(
        &self,
        state_id: &str,
        server_public: &[u8; 32],
        auth_tag: &[u8; 32],
    ) -> Result<KeyMaterial> {
        let result = self
            .call(
                "crypto.ntor.client_finish",
                json!({
                    "state_id": state_id,
                    "server_public": BASE64.encode(server_public),
                    "auth_tag": BASE64.encode(auth_tag)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let key_seed_b64 = result.get("key_seed").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing key_seed in ntor_client_finish response"))
        })?;

        let key_seed_bytes = BASE64
            .decode(key_seed_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode key_seed: {e}")))?;

        let mut key_seed = [0u8; 32];
        if key_seed_bytes.len() >= 32 {
            key_seed.copy_from_slice(&key_seed_bytes[..32]);
        }

        let keys = self.tor_kdf(&key_seed, b"tor_circuit_keys", 72).await?;

        let mut forward_key = [0u8; 16];
        let mut backward_key = [0u8; 16];
        let mut forward_iv = [0u8; 16];
        let mut backward_iv = [0u8; 16];

        forward_key.copy_from_slice(&keys[0..16]);
        backward_key.copy_from_slice(&keys[16..32]);
        forward_iv.copy_from_slice(&keys[32..48]);
        backward_iv.copy_from_slice(&keys[48..64]);

        Ok(KeyMaterial {
            forward_key,
            backward_key,
            forward_iv,
            backward_iv,
        })
    }

    async fn tor_kdf(&self, key_seed: &[u8; 32], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.kdf.derive",
                json!({
                    "key_seed": BASE64.encode(key_seed),
                    "info": BASE64.encode(info),
                    "length": length
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let derived_b64 = result
            .get("derived")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto(String::from("Missing derived in tor_kdf response")))?;

        BASE64
            .decode(derived_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode derived key: {e}")))
    }

    async fn tor_cell_encrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.cell.encrypt",
                json!({
                    "key": BASE64.encode(key),
                    "counter": counter,
                    "data": BASE64.encode(data)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let ciphertext_b64 =
            result.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto(String::from("Missing ciphertext in tor_cell_encrypt response"))
            })?;

        BASE64
            .decode(ciphertext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode ciphertext: {e}")))
    }

    async fn tor_cell_decrypt(&self, key: &[u8; 32], counter: u64, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.cell.decrypt",
                json!({
                    "key": BASE64.encode(key),
                    "counter": counter,
                    "data": BASE64.encode(data)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing plaintext in tor_cell_decrypt response"))
        })?;

        BASE64
            .decode(plaintext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode plaintext: {e}")))
    }

    async fn aes_128_ctr_encrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let mut expanded_key = [0u8; 32];
        expanded_key[..16].copy_from_slice(key);
        expanded_key[16..].copy_from_slice(key);

        let counter = u64::from_be_bytes(iv[..8].try_into().unwrap_or([0u8; 8]));

        self.tor_cell_encrypt(&expanded_key, counter, data).await
    }

    async fn aes_128_ctr_decrypt(
        &self,
        key: &[u8; 16],
        iv: &[u8; 16],
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let mut expanded_key = [0u8; 32];
        expanded_key[..16].copy_from_slice(key);
        expanded_key[16..].copy_from_slice(key);

        let counter = u64::from_be_bytes(iv[..8].try_into().unwrap_or([0u8; 8]));

        self.tor_cell_decrypt(&expanded_key, counter, data).await
    }

    async fn ed25519_sign(&self, secret_key_id: &str, data: &[u8]) -> Result<[u8; 64]> {
        let result = self
            .call(
                "crypto.sign.ed25519",
                json!({
                    "secret_key_id": secret_key_id,
                    "data": BASE64.encode(data)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let sig_b64 = result.get("signature").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing signature in ed25519_sign response"))
        })?;

        let sig_bytes = BASE64
            .decode(sig_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode signature: {e}")))?;

        let mut signature = [0u8; 64];
        if sig_bytes.len() >= 64 {
            signature.copy_from_slice(&sig_bytes[..64]);
        }

        Ok(signature)
    }

    async fn ed25519_verify(
        &self,
        public_key: &[u8; 32],
        data: &[u8],
        signature: &[u8; 64],
    ) -> Result<bool> {
        let result = self
            .call(
                "crypto.verify.ed25519",
                json!({
                    "public_key": BASE64.encode(public_key),
                    "data": BASE64.encode(data),
                    "signature": BASE64.encode(signature)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        result
            .get("valid")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| Error::Crypto(String::from("Missing valid in ed25519_verify response")))
    }

    async fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair> {
        let result = self
            .call(
                "crypto.x25519.generate_ephemeral",
                json!({
                    "purpose": "tor_circuit"
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let public_b64 = result
            .get("public_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto(String::from("Missing public_key in x25519 response")))?;

        let secret_id = result.get("secret_key_id").and_then(|v| v.as_str()).unwrap_or("ephemeral");

        let secret_b64 = result.get("secret_key").and_then(|v| v.as_str());

        let public_bytes = BASE64
            .decode(public_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode public_key: {e}")))?;

        let mut public_key = [0u8; 32];
        if public_bytes.len() >= 32 {
            public_key.copy_from_slice(&public_bytes[..32]);
        }

        let mut secret_key = [0u8; 32];
        if let Some(secret_b64) = secret_b64
            && let Ok(secret_bytes) = BASE64.decode(secret_b64)
            && secret_bytes.len() >= 32
        {
            secret_key.copy_from_slice(&secret_bytes[..32]);
        }

        Ok(X25519Keypair {
            secret_key,
            secret_key_id: secret_id.to_string(),
            public_key,
        })
    }

    async fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8; 32],
        their_public_key: &[u8; 32],
    ) -> Result<[u8; 32]> {
        let result = self
            .call(
                "crypto.x25519.derive_secret",
                json!({
                    "our_secret_key": BASE64.encode(our_secret_key),
                    "their_public_key": BASE64.encode(their_public_key)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let shared_b64 = result.get("shared_secret").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing shared_secret in x25519 response"))
        })?;

        let shared_bytes = BASE64
            .decode(shared_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode shared_secret: {e}")))?;

        let mut shared_secret = [0u8; 32];
        if shared_bytes.len() >= 32 {
            shared_secret.copy_from_slice(&shared_bytes[..32]);
        }

        Ok(shared_secret)
    }

    async fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]> {
        let result = self
            .call(
                "crypto.hash.sha3_256",
                json!({
                    "data": BASE64.encode(data)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto(String::from("Missing hash in sha3_256 response")))?;

        let hash_bytes = BASE64
            .decode(hash_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode hash: {e}")))?;

        let mut hash = [0u8; 32];
        if hash_bytes.len() >= 32 {
            hash.copy_from_slice(&hash_bytes[..32]);
        }

        Ok(hash)
    }

    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<[u8; 32]> {
        let result = self
            .call(
                "crypto.hmac.sha256",
                json!({
                    "key": BASE64.encode(key),
                    "data": BASE64.encode(data)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let mac_b64 = result
            .get("mac")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Crypto(String::from("Missing mac in hmac_sha256 response")))?;

        let mac_bytes = BASE64
            .decode(mac_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode mac: {e}")))?;

        let mut mac = [0u8; 32];
        if mac_bytes.len() >= 32 {
            mac.copy_from_slice(&mac_bytes[..32]);
        }

        Ok(mac)
    }

    async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.aead.chacha20_poly1305_encrypt",
                json!({
                    "key": BASE64.encode(key),
                    "nonce": BASE64.encode(nonce),
                    "plaintext": BASE64.encode(data),
                    "aad": BASE64.encode(aad)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let ciphertext_b64 =
            result.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
                Error::Crypto(String::from("Missing ciphertext in chacha20_poly1305 response"))
            })?;

        BASE64
            .decode(ciphertext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode ciphertext: {e}")))
    }

    async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8; 32],
        nonce: &[u8; 12],
        data: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.aead.chacha20_poly1305_decrypt",
                json!({
                    "key": BASE64.encode(key),
                    "nonce": BASE64.encode(nonce),
                    "ciphertext": BASE64.encode(data),
                    "aad": BASE64.encode(aad)
                }),
            )
            .await
            .map_err(|e| map_crypto_err(&e))?;

        let plaintext_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::Crypto(String::from("Missing plaintext in chacha20_poly1305 response"))
        })?;

        BASE64
            .decode(plaintext_b64)
            .map_err(|e| Error::Crypto(format!("Failed to decode plaintext: {e}")))
    }
}

/// ntor client handshake initialization result
#[derive(Debug, Clone)]
pub struct NtorClientInit {
    /// Client's ephemeral public key (for CREATE2 payload)
    pub client_public: [u8; 32],
    /// State ID for completing handshake (security provider-managed)
    pub state_id: String,
}

/// Key material derived from ntor handshake
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    /// Forward encryption key (client -> relay)
    pub forward_key: [u8; 16],
    /// Backward encryption key (relay -> client)
    pub backward_key: [u8; 16],
    /// Forward IV
    pub forward_iv: [u8; 16],
    /// Backward IV
    pub backward_iv: [u8; 16],
}

/// X25519 keypair for ECDH
///
/// **NOTE**: For circuit building compatibility, we store both the secret
/// key ID (security provider-managed) and a copy of the raw secret.
/// In production with HSM, only the ID would be stored.
#[derive(Debug, Clone)]
pub struct X25519Keypair {
    /// Secret key (32 bytes) - for local circuit operations
    pub secret_key: [u8; 32],
    /// Secret key ID (security provider-managed) - for delegated operations
    pub secret_key_id: String,
    /// Public key (32 bytes)
    pub public_key: [u8; 32],
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::error::Error;

    #[test]
    fn test_crypto_provider_from_env() {
        let _p = CryptoProvider::from_env();
    }

    #[test]
    fn test_crypto_provider_new() {
        let p = CryptoProvider::new(String::from("/tmp/test.sock"));
        assert_eq!(p.socket_path(), "/tmp/test.sock");
    }

    #[test]
    fn ntor_client_init_clone_copies_fields() {
        let a = NtorClientInit {
            client_public: [7u8; 32],
            state_id: String::from("state-1"),
        };
        let b = a.clone();
        assert_eq!(a.client_public, b.client_public);
        assert_eq!(a.state_id, b.state_id);
    }

    #[test]
    fn key_material_clone_preserves_secrets() {
        let km = KeyMaterial {
            forward_key: [1u8; 16],
            backward_key: [2u8; 16],
            forward_iv: [3u8; 16],
            backward_iv: [4u8; 16],
        };
        let c = km.clone();
        assert_eq!(c.forward_key, km.forward_key);
        assert_eq!(c.backward_iv, km.backward_iv);
    }

    #[test]
    fn x25519_keypair_fields_accessible() {
        let kp = X25519Keypair {
            secret_key: [9u8; 32],
            secret_key_id: "id".into(),
            public_key: [8u8; 32],
        };
        assert_eq!(kp.secret_key[0], 9);
        assert_eq!(kp.public_key[0], 8);
    }

    #[test]
    fn map_crypto_err_wraps_provider_error_display() {
        use songbird_crypto_provider::{CryptoProviderError, RpcError};
        let e = CryptoProviderError::Rpc(RpcError::Remote {
            code: -1,
            message: "capability missing".into(),
        });
        let err = super::map_crypto_err(&e);
        match err {
            Error::Crypto(s) => assert!(s.contains("capability missing")),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn map_crypto_err_preserves_io_style_rpc_errors() {
        use songbird_crypto_provider::{CryptoProviderError, RpcError};
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "broken pipe");
        let e = CryptoProviderError::Rpc(RpcError::ReadResponse(inner));
        let err = super::map_crypto_err(&e);
        let Error::Crypto(s) = err else {
            panic!("expected Crypto");
        };
        assert!(!s.is_empty());
    }
}
