// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `ClientHello` message (RFC 8446 Section 4.1.2)
//!
//! The `ClientHello` message is the first message in the TLS handshake.
//! It contains the client's supported versions, cipher suites, and extensions.

use super::Extension;
use crate::error::{Result, TlsError};

/// `ClientHello` message
///
/// ```text
/// struct {
///     ProtocolVersion legacy_version = 0x0303;  // TLS 1.2 for compatibility
///     Random random;                             // 32 bytes
///     opaque legacy_session_id<0..32>;           // Legacy field (empty in TLS 1.3)
///     CipherSuite cipher_suites<2..2^16-2>;      // List of supported cipher suites
///     opaque legacy_compression_methods<1..2^8-1>; // Legacy (single zero byte)
///     Extension extensions<8..2^16-1>;           // Extensions (required in TLS 1.3)
/// } ClientHello;
/// ```
#[derive(Debug, Clone)]
pub struct ClientHello {
    /// Legacy version (always 0x0303 for TLS 1.3)
    pub legacy_version: u16,

    /// Client random (32 bytes of randomness)
    pub random: [u8; 32],

    /// Legacy session ID (empty in TLS 1.3, but kept for middlebox compatibility)
    pub legacy_session_id: Vec<u8>,

    /// List of cipher suites supported by client
    pub cipher_suites: Vec<u16>,

    /// Legacy compression methods (always `[0]` in TLS 1.3)
    pub legacy_compression_methods: Vec<u8>,

    /// Extensions (required - must include `supported_versions` and `key_share`)
    pub extensions: Vec<Extension>,
}

impl ClientHello {
    /// Create a new `ClientHello` message
    #[must_use]
    pub fn new(random: [u8; 32], cipher_suites: Vec<u16>, extensions: Vec<Extension>) -> Self {
        Self {
            legacy_version: 0x0303, // TLS 1.2 for compatibility
            random,
            legacy_session_id: Vec::new(), // Empty in TLS 1.3
            cipher_suites,
            legacy_compression_methods: vec![0], // No compression
            extensions,
        }
    }

    /// Get the supported TLS version from extensions
    #[must_use]
    pub fn get_supported_version(&self) -> Option<u16> {
        for ext in &self.extensions {
            if let Extension::SupportedVersions(versions) = ext {
                return versions.first().copied();
            }
        }
        None
    }

    /// Get the key share extension
    #[must_use]
    pub fn get_key_share(&self) -> Option<&[u8]> {
        for ext in &self.extensions {
            if let Extension::KeyShare(key_share) = ext {
                return Some(key_share);
            }
        }
        None
    }

