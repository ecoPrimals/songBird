// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! `ClientHello` Parsing
//!
//! Handles parsing and validation of `ClientHello` messages from clients.

use crate::error::{Error, Result};
use crate::tls::handshake_v2::keys::CipherSuite;
use tracing::{debug, info};

use super::core::TlsServer;

impl TlsServer {
    /// Parse `ClientHello` to extract parameters
    ///
    /// Returns: (`client_random`, `client_public_key`, `cipher_suites`)
    pub(super) fn parse_client_hello(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u16>)> {
        let mut offset = 0;

        // Skip handshake header (type + length)
        offset += 4;

        // Skip legacy version (2 bytes)
        offset += 2;

        // Client random (32 bytes)
        if data.len() < offset + 32 {
            return Err(Error::TlsHandshake("ClientHello too short for random".to_string()));
        }
        let client_random = data[offset..offset + 32].to_vec();
        offset += 32;

        // Legacy session ID
        if data.len() < offset + 1 {
            return Err(Error::TlsHandshake(
                "ClientHello truncated at session ID length".to_string(),
            ));
        }
        let session_id_len = data[offset] as usize;
        offset += 1 + session_id_len;

        // Cipher suites
        if data.len() < offset + 2 {
            return Err(Error::TlsHandshake(
                "ClientHello truncated at cipher suites length".to_string(),
            ));
        }
        let cipher_suites_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        let mut cipher_suites = Vec::new();
        for i in 0..cipher_suites_len / 2 {
            let suite = u16::from_be_bytes([data[offset + i * 2], data[offset + i * 2 + 1]]);
            cipher_suites.push(suite);
        }
        offset += cipher_suites_len;

        // Skip compression methods
        if data.len() < offset + 1 {
            return Err(Error::TlsHandshake("ClientHello truncated at compression".to_string()));
        }
        let compression_len = data[offset] as usize;
        offset += 1 + compression_len;

        // Parse extensions to find key_share
        if data.len() < offset + 2 {
            return Err(Error::TlsHandshake(
                "ClientHello truncated at extensions length".to_string(),
            ));
        }
        let extensions_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        let extensions_data = &data[offset..offset + extensions_len];
        let client_public_key = Self::extract_key_share(extensions_data)?;

        info!("✅ Parsed ClientHello:");
        info!("   Client random: {} bytes", client_random.len());
        info!(
            "   Cipher suites: {:?}",
            cipher_suites.iter().map(|s| format!("0x{s:04x}")).collect::<Vec<_>>()
        );
        info!("   Client public key: {} bytes", client_public_key.len());

        Ok((client_random, client_public_key, cipher_suites))
    }

    /// Extract client's public key from `key_share` extension
    fn extract_key_share(extensions_data: &[u8]) -> Result<Vec<u8>> {
        let mut offset = 0;

        while offset + 4 <= extensions_data.len() {
            let ext_type =
                u16::from_be_bytes([extensions_data[offset], extensions_data[offset + 1]]);
            let ext_len =
                u16::from_be_bytes([extensions_data[offset + 2], extensions_data[offset + 3]])
                    as usize;
            offset += 4;

            if ext_type == 0x0033 {
                // key_share
                if offset + ext_len > extensions_data.len() {
                    return Err(Error::TlsHandshake("key_share extension truncated".to_string()));
                }

                let ext_data = &extensions_data[offset..offset + ext_len];

                // KeyShareClientHello: client_shares length + entries
                if ext_data.len() < 2 {
                    return Err(Error::TlsHandshake("key_share extension too short".to_string()));
                }

                let _entries_len = u16::from_be_bytes([ext_data[0], ext_data[1]]) as usize;
                let mut entry_offset = 2;

                // Parse first KeyShareEntry
                if ext_data.len() < entry_offset + 4 {
                    return Err(Error::TlsHandshake("KeyShareEntry too short".to_string()));
                }

                let group =
                    u16::from_be_bytes([ext_data[entry_offset], ext_data[entry_offset + 1]]);
                let key_len =
                    u16::from_be_bytes([ext_data[entry_offset + 2], ext_data[entry_offset + 3]])
                        as usize;
                entry_offset += 4;

                if ext_data.len() < entry_offset + key_len {
                    return Err(Error::TlsHandshake("KeyShareEntry key truncated".to_string()));
                }

                let key = ext_data[entry_offset..entry_offset + key_len].to_vec();

                debug!("   Found key_share: group=0x{:04x}, key_len={}", group, key_len);

                return Ok(key);
            }

            offset += ext_len;
        }

        Err(Error::TlsHandshake("key_share extension not found".to_string()))
    }

    /// Select cipher suite (choose first supported by both client and server)
    pub(super) fn select_cipher_suite(&self, client_suites: &[u16]) -> Result<CipherSuite> {
        // Server supported suites (in order of preference)
        const SERVER_SUITES: &[u16] = &[
            0x1301, // TLS_AES_128_GCM_SHA256
            0x1302, // TLS_AES_256_GCM_SHA384
            0x1303, // TLS_CHACHA20_POLY1305_SHA256
        ];

        for server_suite in SERVER_SUITES {
            if client_suites.contains(server_suite) {
                return CipherSuite::from_u16(*server_suite);
            }
        }

        Err(Error::TlsHandshake(format!("No common cipher suite found. Client: {client_suites:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::BearDogProvider;
    use std::sync::Arc;

    fn create_test_server() -> TlsServer {
        let crypto = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
        TlsServer::new(crypto, vec![], vec![])
    }

    #[test]
    fn test_select_cipher_suite() {
        let server = create_test_server();

        // Client supports AES-128-GCM and ChaCha20
        let client_suites = vec![0x1301, 0x1303];
        let suite = server.select_cipher_suite(&client_suites).unwrap();

        assert_eq!(suite, CipherSuite::Aes128GcmSha256);
    }
}
