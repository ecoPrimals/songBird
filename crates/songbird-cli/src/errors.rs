//! CLI-specific error handling - Modernized to use canonical error system

use songbird_types::{SongbirdError, SongbirdResult};

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

// Use canonical result type throughout CLI
pub type CliResult<T> = SongbirdResult<T>;

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
