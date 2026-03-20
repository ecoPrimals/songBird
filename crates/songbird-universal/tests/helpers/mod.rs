// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Shared test helpers and fixtures
//!
//! This module provides common test utilities to reduce duplication
//! and ensure consistent test setup across unified adapter tests.

#![expect(clippy::unwrap_used, reason = "test assertions and harness ergonomics")]
pub mod unified_adapter_fixtures;
// Re-export commonly used items
pub use unified_adapter_fixtures::*;
