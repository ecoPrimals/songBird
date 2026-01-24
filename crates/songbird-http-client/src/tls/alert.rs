//! TLS Alert Protocol (RFC 8446 Section 6)
//!
//! Implements parsing and handling of TLS alert messages.

use std::fmt;

/// TLS Alert Level (RFC 8446 Section 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertLevel {
    /// Warning alert (1)
    Warning = 1,
    /// Fatal alert (2)
    Fatal = 2,
}

impl AlertLevel {
    /// Parse alert level from byte
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(AlertLevel::Warning),
            2 => Some(AlertLevel::Fatal),
            _ => None,
        }
    }
}

impl fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertLevel::Warning => write!(f, "Warning"),
            AlertLevel::Fatal => write!(f, "Fatal"),
        }
    }
}

/// TLS Alert Description (RFC 8446 Section 6.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertDescription {
    /// close_notify (0)
    CloseNotify = 0,
    /// unexpected_message (10)
    UnexpectedMessage = 10,
    /// bad_record_mac (20)
    BadRecordMac = 20,
    /// record_overflow (22)
    RecordOverflow = 22,
    /// handshake_failure (40)
    HandshakeFailure = 40,
    /// bad_certificate (42)
    BadCertificate = 42,
    /// unsupported_certificate (43)
    UnsupportedCertificate = 43,
    /// certificate_revoked (44)
    CertificateRevoked = 44,
    /// certificate_expired (45)
    CertificateExpired = 45,
    /// certificate_unknown (46)
    CertificateUnknown = 46,
    /// illegal_parameter (47)
    IllegalParameter = 47,
    /// unknown_ca (48)
    UnknownCa = 48,
    /// access_denied (49)
    AccessDenied = 49,
    /// decode_error (50)
    DecodeError = 50,
    /// decrypt_error (51)
    DecryptError = 51,
    /// protocol_version (70)
    ProtocolVersion = 70,
    /// insufficient_security (71)
    InsufficientSecurity = 71,
    /// internal_error (80)
    InternalError = 80,
    /// inappropriate_fallback (86)
    InappropriateFallback = 86,
    /// user_canceled (90)
    UserCanceled = 90,
    /// missing_extension (109)
    MissingExtension = 109,
    /// unsupported_extension (110)
    UnsupportedExtension = 110,
    /// unrecognized_name (112)
    UnrecognizedName = 112,
    /// bad_certificate_status_response (113)
    BadCertificateStatusResponse = 113,
    /// unknown_psk_identity (115)
    UnknownPskIdentity = 115,
    /// certificate_required (116)
    CertificateRequired = 116,
    /// no_application_protocol (120)
    NoApplicationProtocol = 120,
}

