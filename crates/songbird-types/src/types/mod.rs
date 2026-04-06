// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical type definitions shared across the Songbird ecosystem
//!
//! This module provides unified type definitions to eliminate duplicates
//! and ensure consistency across all crates.

pub mod canonical;
pub mod hooks;
pub mod severity;

// Re-export canonical types
pub use canonical::{
    CanonicalAddress, CanonicalEndpoint, CanonicalNodeType, CanonicalRequest, CanonicalResponse,
};

// Re-export unified types for convenience
pub use hooks::HookErrorHandling;
pub use severity::{ErrorSeverity, WarningSeverity};
