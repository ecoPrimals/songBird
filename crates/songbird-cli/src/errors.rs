//! CLI-specific error handling - Modernized to use canonical error system

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
                service: "cli".to_string(),
                message: format!("{command}: {message}"),
                suggested_alternatives: vec!["--help".to_string()],
                recovery_actions: vec!["Check command syntax".to_string()],
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
                message: "Operation cancelled by user".to_string(),
                field: Some("user_input".to_string()),
                suggestion: Some("Try again or use --force to skip confirmations".to_string()),
            },
            CliError::Serialization(e) => Self::Serialization {
                message: e.to_string(),
                format: Some("json".to_string()),
                debug_info: None,
            },
            CliError::Io(e) => Self::Configuration {
                message: format!("IO error: {e}"),
                field: Some("file_system".to_string()),
                suggestion: Some("Check file permissions and paths".to_string()),
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
            command: "test".to_string(),
            message: "failed".to_string(),
        };
        let display = format!("{error}");
        assert!(display.contains("Command error"));
        assert!(display.contains("test"));
        assert!(display.contains("failed"));
    }

    #[test]
    fn test_cli_error_config_display() {
        let error = CliError::Config {
            message: "invalid config".to_string(),
            field: Some("port".to_string()),
            suggestion: Some("use port 8080".to_string()),
        };
        let display = format!("{error}");
        assert!(display.contains("Configuration error"));
        assert!(display.contains("invalid config"));
    }

    #[test]
    fn test_cli_error_network_display() {
        let error = CliError::Network {
            message: "connection failed".to_string(),
            interface: Some("eth0".to_string()),
            suggestion: Some("check network".to_string()),
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
            command: "init".to_string(),
            message: "missing args".to_string(),
        };
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("cli"));
    }

    #[test]
    fn test_cli_error_to_songbird_error_config() {
        let cli_error = CliError::Config {
            message: "bad config".to_string(),
            field: Some("timeout".to_string()),
            suggestion: Some("use 30s".to_string()),
        };
        let songbird_error: SongbirdError = cli_error.into();
        let display = format!("{songbird_error}");
        assert!(display.contains("bad config"));
    }

    #[test]
    fn test_cli_error_to_songbird_error_network() {
        let cli_error = CliError::Network {
            message: "timeout".to_string(),
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
            message: "Port must be between 1024 and 65535".to_string(),
            field: Some("server.port".to_string()),
            suggestion: Some("Try using port 8080".to_string()),
        };

        let songbird_error: SongbirdError = error.into();
        if let SongbirdError::Configuration {
            message,
            field,
            suggestion,
        } = songbird_error
        {
            assert_eq!(message, "Port must be between 1024 and 65535");
            assert_eq!(field, Some("server.port".to_string()));
            assert_eq!(suggestion, Some("Try using port 8080".to_string()));
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[test]
    fn test_cli_error_network_with_interface() {
        let error = CliError::Network {
            message: "Cannot bind to interface".to_string(),
            interface: Some("wlan0".to_string()),
            suggestion: Some("Check interface status with 'ip link'".to_string()),
        };

        let songbird_error: SongbirdError = error.into();
        if let SongbirdError::Network {
            message,
            interface,
            suggestion,
        } = songbird_error
        {
            assert!(message.contains("Cannot bind to interface"));
            assert_eq!(interface, Some("wlan0".to_string()));
            assert_eq!(suggestion, Some("Check interface status with 'ip link'".to_string()));
        } else {
            panic!("Expected Network error");
        }
    }
}
