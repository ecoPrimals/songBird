//! Discovery Mode
//!
//! Determines whether to use plaintext or encrypted birdSong broadcasts

use serde::{Deserialize, Serialize};

/// Discovery mode for federation broadcasts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMode {
    /// Plaintext broadcasts (no `BearDog`)
    ///
    /// Suitable for:
    /// - Trusted LANs (university campus, research lab)
    /// - Development/testing
    /// - Fast, zero-config setup
    ///
    /// Privacy: LOW (everything visible to network observers)
    Plaintext,

    /// Encrypted birdSong broadcasts (with `BearDog`)
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
    /// Check if this mode requires `BearDog`
    #[must_use]
    pub fn requires_beardog(&self) -> bool {
        matches!(self, Self::BirdSong)
    }

    /// Check if this mode is privacy-preserving
    #[must_use]
    pub fn is_private(&self) -> bool {
        matches!(self, Self::BirdSong)
    }

    /// Get human-readable description
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Plaintext => "Plaintext (trusted LAN only)",
            Self::BirdSong => "BirdSong (privacy-preserving, encrypted)",
        }
    }
}

impl Default for DiscoveryMode {
    fn default() -> Self {
        // Default to plaintext for backward compatibility
        Self::Plaintext
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
