// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Onion encryption - Multi-layer encryption for circuits
//!
//! **Phase 2B**: Circuit building

use crate::circuit::CircuitHop;
use crate::crypto::BeardogCryptoClient;
use crate::error::{Error, Result};

/// Onion encryption handler
///
/// Manages multi-layer encryption for Tor circuits. Each circuit hop
/// has forward/backward keys and digest states. The sequence counter
/// tracks cell order for AES-CTR IV generation.
pub struct OnionCrypto {
    beardog: BeardogCryptoClient,
    /// Forward sequence counter (cells sent through circuit)
    forward_sequence: std::sync::atomic::AtomicU64,
    /// Backward sequence counter (cells received from circuit)
    backward_sequence: std::sync::atomic::AtomicU64,
}

impl OnionCrypto {
    /// Create new onion crypto handler
    #[must_use]
    pub const fn new(beardog: BeardogCryptoClient) -> Self {
        Self {
            beardog,
            forward_sequence: std::sync::atomic::AtomicU64::new(0),
            backward_sequence: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Encrypt cell with onion layers (client -> exit)
    ///
    /// Encrypts the cell payload with each hop's forward key in reverse order:
    /// Plaintext -> AES(hop3) -> AES(hop2) -> AES(hop1) -> Ciphertext
    ///
    /// The sequence counter is incremented per cell for unique AES-CTR IVs.
    ///
    /// # Arguments
    /// * `cell` - Plaintext cell payload
    /// * `hops` - Circuit hops (in order: guard, middle, exit)
    ///
    /// # Returns
    /// * Encrypted payload (onion-encrypted)
    ///
    /// # Errors
    /// Returns error if `BearDog` encryption fails or hop index overflows.
    pub fn encrypt_forward(&self, cell: &[u8], hops: &[CircuitHop]) -> Result<Vec<u8>> {
        let mut data = cell.to_vec();
        let seq = self.forward_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Encrypt in reverse order (exit -> middle -> guard)
        // So that guard peels first layer, middle peels second, exit gets plaintext
        for (hop_idx, hop) in hops.iter().rev().enumerate() {
            // Generate IV: unique per hop and per cell
            let hop_idx_u32 = u32::try_from(hop_idx)
                .map_err(|_| Error::Protocol("Hop index overflow".to_string()))?;
            let iv = Self::generate_iv(seq, hop_idx_u32);

            // Encrypt with this hop's forward key via BearDog
            data = self.beardog.aes_128_ctr_encrypt(&hop.forward_key, &iv, &data)?;

            // Running digest update (BearDog SHA3-256)
            // When BearDog is fully integrated:
            // hop.forward_digest = self.update_digest(&hop.forward_digest, &data)?;
        }

        Ok(data)
    }

    /// Decrypt cell removing onion layers (exit -> client)
    ///
    /// Decrypts the cell payload with each hop's backward key in forward order:
    /// Ciphertext -> `AES_decrypt(hop1)` -> `AES_decrypt(hop2)` -> `AES_decrypt(hop3)` -> Plaintext
    ///
    /// # Arguments
    /// * `cell` - Encrypted cell payload
    /// * `hops` - Circuit hops (in order: guard, middle, exit)
    ///
    /// # Returns
    /// * Decrypted payload
    ///
    /// # Errors
    /// Returns error if `BearDog` decryption fails or hop index overflows.
    pub fn decrypt_backward(&self, cell: &[u8], hops: &[CircuitHop]) -> Result<Vec<u8>> {
        let mut data = cell.to_vec();
        let seq = self.backward_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Decrypt in forward order (guard -> middle -> exit)
        for (hop_idx, hop) in hops.iter().enumerate() {
            // Generate IV: unique per hop and per cell
            let hop_idx_u32 = u32::try_from(hop_idx)
                .map_err(|_| Error::Protocol("Hop index overflow".to_string()))?;
            let iv = Self::generate_iv(seq, hop_idx_u32);

            // Decrypt with this hop's backward key via BearDog
            data = self.beardog.aes_128_ctr_decrypt(&hop.backward_key, &iv, &data)?;

            // Running digest update (BearDog SHA3-256)
            // When BearDog is fully integrated:
            // hop.backward_digest = self.update_digest(&hop.backward_digest, &data)?;
        }

        Ok(data)
    }

    /// Extract the 4-byte relay digest from a running digest state
    ///
    /// The relay cell `digest` field contains the first 4 bytes of the
    /// running SHA-1/SHA3-256 hash of all relay cell bodies through
    /// a given circuit hop. This is used for integrity verification.
    ///
    /// # Arguments
    /// * `running_digest` - Current running digest state (32 bytes)
    ///
    /// # Returns
    /// * 4-byte digest for the relay cell header
    #[must_use]
    pub fn extract_relay_digest(running_digest: &[u8; 32]) -> [u8; 4] {
        let mut digest = [0u8; 4];
        digest.copy_from_slice(&running_digest[..4]);
        digest
    }

    /// Generate IV for AES-CTR encryption
    ///
    /// Uses a combined sequence + hop counter scheme:
    /// - bytes 0..8: cell sequence counter (big-endian u64)
    /// - bytes 8..12: hop index (big-endian u32)
    /// - bytes 12..16: reserved (zeros)
    fn generate_iv(sequence: u64, hop_index: u32) -> [u8; 16] {
        let mut iv = [0u8; 16];
        iv[0..8].copy_from_slice(&sequence.to_be_bytes());
        iv[8..12].copy_from_slice(&hop_index.to_be_bytes());
        iv
    }

    /// Update running digest (for integrity)
    ///
    /// Computes: `new_digest` = SHA3-256(current_digest || `cell_data`)
    /// This maintains a running hash of all relay cell data through
    /// each circuit hop, used for integrity verification.
    ///
    /// Requires `BearDog` SHA3-256 integration.
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    fn update_digest(&self, current_digest: &[u8; 32], data: &[u8]) -> Result<[u8; 32]> {
        let input = [&current_digest[..], data].concat();
        self.beardog.sha3_256(&input)
    }

    /// Get current forward sequence counter
    pub fn forward_sequence(&self) -> u64 {
        self.forward_sequence.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get current backward sequence counter
    pub fn backward_sequence(&self) -> u64 {
        self.backward_sequence.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::RelayInfo;
    use std::net::IpAddr;

    fn create_test_hop(id: u8) -> CircuitHop {
        let relay = RelayInfo {
            nickname: format!("relay{id}"),
            fingerprint: [id; 20],
            address: IpAddr::from([127, 0, 0, 1]),
            or_port: 9001,
            dir_port: None,
            flags: crate::directory::RelayFlags::empty(),
            bandwidth: 1_000_000,
            ntor_key: None,
            version: None,
        };

        CircuitHop::new(
            relay, [id; 32], // forward_digest
            [id; 32], // backward_digest
            [id; 16], // forward_key
            [id; 16], // backward_key
        )
    }

    #[test]
    fn test_iv_generation() {
        let beardog = BeardogCryptoClient::from_env().expect("Failed to create BearDog client");
        let _crypto = OnionCrypto::new(beardog);

        let iv = OnionCrypto::generate_iv(12345, 0);
        assert_eq!(iv.len(), 16);

        // Check sequence counter bytes
        assert_eq!(u64::from_be_bytes(iv[0..8].try_into().expect("slice length 8")), 12345);

        // Check hop index bytes
        assert_eq!(u32::from_be_bytes(iv[8..12].try_into().expect("slice length 4")), 0);

        // Check reserved bytes are zero
        assert_eq!(&iv[12..16], &[0u8; 4]);

        // Different hops get different IVs
        let iv2 = OnionCrypto::generate_iv(12345, 1);
        assert_ne!(iv, iv2);
    }

    #[test]
    fn test_onion_crypto_creation() {
        let beardog = BeardogCryptoClient::from_env().expect("Failed to create BearDog client");
        let crypto = OnionCrypto::new(beardog);

        assert_eq!(crypto.forward_sequence(), 0);
        assert_eq!(crypto.backward_sequence(), 0);
    }

    #[test]
    fn test_relay_digest_extraction() {
        let digest = [
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
            0xFF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
            0x0D, 0x0E, 0x0F, 0x10,
        ];

        let relay_digest = OnionCrypto::extract_relay_digest(&digest);
        assert_eq!(relay_digest, [0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    #[ignore = "Requires BearDog AES-128-CTR implementation"]
    fn test_encrypt_decrypt_roundtrip() {
        let beardog = BeardogCryptoClient::from_env().expect("Failed to create BearDog client");
        let crypto = OnionCrypto::new(beardog);

        let hops = vec![create_test_hop(1), create_test_hop(2), create_test_hop(3)];

        let plaintext = b"Hello, Tor!";

        // Encrypt
        let encrypted = crypto.encrypt_forward(plaintext, &hops).expect("Encryption failed");

        // Decrypt
        let decrypted = crypto.decrypt_backward(&encrypted, &hops).expect("Decryption failed");

        assert_eq!(decrypted, plaintext);
    }
}
