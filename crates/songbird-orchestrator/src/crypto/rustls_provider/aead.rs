//! AEAD (Authenticated Encryption with Associated Data) implementation for TLS 1.3
//!
//! This module implements ChaCha20-Poly1305 AEAD for rustls, delegating all
//! cryptographic operations to a capability-discovered crypto provider (typically BearDog).
//!
//! # Architecture
//!
//! ```text
//! rustls (TLS protocol)
//!     ↓ needs AEAD encryption/decryption
//! Tls13AeadAlgorithm (this module)
//!     ↓ creates encrypter/decrypter
//! MessageEncrypter/MessageDecrypter
//!     ↓ handles TLS nonce/AAD
//! CryptoProvider (runtime-discovered)
//!     ↓ performs actual crypto
//! BearDog (or any primal with crypto capability)
//! ```
//!
//! # Loose Coupling
//!
//! This implementation discovers the crypto provider at runtime via the
//! capability system. Songbird and BearDog are NOT embedded - they discover
//! each other through Unix sockets and capability advertisement.
//!
//! # TLS 1.3 AEAD Details
//!
//! Per RFC 8446 Section 5.3:
//! - Nonce: 12 bytes constructed as IV XOR sequence_number
//! - AAD: 5 bytes (TLS record header: type, version, length)
//! - Tag: 16 bytes (Poly1305 authentication tag)

use crate::crypto::provider::CryptoProvider;
use rustls::crypto::cipher::{
    make_tls13_aad, AeadKey, Iv, MessageDecrypter, MessageEncrypter, Tls13AeadAlgorithm,
    UnsupportedOperationError, InboundOpaqueMessage, InboundPlainMessage, 
    OutboundOpaqueMessage, OutboundPlainMessage, PrefixedPayload,
};
use rustls::ConnectionTrafficSecrets;
use rustls::Error;
use std::sync::Arc;

/// ChaCha20-Poly1305 AEAD algorithm for TLS 1.3
///
/// This struct implements the `Tls13AeadAlgorithm` trait, providing
/// encryption and decryption capabilities for TLS records using
/// ChaCha20-Poly1305 AEAD.
///
/// All cryptographic operations are delegated to a runtime-discovered
/// crypto provider (no hardcoded dependencies).
#[derive(Debug)]
pub struct BeardogChaCha20Poly1305;

impl Tls13AeadAlgorithm for BeardogChaCha20Poly1305 {
    fn encrypter(&self, key: AeadKey, iv: Iv) -> Box<dyn MessageEncrypter> {
        // Get runtime crypto provider (discovered via capability system)
        let crypto = super::kx_group::get_runtime_crypto_provider()
            .expect("Crypto provider must be initialized before creating encrypter");

        Box::new(BeardogChaCha20Encrypter {
            crypto,
            key: key.as_ref().to_vec(),
            iv: iv.as_ref().to_vec(),
        })
    }

    fn decrypter(&self, key: AeadKey, iv: Iv) -> Box<dyn MessageDecrypter> {
        // Get runtime crypto provider (discovered via capability system)
        let crypto = super::kx_group::get_runtime_crypto_provider()
            .expect("Crypto provider must be initialized before creating decrypter");

        Box::new(BeardogChaCha20Decrypter {
            crypto,
            key: key.as_ref().to_vec(),
            iv: iv.as_ref().to_vec(),
        })
    }

    fn key_len(&self) -> usize {
        32 // ChaCha20 uses 256-bit (32-byte) keys
    }

    fn extract_keys(
        &self,
        key: AeadKey,
        iv: Iv,
    ) -> Result<ConnectionTrafficSecrets, UnsupportedOperationError> {
        // TLS 1.3 with ChaCha20-Poly1305 uses standard key extraction
        Ok(ConnectionTrafficSecrets::Chacha20Poly1305 { key, iv })
    }
}

/// Static instance for use in cipher suites
pub static BEARDOG_CHACHA20_POLY1305: BeardogChaCha20Poly1305 = BeardogChaCha20Poly1305;

