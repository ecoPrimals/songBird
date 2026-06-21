// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
            return Err(TlsError::CertificateError(String::from(
                "Certificate list cannot be empty",
            )));
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

    const STUB_CERTIFICATE_DER: &[u8] = &[1, 2, 3, 4];

    #[test]
    fn test_certificate_new() {
        let cert_data = STUB_CERTIFICATE_DER.to_vec();
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

    fn encode_certificate_wire(cert: &Certificate) -> Vec<u8> {
        use crate::codec::bytes::{write_vec8, write_vec16, write_vec24};

        let mut buf = Vec::new();
        write_vec8(&mut buf, &cert.certificate_request_context).unwrap();
        let mut list_buf = Vec::new();
        for entry in &cert.certificate_list {
            write_vec24(&mut list_buf, &entry.cert_data).unwrap();
            write_vec16(&mut list_buf, &entry.extensions).unwrap();
        }
        write_vec24(&mut buf, &list_buf).unwrap();
        buf
    }

    fn decode_certificate_wire(buf: &[u8]) -> Certificate {
        use crate::codec::bytes::{read_vec8, read_vec16, read_vec24};

        let mut offset = 0;
        let certificate_request_context = read_vec8(buf, &mut offset).unwrap();
        let list_data = read_vec24(buf, &mut offset).unwrap();
        let mut list_offset = 0;
        let mut certificate_list = Vec::new();
        while list_offset < list_data.len() {
            let cert_data = read_vec24(&list_data, &mut list_offset).unwrap();
            let extensions = read_vec16(&list_data, &mut list_offset).unwrap();
            certificate_list.push(CertificateEntry::new_with_extensions(cert_data, extensions));
        }
        Certificate::new_with_context(certificate_request_context, certificate_list)
    }

    #[test]
    fn test_certificate_chain_multiple_entries() {
        let chain = vec![
            CertificateEntry::new(vec![0x01, 0x02]),
            CertificateEntry::new(vec![0x03, 0x04, 0x05]),
            CertificateEntry::new(vec![0x06]),
        ];
        let cert = Certificate::new(chain);
        assert_eq!(cert.certificate_list.len(), 3);
        assert!(cert.validate().is_ok());
        assert_eq!(cert.leaf_certificate().unwrap().cert_data, vec![0x01, 0x02]);
    }

    #[test]
    fn test_leaf_certificate_none_for_empty_chain() {
        let cert = Certificate::new(vec![]);
        assert!(cert.leaf_certificate().is_none());
    }

    #[test]
    fn test_certificate_chain_validation_rejects_empty_entry_in_chain() {
        let cert = Certificate::new(vec![
            CertificateEntry::new(vec![1, 2, 3]),
            CertificateEntry::new(vec![]),
        ]);
        let err = cert.validate().unwrap_err();
        assert!(matches!(err, TlsError::CertificateError(_)));
        assert!(err.to_string().contains("Certificate 1"));
    }

    #[test]
    fn test_certificate_wire_roundtrip_multiple_certs() {
        let cert = Certificate::new_with_context(
            vec![0xAA],
            vec![
                CertificateEntry::new_with_extensions(vec![1, 2, 3], vec![0x10, 0x11]),
                CertificateEntry::new(vec![4, 5, 6, 7]),
            ],
        );
        let wire = encode_certificate_wire(&cert);
        let decoded = decode_certificate_wire(&wire);
        assert_eq!(decoded.certificate_request_context, cert.certificate_request_context);
        assert_eq!(decoded.certificate_list.len(), 2);
        assert_eq!(decoded.certificate_list[0].cert_data, vec![1, 2, 3]);
        assert_eq!(decoded.certificate_list[0].extensions, vec![0x10, 0x11]);
        assert_eq!(decoded.certificate_list[1].cert_data, vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_certificate_wire_roundtrip_empty_list() {
        let cert = Certificate::new(vec![]);
        let wire = encode_certificate_wire(&cert);
        let decoded = decode_certificate_wire(&wire);
        assert!(decoded.certificate_list.is_empty());
        assert!(decoded.validate().is_err());
    }

    #[test]
    fn test_certificate_entry_extensions_preserved_in_chain() {
        let ext_a = vec![0x00, 0x05, 0x00, 0x00, 0x01]; // stub OCSP/status_request bytes
        let ext_b = vec![0x00, 0x00];
        let cert = Certificate::new(vec![
            CertificateEntry::new_with_extensions(vec![0xDE, 0xAD], ext_a.clone()),
            CertificateEntry::new_with_extensions(vec![0xBE, 0xEF], ext_b.clone()),
        ]);
        assert_eq!(cert.certificate_list[0].extensions, ext_a);
        assert_eq!(cert.certificate_list[1].extensions, ext_b);
        let decoded = decode_certificate_wire(&encode_certificate_wire(&cert));
        assert_eq!(decoded.certificate_list[0].extensions, ext_a);
        assert_eq!(decoded.certificate_list[1].extensions, ext_b);
    }
}
