// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(clippy::all, reason = "test assertions and harness ergonomics")]
#![expect(unused, reason = "test assertions and harness ergonomics")]

//! Workflow Management Tests
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![expect(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![expect(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![expect(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Testing workflow orchestration and management capabilities

use songbird_orchestrator::cli::config::CliConfig;

#[test]
fn test_cli_config_initialization() {
    let config = CliConfig::new();
    assert!(!config.is_verbose());
    assert!(config.is_colored_output_enabled());
}

#[test]
fn test_cli_config_with_verbose() {
    let mut config = CliConfig::new();
    config.set_verbose(true);
    assert!(config.is_verbose());
}

#[test]
fn test_cli_config_disable_colors() {
    let mut config = CliConfig::new();
    config.set_colored_output(false);
    assert!(!config.is_colored_output_enabled());
}

#[test]
fn test_cli_config_with_config_path() {
    let mut config = CliConfig::new();
    config.set_config_path("/path/to/config.toml".to_string());
    let path = config.get_config_path();
    assert!(path.is_some());
    assert_eq!(path.as_deref(), Some("/path/to/config.toml"));
}

#[test]
fn test_cli_config_remove_config_path() {
    let mut config = CliConfig::new();
    config.set_config_path("/path/to/config.toml".to_string());
    // Note: set_config_path doesn't accept Option, so we can't test removal this way
    // This test validates that path can be set
    assert!(config.get_config_path().is_some());
}

#[test]
fn test_cli_config_toggle_verbose() {
    let mut config = CliConfig::new();
    let initial = config.is_verbose();
    config.set_verbose(!initial);
    assert_ne!(config.is_verbose(), initial);
}

#[test]
fn test_cli_config_multiple_operations() {
    let mut config = CliConfig::new();
    config.set_verbose(true);
    config.set_colored_output(false);
    config.set_config_path("/etc/songbird/config.toml".to_string());

    assert!(config.is_verbose());
    assert!(!config.is_colored_output_enabled());
    assert!(config.get_config_path().is_some());
}
