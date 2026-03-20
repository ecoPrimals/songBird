// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Alert Protocol (RFC 8446 Section 6)
//!
//! The Alert Protocol is used to signal errors and warnings during the TLS handshake
//! and connection. In TLS 1.3, all alerts are fatal (no warnings).

use crate::error::TlsError;

/// Alert message
///
/// ```text
/// struct {
///     AlertLevel level;
///     AlertDescription description;
/// } Alert;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alert {
    /// Alert level (warning or fatal)
    pub level: AlertLevel,

    /// Alert description (specific error code)
    pub description: AlertDescription,
}

impl Alert {
    /// Create a new Alert message
    #[must_use]
    pub const fn new(level: AlertLevel, description: AlertDescription) -> Self {
        Self {
            level,
            description,
        }
    }

    /// Create a fatal alert
    #[must_use]
    pub const fn fatal(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Fatal,
            description,
        }
    }

    /// Create a warning alert (deprecated in TLS 1.3, but kept for compatibility)
    #[must_use]
    pub const fn warning(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Warning,
            description,
        }
    }

    /// Create a `close_notify` alert (graceful shutdown)
    #[must_use]
    pub const fn close_notify() -> Self {
        Self {
            level: AlertLevel::Warning,
            description: AlertDescription::CloseNotify,
        }
    }

    /// Convert from `TlsError` to Alert
    #[must_use]
    pub const fn from_error(error: &TlsError) -> Self {
        let description = match error {
            TlsError::DecryptError => AlertDescription::DecryptError,
            TlsError::CertificateError(_) => AlertDescription::BadCertificate,
            TlsError::HandshakeFailure(_) => AlertDescription::HandshakeFailure,
            TlsError::Unsupported(_) => AlertDescription::ProtocolVersion,
            TlsError::ProtocolError(_)
            | TlsError::UnexpectedMessage {
                ..
            } => AlertDescription::UnexpectedMessage,
            TlsError::InvalidParameter(_) => AlertDescription::IllegalParameter,
            TlsError::RecordTooLarge {
                ..
            } => AlertDescription::RecordOverflow,
            _ => AlertDescription::InternalError,
        };

        Self::fatal(description)
    }

    /// Check if this is a `close_notify` alert
    #[must_use]
    pub fn is_close_notify(&self) -> bool {
        self.description == AlertDescription::CloseNotify
    }
}

/// Alert level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertLevel {
    /// Non-fatal alert (TLS 1.3 discourages warnings).
    Warning = 1,
    /// Fatal alert; peer must close the connection.
    Fatal = 2,
}

impl From<u8> for AlertLevel {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Warning,
            _ => Self::Fatal, // Default to fatal for unknown levels
        }
    }
}

impl From<AlertLevel> for u8 {
    fn from(level: AlertLevel) -> Self {
        level as Self
    }
}

/// Alert description (error codes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertDescription {
    /// Graceful shutdown requested.
    CloseNotify = 0,
    /// Peer received an unexpected handshake message.
    UnexpectedMessage = 10,
    /// MAC verification failed on a record.
    BadRecordMac = 20,
    /// Record length exceeded the allowed maximum.
    RecordOverflow = 22,
    /// Handshake could not be negotiated.
    HandshakeFailure = 40,
    /// Certificate was malformed or failed validation.
    BadCertificate = 42,
    /// Certificate type is not supported.
    UnsupportedCertificate = 43,
    /// Certificate was revoked.
    CertificateRevoked = 44,
    /// Certificate is expired.
    CertificateExpired = 45,
    /// Certificate chain could not be built.
    CertificateUnknown = 46,
    /// Handshake parameter was illegal.
    IllegalParameter = 47,
    /// Issuing CA is not trusted.
    UnknownCa = 48,
    /// Authenticated user is not permitted.
    AccessDenied = 49,
    /// Message could not be decoded.
    DecodeError = 50,
    /// Decryption or verification failed.
    DecryptError = 51,
    /// Protocol version is not acceptable.
    ProtocolVersion = 70,
    /// Negotiated parameters are too weak.
    InsufficientSecurity = 71,
    /// Implementation-specific failure.
    InternalError = 80,
    /// TLS 1.3 required but client attempted legacy downgrade.
    InappropriateFallback = 86,
    /// Handshake canceled by user action.
    UserCanceled = 90,
    /// Mandatory extension was absent.
    MissingExtension = 109,
    /// Unsupported extension was received.
    UnsupportedExtension = 110,
    /// Server name was not recognized.
    UnrecognizedName = 112,
    /// OCSP stapling response was invalid.
    BadCertificateStatusResponse = 113,
    /// PSK identity is unknown.
    UnknownPskIdentity = 115,
    /// Client certificate was required but not provided.
    CertificateRequired = 116,
    /// ALPN negotiation failed to select a protocol.
    NoApplicationProtocol = 120,
}

