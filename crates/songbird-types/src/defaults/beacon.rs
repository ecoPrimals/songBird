// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Beacon credential tier model per `DARK_FOREST_BEACON_GENETICS_STANDARD.md`.
//!
//! The two-seed model separates **discovery** from **authorization**:
//!
//! - **Mitochondrial (beacon-tier)**: Shared across family, used for Dark
//!   Forest beacon encryption and STUN/NAT traversal. Exposure reveals
//!   discovery visibility only, never authorization material.
//!
//! - **Nuclear (lineage-tier)**: Unique per device, used for BTSP handshakes,
//!   lineage verification, and permission grants. Must never be embedded in
//!   beacons or discovery traffic.

/// JSON-RPC method for beacon-tier encryption (Dark Forest beacons).
///
/// Delegates to `BearDog`'s beacon seed, separate from `birdsong.encrypt`
/// which uses the lineage (nuclear) seed.
pub const BEACON_ENCRYPT_METHOD: &str = "beacon.encrypt";

/// JSON-RPC method for beacon-tier decryption.
pub const BEACON_DECRYPT_METHOD: &str = "beacon.decrypt";

/// JSON-RPC method to retrieve our public beacon identifier.
pub const BEACON_GET_ID_METHOD: &str = "beacon.get_id";

/// Legacy JSON-RPC method for family-scoped encryption (lineage/nuclear tier).
///
/// This uses the lineage seed and should NOT be used for Dark Forest beacons.
/// Retained for backward compatibility when the security provider doesn't yet
/// expose `beacon.*` methods. Songbird's `SecurityBirdSongProvider` attempts
/// `beacon.*` first and falls back to `birdsong.*` transparently.
pub const LEGACY_BIRDSONG_ENCRYPT_METHOD: &str = "birdsong.encrypt";

/// Legacy JSON-RPC method for family-scoped decryption (lineage/nuclear tier).
pub const LEGACY_BIRDSONG_DECRYPT_METHOD: &str = "birdsong.decrypt";
