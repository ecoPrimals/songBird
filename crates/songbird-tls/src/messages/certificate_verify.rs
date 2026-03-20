// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `CertificateVerify` message (RFC 8446 Section 4.4.3)
//!
//! The `CertificateVerify` message provides proof that the sender has the private key
//! corresponding to the certificate in the Certificate message.

use crate::error::{Result, TlsError};

/// `CertificateVerify` message
///
/// ```text
/// struct {
///     SignatureScheme algorithm;
///     opaque signature<0..2^16-1>;
/// } CertificateVerify;
/// ```
#[derive(Debug, Clone)]
pub struct CertificateVerify {
    /// Signature algorithm used (e.g., Ed25519)
    pub algorithm: u16,

    /// Signature over the handshake transcript
    pub signature: Vec<u8>,
}

impl CertificateVerify {
    /// Create a new `CertificateVerify` message
    #[must_use]
    pub const fn new(algorithm: u16, signature: Vec<u8>) -> Self {
        Self {
            algorithm,
            signature,
        }
    }

    /// Check if the signature algorithm is supported
    #[must_use]
    pub const fn is_algorithm_supported(&self) -> bool {
        // For now, we only support Ed25519 (0x0807)
        // Additional algorithms can be added later
        self.algorithm == SIGNATURE_ED25519
    }

    /// Validate `CertificateVerify` message
    ///
    /// # Errors
    ///
    /// Returns an error if the signature is empty, algorithm is unsupported, or
    /// Ed25519 signature length is invalid.
    pub fn validate(&self) -> Result<()> {
        // Signature must not be empty
        if self.signature.is_empty() {
            return Err(TlsError::CertificateError("Signature cannot be empty".to_string()));
        }

        // Check if algorithm is supported
        if !self.is_algorithm_supported() {
            return Err(TlsError::Unsupported(format!(
                "Signature algorithm 0x{:04x} is not supported",
                self.algorithm
            )));
        }

        // Ed25519 signatures must be exactly 64 bytes
        if self.algorithm == SIGNATURE_ED25519 && self.signature.len() != 64 {
            return Err(TlsError::CertificateError(format!(
                "Ed25519 signature must be 64 bytes, got {}",
                self.signature.len()
            )));
        }

        Ok(())
    }

    /// Get the expected signature length for this algorithm
    #[must_use]
    pub const fn expected_signature_length(&self) -> usize {
        match self.algorithm {
            SIGNATURE_ED25519 => 64,
            _ => 0, // Unknown algorithm
        }
    }
}

// Signature algorithm constants
pub const SIGNATURE_ED25519: u16 = 0x0807;
pub const SIGNATURE_ECDSA_SECP256R1_SHA256: u16 = 0x0403;
pub const SIGNATURE_RSA_PSS_RSAE_SHA256: u16 = 0x0804;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_verify_new() {
        let signature = vec![42u8; 64]; // 64-byte Ed25519 signature
        let verify = CertificateVerify::new(SIGNATURE_ED25519, signature.clone());

        assert_eq!(verify.algorithm, SIGNATURE_ED25519);
        assert_eq!(verify.signature, signature);
    }

    #[test]
    fn test_is_algorithm_supported() {
        let verify = CertificateVerify::new(SIGNATURE_ED25519, vec![42u8; 64]);
        assert!(verify.is_algorithm_supported());

        let verify = CertificateVerify::new(0x9999, vec![42u8; 64]);
        assert!(!verify.is_algorithm_supported());
    }

    #[test]
    fn test_certificate_verify_validation_success() {
        let signature = vec![42u8; 64];
        let verify = CertificateVerify::new(SIGNATURE_ED25519, signature);
        assert!(verify.validate().is_ok());
    }

    #[test]
    fn test_certificate_verify_validation_empty_signature() {
        let verify = CertificateVerify::new(SIGNATURE_ED25519, vec![]);
        assert!(verify.validate().is_err());
    }

    #[test]
    fn test_certificate_verify_validation_unsupported_algorithm() {
        let signature = vec![42u8; 64];
        let verify = CertificateVerify::new(0x9999, signature);
        assert!(verify.validate().is_err());
    }

    #[test]
    fn test_certificate_verify_validation_wrong_signature_length() {
        let signature = vec![42u8; 32]; // Wrong length for Ed25519
        let verify = CertificateVerify::new(SIGNATURE_ED25519, signature);
        assert!(verify.validate().is_err());
    }

    #[test]
    fn test_expected_signature_length() {
        let verify = CertificateVerify::new(SIGNATURE_ED25519, vec![42u8; 64]);
        assert_eq!(verify.expected_signature_length(), 64);

        let verify = CertificateVerify::new(0x9999, vec![]);
        assert_eq!(verify.expected_signature_length(), 0);
    }

    #[test]
    fn test_constants() {
        assert_eq!(SIGNATURE_ED25519, 0x0807);
        assert_eq!(SIGNATURE_ECDSA_SECP256R1_SHA256, 0x0403);
    }
}
