// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 `ServerHello` Message Parser
//!
//! This module handles parsing RFC 8446 compliant `ServerHello` messages.
//!
//! ## RFC 8446 Compliance
//!
//! From RFC 8446 Section 4.1.3:
//! ```text
//! struct {
//!     ProtocolVersion legacy_version = 0x0303;    /* TLS 1.2 */
//!     Random random;
//!     opaque legacy_session_id_echo<0..32>;
//!     CipherSuite cipher_suite;
//!     uint8 legacy_compression_method = 0;
//!     Extension extensions<6..2^16-1>;
//! } ServerHello;
//! ```
//!
//! ## Design Philosophy
//!
//! - **Agnostic**: Parses any RFC-compliant `ServerHello`
//! - **Defensive**: Validates all fields
//! - **Informative**: Logs cipher suite selection
//! - **Reusable**: Pure parsing logic, no state
//!
//! ## Reusability
//!
//! This module is designed to be reusable for any TLS 1.3 client implementation.

use crate::error::{Error, Result};
use crate::tls::handshake_v2::keys::CipherSuite;
use tracing::{debug, info, warn};

/// Parsed `ServerHello` data
#[derive(Debug, Clone)]
pub struct ServerHello {
    /// Server random (32 bytes)
    pub server_random: Vec<u8>,

    /// Server's x25519 public key (32 bytes)
    pub server_public_key: Vec<u8>,

    /// Negotiated cipher suite
    pub cipher_suite: CipherSuite,
}

/// Parse a `ServerHello` handshake message
///
/// # Arguments
/// * `data` - Complete handshake message (including type + length header)
///
/// # Returns
/// * `Ok(ServerHello)` - Parsed `ServerHello` data
/// * `Err` - If parsing fails
///
/// # Example
/// ```rust,ignore
/// let server_hello = parse_server_hello(&handshake_message)?;
/// println!("Server chose: {}", server_hello.cipher_suite.name());
/// ```
pub fn parse_server_hello(data: &[u8]) -> Result<ServerHello> {
    debug!("Parsing ServerHello: {} bytes", data.len());

    // Validate handshake message type
    if data.is_empty() || data[0] != 0x02 {
        return Err(Error::TlsHandshake(format!(
            "Invalid ServerHello: expected type 0x02, got 0x{:02x}",
            data.first().copied().unwrap_or(0xFF)
        )));
    }

    // Skip handshake header (type: 1 byte + length: 3 bytes)
    if data.len() < 4 {
        return Err(Error::TlsHandshake("ServerHello too short for header".to_string()));
    }
    let mut offset = 4;

    // Skip legacy version (2 bytes)
    if data.len() < offset + 2 {
        return Err(Error::TlsHandshake("ServerHello too short for version".to_string()));
    }
    offset += 2;

    // Server random (32 bytes)
    if data.len() < offset + 32 {
        return Err(Error::TlsHandshake("ServerHello too short for random".to_string()));
    }
    let server_random = data[offset..offset + 32].to_vec();
    offset += 32;
    debug!("Server random: {} bytes", server_random.len());

    // Legacy session ID
    if data.len() < offset + 1 {
        return Err(Error::TlsHandshake("ServerHello truncated at session ID".to_string()));
    }
    let session_id_len = data[offset] as usize;
    offset += 1 + session_id_len;

    if data.len() < offset {
        return Err(Error::TlsHandshake("ServerHello truncated in session ID".to_string()));
    }

    // Cipher suite (2 bytes) - CRITICAL for selecting correct AEAD algorithm!
    if data.len() < offset + 2 {
        return Err(Error::TlsHandshake("ServerHello truncated at cipher suite".to_string()));
    }
    let cipher_suite_value = u16::from_be_bytes([data[offset], data[offset + 1]]);
    let cipher_suite = CipherSuite::from_u16(cipher_suite_value)?;

    info!("🔐 Server negotiated cipher suite: 0x{:04x}", cipher_suite_value);
    info!("   → {}", cipher_suite.name());

    // Log cipher suite details
    match cipher_suite {
        CipherSuite::Aes128GcmSha256 => {
            info!("   → Most common, hardware accelerated (AES-NI)");
        }
        CipherSuite::Aes256GcmSha384 => {
            info!("   → High security, hardware accelerated (AES-NI)");
        }
        CipherSuite::ChaCha20Poly1305Sha256 => {
            info!("   → Software-only, mobile-optimized");
        }
    }

    offset += 2;

    // Skip legacy compression method (1 byte)
    if data.len() < offset + 1 {
        return Err(Error::TlsHandshake("ServerHello truncated at compression".to_string()));
    }
    offset += 1;

    // Parse extensions to extract server's public key
    if data.len() < offset {
        return Err(Error::TlsHandshake("ServerHello has no extensions".to_string()));
    }
    let extensions_data = &data[offset..];
    let server_public_key = extract_key_share(extensions_data)?;

    debug!("Parsed ServerHello successfully");
    debug!("  Server random: {} bytes", server_random.len());
    debug!("  Server public key: {} bytes", server_public_key.len());
    debug!("  Cipher suite: {}", cipher_suite.name());

    Ok(ServerHello {
        server_random,
        server_public_key,
        cipher_suite,
    })
}

