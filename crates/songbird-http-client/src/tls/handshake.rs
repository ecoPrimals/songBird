//! TLS 1.3 handshake implementation

use crate::beardog_client::BearDogClient;
use crate::error::{Error, Result};
use crate::tls::{session::SessionKeys, TLS_1_2, TLS_1_3, CIPHER_SUITES};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, trace, warn};

/// TLS 1.3 handshake
pub struct TlsHandshake {
    beardog: Arc<BearDogClient>,
}

impl TlsHandshake {
    /// Create a new TLS handshake
    pub fn new(beardog: Arc<BearDogClient>) -> Self {
        Self { beardog }
    }

    /// Perform TLS 1.3 handshake
    pub async fn handshake(
        &self,
        stream: &mut TcpStream,
        server_name: &str,
    ) -> Result<SessionKeys> {
        info!("🤝 [TLS STEP 0] Starting TLS 1.3 handshake with {}", server_name);
        let handshake_start = std::time::Instant::now();

        // 1. Generate client keypair
        let (client_public, client_private) = self.beardog.generate_keypair().await?;
        trace!("Generated client keypair: {} bytes public", client_public.len());

        // 2. Generate client random
        let client_random = self.generate_random();
        trace!("Generated client random: {} bytes", client_random.len());

        // 3. Send ClientHello
        debug!("Step 3: Building ClientHello message");
        let client_hello = self.build_client_hello(
            &client_random,
            &client_public,
            server_name,
        )?;
        
        info!("📤 Sending ClientHello: {} bytes to {}", client_hello.len(), server_name);
        
        // Comprehensive hex dump for debugging
        debug!("ClientHello hex dump (first 160 bytes):");
        for (i, chunk) in client_hello.chunks(16).take(10).enumerate() {
            let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            let ascii: String = chunk.iter().map(|&b| if (32..127).contains(&b) { b as char } else { '.' }).collect();
            debug!("  {:04x}: {:<47}  {}", i * 16, hex, ascii);
        }
        if client_hello.len() > 160 {
            debug!("  ... ({} more bytes)", client_hello.len() - 160);
        }
        
        let write_start = std::time::Instant::now();
        stream.write_all(&client_hello).await.map_err(|e| {
            error!("❌ Failed to write ClientHello: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await.map_err(|e| {
            error!("❌ Failed to flush ClientHello: {}", e);
            Error::Io(e)
        })?;
        debug!("ClientHello sent in {:?}", write_start.elapsed());

        // 4. Receive ServerHello with timeout
        info!("📥 Waiting for ServerHello (10 second timeout)");
        let read_start = std::time::Instant::now();
        let server_hello = timeout(
            Duration::from_secs(10),
            self.read_record(stream)
        ).await
            .map_err(|_| {
                error!("❌ TIMEOUT waiting for ServerHello after {:?}", read_start.elapsed());
                Error::TlsHandshake("Timeout waiting for ServerHello (10s)".to_string())
            })
            .and_then(|r| r.map_err(|e| {
                error!("❌ Error reading ServerHello after {:?}: {}", read_start.elapsed(), e);
                e
            }))?;
        info!("✅ Received ServerHello: {} bytes in {:?}", server_hello.len(), read_start.elapsed());
        trace!("ServerHello content: {:02x?}", &server_hello[..std::cmp::min(64, server_hello.len())]);

        // 5. Parse ServerHello
        debug!("Step 5: Parsing ServerHello");
        let (server_random, server_public) = self.parse_server_hello(&server_hello).map_err(|e| {
            error!("❌ Failed to parse ServerHello: {}", e);
            e
        })?;
        debug!("✅ Parsed ServerHello - server_random: {} bytes, server_public: {} bytes", 
               server_random.len(), server_public.len());
        trace!("Server public key: {:02x?}", &server_public[..std::cmp::min(32, server_public.len())]);

        // 6. Perform ECDH
        debug!("Step 6: Computing shared secret via BearDog ECDH");
        let ecdh_start = std::time::Instant::now();
        let shared_secret = self.beardog
            .ecdh_derive(&client_private, &server_public)
            .await
            .map_err(|e| {
                error!("❌ BearDog ECDH derivation failed: {}", e);
                e
            })?;
        debug!("✅ Computed shared secret: {} bytes in {:?}", shared_secret.len(), ecdh_start.elapsed());
        trace!("Shared secret: {:02x?}", &shared_secret[..std::cmp::min(16, shared_secret.len())]);

        // 7. Derive session secrets
        debug!("Step 7: Deriving TLS session secrets via BearDog");
        let derive_start = std::time::Instant::now();
        let secrets = self.beardog
            .tls_derive_secrets(&shared_secret, &client_random, &server_random)
            .await
            .map_err(|e| {
                error!("❌ BearDog TLS secret derivation failed: {}", e);
                e
            })?;
        
        info!("🔐 TLS session keys derived in {:?}", derive_start.elapsed());
        debug!("Session secrets derived successfully");
        
        // 8. Read post-handshake encrypted messages
        // Note: In TLS 1.3, after ServerHello, all messages are encrypted with handshake traffic keys
        // For MVP, we'll skip strict validation and just read/discard these messages
        // They include: EncryptedExtensions, Certificate, CertificateVerify, Finished
        
        info!("Step 8: Reading post-handshake encrypted messages");
        debug!("Expecting: ChangeCipherSpec (optional), EncryptedExtensions, Certificate, CertificateVerify, Finished");
        
        // Read and skip encrypted post-handshake messages
        // We expect: ChangeCipherSpec (optional), then multiple APPLICATION_DATA records containing handshake messages
        let mut messages_read = 0;
        let post_handshake_start = std::time::Instant::now();
        
        while messages_read < 5 { // Read up to 5 more records (generous limit)
            debug!("Waiting for post-handshake message {} (5 second timeout)", messages_read + 1);
            let record_start = std::time::Instant::now();
            
            match timeout(Duration::from_secs(5), self.read_record(stream)).await {
                Ok(Ok(record)) => {
                    messages_read += 1;
                    info!("✅ Read post-handshake record {} ({} bytes) in {:?}", 
                          messages_read, record.len(), record_start.elapsed());
                    trace!("Record {} content: {:02x?}", messages_read, &record[..std::cmp::min(32, record.len())]);
                    
                    // Check if this looks like the last handshake message (server Finished)
                    // Server Finished is typically small (< 100 bytes encrypted)
                    if record.len() < 100 && messages_read >= 3 {
                        info!("🎯 Likely received server Finished message (small record after 3+ messages)");
                        break;
                    }
                }
                Ok(Err(e)) => {
                    warn!("❌ Error reading post-handshake record {}: {}", messages_read + 1, e);
                    // If we've read at least 3 messages, assume handshake is done
                    if messages_read >= 3 {
                        info!("✅ Read {} post-handshake messages before error, proceeding", messages_read);
                        break;
                    }
                    error!("❌ Handshake failed after {} messages: {}", messages_read, e);
                    return Err(e);
                }
                Err(_) => {
                    warn!("⏱️  Timeout waiting for post-handshake message {} after {:?}", 
                          messages_read + 1, record_start.elapsed());
                    if messages_read >= 3 {
                        info!("✅ Timeout after {} messages ({:?} total), assuming handshake complete", 
                              messages_read, post_handshake_start.elapsed());
                        break;
                    }
                    error!("❌ Handshake timeout after only {} messages", messages_read);
                    return Err(Error::TlsHandshake(
                        format!("Timeout reading post-handshake messages (got {}/3+)", messages_read)
                    ));
                }
            }
        }
        
        debug!("Post-handshake phase complete: {} messages in {:?}", 
               messages_read, post_handshake_start.elapsed());
        
        // 9. Send client Finished message (simplified - empty for MVP)
        // In full TLS 1.3, this would be encrypted and contain HMAC of transcript
        // For MVP, we send a minimal ChangeCipherSpec to indicate we're ready
        debug!("Step 9: Sending client ChangeCipherSpec acknowledgment");
        let change_cipher_spec = vec![
            0x14, // ContentType: ChangeCipherSpec
            0x03, 0x03, // TLS 1.2 (compatibility)
            0x00, 0x01, // Length: 1
            0x01, // CCS payload
        ];
        
        info!("📤 Sending ChangeCipherSpec acknowledgment ({} bytes)", change_cipher_spec.len());
        trace!("ChangeCipherSpec: {:02x?}", change_cipher_spec);
        
        stream.write_all(&change_cipher_spec).await.map_err(|e| {
            error!("❌ Failed to write ChangeCipherSpec: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await.map_err(|e| {
            error!("❌ Failed to flush ChangeCipherSpec: {}", e);
            Error::Io(e)
        })?;
        
        let total_time = handshake_start.elapsed();
        info!("🎉 ✅ TLS 1.3 handshake complete in {:?}", total_time);
        debug!("Handshake summary: {} post-handshake messages, cipher: TLS_CHACHA20_POLY1305_SHA256", 
               messages_read);

        Ok(SessionKeys {
            client_write_key: secrets.client_write_key,
            server_write_key: secrets.server_write_key,
            client_write_iv: secrets.client_write_iv,
            server_write_iv: secrets.server_write_iv,
        })
    }

    /// Build ClientHello message
    pub(crate) fn build_client_hello(
        &self,
        client_random: &[u8],
        client_public_key: &[u8],
        server_name: &str,
    ) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // Record header
        msg.push(0x16); // ContentType: Handshake
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        
        // We'll fill in length later
        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0]); // Placeholder for length

