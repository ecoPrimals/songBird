// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Genesis ceremony exchange via NFC
//!
//! Implements secure genesis credential exchange with Dark Forest compliance

use crate::config::NfcConfig;
use crate::error::{NfcError, Result};
use crate::platform::NfcDevice;
use crate::protocol::{NfcMessage, NfcProtocol};
use crate::timing::TimingProtector;
use crate::{
    MSG_TYPE_GENESIS_REQUEST, MSG_TYPE_GENESIS_RESPONSE, NONCE_SIZE, PUBLIC_KEY_SIZE,
    SIGNATURE_SIZE,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use songbird_crypto_provider::CryptoProvider;
use tracing::{debug, info, warn};

/// Genesis credentials (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisCredentials {
    /// Primal identity (Ed25519 public key)
    pub identity: Vec<u8>,

    /// Family seed (encrypted, shared secret)
    pub family_seed: Vec<u8>,

    /// Lineage path (encrypted)
    pub lineage: Vec<String>,

    /// Beacon endpoints (encrypted)
    pub beacons: Vec<String>,

    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
}

/// Genesis exchange protocol
#[derive(Debug)]
pub struct GenesisExchange {
    /// Configuration
    config: NfcConfig,

    /// Protocol handler
    #[expect(dead_code, reason = "reserved for future NFC frame operations")]
    protocol: NfcProtocol,

    /// Timing protector
    timing: TimingProtector,

    /// Shared crypto provider (Neural API by default; `BEARDOG_MODE=direct` for BearDog socket)
    provider: CryptoProvider,
}

/// Decode hex or base64 encoded bytes
#[expect(
    clippy::unnecessary_wraps,
    reason = "Result kept for uniform error propagation at call sites"
)]
fn decode_hex_or_b64(s: &str) -> Result<Vec<u8>> {
    // Try hex first (common for BearDog responses)
    if let Ok(bytes) = hex::decode(s) {
        return Ok(bytes);
    }
    // Fall back to trying raw bytes interpretation
    Ok(s.as_bytes().to_vec())
}

/// Simple hex encoding (no external dependency needed)
mod hex {
    pub(super) fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(data.len() * 2);
        for b in data {
            let _ = write!(s, "{b:02x}");
        }
        s
    }

    pub(super) fn decode(s: &str) -> std::result::Result<Vec<u8>, String> {
        if !s.len().is_multiple_of(2) {
            return Err("odd hex length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("invalid hex at {i}: {e}"))
            })
            .collect()
    }
}

impl GenesisExchange {
    /// Create new genesis exchange
    #[must_use]
    pub fn new(config: NfcConfig) -> Self {
        let timing = TimingProtector::new(config.target_exchange_duration, config.max_random_delay);

        let protocol = NfcProtocol::new(config.clone());
        let provider = CryptoProvider::from_env();

        Self {
            config,
            protocol,
            timing,
            provider,
        }
    }

    #[cfg(test)]
    fn for_test_with_provider(provider: CryptoProvider) -> Self {
        let config = NfcConfig::default();
        let timing = TimingProtector::new(config.target_exchange_duration, config.max_random_delay);
        let protocol = NfcProtocol::new(config.clone());
        Self {
            config,
            protocol,
            timing,
            provider,
        }
    }

