// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird Compute Bridge - Agnostic compute service coordination
//!
//! This crate provides agnostic compute coordination for Songbird, enabling
//! deployment to ANY compute provider without hardcoded dependencies.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Capability-based compute coordination and provider discovery.
pub mod agnostic_coordinator;

/// Error types for compute bridge operations.
pub mod error;

mod service;

// Re-exports for convenience
pub use agnostic_coordinator::{
    AgnosticComputeCoordinator, ComputeCoordinatorConfig, ComputeError, ComputeProvider,
    DeploymentId, Workload,
};

pub use service::{Args, run};

/// Crate version from `CARGO_PKG_VERSION`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod version_tests {
    #[test]
    fn version_matches_semver_shape() {
        assert!(!super::VERSION.is_empty(), "VERSION should be populated from CARGO_PKG_VERSION");
        assert!(
            super::VERSION.chars().any(|c| c.is_ascii_digit()),
            "expected version to contain digits, got {:?}",
            super::VERSION
        );
    }

    #[test]
    fn version_is_ascii_printable() {
        assert!(
            super::VERSION.is_ascii(),
            "CARGO_PKG_VERSION should be ASCII for logging and HTTP headers; got {:?}",
            super::VERSION
        );
        assert!(
            !super::VERSION.contains(|c: char| c.is_control()),
            "version should not contain control characters: {:?}",
            super::VERSION
        );
    }

    #[test]
    fn version_has_semver_like_segments() {
        let parts: Vec<&str> = super::VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "workspace versions are typically major.minor.patch; got segments {:?}",
            parts
        );
        assert!(
            parts.iter().all(|p| !p.is_empty()),
            "no empty semver segment in {:?}",
            super::VERSION
        );
    }
}
