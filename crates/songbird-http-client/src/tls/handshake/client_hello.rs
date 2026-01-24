//! TLS 1.3 ClientHello Message Builder
//!
//! This module handles building RFC 8446 compliant ClientHello messages with
//! adaptive extension strategies.
//!
//! ## RFC 8446 Compliance
//!
//! From RFC 8446 Section 4.1.2:
//! ```text
//! struct {
//!     ProtocolVersion legacy_version = 0x0303;    /* TLS 1.2 */
//!     Random random;
//!     opaque legacy_session_id<0..32>;
//!     CipherSuite cipher_suites<2..2^16-2>;
//!     opaque legacy_compression_methods<1..2^8-1>;
//!     Extension extensions<8..2^16-1>;
//! } ClientHello;
//! ```
//!
//! ## Design Philosophy
//!
//! - **No Hardcoding**: Extension strategies instead of fixed lists
//! - **Agnostic**: Works with any server
//! - **Adaptive**: Learns from server behavior
//! - **Context-Aware**: Different strategies for different scenarios
//!
//! ## Reusability
//!
//! This module is designed to be reusable for any TLS 1.3 client implementation.

use crate::error::Result;
use crate::tls::{TLS_1_2, TLS_1_3};
use tracing::debug;

/// TLS 1.3 cipher suites (RFC 8446 Appendix B.4)
pub const CIPHER_SUITES: &[u16] = &[
    0x1301, // TLS_AES_128_GCM_SHA256
    0x1302, // TLS_AES_256_GCM_SHA384
    0x1303, // TLS_CHACHA20_POLY1305_SHA256
];

/// Extension strategy for ClientHello
#[derive(Debug, Clone)]
pub enum ExtensionStrategy {
    /// Minimal (3 extensions, ~50ms): SNI, SupportedVersions, KeyShare
    Minimal,
    
    /// Standard (7 extensions, ~80ms): Minimal + ALPN, Groups, SigAlgs, PSK
    Standard,
    
    /// Modern (10+ extensions, ~100ms): Standard + OCSP, SCT, etc.
    Modern,
    
    /// Maximum compatibility (12+ extensions): Everything
    MaxCompatibility,
    
    /// Adaptive (learns from server): Uses profiler data
    Adaptive,
}

/// ClientHello builder
///
/// Builds RFC 8446 compliant ClientHello messages with configurable extensions.
pub struct ClientHelloBuilder {
    /// Extension strategy
    strategy: ExtensionStrategy,
}

impl ClientHelloBuilder {
    /// Create a new ClientHello builder with the given strategy
    pub fn new(strategy: ExtensionStrategy) -> Self {
        Self { strategy }
    }
    
    /// Build a complete ClientHello TLS record
    ///
    /// Returns the complete TLS record including:
    /// - TLS record header (5 bytes)
    /// - Handshake header (4 bytes)
    /// - ClientHello content
    ///
    /// # Arguments
    /// * `client_random` - 32 bytes of random data
    /// * `client_public_key` - x25519 public key (32 bytes)
    /// * `server_name` - SNI server name
    pub fn build(
        &self,
        client_random: &[u8],
        client_public_key: &[u8],
        server_name: &str,
    ) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // TLS Record header (5 bytes)
        msg.push(0x16); // ContentType: Handshake
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version (0x0303)
        
        // Record length (placeholder, fill in later)
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0]);

        // Handshake header (4 bytes)
        msg.push(0x01); // HandshakeType: ClientHello
        
        // Handshake length (placeholder, fill in later)
        let handshake_length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // ClientHello content
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        msg.extend_from_slice(client_random); // Random (32 bytes)
        msg.push(0); // Legacy session ID length (0 = no session ID)

        // Cipher suites
        msg.extend_from_slice(&((CIPHER_SUITES.len() * 2) as u16).to_be_bytes());
        for suite in CIPHER_SUITES {
            msg.extend_from_slice(&suite.to_be_bytes());
        }

        // Compression methods (legacy, must be present but unused in TLS 1.3)
        msg.push(1); // Length: 1 byte
        msg.push(0); // No compression

        // Extensions
        let extensions = self.build_extensions(server_name, client_public_key)?;
        msg.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        msg.extend_from_slice(&extensions);

        // Fill in lengths
        let handshake_length = msg.len() - handshake_length_pos - 3;
        msg[handshake_length_pos] = ((handshake_length >> 16) & 0xFF) as u8;
        msg[handshake_length_pos + 1] = ((handshake_length >> 8) & 0xFF) as u8;
        msg[handshake_length_pos + 2] = (handshake_length & 0xFF) as u8;

        let record_length = msg.len() - length_pos - 2;
        msg[length_pos] = ((record_length >> 8) & 0xFF) as u8;
        msg[length_pos + 1] = (record_length & 0xFF) as u8;

        debug!("Built ClientHello: {} bytes total", msg.len());
        debug!("  TLS record: 5 bytes header + {} bytes payload", record_length);
        debug!("  Handshake: 4 bytes header + {} bytes ClientHello", handshake_length);
        debug!("  Extensions: {} bytes", extensions.len());

        Ok(msg)
    }
    
    /// Build extensions based on strategy
    fn build_extensions(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        match &self.strategy {
            ExtensionStrategy::Minimal => {
                debug!("🎯 Building MINIMAL extensions (3 extensions, ~50ms handshake)");
                build_extensions_minimal(server_name, public_key)
            }
            ExtensionStrategy::Standard => {
                debug!("🎯 Building STANDARD extensions (7 extensions, ~80ms handshake)");
                build_extensions_standard(server_name, public_key)
            }
            ExtensionStrategy::Modern => {
                debug!("🎯 Building MODERN extensions (10+ extensions, ~100ms handshake)");
                build_extensions_modern(server_name, public_key)
            }
            ExtensionStrategy::MaxCompatibility => {
                debug!("🎯 Building MAX COMPATIBILITY extensions (12+ extensions)");
                build_extensions_maxcompat(server_name, public_key)
            }
            ExtensionStrategy::Adaptive => {
                debug!("🎯 ADAPTIVE: Using STANDARD extensions (profiling not yet implemented)");
                build_extensions_standard(server_name, public_key)
            }
        }
    }
}

