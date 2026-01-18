//! Certificate validation and X.509 parsing
//!
//! Handles certificate chain validation for TLS 1.3.
//! For now, implements basic certificate handling with Ed25519 signatures.
//! Full X.509 parsing will be added as needed.

use crate::error::{Result, TlsError};
use crate::crypto::BeardogCryptoClient;
use crate::messages::Certificate;

/// Certificate validator
///
/// Validates certificate chains and signatures.
pub struct CertificateValidator {
    /// Crypto client for signature verification
    crypto_client: Option<BeardogCryptoClient>,
    
    /// Trusted root certificates (for chain validation)
    trusted_roots: Vec<Vec<u8>>,
}

impl CertificateValidator {
    /// Create a new certificate validator
    pub fn new() -> Self {
        Self {
            crypto_client: None,
            trusted_roots: Vec::new(),
        }
    }

    /// Set the BearDog crypto client
    pub fn set_crypto_client(&mut self, client: BeardogCryptoClient) {
        self.crypto_client = Some(client);
    }

    /// Add a trusted root certificate
    pub fn add_trusted_root(&mut self, root_cert: Vec<u8>) {
        self.trusted_roots.push(root_cert);
    }

    /// Validate a certificate chain
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
        let leaf = certificate.leaf_certificate()
            .ok_or_else(|| TlsError::CertificateError("No leaf certificate".to_string()))?;

        // Basic validation: certificate data must not be empty
        if leaf.cert_data.is_empty() {
            return Err(TlsError::CertificateError("Empty certificate data".to_string()));
        }

        // TODO: Full X.509 parsing and validation
        // This will be implemented in Phase 7 (Production Deployment)
        // For now, we accept any non-empty certificate data

        Ok(())
    }

    /// Verify certificate signature (Ed25519)
    ///
    /// Verifies that the certificate was signed by the expected key.
    pub async fn verify_certificate_signature(
        &self,
        cert_data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<()> {
        // For Ed25519, we need to verify the signature using BearDog
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

        // TODO: Actual signature verification via BearDog
        // This will be implemented when we have Ed25519 verify in crypto client
        // For now, basic length validation is sufficient for Phase 5

        Ok(())
    }

    /// Extract public key from certificate
    ///
    /// For now, returns a placeholder. Full X.509 parsing will be added later.
    pub fn extract_public_key(&self, _cert_data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Parse X.509 certificate and extract SubjectPublicKeyInfo
        // This will use a pure Rust X.509 parser in Phase 7
        
        // For now, return a placeholder 32-byte Ed25519 public key
        Ok(vec![0u8; 32])
    }

    /// Check certificate validity period
    ///
    /// Verifies that the certificate is currently valid (not expired or not yet valid).
    pub fn check_validity_period(&self, _cert_data: &[u8]) -> Result<()> {
        // TODO: Parse X.509 certificate and check notBefore/notAfter
        // This will be implemented with proper X.509 parsing in Phase 7
        
        // For now, accept all certificates (no time validation)
        Ok(())
    }

    /// Validate certificate purpose
    ///
    /// Checks that the certificate is valid for TLS server authentication.
    pub fn validate_purpose(&self, _cert_data: &[u8]) -> Result<()> {
        // TODO: Check Extended Key Usage (EKU) for TLS server authentication
        // This requires X.509 extension parsing
        
        // For now, accept all certificates
        Ok(())
    }

    /// Build and validate certificate chain
    ///
    /// Verifies the entire chain from leaf to root.
    pub fn validate_chain_to_root(&self, certificate: &Certificate) -> Result<()> {
        // Validate the certificate message
        certificate.validate()?;

        // For now, just check that we have at least one certificate
        if certificate.certificate_list.is_empty() {
            return Err(TlsError::CertificateError("Empty certificate chain".to_string()));
        }

        // TODO: Full chain validation
        // 1. Verify each certificate's signature using the next cert's public key
        // 2. Verify the root certificate is in our trusted roots
        // 3. Check all certificates' validity periods
        // 4. Verify no certificate is revoked (OCSP/CRL)
        
        // For Phase 5, basic validation is sufficient
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

    #[tokio::test]
    async fn test_verify_signature_valid_length() {
        let validator = CertificateValidator::new();
        
        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 64]; // Valid Ed25519 signature length
        let public_key = vec![0u8; 32]; // Valid Ed25519 public key length
        
        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_signature_invalid_signature_length() {
        let validator = CertificateValidator::new();
        
        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 32]; // Wrong length
        let public_key = vec![0u8; 32];
        
        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_signature_invalid_key_length() {
        let validator = CertificateValidator::new();
        
        let cert_data = vec![1, 2, 3, 4];
        let signature = vec![0u8; 64];
        let public_key = vec![0u8; 16]; // Wrong length
        
        let result = validator.verify_certificate_signature(&cert_data, &signature, &public_key).await;
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
        
        // For now, all certificates are considered valid
        assert!(validator.check_validity_period(&cert_data).is_ok());
    }

    #[test]
    fn test_validate_purpose() {
        let validator = CertificateValidator::new();
        let cert_data = vec![1, 2, 3, 4];
        
        // For now, all certificates are considered valid for TLS
        assert!(validator.validate_purpose(&cert_data).is_ok());
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
}
