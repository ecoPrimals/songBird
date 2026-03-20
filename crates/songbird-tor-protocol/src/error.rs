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
