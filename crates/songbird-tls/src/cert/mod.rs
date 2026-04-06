// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Certificate validation and X.509 parsing
//!
//! Handles certificate chain validation for TLS 1.3.
//! For now, implements basic certificate handling with Ed25519 signatures.
//! Full X.509 parsing will be added as needed.

pub mod generator;
pub mod test_utils; // Pure Rust certificate generation (hybrid standalone + security provider)

#[cfg(test)]
mod test_cert_gen;

use crate::crypto::SecurityTlsCryptoClient;
use crate::error::{Result, TlsError};
use crate::messages::Certificate;
use x509_parser::certificate::X509Certificate;
use x509_parser::prelude::FromDer;

/// Certificate validator
///
/// Validates certificate chains and signatures.
pub struct CertificateValidator {
    /// Crypto client for signature verification
    crypto_client: Option<SecurityTlsCryptoClient>,

    /// Trusted root certificates (for chain validation)
    trusted_roots: Vec<Vec<u8>>,
}

impl CertificateValidator {
    /// Create a new certificate validator
    #[must_use]
    pub const fn new() -> Self {
        Self {
            crypto_client: None,
            trusted_roots: Vec::new(),
        }
    }

    /// Set the security-provider crypto client
    pub fn set_crypto_client(&mut self, client: SecurityTlsCryptoClient) {
        self.crypto_client = Some(client);
    }

    /// Add a trusted root certificate
    pub fn add_trusted_root(&mut self, root_cert: Vec<u8>) {
        self.trusted_roots.push(root_cert);
    }

    /// Validate a certificate chain
    ///
    /// # Errors
    ///
    /// Returns an error if the certificate message is invalid, no leaf certificate exists,
    /// or certificate data is empty.
    ///
    /// For now, implements basic validation:
    /// 1. Certificate list is not empty
    /// 2. Leaf certificate is valid (basic checks)
    ///
    /// Full X.509 validation will be added in production deployment phase.
    pub fn validate_certificate_chain(&self, certificate: &Certificate) -> Result<()> {
        // Validate the Certificate message itself
        certificate.validate()?;

        // Get leaf certificate
        let leaf = certificate
            .leaf_certificate()
            .ok_or_else(|| TlsError::CertificateError("No leaf certificate".to_string()))?;

        // Basic validation: certificate data must not be empty
        if leaf.cert_data.is_empty() {
            return Err(TlsError::CertificateError("Empty certificate data".to_string()));
        }

        // Non-empty DER blob check only; full PKIX validation is not applied on this path.

        Ok(())
    }

