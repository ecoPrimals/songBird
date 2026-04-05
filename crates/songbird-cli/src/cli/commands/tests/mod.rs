// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Allow common test patterns - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unnecessary_wraps, reason = "test assertions and harness ergonomics")]
#![allow(clippy::field_reassign_with_default, reason = "test assertions and harness ergonomics")]
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]

//! Tests for CLI commands
//!
//! Comprehensive test coverage for all CLI commands

#[cfg(test)]
mod version_tests;
mod config_tests;
