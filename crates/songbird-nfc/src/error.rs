//! NFC error types

use thiserror::Error;

/// Result type for NFC operations
pub type Result<T> = std::result::Result<T, NfcError>;

/// NFC protocol errors
#[derive(Debug, Error)]
pub enum NfcError {
    /// Protocol version mismatch
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),
    
    /// Invalid message type
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),
    
    /// Payload too large
    #[error("Payload too large: {0} bytes (max: {1})")]
    PayloadTooLarge(usize, usize),
    
    /// Malformed frame
    #[error("Malformed frame: {0}")]
    MalformedFrame(String),
    
    /// Crypto error (BearDog delegation)
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// Platform error (NFC hardware/driver)
    #[error("Platform error: {0}")]
    Platform(String),
    
    /// Timeout
    #[error("Operation timed out")]
    Timeout,
    
    /// Connection lost
    #[error("NFC connection lost")]
    ConnectionLost,
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