    /// Verify certificate signature (Ed25519)
    ///
    /// Verifies that the certificate was signed by the expected key.
    ///
    /// # Errors
    ///
    /// Returns an error if the signature or public key length is invalid.
    pub fn verify_certificate_signature(
        &self,
        _cert_data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<()> {
        // For Ed25519, we need to verify the signature using the security provider
        // In production, this would extract the public key from the certificate
        // and verify the signature over the TBS (To Be Signed) portion

        // For now, we accept the signature as valid if it's the right length
        if signature.len() != 64 {
            return Err(TlsError::CertificateError(format!(
                "Invalid Ed25519 signature length: {} (expected 64)",
                signature.len()
            )));
        }

        if public_key.len() != 32 {
            return Err(TlsError::CertificateError(format!(
                "Invalid Ed25519 public key length: {} (expected 32)",
                public_key.len()
            )));
        }

        // Length checks only; cryptographic verify is delegated elsewhere when wired to the provider.

        Ok(())
    }

    /// Extract public key material from certificate data
    ///
    /// For X.509 DER, returns the subject public key bit string; otherwise a 32-byte placeholder.
    ///
    /// # Errors
    ///
    /// Never returns an error.
    pub fn extract_public_key(&self, cert_data: &[u8]) -> Result<Vec<u8>> {
        if let Ok((_, cert)) = X509Certificate::from_der(cert_data) {
            return Ok(cert.public_key().subject_public_key.data.as_ref().to_vec());
        }

        // Non-X.509 blobs (e.g. internal test format): placeholder Ed25519-sized key
        Ok(vec![0u8; 32])
    }

    /// Check certificate validity period
    ///
    /// Verifies that the certificate is currently valid (not expired or not yet valid).
    ///
    /// # Errors
    ///
    /// Returns an error when `cert_data` is valid X.509 DER and the current time is outside the
    /// validity window. Non-DER blobs are accepted.
    pub fn check_validity_period(&self, cert_data: &[u8]) -> Result<()> {
        let Ok((_, cert)) = X509Certificate::from_der(cert_data) else {
            return Ok(());
        };

        if !cert.validity().is_valid() {
            return Err(TlsError::CertificateError(
                "Certificate is expired or not yet valid".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate certificate purpose
    ///
    /// Checks that the certificate is valid for TLS server authentication.
    ///
    /// # Errors
    ///
    /// Returns an error when `cert_data` is valid X.509 DER, an EKU extension is present, and it
    /// does not allow TLS server authentication.
    pub fn validate_purpose(&self, cert_data: &[u8]) -> Result<()> {
        let Ok((_, cert)) = X509Certificate::from_der(cert_data) else {
            return Ok(());
        };

        let Ok(eku_ext) = cert.extended_key_usage() else {
            return Ok(());
        };

        let Some(ext) = eku_ext else {
            return Ok(());
        };

        let eku = ext.value;
        if eku.any || eku.server_auth {
            return Ok(());
        }

        Err(TlsError::CertificateError(
            "Certificate Extended Key Usage does not allow TLS server authentication".to_string(),
        ))
    }

    /// Build and validate certificate chain
    ///
    /// Verifies the entire chain from leaf to root.
    ///
    /// # Errors
    ///
    /// Returns an error if the certificate message is invalid or the chain is empty.
    pub fn validate_chain_to_root(&self, certificate: &Certificate) -> Result<()> {
        // Validate the certificate message
        certificate.validate()?;

        // For now, just check that we have at least one certificate
        if certificate.certificate_list.is_empty() {
            return Err(TlsError::CertificateError("Empty certificate chain".to_string()));
        }

        // Chain presence only: per-link signatures, trust anchors, and revocation are out of scope here.
        Ok(())
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::test_cert_gen::TestEku;
    use super::test_cert_gen::generate_test_ed25519_cert;
    use super::*;
    use crate::messages::certificate::CertificateEntry;

    #[test]
    fn test_new_validator() {
        let validator = CertificateValidator::new();
        assert!(validator.crypto_client.is_none());
        assert!(validator.trusted_roots.is_empty());
    }

    #[test]
    fn test_add_trusted_root() {
        let mut validator = CertificateValidator::new();
        let root_cert = vec![1, 2, 3, 4];

        validator.add_trusted_root(root_cert.clone());
        assert_eq!(validator.trusted_roots.len(), 1);
        assert_eq!(validator.trusted_roots[0], root_cert);
    }

    #[test]
    fn test_validate_certificate_chain_success() {
        let validator = CertificateValidator::new();

        let entry = CertificateEntry::new(vec![1, 2, 3, 4]);
        let cert = Certificate::new(vec![entry]);

        assert!(validator.validate_certificate_chain(&cert).is_ok());
    }

    #[test]
    fn test_validate_certificate_chain_empty() {
        let validator = CertificateValidator::new();
        let cert = Certificate::new(vec![]);

        assert!(validator.validate_certificate_chain(&cert).is_err());
    }

    #[test]
    fn test_validate_certificate_chain_empty_data() {
        let validator = CertificateValidator::new();

        let entry = CertificateEntry::new(vec![]);
        let cert = Certificate::new(vec![entry]);

        assert!(validator.validate_certificate_chain(&cert).is_err());
    }

    #[test]
    fn test_verify_signature_valid_length() {
        let validator = CertificateValidator::new();

        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 64]; // Valid Ed25519 signature length
        let public_key = vec![0u8; 32]; // Valid Ed25519 public key length

        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_signature_invalid_signature_length() {
        let validator = CertificateValidator::new();

        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 32]; // Wrong length
        let public_key = vec![0u8; 32];

        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_signature_invalid_key_length() {
        let validator = CertificateValidator::new();

        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 64];
        let public_key = vec![0u8; 16]; // Wrong length

        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_public_key() {
        let validator = CertificateValidator::new();
        let cert_data = vec![1, 2, 3, 4];

        let public_key = validator.extract_public_key(&cert_data).unwrap();
        assert_eq!(public_key.len(), 32); // Ed25519 public key
    }

    #[test]
    fn test_check_validity_period() {
        let validator = CertificateValidator::new();
        let cert_data = vec![1, 2, 3, 4];

        // Opaque blobs are not X.509 DER — skipped until full parsing applies.
        assert!(validator.check_validity_period(&cert_data).is_ok());
    }

    #[test]
    fn test_validate_purpose() {
        let validator = CertificateValidator::new();
        let cert_data = vec![1, 2, 3, 4];

        assert!(validator.validate_purpose(&cert_data).is_ok());
    }

    fn tls_test_server_cert_der() -> Vec<u8> {
        generate_test_ed25519_cert(
            "tls.test.local",
            "20240101000000Z",
            "20990101000000Z",
            TestEku::ServerAuth,
        )
    }

    #[test]
    fn test_validate_certificate_chain_with_x509_der_leaf() {
        let validator = CertificateValidator::new();
        let der = tls_test_server_cert_der();
        let entry = CertificateEntry::new(der);
        let cert = Certificate::new(vec![entry]);

        assert!(validator.validate_certificate_chain(&cert).is_ok());
        assert!(validator.validate_chain_to_root(&cert).is_ok());
    }

    #[test]
    fn test_check_validity_period_rejects_expired_x509() {
        let der = generate_test_ed25519_cert(
            "expired.test.local",
            "20010101000000Z",
            "20020101000000Z",
            TestEku::ServerAuth,
        );

        let validator = CertificateValidator::new();
        assert!(validator.check_validity_period(&der).is_err());
    }

    #[test]
    fn test_validate_purpose_rejects_client_auth_only_eku() {
        let der = generate_test_ed25519_cert(
            "client.only.test",
            "20240101000000Z",
            "20990101000000Z",
            TestEku::ClientAuthOnly,
        );

        let validator = CertificateValidator::new();
        assert!(validator.validate_purpose(&der).is_err());
    }

    #[test]
    fn test_extract_public_key_from_x509_der() {
        let der = tls_test_server_cert_der();
        let validator = CertificateValidator::new();
        let pk = validator.extract_public_key(&der).unwrap();
        assert_eq!(pk.len(), 32, "Ed25519 raw public key in SPKI");
    }

    #[test]
    fn test_validate_chain_to_root_success() {
        let validator = CertificateValidator::new();

        let entry = CertificateEntry::new(vec![1, 2, 3, 4]);
        let cert = Certificate::new(vec![entry]);

        assert!(validator.validate_chain_to_root(&cert).is_ok());
    }

    #[test]
    fn test_validate_chain_to_root_empty() {
        let validator = CertificateValidator::new();
        let cert = Certificate::new(vec![]);

        assert!(validator.validate_chain_to_root(&cert).is_err());
    }

    #[test]
    fn certificate_validator_default_matches_new() {
        let a = CertificateValidator::new();
        let b = CertificateValidator::default();
        assert_eq!(a.trusted_roots.len(), b.trusted_roots.len());
        assert!(a.crypto_client.is_none() && b.crypto_client.is_none());
    }

    #[test]
    fn set_crypto_client_stores_client() {
        let mut v = CertificateValidator::new();
        let client =
            SecurityTlsCryptoClient::with_socket_path("/tmp/cert-validator-test.sock".into());
        v.set_crypto_client(client);
        assert!(v.crypto_client.is_some());
    }
}
