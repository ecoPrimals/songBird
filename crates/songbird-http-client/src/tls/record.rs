//! TLS 1.3 record layer

use crate::crypto::CryptoCapability;
use crate::error::{Error, Result};
use crate::tls::content_type;
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
    pub fn keys(&self) -> &SessionKeys {
        &self.keys
    }

    /// Get write sequence number (for diagnostic logging)
    pub fn write_sequence_number(&self) -> u64 {
        self.write_sequence_number
    }

    /// Write application data
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
        let nonce = self.build_write_nonce();

        // DIAGNOSTIC: Show encryption parameters
        info!("════════════════════════════════════════════════════════════");
        info!("🔐 HTTP REQUEST ENCRYPTION PARAMETERS (DIAGNOSTIC)");
        info!("════════════════════════════════════════════════════════════");
        info!("Plaintext (HTTP request + ContentType): {} bytes", plaintext_with_type.len());
        info!("  HTTP request: {} bytes", data.len());
        info!("  ContentType byte: 0x17 (APPLICATION_DATA)");
        info!("  Total plaintext: {} bytes (before AEAD encryption)", plaintext_with_type.len());
        info!("");
        info!("Sequence number: {} (write_sequence_number)", self.write_sequence_number);
        info!("  ⚠️  CRITICAL: Should be 0 for first HTTP request!");
        info!("");
        info!("Nonce construction (RFC 8446 Section 5.3):");
        info!("  client_write_iv (12 bytes): {}", hex::encode(&self.keys.client_write_iv));
        info!("  Sequence (u64): {}", self.write_sequence_number);
        info!("  Sequence (padded to 12 bytes, big-endian):");
        let mut seq_bytes = [0u8; 12];
        seq_bytes[4..12].copy_from_slice(&self.write_sequence_number.to_be_bytes());
        info!("    {}", hex::encode(seq_bytes));
        info!("  Nonce = IV XOR Sequence:");
        info!("    {}", hex::encode(&nonce));
        info!("");
        info!("AAD (Additional Authenticated Data):");
        info!("  ContentType: 0x{:02x} (APPLICATION_DATA)", aad[0]);
        info!("  TLS version: 0x{:02x} 0x{:02x} (1.2 compatibility)", aad[1], aad[2]);
        info!("  Length: {} bytes (encrypted_length = plaintext + 16-byte tag)", encrypted_length);
        info!("  Length bytes: 0x{:02x} 0x{:02x}", aad[3], aad[4]);
        info!("  Full AAD: {}", hex::encode(aad));
        info!("");
        info!(
            "Cipher suite: 0x{:04x} ({})",
            self.keys.cipher_suite,
            match self.keys.cipher_suite {
                0x1301 => "TLS_AES_128_GCM_SHA256",
                0x1302 => "TLS_AES_256_GCM_SHA384",
                0x1303 => "TLS_CHACHA20_POLY1305_SHA256",
                _ => "UNKNOWN",
            }
        );
        info!(
            "Client write key (application traffic key): {} bytes",
            self.keys.client_write_key.len()
        );
        info!(
            "  Key (first 8 bytes): {}",
            hex::encode(
                &self.keys.client_write_key[..std::cmp::min(8, self.keys.client_write_key.len())]
            )
        );
        info!("════════════════════════════════════════════════════════════");

        // Encrypt data with CLIENT write key (we're writing to server)
        // RFC 8446: Use the negotiated cipher suite for encryption
        debug!("🔐 Encrypting with client_write_key (application traffic key)");
        debug!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);

        let encrypted = match self.keys.cipher_suite {
            0x1301 => {
                // TLS_AES_128_GCM_SHA256
                debug!("   → Using AES-128-GCM for application data");
                self.crypto
                    .aes128_gcm_encrypt(
                        &self.keys.client_write_key,
                        &nonce,
                        &plaintext_with_type,
                        &aad,
                    )
                    .await
            }
            0x1302 => {
                // TLS_AES_256_GCM_SHA384
                debug!("   → Using AES-256-GCM for application data");
                self.crypto
                    .aes256_gcm_encrypt(
                        &self.keys.client_write_key,
                        &nonce,
                        &plaintext_with_type,
                        &aad,
                    )
                    .await
            }
            0x1303 => {
                // TLS_CHACHA20_POLY1305_SHA256
                debug!("   → Using ChaCha20-Poly1305 for application data");
                self.crypto
                    .encrypt(&self.keys.client_write_key, &nonce, &plaintext_with_type, &aad)
                    .await
            }
            _ => {
                error!(
                    "❌ Unsupported cipher suite for encryption: 0x{:04x}",
                    self.keys.cipher_suite
                );
                return Err(Error::TlsRecord(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.keys.cipher_suite
                )));
            }
        }
        .map_err(|e| {
            error!("❌ Application data encryption failed: {}", e);
            error!("   Plaintext length: {} bytes", plaintext_with_type.len());
            error!("   Sequence number: {}", self.write_sequence_number);
            error!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
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
            // Alert
            info!("📢 Received TLS ALERT record");
            // Read alert to see what it is
            if length >= 2 {
                let mut alert_data = vec![0u8; length];
                stream.read_exact(&mut alert_data).await?;
                let alert_level = alert_data[0];
                let alert_desc = alert_data[1];
                let level_str = if alert_level == 1 {
                    "Warning"
                } else {
                    "Fatal"
                };
                let desc_str = match alert_desc {
                    0 => "close_notify",
                    10 => "unexpected_message",
                    20 => "bad_record_mac",
                    40 => "handshake_failure",
                    51 => "decrypt_error",
                    _ => "unknown",
                };

                // close_notify (0) is a normal connection close - not an error!
                if alert_desc == 0 {
                    info!("✅ close_notify: Server closed connection gracefully");
                    // Return empty vec to signal EOF without error
                    return Ok(Vec::new());
                }

                // All other alerts are errors
                error!(
                    "❌ TLS Alert: {} {} (level={}, desc={})",
                    level_str, desc_str, alert_level, alert_desc
                );
                return Err(Error::TlsRecord(format!(
                    "Server sent {} alert: {} (code {})",
                    level_str, desc_str, alert_desc
                )));
            }
        }

        if content_type != content_type::APPLICATION_DATA {
            error!("❌ Expected APPLICATION_DATA (0x17), got 0x{:02x}", content_type);
            return Err(Error::TlsRecord(format!(
                "Expected APPLICATION_DATA (0x17), got {:#04x}",
                content_type
            )));
        }

        // Validate length
        if length < 16 {
            error!("❌ TLS record too short: {} bytes (need at least 16 for AEAD tag)", length);
            error!("   This likely indicates a protocol error or incomplete read");
            return Err(Error::TlsRecord(format!(
                "TLS record too short: {} bytes (need at least 16 for AEAD tag)",
                length
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
                warn!("⚠️  SUSPICIOUS: Encrypted data length ({} bytes) matches expected size for our last request!", encrypted.len());
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

        // AAD = TLS record header (5 bytes)
        let aad = &header;
        debug!("AAD (TLS record header): {:02x?}", aad);

        // Build nonce: server_write_iv XOR read_sequence_number (RFC 8446 Section 5.3)
        let nonce = self.build_read_nonce();
        debug!(
            "Nonce (server_write_iv XOR seq {}): {:02x?}",
            self.read_sequence_number,
            &nonce[..std::cmp::min(12, nonce.len())]
        );

        // Decrypt data with SERVER write key (we're reading from server)
        // RFC 8446: Use the negotiated cipher suite for decryption
        debug!("🔓 Decrypting with server_write_key (application traffic key)");
        debug!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);

        let decrypted = match self.keys.cipher_suite {
            0x1301 => {
                // TLS_AES_128_GCM_SHA256
                debug!("   → Using AES-128-GCM for application data");
                self.crypto
                    .aes128_gcm_decrypt(&self.keys.server_write_key, &nonce, &encrypted, aad)
                    .await
            }
            0x1302 => {
                // TLS_AES_256_GCM_SHA384
                debug!("   → Using AES-256-GCM for application data");
                self.crypto
                    .aes256_gcm_decrypt(&self.keys.server_write_key, &nonce, &encrypted, aad)
                    .await
            }
            0x1303 => {
                // TLS_CHACHA20_POLY1305_SHA256
                debug!("   → Using ChaCha20-Poly1305 for application data");
                self.crypto.decrypt(&self.keys.server_write_key, &nonce, &encrypted, aad).await
            }
            _ => {
                error!(
                    "❌ Unsupported cipher suite for decryption: 0x{:04x}",
                    self.keys.cipher_suite
                );
                return Err(Error::TlsRecord(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.keys.cipher_suite
                )));
            }
        }
        .map_err(|e| {
            error!("❌ Application data decryption failed: {}", e);
            error!("   Encrypted length: {} bytes", encrypted.len());
            error!("   Sequence number: {}", self.read_sequence_number);
            error!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
            e
        })?;

        info!(
            "✅ Decrypted {} bytes → {} bytes (AEAD authentication succeeded)",
            encrypted.len(),
            decrypted.len()
        );

        // DIAGNOSTIC: Show exactly what we decrypted
        info!("════════════════════════════════════════════════════════════");
        info!("🔍 DECRYPTED CONTENT ANALYSIS (DIAGNOSTIC)");
        info!("════════════════════════════════════════════════════════════");
        info!("Ciphertext length: {} bytes (includes 16-byte AEAD tag)", encrypted.len());
        info!("Plaintext length: {} bytes (after AEAD decryption)", decrypted.len());

        if !decrypted.is_empty() {
            // Show first and last bytes
            info!(
                "First 16 bytes (hex): {}",
                hex::encode(&decrypted[..std::cmp::min(16, decrypted.len())])
            );
            if decrypted.len() > 16 {
                info!(
                    "Last 16 bytes (hex): {}",
                    hex::encode(&decrypted[decrypted.len().saturating_sub(16)..])
                );
            }

            // Try to interpret as UTF-8
            let utf8_preview =
                String::from_utf8_lossy(&decrypted[..std::cmp::min(200, decrypted.len())]);
            info!("UTF-8 preview (first 200 bytes):");
            info!("  {}", utf8_preview);

            // Check if this might be a TLS alert
            let last_byte = *decrypted.last().unwrap_or(&0xFF);
            if last_byte == 0x15 {
                warn!("🚨 ALERT DETECTED! Last byte is 0x15 (ALERT ContentType)");
                if decrypted.len() >= 2 {
                    let alert_level = decrypted[0];
                    let alert_desc = decrypted[1];
                    warn!(
                        "   Alert level: 0x{:02x} ({})",
                        alert_level,
                        if alert_level == 1 {
                            "Warning"
                        } else {
                            "Fatal"
                        }
                    );
                    warn!(
                        "   Alert description: 0x{:02x} ({})",
                        alert_desc,
                        match alert_desc {
                            0x28 => "handshake_failure",
                            0x33 => "decrypt_error",
                            0x46 => "certificate_required",
                            0x50 => "protocol_version",
                            _ => "unknown",
                        }
                    );
                }
            } else if last_byte == 0x17 {
                debug!("✅ Last byte is 0x17 (APPLICATION_DATA ContentType) - as expected");
            } else if last_byte == 0x16 {
                debug!("✅ Last byte is 0x16 (HANDSHAKE ContentType)");
            } else {
                warn!("⚠️  Last byte is 0x{:02x} (unexpected ContentType!)", last_byte);
            }
        }
        info!("════════════════════════════════════════════════════════════");

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

        info!("════════════════════════════════════════════════════════════");
        info!("🎯 FINAL PLAINTEXT AFTER CONTENTTYPE STRIPPING");
        info!("════════════════════════════════════════════════════════════");
        info!(
            "ContentType stripped: 0x{:02x} ({})",
            content_type_byte,
            match content_type_byte {
                0x15 => "ALERT",
                0x16 => "HANDSHAKE",
                0x17 => "APPLICATION_DATA",
                _ => "UNKNOWN",
            }
        );

        // Check for handshake messages (like NewSessionTicket) - we need to skip them
        // and read the NEXT record to get actual HTTP data
        if content_type_byte == 0x16 {
            info!("📝 Received HANDSHAKE message in APPLICATION_DATA (likely NewSessionTicket) - reading next record...");
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

        // Check if server sent an alert
        if content_type_byte == 0x15 {
            error!("════════════════════════════════════════════════════════════");
            error!("🚨 SERVER SENT TLS ALERT!");
            error!("════════════════════════════════════════════════════════════");
            if plaintext.len() >= 2 {
                let alert_level = plaintext[0];
                let alert_desc = plaintext[1];
                error!(
                    "Alert level: 0x{:02x} ({})",
                    alert_level,
                    if alert_level == 0x01 {
                        "Warning"
                    } else {
                        "Fatal"
                    }
                );
                error!(
                    "Alert description: 0x{:02x} ({})",
                    alert_desc,
                    match alert_desc {
                        0x00 => "close_notify",
                        0x0A => "unexpected_message",
                        0x14 => "bad_record_mac",
                        0x15 => "decryption_failed",
                        0x16 => "record_overflow",
                        0x28 => "handshake_failure",
                        0x29 => "no_certificate",
                        0x2A => "bad_certificate",
                        0x2B => "unsupported_certificate",
                        0x2C => "certificate_revoked",
                        0x2D => "certificate_expired",
                        0x2E => "certificate_unknown",
                        0x2F => "illegal_parameter",
                        0x30 => "unknown_ca",
                        0x31 => "access_denied",
                        0x32 => "decode_error",
                        0x33 => "decrypt_error",
                        0x3C => "unrecognized_name",
                        0x46 => "certificate_required",
                        0x50 => "protocol_version",
                        0x56 => "insufficient_security",
                        0x5A => "internal_error",
                        0x5F => "user_canceled",
                        0x6D => "no_renegotiation",
                        0x6E => "missing_extension",
                        _ => "unknown",
                    }
                );
                error!("════════════════════════════════════════════════════════════");

                // Return a descriptive error
                return Err(Error::TlsAlert(format!(
                    "Server sent {} alert: {} (0x{:02x})",
                    if alert_level == 0x01 {
                        "Warning"
                    } else {
                        "Fatal"
                    },
                    match alert_desc {
                        0x00 => "close_notify",
                        0x28 => "handshake_failure",
                        0x33 => "decrypt_error",
                        0x50 => "protocol_version",
                        _ => "unknown",
                    },
                    alert_desc
                )));
            } else {
                error!("Alert is too short ({} bytes) - malformed!", plaintext.len());
                error!("════════════════════════════════════════════════════════════");
                return Err(Error::TlsAlert("Server sent malformed TLS alert".to_string()));
            }
        }

        info!("Final plaintext length: {} bytes (ready for HTTP parser)", plaintext.len());

        if !plaintext.is_empty() {
            info!(
                "First 100 bytes (hex): {}",
                hex::encode(&plaintext[..std::cmp::min(100, plaintext.len())])
            );
            let utf8_preview =
                String::from_utf8_lossy(&plaintext[..std::cmp::min(300, plaintext.len())]);
            info!("UTF-8 preview (first 300 bytes):");
            info!("  {}", utf8_preview);

            // Check if it starts with HTTP
            if plaintext.len() >= 8 {
                let start = String::from_utf8_lossy(&plaintext[..8]);
                if start.starts_with("HTTP/") {
                    info!("✅ Plaintext starts with 'HTTP/' - looks like valid HTTP response!");
                } else {
                    warn!("⚠️  Plaintext does NOT start with 'HTTP/' - may not be HTTP response!");
                    warn!("   Instead starts with: {:?}", start);
                }
            }
        } else {
            warn!("⚠️  Final plaintext is EMPTY after ContentType stripping!");
        }
        info!("════════════════════════════════════════════════════════════");

        self.read_sequence_number += 1;
        debug!("  → Incremented read sequence number to {}", self.read_sequence_number);

        Ok(plaintext)
    }

    /// Build nonce for writing (encryption)
    /// RFC 8446 Section 5.3: nonce = IV XOR sequence_number (right-aligned)
    fn build_write_nonce(&self) -> Vec<u8> {
        let mut nonce = self.keys.client_write_iv.clone();
        let seq_bytes = self.write_sequence_number.to_be_bytes();

        // XOR sequence number with IV (right-aligned)
        // For 12-byte IV and 8-byte sequence: XOR last 8 bytes
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        nonce
    }

    /// Build nonce for reading (decryption)
    /// RFC 8446 Section 5.3: nonce = IV XOR sequence_number (right-aligned)
    fn build_read_nonce(&self) -> Vec<u8> {
        let mut nonce = self.keys.server_write_iv.clone();
        let seq_bytes = self.read_sequence_number.to_be_bytes();

        // XOR sequence number with IV (right-aligned)
        // For 12-byte IV and 8-byte sequence: XOR last 8 bytes
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }

        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::BearDogProvider;

    #[test]
    fn test_build_write_nonce() {
        let crypto: Arc<dyn CryptoCapability> = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![0; 12],
            cipher_suite: 0x1303, // ChaCha20-Poly1305 for test
            initial_read_sequence: 0,
        };

        let mut layer = TlsRecordLayer::new(crypto, keys);
        let nonce = layer.build_write_nonce();
        assert_eq!(nonce.len(), 12);

        // Sequence number should affect nonce
        layer.write_sequence_number = 1;
        let nonce2 = layer.build_write_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_build_read_nonce() {
        let crypto: Arc<dyn CryptoCapability> = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![0; 12],
            server_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            cipher_suite: 0x1303, // ChaCha20-Poly1305 for test
            initial_read_sequence: 0,
        };

        let mut layer = TlsRecordLayer::new(crypto, keys);
        let nonce = layer.build_read_nonce();
        assert_eq!(nonce.len(), 12);

        // Sequence number should affect nonce
        layer.read_sequence_number = 1;
        let nonce2 = layer.build_read_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_separate_sequence_numbers() {
        let crypto: Arc<dyn CryptoCapability> = Arc::new(BearDogProvider::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
            cipher_suite: 0x1303, // ChaCha20-Poly1305 for test
            initial_read_sequence: 0,
        };

        let mut layer = TlsRecordLayer::new(crypto, keys);

        // Write and read should use different nonces due to different IVs
        let write_nonce = layer.build_write_nonce();
        let read_nonce = layer.build_read_nonce();
        assert_ne!(write_nonce, read_nonce, "Write and read nonces should differ");

        // Increment sequence numbers independently
        layer.write_sequence_number = 5;
        layer.read_sequence_number = 3;

        let write_nonce2 = layer.build_write_nonce();
        let read_nonce2 = layer.build_read_nonce();

        // Nonces should change
        assert_ne!(write_nonce, write_nonce2);
        assert_ne!(read_nonce, read_nonce2);
        assert_ne!(write_nonce2, read_nonce2);
    }
}
