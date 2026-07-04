// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Codec implementations for TLS messages

use super::bytes::{
    read_u8, read_u16, read_vec8, read_vec16, write_u8, write_u16, write_vec8, write_vec16,
};
use super::{Decode, Encode};
use crate::error::{Result, TlsError};
use crate::messages::{ClientHello, Extension, ServerHello, extensions};

// ============================================================================
// Extension Encoding/Decoding
// ============================================================================

impl Encode for Extension {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()> {
        // Write extension type (u16)
        write_u16(buf, self.extension_type());

        // Write extension data (length-prefixed u16)
        let data_buf = match self {
            Self::SupportedVersions(versions) => {
                let mut data = Vec::new();
                let len_bytes = versions.len() * 2;
                let len_u8 = u8::try_from(len_bytes).map_err(|_| {
                    TlsError::InvalidParameter(String::from("SupportedVersions too long"))
                })?;
                write_u8(&mut data, len_u8); // Length in bytes
                for version in versions {
                    write_u16(&mut data, *version);
                }
                data
            }
            Self::KeyShare(key_data) => {
                let mut data = Vec::new();
                write_u16(&mut data, extensions::GROUP_X25519); // Named group
                write_vec16(&mut data, key_data)?;
                data
            }
            Self::ServerName(name) => {
                let mut data = Vec::new();
                let name_bytes = name.as_bytes();

                // SNI extension format (RFC 6066 Section 3):
                // - Server name list length (u16)
                // - Server name type (u8): 0x00 = host_name
                // - Host name length (u16)
                // - Host name bytes

                // Server name list length = type (1) + length (2) + name bytes
                let list_len = name_bytes.len() + 3;
                let list_len_u16 = u16::try_from(list_len).map_err(|_| {
                    TlsError::InvalidParameter(String::from("Server name list too long"))
                })?;
                write_u16(&mut data, list_len_u16);
                write_u8(&mut data, 0x00); // Type: host_name
                let name_len_u16 = u16::try_from(name_bytes.len()).map_err(|_| {
                    TlsError::InvalidParameter(String::from("Server name too long"))
                })?;
                write_u16(&mut data, name_len_u16);
                data.extend_from_slice(name_bytes);

                data
            }
            Self::SignatureAlgorithms(algs) => {
                let mut data = Vec::new();
                let algs_len = u16::try_from(algs.len() * 2).map_err(|_| {
                    TlsError::InvalidParameter(String::from("SignatureAlgorithms too long"))
                })?;
                write_u16(&mut data, algs_len);
                for alg in algs {
                    write_u16(&mut data, *alg);
                }
                data
            }
            Self::SupportedGroups(groups) => {
                let mut data = Vec::new();
                let groups_len = u16::try_from(groups.len() * 2).map_err(|_| {
                    TlsError::InvalidParameter(String::from("SupportedGroups too long"))
                })?;
                write_u16(&mut data, groups_len);
                for group in groups {
                    write_u16(&mut data, *group);
                }
                data
            }
            Self::Unknown {
                data,
                ..
            } => data.clone(),
        };

        write_vec16(buf, &data_buf)?;
        Ok(())
    }

    fn encoded_size(&self) -> usize {
        // Type (2) + length (2) + data
        4 + match self {
            Self::SupportedVersions(v) => 1 + v.len() * 2,
            Self::KeyShare(k) => 2 + 2 + k.len(),
            Self::ServerName(name) => {
                // list_length (2) + type (1) + name_length (2) + name_bytes
                2 + 1 + 2 + name.len()
            }
            Self::SignatureAlgorithms(a) => 2 + a.len() * 2,
            Self::SupportedGroups(g) => 2 + g.len() * 2,
            Self::Unknown {
                data,
                ..
            } => data.len(),
        }
    }
}

// ============================================================================
// ClientHello Encoding/Decoding
// ============================================================================

impl Encode for ClientHello {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()> {
        // Legacy version (u16)
        write_u16(buf, self.legacy_version);

        // Random (32 bytes)
        buf.extend_from_slice(&self.random);

        // Legacy session ID (u8 length + data)
        write_vec8(buf, &self.legacy_session_id)?;

        // Cipher suites (u16 length + u16 values)
        let cipher_len = u16::try_from(self.cipher_suites.len() * 2)
            .map_err(|_| TlsError::InvalidParameter(String::from("Cipher suites list too long")))?;
        write_u16(buf, cipher_len);
        for suite in &self.cipher_suites {
            write_u16(buf, *suite);
        }

        // Legacy compression methods (u8 length + u8 values)
        write_vec8(buf, &self.legacy_compression_methods)?;

