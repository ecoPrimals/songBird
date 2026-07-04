// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! AEAD encryption and decryption operations
//!
//! ChaCha20-Poly1305 and AES-GCM encryption/decryption for TLS 1.3.

use super::core::SecurityRpcClient;
use crate::error::{Error, Result};
use base64::prelude::*;
use serde_json::json;
use tracing::{debug, error, info, trace};

impl SecurityRpcClient {
    /// Encrypt data with ChaCha20-Poly1305 (TLS 1.3 cipher suite 0x1303)
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response is invalid.
    pub async fn encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        trace!(
            "🔐 Encrypting {} bytes via security provider (key={} bytes, nonce={} bytes, aad={} bytes)",
            plaintext.len(),
            key.len(),
            nonce.len(),
            aad.len()
        );

        let result = self
            .call(
                "crypto.encrypt",
                json!({
                    "algorithm": "chacha20-poly1305",
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await
            .map_err(|e| {
                error!("❌ crypto.encrypt RPC call failed: {}", e);
                e
            })?;

        let ciphertext = result["ciphertext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing ciphertext in response"))
        })?;

        let decoded = BASE64_STANDARD
            .decode(ciphertext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid ciphertext base64: {e}")))?;

        trace!(
            "✅ Encrypted: {} bytes plaintext → {} bytes ciphertext",
            plaintext.len(),
            decoded.len()
        );
        Ok(decoded)
    }

    /// Encrypt data with AES-128-GCM (for `TLS_AES_128_GCM_SHA256` cipher suite)
    ///
    /// # Errors
    ///
    /// Returns an error if key/nonce lengths are invalid, RPC fails, or response is invalid.
    pub async fn encrypt_aes_128_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes with AES-128-GCM via security provider", plaintext.len());

        // Validate lengths
        if key.len() != 16 {
            return Err(Error::SecurityProviderRpc(format!(
                "AES-128-GCM requires 16-byte key, got {}",
                key.len()
            )));
        }
        if nonce.len() != 12 {
            return Err(Error::SecurityProviderRpc(format!(
                "GCM nonce must be 12 bytes, got {}",
                nonce.len()
            )));
        }

