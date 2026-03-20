// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Configuration System
//!
//! Provides agnostic, adaptive TLS configuration that learns and evolves.
//! Eliminates hardcoded values in favor of strategy-based, server-adaptive behavior.
//!
//! ## TLS Version Support
//!
//! - TLS 1.3: Full support (default, preferred)
//! - TLS 1.2: Secure fallback (ECDHE + AEAD only)

use crate::tls::version::{SecurityPolicy, TlsVersion, TlsVersionConfig};
use std::time::Duration;

/// TLS connection configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// TLS version configuration (1.3 preferred, 1.2 secure fallback)
    pub version_config: TlsVersionConfig,

    /// Extension negotiation strategy
    pub extension_strategy: ExtensionStrategy,

    /// Cipher suite selection strategy
    pub cipher_strategy: CipherStrategy,

    /// Connection timeout
    pub timeout: Duration,

    /// Enable server profiling (learn from successes/failures)
    pub enable_profiling: bool,

    /// Maximum retry attempts for failed connections
    pub max_retries: u32,

    /// Fallback behavior on connection failure
    pub fallback_strategy: FallbackStrategy,

    /// Maximum response size (bytes)
    pub max_response_size: usize,

    /// Maximum TLS records to read
    pub max_records: usize,

    /// Enable adaptive learning
    pub enable_adaptive_learning: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            version_config: TlsVersionConfig::balanced(), // TLS 1.3 + secure 1.2 fallback
            extension_strategy: ExtensionStrategy::Adaptive,
            cipher_strategy: CipherStrategy::PreferModern,
            timeout: Duration::from_secs(30),
            enable_profiling: true,
            max_retries: 3,
            fallback_strategy: FallbackStrategy::Progressive,
            max_response_size: 10_000_000, // 10 MB
            max_records: 100,
            enable_adaptive_learning: true,
        }
    }
}

/// Strategy for TLS extension negotiation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionStrategy {
    /// Minimal: Only required extensions (SNI, Supported Versions, Key Share)
    Minimal,

    /// Standard: Common extensions for most servers
    Standard,

    /// Modern: Latest TLS 1.3 features (includes PSK, 0-RTT hints)
    Modern,

    /// `MaxCompatibility`: All possible extensions for maximum compatibility
    MaxCompatibility,

    /// Adaptive: Learn from server responses, start with Standard
    Adaptive,

    /// Custom: User-defined extension set
    Custom(Vec<u16>), // Extension type codes
}

/// Strategy for cipher suite selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CipherStrategy {
    /// Prefer modern ciphers (`ChaCha20` > AES-256-GCM > AES-128-GCM)
    PreferModern,

    /// Prefer compatibility (AES-128-GCM > AES-256-GCM > `ChaCha20`)
    PreferCompatibility,

    /// Only AES (for hardware-accelerated environments)
    OnlyAes,

    /// Only `ChaCha20` (for software-only environments)
    OnlyChaCha,

    /// Adaptive: Learn which cipher suites work best
    Adaptive,

    /// Custom: User-defined cipher suite order
    Custom(Vec<u16>), // Cipher suite codes
}

/// Fallback strategy on connection failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStrategy {
    /// No fallback, fail immediately
    None,

    /// Progressive: Try Modern → Standard → Minimal
    Progressive,

    /// Reverse: Try Minimal → Standard → Modern
    Reverse,

    /// `MaxCompatibility`: Try all possible combinations
    Exhaustive,
}