        // Extensions (u16 length + extensions)
        let mut ext_buf = Vec::new();
        for ext in &self.extensions {
            ext.encode(&mut ext_buf)?;
        }
        write_vec16(buf, &ext_buf)?;

        Ok(())
    }

    fn encoded_size(&self) -> usize {
        2 + // legacy_version
        32 + // random
        1 + self.legacy_session_id.len() + // session_id
        2 + (self.cipher_suites.len() * 2) + // cipher_suites
        1 + self.legacy_compression_methods.len() + // compression
        2 + self.extensions.iter().map(Encode::encoded_size).sum::<usize>() // extensions
    }
}

impl Decode for ClientHello {
    fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        let mut offset = 0;

        // Legacy version
        let legacy_version = read_u16(buf, &mut offset)?;

        // Random (32 bytes)
        if offset + 32 > buf.len() {
            return Err(TlsError::ProtocolError(String::from(
                "ClientHello: not enough data for random",
            )));
        }
        let mut random = [0u8; 32];
        random.copy_from_slice(&buf[offset..offset + 32]);
        offset += 32;

        // Legacy session ID
        let legacy_session_id = read_vec8(buf, &mut offset)?;

        // Cipher suites (each suite is 2 bytes, so length must be even)
        let cipher_suites_len = read_u16(buf, &mut offset)? as usize;
        if !cipher_suites_len.is_multiple_of(2) {
            return Err(TlsError::ProtocolError(String::from(
                "ClientHello: cipher suites length must be even",
            )));
        }
        let mut cipher_suites = Vec::new();
        for _ in 0..(cipher_suites_len / 2) {
            cipher_suites.push(read_u16(buf, &mut offset)?);
        }

        // Legacy compression methods
        let legacy_compression_methods = read_vec8(buf, &mut offset)?;

        // Extensions
        let extensions_data = read_vec16(buf, &mut offset)?;
        let mut extensions = Vec::new();
        let mut ext_offset = 0;
        while ext_offset < extensions_data.len() {
            let ext_type = read_u16(&extensions_data, &mut ext_offset)?;
            let ext_data = read_vec16(&extensions_data, &mut ext_offset)?;

            // Parse known extensions
            let extension = match ext_type {
                extensions::EXT_SUPPORTED_VERSIONS => {
                    let mut versions = Vec::new();
                    let mut ver_offset = 1; // Skip length byte
                    while ver_offset < ext_data.len() {
                        versions.push(read_u16(&ext_data, &mut ver_offset)?);
                    }
                    Extension::SupportedVersions(versions)
                }
                extensions::EXT_KEY_SHARE => {
                    // Simplified: just extract key data
                    let mut ks_offset = 0;
                    let _group = read_u16(&ext_data, &mut ks_offset)?;
                    let key_data = read_vec16(&ext_data, &mut ks_offset)?;
                    Extension::KeyShare(key_data)
                }
                _ => Extension::Unknown {
                    extension_type: ext_type,
                    data: ext_data,
                },
            };
            extensions.push(extension);
        }

        Ok((
            Self {
                legacy_version,
                random,
                legacy_session_id,
                cipher_suites,
                legacy_compression_methods,
                extensions,
            },
            offset,
        ))
    }
}

// ============================================================================
// ServerHello Encoding/Decoding
// ============================================================================

impl Encode for ServerHello {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()> {
        // Legacy version (u16)
        write_u16(buf, self.legacy_version);

        // Random (32 bytes)
        buf.extend_from_slice(&self.random);

        // Legacy session ID echo (u8 length + data)
        write_vec8(buf, &self.legacy_session_id_echo)?;

        // Cipher suite (u16)
        write_u16(buf, self.cipher_suite);

        // Legacy compression method (u8)
        write_u8(buf, self.legacy_compression_method);

        // Extensions (u16 length + extensions)
        let mut ext_buf = Vec::new();
        for ext in &self.extensions {
            ext.encode(&mut ext_buf)?;
        }
        write_vec16(buf, &ext_buf)?;

        Ok(())
    }

    fn encoded_size(&self) -> usize {
        2 + // legacy_version
        32 + // random
        1 + self.legacy_session_id_echo.len() + // session_id_echo
        2 + // cipher_suite
        1 + // compression_method
        2 + self.extensions.iter().map(Encode::encoded_size).sum::<usize>() // extensions
    }
}

impl Decode for ServerHello {
    fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        let mut offset = 0;

        // Legacy version
        let legacy_version = read_u16(buf, &mut offset)?;

