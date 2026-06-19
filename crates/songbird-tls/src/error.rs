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

    #[test]
    fn test_all_display_variants() {
        let cases: Vec<(TlsError, &str)> = vec![
            (TlsError::ProtocolError("bad msg".to_string()), "Protocol error: bad msg"),
            (TlsError::DecryptError, "Decryption failed"),
            (TlsError::CertificateError("bad cert".to_string()), "Certificate error: bad cert"),
            (TlsError::HandshakeFailure("hsk".to_string()), "Handshake failure: hsk"),
            (TlsError::Unsupported("feat".to_string()), "Unsupported: feat"),
            (TlsError::IoError("conn reset".to_string()), "IO error: conn reset"),
            (TlsError::CryptoError("hmac fail".to_string()), "Crypto error: hmac fail"),
            (
                TlsError::CryptoUnavailable,
                "Crypto unavailable: security provider backend not reachable",
            ),
            (TlsError::InternalError("bug".to_string()), "Internal error: bug"),
            (
                TlsError::BufferTooSmall {
                    required: 64,
                    available: 8,
                },
                "Buffer too small: required 64 bytes, available 8 bytes",
            ),
            (TlsError::InvalidParameter("bad len".to_string()), "Invalid parameter: bad len"),
            (
                TlsError::RecordTooLarge {
                    size: 20_000,
                },
                "Record too large: 20000 bytes (max 16384)",
            ),
            (
                TlsError::UnexpectedMessage {
                    expected: "Finished".to_string(),
                    got: "Alert".to_string(),
                },
                "Unexpected message: expected Finished, got Alert",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_from_io_error_preserves_kind_in_message() {
        let kind_only: TlsError =
            std::io::Error::from(std::io::ErrorKind::ConnectionRefused).into();
        let TlsError::IoError(kind_msg) = kind_only else {
            panic!("expected IoError variant");
        };
        assert!(kind_msg.contains("connection refused"));

        let with_detail = std::io::Error::new(std::io::ErrorKind::TimedOut, "peer hung up");
        let TlsError::IoError(detail_msg) = with_detail.into() else {
            panic!("expected IoError variant");
        };
        assert!(detail_msg.contains("peer hung up"));
    }

    #[test]
    fn test_error_clone_and_partial_eq() {
        let original = TlsError::BufferTooSmall {
            required: 10,
            available: 3,
        };
        let cloned = original.clone();
        assert_eq!(format!("{original}"), format!("{cloned}"));
        assert!(matches!(
            cloned,
            TlsError::BufferTooSmall {
                required: 10,
                available: 3
            }
        ));
    }

    #[test]
    fn test_error_matching_patterns_classify_by_alert() {
        fn alert_bucket(err: &TlsError) -> u8 {
            match err {
                TlsError::DecryptError => 51,
                TlsError::CertificateError(_) => 42,
                TlsError::HandshakeFailure(_) => 40,
                TlsError::Unsupported(_) => 70,
                TlsError::ProtocolError(_)
                | TlsError::UnexpectedMessage {
                    ..
                } => 10,
                TlsError::InvalidParameter(_) => 47,
                TlsError::RecordTooLarge {
                    ..
                } => 22,
                TlsError::CryptoUnavailable
                | TlsError::CryptoError(_)
                | TlsError::IoError(_)
                | TlsError::InternalError(_)
                | TlsError::BufferTooSmall {
                    ..
                } => 80,
            }
        }

        assert_eq!(alert_bucket(&TlsError::CryptoUnavailable), 80);
        assert_eq!(
            alert_bucket(&TlsError::UnexpectedMessage {
                expected: "A".to_string(),
                got: "B".to_string(),
            }),
            10
        );
        assert_eq!(
            TlsError::RecordTooLarge {
                size: 999
            }
            .to_alert_code(),
            alert_bucket(&TlsError::RecordTooLarge {
                size: 999
            })
        );
    }

    #[test]
    fn test_std_error_trait_no_source_chain() {
        use std::error::Error;

        let err = TlsError::InternalError("nested".to_string());
        let dyn_err: &dyn Error = &err;
        assert!(dyn_err.source().is_none());
    }

    #[test]
    fn test_all_alert_codes() {
        assert_eq!(TlsError::Unsupported("v".to_string()).to_alert_code(), 70);
        assert_eq!(TlsError::ProtocolError("p".to_string()).to_alert_code(), 10);
        assert_eq!(TlsError::InvalidParameter("i".to_string()).to_alert_code(), 47);
        assert_eq!(
            TlsError::RecordTooLarge {
                size: 20000
            }
            .to_alert_code(),
            22
        );
        assert_eq!(TlsError::CryptoUnavailable.to_alert_code(), 80);
        assert_eq!(TlsError::CryptoError("c".to_string()).to_alert_code(), 80);
        assert_eq!(TlsError::IoError("i".to_string()).to_alert_code(), 80);
        assert_eq!(TlsError::InternalError("i".to_string()).to_alert_code(), 80);
        assert_eq!(
            TlsError::BufferTooSmall {
                required: 1,
                available: 0
            }
            .to_alert_code(),
            80
        );
        assert_eq!(
            TlsError::UnexpectedMessage {
                expected: "x".to_string(),
                got: "y".to_string(),
            }
            .to_alert_code(),
            10
        );
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("provider down");
        let tls_err: TlsError = anyhow_err.into();
        assert!(matches!(tls_err, TlsError::InternalError(_)));
        assert!(tls_err.to_string().contains("provider down"));
    }
}
