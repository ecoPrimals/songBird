// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Error types for Songbird TLS
//!
//! All errors in Songbird TLS use `Result<T, TlsError>` for consistent error handling.
//! No panics, no unwraps in production code - TRUE modern idiomatic Rust!

use std::fmt;

/// Standard result type for TLS operations in this crate.
pub type Result<T> = std::result::Result<T, TlsError>;

/// TLS error types
///
/// Covers all error conditions in TLS 1.3 protocol implementation.
/// Maps to TLS alert codes where appropriate.
#[derive(Debug, Clone)]
pub enum TlsError {
    /// Protocol error - malformed message or invalid state transition
    ProtocolError(String),

    /// Decryption failed (authentication tag mismatch)
    DecryptError,

    /// Invalid certificate or signature
    CertificateError(String),

    /// Handshake failed
    HandshakeFailure(String),

    /// Unsupported feature or configuration
    Unsupported(String),

    /// IO error (connection closed, timeout, etc)
    IoError(String),

    /// Crypto provider error (security provider unavailable or operation failed)
    CryptoError(String),

    /// Security provider / Neural API crypto backend not reachable (no silent mock crypto)
    CryptoUnavailable,

    /// Internal error (should never happen in production)
    InternalError(String),

    /// Buffer too small
    BufferTooSmall {
        /// Minimum bytes required for the operation.
        required: usize,
        /// Bytes actually available in the buffer.
        available: usize,
    },

    /// Invalid parameter
    InvalidParameter(String),

    /// Record too large (> 16KB)
    RecordTooLarge {
        /// Observed record size in bytes.
        size: usize,
    },

    /// Unexpected message type
    UnexpectedMessage {
        /// Message type the state machine expected.
        expected: String,
        /// Message type actually received.
        got: String,
    },
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtocolError(msg) => write!(f, "Protocol error: {msg}"),
            Self::DecryptError => write!(f, "Decryption failed"),
            Self::CertificateError(msg) => write!(f, "Certificate error: {msg}"),
            Self::HandshakeFailure(msg) => write!(f, "Handshake failure: {msg}"),
            Self::Unsupported(msg) => write!(f, "Unsupported: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
            Self::CryptoError(msg) => write!(f, "Crypto error: {msg}"),
            Self::CryptoUnavailable => {
                write!(f, "Crypto unavailable: security provider backend not reachable")
            }
            Self::InternalError(msg) => write!(f, "Internal error: {msg}"),
            Self::BufferTooSmall {
                required,
                available,
            } => {
                write!(
                    f,
                    "Buffer too small: required {required} bytes, available {available} bytes"
                )
            }
            Self::InvalidParameter(msg) => write!(f, "Invalid parameter: {msg}"),
            Self::RecordTooLarge {
                size,
            } => {
                write!(f, "Record too large: {size} bytes (max 16384)")
            }
            Self::UnexpectedMessage {
                expected,
                got,
            } => {
                write!(f, "Unexpected message: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for TlsError {}

// Conversions from common error types
impl From<std::io::Error> for TlsError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<anyhow::Error> for TlsError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalError(err.to_string())
    }
}

/// Map TLS errors to alert codes (RFC 8446 Section 6).
impl TlsError {
    /// Map this error to a TLS alert description byte for wire encoding.
    #[must_use]
    pub const fn to_alert_code(&self) -> u8 {
        match self {
            Self::DecryptError => 51,        // decrypt_error
            Self::CertificateError(_) => 42, // bad_certificate
            Self::HandshakeFailure(_) => 40, // handshake_failure
            Self::Unsupported(_) => 70,      // protocol_version
            Self::ProtocolError(_)
            | Self::UnexpectedMessage {
                ..
            } => 10, // unexpected_message
            Self::InvalidParameter(_) => 47, // illegal_parameter
            Self::RecordTooLarge {
                ..
            } => 22, // record_overflow
            Self::CryptoUnavailable
            | Self::CryptoError(_)
            | Self::IoError(_)
            | Self::InternalError(_)
            | Self::BufferTooSmall {
                ..
            } => 80, // internal_error
        }
    }

    /// Report whether this error should terminate the connection (TLS 1.3 treats all as fatal).
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        // All errors are fatal in TLS 1.3 (no warnings)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TlsError::ProtocolError("test".to_string());
        assert_eq!(err.to_string(), "Protocol error: test");

        let err = TlsError::DecryptError;
        assert_eq!(err.to_string(), "Decryption failed");

        let err = TlsError::BufferTooSmall {
            required: 100,
            available: 50,
        };
        assert_eq!(err.to_string(), "Buffer too small: required 100 bytes, available 50 bytes");
    }

    #[test]
    fn test_alert_codes() {
        assert_eq!(TlsError::DecryptError.to_alert_code(), 51);
        assert_eq!(TlsError::CertificateError("test".to_string()).to_alert_code(), 42);
        assert_eq!(TlsError::HandshakeFailure("test".to_string()).to_alert_code(), 40);
    }

    #[test]
    fn test_is_fatal() {
        // All errors are fatal in TLS 1.3
        assert!(TlsError::DecryptError.is_fatal());
        assert!(TlsError::ProtocolError("test".to_string()).is_fatal());
        assert!(TlsError::HandshakeFailure("test".to_string()).is_fatal());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "test");
        let tls_err: TlsError = io_err.into();
        assert!(matches!(tls_err, TlsError::IoError(_)));
    }
}
