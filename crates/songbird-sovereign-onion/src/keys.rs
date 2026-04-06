// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Onion identity key management

use crate::error::Result;
use crate::security_crypto::SecurityCryptoClient;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Import dalek types and standalone functions only for test/standalone mode
#[cfg(feature = "standalone")]
use crate::address::derive_onion_address;
#[cfg(feature = "standalone")]
use ed25519_dalek::SigningKey;

#[cfg(feature = "standalone")]
const X25519_BASEPOINT_BYTES: [u8; 32] = [
    9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Onion service identity (Ed25519 keypair + derived .onion address)
///
/// TRUE PRIMAL: Stores raw bytes, delegates crypto to `security provider`
#[derive(Debug, Clone)]
pub struct OnionIdentity {
    secret_key: [u8; 32],
    public_key: [u8; 32],
    onion_address: String,
    created_at: u64,
}

/// Serializable format for storing identity (v2 - complete storage)
///
/// Stores all identity components so no crypto derivation is needed on load.
/// This ensures production builds work without local crypto dependencies.
#[derive(Debug, Serialize, Deserialize)]
struct StoredIdentity {
    secret_key_bytes: [u8; 32],
    /// v2: Public key for reconstruction without crypto
    #[serde(default)]
    public_key_bytes: Option<[u8; 32]>,
    /// v2: Onion address for reconstruction without crypto
    #[serde(default)]
    onion_address: Option<String>,
    created_at: u64,
}

impl OnionIdentity {
    /// Generate new random onion identity via the security provider
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use songbird_sovereign_onion::{OnionIdentity, SecurityCryptoClient};
    /// # tokio_test::block_on(async {
    /// let client = SecurityCryptoClient::from_env();
    /// let identity = OnionIdentity::generate_via_security_provider(&client).await.unwrap();
    /// println!("Onion address: {}", identity.onion_address());
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if key generation, address derivation, or clock read fails.
    pub async fn generate_via_security_provider(client: &SecurityCryptoClient) -> Result<Self> {
        let keypair = client.ed25519_generate_keypair().await?;

        let onion_address =
            crate::address::derive_onion_address_via_security_provider(client, &keypair.public_key)
                .await?;

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                crate::error::OnionError::Other(format!("system clock before Unix epoch: {e}"))
            })?
            .as_secs();

        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
            onion_address,
            created_at,
        })
    }

    /// Load identity from stored bytes via the security provider
    ///
    /// # Errors
    ///
    /// Returns error if key bytes are invalid
    #[cfg_attr(
        feature = "standalone",
        expect(
            clippy::unused_async,
            reason = "awaits security provider RPC only when not feature standalone"
        )
    )]
    pub async fn from_stored_via_security_provider(
        client: &SecurityCryptoClient,
        bytes: &[u8],
    ) -> Result<Self> {
        // Extract secret key and timestamp from stored bytes
        let stored: StoredIdentity = serde_json::from_slice(bytes)?;
        let secret_key = &stored.secret_key_bytes;

        #[cfg(feature = "standalone")]
        {
            let _ = client;
            let signing_key = SigningKey::from_bytes(secret_key);
            let verifying_key = signing_key.verifying_key();
            let public_key = verifying_key.to_bytes();
            let onion_address = derive_onion_address(&verifying_key);

            Ok(Self {
                secret_key: *secret_key,
                public_key,
                onion_address,
                created_at: stored.created_at,
            })
        }

        #[cfg(not(feature = "standalone"))]
        {
            let public_key = client.ed25519_public_from_secret(secret_key).await?;
            let onion_address =
                crate::address::derive_onion_address_via_security_provider(client, &public_key)
                    .await?;

            Ok(Self {
                secret_key: *secret_key,
                public_key,
                onion_address,
                created_at: stored.created_at,
            })
        }
    }

    /// Standalone generation (for testing/offline only)
    ///
    /// This bypasses `security provider` and uses local crypto. Only available in test builds
    /// or with the `standalone` feature.
    #[cfg(feature = "standalone")]
    #[must_use]
    pub fn generate() -> Self {
        // Generate random 32-byte secret key
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let onion_address = derive_onion_address(&verifying_key);
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        Self {
            secret_key: secret_bytes,
            public_key: verifying_key.to_bytes(),
            onion_address,
            created_at,
        }
    }

    /// Standalone loading (for testing/offline only)
    ///
    /// # Errors
    ///
    /// Returns an error if the secret key bytes are not a valid Ed25519 seed.
    #[cfg(feature = "standalone")]
    pub fn from_stored(secret_bytes: &[u8; 32], created_at: u64) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let onion_address = derive_onion_address(&verifying_key);

        Ok(Self {
            secret_key: *secret_bytes,
            public_key: verifying_key.to_bytes(),
            onion_address,
            created_at,
        })
    }

    /// Get .onion address
    #[must_use]
    pub fn onion_address(&self) -> &str {
        &self.onion_address
    }

    /// Get Ed25519 public key bytes
    #[must_use]
    pub const fn public_key_bytes(&self) -> &[u8; 32] {
        &self.public_key
    }

    /// Get Ed25519 secret key bytes
    #[must_use]
    pub const fn secret_key_bytes(&self) -> &[u8; 32] {
        &self.secret_key
    }

    /// Get creation timestamp (Unix seconds)
    #[must_use]
    pub const fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Serialize for storage (production safe - v2 complete storage)
    ///
    /// Stores all identity components (secret, public, onion address) so
    /// no crypto derivation is needed on load.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_stored_bytes(&self) -> Result<Vec<u8>> {
        let stored = StoredIdentity {
            secret_key_bytes: self.secret_key,
            public_key_bytes: Some(self.public_key),
            onion_address: Some(self.onion_address.clone()),
            created_at: self.created_at,
        };
        serde_json::to_vec(&stored).map_err(Into::into)
    }

    /// Deserialize from storage (production safe - no crypto needed)
    ///
    /// # Errors
    ///
    /// Returns error if bytes are invalid or legacy v1 format in production.
    ///
    /// Loads identity from raw stored bytes. v2 format includes all components
    /// so no crypto derivation is needed. Falls back to v1 behavior for old storage.
    pub fn from_stored_bytes(bytes: &[u8]) -> Result<Self> {
        let stored: StoredIdentity = serde_json::from_slice(bytes)?;

        // v2 format: All fields present - no crypto needed
        if let (Some(public_key), Some(onion_address)) =
            (stored.public_key_bytes, stored.onion_address)
        {
            return Ok(Self {
                secret_key: stored.secret_key_bytes,
                public_key,
                onion_address,
                created_at: stored.created_at,
            });
        }

        // v1 format fallback: Need to derive public key and address
        #[cfg(feature = "standalone")]
        {
            Self::from_stored(&stored.secret_key_bytes, stored.created_at)
        }

        #[cfg(not(feature = "standalone"))]
        {
            // Production with v1 storage: Regenerate via security provider
            // Delete old storage and generate fresh identity
            Err(crate::OnionError::CryptoError(
                "Legacy v1 storage format detected. Delete ./data/sovereign-onion to regenerate identity.".to_string()
            ))
        }
    }
}

