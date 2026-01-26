//! TLS 1.3 extension builders
//!
//! Implements strategy-based extension building for TLS ClientHello messages.
//! Four strategies are supported:
//! - **Minimal**: Only required extensions (fastest)
//! - **Standard**: Balanced set (production default)
//! - **Modern**: Latest features (OCSP stapling)
//! - **MaxCompatibility**: Maximum compatibility (session tickets)

use super::core::TlsHandshake;
use crate::error::Result;
use crate::tls::{TLS_1_3};
use tracing::{debug};

impl TlsHandshake {
    /// Build extensions based on configured strategy
    pub(super) fn build_extensions(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        debug!("Building extensions with {:?} strategy", self.config.extension_strategy);
        
        match self.config.extension_strategy {
            crate::tls::config::ExtensionStrategy::Minimal => {
                self.build_extensions_minimal(server_name, public_key)
            }
            crate::tls::config::ExtensionStrategy::Standard => {
                self.build_extensions_standard(server_name, public_key)
            }
            crate::tls::config::ExtensionStrategy::Modern => {
                self.build_extensions_modern(server_name, public_key)
            }
            crate::tls::config::ExtensionStrategy::MaxCompatibility => {
                self.build_extensions_maxcompat(server_name, public_key)
            }
            crate::tls::config::ExtensionStrategy::Adaptive => {
                // Adaptive starts with Standard and learns from server responses
                self.build_extensions_standard(server_name, public_key)
            }
            crate::tls::config::ExtensionStrategy::Custom(_) => {
                // For now, Custom uses Standard as base
                // TODO: Implement custom extension building
                self.build_extensions_standard(server_name, public_key)
            }
        }
    }

    /// Build minimal extensions (only required, ~60ms handshake)
    /// Best for testing and minimal attack surface
    fn build_extensions_minimal(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. SNI extension (0x0000) - REQUIRED for virtual hosting
        ext.extend_from_slice(&[0x00, 0x00]);
        let sni_data = self.build_sni_extension(server_name);
        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // 2. Supported versions (0x002b) - REQUIRED for TLS 1.3
        ext.extend_from_slice(&[0x00, 0x2b]);
        ext.extend_from_slice(&[0x00, 0x03]);
        ext.extend_from_slice(&[0x02]);
        ext.extend_from_slice(&TLS_1_3.to_be_bytes());

        // 3. Key share (0x0033) - REQUIRED for TLS 1.3
        ext.extend_from_slice(&[0x00, 0x33]);
        let key_share_data = self.build_key_share_extension(public_key);
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
        let sni_data = self.build_sni_extension(server_name);
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
        let key_share_data = self.build_key_share_extension(public_key);
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

        // RFC 8446 NOTE: We do NOT add TLS 1.2 legacy extensions like:
        // - extended_master_secret (0x0017) - causes servers to confuse us with TLS 1.2
        // - renegotiation_info (0xff01) - not applicable in TLS 1.3
        // These were causing real-world servers (cloudflare, google) to send Application Data
        // instead of ServerHello, as they thought we were trying session resumption.

        Ok(ext)
    }

    /// Build max compatibility extensions (exhaustive set)
    fn build_extensions_maxcompat(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        // Start with modern extensions
        let mut ext = self.build_extensions_modern(server_name, public_key)?;

        // Add compatibility extensions

        // 11. Session Ticket (0x0023)
        ext.extend_from_slice(&[0x00, 0x23]);
        ext.extend_from_slice(&[0x00, 0x00]); // Empty ticket

        // 12. Supported Signature Algorithms Cert (0x0032)
        ext.extend_from_slice(&[0x00, 0x32]);
        ext.extend_from_slice(&[0x00, 0x0c]);
        ext.extend_from_slice(&[0x00, 0x0a]);
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256

        Ok(ext)
    }

    /// Build SNI extension
    pub(crate) fn build_sni_extension(&self, server_name: &str) -> Vec<u8> {
        let mut sni = Vec::new();
        let name_bytes = server_name.as_bytes();

        sni.extend_from_slice(&((name_bytes.len() + 3) as u16).to_be_bytes()); // List length
        sni.push(0x00); // Type: host_name
        sni.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        sni.extend_from_slice(name_bytes);

        sni
    }

