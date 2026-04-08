// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 record layer I/O operations
//!
//! Handles reading and decrypting TLS records from TCP streams.
//! Implements RFC 8446 record layer protocol with comprehensive logging.

use super::core::TlsHandshake;
use crate::crypto::TlsHandshakeSecrets as TlsSecrets;
use crate::error::{Error, Result};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::{debug, error, info, trace, warn};

impl TlsHandshake {
    /// Read a TLS record (generic, works for any record type)
    ///
    /// Returns the content type byte (e.g., 0x14=ChangeCipherSpec, 0x17=ApplicationData)
    /// and the record content
    #[expect(clippy::too_many_lines, reason = "Record parsing has many validation branches")]
    pub(super) async fn read_record(&self, stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> {
        // Read record header
        trace!("Reading TLS record header (5 bytes)");
        let mut header = [0u8; 5];
        let header_start = std::time::Instant::now();
        stream.read_exact(&mut header).await.map_err(|e| {
            error!("❌ Failed to read TLS record header: {}", e);
            Error::Io(e)
        })?;
        trace!("Read header in {:?}: {:02x?}", header_start.elapsed(), header);

        let content_type = header[0];
        let version = u16::from_be_bytes([header[1], header[2]]);
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        let content_type_name = match content_type {
            0x14 => "ChangeCipherSpec",
            0x15 => "Alert",
            0x16 => "Handshake",
            0x17 => "ApplicationData",
            _ => "Unknown",
        };

        debug!(
            "📥 TLS record: type={:#04x} ({}), version={:#06x}, length={} bytes",
            content_type, content_type_name, version, length
        );

        // Special handling for Alert records
        if content_type == 0x15 {
            warn!("⚠️  Received TLS Alert record - server is signaling an issue");
        }

        // Validate content type
        if !(20..=23).contains(&content_type) {
            // Special detection for HTTP responses (server not using TLS)
            // 0x48 = 'H' (start of "HTTP/1.1"), 0x47 = 'G' (start of "GET"), etc.
            if content_type == 0x48 || content_type == 0x47 {
                // Read more bytes to show what was received
                let mut more_data = vec![0u8; 50];
                let _ = stream.read(&mut more_data).await;
                let combined: Vec<u8> = header.iter().chain(more_data.iter()).copied().collect();
                let as_str = String::from_utf8_lossy(&combined);

                error!("❌ Received HTTP response instead of TLS!");
                error!("   Content type 0x{:02x} = ASCII '{}'", content_type, content_type as char);
                error!("   First 50 bytes: {:?}", as_str.trim());
                error!("   This usually means:");
                error!("     1. Connected to port 80 instead of 443");
                error!("     2. Server redirected to HTTP");
                error!("     3. Server doesn't support TLS");

                return Err(Error::TlsHandshake(format!(
                    "Server responded with HTTP instead of TLS (got '{}'). Check port and URL.",
                    as_str.chars().take(30).collect::<String>()
                )));
            }

            error!("❌ Invalid TLS content type: {:#04x}", content_type);
            return Err(Error::TlsHandshake(format!(
                "Invalid TLS content type: {content_type:#04x}"
            )));
        }

        // Validate length (prevent huge allocations)
        if length > 16384 {
            // TLS max record size
            error!("❌ TLS record too large: {} bytes (max 16384)", length);
            return Err(Error::TlsHandshake(format!("TLS record too large: {length} bytes")));
        }

        // Read record content
        trace!("Reading TLS record content ({} bytes)", length);
        let mut content = vec![0u8; length];
        let content_start = std::time::Instant::now();
        stream.read_exact(&mut content).await.map_err(|e| {
            error!("❌ Failed to read TLS record content ({} bytes): {}", length, e);
            Error::Io(e)
        })?;
        debug!("✅ Read {} bytes in {:?}", length, content_start.elapsed());
        trace!("Content preview: {:02x?}", &content[..std::cmp::min(32, content.len())]);

        // Decode Alert if applicable
        if content_type == 0x15 && content.len() >= 2 {
            let alert_level = content[0];
            let alert_description = content[1];
            let level_str = if alert_level == 1 {
                "Warning"
            } else {
                "Fatal"
            };
            let desc_str = match alert_description {
                0 => "close_notify",
                10 => "unexpected_message",
                20 => "bad_record_mac",
                40 => "handshake_failure",
                42 => "bad_certificate",
                43 => "unsupported_certificate",
                44 => "certificate_revoked",
                45 => "certificate_expired",
                46 => "certificate_unknown",
                47 => "illegal_parameter",
                48 => "unknown_ca",
                49 => "access_denied",
                50 => "decode_error",
                51 => "decrypt_error",
                70 => "protocol_version",
                71 => "insufficient_security",
                80 => "internal_error",
                86 => "inappropriate_fallback",
                90 => "user_canceled",
                109 => "missing_extension",
                110 => "unsupported_extension",
                112 => "unrecognized_name",
                113 => "bad_certificate_status_response",
                116 => "certificate_required",
                120 => "no_application_protocol",
                _ => "unknown",
            };
            error!(
                "❌ TLS ALERT: {} ({}) - {} ({})",
                level_str, alert_level, desc_str, alert_description
            );
            error!("   This means the server rejected our ClientHello!");
            error!(
                "   Common causes: missing extensions, unsupported cipher suites, protocol mismatch"
            );
            return Err(Error::TlsHandshake(format!(
                "Server sent {level_str} alert: {desc_str} (code {alert_description})"
            )));
        }

        Ok((content_type, content))
    }

    /// Decrypt a TLS handshake record with handshake traffic keys
    ///
    /// RFC 8446 Section 4.4.1: Transcript hash is computed over PLAINTEXT handshake messages!
    /// After `ServerHello`, all handshake messages (`EncryptedExtensions`, Certificate, etc.) are encrypted.
    /// This method decrypts them so they can be added to the transcript in plaintext form.
    ///
    /// # Arguments
    ///
    /// * `encrypted_record` - The encrypted TLS record content (without TLS record header)
    /// * `keys` - Handshake traffic keys (for decrypting post-handshake messages)
    /// * `sequence_number` - Current sequence number for AEAD nonce generation
    ///
    /// # Returns
    ///
    /// Decrypted plaintext handshake message (without `ContentType` byte)
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails or the negotiated cipher suite is unsupported.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS wire format: values are masked/bounded"
    )]
    pub(super) async fn decrypt_handshake_record(
        &self,
        encrypted_record: &[u8],
        keys: &TlsSecrets,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        info!("🔓 Decrypting handshake record:");
        trace!("   Encrypted length: {} bytes", encrypted_record.len());
        trace!("   Sequence number: {}", sequence_number);

        // Build nonce: server_write_iv XOR sequence_number
        // RFC 8446 Section 5.3: per_record_nonce = IV XOR sequence_number (right-padded to IV length)
        info!("🧮 Computing nonce (RFC 8446 Section 5.3):");
        let mut nonce = keys.server_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();

        // XOR the last 8 bytes of the IV with the sequence number
        // TLS 1.3: nonce = IV[0..4] || (IV[4..12] XOR sequence_number)
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        trace!("   Computed nonce: {:02x?}", nonce);

        // Build AAD (Additional Authenticated Data): TLS record header
        // RFC 8446 Section 5.2: AAD = TLS record header (5 bytes)
        // For encrypted records, ContentType is always 0x17 (ApplicationData) in TLS 1.3
        info!("📋 Building AAD (Additional Authenticated Data):");
        let record_type = 0x17; // ApplicationData (ALL encrypted records use 0x17 in TLS 1.3)
        let version = [0x03, 0x03]; // TLS 1.2 compatibility version
        let length = encrypted_record.len() as u16;
        let aad = [record_type, version[0], version[1], (length >> 8) as u8, (length & 0xFF) as u8];
        trace!("   AAD (TLS record header): {:02x?}", aad);

        // Decrypt via crypto provider - use correct AEAD algorithm based on negotiated cipher suite!
        let decrypt_start = std::time::Instant::now();
        info!(
            "⏳ Calling crypto provider decrypt with cipher suite 0x{:04x}...",
            self.cipher_suite
        );

        let plaintext = match self.cipher_suite {
            0x1301 => {
                // TLS_AES_128_GCM_SHA256 (most common - GitHub, Google, CloudFlare)
                trace!("   → Using AES-128-GCM (negotiated cipher suite)");
                self.crypto
                    .aes128_gcm_decrypt(&keys.server_write_key, &nonce, encrypted_record, &aad)
                    .await
            }
            0x1302 => {
                // TLS_AES_256_GCM_SHA384 (high security)
                trace!("   → Using AES-256-GCM (negotiated cipher suite)");
                self.crypto
                    .aes256_gcm_decrypt(&keys.server_write_key, &nonce, encrypted_record, &aad)
                    .await
            }
            0x1303 => {
                // TLS_CHACHA20_POLY1305_SHA256 (software-only, mobile-optimized)
                trace!("   → Using ChaCha20-Poly1305 (negotiated cipher suite)");
                self.crypto.decrypt(&keys.server_write_key, &nonce, encrypted_record, &aad).await
            }
            _ => {
                error!("❌ Unsupported cipher suite: 0x{:04x}", self.cipher_suite);
                return Err(Error::TlsHandshake(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.cipher_suite
                )));
            }
        }
        .map_err(|e| {
            error!("❌ Handshake record decryption FAILED!");
            error!("   Error: {}", e);
            error!("   Possible causes: wrong key, wrong nonce, wrong AAD, corrupted ciphertext");
            e
        })?;

        info!("✅ Decrypted handshake record successfully in {:?}", decrypt_start.elapsed());
        trace!("   Plaintext length: {} bytes", plaintext.len());

        // RFC 8446 Section 5.2: TLS 1.3 encrypted records have ContentType as last byte
        // Strip the ContentType byte from the end
        if plaintext.is_empty() {
            warn!("⚠️  Empty plaintext after decryption!");
            Ok(plaintext)
        } else {
            let content_type = plaintext[plaintext.len() - 1];
            debug!("ContentType (last byte of plaintext): 0x{:02x}", content_type);
            let message = plaintext[..plaintext.len() - 1].to_vec();
            info!("📤 Returning handshake message: {} bytes (ContentType stripped)", message.len());
            Ok(message)
        }
    }

    /// Parse `ServerHello` message
    ///
    /// Returns: (`server_random`, `server_public_key`, `cipher_suite`)
    pub(crate) fn parse_server_hello(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, u16)> {
        if data.is_empty() || data[0] != 0x02 {
            return Err(Error::TlsHandshake("Invalid ServerHello".to_string()));
        }

        // Skip handshake header (4 bytes)
        let data = &data[4..];

        // Skip version (2 bytes)
        let data = &data[2..];

        // Server random (32 bytes)
        if data.len() < 32 {
            return Err(Error::TlsHandshake("ServerHello too short".to_string()));
        }
        let server_random = data[..32].to_vec();
        let data = &data[32..];

        // Skip legacy session ID
        if data.is_empty() {
            return Err(Error::TlsHandshake("ServerHello truncated".to_string()));
        }
        let session_id_len = data[0] as usize;
        let data = &data[1 + session_id_len..];

        // Parse cipher suite (2 bytes) - CRITICAL for selecting correct AEAD algorithm!
        if data.len() < 3 {
            return Err(Error::TlsHandshake("ServerHello truncated at cipher suite".to_string()));
        }
        let cipher_suite = u16::from_be_bytes([data[0], data[1]]);
        info!("🔐 Server negotiated cipher suite: 0x{:04x}", cipher_suite);

        // Log which TLS 1.3 cipher suite was chosen
        match cipher_suite {
            0x1301 => trace!("   → TLS_AES_128_GCM_SHA256 (most common, hardware accelerated)"),
            0x1302 => trace!("   → TLS_AES_256_GCM_SHA384 (high security, hardware accelerated)"),
            0x1303 => trace!("   → TLS_CHACHA20_POLY1305_SHA256 (software-only, mobile-optimized)"),
            _ => warn!("   → Unknown cipher suite 0x{:04x}", cipher_suite),
        }

        // Skip compression (1 byte)
        let data = &data[3..];

        // Parse extensions
        let server_public = self.extract_key_share(data)?;

        Ok((server_random, server_public, cipher_suite))
    }

    /// Extract public key from `key_share` extension
    #[expect(clippy::unused_self, reason = "method logically belongs on this type")]
    fn extract_key_share(&self, extensions_data: &[u8]) -> Result<Vec<u8>> {
        if extensions_data.len() < 2 {
            return Err(Error::TlsHandshake("Extensions too short".to_string()));
        }

        let _extensions_length =
            u16::from_be_bytes([extensions_data[0], extensions_data[1]]) as usize;
        let mut data = &extensions_data[2..];

        // Parse extensions
        while data.len() >= 4 {
            let ext_type = u16::from_be_bytes([data[0], data[1]]);
            let ext_length = u16::from_be_bytes([data[2], data[3]]) as usize;
            data = &data[4..];

            if data.len() < ext_length {
                return Err(Error::TlsHandshake("Extension truncated".to_string()));
            }

            // Key share extension (0x0033)
            if ext_type == 0x0033 {
                let ext_data = &data[..ext_length];
                // Skip group (2 bytes) and length (2 bytes)
                if ext_data.len() >= 4 {
                    let key_length = u16::from_be_bytes([ext_data[2], ext_data[3]]) as usize;
                    if ext_data.len() >= 4 + key_length {
                        return Ok(ext_data[4..4 + key_length].to_vec());
                    }
                }
            }

            data = &data[ext_length..];
        }

        Err(Error::TlsHandshake("key_share extension not found".to_string()))
    }

    /// Generate 32-byte random (for testing, production should use the `crypto provider`)
    #[expect(clippy::unused_self, reason = "API consistency with other TlsHandshake methods")]
    pub(crate) fn generate_random(&self) -> Vec<u8> {
        let mut random = vec![0u8; 32];

        if getrandom::fill(&mut random).is_err() {
            warn!("⚠️  getrandom failed, falling back to fastrand");
            for byte in &mut random {
                *byte = fastrand::u8(..);
            }
        }

        random
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoCapability;

    fn test_security_socket_path() -> String {
        tempfile::env::temp_dir().join("songbird-test-security.sock").to_string_lossy().into_owned()
    }

    #[test]
    fn test_generate_random() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            test_security_socket_path(),
        )) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(crypto);

        let random1 = handshake.generate_random();
        let random2 = handshake.generate_random();

        assert_eq!(random1.len(), 32);
        assert_eq!(random2.len(), 32);
        assert_ne!(random1, random2, "CSPRNG should produce distinct values");
        assert_ne!(random1, vec![0u8; 32], "should not be all zeros");
    }

    #[test]
    fn test_extract_key_share_too_short() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            test_security_socket_path(),
        )) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(crypto);

        let data = vec![0x00]; // Too short
        let result = handshake.extract_key_share(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_parse_server_hello_invalid() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            test_security_socket_path(),
        )) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(crypto);

        // Empty data
        let result = handshake.parse_server_hello(&[]);
        assert!(result.is_err());

        // Wrong message type
        let result = handshake.parse_server_hello(&[0x01, 0x00, 0x00, 0x00]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_server_hello_truncated() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            test_security_socket_path(),
        )) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(crypto);

        // ServerHello type but truncated
        let data = vec![0x02, 0x00, 0x00, 0x05, 0x03, 0x03]; // Too short for random
        let result = handshake.parse_server_hello(&data);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }
}