        // Random (32 bytes)
        if offset + 32 > buf.len() {
            return Err(TlsError::ProtocolError(String::from(
                "ServerHello: not enough data for random",
            )));
        }
        let mut random = [0u8; 32];
        random.copy_from_slice(&buf[offset..offset + 32]);
        offset += 32;

        // Legacy session ID echo
        let legacy_session_id_echo = read_vec8(buf, &mut offset)?;

        // Cipher suite
        let cipher_suite = read_u16(buf, &mut offset)?;

        // Legacy compression method
        let legacy_compression_method = read_u8(buf, &mut offset)?;

        // Extensions (similar to ClientHello)
        let extensions_data = read_vec16(buf, &mut offset)?;
        let mut extensions = Vec::new();
        let mut ext_offset = 0;
        while ext_offset < extensions_data.len() {
            let ext_type = read_u16(&extensions_data, &mut ext_offset)?;
            let ext_data = read_vec16(&extensions_data, &mut ext_offset)?;

            let extension = match ext_type {
                extensions::EXT_SUPPORTED_VERSIONS => {
                    let mut ver_offset = 0;
                    let version = read_u16(&ext_data, &mut ver_offset)?;
                    Extension::SupportedVersions(vec![version])
                }
                extensions::EXT_KEY_SHARE => {
                    let mut ks_offset = 0;
                    let _group = read_u16(&ext_data, &mut ks_offset)?;
                    let key_data = read_vec16(&ext_data, &mut ks_offset)?;
                    Extension::KeyShare(key_data)
                }
                _ => Extension::Unknown {
                    extension_type: ext_type,
                    data: ext_data,
                },
            };
            extensions.push(extension);
        }

        Ok((
            Self {
                legacy_version,
                random,
                legacy_session_id_echo,
                cipher_suite,
                legacy_compression_method,
                extensions,
            },
            offset,
        ))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::codec::bytes::{write_u16, write_vec8};
    use crate::messages::certificate::CertificateEntry;
    use crate::messages::{Certificate, CertificateVerify, Finished, HandshakeType};

    fn roundtrip_client_hello(hello: &ClientHello) -> ClientHello {
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();
        assert_eq!(hello.encoded_size(), buf.len());
        let (decoded, bytes_read) = ClientHello::decode(&buf).unwrap();
        assert_eq!(bytes_read, buf.len());
        decoded
    }

    fn roundtrip_server_hello(hello: &ServerHello) -> ServerHello {
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();
        assert_eq!(hello.encoded_size(), buf.len());
        let (decoded, bytes_read) = ServerHello::decode(&buf).unwrap();
        assert_eq!(bytes_read, buf.len());
        decoded
    }

