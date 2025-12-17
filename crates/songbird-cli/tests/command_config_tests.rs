// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for config command

use songbird_cli::cli::commands::config::{handle_config_command, ConfigCommand};

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
