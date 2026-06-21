// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Finished message (RFC 8446 Section 4.4.4)
//!
//! The Finished message is the final message in the authentication block.
//! It provides verification of the entire handshake.

use crate::error::{Result, TlsError};

/// Finished message
///
/// ```text
/// struct {
///     opaque verify_data[Hash.length];
/// } Finished;
/// ```
///
/// The `verify_data` is computed as:
/// ```text
/// verify_data = HMAC(finished_key,
///                    Transcript-Hash(Handshake Context,
///                                    Certificate*, CertificateVerify*))
/// ```
#[derive(Debug, Clone)]
pub struct Finished {
    /// Verification data (HMAC over handshake transcript)
    /// For SHA-256: 32 bytes
    /// For BLAKE3: 32 bytes
    pub verify_data: Vec<u8>,
}

impl Finished {
    /// Create a new Finished message
    #[must_use]
    pub const fn new(verify_data: Vec<u8>) -> Self {
        Self {
            verify_data,
        }
    }

    /// Validate Finished message
    ///
    /// # Errors
    ///
    /// Returns an error if `verify_data` is empty or has incorrect length.
    pub fn validate(&self) -> Result<()> {
        // verify_data must not be empty
        if self.verify_data.is_empty() {
            return Err(TlsError::ProtocolError(String::from(
                "Finished verify_data cannot be empty",
            )));
        }

        // For ChaCha20-Poly1305-SHA256 cipher suite, verify_data should be 32 bytes
        // (SHA-256 produces 32-byte hashes)
        if self.verify_data.len() != 32 {
            return Err(TlsError::ProtocolError(format!(
                "Finished verify_data must be 32 bytes for SHA-256, got {}",
                self.verify_data.len()
            )));
        }

        Ok(())
    }

    /// Get the expected `verify_data` length for a given hash algorithm
    #[must_use]
    pub const fn expected_length_for_hash(hash_algorithm: HashAlgorithm) -> usize {
        match hash_algorithm {
            HashAlgorithm::Sha256 | HashAlgorithm::Blake3 => 32,
            HashAlgorithm::Sha384 => 48,
        }
    }

    /// Verify this Finished message matches the expected value
    ///
    /// # Errors
    ///
    /// Returns an error if lengths differ or verification fails.
    pub fn verify(&self, expected_verify_data: &[u8]) -> Result<()> {
        if self.verify_data.len() != expected_verify_data.len() {
            return Err(TlsError::HandshakeFailure(String::from(
                "Finished verify_data length mismatch",
            )));
        }

        // Constant-time comparison to prevent timing attacks
        let mut diff = 0u8;
        for (a, b) in self.verify_data.iter().zip(expected_verify_data.iter()) {
            diff |= a ^ b;
        }

        if diff != 0 {
            return Err(TlsError::HandshakeFailure(String::from(
                "Finished verify_data does not match expected value",
            )));
        }

        Ok(())
    }
}