/// TLS 1.3 record encrypter using ChaCha20-Poly1305
///
/// This encrypter handles:
/// 1. TLS 1.3 nonce construction (IV XOR sequence number)
/// 2. AAD (Additional Authenticated Data) formatting
/// 3. Delegation to crypto provider for actual encryption
///
/// # Nonce Construction (RFC 8446 Section 5.3)
///
/// The nonce for each record is computed as:
/// ```text
/// nonce = per_record_nonce XOR sequence_number
/// ```
/// Where:
/// - `per_record_nonce` is the IV from key derivation (12 bytes)
/// - `sequence_number` is the TLS record sequence number (8 bytes, padded to 12)
struct BeardogChaCha20Encrypter {
    /// Runtime-discovered crypto provider (loose coupling!)
    crypto: Arc<dyn CryptoProvider>,
    /// Encryption key (32 bytes for ChaCha20)
    key: Vec<u8>,
    /// IV for nonce construction (12 bytes)
    iv: Vec<u8>,
}

impl MessageEncrypter for BeardogChaCha20Encrypter {
    fn encrypt(
        &mut self,
        msg: OutboundPlainMessage<'_>,
        seq: u64,
    ) -> Result<OutboundOpaqueMessage, Error> {
        // Construct TLS 1.3 nonce: IV XOR sequence_number
        let nonce = construct_tls13_nonce(&self.iv, seq);

        // Convert payload chunks to a single Vec for encryption
        let plaintext = msg.payload.to_vec();

        // Construct TLS 1.3 AAD (Additional Authenticated Data)
        // AAD format: TLS record header (type, version, length)
        let aad = make_tls13_aad(plaintext.len() + 16); // +16 for auth tag

        // Delegate encryption to crypto provider (runtime-discovered!)
        let (ciphertext, _nonce_returned, tag) = tokio::runtime::Handle::current()
            .block_on(async {
                self.crypto
                    .chacha20_poly1305_encrypt(&plaintext, &self.key, &nonce, Some(&aad[..]))
                    .await
            })
            .map_err(|e| Error::General(format!("AEAD encryption failed: {}", e)))?;

        // Construct TLS record: ciphertext || tag
        let mut payload = PrefixedPayload::with_capacity(ciphertext.len() + tag.len());
        payload.extend_from_slice(&ciphertext);
        payload.extend_from_slice(&tag);

        // Create opaque message with encrypted payload
        Ok(OutboundOpaqueMessage::new(
            msg.typ,
            msg.version,
            payload,
        ))
    }

    fn encrypted_payload_len(&self, payload_len: usize) -> usize {
        // TLS 1.3 encrypted record = plaintext + tag
        payload_len + 16 // Poly1305 tag is 16 bytes
    }
}

/// TLS 1.3 record decrypter using ChaCha20-Poly1305
///
/// This decrypter handles:
/// 1. TLS 1.3 nonce construction (IV XOR sequence number)
/// 2. AAD (Additional Authenticated Data) formatting
/// 3. Delegation to crypto provider for actual decryption
/// 4. Authentication tag verification
struct BeardogChaCha20Decrypter {
    /// Runtime-discovered crypto provider (loose coupling!)
    crypto: Arc<dyn CryptoProvider>,
    /// Decryption key (32 bytes for ChaCha20)
    key: Vec<u8>,
    /// IV for nonce construction (12 bytes)
    iv: Vec<u8>,
}

impl MessageDecrypter for BeardogChaCha20Decrypter {
    fn decrypt<'a>(
        &mut self,
        mut msg: InboundOpaqueMessage<'a>,
        seq: u64,
    ) -> Result<InboundPlainMessage<'a>, Error> {
        // Extract ciphertext and tag from payload
        let payload_len = msg.payload.len();
        if payload_len < 16 {
            return Err(Error::DecryptError);
        }

        let tag_start = payload_len - 16;
        
        // Copy ciphertext and tag out (we'll overwrite the buffer)
        let ciphertext = msg.payload[..tag_start].to_vec();
        let tag = msg.payload[tag_start..].to_vec();

        // Construct TLS 1.3 nonce: IV XOR sequence_number
        let nonce = construct_tls13_nonce(&self.iv, seq);

        // Construct TLS 1.3 AAD (Additional Authenticated Data)
        let aad = make_tls13_aad(payload_len);

        // Delegate decryption to crypto provider (runtime-discovered!)
        let plaintext = tokio::runtime::Handle::current()
            .block_on(async {
                self.crypto
                    .chacha20_poly1305_decrypt(&ciphertext, &self.key, &nonce, &tag, Some(&aad))
                    .await
            })
            .map_err(|_| Error::DecryptError)?; // Authentication failure

        // Write plaintext back into the message's payload buffer (decrypt in-place)
        if plaintext.len() > msg.payload.len() {
            return Err(Error::DecryptError);
        }
        msg.payload[..plaintext.len()].copy_from_slice(&plaintext);
        msg.payload.truncate(plaintext.len());

        // Use rustls's built-in TLS 1.3 unpadding which handles:
        // - Removing zero padding
        // - Extracting content type from last byte
        // - Validating message length
        msg.into_tls13_unpadded_message()
    }
}

