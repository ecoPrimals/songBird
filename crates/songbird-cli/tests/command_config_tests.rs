// SPDX-License-Identifier: AGPL-3.0-or-later
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

//! Tests for config command

use songbird_cli::cli::commands::config::{ConfigCommand, handle_config_command};

#[tokio::test]
async fn test_config_show_basic() {
    let command = ConfigCommand::Show {
        detailed: false,
    };
    let result = handle_config_command(command).await;
    assert!(result.is_ok(), "Show config should succeed");
}

#[tokio::test]
async fn test_config_show_detailed() {
    let command = ConfigCommand::Show {
        detailed: true,
    };
    let result = handle_config_command(command).await;
    assert!(result.is_ok(), "Show detailed config should succeed");
}

#[tokio::test]
async fn test_config_set() {
    let command = ConfigCommand::Set {
        key: "gaming_mode".to_string(),
        value: "enabled".to_string(),
    };
    let result = handle_config_command(command).await;
    assert!(result.is_ok(), "Set config should succeed");
}

#[tokio::test]
async fn test_config_reset_without_confirmation() {
    let command = ConfigCommand::Reset {
        yes: false,
    };
    let result = handle_config_command(command).await;
    assert!(result.is_ok(), "Reset without confirmation should succeed (but not reset)");
}

#[tokio::test]
async fn test_config_reset_with_confirmation() {
    let command = ConfigCommand::Reset {
        yes: true,
    };
    let result = handle_config_command(command).await;
    assert!(result.is_ok(), "Reset with confirmation should succeed");
}

#[tokio::test]
async fn test_config_set_various_keys() {
    let test_cases = vec![("port", "8080"), ("host", "localhost"), ("enabled", "true")];

    for (key, value) in test_cases {
        let command = ConfigCommand::Set {
            key: key.to_string(),
            value: value.to_string(),
        };
        let result = handle_config_command(command).await;
        assert!(result.is_ok(), "Setting {key} should succeed");
    }
}
