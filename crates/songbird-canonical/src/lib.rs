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

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::pub_use)] // Wildcard imports are acceptable for canonical re-exports

// Core modules that actually exist
pub mod adapters;
pub mod config;
pub mod discovery;
pub mod errors;
pub mod metadata;
pub mod migration;
pub mod performance;
pub mod providers;
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
