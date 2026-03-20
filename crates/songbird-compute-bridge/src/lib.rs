// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Songbird Compute Bridge - Agnostic compute service coordination
//!
//! This crate provides agnostic compute coordination for Songbird, enabling
//! deployment to ANY compute provider without hardcoded dependencies.
#![forbid(unsafe_code)]

pub mod agnostic_coordinator;
pub mod error;

mod service;

// Re-exports for convenience
pub use agnostic_coordinator::{
    AgnosticComputeCoordinator, ComputeCoordinatorConfig, ComputeError, ComputeProvider,
    DeploymentId, Workload,
};

pub use service::{Args, run};

/// Compute bridge version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
