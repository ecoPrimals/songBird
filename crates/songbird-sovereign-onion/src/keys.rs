//! Onion identity key management

use crate::beardog_crypto::BeardogCryptoClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// Import dalek types and standalone functions only for test/standalone mode
#[cfg(any(test, feature = "standalone"))]
use crate::address::derive_onion_address;
#[cfg(any(test, feature = "standalone"))]
use ed25519_dalek::SigningKey;

/// Onion service identity (Ed25519 keypair + derived .onion address)
///
/// TRUE PRIMAL: Stores raw bytes, delegates crypto to BearDog
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
    /// Generate new random onion identity via BearDog (TRUE PRIMAL)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use songbird_sovereign_onion::{OnionIdentity, BeardogCryptoClient};
    /// # tokio_test::block_on(async {
    /// let client = BeardogCryptoClient::from_env().unwrap();
    /// let identity = OnionIdentity::generate_via_beardog(&client).await.unwrap();
    /// println!("Onion address: {}", identity.onion_address());
    /// # });
    /// ```
    pub async fn generate_via_beardog(client: &BeardogCryptoClient) -> Result<Self> {
        let keypair = client.ed25519_generate_keypair()?;
        
        // Derive .onion address via BearDog
        let onion_address = crate::address::derive_onion_address_via_beardog(client, &keypair.public_key).await?;
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
            onion_address,
            created_at,
        })
    }

    /// Load identity from stored bytes via BearDog (TRUE PRIMAL)
    ///
    /// # Errors
    ///
    /// Returns error if key bytes are invalid
    pub async fn from_stored_via_beardog(
        client: &BeardogCryptoClient,
        bytes: &[u8],
    ) -> Result<Self> {
        // Extract secret key and timestamp from stored bytes
        let stored: StoredIdentity = serde_json::from_slice(bytes)?;
        let secret_key = &stored.secret_key_bytes;
        let created_at = stored.created_at;
        
        // Derive public key from secret (BearDog should verify this is valid)
        // For Ed25519, we can derive public key from secret locally
        // But to fully delegate, we use a test sign operation to verify the key
        let test_msg = b"test";
        let _signature = client.ed25519_sign(secret_key, test_msg)?;
        
        // For now, derive public key locally (Ed25519 property)
        // TODO: Add crypto.ed25519_public_from_secret to BearDog
        #[cfg(any(test, feature = "standalone"))]
        let public_key = {
            let signing_key = SigningKey::from_bytes(secret_key);
            signing_key.verifying_key().to_bytes()
        };
        
        #[cfg(not(any(test, feature = "standalone")))]
        let public_key = {
            // ⚠️ TRUE PRIMAL: In production, public key must be provided!
            // Ed25519 public key cannot be derived without the crypto library.
            // Production code should either:
            // 1. Store both secret + public bytes in OnionIdentity
            // 2. Use BearDog to derive public from secret (requires BearDog API extension)
            // For now, require public_key parameter in production builds
            return Err(crate::OnionError::CryptoError(
                "Public key required in production mode (use from_stored_bytes_with_public or BearDog delegation)".to_string()
            ).into());
        };
        
        let onion_address = crate::address::derive_onion_address_via_beardog(client, &public_key).await?;

        Ok(Self {
            secret_key: *secret_key,
            public_key,
            onion_address,
            created_at,
        })
    }

    /// Standalone generation (for testing/offline only)
    ///
    /// This bypasses BearDog and uses local crypto. Only available in test builds
    /// or with the `standalone` feature.
    #[cfg(any(test, feature = "standalone"))]
    pub fn generate() -> Self {
        // Generate random 32-byte secret key
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let onion_address = derive_onion_address(&verifying_key);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            secret_key: secret_bytes,
            public_key: verifying_key.to_bytes(),
            onion_address,
            created_at,
        }
    }

    /// Standalone loading (for testing/offline only)
    #[cfg(any(test, feature = "standalone"))]
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
    pub fn onion_address(&self) -> &str {
        &self.onion_address
    }

    /// Get Ed25519 public key bytes
    pub fn public_key_bytes(&self) -> &[u8; 32] {
        &self.public_key
    }

    /// Get Ed25519 secret key bytes
    pub fn secret_key_bytes(&self) -> &[u8; 32] {
        &self.secret_key
    }

    /// Get creation timestamp (Unix seconds)
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Serialize for storage (production safe - v2 complete storage)
    ///
    /// Stores all identity components (secret, public, onion address) so
    /// no crypto derivation is needed on load.
    pub fn to_stored_bytes(&self) -> Vec<u8> {
        let stored = StoredIdentity {
            secret_key_bytes: self.secret_key,
            public_key_bytes: Some(self.public_key),
            onion_address: Some(self.onion_address.clone()),
            created_at: self.created_at,
        };
        serde_json::to_vec(&stored).unwrap()
    }

    /// Deserialize from storage (production safe - no crypto needed)
    ///
    /// Loads identity from raw stored bytes. v2 format includes all components
    /// so no crypto derivation is needed. Falls back to v1 behavior for old storage.
    pub fn from_stored_bytes(bytes: &[u8]) -> Result<Self> {
        let stored: StoredIdentity = serde_json::from_slice(bytes)?;
        
        // v2 format: All fields present - no crypto needed
        if let (Some(public_key), Some(onion_address)) = (stored.public_key_bytes, stored.onion_address) {
            return Ok(Self {
                secret_key: stored.secret_key_bytes,
                public_key,
                onion_address,
                created_at: stored.created_at,
            });
        }
        
        // v1 format fallback: Need to derive public key and address
        #[cfg(any(test, feature = "standalone"))]
        {
            Self::from_stored(&stored.secret_key_bytes, stored.created_at)
        }
        
        #[cfg(not(any(test, feature = "standalone")))]
        {
            // Production with v1 storage: Regenerate via BearDog
            // Delete old storage and generate fresh identity
            Err(crate::OnionError::CryptoError(
                "Legacy v1 storage format detected. Delete ./data/sovereign-onion to regenerate identity.".to_string()
            ).into())
        }
    }
}

