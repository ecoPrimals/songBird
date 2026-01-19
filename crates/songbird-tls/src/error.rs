//! Error types for Songbird TLS
//!
//! All errors in Songbird TLS use `Result<T, TlsError>` for consistent error handling.
//! No panics, no unwraps in production code - TRUE modern idiomatic Rust!

use std::fmt;

/// Result type for TLS operations
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

    /// Crypto provider error (BearDog unavailable or operation failed)
    CryptoError(String),

    /// Internal error (should never happen in production)
    InternalError(String),

    /// Buffer too small
    BufferTooSmall {
        required: usize,
        available: usize,
    },

    /// Invalid parameter
    InvalidParameter(String),

    /// Record too large (> 16KB)
    RecordTooLarge {
        size: usize,
    },

    /// Unexpected message type
    UnexpectedMessage {
        expected: String,
        got: String,
    },
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            TlsError::DecryptError => write!(f, "Decryption failed"),
            TlsError::CertificateError(msg) => write!(f, "Certificate error: {}", msg),
            TlsError::HandshakeFailure(msg) => write!(f, "Handshake failure: {}", msg),
            TlsError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
            TlsError::IoError(msg) => write!(f, "IO error: {}", msg),
            TlsError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            TlsError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            TlsError::BufferTooSmall {
                required,
                available,
            } => {
                write!(
                    f,
                    "Buffer too small: required {} bytes, available {} bytes",
                    required, available
                )
            }
            TlsError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            TlsError::RecordTooLarge {
                size,
            } => {
                write!(f, "Record too large: {} bytes (max 16384)", size)
            }
            TlsError::UnexpectedMessage {
                expected,
                got,
            } => {
                write!(f, "Unexpected message: expected {}, got {}", expected, got)
            }
        }
    }
}

impl std::error::Error for TlsError {}

// Conversions from common error types
impl From<std::io::Error> for TlsError {
    fn from(err: std::io::Error) -> Self {
        TlsError::IoError(err.to_string())
    }
}

impl From<anyhow::Error> for TlsError {
    fn from(err: anyhow::Error) -> Self {
        TlsError::InternalError(err.to_string())
    }
}

/// Map TLS errors to alert codes (RFC 8446 Section 6)
impl TlsError {
    pub fn to_alert_code(&self) -> u8 {
        match self {
            TlsError::DecryptError => 51,        // decrypt_error
            TlsError::CertificateError(_) => 42, // bad_certificate
            TlsError::HandshakeFailure(_) => 40, // handshake_failure
            TlsError::Unsupported(_) => 70,      // protocol_version
            TlsError::ProtocolError(_) => 10,    // unexpected_message
            TlsError::InvalidParameter(_) => 47, // illegal_parameter
            TlsError::RecordTooLarge {
                ..
            } => 22, // record_overflow
            TlsError::UnexpectedMessage {
                ..
            } => 10, // unexpected_message
            TlsError::InternalError(_) => 80,    // internal_error
            _ => 80,                             // internal_error (default)
        }
    }

    pub fn is_fatal(&self) -> bool {
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
