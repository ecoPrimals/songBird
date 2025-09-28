use CanonicalSongbirdConfig;
//! Comprehensive Error Tests for Songbird Orchestrator
//!
//! This test suite covers error types, error handling, validation,
//! and error recovery mechanisms.
//!
//! ## Refactored Test Organization
//!
//! The comprehensive error tests are organized into focused modules:
//! - `error_scenarios` - Complex error scenarios and comprehensive async tests
//! - `error_types` - Individual error type tests for all SongbirdError variants
//! - `error_conversion` - Error conversion and helper function tests
//! - `config_validator` - Configuration validation tests
//! - `error_traits` - Error trait implementation tests

pub mod error_scenarios;
pub mod error_types;
pub mod error_conversion;
pub mod config_validator;
pub mod error_traits;

// Re-export for convenience
pub use error_scenarios::*;
pub use error_types::*;
pub use error_conversion::*;
pub use config_validator::*;
pub use error_traits::*; 