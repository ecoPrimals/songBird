//! Songbird Compute Bridge - Agnostic compute service coordination
//!
//! This crate provides agnostic compute coordination for Songbird, enabling
//! deployment to ANY compute provider without hardcoded dependencies.

pub mod agnostic_coordinator;
pub mod error;

// Re-exports for convenience
pub use agnostic_coordinator::{
    AgnosticComputeCoordinator, ComputeCoordinatorConfig, ComputeError, ComputeProvider,
    DeploymentId, Workload,
};

/// Compute bridge version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