impl AlertDescription {
    /// Parse alert description from byte
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(AlertDescription::CloseNotify),
            10 => Some(AlertDescription::UnexpectedMessage),
            20 => Some(AlertDescription::BadRecordMac),
            22 => Some(AlertDescription::RecordOverflow),
            40 => Some(AlertDescription::HandshakeFailure),
            42 => Some(AlertDescription::BadCertificate),
            43 => Some(AlertDescription::UnsupportedCertificate),
            44 => Some(AlertDescription::CertificateRevoked),
            45 => Some(AlertDescription::CertificateExpired),
            46 => Some(AlertDescription::CertificateUnknown),
            47 => Some(AlertDescription::IllegalParameter),
            48 => Some(AlertDescription::UnknownCa),
            49 => Some(AlertDescription::AccessDenied),
            50 => Some(AlertDescription::DecodeError),
            51 => Some(AlertDescription::DecryptError),
            70 => Some(AlertDescription::ProtocolVersion),
            71 => Some(AlertDescription::InsufficientSecurity),
            80 => Some(AlertDescription::InternalError),
            86 => Some(AlertDescription::InappropriateFallback),
            90 => Some(AlertDescription::UserCanceled),
            109 => Some(AlertDescription::MissingExtension),
            110 => Some(AlertDescription::UnsupportedExtension),
            112 => Some(AlertDescription::UnrecognizedName),
            113 => Some(AlertDescription::BadCertificateStatusResponse),
            115 => Some(AlertDescription::UnknownPskIdentity),
            116 => Some(AlertDescription::CertificateRequired),
            120 => Some(AlertDescription::NoApplicationProtocol),
            _ => None,
        }
    }

    /// Get human-readable explanation of this alert
    pub fn explanation(&self) -> &'static str {
        match self {
            AlertDescription::CloseNotify => "Connection is being closed normally",
            AlertDescription::UnexpectedMessage => "Received unexpected message in current state",
            AlertDescription::BadRecordMac => "Message authentication code validation failed",
            AlertDescription::RecordOverflow => "TLS record exceeded maximum allowed size",
            AlertDescription::HandshakeFailure => "Handshake negotiation failed (generic)",
            AlertDescription::BadCertificate => "Certificate is corrupt or invalid",
            AlertDescription::UnsupportedCertificate => "Certificate type is not supported",
            AlertDescription::CertificateRevoked => "Certificate has been revoked",
            AlertDescription::CertificateExpired => "Certificate has expired",
            AlertDescription::CertificateUnknown => {
                "Certificate validation failed for unknown reason"
            }
            AlertDescription::IllegalParameter => {
                "Field in handshake message is incorrect or inconsistent"
            }
            AlertDescription::UnknownCa => "Certificate authority is not recognized",
            AlertDescription::AccessDenied => "Valid certificate but access denied",
            AlertDescription::DecodeError => "Message could not be decoded",
            AlertDescription::DecryptError => "Decryption or signature verification failed",
            AlertDescription::ProtocolVersion => "Protocol version is not supported",
            AlertDescription::InsufficientSecurity => "Security parameters are inadequate",
            AlertDescription::InternalError => "Server internal error occurred",
            AlertDescription::InappropriateFallback => {
                "Inappropriate protocol version fallback detected"
            }
            AlertDescription::UserCanceled => "Handshake canceled by user",
            AlertDescription::MissingExtension => "Required TLS extension is missing",
            AlertDescription::UnsupportedExtension => "Extension is not supported",
            AlertDescription::UnrecognizedName => "Server name (SNI) is not recognized",
            AlertDescription::BadCertificateStatusResponse => "OCSP response is invalid",
            AlertDescription::UnknownPskIdentity => "PSK identity is unknown",
            AlertDescription::CertificateRequired => {
                "Client certificate is required but not provided"
            }
            AlertDescription::NoApplicationProtocol => {
                "No application protocol (ALPN) could be negotiated"
            }
        }
    }

    /// Get suggested action for this alert
    pub fn suggested_action(&self) -> &'static str {
        match self {
            AlertDescription::ProtocolVersion => {
                "Server may not support TLS 1.3. Try TLS 1.2 or check server capabilities."
            }
            AlertDescription::HandshakeFailure => {
                "Check cipher suites, extensions, and protocol version compatibility."
            }
            AlertDescription::UnsupportedExtension => {
                "Try minimal extension set or adjust ClientHello extensions."
            }
            AlertDescription::MissingExtension => {
                "Add required extension (check server requirements)."
            }
            AlertDescription::NoApplicationProtocol => {
                "Adjust ALPN extension or remove if not required."
            }
            AlertDescription::UnrecognizedName => {
                "Check SNI hostname or try without SNI extension."
            }
            AlertDescription::InsufficientSecurity => "Use stronger cipher suites or key sizes.",
            AlertDescription::IllegalParameter => "Check ClientHello format and field values.",
            _ => "Review server logs or try different configuration.",
        }
    }
}

impl fmt::Display for AlertDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AlertDescription::CloseNotify => "close_notify",
            AlertDescription::UnexpectedMessage => "unexpected_message",
            AlertDescription::BadRecordMac => "bad_record_mac",
            AlertDescription::RecordOverflow => "record_overflow",
            AlertDescription::HandshakeFailure => "handshake_failure",
            AlertDescription::BadCertificate => "bad_certificate",
            AlertDescription::UnsupportedCertificate => "unsupported_certificate",
            AlertDescription::CertificateRevoked => "certificate_revoked",
            AlertDescription::CertificateExpired => "certificate_expired",
            AlertDescription::CertificateUnknown => "certificate_unknown",
            AlertDescription::IllegalParameter => "illegal_parameter",
            AlertDescription::UnknownCa => "unknown_ca",
            AlertDescription::AccessDenied => "access_denied",
            AlertDescription::DecodeError => "decode_error",
            AlertDescription::DecryptError => "decrypt_error",
            AlertDescription::ProtocolVersion => "protocol_version",
            AlertDescription::InsufficientSecurity => "insufficient_security",
            AlertDescription::InternalError => "internal_error",
            AlertDescription::InappropriateFallback => "inappropriate_fallback",
            AlertDescription::UserCanceled => "user_canceled",
            AlertDescription::MissingExtension => "missing_extension",
            AlertDescription::UnsupportedExtension => "unsupported_extension",
            AlertDescription::UnrecognizedName => "unrecognized_name",
            AlertDescription::BadCertificateStatusResponse => "bad_certificate_status_response",
            AlertDescription::UnknownPskIdentity => "unknown_psk_identity",
            AlertDescription::CertificateRequired => "certificate_required",
            AlertDescription::NoApplicationProtocol => "no_application_protocol",
        };
        write!(f, "{}", name)
    }
}

