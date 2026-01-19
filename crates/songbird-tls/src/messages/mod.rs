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
            20 => ContentType::ChangeCipherSpec,
            21 => ContentType::Alert,
            22 => ContentType::Handshake,
            23 => ContentType::ApplicationData,
            _ => ContentType::Invalid,
        }
    }
}

impl From<ContentType> for u8 {
    fn from(ct: ContentType) -> Self {
        ct as u8
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

impl From<u8> for HandshakeType {
    fn from(value: u8) -> Self {
        match value {
            1 => HandshakeType::ClientHello,
            2 => HandshakeType::ServerHello,
            4 => HandshakeType::NewSessionTicket,
            5 => HandshakeType::EndOfEarlyData,
            8 => HandshakeType::EncryptedExtensions,
            11 => HandshakeType::Certificate,
            13 => HandshakeType::CertificateRequest,
            15 => HandshakeType::CertificateVerify,
            20 => HandshakeType::Finished,
            24 => HandshakeType::KeyUpdate,
            254 => HandshakeType::MessageHash,
            _ => panic!("Invalid handshake type: {}", value), // Will be handled properly in codec
        }
    }
}

impl From<HandshakeType> for u8 {
    fn from(ht: HandshakeType) -> Self {
        ht as u8
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
        assert_eq!(HandshakeType::from(1), HandshakeType::ClientHello);
        assert_eq!(u8::from(HandshakeType::ClientHello), 1);

        assert_eq!(HandshakeType::from(2), HandshakeType::ServerHello);
        assert_eq!(u8::from(HandshakeType::ServerHello), 2);
    }
}