/// Extract server's public key from `key_share` extension
///
/// RFC 8446 Section 4.2.8: Key Share Extension
fn extract_key_share(extensions_data: &[u8]) -> Result<Vec<u8>> {
    if extensions_data.len() < 2 {
        return Err(Error::TlsHandshake("Extensions data too short".to_string()));
    }

    // Extensions length (2 bytes)
    let extensions_length = u16::from_be_bytes([extensions_data[0], extensions_data[1]]) as usize;

    if extensions_data.len() < 2 + extensions_length {
        return Err(Error::TlsHandshake("Extensions length exceeds data".to_string()));
    }

    let mut offset = 2;
    let extensions_end = 2 + extensions_length;

    // Parse extensions
    while offset + 4 <= extensions_end {
        let ext_type = u16::from_be_bytes([extensions_data[offset], extensions_data[offset + 1]]);
        let ext_length =
            u16::from_be_bytes([extensions_data[offset + 2], extensions_data[offset + 3]]) as usize;
        offset += 4;

        if offset + ext_length > extensions_end {
            return Err(Error::TlsHandshake("Extension data truncated".to_string()));
        }

        // Key share extension (0x0033)
        if ext_type == 0x0033 {
            debug!("Found key_share extension ({} bytes)", ext_length);
            let ext_data = &extensions_data[offset..offset + ext_length];

            // RFC 8446 Section 4.2.8: KeyShareEntry
            // struct {
            //     NamedGroup group;        // 2 bytes
            //     opaque key_exchange<1..2^16-1>;  // 2 bytes length + data
            // } KeyShareEntry;

            if ext_data.len() < 4 {
                return Err(Error::TlsHandshake("key_share extension too short".to_string()));
            }

            let group = u16::from_be_bytes([ext_data[0], ext_data[1]]);
            let key_length = u16::from_be_bytes([ext_data[2], ext_data[3]]) as usize;

            debug!(
                "  Group: 0x{:04x} ({})",
                group,
                if group == 0x001d {
                    "x25519"
                } else {
                    "unknown"
                }
            );
            debug!("  Key length: {} bytes", key_length);

            if ext_data.len() < 4 + key_length {
                return Err(Error::TlsHandshake("key_share key data truncated".to_string()));
            }

            let public_key = ext_data[4..4 + key_length].to_vec();

            if group != 0x001d {
                warn!("⚠️  Server selected non-x25519 group: 0x{:04x}", group);
            }

            if public_key.len() != 32 {
                warn!("⚠️  x25519 public key is not 32 bytes: {} bytes", public_key.len());
            }

            return Ok(public_key);
        }

        offset += ext_length;
    }

    Err(Error::TlsHandshake("key_share extension not found in ServerHello".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_server_hello_structure() {
        // Minimal valid ServerHello
        let mut msg = vec![
            0x02, // HandshakeType: ServerHello
            0x00, 0x00, 0x46, // Length: 70 bytes
            0x03, 0x03, // Legacy version (TLS 1.2)
        ];

        // Server random (32 bytes)
        msg.extend_from_slice(&[0u8; 32]);

        // Session ID (empty)
        msg.push(0);

        // Cipher suite (AES-128-GCM-SHA256)
        msg.extend_from_slice(&[0x13, 0x01]);

        // Compression (none)
        msg.push(0);

        // Extensions
        let mut ext = Vec::new();
        // Key share extension (0x0033)
        ext.extend_from_slice(&[0x00, 0x33]);
        ext.extend_from_slice(&[0x00, 0x24]); // Length: 36 bytes
        ext.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        ext.extend_from_slice(&[0x00, 0x20]); // Key length: 32 bytes
        ext.extend_from_slice(&[1u8; 32]); // Public key

        // Extensions length
        msg.extend_from_slice(&(ext.len() as u16).to_be_bytes());
        msg.extend_from_slice(&ext);

        let result = parse_server_hello(&msg);
        assert!(result.is_ok());

        let server_hello = result.unwrap();
        assert_eq!(server_hello.server_random.len(), 32);
        assert_eq!(server_hello.server_public_key.len(), 32);
        assert!(matches!(server_hello.cipher_suite, CipherSuite::Aes128GcmSha256));
    }

    #[test]
    fn test_parse_server_hello_invalid_type() {
        let msg = vec![0x01]; // ClientHello type, not ServerHello
        let result = parse_server_hello(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_server_hello_too_short() {
        let msg = vec![0x02, 0x00, 0x00]; // Only 3 bytes
        let result = parse_server_hello(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_key_share() {
        let mut ext_data = Vec::new();

        // Extensions length
        ext_data.extend_from_slice(&[0x00, 0x28]); // 40 bytes

        // Key share extension
        ext_data.extend_from_slice(&[0x00, 0x33]); // Type: key_share
        ext_data.extend_from_slice(&[0x00, 0x24]); // Length: 36 bytes
        ext_data.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        ext_data.extend_from_slice(&[0x00, 0x20]); // Key length: 32 bytes
        ext_data.extend_from_slice(&[42u8; 32]); // Public key

        let result = extract_key_share(&ext_data);
        assert!(result.is_ok());

        let key = result.unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(key[0], 42);
    }

    #[test]
    fn test_extract_key_share_missing() {
        let mut ext_data = Vec::new();

        // Extensions length
        ext_data.extend_from_slice(&[0x00, 0x04]); // 4 bytes

        // Some other extension (not key_share)
        ext_data.extend_from_slice(&[0x00, 0x00]); // Type: SNI
        ext_data.extend_from_slice(&[0x00, 0x00]); // Length: 0

        let result = extract_key_share(&ext_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_suite_parsing() {
        let mut msg = vec![
            0x02, 0x00, 0x00, 0x46, // Header
            0x03, 0x03, // Version
        ];
        msg.extend_from_slice(&[0u8; 32]); // Random
        msg.push(0); // Session ID
        msg.extend_from_slice(&[0x13, 0x03]); // ChaCha20-Poly1305
        msg.push(0); // Compression

        // Minimal key share extension
        let mut ext = Vec::new();
        ext.extend_from_slice(&[0x00, 0x33, 0x00, 0x24]);
        ext.extend_from_slice(&[0x00, 0x1d, 0x00, 0x20]);
        ext.extend_from_slice(&[1u8; 32]);

        msg.extend_from_slice(&(ext.len() as u16).to_be_bytes());
        msg.extend_from_slice(&ext);

        let result = parse_server_hello(&msg).unwrap();
        assert!(matches!(result.cipher_suite, CipherSuite::ChaCha20Poly1305Sha256));
    }
}
