//! TLS Version Configuration and Negotiation
//!
//! Supports TLS 1.3 (primary) with secure TLS 1.2 fallback for legacy servers.
//!
//! ## Security Policy
//!
//! TLS 1.2 fallback is RESTRICTED to secure configurations only:
//! - ECDHE key exchange only (forward secrecy required)
//! - AEAD ciphers only (GCM, ChaCha20-Poly1305)
//! - No CBC mode, no RSA key exchange, no weak ciphers
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     TLS Version Negotiation                         │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  TlsVersionConfig                                                   │
//! │  ├── preferred: TLS 1.3 (always tried first)                       │
//! │  ├── minimum: TLS 1.2 or 1.3 (configurable)                        │
//! │  └── security_policy: Strict, Balanced, Legacy                     │
//! │                                                                     │
//! │  Negotiation Flow:                                                  │
//! │  1. Send ClientHello with [1.3, 1.2] in supported_versions         │
//! │  2. Server responds with chosen version                            │
//! │  3. Songbird continues with appropriate handshake flow             │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use std::fmt;

/// TLS protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum TlsVersion {
    /// TLS 1.2 (0x0303) - Legacy support
    Tls12 = 0x0303,
    /// TLS 1.3 (0x0304) - Preferred
    Tls13 = 0x0304,
}

impl TlsVersion {
    /// Get the wire format bytes
    pub fn to_bytes(self) -> [u8; 2] {
        (self as u16).to_be_bytes()
    }

    /// Parse from wire format
    pub fn from_bytes(bytes: [u8; 2]) -> Option<Self> {
        match u16::from_be_bytes(bytes) {
            0x0303 => Some(TlsVersion::Tls12),
            0x0304 => Some(TlsVersion::Tls13),
            _ => None,
        }
    }

    /// Human-readable name
    pub fn name(self) -> &'static str {
        match self {
            TlsVersion::Tls12 => "TLS 1.2",
            TlsVersion::Tls13 => "TLS 1.3",
        }
    }
}

impl fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Security policy for TLS version negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityPolicy {
    /// TLS 1.3 only - Maximum security, may fail on legacy servers
    Strict,

    /// TLS 1.3 preferred, secure TLS 1.2 fallback (ECDHE+AEAD only)
    #[default]
    Balanced,

    /// TLS 1.3 preferred, broader TLS 1.2 support (still no weak ciphers)
    /// Use only when necessary for specific legacy systems
    Legacy,
}

impl SecurityPolicy {
    /// Get allowed TLS versions for this policy
    pub fn allowed_versions(self) -> Vec<TlsVersion> {
        match self {
            SecurityPolicy::Strict => vec![TlsVersion::Tls13],
            SecurityPolicy::Balanced | SecurityPolicy::Legacy => {
                vec![TlsVersion::Tls13, TlsVersion::Tls12]
            }
        }
    }

    /// Get allowed TLS 1.2 cipher suites for this policy
    pub fn tls_1_2_ciphers(self) -> &'static [u16] {
        match self {
            SecurityPolicy::Strict => &[],
            SecurityPolicy::Balanced => TLS_1_2_SECURE_CIPHERS,
            SecurityPolicy::Legacy => TLS_1_2_EXTENDED_CIPHERS,
        }
    }
}

/// TLS version configuration
#[derive(Debug, Clone)]
pub struct TlsVersionConfig {
    /// Security policy
    pub policy: SecurityPolicy,

    /// Minimum acceptable version
    pub minimum_version: TlsVersion,

    /// Enable version downgrade detection
    pub downgrade_protection: bool,

    /// Log warnings when falling back to 1.2
    pub warn_on_fallback: bool,
}

impl Default for TlsVersionConfig {
    fn default() -> Self {
        Self {
            policy: SecurityPolicy::Balanced,
            minimum_version: TlsVersion::Tls12,
            downgrade_protection: true,
            warn_on_fallback: true,
        }
    }
}

impl TlsVersionConfig {
    /// Create strict TLS 1.3 only configuration
    pub fn strict() -> Self {
        Self {
            policy: SecurityPolicy::Strict,
            minimum_version: TlsVersion::Tls13,
            downgrade_protection: true,
            warn_on_fallback: false,
        }
    }

    /// Create balanced configuration (default)
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Create legacy-compatible configuration
    pub fn legacy() -> Self {
        Self {
            policy: SecurityPolicy::Legacy,
            minimum_version: TlsVersion::Tls12,
            downgrade_protection: true,
            warn_on_fallback: true,
        }
    }

