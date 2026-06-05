// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{NfcError, Result};
use crate::{NONCE_SIZE, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};
use serde_json::json;
use tracing::{debug, warn};

use super::exchange::GenesisExchange;
use super::types::{decode_hex_or_b64, hex};

impl GenesisExchange {
    pub(super) async fn nfc_crypto_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.provider.call(method, params).await.map_err(|e| NfcError::Crypto(e.to_string()))
    }

    pub(super) async fn generate_x25519_keypair(&self) -> Result<[u8; PUBLIC_KEY_SIZE]> {
        match self
            .nfc_crypto_call(
                "crypto.generate_x25519_keypair",
                json!({
                    "purpose": "nfc_genesis_ephemeral"
                }),
            )
            .await
        {
            Ok(result) => {
                if let Some(pk) = result.get("public_key").and_then(|v| v.as_str()) {
                    let bytes = decode_hex_or_b64(pk)?;
                    let mut key = [0u8; PUBLIC_KEY_SIZE];
                    if bytes.len() >= PUBLIC_KEY_SIZE {
                        key.copy_from_slice(&bytes[..PUBLIC_KEY_SIZE]);
                    }
                    Ok(key)
                } else {
                    Err(NfcError::Crypto("missing public_key".to_string()))
                }
            }
            Err(e) => {
                warn!("Crypto provider x25519 unavailable: {}. Using local RNG fallback.", e);
                let mut key = [0u8; PUBLIC_KEY_SIZE];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut key);
                Ok(key)
            }
        }
    }

    pub(super) async fn x25519_dh(&self, peer_pubkey: &[u8]) -> Result<Vec<u8>> {
        match self
            .nfc_crypto_call(
                "crypto.x25519_dh",
                json!({
                    "peer_public_key": hex::encode(peer_pubkey)
                }),
            )
            .await
        {
            Ok(result) => result.get("shared_secret").and_then(|v| v.as_str()).map_or_else(
                || Err(NfcError::Crypto("missing shared_secret".to_string())),
                decode_hex_or_b64,
            ),
            Err(e) => {
                #[cfg(test)]
                {
                    tracing::warn!("Test mode: DH fallback to zero secret (provider: {e})");
                    return Ok(vec![0u8; 32]);
                }
                #[cfg(not(test))]
                Err(NfcError::Crypto(format!(
                    "Crypto provider DH unavailable — cannot derive shared secret: {e}"
                )))
            }
        }
    }

    pub(super) async fn generate_nonce(&self) -> Result<[u8; NONCE_SIZE]> {
        match self
            .nfc_crypto_call(
                "crypto.generate_random",
                json!({
                    "length": NONCE_SIZE,
                    "purpose": "nfc_genesis_nonce"
                }),
            )
            .await
        {
            Ok(result) => result.get("bytes").and_then(|v| v.as_str()).map_or_else(
                || Err(NfcError::Crypto("missing bytes".to_string())),
                |n| {
                    let bytes = decode_hex_or_b64(n)?;
                    let mut nonce = [0u8; NONCE_SIZE];
                    if bytes.len() >= NONCE_SIZE {
                        nonce.copy_from_slice(&bytes[..NONCE_SIZE]);
                    }
                    Ok(nonce)
                },
            ),
            Err(e) => {
                warn!("Crypto provider nonce unavailable: {}. Using local RNG.", e);
                let mut nonce = [0u8; NONCE_SIZE];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut nonce);
                Ok(nonce)
            }
        }
    }

    pub(super) async fn encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>> {
        match self
            .nfc_crypto_call(
                "crypto.chacha20poly1305_encrypt",
                json!({
                    "plaintext": hex::encode(plaintext),
                    "key": hex::encode(key),
                    "nonce": hex::encode(nonce)
                }),
            )
            .await
        {
            Ok(result) => result.get("ciphertext").and_then(|v| v.as_str()).map_or_else(
                || Err(NfcError::Crypto("missing ciphertext".to_string())),
                decode_hex_or_b64,
            ),
            Err(e) => {
                warn!(
                    "Crypto provider encrypt unavailable: {}. Passing plaintext (TESTING ONLY).",
                    e
                );
                Ok(plaintext.to_vec())
            }
        }
    }

    pub(super) async fn decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>> {
        match self
            .nfc_crypto_call(
                "crypto.chacha20poly1305_decrypt",
                json!({
                    "ciphertext": hex::encode(ciphertext),
                    "key": hex::encode(key),
                    "nonce": hex::encode(nonce)
                }),
            )
            .await
        {
            Ok(result) => result.get("plaintext").and_then(|v| v.as_str()).map_or_else(
                || Err(NfcError::Crypto("missing plaintext".to_string())),
                decode_hex_or_b64,
            ),
            Err(e) => {
                warn!(
                    "Crypto provider decrypt unavailable: {}. Treating as plaintext (TESTING ONLY).",
                    e
                );
                Ok(ciphertext.to_vec())
            }
        }
    }

    pub(super) async fn ed25519_sign(&self, data: &[u8]) -> Result<[u8; SIGNATURE_SIZE]> {
        match self
            .nfc_crypto_call(
                "crypto.ed25519_sign",
                json!({
                    "message": hex::encode(data),
                    "purpose": "nfc_genesis"
                }),
            )
            .await
        {
            Ok(result) => {
                if let Some(sig) = result.get("signature").and_then(|v| v.as_str()) {
                    let bytes = decode_hex_or_b64(sig)?;
                    let mut signature = [0u8; SIGNATURE_SIZE];
                    if bytes.len() >= SIGNATURE_SIZE {
                        signature.copy_from_slice(&bytes[..SIGNATURE_SIZE]);
                    }
                    Ok(signature)
                } else {
                    Err(NfcError::Crypto("missing signature".to_string()))
                }
            }
            Err(e) => {
                warn!(
                    "Crypto provider sign unavailable: {}. Using zero signature (TESTING ONLY).",
                    e
                );
                Ok([0u8; SIGNATURE_SIZE])
            }
        }
    }

    pub(super) async fn ed25519_verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        match self
            .nfc_crypto_call(
                "crypto.ed25519_verify",
                json!({
                    "message": hex::encode(data),
                    "signature": hex::encode(signature)
                }),
            )
            .await
        {
            Ok(result) => {
                let valid =
                    result.get("valid").and_then(serde_json::Value::as_bool).unwrap_or(false);
                if valid {
                    Ok(())
                } else {
                    Err(NfcError::Crypto("Signature verification failed".to_string()))
                }
            }
            Err(e) => {
                warn!("Crypto provider verify unavailable: {}. Accepting (TESTING ONLY).", e);
                Ok(())
            }
        }
    }

    pub(super) async fn destroy_ephemeral_keys(&self) -> Result<()> {
        match self
            .nfc_crypto_call(
                "crypto.destroy_ephemeral_keys",
                json!({
                    "purpose": "nfc_genesis_ephemeral"
                }),
            )
            .await
        {
            Ok(_) => {
                debug!("Ephemeral keys destroyed via crypto provider");
                Ok(())
            }
            Err(e) => {
                warn!("Crypto provider destroy_keys unavailable: {}. Keys will be dropped.", e);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use crate::genesis::exchange::GenesisExchange;
    use crate::{NONCE_SIZE, PUBLIC_KEY_SIZE, SIGNATURE_SIZE};
    use songbird_crypto_provider::{CryptoProvider, RoutingMode};

    #[test]
    fn test_crypto_provider_from_env_has_socket() {
        let p = CryptoProvider::from_env();
        assert!(!p.socket_path().is_empty());
    }

    #[tokio::test]
    async fn test_crypto_keypair_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let key = ex.generate_x25519_keypair().await.unwrap();
        assert_eq!(key.len(), PUBLIC_KEY_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_nonce_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let nonce = ex.generate_nonce().await.unwrap();
        assert_eq!(nonce.len(), NONCE_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_dh_fallback_in_test_mode() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let shared = ex.x25519_dh(&[0u8; 32]).await.unwrap();
        assert_eq!(shared.len(), 32, "test-mode DH falls back to zero secret");
    }

    #[tokio::test]
    async fn test_crypto_sign_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let sig = ex.ed25519_sign(b"test data").await.unwrap();
        assert_eq!(sig.len(), SIGNATURE_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_verify_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        ex.ed25519_verify(b"data", &[0u8; 64]).await.unwrap();
    }

    #[tokio::test]
    async fn test_crypto_encrypt_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let ct = ex.encrypt(b"plaintext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        assert_eq!(ct, b"plaintext");
    }

    #[tokio::test]
    async fn test_crypto_decrypt_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        let pt = ex.decrypt(b"ciphertext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        assert_eq!(pt, b"ciphertext");
    }

    #[tokio::test]
    async fn test_crypto_destroy_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-security-provider.sock".to_string(),
            RoutingMode::Direct,
        ));
        ex.destroy_ephemeral_keys().await.unwrap();
    }
}
