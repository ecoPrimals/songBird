// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `ServerHello` message (RFC 8446 Section 4.1.3)
//!
//! The `ServerHello` message is sent by the server in response to `ClientHello`.
//! It contains the server's selected cipher suite, key share, and other parameters.

use super::Extension;
use crate::error::{Result, TlsError};

/// `ServerHello` message
///
/// ```text
/// struct {
///     ProtocolVersion legacy_version = 0x0303;  // TLS 1.2 for compatibility
///     Random random;                             // 32 bytes
///     opaque legacy_session_id_echo<0..32>;      // Echo client's session ID
///     CipherSuite cipher_suite;                  // Selected cipher suite
///     uint8 legacy_compression_method = 0;       // Legacy (always 0)
///     Extension extensions<6..2^16-1>;           // Extensions
/// } ServerHello;
/// ```
#[derive(Debug, Clone)]
pub struct ServerHello {
    /// Legacy version (always 0x0303 for TLS 1.3)
    pub legacy_version: u16,

    /// Server random (32 bytes of randomness)
    pub random: [u8; 32],

    /// Echo of client's legacy session ID
    pub legacy_session_id_echo: Vec<u8>,

    /// Selected cipher suite
    pub cipher_suite: u16,

    /// Legacy compression method (always 0)
    pub legacy_compression_method: u8,

    /// Extensions (must include `supported_versions` and `key_share`)
    pub extensions: Vec<Extension>,
}

impl ServerHello {
    /// Create a new `ServerHello` message
    #[must_use]
    pub const fn new(
        random: [u8; 32],
        legacy_session_id_echo: Vec<u8>,
        cipher_suite: u16,
        extensions: Vec<Extension>,
    ) -> Self {
        Self {
            legacy_version: 0x0303, // TLS 1.2 for compatibility
            random,
            legacy_session_id_echo,
            cipher_suite,
            legacy_compression_method: 0, // No compression
            extensions,
        }
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

    /// Get the supported version from extensions
    #[must_use]
    pub fn get_supported_version(&self) -> Option<u16> {
        for ext in &self.extensions {
            if let Extension::SupportedVersions(versions) = ext {
                return versions.first().copied();
            }
        }
        None
    }

    /// Validate `ServerHello` message
    ///
    /// # Errors
    ///
    /// Returns an error if required extensions are missing.
    pub fn validate(&self) -> Result<()> {
        // Must have extensions
        if self.extensions.is_empty() {
            return Err(TlsError::ProtocolError(String::from(
                "ServerHello must contain extensions in TLS 1.3",
            )));
        }

        // Must have supported_versions extension
        if self.get_supported_version().is_none() {
            return Err(TlsError::ProtocolError(String::from(
                "ServerHello must contain supported_versions extension",
            )));
        }

        // Must have key_share extension
        if self.get_key_share().is_none() {
            return Err(TlsError::ProtocolError(String::from(
                "ServerHello must contain key_share extension",
            )));
        }

        // Legacy session ID echo must be <= 32 bytes
        if self.legacy_session_id_echo.len() > 32 {
            return Err(TlsError::ProtocolError(String::from(
                "Legacy session ID echo must be <= 32 bytes",
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_hello_new() {
        let random = [99u8; 32];
        let session_id = vec![1, 2, 3];
        let cipher_suite = 0x1303; // TLS_CHACHA20_POLY1305_SHA256
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            Extension::KeyShare(vec![5, 6, 7, 8]),
        ];

        let hello = ServerHello::new(random, session_id.clone(), cipher_suite, extensions);

        assert_eq!(hello.legacy_version, 0x0303);
        assert_eq!(hello.random, random);
        assert_eq!(hello.legacy_session_id_echo, session_id);
        assert_eq!(hello.cipher_suite, cipher_suite);
        assert_eq!(hello.legacy_compression_method, 0);
    }

    #[test]
    fn test_server_hello_validation_success() {
        let random = [99u8; 32];
        let extensions =
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![5, 6, 7, 8])];

        let hello = ServerHello::new(random, vec![], 0x1303, extensions);
        assert!(hello.validate().is_ok());
    }

    #[test]
    fn test_server_hello_validation_no_extensions() {
        let random = [99u8; 32];
        let hello = ServerHello::new(random, vec![], 0x1303, vec![]);
        assert!(hello.validate().is_err());
    }

    #[test]
    fn test_server_hello_validation_no_key_share() {
        let random = [99u8; 32];
        let extensions = vec![Extension::SupportedVersions(vec![0x0304])];

        let hello = ServerHello::new(random, vec![], 0x1303, extensions);
        assert!(hello.validate().is_err());
    }

    #[test]
    fn test_get_key_share() {
        let random = [99u8; 32];
        let key_share_data = vec![5, 6, 7, 8];
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]),
            Extension::KeyShare(key_share_data.clone()),
        ];

        let hello = ServerHello::new(random, vec![], 0x1303, extensions);
        assert_eq!(hello.get_key_share(), Some(key_share_data.as_slice()));
    }

