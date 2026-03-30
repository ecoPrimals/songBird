// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC error types (pure Rust — no quinn/rustls dependencies).

use thiserror::Error;

/// Result type for QUIC operations.
pub type Result<T> = std::result::Result<T, QuicError>;

/// QUIC protocol errors.
#[derive(Debug, Error)]
pub enum QuicError {
    /// QUIC transport protocol error (RFC 9000 Section 20).
    #[error("QUIC transport error {code:#x}: {reason}")]
    Transport {
        /// Transport error code.
        code: u64,
        /// Human-readable reason.
        reason: String,
    },

    /// Connection closed by peer.
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Stream error.
    #[error("Stream error: {0}")]
    Stream(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// BearDog crypto delegation error.
    #[error("BearDog crypto error: {0}")]
    Crypto(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Address parse error.
    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),

    /// TLS handshake error.
    #[error("TLS handshake error: {0}")]
    Handshake(String),

    /// Operation timed out.
    #[error("Operation timed out")]
    Timeout,

    /// Not connected.
    #[error("Not connected")]
    NotConnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_config_display() {
        let e = QuicError::Stream("bad stream".into());
        assert!(e.to_string().contains("bad stream"));
        let c = QuicError::Config("bad cfg".into());
        assert!(c.to_string().contains("bad cfg"));
    }

    #[test]
    fn addr_parse_maps_to_invalid_address() {
        let err: QuicError = "not-a-socket!!!".parse::<std::net::SocketAddr>().unwrap_err().into();
        assert!(matches!(err, QuicError::InvalidAddress(_)));
    }

    #[test]
    fn io_error_roundtrip() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let e: QuicError = io.into();
        assert!(matches!(e, QuicError::Io(_)));
    }

    #[test]
    fn timeout_and_not_connected_display() {
        assert_eq!(QuicError::Timeout.to_string(), "Operation timed out");
        assert_eq!(QuicError::NotConnected.to_string(), "Not connected");
    }

    #[test]
    fn transport_error_display() {
        let e = QuicError::Transport {
            code: 0x0A,
            reason: "flow control".into(),
        };
        assert!(e.to_string().contains("flow control"));
        assert!(e.to_string().contains("0xa"));
    }

    #[test]
    fn crypto_error_display() {
        let e = QuicError::Crypto("beardog unavailable".into());
        assert!(e.to_string().contains("beardog unavailable"));
    }

    #[test]
    fn handshake_error_display() {
        let e = QuicError::Handshake("TLS failed".into());
        assert!(e.to_string().contains("TLS failed"));
    }
}
