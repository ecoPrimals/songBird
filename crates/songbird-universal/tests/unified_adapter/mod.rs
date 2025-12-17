// Allow common test patterns - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]

//! Unified Universal Adapter Test Module
//!
//! **Intelligent Organization**: Tests organized by functional domain, not arbitrary splitting
//!
//! This module replaces the monolithic `unified_adapter_core_tests.rs` (1231 lines)
//! with a well-structured module hierarchy that reflects the adapter's architecture.

/// Test fixtures and common utilities shared across all adapter tests
pub mod fixtures;

/// Adapter creation, construction, and factory function tests
///
/// Validates:
/// - Constructor patterns (`new()`, `with_config()`)
/// - Factory functions (`create_universal_adapter()`)
/// - Initialization consistency
/// - Memory efficiency
pub mod creation;

/// Configuration management and validation tests
///
/// Validates:
/// - Default configuration values
/// - Custom configuration scenarios
/// - Configuration validation rules  
/// - Builder patterns
/// - Edge cases (zero timeouts, extreme values)
pub mod configuration;

/// Runtime capability discovery and routing tests
///
/// Validates:
/// - Capability provider discovery
/// - Request routing logic
/// - Registry operations
/// - Empty state handling
pub mod capabilities;

/// Concurrent operations and async behavior tests
///
/// Validates:
/// - Concurrent discovery operations
/// - Async method availability
/// - Thread safety
/// - Performance under load
pub mod concurrency;