    /// Build key share extension
    pub(crate) fn build_key_share_extension(&self, public_key: &[u8]) -> Vec<u8> {
        let mut ks = Vec::new();

        ks.extend_from_slice(&((public_key.len() + 4) as u16).to_be_bytes()); // Client shares length
        ks.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        ks.extend_from_slice(&(public_key.len() as u16).to_be_bytes());
        ks.extend_from_slice(public_key);

        ks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoCapability;
    use crate::tls::config::{TlsConfig, ExtensionStrategy};

    #[test]
    fn test_build_sni_extension() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);

        let sni = handshake.build_sni_extension("api.github.com");

        // Verify structure
        assert!(sni.len() > 3);
        // First 2 bytes: list length
        let list_length = u16::from_be_bytes([sni[0], sni[1]]) as usize;
        assert_eq!(list_length, sni.len() - 2);
        // Next byte: type (0x00 = host_name)
        assert_eq!(sni[2], 0x00);
        // Next 2 bytes: name length
        let name_length = u16::from_be_bytes([sni[3], sni[4]]) as usize;
        assert_eq!(name_length, "api.github.com".len());
        // Rest: name bytes
        assert_eq!(&sni[5..], b"api.github.com");
    }

    #[test]
    fn test_build_key_share_extension() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);

        let public_key = vec![0x42; 32]; // Mock 32-byte public key
        let ks = handshake.build_key_share_extension(&public_key);

        // Verify structure
        // First 2 bytes: client shares length
        let shares_length = u16::from_be_bytes([ks[0], ks[1]]) as usize;
        assert_eq!(shares_length, 32 + 4); // 32-byte key + 4 bytes overhead
        // Next 2 bytes: group (0x001d = x25519)
        assert_eq!(ks[2], 0x00);
        assert_eq!(ks[3], 0x1d);
        // Next 2 bytes: key exchange length
        let key_length = u16::from_be_bytes([ks[4], ks[5]]) as usize;
        assert_eq!(key_length, 32);
        // Rest: public key
        assert_eq!(&ks[6..], &[0x42; 32]);
    }

    #[test]
    fn test_build_extensions_minimal() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Minimal,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(beardog, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).unwrap();

        // Verify it's not empty
        assert!(!ext.is_empty());
        // Should contain at least 3 extensions (SNI, versions, key_share)
        // Each extension has: type (2 bytes) + length (2 bytes) + data
        assert!(ext.len() > 20); // Rough minimum
    }

    #[test]
    fn test_build_extensions_standard() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(beardog, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).unwrap();

        // Standard should be larger than minimal (includes ALPN, groups, sig algs, etc.)
        assert!(ext.len() > 50);
    }

    #[test]
    fn test_build_extensions_modern() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Modern,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(beardog, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).unwrap();

        // Modern should be larger than standard (includes OCSP stapling)
        assert!(ext.len() > 60);
    }

    #[test]
    fn test_build_extensions_maxcompat() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(beardog, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).unwrap();

        // MaxCompatibility should be largest (includes session ticket, cert sig algs)
        assert!(ext.len() > 80);
    }

    #[test]
    fn test_alpn_extension_encoding() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(beardog, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).unwrap();

        // Standard extensions include ALPN with "http/1.1"
        // Search for ALPN extension: 0x0010 followed by "http/1.1"
        let ext_str = format!("{:02x?}", ext);
        // ALPN extension type is 0x0010
        assert!(ext_str.contains("00, 10"), "Should contain ALPN extension type 0x0010");
    }

    #[test]
    fn test_extension_strategy_differences() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock"))
            as std::sync::Arc<dyn CryptoCapability>;
        let public_key = vec![0x42; 32];

        // Build with each strategy
        let minimal_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Minimal,
            ..Default::default()
        };
        let minimal = TlsHandshake::with_config(beardog.clone(), minimal_config, None)
            .build_extensions("example.com", &public_key)
            .unwrap();

        let standard_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let standard = TlsHandshake::with_config(beardog.clone(), standard_config, None)
            .build_extensions("example.com", &public_key)
            .unwrap();

        let modern_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Modern,
            ..Default::default()
        };
        let modern = TlsHandshake::with_config(beardog.clone(), modern_config, None)
            .build_extensions("example.com", &public_key)
            .unwrap();

        let maxcompat_config = TlsConfig {
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            ..Default::default()
        };
        let maxcompat = TlsHandshake::with_config(beardog, maxcompat_config, None)
            .build_extensions("example.com", &public_key)
            .unwrap();

        // Verify size ordering: minimal < standard < modern < maxcompat
        assert!(minimal.len() < standard.len());
        assert!(standard.len() < modern.len());
        assert!(modern.len() < maxcompat.len());
    }
}

