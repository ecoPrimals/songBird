// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Tests for version command

use songbird_cli::cli::commands::version::{
    execute_version_command, show_detailed_version, show_simple_version,
};

#[tokio::test]
async fn test_execute_version_simple() {
    let result = execute_version_command(false).await;
    assert!(result.is_ok(), "Simple version command should succeed");
}

#[tokio::test]
async fn test_execute_version_detailed() {
    let result = execute_version_command(true).await;
    assert!(result.is_ok(), "Detailed version command should succeed");
}

#[tokio::test]
async fn test_show_simple_version() {
    let result = show_simple_version().await;
    assert!(result.is_ok(), "Show simple version should succeed");
}

#[tokio::test]
async fn test_show_detailed_version() {
    let result = show_detailed_version().await;
    assert!(result.is_ok(), "Show detailed version should succeed");
}
