// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for config command
//!
//! Phase 3 Test Coverage Expansion - CLI Commands
//! Target: 0% → 90%+ coverage for config.rs (42 lines)

use super::*;

// =============================================================================
// CONFIG COMMAND ENUM TESTS
// =============================================================================

#[test]
fn test_config_command_show_default() {
    let cmd = ConfigCommand::Show {
        detailed: false,
    };

    match cmd {
        ConfigCommand::Show {
            detailed,
        } => {
            assert!(!detailed);
        }
        _ => panic!("Expected Show variant"),
    }
}

#[test]
fn test_config_command_show_detailed() {
    let cmd = ConfigCommand::Show {
        detailed: true,
    };

    match cmd {
        ConfigCommand::Show {
            detailed,
        } => {
            assert!(detailed);
        }
        _ => panic!("Expected Show variant"),
    }
}

#[test]
fn test_config_command_set() {
    let cmd = ConfigCommand::Set {
        key: "gaming_mode".to_string(),
        value: "enabled".to_string(),
    };

    match cmd {
        ConfigCommand::Set {
            key,
            value,
        } => {
            assert_eq!(key, "gaming_mode");
            assert_eq!(value, "enabled");
        }
        _ => panic!("Expected Set variant"),
    }
}

#[test]
fn test_config_command_reset_no_confirm() {
    let cmd = ConfigCommand::Reset {
        yes: false,
    };

    match cmd {
        ConfigCommand::Reset {
            yes,
        } => {
            assert!(!yes);
        }
        _ => panic!("Expected Reset variant"),
    }
}

#[test]
fn test_config_command_reset_with_confirm() {
    let cmd = ConfigCommand::Reset {
        yes: true,
    };

    match cmd {
        ConfigCommand::Reset {
            yes,
        } => {
            assert!(yes);
        }
        _ => panic!("Expected Reset variant"),
    }
}

#[test]
fn test_config_command_clone() {
    let cmd = ConfigCommand::Show {
        detailed: true,
    };
    let cloned = cmd.clone();

    match (cmd, cloned) {
        (
            ConfigCommand::Show {
                detailed: d1,
            },
            ConfigCommand::Show {
                detailed: d2,
            },
        ) => {
            assert_eq!(d1, d2);
        }
        _ => panic!("Clone failed or wrong variant"),
    }
}

// =============================================================================
// HANDLE CONFIG COMMAND TESTS
// =============================================================================

#[tokio::test]
async fn test_handle_show_simple() {
    let cmd = ConfigCommand::Show {
        detailed: false,
    };
    let result = handle_config_command(cmd).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_show_detailed() {
    let cmd = ConfigCommand::Show {
        detailed: true,
    };
    let result = handle_config_command(cmd).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_set_config() {
    let cmd = ConfigCommand::Set {
        key: "target_latency".to_string(),
        value: "30ms".to_string(),
    };
    let result = handle_config_command(cmd).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_reset_no_confirm() {
    let cmd = ConfigCommand::Reset {
        yes: false,
    };
    let result = handle_config_command(cmd).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_reset_with_confirm() {
    let cmd = ConfigCommand::Reset {
        yes: true,
    };
    let result = handle_config_command(cmd).await;

    assert!(result.is_ok());
}

// =============================================================================
// SHOW CONFIG TESTS
// =============================================================================

#[tokio::test]
async fn test_show_config_simple() {
    let result = show_config(false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_config_detailed() {
    let result = show_config(true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_config_multiple_calls() {
    for _ in 0..3 {
        let result = show_config(false).await;
        assert!(result.is_ok());
    }
}

// =============================================================================
// SET CONFIG TESTS
// =============================================================================

#[tokio::test]
async fn test_set_config_gaming_mode() {
    let result = set_config("gaming_mode".to_string(), "enabled".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_config_target_latency() {
    let result = set_config("target_latency".to_string(), "50ms".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_config_auto_optimize() {
    let result = set_config("auto_optimize".to_string(), "true".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_config_various_keys() {
    let keys = vec![
        ("network.port_range", "27015-27030"),
        ("federation.auto_join", "false"),
        ("matchmaking.skill_based", "true"),
    ];

    for (key, value) in keys {
        let result = set_config(key.to_string(), value.to_string()).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_set_config_empty_value() {
    let result = set_config("test_key".to_string(), "".to_string()).await;
    assert!(result.is_ok());
}

// =============================================================================
// RESET CONFIG TESTS
// =============================================================================

#[tokio::test]
async fn test_reset_config_without_confirmation() {
    let result = reset_config(false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_config_with_confirmation() {
    let result = reset_config(true).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reset_config_multiple_times() {
    let result1 = reset_config(true).await;
    let result2 = reset_config(true).await;
    let result3 = reset_config(true).await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_config_workflow_show_set_show() {
    // Show config
    let show1 = show_config(false).await;
    assert!(show1.is_ok());

    // Set a value
    let set = set_config("test_key".to_string(), "test_value".to_string()).await;
    assert!(set.is_ok());

    // Show config again
    let show2 = show_config(false).await;
    assert!(show2.is_ok());
}

#[tokio::test]
async fn test_config_workflow_set_reset() {
    // Set multiple values
    let set1 = set_config("key1".to_string(), "value1".to_string()).await;
    let set2 = set_config("key2".to_string(), "value2".to_string()).await;

    assert!(set1.is_ok());
    assert!(set2.is_ok());

    // Reset with confirmation
    let reset = reset_config(true).await;
    assert!(reset.is_ok());
}

#[tokio::test]
async fn test_all_config_commands() {
    let commands = vec![
        ConfigCommand::Show {
            detailed: false,
        },
        ConfigCommand::Show {
            detailed: true,
        },
        ConfigCommand::Set {
            key: "test".to_string(),
            value: "value".to_string(),
        },
        ConfigCommand::Reset {
            yes: true,
        },
    ];

    for cmd in commands {
        let result = handle_config_command(cmd).await;
        assert!(result.is_ok());
    }
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[tokio::test]
async fn test_set_config_special_characters() {
    let result =
        set_config("key-with-dashes".to_string(), "value_with_underscores".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_config_long_values() {
    let long_value = "x".repeat(1000);
    let result = set_config("test_key".to_string(), long_value).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_config_operations() {
    let mut handles = vec![];

    for i in 0..10 {
        handles.push(tokio::spawn(async move {
            set_config(format!("key{}", i), format!("value{}", i)).await
        }));
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_reset_without_prior_changes() {
    // Reset should work even if no changes were made
    let result = reset_config(true).await;
    assert!(result.is_ok());
}
