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
    pub fn new(level: AlertLevel, description: AlertDescription) -> Self {
        Self {
            level,
            description,
        }
    }

    /// Create a fatal alert
    pub fn fatal(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Fatal,
            description,
        }
    }

    /// Create a warning alert (deprecated in TLS 1.3, but kept for compatibility)
    pub fn warning(description: AlertDescription) -> Self {
        Self {
            level: AlertLevel::Warning,
            description,
        }
    }

    /// Create a close_notify alert (graceful shutdown)
    pub fn close_notify() -> Self {
        Self {
            level: AlertLevel::Warning,
            description: AlertDescription::CloseNotify,
        }
    }

    /// Convert from TlsError to Alert
    pub fn from_error(error: &TlsError) -> Self {
        let description = match error {
            TlsError::DecryptError => AlertDescription::DecryptError,
            TlsError::CertificateError(_) => AlertDescription::BadCertificate,
            TlsError::HandshakeFailure(_) => AlertDescription::HandshakeFailure,
            TlsError::Unsupported(_) => AlertDescription::ProtocolVersion,
            TlsError::ProtocolError(_) => AlertDescription::UnexpectedMessage,
            TlsError::InvalidParameter(_) => AlertDescription::IllegalParameter,
            TlsError::RecordTooLarge {
                ..
            } => AlertDescription::RecordOverflow,
            TlsError::UnexpectedMessage {
                ..
            } => AlertDescription::UnexpectedMessage,
            _ => AlertDescription::InternalError,
        };

        Alert::fatal(description)
    }

    /// Check if this is a close_notify alert
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
            1 => AlertLevel::Warning,
            2 => AlertLevel::Fatal,
            _ => AlertLevel::Fatal, // Default to fatal for unknown levels
        }
    }
}

impl From<AlertLevel> for u8 {
    fn from(level: AlertLevel) -> Self {
        level as u8
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
            0 => AlertDescription::CloseNotify,
            10 => AlertDescription::UnexpectedMessage,
            20 => AlertDescription::BadRecordMac,
            22 => AlertDescription::RecordOverflow,
            40 => AlertDescription::HandshakeFailure,
            42 => AlertDescription::BadCertificate,
            43 => AlertDescription::UnsupportedCertificate,
            44 => AlertDescription::CertificateRevoked,
            45 => AlertDescription::CertificateExpired,
            46 => AlertDescription::CertificateUnknown,
            47 => AlertDescription::IllegalParameter,
            48 => AlertDescription::UnknownCa,
            49 => AlertDescription::AccessDenied,
            50 => AlertDescription::DecodeError,
            51 => AlertDescription::DecryptError,
            70 => AlertDescription::ProtocolVersion,
            71 => AlertDescription::InsufficientSecurity,
            80 => AlertDescription::InternalError,
            86 => AlertDescription::InappropriateFallback,
            90 => AlertDescription::UserCanceled,
            109 => AlertDescription::MissingExtension,
            110 => AlertDescription::UnsupportedExtension,
            112 => AlertDescription::UnrecognizedName,
            113 => AlertDescription::BadCertificateStatusResponse,
            115 => AlertDescription::UnknownPskIdentity,
            116 => AlertDescription::CertificateRequired,
            120 => AlertDescription::NoApplicationProtocol,
            _ => AlertDescription::InternalError, // Default for unknown codes
        }
    }
}

impl From<AlertDescription> for u8 {
    fn from(desc: AlertDescription) -> Self {
        desc as u8
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
