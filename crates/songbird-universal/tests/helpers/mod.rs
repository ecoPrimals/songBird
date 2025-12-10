//! Shared test helpers and fixtures
//!
//! This module provides common test utilities to reduce duplication
//! and ensure consistent test setup across unified adapter tests.

#![allow(clippy::unwrap_used)]
pub mod unified_adapter_fixtures;
// Re-export commonly used items
pub use unified_adapter_fixtures::*;