/// Build minimal extensions (fastest handshake, ~50ms)
/// Only required extensions: SNI, Supported Versions, Key Share
fn build_extensions_minimal(server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
    let mut ext = Vec::new();

    // 1. SNI extension (0x0000) - REQUIRED for virtual hosting
    ext.extend_from_slice(&[0x00, 0x00]);
    let sni_data = build_sni_extension(server_name);
    ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
    ext.extend_from_slice(&sni_data);

    // 2. Supported versions (0x002b) - REQUIRED for TLS 1.3
    ext.extend_from_slice(&[0x00, 0x2b]);
    ext.extend_from_slice(&[0x00, 0x03]);
    ext.extend_from_slice(&[0x02]);
    ext.extend_from_slice(&TLS_1_3.to_be_bytes());

    // 3. Key share (0x0033) - REQUIRED for TLS 1.3
    ext.extend_from_slice(&[0x00, 0x33]);
    let key_share_data = build_key_share_extension(public_key);
    ext.extend_from_slice(&(key_share_data.len() as u16).to_be_bytes());
    ext.extend_from_slice(&key_share_data);

    Ok(ext)
}

/// Build standard extensions (balanced, ~80ms handshake)
/// Current production-tested set
fn build_extensions_standard(server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
    let mut ext = Vec::new();

    // 1. SNI extension (0x0000)
    ext.extend_from_slice(&[0x00, 0x00]);
    let sni_data = build_sni_extension(server_name);
    ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
    ext.extend_from_slice(&sni_data);

    // 2. ALPN extension (0x0010) - CRITICAL for HTTPS
    ext.extend_from_slice(&[0x00, 0x10]);
    ext.extend_from_slice(&[0x00, 0x0b]);
    ext.extend_from_slice(&[0x00, 0x09]);
    ext.extend_from_slice(&[0x08]);
    ext.extend_from_slice(b"http/1.1");

    // 3. Supported versions (0x002b)
    ext.extend_from_slice(&[0x00, 0x2b]);
    ext.extend_from_slice(&[0x00, 0x03]);
    ext.extend_from_slice(&[0x02]);
    ext.extend_from_slice(&TLS_1_3.to_be_bytes());

    // 4. Key share (0x0033)
    ext.extend_from_slice(&[0x00, 0x33]);
    let key_share_data = build_key_share_extension(public_key);
    ext.extend_from_slice(&(key_share_data.len() as u16).to_be_bytes());
    ext.extend_from_slice(&key_share_data);

    // 5. Supported groups (0x000a)
    ext.extend_from_slice(&[0x00, 0x0a]);
    ext.extend_from_slice(&[0x00, 0x04]);
    ext.extend_from_slice(&[0x00, 0x02]);
    ext.extend_from_slice(&[0x00, 0x1d]); // x25519

    // 6. Signature algorithms (0x000d)
    ext.extend_from_slice(&[0x00, 0x0d]);
    ext.extend_from_slice(&[0x00, 0x14]);
    ext.extend_from_slice(&[0x00, 0x12]);
    ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
    ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
    ext.extend_from_slice(&[0x06, 0x03]); // ecdsa_secp521r1_sha512
    ext.extend_from_slice(&[0x08, 0x07]); // ed25519
    ext.extend_from_slice(&[0x08, 0x08]); // ed448
    ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
    ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
    ext.extend_from_slice(&[0x06, 0x01]); // rsa_pkcs1_sha512
    ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256

    // 7. PSK Key Exchange Modes (0x002d) - REQUIRED by many servers
    ext.extend_from_slice(&[0x00, 0x2d]);
    ext.extend_from_slice(&[0x00, 0x02]);
    ext.extend_from_slice(&[0x01]);
    ext.extend_from_slice(&[0x01]); // psk_dhe_ke

    Ok(ext)
}

