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
    Warning = 1,
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
    CloseNotify = 0,
    UnexpectedMessage = 10,
    BadRecordMac = 20,
    RecordOverflow = 22,
    HandshakeFailure = 40,
    BadCertificate = 42,
    UnsupportedCertificate = 43,
    CertificateRevoked = 44,
    CertificateExpired = 45,
    CertificateUnknown = 46,
    IllegalParameter = 47,
    UnknownCa = 48,
    AccessDenied = 49,
    DecodeError = 50,
    DecryptError = 51,
    ProtocolVersion = 70,
    InsufficientSecurity = 71,
    InternalError = 80,
    InappropriateFallback = 86,
    UserCanceled = 90,
    MissingExtension = 109,
    UnsupportedExtension = 110,
    UnrecognizedName = 112,
    BadCertificateStatusResponse = 113,
    UnknownPskIdentity = 115,
    CertificateRequired = 116,
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
