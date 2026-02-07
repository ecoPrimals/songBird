//! Circuit creation - ntor handshake implementation
//!
//! **Phase 2B**: Circuit building

use crate::crypto::BeardogCryptoClient;
use crate::error::{Error, Result};

/// ntor handshake protocol for circuit creation
///
/// **TRUE PRIMAL**: All crypto operations delegated to BearDog.
pub struct NtorHandshake {
    beardog: BeardogCryptoClient,
}

impl NtorHandshake {
    /// Create new ntor handshake handler
    pub fn new(beardog: BeardogCryptoClient) -> Self {
        Self { beardog }
    }

    /// Client side: Create CREATE2 payload
    ///
    /// # Arguments
    /// * `relay_identity` - Relay's Ed25519 identity key (32 bytes, from consensus)
    /// * `relay_ntor_key` - Relay's X25519 ntor onion key (32 bytes, from descriptor)
    ///
    /// # Returns
    /// * CREATE2 payload (84 bytes)
    /// * HandshakeState for processing CREATED2 response
    pub fn create_handshake(
        &self,
        relay_identity: &[u8; 32],
        relay_ntor_key: &[u8; 32],
    ) -> Result<(Vec<u8>, HandshakeState)> {
        // 1. Generate ephemeral X25519 keypair via BearDog
        let client_ephemeral = self.beardog.x25519_generate_ephemeral()?;

        // 2. Construct CREATE2 payload (84 bytes)
        let mut payload = Vec::with_capacity(84);
        payload.extend_from_slice(relay_identity);               // 32 bytes
        payload.extend_from_slice(relay_ntor_key);               // 32 bytes
        payload.extend_from_slice(&client_ephemeral.public_key); // 32 bytes

        // 3. Save state for CREATED2 processing
        let state = HandshakeState {
            client_ephemeral_secret: client_ephemeral.secret_key,
            client_ephemeral_public: client_ephemeral.public_key,
            relay_identity: *relay_identity,
            relay_ntor_key: *relay_ntor_key,
        };

        Ok((payload, state))
    }

    /// Client side: Process CREATED2 response
    ///
    /// # Arguments
    /// * `state` - Handshake state from `create_handshake()`
    /// * `response` - CREATED2 payload (64 bytes)
    ///
    /// # Returns
    /// * Key material for circuit encryption
    pub fn complete_handshake(
        &self,
        state: HandshakeState,
        response: &[u8],
    ) -> Result<KeyMaterial> {
        // Validate response length
        if response.len() != 64 {
            return Err(Error::Protocol(format!(
                "Invalid CREATED2 response length: {} (expected 64)",
                response.len()
            )));
        }

        // Parse response
        let server_pubkey: [u8; 32] = response[0..32].try_into()
            .map_err(|_| Error::Protocol("Failed to parse server pubkey".to_string()))?;
        let auth: [u8; 32] = response[32..64].try_into()
            .map_err(|_| Error::Protocol("Failed to parse auth".to_string()))?;

        // 2. Derive shared secret via BearDog X25519 (ECDH)
        let shared_secret = self.beardog.x25519_derive_secret(
            &state.client_ephemeral_secret,
            &server_pubkey,
        )?;

        // 3. Compute key material via BearDog SHA3-256 (KDF)
        let secret_input = [
            &shared_secret[..],
            &state.relay_identity[..],
            &state.relay_ntor_key[..],
            &state.client_ephemeral_public[..],
            &server_pubkey[..],
            b"ntor-curve25519-sha3-256-1",
        ].concat();

        let key_material = self.beardog.sha3_256(&secret_input)?;

        // 4. Verify auth
        let expected_auth_input = [
            &key_material[..],
            &state.relay_identity[..],
            &state.relay_ntor_key[..],
            &server_pubkey[..],
            &state.client_ephemeral_public[..],
            b"ntor-curve25519-sha3-256-1:verify",
        ].concat();

        let expected_auth = self.beardog.sha3_256(&expected_auth_input)?;

        if auth != expected_auth {
            return Err(Error::Protocol("ntor auth verification failed".to_string()));
        }

        // 5. Derive forward/backward keys via KDF
        self.derive_circuit_keys(&key_material)
    }

    /// Derive circuit keys from key material (HKDF-style expansion)
    ///
    /// Output: 5 * 32 bytes = 160 bytes total
    /// - Forward digest init (32 bytes)
    /// - Backward digest init (32 bytes)
    /// - Forward AES key (16 bytes from 32)
    /// - Backward AES key (16 bytes from 32)
    /// - KDF IV (32 bytes)
    fn derive_circuit_keys(&self, key_material: &[u8; 32]) -> Result<KeyMaterial> {
        let mut expanded = Vec::with_capacity(160);
        let mut prev = key_material.to_vec();

        // HKDF-style expansion using SHA3-256
        for i in 0..5 {
            let input = [
                &prev[..],
                &[i as u8],
                b"ntor-curve25519-sha3-256-1:key_expand",
            ].concat();

            prev = self.beardog.sha3_256(&input)?.to_vec();
            expanded.extend_from_slice(&prev);
        }

        // Extract keys from expanded material
        Ok(KeyMaterial {
            forward_digest: expanded[0..32].try_into()
                .map_err(|_| Error::Crypto("Failed to extract forward_digest".to_string()))?,
            backward_digest: expanded[32..64].try_into()
                .map_err(|_| Error::Crypto("Failed to extract backward_digest".to_string()))?,
            forward_key: expanded[64..80].try_into()
                .map_err(|_| Error::Crypto("Failed to extract forward_key".to_string()))?,
            backward_key: expanded[96..112].try_into()
                .map_err(|_| Error::Crypto("Failed to extract backward_key".to_string()))?,
        })
    }
}

/// Handshake state for processing CREATED2 response
#[derive(Debug, Clone)]
pub struct HandshakeState {
    /// Client's ephemeral secret key
    pub client_ephemeral_secret: [u8; 32],
    /// Client's ephemeral public key
    pub client_ephemeral_public: [u8; 32],
    /// Relay's identity key
    pub relay_identity: [u8; 32],
    /// Relay's ntor key
    pub relay_ntor_key: [u8; 32],
}

/// Key material derived from ntor handshake
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    /// Forward digest initialization (32 bytes)
    pub forward_digest: [u8; 32],
    /// Backward digest initialization (32 bytes)
    pub backward_digest: [u8; 32],
    /// Forward AES-128 key (16 bytes)
    pub forward_key: [u8; 16],
    /// Backward AES-128 key (16 bytes)
    pub backward_key: [u8; 16],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_state_creation() {
        let state = HandshakeState {
            client_ephemeral_secret: [0u8; 32],
            client_ephemeral_public: [1u8; 32],
            relay_identity: [2u8; 32],
            relay_ntor_key: [3u8; 32],
        };

        assert_eq!(state.client_ephemeral_secret, [0u8; 32]);
        assert_eq!(state.relay_identity, [2u8; 32]);
    }

    #[test]
    fn test_key_material_creation() {
        let keys = KeyMaterial {
            forward_digest: [0u8; 32],
            backward_digest: [1u8; 32],
            forward_key: [2u8; 16],
            backward_key: [3u8; 16],
        };

        assert_eq!(keys.forward_digest.len(), 32);
        assert_eq!(keys.forward_key.len(), 16);
    }
}