/// X25519 ephemeral keypair for session key exchange
///
/// TRUE PRIMAL: Stores raw bytes, delegates crypto to BearDog
pub struct EphemeralKeypair {
    secret_key: [u8; 32],
    public_key: [u8; 32],
}

impl EphemeralKeypair {
    /// Generate via BearDog (TRUE PRIMAL)
    pub fn generate_via_beardog(client: &BeardogCryptoClient) -> Result<Self> {
        let keypair = client.x25519_generate_ephemeral()?;
        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
        })
    }

    /// Derive shared secret via BearDog (TRUE PRIMAL)
    pub fn derive_shared_secret_via_beardog(
        self,
        client: &BeardogCryptoClient,
        peer_public: &[u8; 32],
    ) -> Result<[u8; 32]> {
        client.x25519_derive_secret(&self.secret_key, peer_public)
    }

    /// Standalone generation (for testing/offline only)
    #[cfg(any(test, feature = "standalone"))]
    pub fn generate() -> Self {
        // Generate random secret key bytes
        let mut secret_key = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_key);
        
        // Clamp the secret key (X25519 requirement)
        secret_key[0] &= 248;
        secret_key[31] &= 127;
        secret_key[31] |= 64;
        
        // Derive public key using x25519 basepoint
        const X25519_BASEPOINT_BYTES: [u8; 32] = [
            9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
        let public_key = x25519_dalek::x25519(secret_key, X25519_BASEPOINT_BYTES);

        Self {
            secret_key,
            public_key,
        }
    }

    /// Standalone ECDH (for testing/offline only)
    #[cfg(any(test, feature = "standalone"))]
    pub fn derive_shared_secret(self, peer_public: &[u8; 32]) -> [u8; 32] {
        x25519_dalek::x25519(self.secret_key, *peer_public)
    }

    /// Get public key bytes
    pub fn public_bytes(&self) -> &[u8; 32] {
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
    /// Derive session keys via BearDog (TRUE PRIMAL)
    ///
    /// Uses BearDog's HMAC-SHA256 to implement HKDF for session key derivation.
    pub fn derive_via_beardog(
        client: &BeardogCryptoClient,
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Result<Self> {
        // 1. HKDF-Extract: PRK = HMAC-SHA256(salt=zeros, IKM=shared_secret)
        let prk = client.hmac_sha256(&[0u8; 32], shared_secret)?;
        
        // 2. HKDF-Expand for client key
        let mut client_info = Vec::new();
        client_info.extend_from_slice(b"sovereign-onion client");
        client_info.extend_from_slice(client_nonce);
        client_info.extend_from_slice(server_nonce);
        client_info.push(0x01);
        let client_key = client.hmac_sha256(&prk, &client_info)?;
        
        // 3. HKDF-Expand for server key  
        let mut server_info = Vec::new();
        server_info.extend_from_slice(b"sovereign-onion server");
        server_info.extend_from_slice(client_nonce);
        server_info.extend_from_slice(server_nonce);
        server_info.push(0x01);
        let server_key = client.hmac_sha256(&prk, &server_info)?;
        
        if is_client {
            Ok(Self { send_key: client_key, recv_key: server_key })
        } else {
            Ok(Self { send_key: server_key, recv_key: client_key })
        }
    }

    /// Standalone derivation (for testing/offline only)
    #[cfg(any(test, feature = "standalone"))]
    pub fn derive(
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Self {
        use hmac::{ Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // 1. HKDF-Extract: PRK = HMAC-SHA256(salt=zeros, IKM=shared_secret)
        let mut mac = HmacSha256::new_from_slice(&[0u8; 32]).unwrap();
        mac.update(shared_secret);
        let prk = mac.finalize().into_bytes();

        // 2. HKDF-Expand for client key
        let mut mac = HmacSha256::new_from_slice(&prk).unwrap();
        mac.update(b"sovereign-onion client");
        mac.update(client_nonce);
        mac.update(server_nonce);
        mac.update(&[0x01]); // Counter
        let client_key_full = mac.finalize().into_bytes();
        let client_key: [u8; 32] = client_key_full[..32].try_into().unwrap();

        // 3. HKDF-Expand for server key
        let mut mac = HmacSha256::new_from_slice(&prk).unwrap();
        mac.update(b"sovereign-onion server");
        mac.update(client_nonce);
        mac.update(server_nonce);
        mac.update(&[0x01]); // Counter
        let server_key_full = mac.finalize().into_bytes();
        let server_key: [u8; 32] = server_key_full[..32].try_into().unwrap();

        // Assign keys based on role
        if is_client {
            Self {
                send_key: client_key,
                recv_key: server_key,
            }
        } else {
            Self {
                send_key: server_key,
                recv_key: client_key,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = OnionIdentity::generate();

        // Check onion address format
        assert!(identity.onion_address().ends_with(".onion"));
        assert_eq!(identity.onion_address().len(), 62);

        // Check timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(identity.created_at() <= now);
        assert!(identity.created_at() > now - 10); // Within last 10 seconds
    }

    #[test]
    fn test_identity_serialization() {
        let original = OnionIdentity::generate();
        let bytes = original.to_stored_bytes();
        let restored = OnionIdentity::from_stored_bytes(&bytes).unwrap();

        assert_eq!(original.onion_address(), restored.onion_address());
        assert_eq!(original.created_at(), restored.created_at());
        assert_eq!(
            original.public_key_bytes(),
            restored.public_key_bytes()
        );
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
        let client_keys = SessionKeys::derive(&shared_secret, &client_nonce, &server_nonce, true);

        // Derive from server perspective
        let server_keys = SessionKeys::derive(&shared_secret, &client_nonce, &server_nonce, false);

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

        let keys1 = SessionKeys::derive(&shared_secret, &nonce1, &nonce2, true);
        let keys2 = SessionKeys::derive(&shared_secret, &nonce2, &nonce1, true);

        // Different nonces should produce different keys
        assert_ne!(keys1.send_key, keys2.send_key);
        assert_ne!(keys1.recv_key, keys2.recv_key);
    }
}
