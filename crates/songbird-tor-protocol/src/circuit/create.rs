// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit creation - ntor handshake implementation
//!
//! **Phase 2B**: Circuit building

use crate::crypto::TorProtocolCrypto;
use crate::error::{Error, Result};
use songbird_crypto_provider::CryptoProvider;

/// ntor handshake protocol for circuit creation
///
/// **TRUE PRIMAL**: All crypto operations delegated to the security provider.
pub struct NtorHandshake {
    security_provider: CryptoProvider,
}

impl NtorHandshake {
    /// Create new ntor handshake handler
    #[must_use]
    pub const fn new(security_provider: CryptoProvider) -> Self {
        Self {
            security_provider,
        }
    }

    /// Client side: Create CREATE2 payload
    ///
    /// # Arguments
    /// * `node_id` - Relay's 20-byte identity fingerprint (SHA1 of RSA identity key)
    /// * `relay_ntor_key` - Relay's X25519 ntor onion key (32 bytes, from descriptor)
    ///
    /// # Returns
    /// * CREATE2 payload (84 bytes: 20 + 32 + 32)
    /// * `HandshakeState` for processing CREATED2 response
    ///
    /// # Errors
    /// Returns error if `security provider` crypto operations fail.
    pub async fn create_handshake(
        &self,
        node_id: &[u8; 20],
        relay_ntor_key: &[u8; 32],
    ) -> Result<(Vec<u8>, HandshakeState)> {
        // 1. Generate ephemeral X25519 keypair via security provider
        let client_ephemeral = self.security_provider.x25519_generate_ephemeral().await?;

        // 2. Construct CREATE2 payload (84 bytes per Tor ntor spec)
        // Format: ID (20 bytes) || B (32 bytes) || X (32 bytes)
        // - ID: relay's identity (fingerprint)
        // - B: relay's ntor onion key
        // - X: client's ephemeral public key
        let mut payload = Vec::with_capacity(84);
        payload.extend_from_slice(node_id); // 20 bytes (node ID)
        payload.extend_from_slice(relay_ntor_key); // 32 bytes (B)
        payload.extend_from_slice(&client_ephemeral.public_key); // 32 bytes (X)

        // 3. Save state for CREATED2 processing
        let state = HandshakeState {
            client_ephemeral_secret: client_ephemeral.secret_key,
            client_ephemeral_public: client_ephemeral.public_key,
            node_id: *node_id,
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
    ///
    /// # Errors
    /// Returns error if response is invalid or auth verification fails.
    pub async fn complete_handshake(
        &self,
        state: &HandshakeState,
        response: &[u8],
    ) -> Result<KeyMaterial> {
        // Validate response length
        if response.len() != 64 {
            return Err(Error::Protocol(format!(
                "Invalid CREATED2 response length: {} (expected 64)",
                response.len()
            )));
        }

        // Parse response: Y (server ephemeral public) || AUTH (32 bytes)
        let server_pubkey: [u8; 32] = response[0..32]
            .try_into()
            .map_err(|_| Error::Protocol(String::from("Failed to parse server pubkey")))?;
        let auth: [u8; 32] = response[32..64]
            .try_into()
            .map_err(|_| Error::Protocol(String::from("Failed to parse auth")))?;

        // Per Tor ntor spec, compute TWO shared secrets:
        // 1. EXP(Y,x) - our ephemeral secret with server's ephemeral public
        // 2. EXP(B,x) - our ephemeral secret with server's static ntor key
        let xy = self
            .security_provider
            .x25519_derive_secret(&state.client_ephemeral_secret, &server_pubkey)
            .await?;

        let xb = self
            .security_provider
            .x25519_derive_secret(&state.client_ephemeral_secret, &state.relay_ntor_key)
            .await?;

        // secret_input = EXP(Y,x) || EXP(B,x) || ID || B || X || Y || PROTOID
        // PROTOID = "ntor-curve25519-sha256-1" (uses SHA256, not SHA3)
        let protoid = b"ntor-curve25519-sha256-1";
        let secret_input = [
            &xy[..],                            // 32 bytes - EXP(Y,x)
            &xb[..],                            // 32 bytes - EXP(B,x)
            &state.node_id[..],                 // 20 bytes (node ID / fingerprint)
            &state.relay_ntor_key[..],          // 32 bytes (B)
            &state.client_ephemeral_public[..], // 32 bytes (X)
            &server_pubkey[..],                 // 32 bytes (Y)
            protoid.as_slice(),
        ]
        .concat();

        // KEY_SEED = H(secret_input, t_key) using HMAC-SHA256
        // t_key = "ntor-curve25519-sha256-1:key_extract"
        let key_seed = self
            .security_provider
            .hmac_sha256(b"ntor-curve25519-sha256-1:key_extract", &secret_input)
            .await?;

        // verify = H(secret_input, t_verify)
        // t_verify = "ntor-curve25519-sha256-1:verify"
        let verify = self
            .security_provider
            .hmac_sha256(b"ntor-curve25519-sha256-1:verify", &secret_input)
            .await?;

        // auth_input = verify | ID | B | Y | X | PROTOID | "Server"
        let auth_input = [
            &verify[..],
            &state.node_id[..],                 // 20 bytes (ID)
            &state.relay_ntor_key[..],          // 32 bytes (B)
            &server_pubkey[..],                 // 32 bytes (Y)
            &state.client_ephemeral_public[..], // 32 bytes (X)
            protoid.as_slice(),
            b"Server",
        ]
        .concat();

        // AUTH = H(auth_input, t_mac)
        // t_mac = "ntor-curve25519-sha256-1:mac"
        let expected_auth = self
            .security_provider
            .hmac_sha256(b"ntor-curve25519-sha256-1:mac", &auth_input)
            .await?;

        if auth != expected_auth {
            return Err(Error::Protocol(String::from("ntor auth verification failed")));
        }

        // Derive forward/backward keys via RFC5869 HKDF
        self.derive_circuit_keys(&key_seed).await
    }

    /// Derive circuit keys from `KEY_SEED` using Tor's HKDF-like expansion
    ///
    /// Per Tor spec, uses HMAC-SHA256 based expansion:
    ///   K = `K_1` | `K_2` | `K_3` | ...
    ///   `K_1` = H(`m_expand` | INT8(1), `KEY_SEED`)
    ///   `K_i` = H(`K_{i-1}` | `m_expand` | INT8(i), `KEY_SEED`)
    ///
    /// Output (72 bytes):
    /// - Df (20 bytes): forward digest seed
    /// - Db (20 bytes): backward digest seed  
    /// - Kf (16 bytes): forward AES-128 key
    /// - Kb (16 bytes): backward AES-128 key
    async fn derive_circuit_keys(&self, key_seed: &[u8; 32]) -> Result<KeyMaterial> {
        let m_expand = b"ntor-curve25519-sha256-1:key_expand";

        // Expand using HMAC-SHA256 with counter (Tor-style HKDF-expand)
        // Need 72 bytes = 3 rounds of SHA256 (32 bytes each)
        let mut expanded = Vec::with_capacity(96);
        let mut prev = Vec::new();

        for i in 1u8..=3 {
            // input = prev | m_expand | INT8(i)
            let mut input = prev.clone();
            input.extend_from_slice(m_expand);
            input.push(i);

            // K_i = HMAC-SHA256(key_seed, input)
            let k_i = self.security_provider.hmac_sha256(key_seed, &input).await?;
            expanded.extend_from_slice(&k_i);
            prev = k_i.to_vec();
        }

        // Extract keys per Tor spec order:
        // Df (20 bytes) | Db (20 bytes) | Kf (16 bytes) | Kb (16 bytes)
        // We pad the digest seeds to 32 bytes for our struct
        let mut forward_digest = [0u8; 32];
        forward_digest[..20].copy_from_slice(&expanded[0..20]);

        let mut backward_digest = [0u8; 32];
        backward_digest[..20].copy_from_slice(&expanded[20..40]);

        Ok(KeyMaterial {
            forward_digest,
            backward_digest,
            forward_key: expanded[40..56]
                .try_into()
                .map_err(|_| Error::Crypto(String::from("Failed to extract forward_key")))?,
            backward_key: expanded[56..72]
                .try_into()
                .map_err(|_| Error::Crypto(String::from("Failed to extract backward_key")))?,
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
    /// Relay's node ID (20-byte fingerprint)
    pub node_id: [u8; 20],
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
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::error::Error;

    #[test]
    fn test_handshake_state_creation() {
        let state = HandshakeState {
            client_ephemeral_secret: [0u8; 32],
            client_ephemeral_public: [1u8; 32],
            node_id: [2u8; 20],
            relay_ntor_key: [3u8; 32],
        };

        assert_eq!(state.client_ephemeral_secret, [0u8; 32]);
        assert_eq!(state.node_id, [2u8; 20]);
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

    #[tokio::test]
    async fn complete_handshake_rejects_empty_response() {
        let ntor = NtorHandshake::new(CryptoProvider::new(String::from(
            "/tmp/songbird-tor-protocol-ntor-test.sock",
        )));
        let state = HandshakeState {
            client_ephemeral_secret: [9u8; 32],
            client_ephemeral_public: [8u8; 32],
            node_id: [7u8; 20],
            relay_ntor_key: [6u8; 32],
        };
        let err = ntor.complete_handshake(&state, &[]).await.expect_err("empty");
        assert!(
            matches!(err, Error::Protocol(ref s) if s.contains("64")),
            "unexpected err: {err:?}"
        );
    }

    #[tokio::test]
    async fn complete_handshake_rejects_non_64_byte_response() {
        let ntor = NtorHandshake::new(CryptoProvider::new(String::from(
            "/tmp/songbird-tor-protocol-ntor-test.sock",
        )));
        let state = HandshakeState {
            client_ephemeral_secret: [1u8; 32],
            client_ephemeral_public: [2u8; 32],
            node_id: [3u8; 20],
            relay_ntor_key: [4u8; 32],
        };
        let wrong = [0xabu8; 63];
        let err = ntor.complete_handshake(&state, &wrong).await.expect_err("wrong len");
        assert!(matches!(err, Error::Protocol(_)));

        let wrong65 = [0xabu8; 65];
        let err2 = ntor.complete_handshake(&state, &wrong65).await.expect_err("wrong len 65");
        assert!(matches!(err2, Error::Protocol(_)));
    }
}
