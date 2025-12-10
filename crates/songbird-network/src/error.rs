//! Network error types

use thiserror::Error;

/// Network operation result type
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Network errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// WireGuard configuration error
    #[error("WireGuard configuration error: {0}")]
    WireGuardConfig(String),

    /// WireGuard tunnel error
    #[error("WireGuard tunnel error: {0}")]
    WireGuardTunnel(String),

    /// TLS configuration error
    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    /// Certificate error
    #[error("Certificate error: {0}")]
    Certificate(String),

    /// Key generation error
    #[error("Key generation error: {0}")]
    KeyGeneration(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Peer connection error
    #[error("Peer connection error: {0}")]
    PeerConnection(String),

    /// Tunnel not found
    #[error("Tunnel not found: {0}")]
    TunnelNotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

