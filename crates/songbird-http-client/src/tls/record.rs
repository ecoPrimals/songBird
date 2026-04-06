// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 record layer

use crate::crypto::CryptoCapability;
use crate::error::{Error, Result};
use crate::tls::alert::TlsAlert;
use crate::tls::content_type;
use crate::tls::record_crypto::{build_nonce, cipher_decrypt, cipher_encrypt, cipher_suite_name};
use crate::tls::session::SessionKeys;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, error, info, trace, warn};

/// TLS record layer
pub struct TlsRecordLayer {
    crypto: Arc<dyn CryptoCapability>,
    keys: SessionKeys,
    write_sequence_number: u64,
    read_sequence_number: u64,
    last_written_size: Option<usize>, // Track last write for debugging
}

impl TlsRecordLayer {
    /// Create a new TLS record layer
    pub fn new(crypto: Arc<dyn CryptoCapability>, keys: SessionKeys) -> Self {
        let initial_read_seq = keys.initial_read_sequence;
        info!(
            "📊 TlsRecordLayer initialized with read_sequence_number = {} (from handshake)",
            initial_read_seq
        );
        Self {
            crypto,
            keys,
            write_sequence_number: 0,
            read_sequence_number: initial_read_seq, // Start where handshake left off
            last_written_size: None,
        }
    }

    /// Get session keys (for diagnostic logging)
    #[must_use]
    pub const fn keys(&self) -> &SessionKeys {
        &self.keys
    }

    /// Get write sequence number (for diagnostic logging)
    #[must_use]
    pub const fn write_sequence_number(&self) -> u64 {
        self.write_sequence_number
    }