impl TlsConfig {
    /// Create minimal config (fewest extensions, fastest handshake)
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            extension_strategy: ExtensionStrategy::Minimal,
            cipher_strategy: CipherStrategy::PreferCompatibility,
            enable_profiling: false,
            enable_adaptive_learning: false,
            ..Default::default()
        }
    }

    /// Create standard config (balanced, good default)
    #[must_use]
    pub fn standard() -> Self {
        Self {
            extension_strategy: ExtensionStrategy::Standard,
            cipher_strategy: CipherStrategy::PreferCompatibility,
            enable_profiling: true,
            enable_adaptive_learning: false,
            ..Default::default()
        }
    }

    /// Create modern config (latest features, optimal performance)
    #[must_use]
    pub fn modern() -> Self {
        Self {
            extension_strategy: ExtensionStrategy::Modern,
            cipher_strategy: CipherStrategy::PreferModern,
            enable_profiling: true,
            enable_adaptive_learning: true,
            ..Default::default()
        }
    }

    /// Create max compatibility config (works everywhere)
    #[must_use]
    pub fn max_compatibility() -> Self {
        Self {
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            cipher_strategy: CipherStrategy::PreferCompatibility,
            max_retries: 5,
            fallback_strategy: FallbackStrategy::Exhaustive,
            enable_profiling: true,
            enable_adaptive_learning: true,
            ..Default::default()
        }
    }

    /// Create adaptive config (learns and evolves)
    #[must_use]
    pub fn adaptive() -> Self {
        Self {
            extension_strategy: ExtensionStrategy::Adaptive,
            cipher_strategy: CipherStrategy::Adaptive,
            enable_profiling: true,
            enable_adaptive_learning: true,
            max_retries: 3,
            fallback_strategy: FallbackStrategy::Progressive,
            ..Default::default()
        }
    }

    /// Create TLS 1.3 only config (maximum security, may fail on legacy servers)
    #[must_use]
    pub fn tls_1_3_only() -> Self {
        Self {
            version_config: TlsVersionConfig::strict(),
            extension_strategy: ExtensionStrategy::Modern,
            cipher_strategy: CipherStrategy::PreferModern,
            enable_profiling: true,
            enable_adaptive_learning: true,
            ..Default::default()
        }
    }

    /// Create legacy-compatible config (TLS 1.3 + secure 1.2 fallback)
    #[must_use]
    pub fn legacy_compatible() -> Self {
        Self {
            version_config: TlsVersionConfig::legacy(),
            extension_strategy: ExtensionStrategy::MaxCompatibility,
            cipher_strategy: CipherStrategy::PreferCompatibility,
            max_retries: 5,
            fallback_strategy: FallbackStrategy::Exhaustive,
            enable_profiling: true,
            enable_adaptive_learning: true,
            ..Default::default()
        }
    }

    // Builder methods

    /// Set TLS version configuration
    #[must_use]
    pub const fn with_version_config(mut self, config: TlsVersionConfig) -> Self {
        self.version_config = config;
        self
    }

    /// Set security policy (controls TLS version selection)
    #[must_use]
    pub const fn with_security_policy(mut self, policy: SecurityPolicy) -> Self {
        self.version_config = self.version_config.with_policy(policy);
        self
    }

    /// Set minimum TLS version
    #[must_use]
    pub const fn with_minimum_version(mut self, version: TlsVersion) -> Self {
        self.version_config = self.version_config.with_minimum_version(version);
        self
    }
}

/// Extension set for a given strategy
#[derive(Debug, Clone)]
pub struct ExtensionSet {
    /// Extension types to include
    pub extensions: Vec<ExtensionType>,

    /// Description of this set
    pub description: &'static str,
}

/// TLS extension types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    /// Server Name Indication (0x0000)
    Sni = 0x0000,

    /// Application-Layer Protocol Negotiation (0x0010)
    Alpn = 0x0010,

    /// Supported Versions (0x002b)
    SupportedVersions = 0x002b,

    /// Key Share (0x0033)
    KeyShare = 0x0033,

    /// Supported Groups (0x000a)
    SupportedGroups = 0x000a,

    /// Signature Algorithms (0x000d)
    SignatureAlgorithms = 0x000d,

    /// PSK Key Exchange Modes (0x002d)
    PskKeyExchangeModes = 0x002d,

    /// Session Ticket (0x0023)
    SessionTicket = 0x0023,

    /// Status Request / OCSP (0x0005)
    StatusRequest = 0x0005,

    /// SCT / Certificate Transparency (0x0012)
    Sct = 0x0012,

    /// Compress Certificate (0x001b)
    CompressCertificate = 0x001b,

    /// Record Size Limit (0x001c)
    RecordSizeLimit = 0x001c,
}

impl ExtensionSet {
    /// Minimal extension set (required only)
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            extensions: vec![
                ExtensionType::Sni,
                ExtensionType::SupportedVersions,
                ExtensionType::KeyShare,
            ],
            description: "Minimal (required only)",
        }
    }

    /// Standard extension set (works with most servers)
    #[must_use]
    pub fn standard() -> Self {
        Self {
            extensions: vec![
                ExtensionType::Sni,
                ExtensionType::Alpn,
                ExtensionType::SupportedVersions,
                ExtensionType::KeyShare,
                ExtensionType::SupportedGroups,
                ExtensionType::SignatureAlgorithms,
                ExtensionType::PskKeyExchangeModes,
            ],
            description: "Standard (TLS 1.3 common)",
        }
    }

    /// Modern extension set (latest features)
    #[must_use]
    pub fn modern() -> Self {
        Self {
            extensions: vec![
                ExtensionType::Sni,
                ExtensionType::Alpn,
                ExtensionType::SupportedVersions,
                ExtensionType::KeyShare,
                ExtensionType::SupportedGroups,
                ExtensionType::SignatureAlgorithms,
                ExtensionType::PskKeyExchangeModes,
                ExtensionType::SessionTicket,
                ExtensionType::StatusRequest,
                ExtensionType::RecordSizeLimit,
            ],
            description: "Modern (with resumption & OCSP)",
        }
    }

    /// Maximum compatibility set (all possible extensions)
    #[must_use]
    pub fn max_compatibility() -> Self {
        Self {
            extensions: vec![
                ExtensionType::Sni,
                ExtensionType::Alpn,
                ExtensionType::SupportedVersions,
                ExtensionType::KeyShare,
                ExtensionType::SupportedGroups,
                ExtensionType::SignatureAlgorithms,
                ExtensionType::PskKeyExchangeModes,
                ExtensionType::SessionTicket,
                ExtensionType::StatusRequest,
                ExtensionType::Sct,
                ExtensionType::CompressCertificate,
                ExtensionType::RecordSizeLimit,
            ],
            description: "Max Compatibility (all extensions)",
        }
    }
}

