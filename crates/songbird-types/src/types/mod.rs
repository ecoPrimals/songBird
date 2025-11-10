//! Canonical type definitions shared across the Songbird ecosystem
//!
//! This module provides unified type definitions to eliminate duplicates
//! and ensure consistency across all crates.

pub mod canonical;
pub mod severity;
pub mod hooks;

// Re-export canonical types
pub use canonical::{
    CanonicalAddress, CanonicalEndpoint, CanonicalNodeType, CanonicalRequest, CanonicalResponse,
};

// Re-export unified types for convenience
pub use severity::{ErrorSeverity, WarningSeverity};
pub use hooks::HookErrorHandling;