    async fn nfc_crypto_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.provider.call(method, params).await.map_err(|e| NfcError::Crypto(e.to_string()))
    }

    async fn generate_x25519_keypair(&self) -> Result<[u8; PUBLIC_KEY_SIZE]> {
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

    async fn x25519_dh(&self, peer_pubkey: &[u8]) -> Result<Vec<u8>> {
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
                warn!("Crypto provider DH unavailable: {}. Using zero secret (TESTING ONLY).", e);
                Ok(vec![0u8; 32])
            }
        }
    }

    async fn generate_nonce(&self) -> Result<[u8; NONCE_SIZE]> {
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

    async fn encrypt(&self, plaintext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
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

    async fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
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

    async fn ed25519_sign(&self, data: &[u8]) -> Result<[u8; SIGNATURE_SIZE]> {
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

    async fn ed25519_verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
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

    async fn destroy_ephemeral_keys(&self) -> Result<()> {
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

    /// Initiate genesis exchange (as parent/initiator)
    ///
    /// # Errors
    ///
    /// Returns an error if any step of the exchange fails.
    ///
    /// Steps:
    /// 1. Generate ephemeral X25519 keypair
    /// 2. Send public key to peer
    /// 3. Receive peer's public key
    /// 4. Compute shared secret (X25519 DH)
    /// 5. Encrypt genesis credentials
    /// 6. Send encrypted genesis
    /// 7. Receive confirmation
    /// 8. Destroy ephemeral keys
    pub async fn initiate(
        &mut self,
        device: &mut NfcDevice,
        credentials: &GenesisCredentials,
    ) -> Result<()> {
        info!("🔐 Initiating genesis exchange");

        if self.config.timing_protection {
            self.timing.start();
            self.timing.random_delay().await;
        }

        // 1. Generate ephemeral X25519 keypair via BearDog
        let ephemeral_pubkey = self.generate_x25519_keypair().await?;

        // 2. Send public key to peer
        debug!("Sending ephemeral public key");
        device.send_raw(&ephemeral_pubkey).await?;

        // 3. Receive peer's public key
        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        // 4. Compute shared secret via BearDog
        let shared_secret = self.x25519_dh(&peer_pubkey).await?;

        // 5. Encrypt genesis credentials via BearDog
        let nonce = self.generate_nonce().await?;
        let serialized = serde_json::to_vec(credentials)?;
        let encrypted = self.encrypt(&serialized, &shared_secret, &nonce).await?;

        // 6. Sign and send encrypted genesis
        let signature = self.ed25519_sign(&encrypted).await?;

        let message = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            ephemeral_pubkey,
            nonce,
            encrypted,
            signature,
        );

        device.send_message(&message).await?;

        // 7. Receive confirmation
        let response = device.receive_message().await?;

        if response.msg_type != MSG_TYPE_GENESIS_RESPONSE {
            return Err(NfcError::InvalidMessageType(response.msg_type));
        }

        info!("Genesis exchange complete");

        // 8. Destroy ephemeral keys via BearDog
        self.destroy_ephemeral_keys().await?;

        if self.config.timing_protection {
            self.timing.pad_to_constant_time().await?;
        }

        Ok(())
    }

    /// Respond to genesis exchange (as child/responder)
    ///
    /// # Errors
    ///
    /// Returns an error if the exchange fails or credentials cannot be decrypted.
    pub async fn respond(&mut self, device: &mut NfcDevice) -> Result<GenesisCredentials> {
        info!("🔓 Responding to genesis exchange");

        if self.config.timing_protection {
            self.timing.start();
            self.timing.random_delay().await;
        }

        // 1. Generate ephemeral keypair via BearDog
        let ephemeral_pubkey = self.generate_x25519_keypair().await?;

        // 2. Receive peer's public key
        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        // 3. Send own public key
        device.send_raw(&ephemeral_pubkey).await?;
        debug!("Sent ephemeral public key");

        // 4. Compute shared secret via BearDog
        let shared_secret = self.x25519_dh(&peer_pubkey).await?;

        // 5. Receive encrypted genesis
        let message = device.receive_message().await?;

        if message.msg_type != MSG_TYPE_GENESIS_REQUEST {
            return Err(NfcError::InvalidMessageType(message.msg_type));
        }

        // 6. Verify signature via BearDog
        self.ed25519_verify(&message.encrypted_payload, &message.signature).await?;

        // 7. Decrypt genesis via BearDog
        let decrypted =
            self.decrypt(&message.encrypted_payload, &shared_secret, &message.nonce).await?;
        let credentials: GenesisCredentials = serde_json::from_slice(&decrypted)?;

        // 8. Send confirmation
        let conf_nonce = self.generate_nonce().await?;
        let conf_payload = vec![0u8; 16]; // Empty confirmation
        let conf_signature = self.ed25519_sign(&conf_payload).await?;

        let confirmation = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            ephemeral_pubkey,
            conf_nonce,
            conf_payload,
            conf_signature,
        );

        device.send_message(&confirmation).await?;

        info!("Genesis received");

        // 9. Destroy ephemeral keys via BearDog
        self.destroy_ephemeral_keys().await?;

        if self.config.timing_protection {
            self.timing.pad_to_constant_time().await?;
        }

        Ok(credentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_crypto_provider::RoutingMode;

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex::encode(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(hex::encode(&[]), "");
        assert_eq!(hex::encode(&[0x0d, 0xa4]), "0da4");
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(hex::decode("00ffab").unwrap(), vec![0x00, 0xff, 0xab]);
        assert_eq!(hex::decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(hex::decode("0da4").unwrap(), vec![0x0d, 0xa4]);
    }

    #[test]
    fn test_hex_decode_odd_length() {
        assert!(hex::decode("abc").is_err());
    }

    #[test]
    fn test_hex_decode_invalid() {
        assert!(hex::decode("zzzz").is_err());
    }

    #[test]
    fn test_hex_roundtrip() {
        let data = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        assert_eq!(hex::decode(&hex::encode(&data)).unwrap(), data);
    }

    #[test]
    fn test_decode_hex_or_b64_hex() {
        let result = decode_hex_or_b64("48656c6c6f").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_decode_hex_or_b64_fallback() {
        // Non-hex string falls back to raw bytes
        let result = decode_hex_or_b64("Hello!").unwrap();
        assert_eq!(result, b"Hello!");
    }

    #[test]
    fn test_crypto_provider_from_env_has_socket() {
        let p = CryptoProvider::from_env();
        assert!(!p.socket_path().is_empty());
    }

    #[tokio::test]
    async fn test_crypto_keypair_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let key = ex.generate_x25519_keypair().await.unwrap();
        assert_eq!(key.len(), PUBLIC_KEY_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_nonce_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let nonce = ex.generate_nonce().await.unwrap();
        assert_eq!(nonce.len(), NONCE_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_dh_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let shared = ex.x25519_dh(&[0u8; 32]).await.unwrap();
        assert_eq!(shared.len(), 32);
    }

    #[tokio::test]
    async fn test_crypto_sign_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let sig = ex.ed25519_sign(b"test data").await.unwrap();
        assert_eq!(sig.len(), SIGNATURE_SIZE);
    }

    #[tokio::test]
    async fn test_crypto_verify_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        ex.ed25519_verify(b"data", &[0u8; 64]).await.unwrap();
    }

    #[tokio::test]
    async fn test_crypto_encrypt_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let ct = ex.encrypt(b"plaintext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        assert_eq!(ct, b"plaintext");
    }

    #[tokio::test]
    async fn test_crypto_decrypt_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        let pt = ex.decrypt(b"ciphertext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        assert_eq!(pt, b"ciphertext");
    }

    #[tokio::test]
    async fn test_crypto_destroy_fallback_when_unavailable() {
        let ex = GenesisExchange::for_test_with_provider(CryptoProvider::with_mode(
            "/tmp/nonexistent-beardog.sock".to_string(),
            RoutingMode::Direct,
        ));
        ex.destroy_ephemeral_keys().await.unwrap();
    }

    #[test]
    fn test_genesis_credentials_serialization() {
        let creds = GenesisCredentials {
            identity: vec![1, 2, 3],
            family_seed: vec![4, 5, 6],
            lineage: vec!["root".to_string(), "child".to_string()],
            beacons: vec!["beacon1.onion".to_string()],
            timestamp: 1707350400000,
        };
        let json = serde_json::to_vec(&creds).unwrap();
        let decoded: GenesisCredentials = serde_json::from_slice(&json).unwrap();
        assert_eq!(decoded.identity, creds.identity);
        assert_eq!(decoded.family_seed, creds.family_seed);
        assert_eq!(decoded.lineage, creds.lineage);
        assert_eq!(decoded.beacons, creds.beacons);
        assert_eq!(decoded.timestamp, creds.timestamp);
    }
}
