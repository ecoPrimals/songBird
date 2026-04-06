// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CryptoCapability` trait implementation and JSON response field extraction.

use async_trait::async_trait;
use base64::prelude::*;
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use super::SecurityCryptoProvider;
use crate::crypto::capability::{CryptoCapability, TlsApplicationSecrets, TlsHandshakeSecrets};
use crate::error::{Error, Result};

#[async_trait]
impl CryptoCapability for SecurityCryptoProvider {
    fn name(&self) -> &'static str {
        "security provider"
    }

    async fn is_available(&self) -> bool {
        match self
            .call(
                "crypto.sha256",
                json!({
                    "data": BASE64_STANDARD.encode(b"ping")
                }),
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                warn!("Security provider availability check failed: {}", e);
                false
            }
        }
    }

    async fn generate_x25519_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let result = self.call("crypto.generate_keypair", json!({})).await?;

        let public_b64 = result
            .get("public_key")
            .or_else(|| result.get("public"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::SecurityProviderRpc("Missing public_key in response".to_string())
            })?;

        let private_b64 = result
            .get("secret_key")
            .or_else(|| result.get("private_key"))
            .or_else(|| result.get("secret"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::SecurityProviderRpc("Missing secret_key/private_key in response".to_string())
            })?;

        let public = BASE64_STANDARD
            .decode(public_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 public key: {e}")))?;
        let private = BASE64_STANDARD
            .decode(private_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 private key: {e}")))?;

        debug!(
            "Generated X25519 keypair: {} bytes public, {} bytes private",
            public.len(),
            private.len()
        );

        Ok((public, private))
    }

    async fn derive_x25519_shared_secret(
        &self,
        our_secret: &[u8],
        their_public: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.ecdh_derive",
                json!({
                    "our_secret": BASE64_STANDARD.encode(our_secret),
                    "their_public": BASE64_STANDARD.encode(their_public)
                }),
            )
            .await?;

        let shared_b64 = result.get("shared_secret").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::SecurityProviderRpc("Missing shared_secret in response".to_string())
        })?;

        let shared = BASE64_STANDARD.decode(shared_b64).map_err(|e| {
            Error::SecurityProviderRpc(format!("Invalid base64 shared secret: {e}"))
        })?;

        debug!("Derived shared secret: {} bytes", shared.len());

        Ok(shared)
    }

    async fn aes128_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
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

        Self::extract_ciphertext(&result)
    }

    async fn aes128_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_aes_128_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        Self::extract_plaintext(&result)
    }

    async fn aes256_gcm_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
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

        Self::extract_ciphertext(&result)
    }

    async fn aes256_gcm_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_aes_256_gcm",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        Self::extract_plaintext(&result)
    }

    async fn chacha20_poly1305_encrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.encrypt_chacha20_poly1305",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "plaintext": BASE64_STANDARD.encode(plaintext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        Self::extract_ciphertext(&result)
    }

    async fn chacha20_poly1305_decrypt(
        &self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        aad: &[u8],
    ) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.decrypt_chacha20_poly1305",
                json!({
                    "key": BASE64_STANDARD.encode(key),
                    "nonce": BASE64_STANDARD.encode(nonce),
                    "ciphertext": BASE64_STANDARD.encode(ciphertext),
                    "aad": BASE64_STANDARD.encode(aad)
                }),
            )
            .await?;

        Self::extract_plaintext(&result)
    }

    async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha256",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc("Missing hash in response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 hash: {e}")))
    }

    async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.sha384",
                json!({
                    "data": BASE64_STANDARD.encode(data)
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc("Missing hash in response".to_string()))?;

        BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 hash: {e}")))
    }

    async fn hash_for_cipher(&self, data: &[u8], cipher_suite: u16) -> Result<Vec<u8>> {
        debug!(
            "🔐 hash_for_cipher: cipher_suite=0x{:04x}, data={} bytes",
            cipher_suite,
            data.len()
        );

        let result = self
            .call(
                "crypto.hash_for_cipher",
                json!({
                    "data": BASE64_STANDARD.encode(data),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        let hash_b64 = result
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc("Missing hash in response".to_string()))?;

        let hash = BASE64_STANDARD
            .decode(hash_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 hash: {e}")))?;

        let algorithm = result.get("algorithm").and_then(|v| v.as_str()).unwrap_or("unknown");
        debug!("  → algorithm={}, hash_length={} bytes", algorithm, hash.len());

        Ok(hash)
    }

    async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_extract",
                json!({
                    "salt": BASE64_STANDARD.encode(salt),
                    "ikm": BASE64_STANDARD.encode(ikm)
                }),
            )
            .await?;

        let prk_b64 = result
            .get("prk")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc("Missing prk in response".to_string()))?;

        BASE64_STANDARD
            .decode(prk_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 prk: {e}")))
    }

    async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let result = self
            .call(
                "crypto.hkdf_expand",
                json!({
                    "prk": BASE64_STANDARD.encode(prk),
                    "info": BASE64_STANDARD.encode(info),
                    "length": length
                }),
            )
            .await?;

        let okm_b64 = result
            .get("okm")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc("Missing okm in response".to_string()))?;

        BASE64_STANDARD
            .decode(okm_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 okm: {e}")))
    }

    async fn tls_derive_handshake_secrets(
        &self,
        shared_secret: &[u8],
        client_random: &[u8],
        server_random: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsHandshakeSecrets> {
        info!("🔑 Deriving TLS 1.3 handshake secrets (RFC 8446 Section 7.1)");
        debug!("  → pre_master_secret: {} bytes", shared_secret.len());
        debug!("  → client_random: {} bytes", client_random.len());
        debug!("  → server_random: {} bytes", server_random.len());
        debug!("  → transcript_hash: {} bytes", transcript_hash.len());
        debug!("  → cipher_suite: 0x{:04x}", cipher_suite);

        let result = self
            .call(
                "tls.derive_handshake_secrets",
                json!({
                    "pre_master_secret": BASE64_STANDARD.encode(shared_secret),
                    "client_random": BASE64_STANDARD.encode(client_random),
                    "server_random": BASE64_STANDARD.encode(server_random),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        Ok(TlsHandshakeSecrets {
            client_handshake_secret: Self::extract_b64_field(&result, "client_handshake_secret")?,
            server_handshake_secret: Self::extract_b64_field(&result, "server_handshake_secret")?,
            client_write_key: Self::extract_b64_field(&result, "client_write_key")?,
            client_write_iv: Self::extract_b64_field(&result, "client_write_iv")?,
            server_write_key: Self::extract_b64_field(&result, "server_write_key")?,
            server_write_iv: Self::extract_b64_field(&result, "server_write_iv")?,
            handshake_secret: Self::extract_b64_field(&result, "handshake_secret")?,
        })
    }

    async fn tls_derive_application_secrets(
        &self,
        handshake_secret: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<TlsApplicationSecrets> {
        info!("🔑 Deriving TLS 1.3 application secrets (cipher suite: 0x{:04x})", cipher_suite);
        let result = self
            .call(
                "tls.derive_application_secrets",
                json!({
                    "handshake_secret": BASE64_STANDARD.encode(handshake_secret),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        Ok(TlsApplicationSecrets {
            client_traffic_secret: Self::extract_b64_field(&result, "client_application_secret")?,
            server_traffic_secret: Self::extract_b64_field(&result, "server_application_secret")?,
            client_write_key: Self::extract_b64_field(&result, "client_write_key")?,
            client_write_iv: Self::extract_b64_field(&result, "client_write_iv")?,
            server_write_key: Self::extract_b64_field(&result, "server_write_key")?,
            server_write_iv: Self::extract_b64_field(&result, "server_write_iv")?,
        })
    }

    async fn tls_compute_finished_verify_data(
        &self,
        base_key: &[u8],
        transcript_hash: &[u8],
        cipher_suite: u16,
    ) -> Result<Vec<u8>> {
        debug!(
            "🔐 tls_compute_finished_verify_data: cipher=0x{:04x}, hash={} bytes",
            cipher_suite,
            transcript_hash.len()
        );

        let result = self
            .call(
                "tls.compute_finished_verify_data",
                json!({
                    "base_key": BASE64_STANDARD.encode(base_key),
                    "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
                    "cipher_suite": cipher_suite
                }),
            )
            .await?;

        Self::extract_b64_field(&result, "verify_data")
    }
}

impl SecurityCryptoProvider {
    fn extract_ciphertext(result: &Value) -> Result<Vec<u8>> {
        let ct_b64 = result.get("ciphertext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::SecurityProviderRpc("Missing ciphertext in response".to_string())
        })?;

        BASE64_STANDARD
            .decode(ct_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 ciphertext: {e}")))
    }

    fn extract_plaintext(result: &Value) -> Result<Vec<u8>> {
        let pt_b64 = result.get("plaintext").and_then(|v| v.as_str()).ok_or_else(|| {
            Error::SecurityProviderRpc("Missing plaintext in response".to_string())
        })?;

        BASE64_STANDARD
            .decode(pt_b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 plaintext: {e}")))
    }

    fn extract_b64_field(result: &Value, field: &str) -> Result<Vec<u8>> {
        let b64 = result
            .get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::SecurityProviderRpc(format!("Missing {field} in response")))?;

        BASE64_STANDARD
            .decode(b64)
            .map_err(|e| Error::SecurityProviderRpc(format!("Invalid base64 {field}: {e}")))
    }
}