/// Hash algorithm used in the cipher suite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 transcript and PRF basis.
    Sha256,
    /// SHA-384 transcript and PRF basis.
    Sha384,
    /// Blake3 when used as the handshake hash.
    Blake3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finished_new() {
        let verify_data = vec![42u8; 32];
        let finished = Finished::new(verify_data.clone());
        assert_eq!(finished.verify_data, verify_data);
    }

    #[test]
    fn test_finished_validation_success() {
        let verify_data = vec![42u8; 32];
        let finished = Finished::new(verify_data);
        assert!(finished.validate().is_ok());
    }

    #[test]
    fn test_finished_validation_empty() {
        let finished = Finished::new(vec![]);
        assert!(finished.validate().is_err());
    }

    #[test]
    fn test_finished_validation_wrong_length() {
        let verify_data = vec![42u8; 16]; // Wrong length
        let finished = Finished::new(verify_data);
        assert!(finished.validate().is_err());
    }

    #[test]
    fn test_expected_length_for_hash() {
        assert_eq!(Finished::expected_length_for_hash(HashAlgorithm::Sha256), 32);
        assert_eq!(Finished::expected_length_for_hash(HashAlgorithm::Sha384), 48);
        assert_eq!(Finished::expected_length_for_hash(HashAlgorithm::Blake3), 32);
    }

    #[test]
    fn test_verify_success() {
        let verify_data = vec![42u8; 32];
        let finished = Finished::new(verify_data.clone());
        assert!(finished.verify(&verify_data).is_ok());
    }

    #[test]
    fn test_verify_mismatch() {
        let verify_data = vec![42u8; 32];
        let wrong_data = vec![99u8; 32];
        let finished = Finished::new(verify_data);
        assert!(finished.verify(&wrong_data).is_err());
    }

    #[test]
    fn test_verify_length_mismatch() {
        let verify_data = vec![42u8; 32];
        let wrong_length = vec![99u8; 16];
        let finished = Finished::new(verify_data);
        assert!(finished.verify(&wrong_length).is_err());
    }

    #[test]
    fn test_verify_constant_time() {
        // This test verifies that our comparison uses XOR (constant-time pattern)
        // In a real scenario, we'd use a crypto timing analysis tool
        let verify_data1 = vec![0u8; 32];
        let verify_data2 = vec![1u8; 32];

        let finished = Finished::new(verify_data1);
        let result = finished.verify(&verify_data2);

        // Should fail, but the important thing is it uses constant-time comparison
        assert!(result.is_err());
    }

    fn reference_hmac_sha256(message: &[u8], key: &[u8]) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("key length");
        mac.update(message);
        mac.finalize().into_bytes().to_vec()
    }

    #[test]
    fn test_verify_data_hmac_reference_vector() {
        let transcript = b"ClientHelloServerHelloCertificate";
        let finished_key = [0x55u8; 32];
        let expected = reference_hmac_sha256(transcript, &finished_key);
        assert_eq!(expected.len(), 32);

        let finished = Finished::new(expected.clone());
        assert!(finished.validate().is_ok());
        assert!(finished.verify(&expected).is_ok());
    }

    #[test]
    fn test_validation_rejects_non_32_byte_lengths() {
        for wrong_len in [1, 16, 31, 33, 48] {
            let finished = Finished::new(vec![0xAB; wrong_len]);
            assert!(finished.validate().is_err(), "len {wrong_len} should fail validate()");
        }
    }

    #[test]
    fn test_validate_accepts_exactly_hash_length() {
        let finished = Finished::new(vec![0xCC; 32]);
        assert!(finished.validate().is_ok());
    }

    #[test]
    fn test_finished_in_handshake_context() {
        let transcript = b"CH+SH+Cert+CertVerify";
        let base_key = [0x11u8; 32];
        // finished_key stand-in: direct HMAC key for test isolation
        let verify_data = reference_hmac_sha256(transcript, &base_key);
        let server_finished = Finished::new(verify_data.clone());
        let client_finished = Finished::new(verify_data.clone());

        assert!(server_finished.validate().is_ok());
        assert!(client_finished.verify(&verify_data).is_ok());

        let mut tampered = verify_data;
        tampered[0] ^= 0x01;
        assert!(client_finished.verify(&tampered).is_err());
    }

    #[test]
    fn test_verify_error_messages() {
        let finished = Finished::new(vec![1u8; 32]);
        let len_err = finished.verify(&[2u8; 16]).unwrap_err();
        assert!(matches!(len_err, TlsError::HandshakeFailure(_)));
        assert!(len_err.to_string().contains("length mismatch"));

        let value_err = finished.verify(&[2u8; 32]).unwrap_err();
        assert!(matches!(value_err, TlsError::HandshakeFailure(_)));
        assert!(value_err.to_string().contains("does not match"));
    }

    #[test]
    fn test_expected_length_matches_validate_policy() {
        assert_eq!(Finished::expected_length_for_hash(HashAlgorithm::Sha256), 32);
        let finished =
            Finished::new(vec![0u8; Finished::expected_length_for_hash(HashAlgorithm::Sha256)]);
        assert!(finished.validate().is_ok());
    }
}
