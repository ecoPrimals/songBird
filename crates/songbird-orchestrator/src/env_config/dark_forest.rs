// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::env;

/// Check if Dark Forest beacons are enabled
///
/// Reads `SONGBIRD_DARK_FOREST` environment variable.
///
/// When `true`, Songbird broadcasts Dark Forest beacons (version 2, fully encrypted).
/// When `false`, Songbird broadcasts legacy `BirdSongPacket` (version 1.0, plaintext `family_id`).
///
/// **Default**: `false` (opt-in for privacy, requires `security provider` beacon.* RPC)
#[must_use]
pub fn dark_forest_enabled() -> bool {
    env("SONGBIRD_DARK_FOREST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}

/// Check if legacy `BirdSongPacket` format should be accepted
///
/// Reads `SONGBIRD_ACCEPT_LEGACY_BIRDSONG` environment variable.
///
/// When `true`, accepts both Dark Forest beacons AND legacy `BirdSongPacket`.
/// When `false`, only accepts Dark Forest beacons (rejects legacy).
///
/// **Default**: `true` (backward compatible during migration)
#[must_use]
pub fn accept_legacy_birdsong() -> bool {
    env("SONGBIRD_ACCEPT_LEGACY_BIRDSONG").ok().and_then(|v| v.parse().ok()).unwrap_or(true)
}

/// Check if dual broadcast is enabled (both formats)
///
/// Reads `SONGBIRD_DUAL_BROADCAST` environment variable.
///
/// When `true`, broadcasts BOTH Dark Forest beacons AND legacy `BirdSongPacket`.
/// When `false`, only broadcasts Dark Forest beacons (if enabled).
///
/// **Default**: `false` (minimize network overhead)
#[must_use]
pub fn dual_broadcast() -> bool {
    env("SONGBIRD_DUAL_BROADCAST").ok().and_then(|v| v.parse().ok()).unwrap_or(false)
}