        let result = self
            .call(
                "crypto.encrypt_aes_128_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        let ciphertext = result["ciphertext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing ciphertext in response"))
        })?;

        BASE64_STANDARD
            .decode(ciphertext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid ciphertext base64: {e}")))
    }

    /// Encrypt data with AES-256-GCM (for `TLS_AES_256_GCM_SHA384` cipher suite)
    ///
    /// # Errors
    ///
    /// Returns an error if key/nonce lengths are invalid, RPC fails, or response is invalid.
    pub async fn encrypt_aes_256_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes with AES-256-GCM via security provider", plaintext.len());

        // Validate lengths
        if key.len() != 32 {
            return Err(Error::SecurityProviderRpc(format!(
                "AES-256-GCM requires 32-byte key, got {}",
                key.len()
            )));
        }
        if nonce.len() != 12 {
            return Err(Error::SecurityProviderRpc(format!(
                "GCM nonce must be 12 bytes, got {}",
                nonce.len()
            )));
        }

        let result = self
            .call(
                "crypto.encrypt_aes_256_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        let ciphertext = result["ciphertext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing ciphertext in response"))
        })?;

        BASE64_STANDARD
            .decode(ciphertext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid ciphertext base64: {e}")))
    }

    /// Decrypt data with ChaCha20-Poly1305 (TLS 1.3 cipher suite 0x1303)
    ///
    /// NOTE: ChaCha20-Poly1305 RPC expects SEPARATE ciphertext and tag parameters!
    ///
    /// # Errors
    ///
    /// Returns an error if ciphertext is too short, RPC fails, or response is invalid.
    pub async fn decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        info!("🔓 Security provider crypto.decrypt: {} bytes ciphertext+tag", ciphertext.len());
        debug!(
            "  Key: {} bytes, Nonce: {} bytes, AAD: {} bytes",
            key.len(),
            nonce.len(),
            aad.len()
        );

        // ChaCha20-Poly1305 AEAD: Last 16 bytes are the authentication tag
        if ciphertext.len() < 16 {
            return Err(Error::SecurityProviderRpc(
                "Ciphertext too short for ChaCha20-Poly1305 (need at least 16 bytes for tag)"
                    .to_string(),
            ));
        }

        let (actual_ciphertext, tag) = ciphertext.split_at(ciphertext.len() - 16);
        debug!("  Split: {} bytes ciphertext + 16 bytes tag", actual_ciphertext.len());

        let result = self
            .call(
                "crypto.decrypt",
                json!({
                    "algorithm": "chacha20-poly1305",
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(actual_ciphertext),
                    "tag": BASE64_STANDARD.encode(tag),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await
            .map_err(|e| {
                error!("❌ crypto.decrypt failed: {}", e);
                e
            })?;

        let plaintext = result["plaintext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing plaintext in response"))
        })?;

        let decoded = BASE64_STANDARD
            .decode(plaintext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid plaintext base64: {e}")))?;

        info!("✅ Decrypted: {} bytes → {} bytes", ciphertext.len(), decoded.len());
        Ok(decoded)
    }

    /// Decrypt data with AES-128-GCM (for `TLS_AES_128_GCM_SHA256` cipher suite)
    ///
    /// # Errors
    ///
    /// Returns an error if lengths are invalid, RPC fails, or response is invalid.
    pub async fn decrypt_aes_128_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        info!(
            "🔓 Security provider AES-128-GCM decrypt: {} bytes ciphertext+tag",
            ciphertext.len()
        );

        // Validate lengths
        if ciphertext.len() < 16 {
            return Err(Error::SecurityProviderRpc(String::from(
                "Ciphertext too short for AES-128-GCM",
            )));
        }
        if key.len() != 16 {
            return Err(Error::SecurityProviderRpc(format!(
                "AES-128-GCM requires 16-byte key, got {}",
                key.len()
            )));
        }
        if nonce.len() != 12 {
            return Err(Error::SecurityProviderRpc(format!(
                "GCM nonce must be 12 bytes, got {}",
                nonce.len()
            )));
        }

        // CRITICAL: Pass FULL ciphertext (NOT splitting tag)
        // The provider's aes-gcm integration expects [encrypted_data] + [16-byte tag]
        let result = self
            .call(
                "crypto.decrypt_aes_128_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext), // FULL with tag!
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        let plaintext = result["plaintext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing plaintext in response"))
        })?;

        BASE64_STANDARD
            .decode(plaintext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid plaintext base64: {e}")))
    }

    /// Decrypt data with AES-256-GCM (for `TLS_AES_256_GCM_SHA384` cipher suite)
    ///
    /// # Errors
    ///
    /// Returns an error if lengths are invalid, RPC fails, or response is invalid.
    pub async fn decrypt_aes_256_gcm(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        info!(
            "🔓 Security provider AES-256-GCM decrypt: {} bytes ciphertext+tag",
            ciphertext.len()
        );

        // Validate lengths
        if ciphertext.len() < 16 {
            return Err(Error::SecurityProviderRpc(String::from(
                "Ciphertext too short for AES-256-GCM",
            )));
        }
        if key.len() != 32 {
            return Err(Error::SecurityProviderRpc(format!(
                "AES-256-GCM requires 32-byte key, got {}",
                key.len()
            )));
        }
        if nonce.len() != 12 {
            return Err(Error::SecurityProviderRpc(format!(
                "GCM nonce must be 12 bytes, got {}",
                nonce.len()
            )));
        }

        // CRITICAL: Pass FULL ciphertext (NOT splitting tag)
        let result = self
            .call(
                "crypto.decrypt_aes_256_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext), // FULL with tag!
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        let plaintext = result["plaintext"].as_str().ok_or_else(|| {
            Error::SecurityProviderRpc(String::from("Missing plaintext in response"))
        })?;

        BASE64_STANDARD
            .decode(plaintext)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid plaintext base64: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use crate::SecurityRpcClient;
    use crate::error::Error;

    #[test]
    fn test_aes_key_sizes() {
        // AES-128 = 16 bytes
        // AES-256 = 32 bytes
        // GCM nonce = 12 bytes
        // GCM tag = 16 bytes
        assert_eq!(128 / 8, 16);
        assert_eq!(256 / 8, 32);
    }

    #[tokio::test]
    async fn encrypt_aes_128_gcm_rejects_bad_key_length() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .encrypt_aes_128_gcm(&[0u8; 15], &[0u8; 12], b"pt", b"aad")
            .await
            .expect_err("wrong key length");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("16-byte key")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn encrypt_aes_128_gcm_rejects_bad_nonce_length() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .encrypt_aes_128_gcm(&[0u8; 16], &[0u8; 11], b"pt", b"aad")
            .await
            .expect_err("wrong nonce length");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("12 bytes")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn encrypt_aes_256_gcm_rejects_bad_key_length() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .encrypt_aes_256_gcm(&[0u8; 31], &[0u8; 12], b"pt", b"aad")
            .await
            .expect_err("wrong key length");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("32-byte key")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn decrypt_chacha_rejects_short_ciphertext() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .decrypt(&[0u8; 32], &[0u8; 12], &[0u8; 10], b"aad")
            .await
            .expect_err("ciphertext too short");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("too short")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn decrypt_aes_128_gcm_rejects_short_ciphertext_before_rpc() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .decrypt_aes_128_gcm(&[0u8; 16], &[0u8; 12], &[0u8; 10], b"aad")
            .await
            .expect_err("ciphertext too short");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("too short")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn decrypt_aes_128_gcm_rejects_bad_key_length() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .decrypt_aes_128_gcm(&[0u8; 15], &[0u8; 12], &[0u8; 20], b"aad")
            .await
            .expect_err("wrong key length");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("16-byte key")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn decrypt_aes_256_gcm_rejects_bad_nonce_length() {
        let client = SecurityRpcClient::new_direct("/tmp/songbird-aead-test.sock");
        let err = client
            .decrypt_aes_256_gcm(&[0u8; 32], &[0u8; 11], &[0u8; 32], b"aad")
            .await
            .expect_err("wrong nonce length");
        match err {
            Error::SecurityProviderRpc(msg) => assert!(msg.contains("12 bytes")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
