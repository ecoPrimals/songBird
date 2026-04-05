// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Centralized name constants for self-knowledge and ecosystem fallbacks.
//!
//! Prefer capability-based discovery over hardcoded service or primal names at runtime.

/// This primal's canonical name (self-knowledge).
pub const SELF_NAME: &str = "songbird";

/// Filesystem directory name for XDG-style paths (`~/.config/songbird/`).
pub const APP_DIR: &str = "songbird";

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
