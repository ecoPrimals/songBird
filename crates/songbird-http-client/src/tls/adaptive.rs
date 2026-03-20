// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Adaptive TLS extension negotiation
//!
//! This module provides adaptive behavior for TLS handshakes, allowing Songbird
//! to automatically adjust extension sets based on server requirements and profiles.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Strategy for extension negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionStrategy {
    /// Modern only - prefer latest TLS 1.3 features
    Modern,
    /// Maximum compatibility - include all possible extensions
    MaxCompatibility,
    /// Minimal - only required extensions
    Minimal,
    /// Adaptive - learn from server responses
    Adaptive,
}

/// Server profile for adaptive negotiation
#[derive(Debug, Clone)]
pub struct ServerProfile {
    /// Server hostname
    pub hostname: String,
    /// Last successful extension set
    pub successful_extensions: Vec<ExtensionType>,
    /// Failed extension types
    pub failed_extensions: Vec<ExtensionType>,
    /// Success count
    pub success_count: u32,
    /// Failure count
    pub failure_count: u32,
    /// Last updated timestamp
    pub last_updated: std::time::SystemTime,
}

/// TLS extension types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    /// Server Name Indication (0x0000)
    Sni,
    /// Application-Layer Protocol Negotiation (0x0010)
    Alpn,
    /// Supported Versions (0x002b)
    SupportedVersions,
    /// Key Share (0x0033)
    KeyShare,
    /// Supported Groups (0x000a)
    SupportedGroups,
    /// Signature Algorithms (0x000d)
    SignatureAlgorithms,
    /// PSK Key Exchange Modes (0x002d)
    PskKeyExchangeModes,
}

impl ExtensionType {
    /// Get the wire format ID for this extension
    #[must_use]
    pub const fn id(&self) -> u16 {
        match self {
            Self::Sni => 0x0000,
            Self::Alpn => 0x0010,
            Self::SupportedVersions => 0x002b,
            Self::KeyShare => 0x0033,
            Self::SupportedGroups => 0x000a,
            Self::SignatureAlgorithms => 0x000d,
            Self::PskKeyExchangeModes => 0x002d,
        }
    }

    /// Get human-readable name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sni => "server_name",
            Self::Alpn => "alpn",
            Self::SupportedVersions => "supported_versions",
            Self::KeyShare => "key_share",
            Self::SupportedGroups => "supported_groups",
            Self::SignatureAlgorithms => "signature_algorithms",
            Self::PskKeyExchangeModes => "psk_key_exchange_modes",
        }
    }
}

/// Adaptive extension manager
pub struct AdaptiveExtensions {
    /// Server profiles cache
    profiles: Arc<RwLock<HashMap<String, ServerProfile>>>,
    /// Default strategy
    strategy: ExtensionStrategy,
}

impl AdaptiveExtensions {
    /// Create a new adaptive extension manager
    #[must_use]
    pub fn new(strategy: ExtensionStrategy) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            strategy,
        }
    }

    /// Get extension set for a server
    #[must_use]
    pub fn get_extensions(&self, hostname: &str) -> Vec<ExtensionType> {
        match self.strategy {
            ExtensionStrategy::Modern => Self::modern_extensions(),
            ExtensionStrategy::MaxCompatibility => self.max_compatibility_extensions(),
            ExtensionStrategy::Minimal => self.minimal_extensions(),
            ExtensionStrategy::Adaptive => self.adaptive_extensions(hostname),
        }
    }

    /// Modern extension set (TLS 1.3 preferred)
    fn modern_extensions() -> Vec<ExtensionType> {
        vec![
            ExtensionType::Sni,
            ExtensionType::Alpn,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SupportedGroups,
            ExtensionType::SignatureAlgorithms,
        ]
    }

    /// Maximum compatibility extension set
    fn max_compatibility_extensions(&self) -> Vec<ExtensionType> {
        vec![
            ExtensionType::Sni,
            ExtensionType::Alpn,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SupportedGroups,
            ExtensionType::SignatureAlgorithms,
            ExtensionType::PskKeyExchangeModes,
        ]
    }

    /// Minimal extension set (required only)
    fn minimal_extensions(&self) -> Vec<ExtensionType> {
        vec![
            ExtensionType::Sni,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SignatureAlgorithms,
        ]
    }

    /// Adaptive extension set based on server profile
    fn adaptive_extensions(&self, hostname: &str) -> Vec<ExtensionType> {
        let profiles = self.profiles.read().unwrap();

        if let Some(profile) = profiles.get(hostname) {
            // Use known successful extension set
            if profile.success_count > 0 {
                return profile.successful_extensions.clone();
            }
        }

        // Default to modern if no profile exists
        drop(profiles);
        Self::modern_extensions()
    }

    /// Record successful handshake
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[allow(clippy::significant_drop_tightening)] // Guard must be held while modifying profile
    pub fn record_success(&self, hostname: &str, extensions: Vec<ExtensionType>) {
        let mut profiles = self.profiles.write().unwrap();

        let profile = profiles.entry(hostname.to_string()).or_insert_with(|| ServerProfile {
            hostname: hostname.to_string(),
            successful_extensions: Vec::new(),
            failed_extensions: Vec::new(),
            success_count: 0,
            failure_count: 0,
            last_updated: std::time::SystemTime::now(),
        });

        profile.successful_extensions = extensions;
        profile.success_count += 1;
        profile.last_updated = std::time::SystemTime::now();
    }

    /// Record failed handshake
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[allow(clippy::significant_drop_tightening)] // Guard must be held while modifying profile
    pub fn record_failure(&self, hostname: &str, extensions: Vec<ExtensionType>) {
        let mut profiles = self.profiles.write().unwrap();

        let profile = profiles.entry(hostname.to_string()).or_insert_with(|| ServerProfile {
            hostname: hostname.to_string(),
            successful_extensions: Vec::new(),
            failed_extensions: Vec::new(),
            success_count: 0,
            failure_count: 0,
            last_updated: std::time::SystemTime::now(),
        });

        profile.failed_extensions = extensions;
        profile.failure_count += 1;
        profile.last_updated = std::time::SystemTime::now();
    }

    /// Get server profile
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn get_profile(&self, hostname: &str) -> Option<ServerProfile> {
        let profiles = self.profiles.read().unwrap();
        profiles.get(hostname).cloned()
    }

    /// Clear all profiles (for testing)
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn clear_profiles(&self) {
        let mut profiles = self.profiles.write().unwrap();
        profiles.clear();
    }

    /// Get profile count (for testing)
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    #[must_use]
    pub fn profile_count(&self) -> usize {
        let profiles = self.profiles.read().unwrap();
        profiles.len()
    }
}

