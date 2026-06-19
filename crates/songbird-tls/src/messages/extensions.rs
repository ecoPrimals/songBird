// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS extensions (RFC 8446 Section 4.2)
//!
//! Extensions are used to extend the TLS protocol with additional functionality.
//! In TLS 1.3, many extensions are mandatory (e.g., `supported_versions`, `key_share`).

/// TLS extension types
///
/// For now, we implement only the essential extensions needed for TLS 1.3.
/// Additional extensions can be added as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extension {
    /// `supported_versions` (RFC 8446 Section 4.2.1)
    ///
    /// In `ClientHello`: list of supported versions (e.g., `[0x0304]` for TLS 1.3)
    /// In `ServerHello`: selected version
    SupportedVersions(Vec<u16>),

    /// `key_share` (RFC 8446 Section 4.2.8)
    ///
    /// Contains the client's or server's public key for key exchange.
    /// For X25519, this is 32 bytes.
    KeyShare(Vec<u8>),

    /// `server_name` (RFC 6066 Section 3)
    ///
    /// SNI (Server Name Indication) - hostname the client wants to connect to.
    ServerName(String),

    /// `signature_algorithms` (RFC 8446 Section 4.2.3)
    ///
    /// List of signature algorithms supported by the client.
    /// For Ed25519: 0x0807
    SignatureAlgorithms(Vec<u16>),

    /// `supported_groups` (RFC 8446 Section 4.2.7)
    ///
    /// List of named groups (curves) supported by the client.
    /// For X25519: 0x001d
    SupportedGroups(Vec<u16>),

    /// Unknown extension (for forward compatibility)
    ///
    /// Store extension type and data for extensions we don't recognize.
    /// This allows forward compatibility with future TLS versions.
    Unknown {
        /// On-wire extension type id.
        extension_type: u16,
        /// Raw extension body.
        data: Vec<u8>,
    },
}

impl Extension {
    /// Get the extension type code
    #[must_use]
    pub const fn extension_type(&self) -> u16 {
        match self {
            Self::SupportedVersions(_) => 43,
            Self::KeyShare(_) => 51,
            Self::ServerName(_) => 0,
            Self::SignatureAlgorithms(_) => 13,
            Self::SupportedGroups(_) => 10,
            Self::Unknown {
                extension_type,
                ..
            } => *extension_type,
        }
    }

    /// Check if this is a mandatory extension for TLS 1.3
    #[must_use]
    pub const fn is_mandatory_for_tls13(&self) -> bool {
        matches!(self, Self::SupportedVersions(_) | Self::KeyShare(_))
    }
}

// Extension type constants
/// `server_name` extension type.
pub const EXT_SERVER_NAME: u16 = 0;
/// `supported_groups` extension type.
pub const EXT_SUPPORTED_GROUPS: u16 = 10;
/// `signature_algorithms` extension type.
pub const EXT_SIGNATURE_ALGORITHMS: u16 = 13;
/// `supported_versions` extension type.
pub const EXT_SUPPORTED_VERSIONS: u16 = 43;
/// `key_share` extension type.
pub const EXT_KEY_SHARE: u16 = 51;

// Signature algorithm constants
/// Ed25519 codepoint for signature algorithms extension.
pub const SIG_ED25519: u16 = 0x0807;
/// ECDSA P-256 with SHA-256 codepoint.
pub const SIG_ECDSA_SECP256R1_SHA256: u16 = 0x0403;

