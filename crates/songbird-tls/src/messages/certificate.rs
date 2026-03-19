//! Certificate message (RFC 8446 Section 4.4.2)
//!
//! The Certificate message conveys the server's certificate chain to the client.

use crate::error::{Result, TlsError};

/// Certificate message
///
/// ```text
/// struct {
///     opaque certificate_request_context<0..2^8-1>;
///     CertificateEntry certificate_list<0..2^24-1>;
/// } Certificate;
///
/// struct {
///     opaque cert_data<1..2^24-1>;
///     Extension extensions<0..2^16-1>;
/// } CertificateEntry;
/// ```
#[derive(Debug, Clone)]
pub struct Certificate {
    /// Certificate request context (empty for server certificates)
    pub certificate_request_context: Vec<u8>,

    /// List of certificates in the chain (leaf first, root last)
    pub certificate_list: Vec<CertificateEntry>,
}

/// A single certificate entry
#[derive(Debug, Clone)]
pub struct CertificateEntry {
    /// Certificate data (X.509 certificate in DER format)
    pub cert_data: Vec<u8>,

    /// Extensions (e.g., `status_request` for OCSP stapling)
    pub extensions: Vec<u8>, // Simplified for now - will parse later
}

impl Certificate {
    /// Create a new Certificate message
    #[must_use]
    pub const fn new(certificate_list: Vec<CertificateEntry>) -> Self {
        Self {
            certificate_request_context: Vec::new(), // Empty for server certs
            certificate_list,
        }
    }

    /// Create a Certificate message with context (for client certificates)
    #[must_use]
    pub const fn new_with_context(
        certificate_request_context: Vec<u8>,
        certificate_list: Vec<CertificateEntry>,
    ) -> Self {
        Self {
            certificate_request_context,
            certificate_list,
        }
    }

    /// Get the leaf certificate (first in chain)
    #[must_use]
    pub fn leaf_certificate(&self) -> Option<&CertificateEntry> {
        self.certificate_list.first()
    }

    /// Validate Certificate message
    ///
    /// # Errors
    ///
    /// Returns an error if the certificate list is empty or any certificate has empty data.
    pub fn validate(&self) -> Result<()> {
        // Must have at least one certificate
        if self.certificate_list.is_empty() {
            return Err(TlsError::CertificateError("Certificate list cannot be empty".to_string()));
        }

        // Each certificate must have data
        for (i, entry) in self.certificate_list.iter().enumerate() {
            if entry.cert_data.is_empty() {
                return Err(TlsError::CertificateError(format!("Certificate {i} has empty data")));
            }
        }

        Ok(())
    }
}

impl CertificateEntry {
    /// Create a new certificate entry
    #[must_use]
    pub const fn new(cert_data: Vec<u8>) -> Self {
        Self {
            cert_data,
            extensions: Vec::new(),
        }
    }

    /// Create a certificate entry with extensions
    #[must_use]
    pub const fn new_with_extensions(cert_data: Vec<u8>, extensions: Vec<u8>) -> Self {
        Self {
            cert_data,
            extensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_new() {
        let cert_data = vec![1, 2, 3, 4]; // Mock certificate data
        let entry = CertificateEntry::new(cert_data.clone());
        let cert = Certificate::new(vec![entry]);

        assert!(cert.certificate_request_context.is_empty());
        assert_eq!(cert.certificate_list.len(), 1);
        assert_eq!(cert.certificate_list[0].cert_data, cert_data);
    }

    #[test]
    fn test_certificate_with_context() {
        let context = vec![5, 6, 7];
        let entry = CertificateEntry::new(vec![1, 2, 3, 4]);
        let cert = Certificate::new_with_context(context.clone(), vec![entry]);

        assert_eq!(cert.certificate_request_context, context);
    }

    #[test]
    fn test_leaf_certificate() {
        let entry1 = CertificateEntry::new(vec![1, 2, 3]);
        let entry2 = CertificateEntry::new(vec![4, 5, 6]);
        let cert = Certificate::new(vec![entry1.clone(), entry2]);

        let leaf = cert.leaf_certificate().unwrap();
        assert_eq!(leaf.cert_data, entry1.cert_data);
    }

    #[test]
    fn test_certificate_validation_success() {
        let entry = CertificateEntry::new(vec![1, 2, 3, 4]);
        let cert = Certificate::new(vec![entry]);
        assert!(cert.validate().is_ok());
    }

    #[test]
    fn test_certificate_validation_empty_list() {
        let cert = Certificate::new(vec![]);
        assert!(cert.validate().is_err());
    }

    #[test]
    fn test_certificate_validation_empty_data() {
        let entry = CertificateEntry::new(vec![]);
        let cert = Certificate::new(vec![entry]);
        assert!(cert.validate().is_err());
    }

    #[test]
    fn test_certificate_entry_with_extensions() {
        let cert_data = vec![1, 2, 3];
        let extensions = vec![7, 8, 9];
        let entry = CertificateEntry::new_with_extensions(cert_data.clone(), extensions.clone());

        assert_eq!(entry.cert_data, cert_data);
        assert_eq!(entry.extensions, extensions);
    }
}
