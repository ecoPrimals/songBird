//! Error types for lineage relay system

use thiserror::Error;

/// Result type for lineage relay operations
pub type Result<T> = std::result::Result<T, LineageRelayError>;

/// Errors that can occur in the lineage relay system
#[derive(Debug, Error)]
pub enum LineageRelayError {
    /// Lineage verification failed
    #[error("Lineage verification failed: {0}")]
    LineageVerificationFailed(String),

    /// No ancestors available for relay
    #[error("No ancestors available for relay: {0}")]
    NoRelayAvailable(String),

    /// `BirdSong` encryption/decryption failed
    #[error("BirdSong operation failed: {0}")]
    BirdSongError(String),

    /// Relay authorization denied
    #[error("Relay authorization denied: {0}")]
    RelayDenied(String),

    /// Direct connection failed (expected, not an error condition)
    #[error("Direct connection not possible: {0}")]
    DirectConnectionFailed(String),

    /// Relay session error
    #[error("Relay session error: {0}")]
    SessionError(String),
    
    /// Relay session not found
    #[error("Relay session not found: {0}")]
    SessionNotFound(String),

    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Invalid protocol message
    #[error("Invalid protocol: {0}")]
    InvalidProtocol(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<String> for LineageRelayError {
    fn from(msg: String) -> Self {
        Self::Other(msg)
    }
}

impl From<&str> for LineageRelayError {
    fn from(msg: &str) -> Self {
        Self::Other(msg.to_string())
    }
}
