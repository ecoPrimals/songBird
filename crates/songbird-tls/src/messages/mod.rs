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
    Invalid = 0,
    ChangeCipherSpec = 20,
    Alert = 21,
    Handshake = 22,
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
    ClientHello = 1,
    ServerHello = 2,
    NewSessionTicket = 4,
    EndOfEarlyData = 5,
    EncryptedExtensions = 8,
    Certificate = 11,
    CertificateRequest = 13,
    CertificateVerify = 15,
    Finished = 20,
    KeyUpdate = 24,
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
}
