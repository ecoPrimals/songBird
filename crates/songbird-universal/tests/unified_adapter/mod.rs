// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Allow common test patterns - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unnecessary_wraps, reason = "test assertions and harness ergonomics")]
#![allow(clippy::field_reassign_with_default, reason = "test assertions and harness ergonomics")]
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]

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

