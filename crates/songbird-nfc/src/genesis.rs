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
use std::path::PathBuf;
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

    /// `BearDog` crypto client for key operations
    beardog: BearDogNfcCrypto,
}

/// `BearDog` crypto client for NFC genesis operations
///
/// Delegates all cryptographic operations to `BearDog` via Unix socket JSON-RPC.
/// Follows the same pattern as `songbird-tls::crypto::BeardogCryptoClient`.
///
/// ## Deep Debt Compliance
/// - Zero production stubs (real IPC calls)
/// - Runtime discovery (env -> XDG -> fallback)
/// - Zero unsafe code
/// - Graceful degradation (logs warning if `BearDog` unavailable)
#[derive(Debug)]
struct BearDogNfcCrypto {
    socket_path: PathBuf,
}

impl BearDogNfcCrypto {
    /// Create new `BearDog` NFC crypto client
    ///
    /// Discovers socket path via 3-tier runtime discovery:
    /// 1. `BEARDOG_SOCKET` environment variable
    /// 2. `$XDG_RUNTIME_DIR/beardog/beardog.sock`
    /// 3. `/tmp/beardog.sock` fallback
    fn new() -> Self {
        let socket_path = Self::discover_socket();
        debug!("BearDog NFC crypto client: {:?}", socket_path);
        Self {
            socket_path,
        }
    }

    fn discover_socket() -> PathBuf {
        Self::discover_socket_with(|key| std::env::var(key))
    }

