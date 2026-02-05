//! BirdSong configuration
//!
//! Configuration types for BirdSong encrypted discovery.

use serde::{Deserialize, Serialize};

/// `BirdSong` configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongConfig {
    /// Enable `BirdSong` encryption
    ///
    /// When `true`, uses encryption if provider available.
    /// When `false`, always uses plaintext.
    pub enabled: bool,

    /// Graceful fallback to plaintext if encryption unavailable
    ///
    /// When `true`, falls back to plaintext if provider unavailable.
    /// When `false`, fails if provider unavailable.
    ///
    /// Recommended: `true` for LAN learning, `false` for production internet
    pub fallback_to_plaintext: bool,

    /// Security provider endpoint (auto-discovered if None)
    ///
    /// If provided, connects to specific endpoint.
    /// If None, discovers via capability system.
    pub security_endpoint: Option<String>,

    /// Enable mixed-mode operation
    ///
    /// When `true`, can decrypt both encrypted and plaintext packets.
    /// When `false`, only processes encrypted packets when encryption enabled.
    ///
    /// Recommended: `true` during migration, `false` in steady state
    pub mixed_mode: bool,

    // ✅ NEW (Feb 3, 2026): Dark Forest Beacon Genetics configuration
    /// Enable Dark Forest beacons (fully encrypted, zero metadata leakage)
    ///
    /// When `true`, broadcasts Dark Forest beacons (version 2, fully encrypted).
    /// When `false`, broadcasts legacy `BirdSongPacket` (version 1.0, plaintext `family_id`).
    ///
    /// **Privacy Impact**: Dark Forest beacons eliminate metadata leakage.
    /// Legacy format leaks `family_id` in plaintext.
    ///
    /// Requires `BearDog` `beacon.*` RPC methods for full functionality.
    /// Falls back to legacy if beacon methods unavailable.
    ///
    /// Recommended: `true` for privacy, `false` only for compatibility testing
    pub dark_forest_enabled: bool,

    /// Accept legacy `BirdSongPacket` format (backward compatibility)
    ///
    /// When `true`, accepts both Dark Forest beacons AND legacy packets.
    /// When `false`, only accepts Dark Forest beacons (rejects legacy).
    ///
    /// **Migration Strategy**:
    /// - Phase 1 (Weeks 1-4): `true` (dual format support)
    /// - Phase 2 (Weeks 5-8): `true` (still accepting legacy)
    /// - Phase 3 (Weeks 9+): `false` (Dark Forest only, optional)
    ///
    /// Recommended: `true` during migration, `false` after full rollout
    pub accept_legacy_format: bool,

    /// Broadcast legacy format alongside Dark Forest (migration aid)
    ///
    /// When `true`, broadcasts BOTH Dark Forest AND legacy packets.
    /// When `false`, only broadcasts Dark Forest beacons.
    ///
    /// **Use Cases**:
    /// - Early migration: Help peers discover both formats
    /// - Compatibility testing: Verify dual-format handling
    /// - Gradual rollout: Support mixed network
    ///
    /// **Overhead**: ~2x network bandwidth during dual broadcast.
    ///
    /// Recommended: `true` only during early migration (Weeks 1-2),
    ///              `false` after peers upgraded
    pub dual_broadcast: bool,
}

impl Default for BirdSongConfig {
    fn default() -> Self {
        Self {
            // Legacy configuration (unchanged)
            enabled: false,              // Opt-in for privacy
            fallback_to_plaintext: true, // Graceful degradation
            security_endpoint: None,     // Auto-discover
            mixed_mode: true,            // Support migration

            // Dark Forest configuration (conservative defaults)
            dark_forest_enabled: false, // Opt-in (requires BearDog beacon.*)
            accept_legacy_format: true, // Backward compatible by default
            dual_broadcast: false,      // Minimize network overhead
        }
    }
}

impl BirdSongConfig {
    /// Create config for Dark Forest mode (privacy-first)
    ///
    /// Broadcasts Dark Forest beacons, accepts legacy for compatibility.
    /// Use this for production deployments with privacy requirements.
    #[must_use]
    pub fn dark_forest() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true, // Still accept legacy for compatibility
            dual_broadcast: false,      // Only send Dark Forest
            ..Default::default()
        }
    }

    /// Create config for migration period (dual format)
    ///
    /// Broadcasts BOTH formats, accepts both.
    /// Use this during early migration (Weeks 1-2) to help peers discover.
    #[must_use]
    pub fn migration_mode() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: true,
            dual_broadcast: true, // Send both formats
            ..Default::default()
        }
    }

    /// Create config for legacy-only mode (testing/compatibility)
    ///
    /// Broadcasts legacy format, accepts both.
    /// Use this only for compatibility testing or legacy deployments.
    #[must_use]
    pub fn legacy_only() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: false,
            accept_legacy_format: true,
            dual_broadcast: false,
            ..Default::default()
        }
    }

    /// Create config for Dark Forest only (maximum privacy)
    ///
    /// Broadcasts Dark Forest only, rejects legacy.
    /// Use this after full network migration (Phase 3+).
    ///
    /// **Warning**: Incompatible with legacy peers.
    #[must_use]
    pub fn dark_forest_only() -> Self {
        Self {
            enabled: true,
            dark_forest_enabled: true,
            accept_legacy_format: false, // Reject legacy packets
            dual_broadcast: false,
            fallback_to_plaintext: false, // No fallback
            ..Default::default()
        }
    }

    /// Check if Dark Forest is fully enabled
    ///
    /// Returns `true` only if both `enabled` and `dark_forest_enabled` are true.
    #[must_use]
    pub fn is_dark_forest_active(&self) -> bool {
        self.enabled && self.dark_forest_enabled
    }

    /// Check if accepting any legacy format
    ///
    /// Returns `true` if either legacy format accepted OR dual broadcast enabled.
    #[must_use]
    pub fn accepts_legacy(&self) -> bool {
        self.accept_legacy_format || !self.dark_forest_enabled
    }
}
