//! CLI-specific error handling

use songbird_types::{SongbirdError, SongbirdResult};

/// CLI-specific error types
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Command error: {0}")]
    CommandError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

/// CLI result type (uses canonical SongbirdResult, converts via From trait)
/// For internal CLI operations that need CliError, use Result<T, CliError> directly
pub type CliResult<T> = Result<T, CliError>;

impl From<CliError> for SongbirdError {
    fn from(cli_error: CliError) -> Self {
        match cli_error {
            CliError::CommandError(msg) => Self::configuration(format!("CLI command error: {msg}")),
            CliError::ConfigError(msg) => {
                Self::configuration(format!("CLI configuration error: {msg}"))
            }
            CliError::NetworkError(msg) => Self::network(format!("CLI network error: {msg}")),
            CliError::SerializationError(msg) => {
                Self::configuration(format!("CLI serialization error: {msg}"))
            }
            CliError::IoError(e) => Self::configuration(format!("CLI IO error: {e}")),
        }
    }
}
