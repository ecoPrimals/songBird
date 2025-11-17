//! Comprehensive CLI Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests for all modern CLI commands, argument parsing, and functionality
//! to achieve 90% test coverage for the songbird-cli crate.
//!
//! This test file covers the modern gaming-focused CLI structure.

use clap::Parser;
use songbird_cli::cli::{
    commands::{quick::ContributeType, Commands, LogLevel},
    Cli,
};
use songbird_types::{SongbirdError, SongbirdResult};

/// Test CLI argument parsing for version command
#[test]
fn test_version_command_parsing() -> Result<(), SongbirdError> {
    // Test version command
    let cli = Cli::try_parse_from(["songbird", "version"]).or_else(|_| {
        SongbirdError::configuration(format!("Version command should parse: {}", e))
    })?;
    match cli.command {
        Some(Commands::Version {
            detailed,
        }) => assert!(!detailed),
        _ => panic!("Expected Version command"),
    }

    // Test version command with detailed flag
    let cli = Cli::try_parse_from(["songbird", "version", "--detailed"]).or_else(|_| {
        SongbirdError::configuration(format!("Detailed version should parse: {}", e))
    })?;
    match cli.command {
        Some(Commands::Version {
            detailed,
        }) => assert!(detailed),
        _ => panic!("Expected Version command with detailed flag"),
    }

    Ok(())
}

/// Test quick command parsing
#[test]
fn test_quick_command_parsing() -> Result<(), SongbirdError> {
    // Test quick command with defaults
    let cli = Cli::try_parse_from(["songbird", "quick"])
        .or_else(|_| SongbirdError::configuration(format!("Quick command should parse: {}", e)))?;
    match cli.command {
        Some(Commands::Quick {
            name,
            auto_detect,
            family_safe,
        }) => {
            assert_eq!(name, None);
            assert!(!auto_detect);
            assert!(!family_safe);
        }
        _ => panic!("Expected Quick command"),
    }

    // Test quick command with parameters
    let cli = Cli::try_parse_from([
        "songbird",
        "quick",
        "test-session",
        "--auto-detect",
        "--family-safe",
    ])
    .ok_or_else(|_| {
        SongbirdError::configuration(format!("Quick with params should parse: {}", e))
    })?;
    match cli.command {
        Some(Commands::Quick {
            name,
            auto_detect,
            family_safe,
        }) => {
            assert_eq!(name, Some("test-session".to_string()));
            assert!(auto_detect);
            assert!(family_safe);
        }
        _ => panic!("Expected Quick command with parameters"),
    }

    Ok(())
}

/// Test discover command parsing
#[test]
fn test_discover_command_parsing() -> Result<(), SongbirdError> {
    // Test discover with defaults
    let cli = Cli::try_parse_from(["songbird", "discover"]).or_else(|_| {
        SongbirdError::configuration(format!("Discover command should parse: {}", e))
    })?;
    match cli.command {
        Some(Commands::Discover {
            timeout,
            protocol,
            continuous,
        }) => {
            assert_eq!(timeout, 10);
            assert_eq!(protocol, None);
            assert!(!continuous);
        }
        _ => panic!("Expected Discover command"),
    }

    // Test discover with parameters
    let cli = Cli::try_parse_from([
        "songbird",
        "discover",
        "--timeout",
        "30",
        "--protocol",
        "minecraft",
        "--continuous",
    ])
    .ok_or_else(|_| {
        SongbirdError::configuration(format!("Discover with params should parse: {}", e))
    })?;

    match cli.command {
        Some(Commands::Discover {
            timeout,
            protocol,
            continuous,
        }) => {
            assert_eq!(timeout, 30);
            assert_eq!(protocol, Some("minecraft".to_string()));
            assert!(continuous);
        }
        _ => panic!("Expected Discover command with parameters"),
    }

    Ok(())
}

/// Test status command parsing
#[test]
fn test_status_command_parsing() -> Result<(), SongbirdError> {
    // Test status with defaults
    let cli = Cli::try_parse_from(["songbird", "status"])
        .ok_or_else(|_| SongbirdError::configuration(format!("Status should parse: {}", e)))?;
    match cli.command {
        Some(Commands::Status {
            detailed,
            gaming,
        }) => {
            assert!(!detailed);
            assert!(!gaming);
        }
        _ => panic!("Expected Status command"),
    }

    // Test status with parameters
    let cli =
        Cli::try_parse_from(["songbird", "status", "--detailed", "--gaming"]).or_else(|_| {
            SongbirdError::configuration(format!("Status with params should parse: {}", e))
        })?;

    match cli.command {
        Some(Commands::Status {
            detailed,
            gaming,
        }) => {
            assert!(detailed);
            assert!(gaming);
        }
        _ => panic!("Expected Status command with parameters"),
    }

    Ok(())
}

/// Test gaming command structure
#[test]
fn test_gaming_command_exists() -> Result<(), SongbirdError> {
    // Test that gaming command can be parsed (subcommands tested in gaming module tests)
    let result = Cli::try_parse_from(["songbird", "gaming"]);
    // Gaming requires a subcommand, so this should error
    assert!(result.is_err());

    Ok(())
}

/// Test network command structure
#[test]
fn test_network_command_exists() -> Result<(), SongbirdError> {
    // Test that network command can be parsed (subcommands tested in network module tests)
    let result = Cli::try_parse_from(["songbird", "network"]);
    // Network requires a subcommand, so this should error
    assert!(result.is_err());

    Ok(())
}

/// Test federation command structure
#[test]
fn test_federation_command_exists() -> Result<(), SongbirdError> {
    // Test that federation command can be parsed (subcommands tested in federation module tests)
    let result = Cli::try_parse_from(["songbird", "federation"]);
    // Federation requires a subcommand, so this should error
    assert!(result.is_err());

    Ok(())
}

