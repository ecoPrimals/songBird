// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Mode
//!
//! Determines whether to use plaintext or encrypted birdSong broadcasts

use serde::{Deserialize, Serialize};

/// Discovery mode for federation broadcasts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiscoveryMode {
    /// Plaintext broadcasts (no security provider)
    ///
    /// Suitable for:
    /// - Trusted LANs (university campus, research lab)
    /// - Development/testing
    /// - Fast, zero-config setup
    ///
    /// Privacy: LOW (everything visible to network observers)
    #[default]
    Plaintext,

    /// Encrypted birdSong broadcasts (requires security provider)
    ///
    /// Suitable for:
    /// - Untrusted networks (internet, public `WiFi`, cellular)
    /// - Privacy-sensitive federations
    /// - Cross-organization collaboration
    ///
    /// Privacy: HIGH (only family can decrypt)
    BirdSong,
}

impl DiscoveryMode {
    /// Whether this mode needs a capability-discovered security provider
    #[must_use]
    pub const fn requires_security_provider(&self) -> bool {
        matches!(self, Self::BirdSong)
    }

    /// Deprecated alias for [`Self::requires_security_provider`].
    #[deprecated(note = "use requires_security_provider")]
    #[must_use]
    pub const fn requires_beardog(&self) -> bool {
        self.requires_security_provider()
    }

    /// Check if this mode is privacy-preserving
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Self::BirdSong)
    }

    /// Get human-readable description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Plaintext => "Plaintext (trusted LAN only)",
            Self::BirdSong => "BirdSong (privacy-preserving, encrypted)",
        }
    }
}

impl std::fmt::Display for DiscoveryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plaintext => write!(f, "plaintext"),
            Self::BirdSong => write!(f, "birdsong"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn default_is_plaintext() {
        assert_eq!(DiscoveryMode::default(), DiscoveryMode::Plaintext);
    }

    #[test]
    fn display_and_description() {
        assert_eq!(format!("{}", DiscoveryMode::Plaintext), "plaintext");
        assert_eq!(format!("{}", DiscoveryMode::BirdSong), "birdsong");
        assert!(DiscoveryMode::Plaintext.description().contains("Plaintext"));
        assert!(DiscoveryMode::BirdSong.description().contains("BirdSong"));
    }

    #[test]
    fn requires_security_provider_and_privacy_only_for_birdsong() {
        assert!(!DiscoveryMode::Plaintext.requires_security_provider());
        assert!(!DiscoveryMode::Plaintext.is_private());
        assert!(DiscoveryMode::BirdSong.requires_security_provider());
        assert!(DiscoveryMode::BirdSong.is_private());
    }

    #[test]
    fn serde_roundtrip() {
        let m = DiscoveryMode::BirdSong;
        let json = serde_json::to_string(&m).unwrap();
        let back: DiscoveryMode = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(DiscoveryMode::Plaintext, DiscoveryMode::BirdSong);
    }
}