impl Clone for AdaptiveExtensions {
    fn clone(&self) -> Self {
        Self {
            profiles: Arc::clone(&self.profiles),
            strategy: self.strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modern_extensions() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Modern);
        let extensions = adaptive.get_extensions("example.com");

        assert_eq!(extensions.len(), 6);
        assert!(extensions.contains(&ExtensionType::Sni));
        assert!(extensions.contains(&ExtensionType::Alpn));
        assert!(extensions.contains(&ExtensionType::SupportedVersions));
        assert!(extensions.contains(&ExtensionType::KeyShare));
    }

    #[test]
    fn test_minimal_extensions() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Minimal);
        let extensions = adaptive.get_extensions("example.com");

        assert_eq!(extensions.len(), 4);
        assert!(extensions.contains(&ExtensionType::Sni));
        assert!(extensions.contains(&ExtensionType::SupportedVersions));
        assert!(extensions.contains(&ExtensionType::KeyShare));
        assert!(extensions.contains(&ExtensionType::SignatureAlgorithms));
    }

    #[test]
    fn test_max_compatibility_extensions() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::MaxCompatibility);
        let extensions = adaptive.get_extensions("example.com");

        assert_eq!(extensions.len(), 7);
        assert!(extensions.contains(&ExtensionType::PskKeyExchangeModes));
    }

    #[test]
    fn test_adaptive_learning() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);

        // First call - no profile, should use modern defaults
        let extensions1 = adaptive.get_extensions("github.com");
        assert_eq!(extensions1.len(), 6);

        // Record successful handshake with minimal set
        let minimal = vec![
            ExtensionType::Sni,
            ExtensionType::SupportedVersions,
            ExtensionType::KeyShare,
            ExtensionType::SignatureAlgorithms,
        ];
        adaptive.record_success("github.com", minimal.clone());

        // Second call - should use learned profile
        let extensions2 = adaptive.get_extensions("github.com");
        assert_eq!(extensions2.len(), 4);
        assert_eq!(extensions2, minimal);
    }

    #[test]
    fn test_profile_recording() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);

        let extensions = vec![ExtensionType::Sni, ExtensionType::Alpn];
        adaptive.record_success("example.com", extensions.clone());

        let profile = adaptive.get_profile("example.com").unwrap();
        assert_eq!(profile.hostname, "example.com");
        assert_eq!(profile.success_count, 1);
        assert_eq!(profile.failure_count, 0);
        assert_eq!(profile.successful_extensions, extensions);
    }

    #[test]
    fn test_failure_recording() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);

        let extensions = vec![ExtensionType::Alpn];
        adaptive.record_failure("badserver.com", extensions.clone());

        let profile = adaptive.get_profile("badserver.com").unwrap();
        assert_eq!(profile.failure_count, 1);
        assert_eq!(profile.failed_extensions, extensions);
    }

    #[test]
    fn test_extension_ids() {
        assert_eq!(ExtensionType::Sni.id(), 0x0000);
        assert_eq!(ExtensionType::Alpn.id(), 0x0010);
        assert_eq!(ExtensionType::SupportedVersions.id(), 0x002b);
        assert_eq!(ExtensionType::KeyShare.id(), 0x0033);
    }

    #[test]
    fn test_extension_names() {
        assert_eq!(ExtensionType::Sni.name(), "server_name");
        assert_eq!(ExtensionType::Alpn.name(), "alpn");
        assert_eq!(ExtensionType::SupportedVersions.name(), "supported_versions");
    }

    #[test]
    fn test_clear_profiles() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);

        adaptive.record_success("example.com", vec![ExtensionType::Sni]);
        assert_eq!(adaptive.profile_count(), 1);

        adaptive.clear_profiles();
        assert_eq!(adaptive.profile_count(), 0);
    }

    #[test]
    fn test_multiple_servers() {
        let adaptive = AdaptiveExtensions::new(ExtensionStrategy::Adaptive);

        adaptive.record_success("github.com", vec![ExtensionType::Sni]);
        adaptive.record_success("google.com", vec![ExtensionType::Alpn]);
        adaptive.record_success("cloudflare.com", vec![ExtensionType::KeyShare]);

        assert_eq!(adaptive.profile_count(), 3);

        let github_profile = adaptive.get_profile("github.com").unwrap();
        assert_eq!(github_profile.successful_extensions, vec![ExtensionType::Sni]);

        let google_profile = adaptive.get_profile("google.com").unwrap();
        assert_eq!(google_profile.successful_extensions, vec![ExtensionType::Alpn]);
    }
}
