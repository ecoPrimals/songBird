// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS message types
//!
//! Defines all TLS 1.3 message structures per RFC 8446.
//! Pure Rust types with no external dependencies.

pub mod alert;
pub mod certificate;
pub mod certificate_verify;
pub mod client_hello;
pub mod extensions;
pub mod finished;
pub mod server_hello;

pub use alert::Alert;
pub use certificate::Certificate;
pub use certificate_verify::CertificateVerify;
pub use client_hello::ClientHello;
pub use extensions::Extension;
pub use finished::Finished;
pub use server_hello::ServerHello;

/// Content type for TLS records
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ContentType {
    /// Placeholder for unknown or invalid content types.
    Invalid = 0,
    /// Legacy change cipher spec framing.
    ChangeCipherSpec = 20,
    /// Alert protocol payload.
    Alert = 21,
    /// Handshake subprotocol payload.
    Handshake = 22,
    /// Application data payload.
    ApplicationData = 23,
}

impl From<u8> for ContentType {
    fn from(value: u8) -> Self {
        match value {
            20 => Self::ChangeCipherSpec,
            21 => Self::Alert,
            22 => Self::Handshake,
            23 => Self::ApplicationData,
            _ => Self::Invalid,
        }
    }
}

impl From<ContentType> for u8 {
    fn from(ct: ContentType) -> Self {
        ct as Self
    }
}

/// Handshake message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HandshakeType {
    /// Client hello flight.
    ClientHello = 1,
    /// Server hello flight.
    ServerHello = 2,
    /// New session ticket post-handshake message.
    NewSessionTicket = 4,
    /// End of 0-RTT early data marker.
    EndOfEarlyData = 5,
    /// Encrypted extensions in TLS 1.3.
    EncryptedExtensions = 8,
    /// Certificate payload.
    Certificate = 11,
    /// Certificate request (optional client auth).
    CertificateRequest = 13,
    /// Certificate verify (signature over transcript).
    CertificateVerify = 15,
    /// Finished `verify_data` message.
    Finished = 20,
    /// Key update post-handshake message.
    KeyUpdate = 24,
    /// Synthetic transcript hash message.
    MessageHash = 254,
}

impl TryFrom<u8> for HandshakeType {
    type Error = crate::error::TlsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ClientHello),
            2 => Ok(Self::ServerHello),
            4 => Ok(Self::NewSessionTicket),
            5 => Ok(Self::EndOfEarlyData),
            8 => Ok(Self::EncryptedExtensions),
            11 => Ok(Self::Certificate),
            13 => Ok(Self::CertificateRequest),
            15 => Ok(Self::CertificateVerify),
            20 => Ok(Self::Finished),
            24 => Ok(Self::KeyUpdate),
            254 => Ok(Self::MessageHash),
            _ => Err(crate::error::TlsError::ProtocolError(format!(
                "Invalid handshake type: {value}"
            ))),
        }
    }
}

impl From<HandshakeType> for u8 {
    fn from(ht: HandshakeType) -> Self {
        ht as Self
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_content_type_conversion() {
        assert_eq!(ContentType::from(22), ContentType::Handshake);
        assert_eq!(u8::from(ContentType::Handshake), 22);

        assert_eq!(ContentType::from(23), ContentType::ApplicationData);
        assert_eq!(u8::from(ContentType::ApplicationData), 23);

        assert_eq!(ContentType::from(255), ContentType::Invalid);
    }

    #[test]
    fn test_handshake_type_conversion() {
        assert_eq!(HandshakeType::try_from(1).unwrap(), HandshakeType::ClientHello);
        assert_eq!(u8::from(HandshakeType::ClientHello), 1);

        assert_eq!(HandshakeType::try_from(2).unwrap(), HandshakeType::ServerHello);
        assert_eq!(u8::from(HandshakeType::ServerHello), 2);
    }

    #[test]
    fn content_type_change_cipher_spec_roundtrip() {
        assert_eq!(ContentType::from(20), ContentType::ChangeCipherSpec);
        assert_eq!(u8::from(ContentType::ChangeCipherSpec), 20);
    }

    #[test]
    fn content_type_alert_roundtrip() {
        assert_eq!(ContentType::from(21), ContentType::Alert);
        assert_eq!(u8::from(ContentType::Alert), 21);
    }

    #[test]
    fn handshake_type_all_rfc8446_variants() {
        assert_eq!(HandshakeType::try_from(4).unwrap(), HandshakeType::NewSessionTicket);
        assert_eq!(HandshakeType::try_from(5).unwrap(), HandshakeType::EndOfEarlyData);
        assert_eq!(HandshakeType::try_from(8).unwrap(), HandshakeType::EncryptedExtensions);
        assert_eq!(HandshakeType::try_from(13).unwrap(), HandshakeType::CertificateRequest);
        assert_eq!(HandshakeType::try_from(15).unwrap(), HandshakeType::CertificateVerify);
        assert_eq!(HandshakeType::try_from(24).unwrap(), HandshakeType::KeyUpdate);
        assert_eq!(HandshakeType::try_from(254).unwrap(), HandshakeType::MessageHash);
    }

    #[test]
    fn handshake_type_invalid_returns_protocol_error() {
        let err = HandshakeType::try_from(99).unwrap_err();
        assert!(matches!(err, crate::error::TlsError::ProtocolError(_)));
    }

    #[test]
    fn handshake_type_byte_tags_differ() {
        assert_ne!(u8::from(HandshakeType::ClientHello), u8::from(HandshakeType::ServerHello));
    }

    #[test]
    fn content_type_eq_matches_discriminant() {
        assert_eq!(ContentType::Handshake, ContentType::Handshake);
        assert_ne!(ContentType::Handshake, ContentType::ApplicationData);
    }
}