/// X25519 ephemeral keypair for session key exchange
///
/// TRUE PRIMAL: Stores raw bytes, delegates crypto to `security provider`
pub struct EphemeralKeypair {
    secret_key: [u8; 32],
    public_key: [u8; 32],
}

impl EphemeralKeypair {
    /// Generate via `security provider` (TRUE PRIMAL)
    ///
    /// # Errors
    ///
    /// Returns an error if `security provider` RPC fails.
    pub async fn generate_via_security_provider(client: &SecurityCryptoClient) -> Result<Self> {
        let keypair = client.x25519_generate_ephemeral().await?;
        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
        })
    }

    /// Derive shared secret via the security provider
    ///
    /// # Errors
    ///
    /// Returns an error if `security provider` RPC fails.
    pub async fn derive_shared_secret_via_security_provider(
        self,
        client: &SecurityCryptoClient,
        peer_public: &[u8; 32],
    ) -> Result<[u8; 32]> {
        client.x25519_derive_secret(&self.secret_key, peer_public).await
    }

    /// Standalone generation (for testing/offline only)
    #[cfg(feature = "standalone")]
    #[must_use]
    pub fn generate() -> Self {
        // Generate random secret key bytes
        let mut secret_key = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_key);

        // Clamp the secret key (X25519 requirement)
        secret_key[0] &= 0b1111_1000;
        secret_key[31] &= 127;
        secret_key[31] |= 64;

        // Derive public key using x25519 basepoint
        let public_key = x25519_dalek::x25519(secret_key, X25519_BASEPOINT_BYTES);

        Self {
            secret_key,
            public_key,
        }
    }

    /// Standalone ECDH (for testing/offline only)
    #[cfg(feature = "standalone")]
    #[must_use]
    pub fn derive_shared_secret(self, peer_public: &[u8; 32]) -> [u8; 32] {
        x25519_dalek::x25519(self.secret_key, *peer_public)
    }

    /// Get public key bytes
    #[must_use]
    pub const fn public_bytes(&self) -> &[u8; 32] {
        &self.public_key
    }
}

