// Allow common test patterns - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]

//! Tests for CLI commands
//!
//! Comprehensive test coverage for all CLI commands

#[cfg(test)]
mod version_tests;
mod config_tests;
