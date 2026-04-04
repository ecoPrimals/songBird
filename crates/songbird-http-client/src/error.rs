// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for Songbird HTTP Client

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in Songbird HTTP Client
#[derive(Error, Debug)]
pub enum Error {
    /// Security / crypto provider RPC communication error
    #[error("Security provider RPC error: {0}")]
    SecurityProviderRpc(String),

    /// Deprecated alias for [`Error::SecurityProviderRpc`].
    #[deprecated(note = "Use SecurityProviderRpc")]
    #[error("Security provider RPC error: {0}")]
    BearDogRpc(String),

    /// Crypto provider delegation required but not available for this operation
    #[error("Crypto provider unavailable: {0}")]
    CryptoUnavailable(String),

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
        Self::Hyper(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use std::io;

    #[test]
    fn error_display_includes_variant_context() {
        let e = Error::InvalidUrl("not-a-url".into());
        assert!(e.to_string().contains("Invalid URL"));
        assert!(e.to_string().contains("not-a-url"));
    }

    #[test]
    fn io_error_converts_via_from() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let e: Error = io_err.into();
        assert!(matches!(e, Error::Io(_)));
    }

    #[test]
    fn timeout_variant_display() {
        assert_eq!(Error::Timeout.to_string(), "Request timeout");
    }

    #[test]
    fn anyhow_converts_to_other() {
        let e: Error = anyhow::anyhow!("wrapped").into();
        assert!(matches!(e, Error::Other(s) if s.contains("wrapped")));
    }
}
