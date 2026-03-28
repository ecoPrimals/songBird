// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for Tor protocol implementation

use thiserror::Error;

/// Result type for Tor protocol operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in Tor protocol operations
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Crypto error (from `BearDog`)
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// `BearDog` crypto delegation required but unavailable or not yet wired
    #[error("BearDog crypto unavailable: {0}")]
    CryptoUnavailable(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    Consensus(String),

    /// Circuit error
    #[error("Circuit error: {0}")]
    Circuit(String),

    /// Stream error
    #[error("Stream error: {0}")]
    Stream(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::Error;
    use std::io;

    #[test]
    fn io_from_displays() {
        let e: Error = io::Error::new(io::ErrorKind::PermissionDenied, "denied").into();
        assert!(e.to_string().contains("I/O error"));
        assert!(e.to_string().contains("denied"));
    }

    #[test]
    fn string_variants_display() {
        assert!(Error::Http("h".into()).to_string().contains("HTTP"));
        assert!(Error::Network("n".into()).to_string().contains("Network"));
        assert!(Error::Parse("p".into()).to_string().contains("Parse"));
        assert!(Error::Crypto("c".into()).to_string().contains("Crypto"));
        assert!(Error::CryptoUnavailable("u".into()).to_string().contains("BearDog"));
        assert!(Error::Protocol("pr".into()).to_string().contains("Protocol"));
        assert!(Error::Consensus("co".into()).to_string().contains("Consensus"));
        assert!(Error::Circuit("ci".into()).to_string().contains("Circuit"));
        assert!(Error::Stream("st".into()).to_string().contains("Stream"));
        assert_eq!(Error::Timeout.to_string(), "Operation timed out");
        assert!(Error::NotFound("nf".into()).to_string().contains("Not found"));
    }
}
