//! Onion identity key management

use crate::address::derive_onion_address;
use crate::error::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Onion service identity (Ed25519 keypair + derived .onion address)
#[derive(Debug, Clone)]
pub struct OnionIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    onion_address: String,
    created_at: u64,
}

/// Serializable format for storing identity
#[derive(Debug, Serialize, Deserialize)]
struct StoredIdentity {
    secret_key_bytes: [u8; 32],
    created_at: u64,
}

impl OnionIdentity {
    /// Generate new random onion identity
    ///
    /// # Example
    ///
    /// ```
    /// use songbird_sovereign_onion::OnionIdentity;
    ///
    /// let identity = OnionIdentity::generate();
    /// println!("Onion address: {}", identity.onion_address());
    /// ```
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
            signing_key,
            verifying_key,
            onion_address,
            created_at,
        }
    }

    /// Load identity from stored secret key bytes
    ///
    /// # Errors
    ///
    /// Returns error if key bytes are invalid
    pub fn from_stored(secret_bytes: &[u8; 32], created_at: u64) -> Result<Self> {
        let signing_key =
            SigningKey::from_bytes(secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let onion_address = derive_onion_address(&verifying_key);

        Ok(Self {
            signing_key,
            verifying_key,
            onion_address,
            created_at,
        })
    }

    /// Get .onion address
    pub fn onion_address(&self) -> &str {
        &self.onion_address
    }

    /// Get Ed25519 verifying (public) key
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Get Ed25519 signing (secret) key
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Get creation timestamp (Unix seconds)
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Serialize for storage
    pub(crate) fn to_stored_bytes(&self) -> Vec<u8> {
        let stored = StoredIdentity {
            secret_key_bytes: self.signing_key.to_bytes(),
            created_at: self.created_at,
        };
        serde_json::to_vec(&stored).unwrap()
    }

    /// Deserialize from storage
    pub(crate) fn from_stored_bytes(bytes: &[u8]) -> Result<Self> {
        let stored: StoredIdentity = serde_json::from_slice(bytes)?;
        Self::from_stored(&stored.secret_key_bytes, stored.created_at)
    }
}

/// X25519 ephemeral keypair for session key exchange
pub struct EphemeralKeypair {
    secret: x25519_dalek::EphemeralSecret,
    public: [u8; 32],
}

impl EphemeralKeypair {
    /// Generate new random ephemeral keypair
    pub fn generate() -> Self {
        let secret = x25519_dalek::EphemeralSecret::random_from_rng(rand::thread_rng());
        let public = x25519_dalek::PublicKey::from(&secret).to_bytes();

        Self { secret, public }
    }

    /// Get public key bytes
    pub fn public_bytes(&self) -> &[u8; 32] {
        &self.public
    }

    /// Perform X25519 ECDH to derive shared secret
    pub fn derive_shared_secret(self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer_key = x25519_dalek::PublicKey::from(*peer_public);
        let shared = self.secret.diffie_hellman(&peer_key);
        shared.to_bytes()
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
    /// Derive session keys from shared secret using HKDF-SHA256
    ///
    /// # Arguments
    ///
    /// * `shared_secret` - X25519 ECDH result
    /// * `client_nonce` - Random nonce from client
    /// * `server_nonce` - Random nonce from server
    /// * `is_client` - True if we are the client (affects key assignment)
    pub fn derive(
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Self {
        use hmac::{Hmac, Mac};
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
            original.verifying_key().as_bytes(),
            restored.verifying_key().as_bytes()
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
