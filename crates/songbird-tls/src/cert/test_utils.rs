// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Certificate utilities for testing and development
//!
//! These utilities help generate self-signed Ed25519 certificates for testing
//! and development. In production, use proper certificate management.

use crate::error::{Result, TlsError};
use crate::messages::{Certificate, certificate::CertificateEntry};

/// Generate a self-signed Ed25519 certificate for testing
///
/// # Errors
///
/// Returns an error if the domain name is too long (> 255 bytes) or certificate creation fails.
///
/// This is a simplified test certificate. In production, you should:
/// 1. Use proper X.509 certificate generation
/// 2. Get certificates from a CA or use Let's Encrypt
/// 3. Use the security provider for key generation and signing
///
/// # Example
///
/// ```rust,ignore
/// use songbird_tls::cert::generate_test_certificate;
///
/// let cert = generate_test_certificate("test.example.com")?;
/// ```
pub fn generate_test_certificate(domain: &str) -> Result<Certificate> {
    // For testing, create a minimal certificate structure
    // In production, this would be a proper X.509 certificate

    let cert_data = create_test_cert_data(domain)?;

    let entry = CertificateEntry {
        cert_data,
        extensions: vec![], // No extensions for test certs
    };

    Ok(Certificate::new(vec![entry]))
}

/// Create test certificate data (simplified DER encoding)
///
/// This is NOT a real X.509 certificate - it's a placeholder for testing.
/// Real certificates should be generated with proper tooling.
fn create_test_cert_data(domain: &str) -> Result<Vec<u8>> {
    // Simplified test certificate structure
    // Format: [version, domain_len, domain, validity, key_placeholder]

    let mut cert_data = Vec::new();

    // Version (1 byte) - v3
    cert_data.push(0x03);

    // Domain name length (1 byte)
    if domain.len() > 255 {
        return Err(TlsError::CertificateError(
            "Domain name too long for test certificate".to_string(),
        ));
    }
    cert_data.push(u8::try_from(domain.len()).map_err(|_| {
        TlsError::CertificateError("Domain name too long for test certificate".to_string())
    })?);

    // Domain name
    cert_data.extend_from_slice(domain.as_bytes());

    // Validity period (4 bytes) - 1 year in seconds
    let validity: u32 = 365 * 24 * 60 * 60;
    cert_data.extend_from_slice(&validity.to_be_bytes());

    // Ed25519 public key placeholder (32 bytes)
    // In production, this would be a real public key from the security provider
    cert_data.extend_from_slice(&[0x42u8; 32]);

    // Signature placeholder (64 bytes)
    // In production, this would be a real Ed25519 signature from the security provider
    cert_data.extend_from_slice(&[0x73u8; 64]);

    Ok(cert_data)
}

/// Validate a test certificate
///
/// # Errors
///
/// Returns an error if the certificate chain is empty, leaf certificate is empty,
/// or certificate data is too small.
///
/// This performs basic validation on test certificates.
/// In production, use proper X.509 validation.
pub fn validate_test_certificate(cert: &Certificate) -> Result<()> {
    // Check that we have at least one certificate
    if cert.certificate_list.is_empty() {
        return Err(TlsError::CertificateError("Certificate chain is empty".to_string()));
    }

    // Check leaf certificate
    let leaf = &cert.certificate_list[0];
    if leaf.cert_data.is_empty() {
        return Err(TlsError::CertificateError("Leaf certificate is empty".to_string()));
    }

    // Basic size validation
    if leaf.cert_data.len() < 100 {
        return Err(TlsError::CertificateError(
            "Certificate data too small (likely invalid)".to_string(),
        ));
    }

    Ok(())
}

/// Extract domain name from test certificate
///
/// # Errors
///
/// Returns an error if the certificate chain is empty, certificate data is too small,
/// or the domain contains invalid UTF-8.
///
/// This extracts the domain from our simplified test certificate format.
/// In production, use proper X.509 parsing.
pub fn extract_domain(cert: &Certificate) -> Result<String> {
    if cert.certificate_list.is_empty() {
        return Err(TlsError::CertificateError("Certificate chain is empty".to_string()));
    }

    let cert_data = &cert.certificate_list[0].cert_data;

    // Our test format: [version, domain_len, domain, ...]
    if cert_data.len() < 3 {
        return Err(TlsError::CertificateError("Certificate data too small".to_string()));
    }

    if cert_data[0] != 0x03 {
        return Err(TlsError::CertificateError(
            "Invalid test certificate format (wrong version)".to_string(),
        ));
    }
    let domain_len = cert_data[1] as usize;

    if cert_data.len() < 2 + domain_len {
        return Err(TlsError::CertificateError("Certificate data truncated".to_string()));
    }

    let domain_bytes = &cert_data[2..2 + domain_len];
    String::from_utf8(domain_bytes.to_vec())
        .map_err(|_| TlsError::CertificateError("Invalid UTF-8 in domain name".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_certificate() {
        let cert = generate_test_certificate("example.com").unwrap();

        assert_eq!(cert.certificate_list.len(), 1);
        assert!(!cert.certificate_list[0].cert_data.is_empty());
    }

    #[test]
    fn test_validate_test_certificate() {
        let cert = generate_test_certificate("test.local").unwrap();

        // Should pass validation
        assert!(validate_test_certificate(&cert).is_ok());
    }

    #[test]
    fn test_validate_empty_certificate() {
        let cert = Certificate::new(vec![]);

        // Should fail - empty chain
        assert!(validate_test_certificate(&cert).is_err());
    }

    #[test]
    fn test_extract_domain() {
        let cert = generate_test_certificate("example.com").unwrap();

        let domain = extract_domain(&cert).unwrap();
        assert_eq!(domain, "example.com");
    }

    #[test]
    fn test_generate_long_domain() {
        let domain = "a".repeat(255);
        let cert = generate_test_certificate(&domain).unwrap();

        let extracted = extract_domain(&cert).unwrap();
        assert_eq!(extracted, domain);
    }

    #[test]
    fn test_generate_too_long_domain() {
        // Test with domain that's too long
        let domain = "a".repeat(256);
        let result = generate_test_certificate(&domain);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_certificate_with_small_data() {
        let entry = CertificateEntry {
            cert_data: vec![1, 2, 3], // Too small
            extensions: vec![],
        };
        let cert = Certificate::new(vec![entry]);

        assert!(validate_test_certificate(&cert).is_err());
    }
}
