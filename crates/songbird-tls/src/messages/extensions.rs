//! TLS extensions (RFC 8446 Section 4.2)
//!
//! Extensions are used to extend the TLS protocol with additional functionality.
//! In TLS 1.3, many extensions are mandatory (e.g., supported_versions, key_share).

/// TLS extension types
///
/// For now, we implement only the essential extensions needed for TLS 1.3.
/// Additional extensions can be added as needed.
#[derive(Debug, Clone, PartialEq)]
pub enum Extension {
    /// supported_versions (RFC 8446 Section 4.2.1)
    ///
    /// In ClientHello: list of supported versions (e.g., [0x0304] for TLS 1.3)
    /// In ServerHello: selected version
    SupportedVersions(Vec<u16>),

    /// key_share (RFC 8446 Section 4.2.8)
    ///
    /// Contains the client's or server's public key for key exchange.
    /// For X25519, this is 32 bytes.
    KeyShare(Vec<u8>),

    /// server_name (RFC 6066 Section 3)
    ///
    /// SNI (Server Name Indication) - hostname the client wants to connect to.
    ServerName(String),

    /// signature_algorithms (RFC 8446 Section 4.2.3)
    ///
    /// List of signature algorithms supported by the client.
    /// For Ed25519: 0x0807
    SignatureAlgorithms(Vec<u16>),

    /// supported_groups (RFC 8446 Section 4.2.7)
    ///
    /// List of named groups (curves) supported by the client.
    /// For X25519: 0x001d
    SupportedGroups(Vec<u16>),

    /// Unknown extension (for forward compatibility)
    ///
    /// Store extension type and data for extensions we don't recognize.
    /// This allows forward compatibility with future TLS versions.
    Unknown {
        extension_type: u16,
        data: Vec<u8>,
    },
}

impl Extension {
    /// Get the extension type code
    pub fn extension_type(&self) -> u16 {
        match self {
            Extension::SupportedVersions(_) => 43,
            Extension::KeyShare(_) => 51,
            Extension::ServerName(_) => 0,
            Extension::SignatureAlgorithms(_) => 13,
            Extension::SupportedGroups(_) => 10,
            Extension::Unknown {
                extension_type,
                ..
            } => *extension_type,
        }
    }

    /// Check if this is a mandatory extension for TLS 1.3
    pub fn is_mandatory_for_tls13(&self) -> bool {
        matches!(self, Extension::SupportedVersions(_) | Extension::KeyShare(_))
    }
}

// Extension type constants
pub const EXT_SERVER_NAME: u16 = 0;
pub const EXT_SUPPORTED_GROUPS: u16 = 10;
pub const EXT_SIGNATURE_ALGORITHMS: u16 = 13;
pub const EXT_SUPPORTED_VERSIONS: u16 = 43;
pub const EXT_KEY_SHARE: u16 = 51;

// Signature algorithm constants
pub const SIG_ED25519: u16 = 0x0807;
pub const SIG_ECDSA_SECP256R1_SHA256: u16 = 0x0403;

// Named group constants
pub const GROUP_X25519: u16 = 0x001d;
pub const GROUP_SECP256R1: u16 = 0x0017;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(Extension::SupportedVersions(vec![]).extension_type(), 43);
        assert_eq!(Extension::KeyShare(vec![]).extension_type(), 51);
        assert_eq!(Extension::ServerName(String::new()).extension_type(), 0);
        assert_eq!(Extension::SignatureAlgorithms(vec![]).extension_type(), 13);
        assert_eq!(Extension::SupportedGroups(vec![]).extension_type(), 10);
    }

    #[test]
    fn test_is_mandatory() {
        assert!(Extension::SupportedVersions(vec![]).is_mandatory_for_tls13());
        assert!(Extension::KeyShare(vec![]).is_mandatory_for_tls13());
        assert!(!Extension::ServerName(String::new()).is_mandatory_for_tls13());
        assert!(!Extension::SignatureAlgorithms(vec![]).is_mandatory_for_tls13());
    }

    #[test]
    fn test_unknown_extension() {
        let ext = Extension::Unknown {
            extension_type: 999,
            data: vec![1, 2, 3],
        };
        assert_eq!(ext.extension_type(), 999);
        assert!(!ext.is_mandatory_for_tls13());
    }

    #[test]
    fn test_constants() {
        assert_eq!(EXT_SUPPORTED_VERSIONS, 43);
        assert_eq!(EXT_KEY_SHARE, 51);
        assert_eq!(SIG_ED25519, 0x0807);
        assert_eq!(GROUP_X25519, 0x001d);
    }
}
