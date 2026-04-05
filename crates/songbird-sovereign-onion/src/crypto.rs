// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Cryptographic operations (ChaCha20-Poly1305 AEAD)

use crate::error::Result;
use crate::security_crypto::SecurityCryptoClient;

#[cfg(feature = "standalone")]
use crate::OnionError;

#[cfg(feature = "standalone")]
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};

/// Encrypt data via the security provider
///
/// # Arguments
///
/// * `client` - delegated crypto client
/// * `key` - 32-byte encryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `plaintext` - Data to encrypt
///
/// # Returns
///
/// Ciphertext with 16-byte Poly1305 MAC tag appended
///
/// # Errors
///
/// Returns an error if encryption RPC fails.
pub async fn encrypt_data_via_security_provider(
    client: &SecurityCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());

    client.chacha20_poly1305_encrypt(key, &nonce, plaintext).await
}

/// Decrypt data via the security provider
///
/// # Arguments
///
/// * `client` - delegated crypto client
/// * `key` - 32-byte decryption key
/// * `sequence` - Monotonic sequence number (for nonce)
/// * `ciphertext` - Encrypted data with MAC tag
///
/// # Returns
///
/// Decrypted plaintext (MAC verified)
///
/// # Errors
///
/// Returns an error if decryption or MAC verification fails.
pub async fn decrypt_data_via_security_provider(
    client: &SecurityCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());

    client.chacha20_poly1305_decrypt(key, &nonce, ciphertext).await
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
///
/// # Errors
///
/// Returns an error if encryption fails.
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
///
/// # Errors
///
/// Returns an error if decryption or MAC verification fails.
#[cfg(feature = "standalone")]
pub fn decrypt_data(key: &[u8; 32], sequence: u64, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());

    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&sequence.to_le_bytes());
    let nonce = Nonce::from(nonce_bytes);

    cipher.decrypt(&nonce, ciphertext).map_err(|_| {
        OnionError::DecryptionError(
            "ChaCha20-Poly1305 decryption failed (MAC verification failed)".into(),
        )
    })
}

#[cfg(all(test, feature = "standalone"))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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

#[cfg(test)]
mod security_crypto_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::security_crypto::SecurityCryptoClient;

    #[tokio::test(start_paused = true)]
    async fn encrypt_via_security_provider_errors_when_rpc_unreachable() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let key = [9u8; 32];
        let r = encrypt_data_via_security_provider(&client, &key, 0, b"data").await;
        assert!(r.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn decrypt_via_security_provider_errors_when_rpc_unreachable() {
        let client =
            SecurityCryptoClient::from_neural_api_socket("/tmp/songbird-onion-test-invalid.sock");
        let key = [9u8; 32];
        let r = decrypt_data_via_security_provider(&client, &key, 0, &[0u8; 16]).await;
        assert!(r.is_err());
    }
}
