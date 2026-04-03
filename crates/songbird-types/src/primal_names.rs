// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Centralized primal name constants.
//!
//! Eliminates raw string literals scattered across production code.
//! Self-knowledge constants are used freely; external primal names
//! are kept here to track dependency on specific primals — prefer
//! capability-based discovery over hardcoded names at runtime.

/// This primal's canonical name (self-knowledge).
pub const SELF_NAME: &str = "songbird";

/// Filesystem directory name for XDG-style paths (`~/.config/songbird/`).
pub const APP_DIR: &str = "songbird";

/// External primal name for the security/crypto capability provider.
///
/// Use capability-domain discovery (`Capability::Security`, `Capability::Crypto`)
/// instead of this constant for IPC routing.
#[deprecated(note = "use capability-domain discovery, not primal identity")]
pub const BEARDOG: &str = "beardog";

/// External primal name for the coordinator / AI orchestration capability provider.
#[deprecated(note = "use capability-domain discovery, not primal identity")]
pub const SQUIRREL: &str = "squirrel";

/// External primal name for the compute capability provider.
#[deprecated(note = "use capability-domain discovery, not primal identity")]
pub const TOADSTOOL: &str = "toadstool";

/// External primal name for the storage / gateway capability provider.
#[deprecated(note = "use capability-domain discovery, not primal identity")]
pub const NESTGATE: &str = "nestgate";

/// External: biomeOS Neural API service name for socket discovery.
///
/// Used as a fallback when capability-based discovery is unavailable.
/// Prefer `capability_discovery("crypto")` over direct name-based lookup.
pub const NEURAL_API: &str = "neural-api";

/// Ecosystem socket directory name (`$XDG_RUNTIME_DIR/biomeos/`).
pub const BIOMEOS_DIR: &str = "biomeos";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_api_constant() {
        assert_eq!(NEURAL_API, "neural-api");
    }

    #[test]
    fn test_biomeos_dir_constant() {
        assert_eq!(BIOMEOS_DIR, "biomeos");
    }
}