/// Construct TLS 1.3 nonce from IV and sequence number
///
/// Per RFC 8446 Section 5.3:
/// ```text
/// nonce = per_record_nonce XOR sequence_number
/// ```
///
/// The sequence number is left-padded with zeros to 12 bytes before XOR.
fn construct_tls13_nonce(iv: &[u8], seq: u64) -> Vec<u8> {
    assert_eq!(iv.len(), 12, "IV must be 12 bytes for TLS 1.3");

    let mut nonce = iv.to_vec();

    // XOR the last 8 bytes with sequence number (big-endian)
    let seq_bytes = seq.to_be_bytes();
    for i in 0..8 {
        nonce[4 + i] ^= seq_bytes[i];
    }

    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_construction() {
        // Test TLS 1.3 nonce construction (IV XOR sequence)
        let iv = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c];
        let seq = 0x0000_0000_0000_0001u64;

        let nonce = construct_tls13_nonce(&iv, seq);

        assert_eq!(nonce.len(), 12);
        // First 4 bytes unchanged
        assert_eq!(&nonce[0..4], &[0x01, 0x02, 0x03, 0x04]);
        // Last 8 bytes XORed with sequence
        assert_eq!(
            &nonce[4..],
            &[0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0d]
        ); // 0x0c XOR 0x01 = 0x0d
    }

    #[test]
    fn test_nonce_construction_zero_seq() {
        let iv = vec![0xff; 12];
        let seq = 0u64;

        let nonce = construct_tls13_nonce(&iv, seq);

        // With sequence 0, nonce should equal IV
        assert_eq!(nonce, iv);
    }

    #[test]
    fn test_nonce_construction_max_seq() {
        let iv = vec![0x00; 12];
        let seq = u64::MAX;

        let nonce = construct_tls13_nonce(&iv, seq);

        // First 4 bytes should be 0, last 8 should be all 0xff
        assert_eq!(&nonce[0..4], &[0x00, 0x00, 0x00, 0x00]);
        assert_eq!(&nonce[4..], &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    #[should_panic(expected = "IV must be 12 bytes")]
    fn test_nonce_construction_invalid_iv_length() {
        let iv = vec![0x01; 16]; // Wrong length!
        construct_tls13_nonce(&iv, 0);
    }

    #[test]
    fn test_key_len() {
        let algo = BeardogChaCha20Poly1305;
        assert_eq!(algo.key_len(), 32); // ChaCha20 uses 256-bit keys
    }

    #[test]
    fn test_extract_keys() {
        let algo = BeardogChaCha20Poly1305;
        // AeadKey and Iv have private constructors, so we can't test extract_keys directly
        // This is an implementation detail test that would need access to rustls internals
        // We'll skip it and rely on integration tests instead
        
        // Just verify the key_len is correct
        assert_eq!(algo.key_len(), 32);
    }

    #[test]
    fn test_encrypted_payload_len() {
        // This test doesn't require crypto provider initialization
        // We're just testing the length calculation

        let dummy_key = vec![0u8; 32];
        let dummy_iv = vec![0u8; 12];

        // Create a mock crypto provider for this test
        use crate::crypto::provider::MockCryptoProvider;
        let crypto = Arc::new(MockCryptoProvider::new()) as Arc<dyn CryptoProvider>;

        let encrypter = BeardogChaCha20Encrypter {
            crypto,
            key: dummy_key,
            iv: dummy_iv,
        };

        // Test various payload lengths
        assert_eq!(encrypter.encrypted_payload_len(0), 16); // Just tag
        assert_eq!(encrypter.encrypted_payload_len(100), 116); // 100 + 16
        assert_eq!(encrypter.encrypted_payload_len(1024), 1040); // 1024 + 16
    }
}