impl From<u8> for AlertDescription {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::CloseNotify,
            10 => Self::UnexpectedMessage,
            20 => Self::BadRecordMac,
            22 => Self::RecordOverflow,
            40 => Self::HandshakeFailure,
            42 => Self::BadCertificate,
            43 => Self::UnsupportedCertificate,
            44 => Self::CertificateRevoked,
            45 => Self::CertificateExpired,
            46 => Self::CertificateUnknown,
            47 => Self::IllegalParameter,
            48 => Self::UnknownCa,
            49 => Self::AccessDenied,
            50 => Self::DecodeError,
            51 => Self::DecryptError,
            70 => Self::ProtocolVersion,
            71 => Self::InsufficientSecurity,
            86 => Self::InappropriateFallback,
            90 => Self::UserCanceled,
            109 => Self::MissingExtension,
            110 => Self::UnsupportedExtension,
            112 => Self::UnrecognizedName,
            113 => Self::BadCertificateStatusResponse,
            115 => Self::UnknownPskIdentity,
            116 => Self::CertificateRequired,
            120 => Self::NoApplicationProtocol,
            _ => Self::InternalError,
        }
    }
}

impl From<AlertDescription> for u8 {
    fn from(desc: AlertDescription) -> Self {
        desc as Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_new() {
        let alert = Alert::new(AlertLevel::Fatal, AlertDescription::HandshakeFailure);
        assert_eq!(alert.level, AlertLevel::Fatal);
        assert_eq!(alert.description, AlertDescription::HandshakeFailure);
    }

    #[test]
    fn test_alert_fatal() {
        let alert = Alert::fatal(AlertDescription::DecryptError);
        assert_eq!(alert.level, AlertLevel::Fatal);
        assert_eq!(alert.description, AlertDescription::DecryptError);
    }

    #[test]
    fn test_alert_warning() {
        let alert = Alert::warning(AlertDescription::CloseNotify);
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.description, AlertDescription::CloseNotify);
    }

    #[test]
    fn test_alert_close_notify() {
        let alert = Alert::close_notify();
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.description, AlertDescription::CloseNotify);
        assert!(alert.is_close_notify());
    }

    #[test]
    fn test_alert_from_error() {
        let error = TlsError::DecryptError;
        let alert = Alert::from_error(&error);
        assert_eq!(alert.level, AlertLevel::Fatal);
        assert_eq!(alert.description, AlertDescription::DecryptError);

        let error = TlsError::CertificateError("test".to_string());
        let alert = Alert::from_error(&error);
        assert_eq!(alert.description, AlertDescription::BadCertificate);
    }

    #[test]
    fn test_alert_level_conversion() {
        assert_eq!(u8::from(AlertLevel::Warning), 1);
        assert_eq!(u8::from(AlertLevel::Fatal), 2);
        assert_eq!(AlertLevel::from(1), AlertLevel::Warning);
        assert_eq!(AlertLevel::from(2), AlertLevel::Fatal);
    }

    #[test]
    fn test_alert_description_conversion() {
        assert_eq!(u8::from(AlertDescription::CloseNotify), 0);
        assert_eq!(u8::from(AlertDescription::HandshakeFailure), 40);
        assert_eq!(u8::from(AlertDescription::DecryptError), 51);

        assert_eq!(AlertDescription::from(0), AlertDescription::CloseNotify);
        assert_eq!(AlertDescription::from(40), AlertDescription::HandshakeFailure);
        assert_eq!(AlertDescription::from(51), AlertDescription::DecryptError);
    }

    #[test]
    fn test_unknown_conversions() {
        // Unknown alert level defaults to Fatal
        assert_eq!(AlertLevel::from(99), AlertLevel::Fatal);

        // Unknown alert description defaults to InternalError
        assert_eq!(AlertDescription::from(255), AlertDescription::InternalError);
    }
}