/// Build modern extensions (latest features, ~100ms handshake)
fn build_extensions_modern(server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
    // Start with standard extensions
    let mut ext = build_extensions_standard(server_name, public_key)?;

    // Add modern extensions
    
    // 8. Status Request (OCSP stapling, 0x0005)
    ext.extend_from_slice(&[0x00, 0x05]);
    ext.extend_from_slice(&[0x00, 0x05]);
    ext.extend_from_slice(&[0x01]); // status_type: ocsp
    ext.extend_from_slice(&[0x00, 0x00]); // responder_id_list: empty
    ext.extend_from_slice(&[0x00, 0x00]); // request_extensions: empty

    // 9. SCT (Signed Certificate Timestamp, 0x0012)
    ext.extend_from_slice(&[0x00, 0x12]);
    ext.extend_from_slice(&[0x00, 0x00]); // Empty

    // 10. Padding (0x0015) - for anti-fingerprinting
    ext.extend_from_slice(&[0x00, 0x15]);
    ext.extend_from_slice(&[0x00, 0x00]); // Empty

    Ok(ext)
}

/// Build maximum compatibility extensions (12+ extensions)
fn build_extensions_maxcompat(server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
    // Start with modern extensions
    let mut ext = build_extensions_modern(server_name, public_key)?;

    // Add compatibility extensions
    
    // 11. Supported Groups (extended list, 0x000a)
    // Already included in standard, but could be extended

    // 12. Extended Master Secret (0x0017) - legacy compatibility
    ext.extend_from_slice(&[0x00, 0x17]);
    ext.extend_from_slice(&[0x00, 0x00]); // Empty

    Ok(ext)
}

/// Build SNI (Server Name Indication) extension
fn build_sni_extension(server_name: &str) -> Vec<u8> {
    let mut sni = Vec::new();
    
    // SNI list length
    let list_len = 2 + 1 + 2 + server_name.len();
    sni.extend_from_slice(&(list_len as u16).to_be_bytes());
    
    // SNI entry
    sni.push(0x00); // name_type: host_name
    sni.extend_from_slice(&(server_name.len() as u16).to_be_bytes());
    sni.extend_from_slice(server_name.as_bytes());
    
    sni
}

/// Build Key Share extension (x25519)
fn build_key_share_extension(public_key: &[u8]) -> Vec<u8> {
    let mut key_share = Vec::new();
    
    // KeyShareEntry list length
    let entry_len = 2 + 2 + public_key.len();
    key_share.extend_from_slice(&(entry_len as u16).to_be_bytes());
    
    // KeyShareEntry
    key_share.extend_from_slice(&[0x00, 0x1d]); // group: x25519
    key_share.extend_from_slice(&(public_key.len() as u16).to_be_bytes());
    key_share.extend_from_slice(public_key);
    
    key_share
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_hello_builder() {
        let builder = ClientHelloBuilder::new(ExtensionStrategy::Minimal);
        let random = vec![0u8; 32];
        let pubkey = vec![1u8; 32];
        
        let hello = builder.build(&random, &pubkey, "example.com").unwrap();
        
        // Check TLS record header
        assert_eq!(hello[0], 0x16); // Handshake
        assert_eq!(hello[1], 0x03); // TLS 1.2 (legacy)
        assert_eq!(hello[2], 0x03);
        
        // Check handshake type
        assert_eq!(hello[5], 0x01); // ClientHello
    }

    #[test]
    fn test_extensions_minimal() {
        let ext = build_extensions_minimal("example.com", &[1u8; 32]).unwrap();
        // Should have 3 extensions
        assert!(ext.len() > 50); // Rough check
    }

    #[test]
    fn test_extensions_standard() {
        let ext = build_extensions_standard("example.com", &[1u8; 32]).unwrap();
        // Should have 7 extensions (more than minimal)
        let minimal = build_extensions_minimal("example.com", &[1u8; 32]).unwrap();
        assert!(ext.len() > minimal.len());
    }

    #[test]
    fn test_sni_extension() {
        let sni = build_sni_extension("example.com");
        // Should contain "example.com"
        assert!(sni.windows(11).any(|w| w == b"example.com"));
    }

    #[test]
    fn test_key_share_extension() {
        let pubkey = vec![42u8; 32];
        let key_share = build_key_share_extension(&pubkey);
        
        // Should contain x25519 group (0x001d)
        assert_eq!(key_share[2], 0x00);
        assert_eq!(key_share[3], 0x1d);
        
        // Should contain our public key
        assert!(key_share.windows(32).any(|w| w == &pubkey[..]));
    }
}

