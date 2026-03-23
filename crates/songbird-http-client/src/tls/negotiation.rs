// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Algorithm Negotiation System
//!
//! Flexible, adaptive negotiation for TLS handshakes that can handle:
//! - Multiple signature algorithm preferences
//! - Conflicting server requirements
//! - Learning from handshake failures
//! - BTSP extension compatibility

use std::collections::HashMap;
use tracing::{debug, warn};

/// Signature algorithm identifier (RFC 8446)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SignatureAlgorithm {
    // ECDSA variants
    EcdsaSecp256r1Sha256 = 0x0403,
    EcdsaSecp384r1Sha384 = 0x0503,
    EcdsaSecp521r1Sha512 = 0x0603,

    // EdDSA variants
    Ed25519 = 0x0807,
    Ed448 = 0x0808,

    // RSA PKCS1 variants
    RsaPkcs1Sha256 = 0x0401,
    RsaPkcs1Sha384 = 0x0501,
    RsaPkcs1Sha512 = 0x0601,

    // RSA PSS variants
    RsaPssRsaeSha256 = 0x0804,
    RsaPssRsaeSha384 = 0x0805,
    RsaPssRsaeSha512 = 0x0806,

    // RSA PSS PSS variants
    RsaPssPssSha256 = 0x0809,
    RsaPssPssSha384 = 0x080a,
    RsaPssPssSha512 = 0x080b,
}

impl SignatureAlgorithm {
    /// Get algorithm as u16 for wire format
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Get human-readable name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::EcdsaSecp256r1Sha256 => "ecdsa_secp256r1_sha256",
            Self::EcdsaSecp384r1Sha384 => "ecdsa_secp384r1_sha384",
            Self::EcdsaSecp521r1Sha512 => "ecdsa_secp521r1_sha512",
            Self::Ed25519 => "ed25519",
            Self::Ed448 => "ed448",
            Self::RsaPkcs1Sha256 => "rsa_pkcs1_sha256",
            Self::RsaPkcs1Sha384 => "rsa_pkcs1_sha384",
            Self::RsaPkcs1Sha512 => "rsa_pkcs1_sha512",
            Self::RsaPssRsaeSha256 => "rsa_pss_rsae_sha256",
            Self::RsaPssRsaeSha384 => "rsa_pss_rsae_sha384",
            Self::RsaPssRsaeSha512 => "rsa_pss_rsae_sha512",
            Self::RsaPssPssSha256 => "rsa_pss_pss_sha256",
            Self::RsaPssPssSha384 => "rsa_pss_pss_sha384",
            Self::RsaPssPssSha512 => "rsa_pss_pss_sha512",
        }
    }

    /// Get algorithm family
    #[must_use]
    pub const fn family(self) -> AlgorithmFamily {
        match self {
            Self::EcdsaSecp256r1Sha256
            | Self::EcdsaSecp384r1Sha384
            | Self::EcdsaSecp521r1Sha512 => AlgorithmFamily::Ecdsa,

            Self::Ed25519 | Self::Ed448 => AlgorithmFamily::EdDsa,

            Self::RsaPkcs1Sha256
            | Self::RsaPkcs1Sha384
            | Self::RsaPkcs1Sha512
            | Self::RsaPssRsaeSha256
            | Self::RsaPssRsaeSha384
            | Self::RsaPssRsaeSha512
            | Self::RsaPssPssSha256
            | Self::RsaPssPssSha384
            | Self::RsaPssPssSha512 => AlgorithmFamily::Rsa,
        }
    }

    /// Is this algorithm currently supported by `BearDog`?
    #[must_use]
    pub const fn is_supported(self) -> bool {
        matches!(
            self,
            Self::EcdsaSecp256r1Sha256 // Supported via secp256k1?
            | Self::Ed25519              // Supported
            | Self::Ed448 // Likely supported
        )
    }
}

/// Algorithm family grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmFamily {
    Ecdsa,
    EdDsa,
    Rsa,
}