    /// Validate `ClientHello` message
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or invalid.
    pub fn validate(&self) -> Result<()> {
        // Must have at least one cipher suite
        if self.cipher_suites.is_empty() {
            return Err(TlsError::ProtocolError(
                "ClientHello must contain at least one cipher suite".to_string(),
            ));
        }

        // Must have extensions in TLS 1.3
        if self.extensions.is_empty() {
            return Err(TlsError::ProtocolError(
                "ClientHello must contain extensions in TLS 1.3".to_string(),
            ));
        }

        // Must have supported_versions extension
        if self.get_supported_version().is_none() {
            return Err(TlsError::ProtocolError(
                "ClientHello must contain supported_versions extension".to_string(),
            ));
        }

        // Legacy session ID must be <= 32 bytes
        if self.legacy_session_id.len() > 32 {
            return Err(TlsError::ProtocolError(
                "Legacy session ID must be <= 32 bytes".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_hello_new() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303]; // TLS_CHACHA20_POLY1305_SHA256
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            Extension::KeyShare(vec![1, 2, 3, 4]),
        ];

        let hello = ClientHello::new(random, cipher_suites, extensions);

        assert_eq!(hello.legacy_version, 0x0303);
        assert_eq!(hello.random, random);
        assert_eq!(hello.cipher_suites, vec![0x1303]);
        assert_eq!(hello.extensions.len(), 2);
    }

    #[test]
    fn test_client_hello_validation_success() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions =
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])];

        let hello = ClientHello::new(random, cipher_suites, extensions);
        assert!(hello.validate().is_ok());
    }

    #[test]
    fn test_client_hello_validation_no_cipher_suites() {
        let random = [42u8; 32];
        let extensions = vec![Extension::SupportedVersions(vec![0x0304])];

        let hello = ClientHello::new(random, vec![], extensions);
        assert!(hello.validate().is_err());
    }

    #[test]
    fn test_client_hello_validation_no_extensions() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];

        let hello = ClientHello::new(random, cipher_suites, vec![]);
        assert!(hello.validate().is_err());
    }

    #[test]
    fn test_get_supported_version() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![Extension::SupportedVersions(vec![0x0304, 0x0303])];

        let hello = ClientHello::new(random, cipher_suites, extensions);
        assert_eq!(hello.get_supported_version(), Some(0x0304));
    }

    #[test]
    fn test_get_key_share() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let key_share_data = vec![1, 2, 3, 4];
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]),
            Extension::KeyShare(key_share_data.clone()),
        ];

        let hello = ClientHello::new(random, cipher_suites, extensions);
        assert_eq!(hello.get_key_share(), Some(key_share_data.as_slice()));
    }

    #[test]
    fn test_various_cipher_suite_combinations_validate() {
        let extensions =
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0xAB; 32])];
        for suites in [
            vec![0x1301],                 // AES-128-GCM
            vec![0x1302],                 // AES-256-GCM
            vec![0x1303],                 // ChaCha20-Poly1305
            vec![0x1301, 0x1302, 0x1303], // full preference list
            vec![0x1303, 0x1301],         // reordered
        ] {
            let hello = ClientHello::new([1u8; 32], suites.clone(), extensions.clone());
            assert!(hello.validate().is_ok());
            assert_eq!(hello.cipher_suites, suites);
        }
    }

    #[test]
    fn test_empty_session_id_is_default_and_valid() {
        let hello = ClientHello::new(
            [2u8; 32],
            vec![0x1303],
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])],
        );
        assert!(hello.legacy_session_id.is_empty());
        assert!(hello.validate().is_ok());
    }

    #[test]
    fn test_max_length_session_id_validates() {
        let mut hello = ClientHello::new(
            [3u8; 32],
            vec![0x1303],
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])],
        );
        hello.legacy_session_id = vec![0xAA; 32];
        assert_eq!(hello.legacy_session_id.len(), 32);
        assert!(hello.validate().is_ok());
    }

    #[test]
    fn test_session_id_over_32_bytes_rejected() {
        let mut hello = ClientHello::new(
            [4u8; 32],
            vec![0x1303],
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])],
        );
        hello.legacy_session_id = vec![0xBB; 33];
        let err = hello.validate().unwrap_err();
        assert!(matches!(err, TlsError::ProtocolError(_)));
        assert!(err.to_string().contains("Legacy session ID"));
    }

    #[test]
    fn test_legacy_fields_defaults() {
        let hello = ClientHello::new(
            [5u8; 32],
            vec![0x1303],
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1])],
        );
        assert_eq!(hello.legacy_version, 0x0303);
        assert_eq!(hello.legacy_compression_methods, vec![0]);
    }

    #[test]
    fn test_client_hello_with_all_extension_variants() {
        use crate::messages::extensions::{GROUP_X25519, SIG_ED25519};

        let extensions = vec![
            Extension::SupportedVersions(vec![0x0303, 0x0304]),
            Extension::KeyShare(vec![0xCD; 32]),
            Extension::ServerName("songbird.test".to_string()),
            Extension::SignatureAlgorithms(vec![SIG_ED25519]),
            Extension::SupportedGroups(vec![GROUP_X25519]),
            Extension::Unknown {
                extension_type: 0xBEEF,
                data: vec![1, 2, 3],
            },
        ];
        let hello = ClientHello::new([6u8; 32], vec![0x1303], extensions);
        assert!(hello.validate().is_ok());
        assert_eq!(hello.extensions.len(), 6);
        assert_eq!(hello.get_supported_version(), Some(0x0303));
    }

    #[test]
    fn test_validation_missing_supported_versions_extension() {
        let hello =
            ClientHello::new([7u8; 32], vec![0x1303], vec![Extension::KeyShare(vec![1, 2, 3, 4])]);
        let err = hello.validate().unwrap_err();
        assert!(matches!(err, TlsError::ProtocolError(_)));
        assert!(err.to_string().contains("supported_versions"));
    }
}