// Named group constants
/// X25519 named group id.
pub const GROUP_X25519: u16 = 0x001d;
/// secp256r1 named group id.
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

    use crate::codec::Encode;
    use crate::codec::bytes::{read_u16, read_vec16};
    use crate::error::Result;

    fn decode_extension_wire(buf: &[u8]) -> Result<Extension> {
        let mut offset = 0;
        let ext_type = read_u16(buf, &mut offset)?;
        let ext_data = read_vec16(buf, &mut offset)?;
        match ext_type {
            EXT_SUPPORTED_VERSIONS => {
                let mut versions = Vec::new();
                let mut ver_offset = 1;
                while ver_offset < ext_data.len() {
                    versions.push(read_u16(&ext_data, &mut ver_offset)?);
                }
                Ok(Extension::SupportedVersions(versions))
            }
            EXT_KEY_SHARE => {
                let mut ks_offset = 0;
                let _group = read_u16(&ext_data, &mut ks_offset)?;
                let key_data = read_vec16(&ext_data, &mut ks_offset)?;
                Ok(Extension::KeyShare(key_data))
            }
            EXT_SIGNATURE_ALGORITHMS => {
                let mut off = 0;
                let algs_len = read_u16(&ext_data, &mut off)? as usize;
                let end = off + algs_len;
                let mut algs = Vec::new();
                while off < end {
                    algs.push(read_u16(&ext_data, &mut off)?);
                }
                Ok(Extension::SignatureAlgorithms(algs))
            }
            EXT_SUPPORTED_GROUPS => {
                let mut off = 0;
                let groups_len = read_u16(&ext_data, &mut off)? as usize;
                let end = off + groups_len;
                let mut groups = Vec::new();
                while off < end {
                    groups.push(read_u16(&ext_data, &mut off)?);
                }
                Ok(Extension::SupportedGroups(groups))
            }
            EXT_SERVER_NAME => Ok(Extension::Unknown {
                extension_type: ext_type,
                data: ext_data,
            }),
            t => Ok(Extension::Unknown {
                extension_type: t,
                data: ext_data,
            }),
        }
    }

    fn assert_extension_roundtrip(ext: &Extension) {
        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();
        assert_eq!(ext.encoded_size(), buf.len());
        let decoded = decode_extension_wire(&buf).unwrap();
        assert_eq!(decoded, *ext);
    }

    #[test]
    fn extension_supported_versions_empty_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedVersions(vec![]));
    }

    #[test]
    fn extension_supported_versions_single_tls13_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedVersions(vec![0x0304]));
    }

    #[test]
    fn extension_supported_versions_multiple_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedVersions(vec![0x0303, 0x0304]));
    }

    #[test]
    fn extension_key_share_empty_roundtrip() {
        assert_extension_roundtrip(&Extension::KeyShare(vec![]));
    }

    #[test]
    fn extension_key_share_x25519_length_roundtrip() {
        assert_extension_roundtrip(&Extension::KeyShare(vec![0xAB; 32]));
    }

    #[test]
    fn extension_signature_algorithms_empty_roundtrip() {
        assert_extension_roundtrip(&Extension::SignatureAlgorithms(vec![]));
    }

    #[test]
    fn extension_signature_algorithms_single_ed25519_roundtrip() {
        assert_extension_roundtrip(&Extension::SignatureAlgorithms(vec![SIG_ED25519]));
    }

    #[test]
    fn extension_signature_algorithms_multiple_roundtrip() {
        assert_extension_roundtrip(&Extension::SignatureAlgorithms(vec![
            SIG_ECDSA_SECP256R1_SHA256,
            SIG_ED25519,
        ]));
    }

    #[test]
    fn extension_signature_algorithms_max_u16_roundtrip() {
        assert_extension_roundtrip(&Extension::SignatureAlgorithms(vec![0xFFFF]));
    }

    #[test]
    fn extension_supported_groups_empty_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedGroups(vec![]));
    }

    #[test]
    fn extension_supported_groups_single_x25519_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedGroups(vec![GROUP_X25519]));
    }

    #[test]
    fn extension_supported_groups_multiple_roundtrip() {
        assert_extension_roundtrip(&Extension::SupportedGroups(vec![
            GROUP_SECP256R1,
            GROUP_X25519,
        ]));
    }

    #[test]
    fn extension_unknown_roundtrip() {
        assert_extension_roundtrip(&Extension::Unknown {
            extension_type: 0xBEEF,
            data: vec![1, 2, 3, 4, 5],
        });
    }

    #[test]
    fn extension_unknown_empty_data_roundtrip() {
        assert_extension_roundtrip(&Extension::Unknown {
            extension_type: 0x1234,
            data: vec![],
        });
    }

    #[test]
    fn extension_server_name_not_mandatory_for_tls13() {
        let ext = Extension::ServerName("example.com".to_string());
        assert!(!ext.is_mandatory_for_tls13());
        assert_eq!(ext.extension_type(), EXT_SERVER_NAME);
    }

    #[test]
    fn extension_supported_groups_not_mandatory_for_tls13() {
        assert!(!Extension::SupportedGroups(vec![GROUP_X25519]).is_mandatory_for_tls13());
    }
}
