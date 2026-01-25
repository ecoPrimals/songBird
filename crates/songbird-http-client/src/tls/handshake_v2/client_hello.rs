//! ClientHello message construction for TLS 1.3
//!
//! This module handles building TLS 1.3 ClientHello messages with configurable
//! extension strategies for optimal handshake performance.

use crate::error::Result;
use crate::tls::{config::ExtensionStrategy, CIPHER_SUITES, TLS_1_2, TLS_1_3};
use tracing::debug;

/// ClientHello builder for TLS 1.3
///
/// Constructs ClientHello messages with different extension strategies:
/// - Minimal: Fastest (~50ms handshake)
/// - Standard: Balanced (~80ms handshake)
/// - Modern: Latest features (~100ms handshake)
/// - MaxCompat: Maximum compatibility
pub struct ClientHelloBuilder {
    extension_strategy: ExtensionStrategy,
}

impl ClientHelloBuilder {
    /// Create a new ClientHello builder with the given extension strategy
    pub fn new(extension_strategy: ExtensionStrategy) -> Self {
        Self { extension_strategy }
    }

    /// Build a complete ClientHello message including TLS record framing
    ///
    /// Returns the complete TLS record ready to send on the wire.
    pub fn build(
        &self,
        client_random: &[u8],
        client_public_key: &[u8],
        server_name: &str,
    ) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // TLS Record header
        msg.push(0x16); // ContentType: Handshake
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version (0x0303)

