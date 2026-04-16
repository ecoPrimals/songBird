// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC crypto provider — bridges QUIC packet protection to `security provider`.
//!
//! All cryptographic operations needed by the QUIC transport are implemented on
//! [`SecurityQuicCrypto`], delegating to `security provider` via JSON-RPC IPC, following
//! the same Tower Atomic pattern as `songbird-tls` and `songbird-http-client`.

use crate::error::{QuicError, Result};

/// Cipher suite identifiers used by QUIC (RFC 9001 Section 5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum QuicCipherSuite {
    /// `TLS_AES_128_GCM_SHA256` (`0x1301`)
    Aes128Gcm = 0x1301,
    /// `TLS_AES_256_GCM_SHA384` (`0x1302`)
    Aes256Gcm = 0x1302,
    /// `TLS_CHACHA20_POLY1305_SHA256` (`0x1303`)
    ChaCha20Poly1305 = 0x1303,
}

impl QuicCipherSuite {
    /// AEAD key length in bytes.
    #[must_use]
    pub const fn key_len(self) -> usize {
        match self {
            Self::Aes128Gcm => 16,
            Self::Aes256Gcm | Self::ChaCha20Poly1305 => 32,
        }
    }

    /// AEAD nonce/IV length in bytes (always 12 for all QUIC cipher suites).
    #[must_use]
    pub const fn iv_len(self) -> usize {
        12
    }

    /// AEAD authentication tag length (always 16).
    #[must_use]
    pub const fn tag_len(self) -> usize {
        16
    }

    /// Hash output length for the associated hash function.
    #[must_use]
    pub const fn hash_len(self) -> usize {
        match self {
            Self::Aes128Gcm | Self::ChaCha20Poly1305 => 32,
            Self::Aes256Gcm => 48,
        }
    }

    /// Header protection sample length (always 16 bytes).
    #[must_use]
    pub const fn hp_sample_len(self) -> usize {
        16
    }

    /// Header protection key length.
    #[must_use]
    pub const fn hp_key_len(self) -> usize {
        self.key_len()
    }
}

/// `security provider`-backed QUIC crypto provider.
///
/// Delegates all crypto operations to `security provider` via `songbird-crypto-provider`
/// JSON-RPC. Uses the same socket discovery and routing as `songbird-tls`.
#[derive(Debug)]
pub struct SecurityQuicCrypto {
    provider: songbird_crypto_provider::CryptoProvider,
}

impl SecurityQuicCrypto {
    /// Create from a `songbird-crypto-provider` instance.
    #[must_use]
    pub const fn new(provider: songbird_crypto_provider::CryptoProvider) -> Self {
        Self {
            provider,
        }
    }

    /// Create using default socket discovery (same as songbird-tls / songbird-http-client).
    #[must_use]
    pub fn discover() -> Self {
        let provider = songbird_crypto_provider::CryptoProvider::from_env();
        Self {
            provider,
        }
    }

