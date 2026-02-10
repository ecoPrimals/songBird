//! QUIC error types

use thiserror::Error;

/// Result type for QUIC operations
pub type Result<T> = std::result::Result<T, QuicError>;

/// QUIC protocol errors
#[derive(Debug, Error)]
pub enum QuicError {
    /// Quinn protocol error
    #[error("QUIC protocol error: {0}")]
    Protocol(#[from] quinn::ConnectionError),

    /// Connect error
    #[error("Connect error: {0}")]
    Connect(#[from] quinn::ConnectError),

    /// Connection closed
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Stream error
    #[error("Stream error: {0}")]
    Stream(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// `BearDog` crypto error
    #[error("BearDog crypto error: {0}")]
    Crypto(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Address parse error
    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// Write error
    #[error("Write error: {0}")]
    WriteError(#[from] quinn::WriteError),

    /// Read error
    #[error("Read error: {0}")]
    ReadError(#[from] quinn::ReadError),

    /// Stream closed
    #[error("Stream closed: {0}")]
    ClosedStream(#[from] quinn::ClosedStream),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Not connected
    #[error("Not connected")]
    NotConnected,
}
