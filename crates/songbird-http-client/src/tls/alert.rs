// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Warning),
            2 => Some(Self::Fatal),
            _ => None,
        }
    }
}

impl fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Warning => write!(f, "Warning"),
            Self::Fatal => write!(f, "Fatal"),
        }
    }
}

/// TLS Alert Description (RFC 8446 Section 6.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertDescription {
    /// `close_notify` (0)
    CloseNotify = 0,
    /// `unexpected_message` (10)
    UnexpectedMessage = 10,
    /// `bad_record_mac` (20)
    BadRecordMac = 20,
    /// `record_overflow` (22)
    RecordOverflow = 22,
    /// `handshake_failure` (40)
    HandshakeFailure = 40,
    /// `bad_certificate` (42)
    BadCertificate = 42,
    /// `unsupported_certificate` (43)
    UnsupportedCertificate = 43,
    /// `certificate_revoked` (44)
    CertificateRevoked = 44,
    /// `certificate_expired` (45)
    CertificateExpired = 45,
    /// `certificate_unknown` (46)
    CertificateUnknown = 46,
    /// `illegal_parameter` (47)
    IllegalParameter = 47,
    /// `unknown_ca` (48)
    UnknownCa = 48,
    /// `access_denied` (49)
    AccessDenied = 49,
    /// `decode_error` (50)
    DecodeError = 50,
    /// `decrypt_error` (51)
    DecryptError = 51,
    /// `protocol_version` (70)
    ProtocolVersion = 70,
    /// `insufficient_security` (71)
    InsufficientSecurity = 71,
    /// `internal_error` (80)
    InternalError = 80,
    /// `inappropriate_fallback` (86)
    InappropriateFallback = 86,
    /// `user_canceled` (90)
    UserCanceled = 90,
    /// `missing_extension` (109)
    MissingExtension = 109,
    /// `unsupported_extension` (110)
    UnsupportedExtension = 110,
    /// `unrecognized_name` (112)
    UnrecognizedName = 112,
    /// `bad_certificate_status_response` (113)
    BadCertificateStatusResponse = 113,
    /// `unknown_psk_identity` (115)
    UnknownPskIdentity = 115,
    /// `certificate_required` (116)
    CertificateRequired = 116,
    /// `no_application_protocol` (120)
    NoApplicationProtocol = 120,
}

impl AlertDescription {
    /// Parse alert description from byte
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::CloseNotify),
            10 => Some(Self::UnexpectedMessage),
            20 => Some(Self::BadRecordMac),
            22 => Some(Self::RecordOverflow),
            40 => Some(Self::HandshakeFailure),
            42 => Some(Self::BadCertificate),
            43 => Some(Self::UnsupportedCertificate),
            44 => Some(Self::CertificateRevoked),
            45 => Some(Self::CertificateExpired),
            46 => Some(Self::CertificateUnknown),
            47 => Some(Self::IllegalParameter),
            48 => Some(Self::UnknownCa),
            49 => Some(Self::AccessDenied),
            50 => Some(Self::DecodeError),
            51 => Some(Self::DecryptError),
            70 => Some(Self::ProtocolVersion),
            71 => Some(Self::InsufficientSecurity),
            80 => Some(Self::InternalError),
            86 => Some(Self::InappropriateFallback),
            90 => Some(Self::UserCanceled),
            109 => Some(Self::MissingExtension),
            110 => Some(Self::UnsupportedExtension),
            112 => Some(Self::UnrecognizedName),
            113 => Some(Self::BadCertificateStatusResponse),
            115 => Some(Self::UnknownPskIdentity),
            116 => Some(Self::CertificateRequired),
            120 => Some(Self::NoApplicationProtocol),
            _ => None,
        }
    }

    /// Get human-readable explanation of this alert
    #[must_use]
    pub const fn explanation(&self) -> &'static str {
        match self {
            Self::CloseNotify => "Connection is being closed normally",
            Self::UnexpectedMessage => "Received unexpected message in current state",
            Self::BadRecordMac => "Message authentication code validation failed",
            Self::RecordOverflow => "TLS record exceeded maximum allowed size",
            Self::HandshakeFailure => "Handshake negotiation failed (generic)",
            Self::BadCertificate => "Certificate is corrupt or invalid",
            Self::UnsupportedCertificate => "Certificate type is not supported",
            Self::CertificateRevoked => "Certificate has been revoked",
            Self::CertificateExpired => "Certificate has expired",
            Self::CertificateUnknown => "Certificate validation failed for unknown reason",
            Self::IllegalParameter => "Field in handshake message is incorrect or inconsistent",
            Self::UnknownCa => "Certificate authority is not recognized",
            Self::AccessDenied => "Valid certificate but access denied",
            Self::DecodeError => "Message could not be decoded",
            Self::DecryptError => "Decryption or signature verification failed",
            Self::ProtocolVersion => "Protocol version is not supported",
            Self::InsufficientSecurity => "Security parameters are inadequate",
            Self::InternalError => "Server internal error occurred",
            Self::InappropriateFallback => "Inappropriate protocol version fallback detected",
            Self::UserCanceled => "Handshake canceled by user",
            Self::MissingExtension => "Required TLS extension is missing",
            Self::UnsupportedExtension => "Extension is not supported",
            Self::UnrecognizedName => "Server name (SNI) is not recognized",
            Self::BadCertificateStatusResponse => "OCSP response is invalid",
            Self::UnknownPskIdentity => "PSK identity is unknown",
            Self::CertificateRequired => "Client certificate is required but not provided",
            Self::NoApplicationProtocol => "No application protocol (ALPN) could be negotiated",
        }
    }

    /// Get suggested action for this alert
    #[must_use]
    pub const fn suggested_action(&self) -> &'static str {
        match self {
            Self::ProtocolVersion => {
                "Server may not support TLS 1.3. Try TLS 1.2 or check server capabilities."
            }
            Self::HandshakeFailure => {
                "Check cipher suites, extensions, and protocol version compatibility."
            }
            Self::UnsupportedExtension => {
                "Try minimal extension set or adjust ClientHello extensions."
            }
            Self::MissingExtension => "Add required extension (check server requirements).",
            Self::NoApplicationProtocol => "Adjust ALPN extension or remove if not required.",
            Self::UnrecognizedName => "Check SNI hostname or try without SNI extension.",
            Self::InsufficientSecurity => "Use stronger cipher suites or key sizes.",
            Self::IllegalParameter => "Check ClientHello format and field values.",
            _ => "Review server logs or try different configuration.",
        }
    }
}