/// Cipher suite set for a given strategy
#[derive(Debug, Clone)]
pub struct CipherSuiteSet {
    /// Cipher suites in preference order
    pub suites: Vec<u16>,

    /// Description of this set
    pub description: &'static str,
}

impl CipherSuiteSet {
    /// Modern cipher preference (`ChaCha20` first)
    #[must_use]
    pub fn modern() -> Self {
        Self {
            suites: vec![
                0x1303, // TLS_CHACHA20_POLY1305_SHA256
                0x1302, // TLS_AES_256_GCM_SHA384
                0x1301, // TLS_AES_128_GCM_SHA256
            ],
            description: "Modern (ChaCha20 preferred)",
        }
    }

    /// Compatibility cipher preference (AES-128 first)
    #[must_use]
    pub fn compatibility() -> Self {
        Self {
            suites: vec![
                0x1301, // TLS_AES_128_GCM_SHA256
                0x1302, // TLS_AES_256_GCM_SHA384
                0x1303, // TLS_CHACHA20_POLY1305_SHA256
            ],
            description: "Compatibility (AES-128 preferred)",
        }
    }

    /// AES-only (hardware accelerated)
    #[must_use]
    pub fn aes_only() -> Self {
        Self {
            suites: vec![
                0x1301, // TLS_AES_128_GCM_SHA256
                0x1302, // TLS_AES_256_GCM_SHA384
            ],
            description: "AES-only (hardware accelerated)",
        }
    }

    /// ChaCha20-only (software optimized)
    #[must_use]
    pub fn chacha_only() -> Self {
        Self {
            suites: vec![
                0x1303, // TLS_CHACHA20_POLY1305_SHA256
            ],
            description: "ChaCha20-only (software optimized)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_presets() {
        let minimal = TlsConfig::minimal();
        assert_eq!(minimal.extension_strategy, ExtensionStrategy::Minimal);
        assert!(!minimal.enable_adaptive_learning);

        let modern = TlsConfig::modern();
        assert_eq!(modern.extension_strategy, ExtensionStrategy::Modern);
        assert!(modern.enable_adaptive_learning);

        let adaptive = TlsConfig::adaptive();
        assert_eq!(adaptive.extension_strategy, ExtensionStrategy::Adaptive);
        assert!(adaptive.enable_profiling);
    }

    #[test]
    fn test_extension_sets() {
        let minimal = ExtensionSet::minimal();
        assert_eq!(minimal.extensions.len(), 3); // SNI, Versions, KeyShare

        let standard = ExtensionSet::standard();
        assert_eq!(standard.extensions.len(), 7); // Our current implementation

        let modern = ExtensionSet::modern();
        assert!(modern.extensions.len() > 7); // Includes optional extensions

        let max = ExtensionSet::max_compatibility();
        assert!(max.extensions.len() >= 12); // All possible extensions
    }

    #[test]
    fn test_cipher_suite_sets() {
        let modern = CipherSuiteSet::modern();
        assert_eq!(modern.suites[0], 0x1303); // ChaCha20 first

        let compat = CipherSuiteSet::compatibility();
        assert_eq!(compat.suites[0], 0x1301); // AES-128 first

        let aes = CipherSuiteSet::aes_only();
        assert_eq!(aes.suites.len(), 2); // Only AES ciphers

        let chacha = CipherSuiteSet::chacha_only();
        assert_eq!(chacha.suites.len(), 1); // Only ChaCha20
    }

    #[test]
    fn test_fallback_strategies() {
        let config = TlsConfig::max_compatibility();
        assert_eq!(config.fallback_strategy, FallbackStrategy::Exhaustive);
        assert_eq!(config.max_retries, 5);
    }
}