    #[test]
    fn test_client_hello_encode_decode() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303]; // TLS_CHACHA20_POLY1305_SHA256
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            Extension::KeyShare(vec![1, 2, 3, 4, 5, 6, 7, 8]),
        ];

        let hello = ClientHello::new(random, cipher_suites, extensions);

        // Encode
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();

        // Decode
        let (decoded, bytes_read) = ClientHello::decode(&buf).unwrap();

        assert_eq!(bytes_read, buf.len());
        assert_eq!(decoded.legacy_version, hello.legacy_version);
        assert_eq!(decoded.random, hello.random);
        assert_eq!(decoded.cipher_suites, vec![0x1303]);
    }

    #[test]
    fn test_server_hello_encode_decode() {
        let random = [99u8; 32];
        let cipher_suite = 0x1303;
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]),
            Extension::KeyShare(vec![9, 10, 11, 12]),
        ];

        let hello = ServerHello::new(random, vec![], cipher_suite, extensions);

        // Encode
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();

        // Decode
        let (decoded, bytes_read) = ServerHello::decode(&buf).unwrap();

        assert_eq!(bytes_read, buf.len());
        assert_eq!(decoded.legacy_version, hello.legacy_version);
        assert_eq!(decoded.random, hello.random);
        assert_eq!(decoded.cipher_suite, cipher_suite);
    }

    #[test]
    fn test_extension_supported_versions_encode() {
        let ext = Extension::SupportedVersions(vec![0x0304, 0x0303]);
        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();

        // Should be: type (2) + length (2) + versions_length (1) + versions (4)
        assert_eq!(buf.len(), 2 + 2 + 1 + 4);
        assert_eq!(&buf[0..2], &[0x00, 0x2b]); // EXT_SUPPORTED_VERSIONS = 43
    }

    #[test]
    fn test_extension_key_share_encode() {
        let key_data = vec![1, 2, 3, 4];
        let ext = Extension::KeyShare(key_data);
        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();

        // Should contain: type (2) + length (2) + group (2) + key_length (2) + key_data
        assert!(buf.len() > 8);
        assert_eq!(&buf[0..2], &[0x00, 0x33]); // EXT_KEY_SHARE = 51
    }

    #[test]
    fn test_extension_server_name_encode() {
        // Test SNI encoding for "example.com"
        let server_name = String::from("example.com");
        let ext = Extension::ServerName(server_name.clone());
        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();

        // Expected format (RFC 6066 Section 3):
        // - Extension type (u16): 0x0000 (server_name)
        // - Extension length (u16): list_length + 2
        // - Server name list length (u16): type + name_length + name_bytes
        // - Name type (u8): 0x00 (host_name)
        // - Name length (u16): len("example.com") = 11
        // - Name bytes: "example.com"

        let name_bytes = server_name.as_bytes();
        let expected_list_len = 1 + 2 + name_bytes.len(); // type (1) + length (2) + name
        let expected_ext_len = 2 + expected_list_len; // list_length field (2) + list content

        // Total: type (2) + ext_length (2) + list_length (2) + type (1) + name_length (2) + name
        let expected_total = 2 + 2 + 2 + 1 + 2 + name_bytes.len();

        assert_eq!(buf.len(), expected_total);

        // Verify extension type (SNI = 0x0000)
        assert_eq!(&buf[0..2], &[0x00, 0x00]);

        // Verify extension length
        let ext_len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
        assert_eq!(ext_len, expected_ext_len);

        // Verify server name list length
        let list_len = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        assert_eq!(list_len, expected_list_len);

        // Verify name type (host_name = 0x00)
        assert_eq!(buf[6], 0x00);

        // Verify name length
        let name_len = u16::from_be_bytes([buf[7], buf[8]]) as usize;
        assert_eq!(name_len, name_bytes.len());

        // Verify name bytes
        assert_eq!(&buf[9..9 + name_bytes.len()], name_bytes);
    }

    #[test]
    fn test_extension_server_name_encoded_size() {
        let server_name = String::from("example.com");
        let ext = Extension::ServerName(server_name);

        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();

        // Verify encoded_size() matches actual encoded length
        assert_eq!(ext.encoded_size(), buf.len());
    }

    #[test]
    fn client_hello_empty_extensions_list_roundtrip() {
        let hello = ClientHello {
            legacy_version: 0x0303,
            random: [7u8; 32],
            legacy_session_id: vec![],
            cipher_suites: vec![0x1303],
            legacy_compression_methods: vec![0],
            extensions: vec![],
        };
        let decoded = roundtrip_client_hello(&hello);
        assert!(decoded.extensions.is_empty());
    }

    #[test]
    fn client_hello_multiple_extensions_roundtrip() {
        let hello = ClientHello::new(
            [1u8; 32],
            vec![0x1301, 0x1302, 0x1303],
            vec![
                Extension::SupportedVersions(vec![0x0303, 0x0304]),
                Extension::KeyShare(vec![0xAB; 32]),
                Extension::ServerName(String::from("tls.example")),
                Extension::SignatureAlgorithms(vec![0x0807]),
                Extension::SupportedGroups(vec![0x001d]),
                Extension::Unknown {
                    extension_type: 0xBEEF,
                    data: vec![9, 8, 7],
                },
            ],
        );
        let decoded = roundtrip_client_hello(&hello);
        assert_eq!(decoded.cipher_suites.len(), 3);
        assert_eq!(decoded.extensions.len(), 6);
        assert_eq!(decoded.get_key_share(), Some([0xAB; 32].as_slice()));
    }

    #[test]
    fn client_hello_max_session_id_and_cipher_suites_roundtrip() {
        let hello = ClientHello {
            legacy_version: 0x0303,
            random: [0xCC; 32],
            legacy_session_id: vec![0xDD; 32],
            cipher_suites: vec![0x1303; 100],
            legacy_compression_methods: vec![0],
            extensions: vec![
                Extension::SupportedVersions(vec![0x0304]),
                Extension::KeyShare(vec![1, 2, 3, 4]),
            ],
        };
        let decoded = roundtrip_client_hello(&hello);
        assert_eq!(decoded.legacy_session_id.len(), 32);
        assert_eq!(decoded.cipher_suites.len(), 100);
    }

    #[test]
    fn client_hello_decode_truncated_before_random() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x0303);
        buf.push(0x01);
        let err = ClientHello::decode(&buf).unwrap_err();
        assert!(matches!(err, TlsError::ProtocolError(_)));
    }

    #[test]
    fn client_hello_decode_odd_cipher_suites_length() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x0303);
        buf.extend_from_slice(&[0u8; 32]);
        write_vec8(&mut buf, &[]).unwrap();
        write_u16(&mut buf, 3); // odd length — invalid
        buf.extend_from_slice(&[0x13, 0x03, 0x13]);
        let err = ClientHello::decode(&buf).unwrap_err();
        assert!(matches!(err, TlsError::ProtocolError(_)));
    }

    #[test]
    fn client_hello_decode_truncated_extensions_block() {
        let hello = ClientHello::new(
            [2u8; 32],
            vec![0x1303],
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])],
        );
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();
        buf.truncate(buf.len() - 2);
        assert!(ClientHello::decode(&buf).is_err());
    }

    #[test]
    fn server_hello_session_id_echo_and_unknown_extension_roundtrip() {
        let hello = ServerHello::new(
            [0xEE; 32],
            vec![1, 2, 3, 4, 5],
            0x1302,
            vec![
                Extension::SupportedVersions(vec![0x0304]),
                Extension::KeyShare(vec![0x11; 32]),
                Extension::Unknown {
                    extension_type: 0x1234,
                    data: vec![0xAA, 0xBB],
                },
            ],
        );
        let decoded = roundtrip_server_hello(&hello);
        assert_eq!(decoded.legacy_session_id_echo, vec![1, 2, 3, 4, 5]);
        assert_eq!(decoded.cipher_suite, 0x1302);
        assert!(decoded.extensions.iter().any(|e| {
            matches!(
                e,
                Extension::Unknown {
                    extension_type: 0x1234,
                    ..
                }
            )
        }));
    }

    #[test]
    fn server_hello_encrypted_extensions_style_empty_extension_data_roundtrip() {
        // EncryptedExtensions body is an extensions block; model via ServerHello extensions.
        let hello = ServerHello::new(
            [0x55; 32],
            vec![],
            0x1303,
            vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![0x22; 32])],
        );
        let decoded = roundtrip_server_hello(&hello);
        assert_eq!(decoded.extensions.len(), 2);
    }

    #[test]
    fn handshake_type_byte_roundtrips_for_all_handshake_messages() {
        let cases = [
            (HandshakeType::ClientHello, 1u8),
            (HandshakeType::ServerHello, 2),
            (HandshakeType::EncryptedExtensions, 8),
            (HandshakeType::Certificate, 11),
            (HandshakeType::CertificateVerify, 15),
            (HandshakeType::Finished, 20),
        ];
        for (ht, byte) in cases {
            assert_eq!(u8::from(ht), byte);
            assert_eq!(HandshakeType::try_from(byte).unwrap(), ht);
        }
    }

    #[test]
    fn handshake_type_invalid_byte_returns_protocol_error() {
        let err = HandshakeType::try_from(0xFF).unwrap_err();
        assert!(
            matches!(err, TlsError::ProtocolError(msg) if msg.contains("Invalid handshake type"))
        );
    }

    #[test]
    fn handshake_message_types_have_distinct_type_bytes() {
        // Domain types without wire codecs yet — ensure type discriminants stay stable.
        let _cert = Certificate::new(vec![CertificateEntry::new(vec![1, 2, 3])]);
        let _verify = CertificateVerify::new(0x0807, vec![0u8; 64]);
        let _finished = Finished::new(vec![0u8; 32]);
        assert_ne!(u8::from(HandshakeType::Certificate), u8::from(HandshakeType::Finished));
        assert_ne!(
            u8::from(HandshakeType::CertificateVerify),
            u8::from(HandshakeType::EncryptedExtensions)
        );
    }

    #[test]
    fn extension_signature_algorithms_encode_decode_via_client_hello() {
        let hello = ClientHello::new(
            [3u8; 32],
            vec![0x1303],
            vec![
                Extension::SupportedVersions(vec![0x0304]),
                Extension::KeyShare(vec![1, 2, 3, 4]),
                Extension::SignatureAlgorithms(vec![0x0807, 0x0403]),
            ],
        );
        let decoded = roundtrip_client_hello(&hello);
        assert_eq!(decoded.extensions.len(), 3);
    }

    #[test]
    fn extension_supported_groups_encode_decode_via_client_hello() {
        let hello = ClientHello::new(
            [4u8; 32],
            vec![0x1303],
            vec![
                Extension::SupportedVersions(vec![0x0304]),
                Extension::KeyShare(vec![1, 2, 3, 4]),
                Extension::SupportedGroups(vec![0x001d, 0x0017]),
            ],
        );
        let decoded = roundtrip_client_hello(&hello);
        assert_eq!(decoded.extensions.len(), 3);
    }
}