    /// Get supported versions in preference order
    pub fn supported_versions(&self) -> Vec<TlsVersion> {
        self.policy.allowed_versions().into_iter().filter(|v| *v >= self.minimum_version).collect()
    }

    /// Check if a version is acceptable
    pub fn is_acceptable(&self, version: TlsVersion) -> bool {
        version >= self.minimum_version && self.policy.allowed_versions().contains(&version)
    }

    /// Get TLS 1.2 cipher suites (empty if 1.3 only)
    pub fn tls_1_2_ciphers(&self) -> &'static [u16] {
        if self.minimum_version > TlsVersion::Tls12 {
            &[]
        } else {
            self.policy.tls_1_2_ciphers()
        }
    }

    /// Builder: set policy
    pub fn with_policy(mut self, policy: SecurityPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Builder: set minimum version
    pub fn with_minimum_version(mut self, version: TlsVersion) -> Self {
        self.minimum_version = version;
        self
    }

    /// Builder: set downgrade protection
    pub fn with_downgrade_protection(mut self, enabled: bool) -> Self {
        self.downgrade_protection = enabled;
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SECURE TLS 1.2 CIPHER SUITES
// ═══════════════════════════════════════════════════════════════════════════

/// Secure TLS 1.2 cipher suites (ECDHE + AEAD only)
///
/// These provide forward secrecy and authenticated encryption.
/// NO RSA key exchange, NO CBC mode.
pub const TLS_1_2_SECURE_CIPHERS: &[u16] = &[
    // ECDHE-RSA with AEAD
    0xC02F, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    0xC030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
    0xCCA8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
    // ECDHE-ECDSA with AEAD
    0xC02B, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
    0xC02C, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
    0xCCA9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
];

/// Extended TLS 1.2 cipher suites (for legacy compatibility)
///
/// Includes DHE ciphers for servers without ECDHE.
/// Still NO RSA key exchange, NO CBC mode.
pub const TLS_1_2_EXTENDED_CIPHERS: &[u16] = &[
    // ECDHE-RSA with AEAD (preferred)
    0xC02F, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    0xC030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
    0xCCA8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
    // ECDHE-ECDSA with AEAD
    0xC02B, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
    0xC02C, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
    0xCCA9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
    // DHE-RSA with AEAD (fallback for servers without ECDHE)
    0x009E, // TLS_DHE_RSA_WITH_AES_128_GCM_SHA256
    0x009F, // TLS_DHE_RSA_WITH_AES_256_GCM_SHA384
    0xCCAA, // TLS_DHE_RSA_WITH_CHACHA20_POLY1305_SHA256
];

/// Get human-readable name for TLS 1.2 cipher suite
pub fn tls_1_2_cipher_name(suite: u16) -> &'static str {
    match suite {
        0xC02F => "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        0xC030 => "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
        0xCCA8 => "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        0xC02B => "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
        0xC02C => "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
        0xCCA9 => "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
        0x009E => "TLS_DHE_RSA_WITH_AES_128_GCM_SHA256",
        0x009F => "TLS_DHE_RSA_WITH_AES_256_GCM_SHA384",
        0xCCAA => "TLS_DHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
        _ => "Unknown",
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TLS VERSION DETECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Negotiated TLS version result
#[derive(Debug, Clone)]
pub struct NegotiatedVersion {
    /// The negotiated version
    pub version: TlsVersion,
    /// The cipher suite (format depends on version)
    pub cipher_suite: u16,
    /// Whether downgrade was detected
    pub downgrade_detected: bool,
}

/// Detect TLS version from ServerHello
///
/// TLS 1.3 uses supported_versions extension for actual version.
/// TLS 1.2 uses legacy_version field directly.
pub fn detect_server_version(server_hello: &[u8]) -> Option<NegotiatedVersion> {
    if server_hello.len() < 38 {
        return None;
    }

    // Skip handshake header if present
    let data = if server_hello[0] == 0x02 {
        // Handshake message type 2 = ServerHello
        &server_hello[4..] // Skip type (1) + length (3)
    } else {
        server_hello
    };

    if data.len() < 34 {
        return None;
    }

    // Legacy version at offset 0
    let legacy_version = u16::from_be_bytes([data[0], data[1]]);

    // Skip to session_id_length (after random)
    let session_id_len = data[34] as usize;
    let mut offset = 35 + session_id_len;

    if offset + 2 > data.len() {
        return None;
    }

    // Cipher suite
    let cipher_suite = u16::from_be_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    // Skip compression method
    offset += 1;

    // Parse extensions to find supported_versions
    if offset + 2 > data.len() {
        // No extensions - use legacy version
        let version = TlsVersion::from_bytes([data[0], data[1]])?;
        return Some(NegotiatedVersion {
            version,
            cipher_suite,
            downgrade_detected: false,
        });
    }

    let extensions_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
    offset += 2;

    let extensions_end = offset + extensions_len;

    // Search for supported_versions extension (0x002b)
    while offset + 4 <= extensions_end {
        let ext_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let ext_len = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;
        offset += 4;

        if ext_type == 0x002b && ext_len >= 2 {
            // supported_versions extension found
            let actual_version = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let version = TlsVersion::from_bytes([data[offset], data[offset + 1]])?;

            // Check for downgrade
            let downgrade_detected = legacy_version == 0x0303 && actual_version == 0x0304;

            return Some(NegotiatedVersion {
                version,
                cipher_suite,
                downgrade_detected,
            });
        }

        offset += ext_len;
    }

    // No supported_versions - this is TLS 1.2 or earlier
    let version = TlsVersion::from_bytes(legacy_version.to_be_bytes())?;
    Some(NegotiatedVersion {
        version,
        cipher_suite,
        downgrade_detected: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_version_ordering() {
        assert!(TlsVersion::Tls13 > TlsVersion::Tls12);
    }

    #[test]
    fn test_tls_version_bytes() {
        assert_eq!(TlsVersion::Tls12.to_bytes(), [0x03, 0x03]);
        assert_eq!(TlsVersion::Tls13.to_bytes(), [0x03, 0x04]);
    }

    #[test]
    fn test_security_policy_versions() {
        let strict = SecurityPolicy::Strict;
        assert_eq!(strict.allowed_versions(), vec![TlsVersion::Tls13]);

        let balanced = SecurityPolicy::Balanced;
        assert_eq!(balanced.allowed_versions(), vec![TlsVersion::Tls13, TlsVersion::Tls12]);
    }

    #[test]
    fn test_security_policy_ciphers() {
        let strict = SecurityPolicy::Strict;
        assert!(strict.tls_1_2_ciphers().is_empty());

        let balanced = SecurityPolicy::Balanced;
        assert!(!balanced.tls_1_2_ciphers().is_empty());
        assert!(balanced.tls_1_2_ciphers().contains(&0xC02F));
    }

    #[test]
    fn test_version_config_strict() {
        let config = TlsVersionConfig::strict();
        assert_eq!(config.policy, SecurityPolicy::Strict);
        assert_eq!(config.minimum_version, TlsVersion::Tls13);
        assert!(!config.is_acceptable(TlsVersion::Tls12));
        assert!(config.is_acceptable(TlsVersion::Tls13));
    }

    #[test]
    fn test_version_config_balanced() {
        let config = TlsVersionConfig::balanced();
        assert!(config.is_acceptable(TlsVersion::Tls12));
        assert!(config.is_acceptable(TlsVersion::Tls13));
    }

    #[test]
    fn test_supported_versions() {
        let config = TlsVersionConfig::balanced();
        let versions = config.supported_versions();
        assert_eq!(versions, vec![TlsVersion::Tls13, TlsVersion::Tls12]);
    }

    #[test]
    fn test_tls_1_2_cipher_names() {
        assert_eq!(tls_1_2_cipher_name(0xC02F), "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256");
        assert_eq!(tls_1_2_cipher_name(0xCCA9), "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256");
    }

    #[test]
    fn test_no_weak_ciphers() {
        // Ensure our cipher lists don't contain weak ciphers
        let weak_ciphers = [
            0x0005, // TLS_RSA_WITH_RC4_128_SHA
            0x000A, // TLS_RSA_WITH_3DES_EDE_CBC_SHA
            0x002F, // TLS_RSA_WITH_AES_128_CBC_SHA
            0x0035, // TLS_RSA_WITH_AES_256_CBC_SHA
        ];

        for cipher in TLS_1_2_SECURE_CIPHERS {
            assert!(
                !weak_ciphers.contains(cipher),
                "Secure ciphers should not contain weak cipher 0x{:04x}",
                cipher
            );
        }

        for cipher in TLS_1_2_EXTENDED_CIPHERS {
            assert!(
                !weak_ciphers.contains(cipher),
                "Extended ciphers should not contain weak cipher 0x{:04x}",
                cipher
            );
        }
    }
}
