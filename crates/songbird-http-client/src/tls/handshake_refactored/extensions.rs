// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 extension builders
//!
//! Implements strategy-based extension building for TLS `ClientHello` messages.
//! Four strategies are supported:
//! - **Minimal**: Only required extensions (fastest)
//! - **Standard**: Balanced set (production default)
//! - **Modern**: Latest features (OCSP stapling)
//! - **`MaxCompatibility`**: Maximum compatibility (session tickets)

use super::core::TlsHandshake;
use super::tls_wire_u16;
use crate::error::Result;
use crate::tls::TLS_1_3;
use tracing::debug;

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
                // Custom strategy currently maps to the standard extension set (no per-policy builder).
                self.build_extensions_standard(server_name, public_key)
            }
        }
    }

    /// Build minimal extensions (only required, ~60ms handshake)
    /// Best for testing and minimal attack surface
    ///
    /// RFC 8446 Section 4.2: Minimum required extensions for TLS 1.3:
    /// - `supported_versions` (tells server we want TLS 1.3)
    /// - `supported_groups` (which curves we support)
    /// - `signature_algorithms` (which signatures we accept)
    /// - `key_share` (our X25519 public key)
    fn build_extensions_minimal(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. SNI extension (0x0000) - REQUIRED for virtual hosting
        ext.extend_from_slice(&[0x00, 0x00]);
        let sni_data = self.build_sni_extension(server_name)?;
        ext.extend_from_slice(&tls_wire_u16(sni_data.len())?.to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // 2. Supported Groups (0x000a) - MUST come before key_share per RFC 8446
        ext.extend_from_slice(&[0x00, 0x0a]);
        ext.extend_from_slice(&[0x00, 0x06]); // Extension length
        ext.extend_from_slice(&[0x00, 0x04]); // Named group list length
        ext.extend_from_slice(&[0x00, 0x1d]); // x25519
        ext.extend_from_slice(&[0x00, 0x17]); // secp256r1 (P-256 fallback)

        // 3. Signature algorithms (0x000d) - REQUIRED for TLS 1.3
        // Compact set with PSS variants required by CDN cert chains
        ext.extend_from_slice(&[0x00, 0x0d]);
        ext.extend_from_slice(&[0x00, 0x0e]); // Extension length (6 algorithms * 2 + 2 = 14)
        ext.extend_from_slice(&[0x00, 0x0c]); // List length (6 algorithms * 2 = 12)
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256
        ext.extend_from_slice(&[0x08, 0x05]); // rsa_pss_rsae_sha384
        ext.extend_from_slice(&[0x08, 0x06]); // rsa_pss_rsae_sha512
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384

        // 4. Supported versions (0x002b) - REQUIRED for TLS 1.3
        ext.extend_from_slice(&[0x00, 0x2b]);
        ext.extend_from_slice(&[0x00, 0x03]);
        ext.extend_from_slice(&[0x02]);
        ext.extend_from_slice(&TLS_1_3.to_be_bytes()); // Only TLS 1.3!

        // 5. Key share (0x0033) - REQUIRED for TLS 1.3 fresh handshake
        ext.extend_from_slice(&[0x00, 0x33]);
        let key_share_data = self.build_key_share_extension(public_key)?;
        ext.extend_from_slice(
            &u16::try_from(key_share_data.len()).expect("key share fits in u16").to_be_bytes(),
        );
        ext.extend_from_slice(&key_share_data);

        Ok(ext)
    }

    /// Build standard extensions (balanced, ~80ms handshake)
    /// Current production-tested set for fresh TLS 1.3 handshakes
    ///
    /// RFC 8446 Section 4.2: Extension ordering and content matters!
    /// - `key_share` MUST come BEFORE psk (if present)
    /// - We do NOT include `pre_shared_key` since we're not resuming
    fn build_extensions_standard(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. SNI extension (0x0000) - REQUIRED for virtual hosting
        ext.extend_from_slice(&[0x00, 0x00]);
        let sni_data = self.build_sni_extension(server_name)?;
        ext.extend_from_slice(&tls_wire_u16(sni_data.len())?.to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // 2. Supported Groups (0x000a) - MUST come before key_share per RFC 8446
        ext.extend_from_slice(&[0x00, 0x0a]);
        ext.extend_from_slice(&[0x00, 0x06]); // Extension length
        ext.extend_from_slice(&[0x00, 0x04]); // Named group list length
        ext.extend_from_slice(&[0x00, 0x1d]); // x25519
        ext.extend_from_slice(&[0x00, 0x17]); // secp256r1 (P-256 fallback)

        // 3. Signature algorithms (0x000d) - REQUIRED for TLS 1.3
        // Full set including RSA-PSS variants required by Cloudflare/CDN cert chains
        ext.extend_from_slice(&[0x00, 0x0d]);
        ext.extend_from_slice(&[0x00, 0x1e]); // Extension length (14 algorithms * 2 + 2 = 30)
        ext.extend_from_slice(&[0x00, 0x1c]); // List length (14 algorithms * 2 = 28)
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x06, 0x03]); // ecdsa_secp521r1_sha512
        ext.extend_from_slice(&[0x08, 0x07]); // ed25519
        ext.extend_from_slice(&[0x08, 0x08]); // ed448
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256
        ext.extend_from_slice(&[0x08, 0x05]); // rsa_pss_rsae_sha384
        ext.extend_from_slice(&[0x08, 0x06]); // rsa_pss_rsae_sha512
        ext.extend_from_slice(&[0x08, 0x09]); // rsa_pss_pss_sha256
        ext.extend_from_slice(&[0x08, 0x0a]); // rsa_pss_pss_sha384
        ext.extend_from_slice(&[0x08, 0x0b]); // rsa_pss_pss_sha512
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
        ext.extend_from_slice(&[0x06, 0x01]); // rsa_pkcs1_sha512

        // 4. Supported versions (0x002b) - REQUIRED for TLS 1.3
        // RFC 8446: This is how servers know we want TLS 1.3
        ext.extend_from_slice(&[0x00, 0x2b]);
        ext.extend_from_slice(&[0x00, 0x03]); // Extension length
        ext.extend_from_slice(&[0x02]); // Versions list length
        ext.extend_from_slice(&TLS_1_3.to_be_bytes()); // Only TLS 1.3!

        // 5. Key share (0x0033) - REQUIRED for TLS 1.3 fresh handshake
        ext.extend_from_slice(&[0x00, 0x33]);
        let key_share_data = self.build_key_share_extension(public_key)?;
        ext.extend_from_slice(
            &u16::try_from(key_share_data.len()).expect("key share fits in u16").to_be_bytes(),
        );
        ext.extend_from_slice(&key_share_data);

        // 6. ALPN extension (0x0010) - CRITICAL for HTTPS
        ext.extend_from_slice(&[0x00, 0x10]);
        ext.extend_from_slice(&[0x00, 0x0b]); // Extension length
        ext.extend_from_slice(&[0x00, 0x09]); // Protocol list length
        ext.extend_from_slice(&[0x08]); // Protocol name length
        ext.extend_from_slice(b"http/1.1");

        // NOTE: We deliberately OMIT these extensions for fresh handshakes:
        // - psk_key_exchange_modes (0x002d): Only needed if we include pre_shared_key
        // - pre_shared_key (0x0029): We're not resuming, so no PSK
        // - session_ticket (0x0023): We're not resuming, so no ticket
        //
        // Including psk_key_exchange_modes WITHOUT pre_shared_key can confuse some
        // servers into thinking we want to resume (causing Application Data 0x17
        // response instead of ServerHello 0x16).

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

        // 12. Supported Signature Algorithms Cert (0x0032) — full set for CDN cert chains
        ext.extend_from_slice(&[0x00, 0x32]);
        ext.extend_from_slice(&[0x00, 0x12]); // Extension length (8 algorithms * 2 + 2 = 18)
        ext.extend_from_slice(&[0x00, 0x10]); // List length (8 algorithms * 2 = 16)
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256
        ext.extend_from_slice(&[0x08, 0x05]); // rsa_pss_rsae_sha384
        ext.extend_from_slice(&[0x08, 0x06]); // rsa_pss_rsae_sha512
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
        ext.extend_from_slice(&[0x06, 0x01]); // rsa_pkcs1_sha512

        Ok(ext)
    }

    /// Build SNI extension
    #[allow(clippy::unused_self, reason = "API consistency with other TlsHandshake methods")]
    pub(crate) fn build_sni_extension(&self, server_name: &str) -> Result<Vec<u8>> {
        let mut sni = Vec::new();
        let name_bytes = server_name.as_bytes();

        sni.extend_from_slice(
            &tls_wire_u16(name_bytes.len().checked_add(3).ok_or_else(|| {
                crate::error::Error::TlsHandshake("SNI hostname length overflow".into())
            })?)?
            .to_be_bytes(),
        ); // List length
        sni.push(0x00); // Type: host_name
        sni.extend_from_slice(&tls_wire_u16(name_bytes.len())?.to_be_bytes());
        sni.extend_from_slice(name_bytes);

        Ok(sni)
    }

    /// Build key share extension
    #[allow(clippy::unused_self, reason = "API consistency with other TlsHandshake methods")]
    pub(crate) fn build_key_share_extension(&self, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ks = Vec::new();

        ks.extend_from_slice(
            &tls_wire_u16(public_key.len().checked_add(4).ok_or_else(|| {
                crate::error::Error::TlsHandshake("key share length overflow".into())
            })?)?
            .to_be_bytes(),
        ); // Client shares length
        ks.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        ks.extend_from_slice(&tls_wire_u16(public_key.len())?.to_be_bytes());
        ks.extend_from_slice(public_key);

        Ok(ks)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::tls::config::{ExtensionStrategy, TlsConfig};

    #[test]
    fn test_build_sni_extension() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let handshake = TlsHandshake::new(crypto);

        let sni = handshake.build_sni_extension("api.github.com").expect("sni");

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
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let handshake = TlsHandshake::new(crypto);

        let public_key = vec![0x42; 32]; // Mock 32-byte public key
        let ks = handshake.build_key_share_extension(&public_key).expect("key share");

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
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Minimal,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(crypto, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).expect("extensions");

        // Verify it's not empty
        assert!(!ext.is_empty());
        // Should contain at least 3 extensions (SNI, versions, key_share)
        // Each extension has: type (2 bytes) + length (2 bytes) + data
        assert!(ext.len() > 20); // Rough minimum
    }

    #[test]
    fn test_build_extensions_standard() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(crypto, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).expect("extensions");

        // Standard should be larger than minimal (includes ALPN, groups, sig algs, etc.)
        assert!(ext.len() > 50);
    }

    #[test]
    fn test_build_extensions_modern() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Modern,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(crypto, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).expect("extensions");

        // Modern should be larger than standard (includes OCSP stapling)
        assert!(ext.len() > 60);
    }

    #[test]
    fn test_build_extensions_maxcompat() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(crypto, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).expect("extensions");

        // MaxCompatibility should be largest (includes session ticket, cert sig algs)
        assert!(ext.len() > 80);
    }

    #[test]
    fn test_alpn_extension_encoding() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let handshake = TlsHandshake::with_config(crypto, config, None);

        let public_key = vec![0x42; 32];
        let ext = handshake.build_extensions("example.com", &public_key).expect("extensions");

        // Standard extensions include ALPN with "http/1.1"
        // Search for ALPN extension: 0x0010 followed by "http/1.1"
        let ext_str = format!("{ext:02x?}");
        // ALPN extension type is 0x0010
        assert!(ext_str.contains("00, 10"), "Should contain ALPN extension type 0x0010");
    }

    #[test]
    fn test_extension_strategy_differences() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let public_key = vec![0x42; 32];

        // Build with each strategy
        let minimal_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Minimal,
            ..Default::default()
        };
        let minimal = TlsHandshake::with_config(crypto.clone(), minimal_config, None)
            .build_extensions("example.com", &public_key)
            .expect("extensions");

        let standard_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let standard = TlsHandshake::with_config(crypto.clone(), standard_config, None)
            .build_extensions("example.com", &public_key)
            .expect("extensions");

        let modern_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Modern,
            ..Default::default()
        };
        let modern = TlsHandshake::with_config(crypto.clone(), modern_config, None)
            .build_extensions("example.com", &public_key)
            .expect("extensions");

        let maxcompat_config = TlsConfig {
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            ..Default::default()
        };
        let maxcompat = TlsHandshake::with_config(crypto, maxcompat_config, None)
            .build_extensions("example.com", &public_key)
            .expect("extensions");

        // Verify size ordering: minimal < standard < modern < maxcompat
        assert!(minimal.len() < standard.len());
        assert!(standard.len() < modern.len());
        assert!(modern.len() < maxcompat.len());
    }

    #[test]
    fn test_build_extensions_adaptive_maps_to_standard() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Adaptive,
            ..Default::default()
        };
        let adaptive_hs = TlsHandshake::with_config(crypto.clone(), config, None);

        let standard_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let standard_hs = TlsHandshake::with_config(crypto, standard_config, None);

        let key = vec![0x42; 32];
        let adaptive = adaptive_hs.build_extensions("example.com", &key).expect("extensions");
        let standard = standard_hs.build_extensions("example.com", &key).expect("extensions");
        assert_eq!(adaptive, standard);
    }

    #[test]
    fn test_build_extensions_custom_maps_to_standard() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let config = TlsConfig {
            extension_strategy: ExtensionStrategy::Custom(vec![0x000a, 0x002b]),
            ..Default::default()
        };
        let custom_hs = TlsHandshake::with_config(crypto.clone(), config, None);

        let standard_config = TlsConfig {
            extension_strategy: ExtensionStrategy::Standard,
            ..Default::default()
        };
        let standard_hs = TlsHandshake::with_config(crypto, standard_config, None);

        let key = vec![0x42; 32];
        let custom = custom_hs.build_extensions("example.com", &key).expect("extensions");
        let standard = standard_hs.build_extensions("example.com", &key).expect("extensions");
        assert_eq!(custom, standard);
    }
}