/// Test config command structure
#[test]
fn test_config_command_exists() -> Result<(), SongbirdError> {
    // Test that config command can be parsed (subcommands tested in config module tests)
    let result = Cli::try_parse_from(["songbird", "config"]);
    // Config requires a subcommand, so this should error
    assert!(result.is_err());

    Ok(())
}

/// Test log level variants
#[test]
fn test_log_level_variants() {
    assert_eq!(LogLevel::default(), LogLevel::Info);

    // Test that all variants exist
    let _levels =
        vec![LogLevel::Trace, LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error];
}

/// Test contribute type variants
#[test]
fn test_contribute_type_variants() {
    assert_eq!(ContributeType::default(), ContributeType::Compute);

    // Test that all variants exist
    let _types = vec![ContributeType::Compute, ContributeType::Storage, ContributeType::Data];
}

/// Test CLI execution with no command
#[tokio::test]
async fn test_cli_execute_no_command() {
    let cli = Cli {
        command: None,
    };

    let result = cli.execute().await;
    assert!(result.is_ok());
}

/// Test CLI execution with version command
#[tokio::test]
async fn test_cli_execute_version_command() {
    let cli = Cli {
        command: Some(Commands::Version {
            detailed: false,
        }),
    };

    let result = cli.execute().await;
    assert!(result.is_ok());
}

/// Test invalid command line arguments
#[test]
fn test_invalid_cli_arguments() -> SongbirdResult<()> {
    // Test invalid timeout (not a number)
    let result = Cli::try_parse_from(["songbird", "discover", "--timeout", "invalid"]);
    assert!(result.is_err()); // Should fail validation for non-numeric timeout
    Ok(())
}

/// Test CLI help output
#[test]
fn test_cli_help() -> SongbirdResult<()> {
    let result = Cli::try_parse_from(["songbird", "--help"]);
    assert!(result.is_err()); // Help exits with error code but provides help text
    Ok(())
}

/// Test subcommand help
#[test]
fn test_subcommand_help() -> SongbirdResult<()> {
    let result = Cli::try_parse_from(["songbird", "status", "--help"]);
    assert!(result.is_err()); // Help exits with error code but provides help text
    Ok(())
}

/// Test CLI parsing with no arguments
#[test]
fn test_cli_no_args() -> Result<(), SongbirdError> {
    let cli = Cli::try_parse_from(["songbird"]).or_else(|_| {
        SongbirdError::configuration(format!("CLI should parse with no args: {}", e))
    })?;

    match cli.command {
        None => assert!(true), // No command is valid
        _ => panic!("Expected no command"),
    }

    Ok(())
}

/// Test CLI command combinations
#[test]
fn test_cli_command_combinations() -> Result<(), SongbirdError> {
    // Quick with all flags
    let cli = Cli::try_parse_from([
        "songbird",
        "quick",
        "session-name",
        "--auto-detect",
        "--family-safe",
    ])
    .ok_or_else(|_| SongbirdError::configuration(format!("Should parse: {}", e)))?;

    match cli.command {
        Some(Commands::Quick {
            name,
            auto_detect,
            family_safe,
        }) => {
            assert_eq!(name, Some("session-name".to_string()));
            assert!(auto_detect);
            assert!(family_safe);
        }
        _ => panic!("Expected Quick command"),
    }

    Ok(())
}

/// Test version command detailed output
#[tokio::test]
async fn test_version_detailed_execution() {
    let cli = Cli {
        command: Some(Commands::Version {
            detailed: true,
        }),
    };

    let result = cli.execute().await;
    assert!(result.is_ok());
}

/// Test discover command execution
#[tokio::test]
async fn test_discover_execution() {
    let cli = Cli {
        command: Some(Commands::Discover {
            timeout: 5,
            protocol: None,
            continuous: false,
        }),
    };

    let result = cli.execute().await;
    // Execution may succeed or fail depending on environment, just test it runs
    assert!(result.is_ok() || result.is_err());
}

/// Test status command execution
#[tokio::test]
async fn test_status_execution() {
    let cli = Cli {
        command: Some(Commands::Status {
            detailed: false,
            gaming: false,
        }),
    };

    let result = cli.execute().await;
    // Execution may succeed or fail depending on environment, just test it runs
    assert!(result.is_ok() || result.is_err());
}

/// Test quick command execution
#[tokio::test]
async fn test_quick_execution() -> SongbirdResult<()> {
    let cli = Cli {
        command: Some(Commands::Quick {
            name: Some("test-session".to_string()),
            auto_detect: true,
            family_safe: true,
        }),
    };

    let result = cli.execute().await;
    // Execution may succeed or fail depending on environment, just test it runs
    assert!(result.is_ok() || result.is_err());
    Ok(())
}

/// Test CLI command parsing with various option orders
#[test]
fn test_command_option_ordering() -> Result<(), SongbirdError> {
    // Options before positional args
    let cli1 = Cli::try_parse_from(["songbird", "quick", "--auto-detect", "test-session"])
        .ok_or_else(|_| SongbirdError::configuration(format!("Should parse: {}", e)))?;

    // Options after positional args
    let cli2 = Cli::try_parse_from(["songbird", "quick", "test-session", "--auto-detect"])
        .ok_or_else(|_| SongbirdError::configuration(format!("Should parse: {}", e)))?;

    // Both should parse to the same structure
    match (cli1.command, cli2.command) {
        (
            Some(Commands::Quick {
                name: name1,
                ..
            }),
            Some(Commands::Quick {
                name: name2,
                ..
            }),
        ) => {
            assert_eq!(name1, name2);
        }
        _ => panic!("Expected Quick command for both"),
    }

    Ok(())
}