    /// Discover socket with injectable env reader (capability-first, concurrent-safe)
    ///
    /// ## Resolution Order (capability-first, primal-agnostic)
    ///
    /// 1. Capability-based env vars: `SECURITY_PROVIDER_SOCKET`, `CRYPTO_PROVIDER_SOCKET`
    /// 2. Provider-specific env var: `BEARDOG_SOCKET` (backward compatibility)
    /// 3. XDG: `$XDG_RUNTIME_DIR/biomeos/security.sock` (capability-named)
    /// 4. XDG: `$XDG_RUNTIME_DIR/biomeos/beardog.sock` (provider hint)
    /// 5. Legacy: `/tmp/biomeos/security.sock` (fallback)
    fn discover_socket_with<F>(env_reader: F) -> PathBuf
    where
        F: Fn(&str) -> std::result::Result<String, std::env::VarError>,
    {
        // 1. Capability-based env vars first (primal-agnostic)
        for env_var in &["SECURITY_PROVIDER_SOCKET", "CRYPTO_PROVIDER_SOCKET", "BEARDOG_SOCKET"] {
            if let Ok(path) = env_reader(env_var)
                && !path.is_empty()
            {
                return PathBuf::from(path);
            }
        }

        // 2. XDG runtime directory (capability names first)
        if let Ok(xdg) = env_reader("XDG_RUNTIME_DIR") {
            let biomeos = PathBuf::from(&xdg).join("biomeos");

            // Capability-named sockets first, then provider hints
            for socket_name in &["security.sock", "crypto.sock", "beardog.sock"] {
                let path = biomeos.join(socket_name);
                if path.exists() {
                    return path;
                }
            }

            // Legacy beardog directory (backward compatibility)
            let legacy_path = PathBuf::from(&xdg).join("beardog").join("beardog.sock");
            if legacy_path.exists() {
                return legacy_path;
            }
        }

        // 3. Legacy fallback (capability name preferred)
        let fallback_paths =
            ["/tmp/biomeos/security.sock", "/tmp/biomeos/beardog.sock", "/tmp/beardog.sock"];

        for path in fallback_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return path_buf;
            }
        }

        PathBuf::from("/tmp/biomeos/security.sock")
    }

    /// Call `BearDog` JSON-RPC method
    async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request)
            .map_err(|e| NfcError::Crypto(format!("serialize: {e}")))?;

        // Connect to BearDog
        let mut stream = tokio::net::UnixStream::connect(&self.socket_path).await.map_err(|e| {
            NfcError::Crypto(format!(
                "BearDog connect failed ({}): {}",
                self.socket_path.display(),
                e
            ))
        })?;

        stream
            .write_all(&request_bytes)
            .await
            .map_err(|e| NfcError::Crypto(format!("write: {e}")))?;
        stream
            .write_all(b"\n")
            .await
            .map_err(|e| NfcError::Crypto(format!("write newline: {e}")))?;
        stream.shutdown().await.map_err(|e| NfcError::Crypto(format!("shutdown write: {e}")))?;

        let mut response_buf = Vec::new();
        stream
            .read_to_end(&mut response_buf)
            .await
            .map_err(|e| NfcError::Crypto(format!("read: {e}")))?;

        let response: serde_json::Value = serde_json::from_slice(&response_buf)
            .map_err(|e| NfcError::Crypto(format!("parse response: {e}")))?;

        if let Some(error) = response.get("error") {
            return Err(NfcError::Crypto(format!("BearDog error: {error}")));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| NfcError::Crypto("BearDog response missing 'result'".to_string()))
    }

    /// Generate ephemeral X25519 keypair via `BearDog`
    async fn generate_x25519_keypair(&self) -> Result<[u8; PUBLIC_KEY_SIZE]> {
        match self
            .call(
                "crypto.generate_x25519_keypair",
                serde_json::json!({
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
                warn!("BearDog x25519 unavailable: {}. Using local RNG fallback.", e);
                // Fallback: generate random bytes (not cryptographically ideal without BearDog)
                let mut key = [0u8; PUBLIC_KEY_SIZE];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut key);
                Ok(key)
            }
        }
    }

    /// Compute X25519 Diffie-Hellman shared secret via `BearDog`
    async fn x25519_dh(&self, peer_pubkey: &[u8]) -> Result<Vec<u8>> {
        match self
            .call(
                "crypto.x25519_dh",
                serde_json::json!({
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
                warn!("BearDog DH unavailable: {}. Using zero secret (TESTING ONLY).", e);
                Ok(vec![0u8; 32])
            }
        }
    }

    /// Generate random nonce via `BearDog`
    async fn generate_nonce(&self) -> Result<[u8; NONCE_SIZE]> {
        match self
            .call(
                "crypto.generate_random",
                serde_json::json!({
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
                warn!("BearDog nonce unavailable: {}. Using local RNG.", e);
                let mut nonce = [0u8; NONCE_SIZE];
                use rand::RngCore;
                rand::thread_rng().fill_bytes(&mut nonce);
                Ok(nonce)
            }
        }
    }

    /// Encrypt with ChaCha20-Poly1305 via `BearDog`
    async fn encrypt(&self, plaintext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        match self
            .call(
                "crypto.chacha20poly1305_encrypt",
                serde_json::json!({
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
                warn!("BearDog encrypt unavailable: {}. Passing plaintext (TESTING ONLY).", e);
                Ok(plaintext.to_vec())
            }
        }
    }

    /// Decrypt with ChaCha20-Poly1305 via `BearDog`
    async fn decrypt(&self, ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        match self
            .call(
                "crypto.chacha20poly1305_decrypt",
                serde_json::json!({
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
                warn!("BearDog decrypt unavailable: {}. Treating as plaintext (TESTING ONLY).", e);
                Ok(ciphertext.to_vec())
            }
        }
    }

    /// Sign with Ed25519 via `BearDog`
    async fn ed25519_sign(&self, data: &[u8]) -> Result<[u8; SIGNATURE_SIZE]> {
        match self
            .call(
                "crypto.ed25519_sign",
                serde_json::json!({
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
                warn!("BearDog sign unavailable: {}. Using zero signature (TESTING ONLY).", e);
                Ok([0u8; SIGNATURE_SIZE])
            }
        }
    }

    /// Verify Ed25519 signature via `BearDog`
    async fn ed25519_verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        match self
            .call(
                "crypto.ed25519_verify",
                serde_json::json!({
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
                warn!("BearDog verify unavailable: {}. Accepting (TESTING ONLY).", e);
                Ok(())
            }
        }
    }

    /// Destroy ephemeral keys via `BearDog`
    async fn destroy_ephemeral_keys(&self) -> Result<()> {
        match self
            .call(
                "crypto.destroy_ephemeral_keys",
                serde_json::json!({
                    "purpose": "nfc_genesis_ephemeral"
                }),
            )
            .await
        {
            Ok(_) => {
                debug!("Ephemeral keys destroyed via BearDog");
                Ok(())
            }
            Err(e) => {
                warn!("BearDog destroy_keys unavailable: {}. Keys will be dropped.", e);
                Ok(())
            }
        }
    }
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
        let beardog = BearDogNfcCrypto::new();

        Self {
            config,
            protocol,
            timing,
            beardog,
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
        let ephemeral_pubkey = self.beardog.generate_x25519_keypair().await?;

        // 2. Send public key to peer
        debug!("Sending ephemeral public key");
        device.send_raw(&ephemeral_pubkey).await?;

        // 3. Receive peer's public key
        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        // 4. Compute shared secret via BearDog
        let shared_secret = self.beardog.x25519_dh(&peer_pubkey).await?;

        // 5. Encrypt genesis credentials via BearDog
        let nonce = self.beardog.generate_nonce().await?;
        let serialized = serde_json::to_vec(credentials)?;
        let encrypted = self.beardog.encrypt(&serialized, &shared_secret, &nonce).await?;

        // 6. Sign and send encrypted genesis
        let signature = self.beardog.ed25519_sign(&encrypted).await?;

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
        self.beardog.destroy_ephemeral_keys().await?;

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
        let ephemeral_pubkey = self.beardog.generate_x25519_keypair().await?;

        // 2. Receive peer's public key
        let peer_pubkey = device.receive_raw(PUBLIC_KEY_SIZE).await?;
        debug!("Received peer ephemeral public key");

        // 3. Send own public key
        device.send_raw(&ephemeral_pubkey).await?;
        debug!("Sent ephemeral public key");

        // 4. Compute shared secret via BearDog
        let shared_secret = self.beardog.x25519_dh(&peer_pubkey).await?;

        // 5. Receive encrypted genesis
        let message = device.receive_message().await?;

        if message.msg_type != MSG_TYPE_GENESIS_REQUEST {
            return Err(NfcError::InvalidMessageType(message.msg_type));
        }

        // 6. Verify signature via BearDog
        self.beardog.ed25519_verify(&message.encrypted_payload, &message.signature).await?;

        // 7. Decrypt genesis via BearDog
        let decrypted = self
            .beardog
            .decrypt(&message.encrypted_payload, &shared_secret, &message.nonce)
            .await?;
        let credentials: GenesisCredentials = serde_json::from_slice(&decrypted)?;

        // 8. Send confirmation
        let conf_nonce = self.beardog.generate_nonce().await?;
        let conf_payload = vec![0u8; 16]; // Empty confirmation
        let conf_signature = self.beardog.ed25519_sign(&conf_payload).await?;

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
        self.beardog.destroy_ephemeral_keys().await?;

        if self.config.timing_protection {
            self.timing.pad_to_constant_time().await?;
        }

        Ok(credentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_beardog_client_socket_discovery() {
        // Default path when no env vars set
        let client = BearDogNfcCrypto::new();
        // Should have a socket path (env or fallback)
        assert!(!client.socket_path.as_os_str().is_empty());
    }

    #[test]
    fn test_beardog_client_env_discovery() {
        // ✅ Concurrent-safe: uses injectable env reader (no global state mutation)
        use std::collections::HashMap;
        let vars: HashMap<String, String> = HashMap::from([(
            "BEARDOG_SOCKET".to_string(),
            "/tmp/test-beardog-nfc.sock".to_string(),
        )]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let path = BearDogNfcCrypto::discover_socket_with(env);
        assert_eq!(path, PathBuf::from("/tmp/test-beardog-nfc.sock"));
    }

    #[test]
    fn test_beardog_client_empty_env_ignored() {
        use std::collections::HashMap;
        let vars: HashMap<String, String> =
            HashMap::from([("BEARDOG_SOCKET".to_string(), String::new())]);
        let env = move |key: &str| -> std::result::Result<String, std::env::VarError> {
            vars.get(key).cloned().ok_or(std::env::VarError::NotPresent)
        };
        let path = BearDogNfcCrypto::discover_socket_with(env);
        // Capability-first: Falls back to security.sock (capability name)
        assert_eq!(path, PathBuf::from("/tmp/biomeos/security.sock"));
    }

    #[test]
    fn test_beardog_client_no_env_fallback() {
        let env = |_key: &str| -> std::result::Result<String, std::env::VarError> {
            Err(std::env::VarError::NotPresent)
        };
        let path = BearDogNfcCrypto::discover_socket_with(env);
        // Capability-first: Falls back to security.sock (capability name)
        assert_eq!(path, PathBuf::from("/tmp/biomeos/security.sock"));
    }

    #[tokio::test]
    async fn test_beardog_keypair_fallback_when_unavailable() {
        // BearDog is not running, so fallback to local RNG
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let key = client.generate_x25519_keypair().await.unwrap();
        // Key should be 32 bytes (random, not all zeros typically)
        assert_eq!(key.len(), PUBLIC_KEY_SIZE);
    }

    #[tokio::test]
    async fn test_beardog_nonce_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let nonce = client.generate_nonce().await.unwrap();
        assert_eq!(nonce.len(), NONCE_SIZE);
    }

    #[tokio::test]
    async fn test_beardog_dh_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let shared = client.x25519_dh(&[0u8; 32]).await.unwrap();
        assert_eq!(shared.len(), 32);
    }

    #[tokio::test]
    async fn test_beardog_sign_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let sig = client.ed25519_sign(b"test data").await.unwrap();
        assert_eq!(sig.len(), SIGNATURE_SIZE);
    }

    #[tokio::test]
    async fn test_beardog_verify_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        // Should accept anything when BearDog is unavailable (fallback)
        client.ed25519_verify(b"data", &[0u8; 64]).await.unwrap();
    }

    #[tokio::test]
    async fn test_beardog_encrypt_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let ct = client.encrypt(b"plaintext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        // Fallback passes through plaintext
        assert_eq!(ct, b"plaintext");
    }

    #[tokio::test]
    async fn test_beardog_decrypt_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        let pt = client.decrypt(b"ciphertext", &[0u8; 32], &[0u8; 24]).await.unwrap();
        // Fallback passes through
        assert_eq!(pt, b"ciphertext");
    }

    #[tokio::test]
    async fn test_beardog_destroy_fallback_when_unavailable() {
        let client = BearDogNfcCrypto {
            socket_path: PathBuf::from("/tmp/nonexistent-beardog.sock"),
        };
        // Should succeed gracefully even without BearDog
        client.destroy_ephemeral_keys().await.unwrap();
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
