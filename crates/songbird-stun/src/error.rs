//! STUN error types

use std::net::AddrParseError;
use thiserror::Error;

/// STUN result type
pub type StunResult<T> = Result<T, StunError>;

/// STUN error types
#[derive(Debug, Error)]
pub enum StunError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Address parse error
    #[error("Address parse error: {0}")]
    AddrParse(#[from] AddrParseError),

    /// Timeout error
    #[error("STUN request timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// Invalid STUN response
    #[error("Invalid STUN response: {0}")]
    InvalidResponse(String),

    /// STUN server error
    #[error("STUN server error: {0}")]
    ServerError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),
}