    #[test]
    fn test_get_supported_version() {
        let random = [99u8; 32];
        let extensions =
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![5, 6, 7, 8])];

        let hello = ServerHello::new(random, vec![], 0x1303, extensions);
        assert_eq!(hello.get_supported_version(), Some(0x0304));
    }

    /// RFC 8446 Section 4.1.3 — fixed random value marks HelloRetryRequest.
    const HELLO_RETRY_REQUEST_RANDOM: [u8; 32] = [
        0xCF, 0x21, 0xAD, 0x74, 0xE5, 0x9A, 0x61, 0x11, 0xBE, 0x1B, 0x8B, 0x88, 0x9A, 0x82, 0x41,
        0xAC, 0xF2, 0xC1, 0x9F, 0x22, 0x57, 0xA3, 0x8B, 0x92, 0xC1, 0xDD, 0x8C, 0x90, 0x55, 0x40,
        0xC4, 0x70,
    ];

    fn is_hello_retry_request(hello: &ServerHello) -> bool {
        hello.random == HELLO_RETRY_REQUEST_RANDOM
    }

    #[test]
    fn test_cipher_suite_selection_echo() {
        for suite in [0x1301, 0x1302, 0x1303] {
            let hello = ServerHello::new(
                [1u8; 32],
                vec![],
                suite,
                vec![
                    Extension::SupportedVersions(vec![0x0304]),
                    Extension::KeyShare(vec![0x11; 32]),
                ],
            );
            assert_eq!(hello.cipher_suite, suite);
            assert!(hello.validate().is_ok());
        }
    }

    #[test]
    fn test_random_bytes_preserved() {
        let random: [u8; 32] = core::array::from_fn(|i| u8::try_from(i).unwrap());
        let hello = ServerHello::new(
            random,
            vec![],
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x22; 32])],
        );
        assert_eq!(hello.random, random);
        assert_ne!(hello.random, [0u8; 32]);
    }

    #[test]
    fn test_server_extensions_key_share_and_supported_versions() {
        let key_share = vec![0x33; 32];
        let hello = ServerHello::new(
            [2u8; 32],
            vec![],
            0x1303,
            vec![
                Extension::SupportedVersions(vec![0x0304]),
                Extension::KeyShare(key_share.clone()),
            ],
        );
        assert_eq!(hello.get_supported_version(), Some(0x0304));
        assert_eq!(hello.get_key_share(), Some(key_share.as_slice()));
    }

    #[test]
    fn test_hello_retry_request_magic_random_detected() {
        let hrr = ServerHello::new(
            HELLO_RETRY_REQUEST_RANDOM,
            vec![],
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x44; 32])],
        );
        assert!(is_hello_retry_request(&hrr));
    }

    #[test]
    fn test_normal_server_hello_is_not_hello_retry_request() {
        let hello = ServerHello::new(
            [0x77; 32],
            vec![],
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x55; 32])],
        );
        assert!(!is_hello_retry_request(&hello));
    }

    #[test]
    fn test_max_session_id_echo_validates() {
        let session_echo = vec![0xEE; 32];
        let hello = ServerHello::new(
            [3u8; 32],
            session_echo.clone(),
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x66; 32])],
        );
        assert_eq!(hello.legacy_session_id_echo, session_echo);
        assert!(hello.validate().is_ok());
    }

    #[test]
    fn test_session_id_echo_over_32_bytes_rejected() {
        let hello = ServerHello::new(
            [4u8; 32],
            vec![0xFF; 33],
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x77; 32])],
        );
        let err = hello.validate().unwrap_err();
        assert!(matches!(err, TlsError::ProtocolError(_)));
        assert!(err.to_string().contains("Legacy session ID echo"));
    }
}