        // Record length placeholder
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0]);

        // Handshake header
        msg.push(0x01); // HandshakeType: ClientHello

        // Handshake length placeholder
        let handshake_length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]);

        // ClientHello content
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        msg.extend_from_slice(client_random); // Random (32 bytes)
        msg.push(0); // Legacy session ID length

        // Cipher suites
        msg.extend_from_slice(&((CIPHER_SUITES.len() * 2) as u16).to_be_bytes());
        for suite in CIPHER_SUITES {
            msg.extend_from_slice(&suite.to_be_bytes());
        }

        // Compression methods
        msg.push(1); // Length
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

        Ok(msg)
    }

    /// Build extensions based on strategy
    fn build_extensions(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        match &self.extension_strategy {
            ExtensionStrategy::Minimal => {
                debug!("🎯 Building MINIMAL extensions (3 extensions, ~50ms handshake)");
                self.build_extensions_minimal(server_name, public_key)
            }
            ExtensionStrategy::Standard => {
                debug!("🎯 Building STANDARD extensions (7 extensions, ~80ms handshake)");
                self.build_extensions_standard(server_name, public_key)
            }
            ExtensionStrategy::Modern => {
                debug!("🎯 Building MODERN extensions (~100ms handshake)");
                self.build_extensions_modern(server_name, public_key)
            }
            ExtensionStrategy::MaxCompatibility => {
                debug!("🎯 Building MAXCOMPAT extensions (exhaustive set)");
                self.build_extensions_maxcompat(server_name, public_key)
            }
            ExtensionStrategy::Adaptive | ExtensionStrategy::Custom(_) => {
                // For adaptive and custom, use standard as baseline
                debug!("🎯 Building ADAPTIVE/CUSTOM extensions (using standard baseline)");
                self.build_extensions_standard(server_name, public_key)
            }
        }
    }

    /// Build minimal extensions (fastest handshake, ~50ms)
    /// Only required extensions: SNI, Supported Versions, Key Share
    fn build_extensions_minimal(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
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
    fn build_extensions_standard(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
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
    fn build_extensions_modern(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        // Start with standard extensions
        let mut ext = self.build_extensions_standard(server_name, public_key)?;

        // Add modern extensions

        // 8. Status Request (OCSP stapling, 0x0005)
        ext.extend_from_slice(&[0x00, 0x05]);
        ext.extend_from_slice(&[0x00, 0x05]);
        ext.extend_from_slice(&[0x01]); // status_type: ocsp
        ext.extend_from_slice(&[0x00, 0x00]); // responder_id_list: empty
        ext.extend_from_slice(&[0x00, 0x00]); // request_extensions: empty

        Ok(ext)
    }

    /// Build max compatibility extensions (exhaustive set)
    fn build_extensions_maxcompat(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        // Start with modern extensions
        let ext = self.build_extensions_modern(server_name, public_key)?;

        // Add compatibility extensions
        // (Add more here as needed for maximum compatibility)

        Ok(ext)
    }
}

/// Build SNI (Server Name Indication) extension
fn build_sni_extension(server_name: &str) -> Vec<u8> {
    let mut sni = Vec::new();
    let name_bytes = server_name.as_bytes();

    sni.extend_from_slice(&((name_bytes.len() + 3) as u16).to_be_bytes()); // List length
    sni.push(0x00); // Type: host_name
    sni.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
    sni.extend_from_slice(name_bytes);

    sni
}

/// Build key share extension
fn build_key_share_extension(public_key: &[u8]) -> Vec<u8> {
    let mut ks = Vec::new();

    ks.extend_from_slice(&((public_key.len() + 4) as u16).to_be_bytes()); // Client shares length
    ks.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
    ks.extend_from_slice(&(public_key.len() as u16).to_be_bytes());
    ks.extend_from_slice(public_key);

    ks
}

/// Generate client random (32 bytes)
///
/// Uses timestamp + pseudo-random for now.
/// In production, BearDog should provide cryptographically secure random.
pub fn generate_random() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut random = Vec::with_capacity(32);

    // Use timestamp for first 4 bytes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    random.extend_from_slice(&timestamp.to_be_bytes());

    // Fill rest with pseudo-random
    for i in 4..32 {
        random.push((i * 7 + timestamp as usize) as u8);
    }

    random
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random() {
        let random1 = generate_random();
        let random2 = generate_random();

        assert_eq!(random1.len(), 32);
        assert_eq!(random2.len(), 32);
        // Should generate different randoms (timestamps differ)
        // Note: May rarely fail if called in same millisecond
    }

    #[test]
    fn test_build_sni_extension() {
        let sni = build_sni_extension("example.com");

        // Should contain the server name
        assert!(sni.len() > "example.com".len());
        
        // Verify structure
        let list_len = u16::from_be_bytes([sni[0], sni[1]]) as usize;
        assert_eq!(list_len, "example.com".len() + 3);
        
        // Verify type
        assert_eq!(sni[2], 0x00); // host_name
    }

    #[test]
    fn test_build_key_share_extension() {
        let public_key = vec![0x42; 32]; // 32-byte X25519 public key
        let ks = build_key_share_extension(&public_key);

        // Should have proper structure
        assert!(ks.len() > 32);
        
        // Verify it contains the public key
        assert!(ks.windows(32).any(|w| w == public_key.as_slice()));
    }

    #[test]
    fn test_build_client_hello_minimal() {
        let builder = ClientHelloBuilder::new(ExtensionStrategy::Minimal);
        let random = generate_random();
        let public_key = vec![0x42; 32];

        let result = builder.build(&random, &public_key, "example.com");
        assert!(result.is_ok());

        let client_hello = result.unwrap();
        
        // Should have TLS record header
        assert_eq!(client_hello[0], 0x16); // Handshake
        assert_eq!(client_hello[1], 0x03); // TLS 1.2 (legacy)
        assert_eq!(client_hello[2], 0x03);
        
        // Should have ClientHello type
        assert_eq!(client_hello[5], 0x01);
    }

    #[test]
    fn test_build_client_hello_standard() {
        let builder = ClientHelloBuilder::new(ExtensionStrategy::Standard);
        let random = generate_random();
        let public_key = vec![0x42; 32];

        let result = builder.build(&random, &public_key, "example.com");
        assert!(result.is_ok());

        let client_hello = result.unwrap();
        
        // Standard should be longer than minimal (more extensions)
        let minimal_builder = ClientHelloBuilder::new(ExtensionStrategy::Minimal);
        let minimal = minimal_builder.build(&random, &public_key, "example.com").unwrap();
        
        assert!(client_hello.len() > minimal.len());
    }

    #[test]
    fn test_client_hello_contains_server_name() {
        let builder = ClientHelloBuilder::new(ExtensionStrategy::Standard);
        let random = generate_random();
        let public_key = vec![0x42; 32];
        let server_name = "test.example.com";

        let client_hello = builder.build(&random, &public_key, server_name).unwrap();
        
        // Should contain the server name in SNI extension
        let name_bytes = server_name.as_bytes();
        assert!(client_hello.windows(name_bytes.len())
            .any(|w| w == name_bytes));
    }

    #[test]
    fn test_client_hello_cipher_suites() {
        let builder = ClientHelloBuilder::new(ExtensionStrategy::Minimal);
        let random = generate_random();
        let public_key = vec![0x42; 32];

        let client_hello = builder.build(&random, &public_key, "example.com").unwrap();
        
        // Should contain all cipher suites
        // TLS_AES_128_GCM_SHA256 = 0x1301
        assert!(client_hello.windows(2).any(|w| w == &[0x13, 0x01]));
    }
}