/// Session keys for encrypted communication
#[derive(Debug, Clone)]
pub struct SessionKeys {
    /// Key for encrypting data we send
    pub send_key: [u8; 32],
    /// Key for decrypting data we receive
    pub recv_key: [u8; 32],
}

impl SessionKeys {
    /// Derive session keys via `security provider` (TRUE PRIMAL)
    ///
    /// Uses `security provider`'s HMAC-SHA256 to implement HKDF for session key derivation.
    ///
    /// # Errors
    ///
    /// Returns an error if `security provider` HMAC RPC fails.
    pub async fn derive_via_security_provider(
        client: &SecurityCryptoClient,
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Result<Self> {
        // 1. HKDF-Extract: PRK = HMAC-SHA256(salt=zeros, IKM=shared_secret)
        let prk = client.hmac_sha256(&[0u8; 32], shared_secret).await?;

        // 2. HKDF-Expand for client key
        let mut client_info = Vec::new();
        client_info.extend_from_slice(b"sovereign-onion client");
        client_info.extend_from_slice(client_nonce);
        client_info.extend_from_slice(server_nonce);
        client_info.push(0x01);
        let client_key = client.hmac_sha256(&prk, &client_info).await?;

        // 3. HKDF-Expand for server key
        let mut server_info = Vec::new();
        server_info.extend_from_slice(b"sovereign-onion server");
        server_info.extend_from_slice(client_nonce);
        server_info.extend_from_slice(server_nonce);
        server_info.push(0x01);
        let server_key = client.hmac_sha256(&prk, &server_info).await?;

        if is_client {
            Ok(Self {
                send_key: client_key,
                recv_key: server_key,
            })
        } else {
            Ok(Self {
                send_key: server_key,
                recv_key: client_key,
            })
        }
    }

    /// Standalone derivation (for testing/offline only)
    ///
    /// # Errors
    ///
    /// Returns an error if HMAC construction or key material extraction fails.
    #[cfg(feature = "standalone")]
    pub fn derive(
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Result<Self> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // 1. HKDF-Extract: PRK = HMAC-SHA256(salt=zeros, IKM=shared_secret)
        let mut mac = HmacSha256::new_from_slice(&[0u8; 32])
            .map_err(|e| crate::OnionError::CryptoError(format!("HKDF extract HMAC key: {e}")))?;
        mac.update(shared_secret);
        let prk = mac.finalize().into_bytes();

        // 2. HKDF-Expand for client key
        let mut mac = HmacSha256::new_from_slice(&prk).map_err(|e| {
            crate::OnionError::CryptoError(format!("HKDF expand (client) HMAC key: {e}"))
        })?;
        mac.update(b"sovereign-onion client");
        mac.update(client_nonce);
        mac.update(server_nonce);
        mac.update(&[0x01]); // Counter
        let client_key_full = mac.finalize().into_bytes();
        let client_key: [u8; 32] = client_key_full[..32]
            .try_into()
            .map_err(|_| crate::OnionError::CryptoError("HKDF client key length".into()))?;

        // 3. HKDF-Expand for server key
        let mut mac = HmacSha256::new_from_slice(&prk).map_err(|e| {
            crate::OnionError::CryptoError(format!("HKDF expand (server) HMAC key: {e}"))
        })?;
        mac.update(b"sovereign-onion server");
        mac.update(client_nonce);
        mac.update(server_nonce);
        mac.update(&[0x01]); // Counter
        let server_key_full = mac.finalize().into_bytes();
        let server_key: [u8; 32] = server_key_full[..32]
            .try_into()
            .map_err(|_| crate::OnionError::CryptoError("HKDF server key length".into()))?;

        // Assign keys based on role
        if is_client {
            Ok(Self {
                send_key: client_key,
                recv_key: server_key,
            })
        } else {
            Ok(Self {
                send_key: server_key,
                recv_key: client_key,
            })
        }
    }
}

#[cfg(all(test, feature = "standalone"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = OnionIdentity::generate();