/// Negotiation strategy for algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationStrategy {
    /// Prefer modern, secure algorithms (`EdDSA` > ECDSA > RSA)
    PreferModern,

    /// Maximize compatibility (advertise all algorithms)
    MaxCompatibility,

    /// Use only algorithms we fully support in `BearDog`
    OnlySupported,

    /// Custom priority list
    Custom,

    /// Adaptive: learn from previous handshakes
    Adaptive,
}

/// Profile for a specific server's TLS preferences
#[derive(Debug, Clone)]
pub struct ServerProfile {
    /// Server hostname
    pub hostname: String,

    /// Known working signature algorithm
    pub preferred_algorithm: Option<SignatureAlgorithm>,

    /// Algorithms that failed
    pub failed_algorithms: Vec<SignatureAlgorithm>,

    /// Number of successful handshakes
    pub success_count: u32,

    /// Number of failed handshakes
    pub failure_count: u32,

    /// Last handshake timestamp
    pub last_handshake: Option<std::time::Instant>,
}

impl ServerProfile {
    #[must_use]
    pub const fn new(hostname: String) -> Self {
        Self {
            hostname,
            preferred_algorithm: None,
            failed_algorithms: Vec::new(),
            success_count: 0,
            failure_count: 0,
            last_handshake: None,
        }
    }

    /// Record a successful handshake with the given algorithm
    pub fn record_success(&mut self, algorithm: SignatureAlgorithm) {
        self.preferred_algorithm = Some(algorithm);
        self.success_count += 1;
        self.last_handshake = Some(std::time::Instant::now());
        debug!("✅ Recorded successful handshake for {} with {}", self.hostname, algorithm.name());
    }

    /// Record a failed handshake
    pub fn record_failure(&mut self, attempted_algorithms: &[SignatureAlgorithm]) {
        self.failed_algorithms.extend(attempted_algorithms);
        self.failure_count += 1;
        self.last_handshake = Some(std::time::Instant::now());
        warn!(
            "❌ Recorded failed handshake for {} (attempt #{})",
            self.hostname, self.failure_count
        );
    }
}

/// TLS Algorithm Negotiator
///
/// Manages algorithm selection, learning, and adaptation for TLS handshakes.
pub struct AlgorithmNegotiator {
    /// Global strategy
    strategy: NegotiationStrategy,

    /// Server-specific profiles (learned over time)
    server_profiles: HashMap<String, ServerProfile>,

    /// Custom algorithm priority (when strategy is Custom)
    custom_priority: Vec<SignatureAlgorithm>,
}

impl AlgorithmNegotiator {
    /// Create negotiator with default strategy (`MaxCompatibility`)
    #[must_use]
    pub fn new() -> Self {
        Self {
            strategy: NegotiationStrategy::MaxCompatibility,
            server_profiles: HashMap::new(),
            custom_priority: Vec::new(),
        }
    }

    /// Create negotiator with specific strategy
    #[must_use]
    pub fn with_strategy(strategy: NegotiationStrategy) -> Self {
        Self {
            strategy,
            server_profiles: HashMap::new(),
            custom_priority: Vec::new(),
        }
    }

    /// Set custom algorithm priority
    pub fn set_custom_priority(&mut self, algorithms: Vec<SignatureAlgorithm>) {
        self.custom_priority = algorithms;
        self.strategy = NegotiationStrategy::Custom;
    }

    /// Get algorithms to advertise for a given server
    #[must_use]
    pub fn get_algorithms_for_server(&self, hostname: &str) -> Vec<SignatureAlgorithm> {
        match self.strategy {
            NegotiationStrategy::PreferModern => self.modern_algorithms(),
            NegotiationStrategy::MaxCompatibility => self.all_algorithms(),
            NegotiationStrategy::OnlySupported => self.supported_algorithms(),
            NegotiationStrategy::Custom => self.custom_priority.clone(),
            NegotiationStrategy::Adaptive => self.adaptive_algorithms(hostname),
        }
    }

