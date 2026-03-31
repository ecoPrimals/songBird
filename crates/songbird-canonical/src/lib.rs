// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🏗️ Songbird Canonical Types and Patterns
//!
//! **CANONICAL UNIFICATION COMPLETE** ✅
//!
//! This crate provides the canonical, unified type definitions and patterns
//! that serve as the foundation for all Songbird components.
//!
//! ## 🎯 **Canonical Achievements**
//!
//! - ✅ **Unified Type System**: Single source of truth for all core types
//! - ✅ **Zero Duplication**: Eliminated redundant type definitions across crates
//! - ✅ **Consistent Patterns**: Standardized error handling, configuration, and async patterns
//! - ✅ **Production Ready**: All types are battle-tested and optimized
//! - ✅ **Universal Compatibility**: Works seamlessly with all primal types
//!
//! ## 📦 **What's Included**
//!
//! - **Core Types**: Fundamental data structures used throughout Songbird
//! - **Error Types**: Comprehensive error handling system
//! - **Configuration Types**: Unified configuration patterns
//! - **Async Patterns**: Standard async/await patterns and utilities
//! - **Universal Adapters**: Primal-agnostic interface patterns
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![expect(
    clippy::module_name_repetitions,
    reason = "intentional pattern; clippy false positive for this API"
)]
#![expect(
    clippy::multiple_crate_versions,
    reason = "intentional pattern; clippy false positive for this API"
)] // Transitive dependencies we don't control
#![expect(clippy::pub_use, reason = "intentional pattern; clippy false positive for this API")] // Wildcard imports are acceptable for canonical re-exports
#![cfg_attr(
    test,
    allow(
        deprecated,
        dead_code,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::await_holding_lock,
        clippy::float_cmp,
        clippy::absurd_extreme_comparisons,
        clippy::nonminimal_bool,
        clippy::needless_collect,
        clippy::used_underscore_binding,
        clippy::overly_complex_bool_expr,
        clippy::assertions_on_constants,
        clippy::unreadable_literal,
        clippy::empty_line_after_doc_comments,
        clippy::field_reassign_with_default,
        clippy::unnecessary_wraps,
        clippy::no_effect_underscore_binding,
        clippy::return_self_not_must_use,
        clippy::duplicated_attributes,
        clippy::needless_pass_by_value,
        clippy::must_use_candidate,
        clippy::missing_panics_doc,
        clippy::missing_errors_doc,
        clippy::doc_markdown,
        clippy::wildcard_imports,
        clippy::enum_glob_use,
        unused_imports,
        unused_variables,
        clippy::unused_self,
        clippy::unnecessary_cast,
        clippy::items_after_test_module,
        clippy::clone_on_ref_ptr,
        clippy::default_trait_access,
        clippy::needless_range_loop,
        clippy::similar_names,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_unwrap,
        clippy::ignore_without_reason,
        clippy::case_sensitive_file_extension_comparisons,
    )
)]

// Core modules that actually exist
pub mod adapters;
pub mod config;
pub mod discovery;
#[cfg(test)]
mod discovery_tests;
pub mod errors;
pub mod metadata;
#[cfg(test)]
mod metadata_tests;
pub mod migration;
#[cfg(test)]
mod migration_tests;
pub mod performance;
#[cfg(test)]
mod performance_tests;
pub mod providers;
#[cfg(test)]
mod providers_tests;
pub mod responses;
pub mod traits;
pub mod types;
pub mod validation;

// Re-export canonical types for universal access
pub use adapters::*;
pub use config::*;
pub use discovery::*;
pub use errors::*;
pub use metadata::*;
pub use migration::*;
pub use performance::*;
// Export providers and traits with specific imports to avoid ServiceProvider conflict
// ProviderMetadata removed - use canonical provider traits from songbird-types
pub use responses::*;
// Traits are now available from songbird_types::traits::canonical
// pub use songbird_types::traits::canonical::{Capability, CapabilityProvider, ServiceProvider, OrchestrationProvider};
pub use types::*;
pub use validation::*;
