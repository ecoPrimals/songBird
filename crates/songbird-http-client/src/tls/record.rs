//! TLS 1.3 record layer

use crate::beardog_client::BearDogClient;
use crate::error::{Error, Result};
use crate::tls::content_type;
use crate::tls::session::SessionKeys;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{trace, debug, warn, error, info};

/// TLS record layer
pub struct TlsRecordLayer {
    beardog: Arc<BearDogClient>,
    keys: SessionKeys,
    write_sequence_number: u64,
    read_sequence_number: u64,
    last_written_size: Option<usize>,  // Track last write for debugging
}

impl TlsRecordLayer {
    /// Create a new TLS record layer
    pub fn new(beardog: Arc<BearDogClient>, keys: SessionKeys) -> Self {
        Self {
            beardog,
            keys,
            write_sequence_number: 0,
            read_sequence_number: 0,
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
        trace!("HTTP request preview: {}", String::from_utf8_lossy(&data[..std::cmp::min(200, data.len())]));

        // RFC 8446 Section 5.2: TLS 1.3 encrypted records include ContentType at END of plaintext
        // Add ContentType byte (0x17 = APPLICATION_DATA) to end of data before encryption
        let mut plaintext_with_type = data.to_vec();
        plaintext_with_type.push(content_type::APPLICATION_DATA);
        debug!("Added ContentType byte (0x17) at end: {} bytes total plaintext", plaintext_with_type.len());

        // Calculate encrypted length (plaintext + ContentType + 16-byte AEAD tag)
        let encrypted_length = plaintext_with_type.len() + 16;
        
        // Build AAD (TLS record header)
        let aad = [
            content_type::APPLICATION_DATA,
            0x03, 0x03,  // TLS 1.2 (compatibility)
            (encrypted_length >> 8) as u8,
            (encrypted_length & 0xFF) as u8,
        ];
        
        debug!("AAD (TLS record header): {:02x?}", aad);

        // Build nonce: client_write_iv XOR write_sequence_number (RFC 8446 Section 5.3)
        let nonce = self.build_write_nonce();
        debug!("Nonce (client_write_iv XOR seq {}): {:02x?}", self.write_sequence_number,
               &nonce[..std::cmp::min(12, nonce.len())]);
        
        // Encrypt data with CLIENT write key (we're writing to server)
        // RFC 8446: Use the negotiated cipher suite for encryption
        debug!("🔐 Encrypting with client_write_key (application traffic key)");
        debug!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
        
        let encrypted = match self.keys.cipher_suite {
            0x1301 => {  // TLS_AES_128_GCM_SHA256
                debug!("   → Using AES-128-GCM for application data");
                self.beardog.encrypt_aes_128_gcm(
                    &self.keys.client_write_key,
                    &nonce,
                    &plaintext_with_type,
                    &aad,
                ).await
            }
            0x1302 => {  // TLS_AES_256_GCM_SHA384
                debug!("   → Using AES-256-GCM for application data");
                self.beardog.encrypt_aes_256_gcm(
                    &self.keys.client_write_key,
                    &nonce,
                    &plaintext_with_type,
                    &aad,
                ).await
            }
            0x1303 => {  // TLS_CHACHA20_POLY1305_SHA256
                debug!("   → Using ChaCha20-Poly1305 for application data");
                self.beardog.encrypt(
                    &self.keys.client_write_key,
                    &nonce,
                    &plaintext_with_type,
                    &aad,
                ).await
            }
            _ => {
                error!("❌ Unsupported cipher suite for encryption: 0x{:04x}", self.keys.cipher_suite);
                return Err(Error::TlsRecord(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.keys.cipher_suite
                )));
            }
        }.map_err(|e| {
            error!("❌ Application data encryption failed: {}", e);
            error!("   Plaintext length: {} bytes", plaintext_with_type.len());
            error!("   Sequence number: {}", self.write_sequence_number);
            error!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
            e
        })?;

        info!("✅ Encrypted {} bytes → {} bytes", plaintext_with_type.len(), encrypted.len());

        // Build complete TLS record
        let mut record = Vec::new();
        record.extend_from_slice(&aad);  // Header (5 bytes)
        record.extend_from_slice(&encrypted);  // Ciphertext + tag

        debug!("Writing TLS record: {} bytes total (5-byte header + {} bytes encrypted)", 
               record.len(), encrypted.len());

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
        debug!("  → Stored last written size: {} bytes (for request/response validation)", data.len());

        Ok(())
    }