    /// HKDF-Extract (`salt`, `ikm`) via security provider.
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "salt": base64_encode(salt),
            "ikm": base64_encode(ikm),
        });
        let result = self
            .provider
            .call("crypto.hkdf_extract", params)
            .await
            .map_err(|e| QuicError::Crypto(format!("hkdf_extract: {e}")))?;
        decode_base64_field(&result, "prk")
    }

    /// HKDF-Expand to `length` bytes (`prk`, `info`).
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "prk": base64_encode(prk),
            "info": base64_encode(info),
            "length": length,
        });
        let result = self
            .provider
            .call("crypto.hkdf_expand", params)
            .await
            .map_err(|e| QuicError::Crypto(format!("hkdf_expand: {e}")))?;
        decode_base64_field(&result, "okm")
    }

    /// SHA-256 digest.
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({ "data": base64_encode(data) });
        let result = self
            .provider
            .call("crypto.sha256", params)
            .await
            .map_err(|e| QuicError::Crypto(format!("sha256: {e}")))?;
        decode_base64_field(&result, "hash")
    }

    /// SHA-384 digest.
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>> {
        let params = serde_json::json!({ "data": base64_encode(data) });
        let result = self
            .provider
            .call("crypto.sha384", params)
            .await
            .map_err(|e| QuicError::Crypto(format!("sha384: {e}")))?;
        decode_base64_field(&result, "hash")
    }

    /// AEAD encrypt for QUIC (`suite`, `aad`).
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn aead_encrypt(
        &self,
        suite: QuicCipherSuite,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let method = match suite {
            QuicCipherSuite::Aes128Gcm => "crypto.encrypt_aes_128_gcm",
            QuicCipherSuite::Aes256Gcm => "crypto.encrypt_aes_256_gcm",
            QuicCipherSuite::ChaCha20Poly1305 => "crypto.encrypt_chacha20_poly1305",
        };
        let params = serde_json::json!({
            "key": base64_encode(key),
            "nonce": base64_encode(nonce),
            "plaintext": base64_encode(plaintext),
            "aad": base64_encode(aad),
        });
        let result = self
            .provider
            .call(method, params)
            .await
            .map_err(|e| QuicError::Crypto(format!("aead_encrypt: {e}")))?;
        decode_base64_field(&result, "ciphertext")
    }

    /// AEAD decrypt for QUIC (`suite`, `aad`).
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn aead_decrypt(
        &self,
        suite: QuicCipherSuite,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let method = match suite {
            QuicCipherSuite::Aes128Gcm => "crypto.decrypt_aes_128_gcm",
            QuicCipherSuite::Aes256Gcm => "crypto.decrypt_aes_256_gcm",
            QuicCipherSuite::ChaCha20Poly1305 => "crypto.decrypt_chacha20_poly1305",
        };
        let params = serde_json::json!({
            "key": base64_encode(key),
            "nonce": base64_encode(nonce),
            "ciphertext": base64_encode(ciphertext),
            "aad": base64_encode(aad),
        });
        let result = self
            .provider
            .call(method, params)
            .await
            .map_err(|e| QuicError::Crypto(format!("aead_decrypt: {e}")))?;
        decode_base64_field(&result, "plaintext")
    }

    /// RFC 9001 header protection mask (5 bytes) from `sample`.
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn header_protection_mask(
        &self,
        suite: QuicCipherSuite,
        hp_key: &[u8],
        sample: &[u8],
    ) -> Result<[u8; 5]> {
        let method = match suite {
            QuicCipherSuite::Aes128Gcm | QuicCipherSuite::Aes256Gcm => "crypto.aes_ecb_encrypt",
            QuicCipherSuite::ChaCha20Poly1305 => "crypto.chacha20_block",
        };
        let params = match suite {
            QuicCipherSuite::Aes128Gcm | QuicCipherSuite::Aes256Gcm => {
                serde_json::json!({
                    "key": base64_encode(hp_key),
                    "block": base64_encode(sample),
                })
            }
            QuicCipherSuite::ChaCha20Poly1305 => {
                // ChaCha20 HP: counter = sample[0..4] LE, nonce = sample[4..16]
                serde_json::json!({
                    "key": base64_encode(hp_key),
                    "counter": base64_encode(&sample[..4]),
                    "nonce": base64_encode(&sample[4..16]),
                })
            }
        };
        let result = self
            .provider
            .call(method, params)
            .await
            .map_err(|e| QuicError::Crypto(format!("header_protection_mask: {e}")))?;
        let output = decode_base64_field(&result, "output")?;
        if output.len() < 5 {
            return Err(QuicError::Crypto("HP mask output too short".into()));
        }
        let mut mask = [0u8; 5];
        mask.copy_from_slice(&output[..5]);
        Ok(mask)
    }

    /// Generate an X25519 keypair (public, secret).
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn generate_x25519_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let result = self
            .provider
            .call("crypto.generate_keypair", serde_json::json!({}))
            .await
            .map_err(|e| QuicError::Crypto(format!("generate_keypair: {e}")))?;
        let public = decode_base64_field_multi(&result, &["public_key", "public"])?;
        let secret = decode_base64_field_multi(&result, &["secret_key", "private_key", "secret"])?;
        Ok((public, secret))
    }

    /// ECDH shared secret from our secret key and peer public key.
    ///
    /// # Errors
    ///
    /// Returns `QuicError::Crypto` if the RPC call fails.
    pub async fn derive_x25519_shared_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>> {
        let params = serde_json::json!({
            "our_secret": base64_encode(our_secret),
            "their_public": base64_encode(their_public),
        });
        let result = self
            .provider
            .call("crypto.ecdh_derive", params)
            .await
            .map_err(|e| QuicError::Crypto(format!("ecdh_derive: {e}")))?;
        decode_base64_field(&result, "shared_secret")
    }
}

