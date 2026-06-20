// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CLI-specific error handling - Modernized to use canonical error system

#![allow(missing_docs, reason = "CLI error variants mirror user-facing messages")]

use songbird_types::SongbirdError;

/// CLI-specific error types - Modernized to integrate with `SongbirdError`
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command error: {command} - {message}")]
    Command {
        command: String,
        message: String,
    },

    #[error("Configuration error: {message}")]
    Config {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },

    #[error("Network error: {message}")]
    Network {
        message: String,
        interface: Option<String>,
        suggestion: Option<String>,
    },

    #[error("User cancelled operation")]
    UserCancelled,

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error")]
    Io(#[from] std::io::Error),
}

// Re-export canonical result type for CLI convenience
pub use songbird_types::errors::SongbirdResult;

impl From<CliError> for SongbirdError {
    fn from(cli_error: CliError) -> Self {
        match cli_error {
            CliError::Command {
                command,
                message,
            } => Self::Service {
                service: String::from("cli"),
                message: format!("{command}: {message}"),
                suggested_alternatives: vec![String::from("--help")],
                recovery_actions: vec![String::from("Check command syntax")],
            },
            CliError::Config {
                message,
                field,
                suggestion,
            } => Self::Configuration {
                message,
                field,
                suggestion,
            },
            CliError::Network {
                message,
                interface,
                suggestion,
            } => Self::Network {
                message: format!("CLI network error: {message}"),
                interface,
                suggestion,
            },
            CliError::UserCancelled => Self::Configuration {
                message: String::from("Operation cancelled by user"),
                field: Some(String::from("user_input")),
                suggestion: Some(String::from("Try again or use --force to skip confirmations")),
            },
            CliError::Serialization(e) => Self::Serialization {
                message: e.to_string(),
                format: Some(String::from("json")),
                debug_info: None,
            },
            CliError::Io(e) => Self::Configuration {
                message: format!("IO error: {e}"),
                field: Some(String::from("file_system")),
                suggestion: Some(String::from("Check file permissions and paths")),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_error_command_display() {
        let error = CliError::Command {
            command: String::from("test"),
            message: String::from("failed"),
        };
        let display = format!("{error}");
        assert!(display.contains("Command error"));
        assert!(display.contains("test"));
        assert!(display.contains("failed"));
    }

    #[test]
    fn test_cli_error_config_display() {
        let error = CliError::Config {
            message: String::from("invalid config"),
            field: Some(String::from("port")),
            suggestion: Some(String::from("use port 8080")),
        };
        let display = format!("{error}");
        assert!(display.contains("Configuration error"));
        assert!(display.contains("invalid config"));
    }

    #[test]
    fn test_cli_error_network_display() {
        let error = CliError::Network {
            message: String::from("connection failed"),
            interface: Some(String::from("eth0")),
            suggestion: Some(String::from("check network")),
        };
        let display = format!("{error}");
        assert!(display.contains("Network error"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_cli_error_user_cancelled() {
        let error = CliError::UserCancelled;
        let display = format!("{error}");
        assert_eq!(display, "User cancelled operation");
    }

    #[test]
    fn test_cli_error_to_songbird_error_command() {
        let cli_error = CliError::Command {
            command: String::from("init"),
            message: String::from("missing args"),
        };
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("cli"));
    }

    #[test]
    fn test_cli_error_to_songbird_error_config() {
        let cli_error = CliError::Config {
            message: String::from("bad config"),
            field: Some(String::from("timeout")),
            suggestion: Some(String::from("use 30s")),
        };
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("bad config"));
    }

    #[test]
    fn test_cli_error_to_songbird_error_network() {
        let cli_error = CliError::Network {
            message: String::from("timeout"),
            interface: None,
            suggestion: None,
        };
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("timeout"));
    }

    #[test]
    fn test_cli_error_to_songbird_error_user_cancelled() {
        let cli_error = CliError::UserCancelled;
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("cancelled"));
    }

    #[test]
    fn test_cli_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cli_error: CliError = io_error.into();
        assert!(matches!(cli_error, CliError::Io(_)));
    }

    #[test]
    fn test_cli_error_from_serde_error() {
        let json_str = "invalid json {]";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        assert!(result.is_err());
        let serde_error = result.unwrap_err();
        let cli_error: CliError = serde_error.into();
        assert!(matches!(cli_error, CliError::Serialization(_)));
    }

    #[test]
    fn test_cli_error_config_with_all_fields() {
        let error = CliError::Config {
            message: String::from("Port must be between 1024 and 65535"),
            field: Some(String::from("server.port")),
            suggestion: Some(String::from("Try using port 8080")),
        };

        let songbird_error: SongbirdError = error.into();
        if let SongbirdError::Configuration {
            message,
            field,
            suggestion,
        } = songbird_error
        {
            assert_eq!(message, "Port must be between 1024 and 65535");
            assert_eq!(field, Some(String::from("server.port")));
            assert_eq!(suggestion, Some(String::from("Try using port 8080")));
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[test]
    fn test_cli_error_network_with_interface() {
        let error = CliError::Network {
            message: String::from("Cannot bind to interface"),
            interface: Some(String::from("wlan0")),
            suggestion: Some(String::from("Check interface status with 'ip link'")),
        };

        let songbird_error: SongbirdError = error.into();
        if let SongbirdError::Network {
            message,
            interface,
            suggestion,
        } = songbird_error
        {
            assert!(message.contains("Cannot bind to interface"));
            assert_eq!(interface, Some(String::from("wlan0")));
            assert_eq!(suggestion, Some(String::from("Check interface status with 'ip link'")));
        } else {
            panic!("Expected Network error");
        }
    }
}