        // Check onion address format
        assert!(
            std::path::Path::new(identity.onion_address())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("onion"))
        );
        assert_eq!(identity.onion_address().len(), 62);

        // Check timestamp
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(identity.created_at() <= now);
        assert!(identity.created_at() > now - 10); // Within last 10 seconds
    }

    #[test]
    fn test_identity_serialization() {
        let original = OnionIdentity::generate();
        let bytes = original.to_stored_bytes().unwrap();
        let restored = OnionIdentity::from_stored_bytes(&bytes).unwrap();

        assert_eq!(original.onion_address(), restored.onion_address());
        assert_eq!(original.created_at(), restored.created_at());
        assert_eq!(original.public_key_bytes(), restored.public_key_bytes());
    }

    #[test]
    fn test_ephemeral_keypair() {
        let keypair1 = EphemeralKeypair::generate();
        let keypair2 = EphemeralKeypair::generate();

        // Public keys should be different
        assert_ne!(keypair1.public_bytes(), keypair2.public_bytes());

        // Store public keys before moving
        let pubkey1 = *keypair1.public_bytes();
        let pubkey2 = *keypair2.public_bytes();

        // Shared secrets should match (ECDH property)
        let secret1 = keypair1.derive_shared_secret(&pubkey2);
        let secret2 = keypair2.derive_shared_secret(&pubkey1);
        assert_eq!(secret1, secret2);
    }

    #[test]
    fn test_session_keys_derivation() {
        let shared_secret = [0x42u8; 32];
        let client_nonce = [0x01u8; 24];
        let server_nonce = [0x02u8; 24];

        // Derive from client perspective
        let client_keys =
            SessionKeys::derive(&shared_secret, &client_nonce, &server_nonce, true).unwrap();

        // Derive from server perspective
        let server_keys =
            SessionKeys::derive(&shared_secret, &client_nonce, &server_nonce, false).unwrap();

        // Client's send key = Server's recv key
        assert_eq!(client_keys.send_key, server_keys.recv_key);

        // Client's recv key = Server's send key
        assert_eq!(client_keys.recv_key, server_keys.send_key);

        // Keys should be different
        assert_ne!(client_keys.send_key, client_keys.recv_key);
    }

    #[test]
    fn test_session_keys_unique() {
        let shared_secret = [0x42u8; 32];
        let nonce1 = [0x01u8; 24];
        let nonce2 = [0x02u8; 24];

        let keys1 = SessionKeys::derive(&shared_secret, &nonce1, &nonce2, true).unwrap();
        let keys2 = SessionKeys::derive(&shared_secret, &nonce2, &nonce1, true).unwrap();

        // Different nonces should produce different keys
        assert_ne!(keys1.send_key, keys2.send_key);
        assert_ne!(keys1.recv_key, keys2.recv_key);
    }
}

#[cfg(test)]
mod stored_identity_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use crate::keys::OnionIdentity;
    use serde_json::json;

    #[test]
    fn to_from_stored_bytes_v2_roundtrip_without_local_ed25519() {
        let j = json!({
            "secret_key_bytes": vec![9u8; 32],
            "public_key_bytes": vec![8u8; 32],
            "onion_address": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion",
            "created_at": 42u64
        });
        let bytes = serde_json::to_vec(&j).expect("serialize fixture");
        let id = OnionIdentity::from_stored_bytes(&bytes).expect("parse v2 stored identity");
        assert_eq!(id.created_at(), 42, "created_at");
        assert_eq!(
            id.onion_address(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion",
            "onion address"
        );
        assert_eq!(id.public_key_bytes(), &[8u8; 32], "public key bytes");
        let round = id.to_stored_bytes().expect("serialize stored identity");
        let id2 = OnionIdentity::from_stored_bytes(&round).expect("second parse");
        assert_eq!(id2.secret_key_bytes(), id.secret_key_bytes(), "secret roundtrip");
    }

    #[test]
    fn from_stored_bytes_rejects_invalid_json() {
        let r = OnionIdentity::from_stored_bytes(b"{not json");
        assert!(r.is_err(), "expected serde error, got {r:?}");
    }

    /// Production builds without `standalone` cannot reconstruct v1 blobs (secret-only).
    #[cfg(not(feature = "standalone"))]
    #[test]
    fn from_stored_bytes_v1_legacy_returns_crypto_error() {
        let j = json!({
            "secret_key_bytes": vec![3u8; 32],
            "created_at": 0u64
        });
        let bytes = serde_json::to_vec(&j).expect("serialize v1 fixture");
        let r = OnionIdentity::from_stored_bytes(&bytes);
        assert!(
            matches!(r, Err(crate::error::OnionError::CryptoError(ref s)) if s.contains("Legacy")),
            "expected legacy storage error, got {r:?}"
        );
    }
}