impl fmt::Display for AlertDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::CloseNotify => "close_notify",
            Self::UnexpectedMessage => "unexpected_message",
            Self::BadRecordMac => "bad_record_mac",
            Self::RecordOverflow => "record_overflow",
            Self::HandshakeFailure => "handshake_failure",
            Self::BadCertificate => "bad_certificate",
            Self::UnsupportedCertificate => "unsupported_certificate",
            Self::CertificateRevoked => "certificate_revoked",
            Self::CertificateExpired => "certificate_expired",
            Self::CertificateUnknown => "certificate_unknown",
            Self::IllegalParameter => "illegal_parameter",
            Self::UnknownCa => "unknown_ca",
            Self::AccessDenied => "access_denied",
            Self::DecodeError => "decode_error",
            Self::DecryptError => "decrypt_error",
            Self::ProtocolVersion => "protocol_version",
            Self::InsufficientSecurity => "insufficient_security",
            Self::InternalError => "internal_error",
            Self::InappropriateFallback => "inappropriate_fallback",
            Self::UserCanceled => "user_canceled",
            Self::MissingExtension => "missing_extension",
            Self::UnsupportedExtension => "unsupported_extension",
            Self::UnrecognizedName => "unrecognized_name",
            Self::BadCertificateStatusResponse => "bad_certificate_status_response",
            Self::UnknownPskIdentity => "unknown_psk_identity",
            Self::CertificateRequired => "certificate_required",
            Self::NoApplicationProtocol => "no_application_protocol",
        };
        write!(f, "{name}")
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
    ///
    /// # Errors
    ///
    /// Returns an error if the data is too short or contains unknown level/description codes.
    pub fn parse(data: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(data.len() >= 2, "Alert too short: {} bytes (need 2)", data.len());

        let raw_level = data[0];
        let raw_description = data[1];

        let level = AlertLevel::from_u8(raw_level)
            .ok_or_else(|| anyhow::anyhow!("Unknown alert level: {raw_level}"))?;

        let description = AlertDescription::from_u8(raw_description)
            .ok_or_else(|| anyhow::anyhow!("Unknown alert description: {raw_description}"))?;

        Ok(Self {
            level,
            description,
            raw_level,
            raw_description,
        })
    }

    /// Check if this is a fatal alert
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(self.level, AlertLevel::Fatal)
    }

    /// Get formatted display string with full details
    #[must_use]
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
        let err = TlsAlert::parse(&data).unwrap_err().to_string();
        assert!(err.contains("too short"));
    }

    #[test]
    fn test_parse_unknown_level() {
        let data = [99, 0]; // Invalid level
        let err = TlsAlert::parse(&data).unwrap_err().to_string();
        assert!(err.contains("Unknown alert level"));
    }

    #[test]
    fn test_parse_unknown_description() {
        let data = [2, 99]; // Invalid description
        let err = TlsAlert::parse(&data).unwrap_err().to_string();
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
