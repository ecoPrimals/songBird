//! Codec implementations for TLS messages

use super::{bytes::*, Decode, Encode};
use crate::error::{Result, TlsError};
use crate::messages::*;

// ============================================================================
// Extension Encoding/Decoding
// ============================================================================

impl Encode for Extension {
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()> {
        // Write extension type (u16)
        write_u16(buf, self.extension_type());

        // Write extension data (length-prefixed u16)
        let data_buf = match self {
            Extension::SupportedVersions(versions) => {
                let mut data = Vec::new();
                write_u8(&mut data, (versions.len() * 2) as u8); // Length in bytes
                for version in versions {
                    write_u16(&mut data, *version);
                }
                data
            }
            Extension::KeyShare(key_data) => {
                let mut data = Vec::new();
                write_u16(&mut data, extensions::GROUP_X25519); // Named group
                write_vec16(&mut data, key_data)?;
                data
            }
            Extension::ServerName(_name) => {
                let mut data = Vec::new();
                write_vec16(&mut data, &[])?; // Server name list
                                              // TODO: Implement full SNI encoding
                data
            }
            Extension::SignatureAlgorithms(algs) => {
                let mut data = Vec::new();
                write_u16(&mut data, (algs.len() * 2) as u16);
                for alg in algs {
                    write_u16(&mut data, *alg);
                }
                data
            }
            Extension::SupportedGroups(groups) => {
                let mut data = Vec::new();
                write_u16(&mut data, (groups.len() * 2) as u16);
                for group in groups {
                    write_u16(&mut data, *group);
                }
                data
            }
            Extension::Unknown {
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
            Extension::SupportedVersions(v) => 1 + v.len() * 2,
            Extension::KeyShare(k) => 2 + 2 + k.len(),
            Extension::ServerName(_) => 2,
            Extension::SignatureAlgorithms(a) => 2 + a.len() * 2,
            Extension::SupportedGroups(g) => 2 + g.len() * 2,
            Extension::Unknown {
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
        write_u16(buf, (self.cipher_suites.len() * 2) as u16);
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
        2 + self.extensions.iter().map(|e| e.encoded_size()).sum::<usize>() // extensions
    }
}

impl Decode for ClientHello {
    fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        let mut offset = 0;

        // Legacy version
        let legacy_version = read_u16(buf, &mut offset)?;

        // Random (32 bytes)
        if offset + 32 > buf.len() {
            return Err(TlsError::ProtocolError(
                "ClientHello: not enough data for random".to_string(),
            ));
        }
        let mut random = [0u8; 32];
        random.copy_from_slice(&buf[offset..offset + 32]);
        offset += 32;

        // Legacy session ID
        let legacy_session_id = read_vec8(buf, &mut offset)?;

        // Cipher suites (each suite is 2 bytes, so length must be even)
        let cipher_suites_len = read_u16(buf, &mut offset)? as usize;
        if !cipher_suites_len.is_multiple_of(2) {
            return Err(TlsError::ProtocolError(
                "ClientHello: cipher suites length must be even".to_string(),
            ));
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
            ClientHello {
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
        2 + self.extensions.iter().map(|e| e.encoded_size()).sum::<usize>() // extensions
    }
}

impl Decode for ServerHello {
    fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        let mut offset = 0;

        // Legacy version
        let legacy_version = read_u16(buf, &mut offset)?;

        // Random (32 bytes)
        if offset + 32 > buf.len() {
            return Err(TlsError::ProtocolError(
                "ServerHello: not enough data for random".to_string(),
            ));
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
            ServerHello {
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
    use super::*;

    #[test]
    fn test_client_hello_encode_decode() {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303]; // TLS_CHACHA20_POLY1305_SHA256
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]), // TLS 1.3
            Extension::KeyShare(vec![1, 2, 3, 4, 5, 6, 7, 8]),
        ];

        let hello = ClientHello::new(random, cipher_suites.clone(), extensions.clone());

        // Encode
        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();

        // Decode
        let (decoded, bytes_read) = ClientHello::decode(&buf).unwrap();

        assert_eq!(bytes_read, buf.len());
        assert_eq!(decoded.legacy_version, hello.legacy_version);
        assert_eq!(decoded.random, hello.random);
        assert_eq!(decoded.cipher_suites, cipher_suites);
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
        let ext = Extension::KeyShare(key_data.clone());
        let mut buf = Vec::new();
        ext.encode(&mut buf).unwrap();

        // Should contain: type (2) + length (2) + group (2) + key_length (2) + key_data
        assert!(buf.len() > 8);
        assert_eq!(&buf[0..2], &[0x00, 0x33]); // EXT_KEY_SHARE = 51
    }
}