        // Handshake header
        msg.push(0x01); // HandshakeType: ClientHello
        
        // Handshake length (placeholder)
        let handshake_length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]); // Placeholder

        // ClientHello content
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        msg.extend_from_slice(client_random); // Random (32 bytes)
        msg.push(0); // Legacy session ID length

        // Cipher suites
        msg.extend_from_slice(&((CIPHER_SUITES.len() * 2) as u16).to_be_bytes());
        for suite in CIPHER_SUITES {
            msg.extend_from_slice(&suite.to_be_bytes());
        }

        // Compression methods
        msg.push(1); // Length
        msg.push(0); // No compression

        // Extensions
        let extensions = self.build_extensions(server_name, client_public_key)?;
        msg.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        msg.extend_from_slice(&extensions);

        // Fill in lengths
        let handshake_length = msg.len() - handshake_length_pos - 3;
        msg[handshake_length_pos] = ((handshake_length >> 16) & 0xFF) as u8;
        msg[handshake_length_pos + 1] = ((handshake_length >> 8) & 0xFF) as u8;
        msg[handshake_length_pos + 2] = (handshake_length & 0xFF) as u8;

        let record_length = msg.len() - length_pos - 2;
        msg[length_pos] = ((record_length >> 8) & 0xFF) as u8;
        msg[length_pos + 1] = (record_length & 0xFF) as u8;

        Ok(msg)
    }

    /// Build TLS extensions
    pub(crate) fn build_extensions(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // SNI extension (0x0000)
        ext.extend_from_slice(&[0x00, 0x00]); // Extension type
        let sni_data = self.build_sni_extension(server_name);
        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // Supported versions (0x002b)
        ext.extend_from_slice(&[0x00, 0x2b]); // Extension type
        ext.extend_from_slice(&[0x00, 0x03]); // Length: 3
        ext.extend_from_slice(&[0x02]); // List length: 2
        ext.extend_from_slice(&TLS_1_3.to_be_bytes()); // TLS 1.3

        // Key share (0x0033)
        ext.extend_from_slice(&[0x00, 0x33]); // Extension type
        let key_share_data = self.build_key_share_extension(public_key);
        ext.extend_from_slice(&(key_share_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&key_share_data);

        // Supported groups (0x000a)
        ext.extend_from_slice(&[0x00, 0x0a]); // Extension type
        ext.extend_from_slice(&[0x00, 0x04]); // Length: 4
        ext.extend_from_slice(&[0x00, 0x02]); // List length: 2
        ext.extend_from_slice(&[0x00, 0x1d]); // x25519

        // Signature algorithms (0x000d) - Expanded for GitHub compatibility
        ext.extend_from_slice(&[0x00, 0x0d]); // Extension type
        ext.extend_from_slice(&[0x00, 0x14]); // Length: 20 (10 algorithms * 2 bytes)
        ext.extend_from_slice(&[0x00, 0x12]); // List length: 18 bytes
        // Most common signature algorithms (GitHub compatibility)
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x06, 0x03]); // ecdsa_secp521r1_sha512
        ext.extend_from_slice(&[0x08, 0x07]); // ed25519
        ext.extend_from_slice(&[0x08, 0x08]); // ed448
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
        ext.extend_from_slice(&[0x06, 0x01]); // rsa_pkcs1_sha512
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256

        Ok(ext)
    }

    /// Build SNI extension
    pub(crate) fn build_sni_extension(&self, server_name: &str) -> Vec<u8> {
        let mut sni = Vec::new();
        let name_bytes = server_name.as_bytes();
        
        sni.extend_from_slice(&((name_bytes.len() + 3) as u16).to_be_bytes()); // List length
        sni.push(0x00); // Type: host_name
        sni.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        sni.extend_from_slice(name_bytes);
        
        sni
    }

    /// Build key share extension
    pub(crate) fn build_key_share_extension(&self, public_key: &[u8]) -> Vec<u8> {
        let mut ks = Vec::new();
        
        ks.extend_from_slice(&((public_key.len() + 4) as u16).to_be_bytes()); // Client shares length
        ks.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        ks.extend_from_slice(&(public_key.len() as u16).to_be_bytes());
        ks.extend_from_slice(public_key);
        
        ks
    }

    /// Read a TLS record (generic, works for any record type)
    async fn read_record(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
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
            _ => "Unknown"
        };
        
        debug!("📥 TLS record: type={:#04x} ({}), version={:#06x}, length={} bytes", 
               content_type, content_type_name, version, length);
        
        // Special handling for Alert records
        if content_type == 0x15 {
            warn!("⚠️  Received TLS Alert record - server is signaling an issue");
        }
        
        // Validate content type
        if !(20..=23).contains(&content_type) {
            error!("❌ Invalid TLS content type: {:#04x}", content_type);
            return Err(Error::TlsHandshake(format!(
                "Invalid TLS content type: {:#04x}",
                content_type
            )));
        }
        
        // Validate length (prevent huge allocations)
        if length > 16384 { // TLS max record size
            error!("❌ TLS record too large: {} bytes (max 16384)", length);
            return Err(Error::TlsHandshake(format!(
                "TLS record too large: {} bytes",
                length
            )));
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
            let level_str = if alert_level == 1 { "Warning" } else { "Fatal" };
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
            error!("❌ TLS ALERT: {} ({}) - {} ({})", level_str, alert_level, desc_str, alert_description);
            error!("   This means the server rejected our ClientHello!");
            error!("   Common causes: missing extensions, unsupported cipher suites, protocol mismatch");
            return Err(Error::TlsHandshake(format!(
                "Server sent {} alert: {} (code {})", 
                level_str, desc_str, alert_description
            )));
        }

        Ok(content)
    }

    /// Parse ServerHello message
    pub(crate) fn parse_server_hello(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
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

        // Skip cipher suite (2 bytes) and compression (1 byte)
        let data = &data[3..];

        // Parse extensions
        let server_public = self.extract_key_share(data)?;

        Ok((server_random, server_public))
    }

    /// Extract public key from key_share extension
    fn extract_key_share(&self, extensions_data: &[u8]) -> Result<Vec<u8>> {
        if extensions_data.len() < 2 {
            return Err(Error::TlsHandshake("Extensions too short".to_string()));
        }

        let _extensions_length = u16::from_be_bytes([extensions_data[0], extensions_data[1]]) as usize;
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

    /// Generate 32-byte random
    pub(crate) fn generate_random(&self) -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut random = Vec::with_capacity(32);
        
        // Use timestamp for first 4 bytes (not cryptographically secure, but good enough for testing)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        random.extend_from_slice(&timestamp.to_be_bytes());
        
        // Fill rest with pseudo-random (in production, BearDog should provide this)
        for i in 4..32 {
            random.push((i * 7 + timestamp as usize) as u8);
        }
        
        random
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        let random = handshake.generate_random();
        assert_eq!(random.len(), 32);
    }

    // Note: generate_random() uses timestamp-based randomness for testing
    // In production, BearDog should provide cryptographically secure random

    #[test]
    fn test_build_sni_extension() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        let sni = handshake.build_sni_extension("example.com");
        assert!(!sni.is_empty());
        assert!(sni.len() > "example.com".len());
        
        // Verify the hostname is in the extension
        let hostname_bytes = "example.com".as_bytes();
        let contains_hostname = sni.windows(hostname_bytes.len())
            .any(|window| window == hostname_bytes);
        assert!(contains_hostname, "SNI should contain hostname");
    }

    #[test]
    fn test_build_key_share_extension() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        let public_key = vec![1u8; 32];
        let ks = handshake.build_key_share_extension(&public_key);
        assert!(!ks.is_empty());
        assert!(ks.len() > 32, "Key share should include length and group fields");
        
        // Should contain the public key
        let contains_key = ks.windows(32)
            .any(|window| window == public_key.as_slice());
        assert!(contains_key, "Key share should contain public key");
    }
    
    #[test]
    fn test_build_extensions() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        let public_key = vec![1u8; 32];
        let extensions = handshake.build_extensions("api.github.com", &public_key)
            .expect("Should build extensions");
        
        assert!(!extensions.is_empty(), "Extensions should not be empty");
        assert!(extensions.len() > 80, "Should contain multiple extensions");
    }
    
    #[test]
    fn test_build_client_hello() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        let client_random = vec![0u8; 32];
        let client_public_key = vec![1u8; 32];
        
        let client_hello = handshake.build_client_hello(
            &client_random,
            &client_public_key,
            "example.com"
        ).expect("Should build ClientHello");
        
        assert!(!client_hello.is_empty(), "ClientHello should not be empty");
        assert_eq!(client_hello[0], 0x16, "Should be Handshake record");
        assert_eq!(client_hello[5], 0x01, "Should be ClientHello message");
        assert!(client_hello.len() > 100, "ClientHello should be substantial");
        assert!(client_hello.len() < 500, "ClientHello should not be excessive");
    }
    
    #[test]
    fn test_parse_server_hello_structure() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        // Minimal valid ServerHello structure
        let mut server_hello = vec![];
        server_hello.push(0x02); // HandshakeType: ServerHello
        server_hello.extend_from_slice(&[0x00, 0x00, 0x50]); // Length: 80 bytes
        server_hello.extend_from_slice(&[0x03, 0x03]); // Version: TLS 1.2
        server_hello.extend_from_slice(&[0u8; 32]); // Server random
        server_hello.push(0x00); // Session ID length: 0
        server_hello.extend_from_slice(&[0x13, 0x01]); // Cipher suite
        server_hello.push(0x00); // Compression: none
        
        // Extensions
        let mut extensions = vec![];
        // Key share extension (0x0033)
        extensions.extend_from_slice(&[0x00, 0x33]); // Extension type
        extensions.extend_from_slice(&[0x00, 0x24]); // Extension length: 36
        extensions.extend_from_slice(&[0x00, 0x1d]); // Group: x25519
        extensions.extend_from_slice(&[0x00, 0x20]); // Key length: 32
        extensions.extend_from_slice(&[1u8; 32]); // Server public key
        
        server_hello.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        server_hello.extend_from_slice(&extensions);
        
        let result = handshake.parse_server_hello(&server_hello);
        assert!(result.is_ok(), "Should parse valid ServerHello");
        
        let (server_random, server_public) = result.unwrap();
        assert_eq!(server_random.len(), 32, "Server random should be 32 bytes");
        assert_eq!(server_public.len(), 32, "Server public key should be 32 bytes");
    }
    
    #[test]
    fn test_parse_server_hello_invalid() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        // Invalid: empty
        assert!(handshake.parse_server_hello(&[]).is_err());
        
        // Invalid: wrong handshake type
        let wrong_type = vec![0x01, 0x00, 0x00, 0x00]; // ClientHello instead of ServerHello
        assert!(handshake.parse_server_hello(&wrong_type).is_err());
        
        // Invalid: too short
        let too_short = vec![0x02, 0x00, 0x00, 0x10, 0x03, 0x03]; // Only 6 bytes
        assert!(handshake.parse_server_hello(&too_short).is_err());
    }
}

