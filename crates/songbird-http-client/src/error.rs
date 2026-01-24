//! Error types for Songbird HTTP Client

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in Songbird HTTP Client
#[derive(Error, Debug)]
pub enum Error {
    /// BearDog RPC communication error
    #[error("BearDog RPC error: {0}")]
    BearDogRpc(String),

    /// TLS handshake error
    #[error("TLS handshake failed: {0}")]
    TlsHandshake(String),

    /// TLS record layer error
    #[error("TLS record layer error: {0}")]
    TlsRecord(String),

    /// TLS alert received from server
    #[error("TLS alert: {0}")]
    TlsAlert(String),

    /// HTTP protocol error
    #[error("HTTP protocol error: {0}")]
    HttpProtocol(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    /// Hyper error
    #[error("Hyper error: {0}")]
    Hyper(String),

    /// Timeout error
    #[error("Request timeout")]
    Timeout,

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::Hyper(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}