/// Backward-compatible name for [`SecurityQuicCrypto`] (single implementation; no trait objects).
pub type QuicCryptoProvider = SecurityQuicCrypto;

use base64::Engine;

fn base64_encode(data: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn decode_base64_field(value: &serde_json::Value, field: &str) -> Result<Vec<u8>> {
    let s = value[field]
        .as_str()
        .ok_or_else(|| QuicError::Crypto(format!("Missing field '{field}' in response")))?;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| QuicError::Crypto(format!("Base64 decode '{field}': {e}")))
}

fn decode_base64_field_multi(value: &serde_json::Value, fields: &[&str]) -> Result<Vec<u8>> {
    for field in fields {
        if let Some(s) = value[field].as_str() {
            return base64::engine::general_purpose::STANDARD
                .decode(s)
                .map_err(|e| QuicError::Crypto(format!("Base64 decode '{field}': {e}")));
        }
    }
    Err(QuicError::Crypto(format!("None of fields {fields:?} found in response")))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn cipher_suite_properties() {
        assert_eq!(QuicCipherSuite::Aes128Gcm.key_len(), 16);
        assert_eq!(QuicCipherSuite::Aes256Gcm.key_len(), 32);
        assert_eq!(QuicCipherSuite::ChaCha20Poly1305.key_len(), 32);

        assert_eq!(QuicCipherSuite::Aes128Gcm.iv_len(), 12);
        assert_eq!(QuicCipherSuite::Aes256Gcm.iv_len(), 12);
        assert_eq!(QuicCipherSuite::ChaCha20Poly1305.iv_len(), 12);

        assert_eq!(QuicCipherSuite::Aes128Gcm.tag_len(), 16);
        assert_eq!(QuicCipherSuite::Aes128Gcm.hp_sample_len(), 16);

        assert_eq!(QuicCipherSuite::Aes128Gcm.hash_len(), 32);
        assert_eq!(QuicCipherSuite::Aes256Gcm.hash_len(), 48);
        assert_eq!(QuicCipherSuite::ChaCha20Poly1305.hash_len(), 32);

        assert_eq!(QuicCipherSuite::Aes128Gcm.hp_key_len(), 16);
        assert_eq!(QuicCipherSuite::ChaCha20Poly1305.hp_key_len(), 32);
    }

    #[test]
    fn cipher_suite_tls_assigned_numbers_match_rfc_9001() {
        assert_eq!(QuicCipherSuite::Aes128Gcm as u16, 0x1301, "AES-128-GCM-SHA256");
        assert_eq!(QuicCipherSuite::Aes256Gcm as u16, 0x1302, "AES-256-GCM-SHA384");
        assert_eq!(QuicCipherSuite::ChaCha20Poly1305 as u16, 0x1303, "CHACHA20-POLY1305-SHA256");
    }

    #[test]
    fn security_quic_crypto_new_and_discover_construct() {
        let provider = songbird_crypto_provider::CryptoProvider::from_env();
        let _via_new = SecurityQuicCrypto::new(provider);
        let _via_discover = SecurityQuicCrypto::discover();
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn decode_base64_field_errors_on_missing_key() {
        let v = serde_json::json!({ "other": "YQ==" });
        let e = decode_base64_field(&v, "prk").expect_err("missing prk");
        assert!(
            e.to_string().contains("prk") || e.to_string().contains("Missing"),
            "unexpected: {e}"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn decode_base64_field_errors_on_invalid_base64() {
        let v = serde_json::json!({ "hash": "@@@not-base64@@@" });
        let e = decode_base64_field(&v, "hash").expect_err("invalid base64");
        assert!(
            e.to_string().contains("Base64") || e.to_string().contains("hash"),
            "unexpected: {e}"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn decode_base64_field_multi_errors_when_no_candidate_present() {
        let v = serde_json::json!({ "foo": "YQ==" });
        let e = decode_base64_field_multi(&v, &["public_key", "public"]).expect_err("no fields");
        assert!(e.to_string().contains("None of fields"), "unexpected: {e}");
    }
}
