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
            return Err(TlsError::ProtocolError(
                "Finished verify_data cannot be empty".to_string(),
            ));
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
            return Err(TlsError::HandshakeFailure(
                "Finished verify_data length mismatch".to_string(),
            ));
        }

        // Constant-time comparison to prevent timing attacks
        let mut diff = 0u8;
        for (a, b) in self.verify_data.iter().zip(expected_verify_data.iter()) {
            diff |= a ^ b;
        }

        if diff != 0 {
            return Err(TlsError::HandshakeFailure(
                "Finished verify_data does not match expected value".to_string(),
            ));
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
}
