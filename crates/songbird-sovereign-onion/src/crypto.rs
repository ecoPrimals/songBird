//! Cryptographic operations (ChaCha20-Poly1305 AEAD)

use crate::beardog_crypto::BeardogCryptoClient;
use crate::error::{OnionError, Result};

#[cfg(feature = "standalone")]
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};

/// Encrypt data via BearDog (TRUE PRIMAL)
///
/// # Arguments
///
/// * `client` - BearDog crypto client
/// * `key` - 32-byte encryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `plaintext` - Data to encrypt
///
/// # Returns
///
/// Ciphertext with 16-byte Poly1305 MAC tag appended
pub fn encrypt_data_via_beardog(
    client: &BeardogCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());
    
    client.chacha20_poly1305_encrypt(key, &nonce, plaintext)
}

/// Decrypt data via BearDog (TRUE PRIMAL)
///
/// # Arguments
///
/// * `client` - BearDog crypto client
/// * `key` - 32-byte decryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `ciphertext` - Encrypted data with MAC tag
///
/// # Returns
///
/// Decrypted plaintext (MAC verified)
pub fn decrypt_data_via_beardog(
    client: &BeardogCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());
    
    client.chacha20_poly1305_decrypt(key, &nonce, ciphertext)
}

/// Standalone: Encrypt data with ChaCha20-Poly1305
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `plaintext` - Data to encrypt
///
/// # Returns
///
/// Ciphertext with 16-byte Poly1305 MAC tag appended
#[cfg(feature = "standalone")]
pub fn encrypt_data(key: &[u8; 32], sequence: u64, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());

    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&sequence.to_le_bytes());
    let nonce = Nonce::from(nonce_bytes);

    cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| OnionError::EncryptionError("ChaCha20-Poly1305 encryption failed".into()))
}

/// Standalone: Decrypt data with ChaCha20-Poly1305
///
/// # Arguments
///
/// * `key` - 32-byte decryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `ciphertext` - Encrypted data with MAC tag
///
/// # Returns
///
/// Decrypted plaintext (MAC verified)
#[cfg(feature = "standalone")]
pub fn decrypt_data(key: &[u8; 32], sequence: u64, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());

    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&sequence.to_le_bytes());
    let nonce = Nonce::from(nonce_bytes);

    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| OnionError::DecryptionError("ChaCha20-Poly1305 decryption failed (MAC verification failed)".into()))
}

#[cfg(all(test, feature = "standalone"))]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0x42u8; 32];
        let sequence = 1;
        let plaintext = b"Hello, Sovereign Onion!";

        let ciphertext = encrypt_data(&key, sequence, plaintext).unwrap();
        assert_ne!(ciphertext.as_slice(), plaintext); // Should be different
        assert_eq!(ciphertext.len(), plaintext.len() + 16); // +16 for Poly1305 tag

        let decrypted = decrypt_data(&key, sequence, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let sequence = 1;
        let plaintext = b"Secret message";

        let ciphertext = encrypt_data(&key1, sequence, plaintext).unwrap();
        let result = decrypt_data(&key2, sequence, &ciphertext);

        assert!(result.is_err());
        assert!(matches!(result, Err(OnionError::DecryptionError(_))));
    }

    #[test]
    fn test_decrypt_wrong_sequence() {
        let key = [0x42u8; 32];
        let plaintext = b"Test";

        let ciphertext = encrypt_data(&key, 1, plaintext).unwrap();
        let result = decrypt_data(&key, 2, &ciphertext);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        let key = [0x42u8; 32];
        let plaintext = b"Test";

        let mut ciphertext = encrypt_data(&key, 1, plaintext).unwrap();
        ciphertext[0] ^= 0xFF; // Corrupt one byte

        let result = decrypt_data(&key, 1, &ciphertext);
        assert!(result.is_err());
    }
}
