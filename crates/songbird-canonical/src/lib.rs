//! # Songbird Canonical Type System
//!
//! This crate provides the **single, canonical type system** for the entire Songbird ecosystem.
//! All crates MUST use these types to ensure consistency, AI-first compatibility, and future-proofing.
//!
//! ## Core Principles
//!
//! 1. **Single Source of Truth**: Only one way to represent each concept
//! 2. **AI-First Design**: All types optimized for AI interaction and automation
//! 3. **Compile-Time Safety**: Maximum compile-time guarantees, minimal runtime errors
//! 4. **Future-Proof**: Extensible design that grows with the ecosystem
//! 5. **Zero Ambiguity**: Clear, unambiguous APIs that prevent misuse
//!
//! ## Usage
//!
//! ```rust,no_run
//! // Example of canonical function signatures
//! struct MyService;
//!
//! impl MyService {
//!     // THE ONLY way to define functions in Songbird
//!     async fn my_service_function(&self) -> SongbirdResult<String> {
//!         Ok("Hello, World!".to_string())
//!     }
//! }
//! ```

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod errors;
pub mod metadata;
pub mod migration;
pub mod responses;
pub mod types;
pub mod validation;

// Re-export the canonical types
pub use errors::*;
pub use metadata::*;
pub use migration::*;
pub use types::*;
pub use validation::*;

// Note: SongbirdResponse moved to songbird-errors to break circular dependency
// Import it from there when needed

// Note: Result types moved to songbird-errors to break circular dependency
// Import SongbirdResult from songbird-errors when needed

/// Helper function to create successful results
///
/// # Errors
///
/// This function never returns an error - it always creates a successful result.
#[allow(clippy::result_unit_err)] // This is a helper function for compatibility
pub const fn success_result<T>(data: T) -> T {
    data
}
