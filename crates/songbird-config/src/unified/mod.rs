// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unified Configuration System - DEPRECATED
//!
//! **DEPRECATED**: Use `crate::canonical::` module instead.
//!
//! This module is being phased out in favor of the fully consolidated
//! canonical configuration system. All types have been moved to `canonical::`.
//!
//! Migration Guide:
//! - `unified::core::*` → `canonical::core::*`
//! - `unified::observability::*` → `canonical::observability::*`

#[deprecated(
    since = "0.2.0",
    note = "Use `canonical::core::*` instead. This module is being phased out."
)]
#[allow(deprecated)]
pub use core::*;

#[deprecated(
    since = "0.2.0",
    note = "Use `canonical::federation::*` instead. This module is being phased out."
)]
#[allow(deprecated)]
pub use federation::*;

// Re-export canonical observability with deprecation notice
#[deprecated(
    since = "0.2.0",
    note = "Use `canonical::observability::UnifiedObservabilityConfig` instead. All types consolidated."
)]
pub use crate::canonical::observability::UnifiedObservabilityConfig;

// Core modules - DEPRECATED, use canonical instead
#[deprecated(since = "0.2.0", note = "Use `crate::canonical::core` instead")]
pub mod core;

#[deprecated(since = "0.2.0", note = "Use `crate::canonical::federation` instead")]
pub mod federation;

#[deprecated(since = "0.2.0", note = "Use `crate::canonical::robustness` instead")]
pub mod robustness;

// Observability moved to canonical
// Use: crate::canonical::observability instead

// Re-export the main config type with deprecation notice
#[deprecated(
    since = "0.2.0",
    note = "Use `crate::canonical::CanonicalSongbirdConfig` instead. Migration: `unified::core::SongbirdConfig` → `canonical::CanonicalSongbirdConfig`"
)]
#[allow(deprecated)]
pub use core::SongbirdConfig;