    /// Write application data
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or writing to the stream fails.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS wire format: values are masked/bounded"
    )]
    pub async fn write_application_data(
        &mut self,
        stream: &mut TcpStream,
        data: &[u8],
    ) -> Result<()> {
        info!("📤 Writing {} bytes of HTTP application data", data.len());
        debug!("  Write sequence number: {}", self.write_sequence_number);
        trace!(
            "HTTP request preview: {}",
            String::from_utf8_lossy(&data[..std::cmp::min(200, data.len())])
        );

        // RFC 8446 Section 5.2: TLS 1.3 encrypted records include ContentType at END of plaintext
        // Add ContentType byte (0x17 = APPLICATION_DATA) to end of data before encryption
        let mut plaintext_with_type = data.to_vec();
        plaintext_with_type.push(content_type::APPLICATION_DATA);
        debug!(
            "Added ContentType byte (0x17) at end: {} bytes total plaintext",
            plaintext_with_type.len()
        );

        // Calculate encrypted length (plaintext + ContentType + 16-byte AEAD tag)
        let encrypted_length = plaintext_with_type.len() + 16;

        // Build AAD (TLS record header)
        let aad = [
            content_type::APPLICATION_DATA,
            0x03,
            0x03, // TLS 1.2 (compatibility)
            (encrypted_length >> 8) as u8,
            (encrypted_length & 0xFF) as u8,
        ];

        debug!("AAD (TLS record header): {:02x?}", aad);

        // Build nonce: client_write_iv XOR write_sequence_number (RFC 8446 Section 5.3)
        let nonce = build_nonce(&self.keys.client_write_iv, self.write_sequence_number);

        trace!(
            "Encryption: seq={}, cipher=0x{:04x} ({}), plaintext={} bytes",
            self.write_sequence_number,
            self.keys.cipher_suite,
            cipher_suite_name(self.keys.cipher_suite),
            plaintext_with_type.len()
        );
        trace!("  Nonce: {}", hex::encode(&nonce));
        trace!("  AAD: {}", hex::encode(aad));

        debug!("Encrypting with client_write_key, cipher 0x{:04x}", self.keys.cipher_suite);

        let encrypted = cipher_encrypt(
            &self.crypto,
            self.keys.cipher_suite,
            &self.keys.client_write_key,
            &nonce,
            &plaintext_with_type,
            &aad,
        )
        .await
        .map_err(|e| {
            error!("Application data encryption failed: {e}");
            error!(
                "  plaintext={} bytes, seq={}, cipher=0x{:04x}",
                plaintext_with_type.len(),
                self.write_sequence_number,
                self.keys.cipher_suite
            );
            e
        })?;

        info!("✅ Encrypted {} bytes → {} bytes", plaintext_with_type.len(), encrypted.len());

        // Build complete TLS record
        let mut record = Vec::new();
        record.extend_from_slice(&aad); // Header (5 bytes)
        record.extend_from_slice(&encrypted); // Ciphertext + tag

        debug!(
            "Writing TLS record: {} bytes total (5-byte header + {} bytes encrypted)",
            record.len(),
            encrypted.len()
        );

        // Write to stream
        stream.write_all(&record).await.map_err(|e| {
            error!("❌ Failed to write TLS record: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await?;

        self.write_sequence_number += 1;
        debug!("  → Incremented write sequence number to {}", self.write_sequence_number);

        // Track last written size for debugging
        self.last_written_size = Some(data.len());
        debug!(
            "  → Stored last written size: {} bytes (for request/response validation)",
            data.len()
        );

        Ok(())
    }

    /// Read application data
    ///
    /// # Errors
    ///
    /// Returns an error if the record cannot be read or decrypted.
    #[expect(
        clippy::too_many_lines,
        reason = "TLS protocol requires sequential state machine steps"
    )]
    pub async fn read_application_data(&mut self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        info!("📥 Reading HTTP application data (APPLICATION DATA phase)");
        debug!("  Read sequence number: {}", self.read_sequence_number);

        // Validate TCP stream state
        if let Ok(peer) = stream.peer_addr() {
            debug!("TCP stream peer address: {}", peer);
        } else {
            warn!("⚠️  Unable to get peer address (stream may be closed)");
        }

        // Read record header (5 bytes)
        let mut header = [0u8; 5];
        match stream.read_exact(&mut header).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Server closed connection (normal after sending complete response)
                info!("✅ Server closed connection (EOF) - response complete");
                return Ok(Vec::new()); // Signal EOF without error
            }
            Err(e) => {
                error!("❌ Failed to read TLS record header: {}", e);
                return Err(Error::Io(e));
            }
        }

        let content_type = header[0];
        let tls_version = u16::from_be_bytes([header[1], header[2]]);
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        info!("📋 TLS record header:");
        info!(
            "  Content type: 0x{:02x} ({})",
            content_type,
            if content_type == 0x17 {
                "APPLICATION_DATA"
            } else if content_type == 0x15 {
                "ALERT"
            } else if content_type == 0x16 {
                "HANDSHAKE"
            } else {
                "UNKNOWN"
            }
        );
        info!("  TLS version: 0x{:04x}", tls_version);
        info!("  Encrypted length: {} bytes", length);

        // Check for TLS alerts (close_notify, etc.)
        if content_type == 0x15 {
            info!("Received TLS ALERT record");
            if length >= 2 {
                let mut alert_data = vec![0u8; length];
                stream.read_exact(&mut alert_data).await?;

                if let Ok(alert) = TlsAlert::parse(&alert_data) {
                    if alert.description == crate::tls::alert::AlertDescription::CloseNotify {
                        info!("close_notify: Server closed connection gracefully");
                        return Ok(Vec::new());
                    }
                    error!("TLS Alert: {}", alert.to_detailed_string());
                    return Err(Error::TlsAlert(alert.to_string()));
                }
                error!("Unrecognised TLS alert: {:02x?}", &alert_data[..2]);
                return Err(Error::TlsRecord("Unrecognised TLS alert".to_string()));
            }
        }

        if content_type != content_type::APPLICATION_DATA {
            error!("❌ Expected APPLICATION_DATA (0x17), got 0x{:02x}", content_type);
            return Err(Error::TlsRecord(format!(
                "Expected APPLICATION_DATA (0x17), got {content_type:#04x}"
            )));
        }

        // Validate length
        if length < 16 {
            error!("❌ TLS record too short: {} bytes (need at least 16 for AEAD tag)", length);
            error!("   This likely indicates a protocol error or incomplete read");
            return Err(Error::TlsRecord(format!(
                "TLS record too short: {length} bytes (need at least 16 for AEAD tag)"
            )));
        }

        // Read encrypted data (includes ContentType byte + 16-byte AEAD tag)
        let mut encrypted = vec![0u8; length];
        stream.read_exact(&mut encrypted).await.map_err(|e| {
            error!("❌ Failed to read encrypted data ({} bytes): {}", length, e);
            Error::Io(e)
        })?;

        debug!("✅ Read {} bytes of encrypted application data", encrypted.len());
        trace!(
            "Encrypted data (first 32 bytes): {:02x?}",
            &encrypted[..std::cmp::min(32, encrypted.len())]
        );

        // VALIDATION: Check if we're suspiciously reading data similar to what we just wrote
        if let Some(last_write_size) = self.last_written_size {
            // Compare encrypted length to last written plaintext
            // Encrypted = plaintext + ContentType(1) + AEAD tag(16)
            let expected_encrypted_size = last_write_size + 1 + 16;

            if encrypted.len() == expected_encrypted_size {
                warn!(
                    "⚠️  SUSPICIOUS: Encrypted data length ({} bytes) matches expected size for our last request!",
                    encrypted.len()
                );
                warn!("   Last written plaintext: {} bytes", last_write_size);
                warn!(
                    "   Expected encrypted size: {} bytes (plaintext + 1 + 16)",
                    expected_encrypted_size
                );
                warn!("   Actual encrypted size: {} bytes", encrypted.len());
                warn!("   → Are we reading our own request instead of server's response?");
            } else {
                debug!(
                    "✅ Size validation: {} bytes received vs {} bytes sent (different - good!)",
                    encrypted.len(),
                    expected_encrypted_size
                );
            }
        }

        let aad = &header;
        let nonce = build_nonce(&self.keys.server_write_iv, self.read_sequence_number);
        debug!(
            "Decrypting: seq={}, cipher=0x{:04x}",
            self.read_sequence_number, self.keys.cipher_suite
        );

        let decrypted = cipher_decrypt(
            &self.crypto,
            self.keys.cipher_suite,
            &self.keys.server_write_key,
            &nonce,
            &encrypted,
            aad,
        )
        .await
        .map_err(|e| {
            error!("Application data decryption failed: {e}");
            error!(
                "  encrypted={} bytes, seq={}, cipher=0x{:04x}",
                encrypted.len(),
                self.read_sequence_number,
                self.keys.cipher_suite
            );
            e
        })?;

        info!(
            "✅ Decrypted {} bytes → {} bytes (AEAD authentication succeeded)",
            encrypted.len(),
            decrypted.len()
        );

        trace!("Decrypted {} bytes → {} bytes (plaintext)", encrypted.len(), decrypted.len());
        if !decrypted.is_empty() {
            trace!(
                "First 16 bytes: {}",
                hex::encode(&decrypted[..std::cmp::min(16, decrypted.len())])
            );
        }

        // RFC 8446 Section 5.4: TLSInnerPlaintext structure is:
        // [content] [ContentType byte] [padding zeros...]
        // We need to: 1) strip trailing padding zeros, 2) strip ContentType byte
        if decrypted.is_empty() {
            warn!("⚠️  Empty plaintext after decryption (no ContentType to strip)");
            self.read_sequence_number += 1;
            return Ok(decrypted);
        }

        let mut plaintext = decrypted;

        // Step 1: Strip any trailing zero bytes (padding)
        let original_len = plaintext.len();
        while plaintext.len() > 1 && plaintext[plaintext.len() - 1] == 0x00 {
            plaintext.truncate(plaintext.len() - 1);
        }
        if plaintext.len() < original_len {
            debug!(
                "🔪 Stripped {} bytes of padding (trailing zeros)",
                original_len - plaintext.len()
            );
        }

        // Step 2: Strip ContentType byte (should be 0x16 for handshake or 0x17 for application data)
        let content_type_byte = plaintext[plaintext.len() - 1];
        info!("📋 ContentType byte at end of plaintext: 0x{:02x}", content_type_byte);
        plaintext.truncate(plaintext.len() - 1);

        trace!("Inner ContentType: 0x{content_type_byte:02x}");

        // Check for handshake messages (like NewSessionTicket) - we need to skip them
        // and read the NEXT record to get actual HTTP data
        if content_type_byte == 0x16 {
            info!(
                "📝 Received HANDSHAKE message in APPLICATION_DATA (likely NewSessionTicket) - reading next record..."
            );
            if !plaintext.is_empty() {
                let hs_type = plaintext[0];
                info!(
                    "   Handshake type: 0x{:02x} ({})",
                    hs_type,
                    match hs_type {
                        0x04 => "NewSessionTicket",
                        0x08 => "EncryptedExtensions",
                        0x0b => "Certificate",
                        0x0f => "CertificateVerify",
                        0x14 => "Finished",
                        _ => "Unknown",
                    }
                );
            }
            // Increment sequence number and RECURSE to read the next record
            self.read_sequence_number += 1;
            // Use Box::pin for async recursion
            return Box::pin(self.read_application_data(stream)).await;
        }

        // Check if server sent an alert inside the inner plaintext
        if content_type_byte == 0x15 {
            match TlsAlert::parse(&plaintext) {
                Ok(alert) => {
                    if alert.description == crate::tls::alert::AlertDescription::CloseNotify {
                        debug!("close_notify (inner): Server closed connection gracefully");
                        return Ok(Vec::new());
                    }
                    error!("TLS alert (inner): {}", alert.to_detailed_string());
                    return Err(Error::TlsAlert(alert.to_string()));
                }
                Err(e) => {
                    error!("Malformed TLS alert ({} bytes): {e}", plaintext.len());
                    return Err(Error::TlsAlert("Server sent malformed TLS alert".to_string()));
                }
            }
        }

        if plaintext.is_empty() {
            warn!("Empty plaintext after ContentType stripping");
        } else {
            trace!("Plaintext ready: {} bytes", plaintext.len());
        }

        self.read_sequence_number += 1;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::crypto::SecurityCryptoProvider;
    use crate::tls::record_crypto::build_nonce;

    #[test]
    fn test_build_write_nonce() {
        let iv = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let nonce = build_nonce(&iv, 0);
        assert_eq!(nonce.len(), 12);

        let nonce2 = build_nonce(&iv, 1);
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_build_read_nonce() {
        let iv = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let nonce = build_nonce(&iv, 0);
        assert_eq!(nonce.len(), 12);

        let nonce2 = build_nonce(&iv, 1);
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_separate_nonces_for_different_ivs() {
        let write_iv = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let read_iv = vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

        let write_nonce = build_nonce(&write_iv, 0);
        let read_nonce = build_nonce(&read_iv, 0);
        assert_ne!(write_nonce, read_nonce, "Write and read nonces should differ");

        let write_nonce2 = build_nonce(&write_iv, 5);
        let read_nonce2 = build_nonce(&read_iv, 3);
        assert_ne!(write_nonce, write_nonce2);
        assert_ne!(read_nonce, read_nonce2);
    }

    #[test]
    fn test_tls_record_layer_accessors_and_initial_read_sequence() {
        let crypto: Arc<dyn CryptoCapability> =
            Arc::new(SecurityCryptoProvider::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![0; 12],
            server_write_iv: vec![0; 12],
            cipher_suite: 0x1301,
            initial_read_sequence: 7,
        };

        let layer = TlsRecordLayer::new(crypto, keys);
        assert_eq!(layer.write_sequence_number(), 0);
        assert_eq!(layer.keys().cipher_suite, 0x1301);
        assert_eq!(layer.read_sequence_number, 7);
    }
}