    /// Modern algorithm preference (`EdDSA` > ECDSA > RSA)
    #[expect(clippy::unused_self, reason = "method logically belongs on this type")]
    fn modern_algorithms(&self) -> Vec<SignatureAlgorithm> {
        vec![
            // EdDSA (most modern)
            SignatureAlgorithm::Ed25519,
            SignatureAlgorithm::Ed448,
            // ECDSA (widely supported, modern)
            SignatureAlgorithm::EcdsaSecp256r1Sha256,
            SignatureAlgorithm::EcdsaSecp384r1Sha384,
            SignatureAlgorithm::EcdsaSecp521r1Sha512,
            // RSA PSS (modern RSA)
            SignatureAlgorithm::RsaPssRsaeSha256,
            SignatureAlgorithm::RsaPssRsaeSha384,
            // RSA PKCS1 (legacy, but needed for some servers)
            SignatureAlgorithm::RsaPkcs1Sha256,
        ]
    }

    /// All algorithms for maximum compatibility
    #[expect(clippy::unused_self, reason = "method logically belongs on this type")]
    fn all_algorithms(&self) -> Vec<SignatureAlgorithm> {
        vec![
            // ECDSA (most common for GitHub, CloudFlare, etc.)
            SignatureAlgorithm::EcdsaSecp256r1Sha256,
            SignatureAlgorithm::EcdsaSecp384r1Sha384,
            SignatureAlgorithm::EcdsaSecp521r1Sha512,
            // EdDSA (modern, secure)
            SignatureAlgorithm::Ed25519,
            SignatureAlgorithm::Ed448,
            // RSA PKCS1 (legacy, widely supported)
            SignatureAlgorithm::RsaPkcs1Sha256,
            SignatureAlgorithm::RsaPkcs1Sha384,
            SignatureAlgorithm::RsaPkcs1Sha512,
            // RSA PSS (modern RSA)
            SignatureAlgorithm::RsaPssRsaeSha256,
        ]
    }

    /// Only algorithms we fully support
    #[expect(clippy::unused_self, reason = "method logically belongs on this type")]
    fn supported_algorithms(&self) -> Vec<SignatureAlgorithm> {
        let _ = SignatureAlgorithm::Ed25519.family(); // Example
        vec![
            SignatureAlgorithm::Ed25519,
            // Add more as BearDog implements them
        ]
    }

    /// Adaptive selection based on learned server preferences
    fn adaptive_algorithms(&self, hostname: &str) -> Vec<SignatureAlgorithm> {
        if let Some(profile) = self.server_profiles.get(hostname)
            && let Some(preferred) = profile.preferred_algorithm
        {
            debug!("🧠 Using learned preference for {}: {}", hostname, preferred.name());

            // Put known-working algorithm first, then others
            let mut algs = vec![preferred];
            algs.extend(
                self.all_algorithms()
                    .into_iter()
                    .filter(|a| *a != preferred && !profile.failed_algorithms.contains(a)),
            );
            return algs;
        }

        // No profile yet, use MaxCompatibility
        debug!("🆕 No profile for {}, using MaxCompatibility", hostname);
        self.all_algorithms()
    }

    /// Record successful handshake (for adaptive learning)
    pub fn record_success(&mut self, hostname: &str, algorithm: SignatureAlgorithm) {
        let profile = self
            .server_profiles
            .entry(hostname.to_string())
            .or_insert_with(|| ServerProfile::new(hostname.to_string()));

        profile.record_success(algorithm);
    }

    /// Record failed handshake (for adaptive learning)
    pub fn record_failure(&mut self, hostname: &str, attempted_algorithms: &[SignatureAlgorithm]) {
        let profile = self
            .server_profiles
            .entry(hostname.to_string())
            .or_insert_with(|| ServerProfile::new(hostname.to_string()));

        profile.record_failure(attempted_algorithms);
    }

    /// Get profile for a server (for inspection/debugging)
    #[must_use]
    pub fn get_profile(&self, hostname: &str) -> Option<&ServerProfile> {
        self.server_profiles.get(hostname)
    }