/// Parsed TLS Alert message
#[derive(Debug, Clone)]
pub struct TlsAlert {
    pub level: AlertLevel,
    pub description: AlertDescription,
    pub raw_level: u8,
    pub raw_description: u8,
}

impl TlsAlert {
    /// Parse TLS alert from bytes
    ///
    /// Alert format (RFC 8446 Section 6):
    /// - Level: 1 byte (1=warning, 2=fatal)
    /// - Description: 1 byte (alert code)
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 2 {
            return Err(format!("Alert too short: {} bytes (need 2)", data.len()));
        }

        let raw_level = data[0];
        let raw_description = data[1];

        let level = AlertLevel::from_u8(raw_level)
            .ok_or_else(|| format!("Unknown alert level: {}", raw_level))?;

        let description = AlertDescription::from_u8(raw_description)
            .ok_or_else(|| format!("Unknown alert description: {}", raw_description))?;

        Ok(TlsAlert {
            level,
            description,
            raw_level,
            raw_description,
        })
    }

    /// Check if this is a fatal alert
    pub fn is_fatal(&self) -> bool {
        matches!(self.level, AlertLevel::Fatal)
    }

    /// Get formatted display string with full details
    pub fn to_detailed_string(&self) -> String {
        format!(
            "{} Alert: {} ({})\n  Code: Level={}, Description={}\n  Explanation: {}\n  Action: {}",
            self.level,
            self.description,
            if self.is_fatal() {
                "connection terminated"
            } else {
                "warning"
            },
            self.raw_level,
            self.raw_description,
            self.description.explanation(),
            self.description.suggested_action()
        )
    }
}

impl fmt::Display for TlsAlert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Alert: {} (code {}/{})",
            self.level, self.description, self.raw_level, self.raw_description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_close_notify() {
        let data = [1, 0]; // Warning, close_notify
        let alert = TlsAlert::parse(&data).unwrap();
        assert_eq!(alert.level, AlertLevel::Warning);
        assert_eq!(alert.description, AlertDescription::CloseNotify);
        assert!(!alert.is_fatal());
    }

    #[test]
    fn test_parse_handshake_failure() {
        let data = [2, 40]; // Fatal, handshake_failure
        let alert = TlsAlert::parse(&data).unwrap();
        assert_eq!(alert.level, AlertLevel::Fatal);
        assert_eq!(alert.description, AlertDescription::HandshakeFailure);
        assert!(alert.is_fatal());
    }

    #[test]
    fn test_parse_protocol_version() {
        let data = [2, 70]; // Fatal, protocol_version
        let alert = TlsAlert::parse(&data).unwrap();
        assert_eq!(alert.level, AlertLevel::Fatal);
        assert_eq!(alert.description, AlertDescription::ProtocolVersion);
        assert!(alert.is_fatal());
    }

    #[test]
    fn test_parse_too_short() {
        let data = [2]; // Only 1 byte
        let err = TlsAlert::parse(&data).unwrap_err();
        assert!(err.contains("too short"));
    }

    #[test]
    fn test_parse_unknown_level() {
        let data = [99, 0]; // Invalid level
        let err = TlsAlert::parse(&data).unwrap_err();
        assert!(err.contains("Unknown alert level"));
    }

    #[test]
    fn test_parse_unknown_description() {
        let data = [2, 99]; // Invalid description
        let err = TlsAlert::parse(&data).unwrap_err();
        assert!(err.contains("Unknown alert description"));
    }

    #[test]
    fn test_display_formats() {
        let data = [2, 40]; // Fatal, handshake_failure
        let alert = TlsAlert::parse(&data).unwrap();

        assert!(alert.to_string().contains("Fatal"));
        assert!(alert.to_string().contains("handshake_failure"));

        let detailed = alert.to_detailed_string();
        assert!(detailed.contains("Explanation"));
        assert!(detailed.contains("Action"));
    }

    #[test]
    fn test_alert_explanations() {
        let desc = AlertDescription::ProtocolVersion;
        assert!(desc.explanation().contains("not supported"));
        assert!(desc.suggested_action().contains("TLS 1.3"));
    }
}
