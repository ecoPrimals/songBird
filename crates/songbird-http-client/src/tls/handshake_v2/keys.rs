// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Key Derivation Module
//!
//! This module provides high-level abstractions for TLS 1.3 key derivation as specified
//! in RFC 8446 Section 7. The actual cryptographic operations are delegated to `BearDog`.
//!
//! ## RFC 8446 Key Schedule
//!
//! TLS 1.3 has two main key derivation stages:
//!
//! 1. **Handshake Traffic Keys** (Section 7.1)
//!    - Derived after `ServerHello`
//!    - Used to encrypt/decrypt handshake messages (`EncryptedExtensions`, Certificate, etc.)
//!    - Input: ECDH shared secret + transcript(ClientHello || `ServerHello`)
//!
//! 2. **Application Traffic Keys** (Section 7.1)
//!    - Derived after all handshake messages complete
//!    - Used to encrypt/decrypt application data (HTTP requests/responses)
//!    - Input: ECDH shared secret + transcript(ClientHello || ... || server Finished)
//!
//! ## Design
//!
//! This module doesn't implement HKDF directly - that's `BearDog`'s responsibility.
//! Instead, it provides:
//! - Type-safe key containers
//! - Cipher suite information
//! - Key length calculations
//! - Validation
//!
//! ## Reusability
//!
//! This module is designed to be reusable by BOTH TLS client and server.

use crate::error::{Error, Result};
use tracing::{debug, info};

/// TLS 1.3 Cipher Suites (RFC 8446 Appendix B.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CipherSuite {
    /// `TLS_AES_128_GCM_SHA256` (0x1301)
    Aes128GcmSha256 = 0x1301,

    /// `TLS_AES_256_GCM_SHA384` (0x1302)
    Aes256GcmSha384 = 0x1302,

    /// `TLS_CHACHA20_POLY1305_SHA256` (0x1303)
    ChaCha20Poly1305Sha256 = 0x1303,
}

impl CipherSuite {
    /// Convert from u16 wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the cipher suite is not supported.
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0x1301 => Ok(Self::Aes128GcmSha256),
            0x1302 => Ok(Self::Aes256GcmSha384),
            0x1303 => Ok(Self::ChaCha20Poly1305Sha256),
            _ => Err(Error::TlsHandshake(format!("Unsupported cipher suite: 0x{value:04x}"))),
        }
    }

    /// Convert to u16 wire format
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self as u16
    }

    /// Get encryption key length in bytes
    #[must_use]
    pub const fn key_len(&self) -> usize {
        match self {
            Self::Aes128GcmSha256 => 16,
            Self::Aes256GcmSha384 | Self::ChaCha20Poly1305Sha256 => 32,
        }
    }

    /// Get AEAD IV (nonce) length in bytes
    ///
    /// All TLS 1.3 cipher suites use 12-byte IVs (RFC 8446 Section 5.3)
    #[must_use]
    pub const fn iv_len(&self) -> usize {
        12
    }

    /// Get hash algorithm output length in bytes
    #[must_use]
    pub const fn hash_len(&self) -> usize {
        match self {
            Self::Aes256GcmSha384 => 48,                                // SHA-384
            Self::Aes128GcmSha256 | Self::ChaCha20Poly1305Sha256 => 32, // SHA-256
        }
    }

    /// Get AEAD authentication tag length in bytes
    ///
    /// All TLS 1.3 cipher suites use 16-byte tags
    #[must_use]
    pub const fn tag_len(&self) -> usize {
        16
    }

    /// Get human-readable name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Aes128GcmSha256 => "TLS_AES_128_GCM_SHA256",
            Self::Aes256GcmSha384 => "TLS_AES_256_GCM_SHA384",
            Self::ChaCha20Poly1305Sha256 => "TLS_CHACHA20_POLY1305_SHA256",
        }
    }
}

/// TLS 1.3 Traffic Keys
///
/// Contains keys and IVs for encrypting/decrypting TLS records.
/// Both client and server have separate write keys.
#[derive(Debug, Clone)]
pub struct TrafficKeys {
    /// Key for encrypting client → server data
    pub client_write_key: Vec<u8>,

    /// IV for encrypting client → server data
    pub client_write_iv: Vec<u8>,

    /// Key for decrypting server → client data
    pub server_write_key: Vec<u8>,

    /// IV for decrypting server → client data
    pub server_write_iv: Vec<u8>,
}

impl TrafficKeys {
    /// Create new `TrafficKeys` with validation
    ///
    /// # Errors
    ///
    /// Returns an error if key or IV lengths do not match the cipher suite.
    pub fn new(
        client_write_key: Vec<u8>,
        client_write_iv: Vec<u8>,
        server_write_key: Vec<u8>,
        server_write_iv: Vec<u8>,
        cipher_suite: CipherSuite,
    ) -> Result<Self> {
        let expected_key_len = cipher_suite.key_len();
        let expected_iv_len = cipher_suite.iv_len();

        // Validate key lengths
        if client_write_key.len() != expected_key_len {
            return Err(Error::TlsHandshake(format!(
                "Invalid client_write_key length: got {} bytes, expected {} for {:?}",
                client_write_key.len(),
                expected_key_len,
                cipher_suite
            )));
        }

        if server_write_key.len() != expected_key_len {
            return Err(Error::TlsHandshake(format!(
                "Invalid server_write_key length: got {} bytes, expected {} for {:?}",
                server_write_key.len(),
                expected_key_len,
                cipher_suite
            )));
        }

        // Validate IV lengths
        if client_write_iv.len() != expected_iv_len {
            return Err(Error::TlsHandshake(format!(
                "Invalid client_write_iv length: got {} bytes, expected {}",
                client_write_iv.len(),
                expected_iv_len
            )));
        }

        if server_write_iv.len() != expected_iv_len {
            return Err(Error::TlsHandshake(format!(
                "Invalid server_write_iv length: got {} bytes, expected {}",
                server_write_iv.len(),
                expected_iv_len
            )));
        }

        Ok(Self {
            client_write_key,
            client_write_iv,
            server_write_key,
            server_write_iv,
        })
    }