    /// Clear learned profiles (reset adaptive learning)
    pub fn clear_profiles(&mut self) {
        self.server_profiles.clear();
        debug!("🗑️  Cleared all server profiles");
    }
}

impl Default for AlgorithmNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper: Convert algorithm list to wire format (u16 pairs)
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "TLS wire format: values are masked/bounded")]
pub fn algorithms_to_wire(algorithms: &[SignatureAlgorithm]) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Extension type: signature_algorithms (0x000d)
    bytes.extend_from_slice(&[0x00, 0x0d]);

    // Extension length
    let ext_len = 2 + (algorithms.len() * 2);
    bytes.extend_from_slice(&(ext_len as u16).to_be_bytes());

    // List length
    let list_len = algorithms.len() * 2;
    bytes.extend_from_slice(&(list_len as u16).to_be_bytes());

    // Algorithms
    for alg in algorithms {
        bytes.extend_from_slice(&alg.as_u16().to_be_bytes());
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_families() {
        assert_eq!(SignatureAlgorithm::Ed25519.family(), AlgorithmFamily::EdDsa);
        assert_eq!(SignatureAlgorithm::EcdsaSecp256r1Sha256.family(), AlgorithmFamily::Ecdsa);
        assert_eq!(SignatureAlgorithm::RsaPkcs1Sha256.family(), AlgorithmFamily::Rsa);
    }

    #[test]
    fn test_negotiator_max_compatibility() {
        let negotiator = AlgorithmNegotiator::new();
        let algs = negotiator.get_algorithms_for_server("api.github.com");

        // Should include ECDSA (for GitHub)
        assert!(algs.contains(&SignatureAlgorithm::EcdsaSecp256r1Sha256));

        // Should include EdDSA (for modern servers)
        assert!(algs.contains(&SignatureAlgorithm::Ed25519));

        // Should include RSA (for legacy servers)
        assert!(algs.contains(&SignatureAlgorithm::RsaPkcs1Sha256));
    }

    #[test]
    fn test_negotiator_adaptive_learning() {
        let mut negotiator = AlgorithmNegotiator::with_strategy(NegotiationStrategy::Adaptive);

        // First call: no profile, should use MaxCompatibility
        let algs1 = negotiator.get_algorithms_for_server("test.example.com");
        assert!(algs1.len() >= 8);

        // Record success with specific algorithm
        negotiator.record_success("test.example.com", SignatureAlgorithm::EcdsaSecp256r1Sha256);

        // Second call: should prefer learned algorithm
        let algs2 = negotiator.get_algorithms_for_server("test.example.com");
        assert_eq!(algs2[0], SignatureAlgorithm::EcdsaSecp256r1Sha256);
    }

    #[test]
    fn test_wire_format() {
        let algs = vec![SignatureAlgorithm::EcdsaSecp256r1Sha256, SignatureAlgorithm::Ed25519];

        let wire = algorithms_to_wire(&algs);

        // Check extension type (0x000d)
        assert_eq!(&wire[0..2], &[0x00, 0x0d]);

        // Check algorithms are present
        assert!(wire.contains(&0x04) && wire.contains(&0x03)); // ECDSA
        assert!(wire.contains(&0x08) && wire.contains(&0x07)); // Ed25519
    }

    #[test]
    fn test_server_profile_learning() {
        let mut profile = ServerProfile::new("test.com".to_string());

        assert_eq!(profile.success_count, 0);
        assert_eq!(profile.failure_count, 0);

        profile.record_success(SignatureAlgorithm::Ed25519);
        assert_eq!(profile.success_count, 1);
        assert_eq!(profile.preferred_algorithm, Some(SignatureAlgorithm::Ed25519));

        profile.record_failure(&[SignatureAlgorithm::RsaPkcs1Sha256]);
        assert_eq!(profile.failure_count, 1);
        assert!(profile.failed_algorithms.contains(&SignatureAlgorithm::RsaPkcs1Sha256));
    }
}