    /// Read application data
    pub async fn read_application_data(
        &mut self,
        stream: &mut TcpStream,
    ) -> Result<Vec<u8>> {
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
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Server closed connection (normal after sending complete response)
                info!("✅ Server closed connection (EOF) - response complete");
                return Ok(Vec::new());  // Signal EOF without error
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
        info!("  Content type: 0x{:02x} ({})", content_type, 
              if content_type == 0x17 { "APPLICATION_DATA" } 
              else if content_type == 0x15 { "ALERT" }
              else if content_type == 0x16 { "HANDSHAKE" }
              else { "UNKNOWN" });
        info!("  TLS version: 0x{:04x}", tls_version);
        info!("  Encrypted length: {} bytes", length);

        // Check for TLS alerts (close_notify, etc.)
        if content_type == 0x15 {  // Alert
            info!("📢 Received TLS ALERT record");
            // Read alert to see what it is
            if length >= 2 {
                let mut alert_data = vec![0u8; length];
                stream.read_exact(&mut alert_data).await?;
                let alert_level = alert_data[0];
                let alert_desc = alert_data[1];
                let level_str = if alert_level == 1 { "Warning" } else { "Fatal" };
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
                error!("❌ TLS Alert: {} {} (level={}, desc={})", level_str, desc_str, alert_level, alert_desc);
                return Err(Error::TlsRecord(format!("Server sent {} alert: {} (code {})", level_str, desc_str, alert_desc)));
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
        trace!("Encrypted data (first 32 bytes): {:02x?}", &encrypted[..std::cmp::min(32, encrypted.len())]);

        // VALIDATION: Check if we're suspiciously reading data similar to what we just wrote
        if let Some(last_write_size) = self.last_written_size {
            // Compare encrypted length to last written plaintext
            // Encrypted = plaintext + ContentType(1) + AEAD tag(16)
            let expected_encrypted_size = last_write_size + 1 + 16;
            
            if encrypted.len() == expected_encrypted_size {
                warn!("⚠️  SUSPICIOUS: Encrypted data length ({} bytes) matches expected size for our last request!", encrypted.len());
                warn!("   Last written plaintext: {} bytes", last_write_size);
                warn!("   Expected encrypted size: {} bytes (plaintext + 1 + 16)", expected_encrypted_size);
                warn!("   Actual encrypted size: {} bytes", encrypted.len());
                warn!("   → Are we reading our own request instead of server's response?");
            } else {
                debug!("✅ Size validation: {} bytes received vs {} bytes sent (different - good!)",
                       encrypted.len(), expected_encrypted_size);
            }
        }

        // AAD = TLS record header (5 bytes)
        let aad = &header;
        debug!("AAD (TLS record header): {:02x?}", aad);

        // Build nonce: server_write_iv XOR read_sequence_number (RFC 8446 Section 5.3)
        let nonce = self.build_read_nonce();
        debug!("Nonce (server_write_iv XOR seq {}): {:02x?}", self.read_sequence_number, 
               &nonce[..std::cmp::min(12, nonce.len())]);

        // Decrypt data with SERVER write key (we're reading from server)
        // RFC 8446: Use the negotiated cipher suite for decryption
        debug!("🔓 Decrypting with server_write_key (application traffic key)");
        debug!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
        
        let decrypted = match self.keys.cipher_suite {
            0x1301 => {  // TLS_AES_128_GCM_SHA256
                debug!("   → Using AES-128-GCM for application data");
                self.beardog.decrypt_aes_128_gcm(
                    &self.keys.server_write_key,
                    &nonce,
                    &encrypted,
                    aad,
                ).await
            }
            0x1302 => {  // TLS_AES_256_GCM_SHA384
                debug!("   → Using AES-256-GCM for application data");
                self.beardog.decrypt_aes_256_gcm(
                    &self.keys.server_write_key,
                    &nonce,
                    &encrypted,
                    aad,
                ).await
            }
            0x1303 => {  // TLS_CHACHA20_POLY1305_SHA256
                debug!("   → Using ChaCha20-Poly1305 for application data");
                self.beardog.decrypt(
                    &self.keys.server_write_key,
                    &nonce,
                    &encrypted,
                    aad,
                ).await
            }
            _ => {
                error!("❌ Unsupported cipher suite for decryption: 0x{:04x}", self.keys.cipher_suite);
                return Err(Error::TlsRecord(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.keys.cipher_suite
                )));
            }
        }.map_err(|e| {
            error!("❌ Application data decryption failed: {}", e);
            error!("   Encrypted length: {} bytes", encrypted.len());
            error!("   Sequence number: {}", self.read_sequence_number);
            error!("   Cipher suite: 0x{:04x}", self.keys.cipher_suite);
            e
        })?;

        info!("✅ Decrypted {} bytes → {} bytes (AEAD authentication succeeded)", 
              encrypted.len(), decrypted.len());
        trace!("Decrypted data (first 64 bytes): {:02x?}", &decrypted[..std::cmp::min(64, decrypted.len())]);

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
            debug!("🔪 Stripped {} bytes of padding (trailing zeros)", original_len - plaintext.len());
        }
        
        // Step 2: Strip ContentType byte (should be 0x16 for handshake or 0x17 for application data)
        let content_type_byte = plaintext[plaintext.len() - 1];
        debug!("ContentType byte at end of plaintext: 0x{:02x}", content_type_byte);
        plaintext.truncate(plaintext.len() - 1);
        
        info!("✅ Stripped ContentType byte (0x{:02x}): {} bytes plaintext (HTTP data)", 
              content_type_byte, plaintext.len());
        trace!("HTTP data preview: {}", String::from_utf8_lossy(&plaintext[..std::cmp::min(200, plaintext.len())]));

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

    #[test]
    fn test_build_write_nonce() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![0; 12],
            cipher_suite: 0x1303,  // ChaCha20-Poly1305 for test
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        let nonce = layer.build_write_nonce();
        assert_eq!(nonce.len(), 12);
        
        // Sequence number should affect nonce
        layer.write_sequence_number = 1;
        let nonce2 = layer.build_write_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_build_read_nonce() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![0; 12],
            server_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            cipher_suite: 0x1303,  // ChaCha20-Poly1305 for test
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        let nonce = layer.build_read_nonce();
        assert_eq!(nonce.len(), 12);
        
        // Sequence number should affect nonce
        layer.read_sequence_number = 1;
        let nonce2 = layer.build_read_nonce();
        assert_ne!(nonce, nonce2);
    }

    #[test]
    fn test_separate_sequence_numbers() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let keys = SessionKeys {
            client_write_key: vec![0; 32],
            server_write_key: vec![0; 32],
            client_write_iv: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            server_write_iv: vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
            cipher_suite: 0x1303,  // ChaCha20-Poly1305 for test
        };
        
        let mut layer = TlsRecordLayer::new(beardog, keys);
        
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


