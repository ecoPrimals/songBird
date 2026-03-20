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