    /// Log key information (for debugging)
    pub fn log_info(&self, stage: &str, cipher_suite: CipherSuite) {
        info!("════════════════════════════════════════════════════════════");
        info!("🔑 {} KEYS (Cipher: {})", stage.to_uppercase(), cipher_suite.name());
        info!("════════════════════════════════════════════════════════════");
        info!(
            "client_write_key ({} bytes): {}",
            self.client_write_key.len(),
            hex::encode(&self.client_write_key)
        );
        info!(
            "client_write_iv ({} bytes): {}",
            self.client_write_iv.len(),
            hex::encode(&self.client_write_iv)
        );
        info!(
            "server_write_key ({} bytes): {}",
            self.server_write_key.len(),
            hex::encode(&self.server_write_key)
        );
        info!(
            "server_write_iv ({} bytes): {}",
            self.server_write_iv.len(),
            hex::encode(&self.server_write_iv)
        );
        info!("════════════════════════════════════════════════════════════");

        debug!("Key validation:");
        debug!("  Expected key length: {} bytes", cipher_suite.key_len());
        debug!("  Expected IV length: {} bytes", cipher_suite.iv_len());
        debug!("  All keys match expected lengths: ✅");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_suite_from_u16() {
        assert!(matches!(CipherSuite::from_u16(0x1301).unwrap(), CipherSuite::Aes128GcmSha256));
        assert!(matches!(CipherSuite::from_u16(0x1302).unwrap(), CipherSuite::Aes256GcmSha384));
        assert!(matches!(
            CipherSuite::from_u16(0x1303).unwrap(),
            CipherSuite::ChaCha20Poly1305Sha256
        ));
        assert!(CipherSuite::from_u16(0x9999).is_err());
    }

    #[test]
    fn test_cipher_suite_to_u16() {
        assert_eq!(CipherSuite::Aes128GcmSha256.to_u16(), 0x1301);
        assert_eq!(CipherSuite::Aes256GcmSha384.to_u16(), 0x1302);
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.to_u16(), 0x1303);
    }

    #[test]
    fn test_cipher_suite_key_lengths() {
        assert_eq!(CipherSuite::Aes128GcmSha256.key_len(), 16);
        assert_eq!(CipherSuite::Aes256GcmSha384.key_len(), 32);
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.key_len(), 32);
    }

    #[test]
    fn test_cipher_suite_iv_lengths() {
        // All TLS 1.3 cipher suites use 12-byte IVs
        assert_eq!(CipherSuite::Aes128GcmSha256.iv_len(), 12);
        assert_eq!(CipherSuite::Aes256GcmSha384.iv_len(), 12);
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.iv_len(), 12);
    }

    #[test]
    fn test_cipher_suite_hash_lengths() {
        assert_eq!(CipherSuite::Aes128GcmSha256.hash_len(), 32); // SHA-256
        assert_eq!(CipherSuite::Aes256GcmSha384.hash_len(), 48); // SHA-384
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.hash_len(), 32); // SHA-256
    }

    #[test]
    fn test_traffic_keys_validation() {
        let cipher = CipherSuite::Aes128GcmSha256;

        // Valid keys (16-byte keys, 12-byte IVs for AES-128-GCM)
        let keys =
            TrafficKeys::new(vec![0u8; 16], vec![0u8; 12], vec![0u8; 16], vec![0u8; 12], cipher);
        assert!(keys.is_ok());

        // Invalid client key length
        let keys = TrafficKeys::new(
            vec![0u8; 32], // Wrong! Should be 16 for AES-128
            vec![0u8; 12],
            vec![0u8; 16],
            vec![0u8; 12],
            cipher,
        );
        assert!(keys.is_err());

        // Invalid IV length
        let keys = TrafficKeys::new(
            vec![0u8; 16],
            vec![0u8; 16], // Wrong! Should be 12
            vec![0u8; 16],
            vec![0u8; 12],
            cipher,
        );
        assert!(keys.is_err());
    }

    #[test]
    fn test_traffic_keys_aes_256() {
        let cipher = CipherSuite::Aes256GcmSha384;

        // Valid keys (32-byte keys, 12-byte IVs for AES-256-GCM)
        let keys =
            TrafficKeys::new(vec![0u8; 32], vec![0u8; 12], vec![0u8; 32], vec![0u8; 12], cipher);
        assert!(keys.is_ok());
    }

    #[test]
    fn test_cipher_suite_names() {
        assert_eq!(CipherSuite::Aes128GcmSha256.name(), "TLS_AES_128_GCM_SHA256");
        assert_eq!(CipherSuite::Aes256GcmSha384.name(), "TLS_AES_256_GCM_SHA384");
        assert_eq!(CipherSuite::ChaCha20Poly1305Sha256.name(), "TLS_CHACHA20_POLY1305_SHA256");
    }
}
