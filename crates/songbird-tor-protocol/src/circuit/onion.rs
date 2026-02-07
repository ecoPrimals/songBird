//! Onion encryption - Multi-layer encryption for circuits
//!
//! **Phase 2B**: Circuit building

use crate::crypto::BeardogCryptoClient;
use crate::circuit::CircuitHop;
use crate::error::Result;

/// Onion encryption handler
pub struct OnionCrypto {
    beardog: BeardogCryptoClient,
}

impl OnionCrypto {
    /// Create new onion crypto handler
    pub fn new(beardog: BeardogCryptoClient) -> Self {
        Self { beardog }
    }

    /// Encrypt cell with onion layers (client → exit)
    ///
    /// Encrypts the cell payload with each hop's forward key in reverse order:
    /// Plaintext → AES(hop3) → AES(hop2) → AES(hop1) → Ciphertext
    ///
    /// # Arguments
    /// * `cell` - Plaintext cell payload
    /// * `hops` - Circuit hops (in order: guard, middle, exit)
    ///
    /// # Returns
    /// * Encrypted payload (onion-encrypted)
    pub fn encrypt_forward(&self, cell: &[u8], hops: &[CircuitHop]) -> Result<Vec<u8>> {
        let mut data = cell.to_vec();

        // Encrypt in reverse order (exit → middle → guard)
        // So that guard peels first layer, middle peels second, exit gets plaintext
        for hop in hops.iter().rev() {
            // Generate IV from sequence counter (simplified)
            let iv = self.generate_iv(0); // TODO: Use actual sequence counter

            // Encrypt with this hop's forward key via BearDog
            data = self.beardog.aes_128_ctr_encrypt(&hop.forward_key, &iv, &data)?;

            // Update forward digest (TODO: implement running digest)
            // let new_digest = self.beardog.sha3_256(&[&hop.forward_digest[..], &data[..]].concat())?;
        }

        Ok(data)
    }

    /// Decrypt cell removing onion layers (exit → client)
    ///
    /// Decrypts the cell payload with each hop's backward key in forward order:
    /// Ciphertext → AES_decrypt(hop1) → AES_decrypt(hop2) → AES_decrypt(hop3) → Plaintext
    ///
    /// # Arguments
    /// * `cell` - Encrypted cell payload
    /// * `hops` - Circuit hops (in order: guard, middle, exit)
    ///
    /// # Returns
    /// * Decrypted payload
    pub fn decrypt_backward(&self, cell: &[u8], hops: &[CircuitHop]) -> Result<Vec<u8>> {
        let mut data = cell.to_vec();

        // Decrypt in forward order (guard → middle → exit)
        for hop in hops.iter() {
            // Generate IV from sequence counter (simplified)
            let iv = self.generate_iv(0); // TODO: Use actual sequence counter

            // Decrypt with this hop's backward key via BearDog
            data = self.beardog.aes_128_ctr_decrypt(&hop.backward_key, &iv, &data)?;

            // Update backward digest (TODO: implement running digest)
            // let new_digest = self.beardog.sha3_256(&[&hop.backward_digest[..], &data[..]].concat())?;
        }

        Ok(data)
    }

    /// Generate IV for AES-CTR encryption
    ///
    /// Tor uses a counter-based IV scheme:
    /// - 8 bytes: sequence counter (big-endian)
    /// - 4 bytes: reserved (zeros)
    /// - 4 bytes: cell counter (zeros for now)
    fn generate_iv(&self, sequence: u64) -> [u8; 16] {
        let mut iv = [0u8; 16];
        iv[0..8].copy_from_slice(&sequence.to_be_bytes());
        iv
    }

    /// Update running digest (for integrity)
    ///
    /// TODO: Implement proper running digest calculation using SHA3-256
    #[allow(dead_code)]
    fn update_digest(&self, current_digest: &[u8; 32], data: &[u8]) -> Result<[u8; 32]> {
        let input = [&current_digest[..], data].concat();
        self.beardog.sha3_256(&input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::RelayInfo;
    use std::net::IpAddr;

    fn create_test_hop(id: u8) -> CircuitHop {
        let relay = RelayInfo {
            nickname: format!("relay{}", id),
            fingerprint: [id; 20],
            address: IpAddr::from([127, 0, 0, 1]),
            or_port: 9001,
            dir_port: None,
            flags: crate::directory::RelayFlags::empty(),
            bandwidth: 1000000,
            ntor_key: None,
            version: None,
        };

        CircuitHop::new(
            relay,
            [id; 32],      // forward_digest
            [id; 32],      // backward_digest
            [id; 16],      // forward_key
            [id; 16],      // backward_key
        )
    }

    #[test]
    fn test_iv_generation() {
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
        let crypto = OnionCrypto::new(beardog);

        let iv = crypto.generate_iv(12345);
        assert_eq!(iv.len(), 16);
        
        // Check sequence counter bytes
        assert_eq!(u64::from_be_bytes(iv[0..8].try_into().unwrap()), 12345);
        
        // Check reserved bytes are zero
        assert_eq!(&iv[8..16], &[0u8; 8]);
    }

    #[test]
    fn test_onion_crypto_creation() {
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
        let _crypto = OnionCrypto::new(beardog);
        
        // Test passes if it creates successfully
    }

    #[test]
    #[ignore = "Requires BearDog AES-128-CTR implementation"]
    fn test_encrypt_decrypt_roundtrip() {
        let beardog = BeardogCryptoClient::from_env()
            .expect("Failed to create BearDog client");
        let crypto = OnionCrypto::new(beardog);

        let hops = vec![
            create_test_hop(1),
            create_test_hop(2),
            create_test_hop(3),
        ];

        let plaintext = b"Hello, Tor!";
        
        // Encrypt
        let encrypted = crypto.encrypt_forward(plaintext, &hops)
            .expect("Encryption failed");
        
        // Decrypt
        let decrypted = crypto.decrypt_backward(&encrypted, &hops)
            .expect("Decryption failed");
        
        assert_eq!(decrypted, plaintext);
    }
}
