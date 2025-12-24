//! Error types for primal coordination

use thiserror::Error;

/// Result type for primal coordination operations
pub type Result<T> = std::result::Result<T, PrimalCoordinationError>;

/// Errors that can occur during primal coordination
#[derive(Debug, Error)]
pub enum PrimalCoordinationError {
    /// Primal or capability not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Failed to connect to a primal
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Communication error with primal
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Unexpected response from primal
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Error from primal
    #[error("Primal error: {0}")]
    PrimalError(String),

    /// No capable primal found for operation
    #[error("No capable primal found: {0}")]
    NoCapablePrimal(String),

    /// Internal coordination error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Discovery error
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("Error: {0}")]
    Other(#[from] anyhow::Error),
}

