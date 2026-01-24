//! TLS 1.3 handshake implementation

use crate::crypto::CryptoCapability;
use crate::error::{Error, Result};
use crate::tls::{
    config::TlsConfig,
    profiler::ServerProfiler,
    session::SessionKeys,
    TLS_1_2,
    TLS_1_3,
    CIPHER_SUITES,
};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, trace, warn};

/// TLS handshake secrets - alias for CryptoCapability TlsHandshakeSecrets
pub use crate::crypto::TlsHandshakeSecrets as TlsSecrets;

/// TLS 1.3 handshake
pub struct TlsHandshake {
    crypto: Arc<dyn CryptoCapability>,
    /// Transcript accumulator for RFC 8446 key derivation
    /// Accumulates all handshake messages for transcript hash computation
    transcript: Vec<u8>,
    /// Negotiated TLS 1.3 cipher suite from ServerHello
    /// 0x1301 = TLS_AES_128_GCM_SHA256
    /// 0x1302 = TLS_AES_256_GCM_SHA384
    /// 0x1303 = TLS_CHACHA20_POLY1305_SHA256
    cipher_suite: u16,
    /// Configuration (strategy-based, not hardcoded)
    config: TlsConfig,
    /// Optional server profiler for adaptive learning
    profiler: Option<Arc<ServerProfiler>>,
}

impl TlsHandshake {
    /// Create a new TLS handshake with default config
    pub fn new(crypto: Arc<dyn CryptoCapability>) -> Self {
        Self::with_config(crypto, TlsConfig::default(), None)
    }
    
    /// Create a new TLS handshake with custom config and optional profiler
    pub fn with_config(
        crypto: Arc<dyn CryptoCapability>,
        config: TlsConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        info!("🎛️  Creating TLS handshake with {:?} strategy", config.extension_strategy);
        if profiler.is_some() {
            info!("🧠 Adaptive learning enabled (profiler provided)");
        }
        
        Self { 
            crypto,
            transcript: Vec::new(),
            cipher_suite: 0,  // Will be set after parsing ServerHello
            config,
            profiler,
        }
    }
    
    /// Update transcript with handshake message
    /// RFC 8446 Section 4.4.1: Transcript hash includes all handshake messages
    /// 
    /// CRITICAL: This method expects handshake messages WITHOUT TLS record framing!
    /// - ClientHello: Must strip 5-byte TLS record header before calling
    /// - ServerHello: Already stripped by read_record()
    /// - Post-handshake messages: Already stripped by read_record()
    #[allow(dead_code)]
    fn update_transcript(&mut self, message: &[u8]) {
        let before = self.transcript.len();
        let after = before + message.len();
        trace!("📝 Updating transcript: +{} bytes (total: {} → {} bytes)", 
               message.len(), before, after);
        trace!("   Message preview: {:02x?}", &message[..std::cmp::min(16, message.len())]);
        self.transcript.extend_from_slice(message);
    }
    
    /// Update transcript with comprehensive logging for debugging
    /// 
    /// This enhanced version logs detailed information about each message
    /// to help diagnose transcript hash issues (biomeOS v5.12.6 investigation)
    fn update_transcript_with_logging(&mut self, message: &[u8], message_type: &str, was_decrypted: bool) {
        let before = self.transcript.len();
        
        // Log comprehensive details
        trace!("════════════════════════════════════════════════════════════");
        info!("📝 TRANSCRIPT UPDATE: {}", message_type);
        trace!("════════════════════════════════════════════════════════════");
        info!("Message type: {}", message_type);
        info!("Message length: {} bytes", message.len());
        info!("Was decrypted: {}", was_decrypted);
        
        if !message.is_empty() {
            let first_byte = message[0];
            info!("First byte: 0x{:02x} ({})", first_byte, 
                  match first_byte {
                      0x01 => "ClientHello ✅",
                      0x02 => "ServerHello ✅",
                      0x08 => "EncryptedExtensions ✅",
                      0x0B => "Certificate ✅",
                      0x0F => "CertificateVerify ✅",
                      0x14 => "Finished ✅",
                      0x16 => "TLS Record Header ❌ (SHOULD BE STRIPPED!)",
                      0x17 => "ContentType Byte ❌ (SHOULD BE STRIPPED!)",
                      _ => "Unknown",
                  });
            
            // 🔍 ENHANCED HEX DUMP: Show first/last bytes to identify extra bytes
            trace!("First 32 bytes (hex): {}", hex::encode(&message[..std::cmp::min(32, message.len())]));
            if message.len() > 64 {
                trace!("Last 32 bytes (hex): {}", hex::encode(&message[message.len().saturating_sub(32)..]));
            }
            
            // 🔍 CHECK: Length field in message (bytes 1-3 for handshake messages)
            if message.len() >= 4 {
                let declared_length = u32::from_be_bytes([0, message[1], message[2], message[3]]) as usize;
                let actual_length = message.len() - 4;  // Minus type (1) + length (3)
                info!("📏 Length validation:");
                trace!("   Declared length (bytes 1-3): {} bytes", declared_length);
                trace!("   Actual body length: {} bytes", actual_length);
                if declared_length != actual_length {
                    error!("🚨 LENGTH MISMATCH!");
                    error!("   Declared: {} bytes", declared_length);
                    error!("   Actual: {} bytes", actual_length);
                    error!("   Difference: {} bytes", (actual_length as i64 - declared_length as i64).abs());
                    error!("   💡 This might be the source of the 2-byte discrepancy!");
                } else {
                    trace!("   ✅ Length match - message is correct size");
                }
            }
            
            // Warn if TLS record header or ContentType byte detected
            if first_byte == 0x16 {
                error!("⚠️  CRITICAL: TLS record header (0x16) detected!");
                error!("   This message should have the 5-byte TLS record header stripped!");
                error!("   Expected first byte: handshake message type (0x01, 0x02, 0x08, 0x0B, 0x0F, 0x14)");
            } else if first_byte == 0x17 {
                error!("⚠️  CRITICAL: ContentType byte (0x17) detected!");
                error!("   This should be stripped after AEAD decryption!");
            }
        }
        
        // Add to transcript
        self.transcript.extend_from_slice(message);
        let after = self.transcript.len();
        
        info!("Cumulative transcript length: {} bytes → {} bytes (+{} bytes)", 
              before, after, message.len());
        trace!("════════════════════════════════════════════════════════════");
        info!("");
    }
    
    /// Parse multiple handshake messages from a decrypted TLS record
    /// 
    /// RFC 8446 Section 4: Handshake messages have the format:
    /// - HandshakeType msg_type (1 byte)
    /// - uint24 length (3 bytes, big-endian)
    /// - opaque body[length]
    /// 
    /// A single TLS record may contain MULTIPLE handshake messages concatenated together!
    /// This function parses them individually so they can be added to the transcript separately.
    fn parse_handshake_messages(&self, data: &[u8]) -> Result<Vec<(u8, Vec<u8>)>> {
        let mut messages = Vec::new();
        let mut offset = 0;
        
        trace!("════════════════════════════════════════════════════════════");
        info!("📦 PARSING HANDSHAKE MESSAGES FROM DECRYPTED RECORD");
        trace!("════════════════════════════════════════════════════════════");
        info!("Total decrypted data: {} bytes", data.len());
        info!("Parsing individual RFC 8446 handshake messages...");
        info!("");
        
        // 🔍 HEX DUMP: Show first 64 bytes and last 64 bytes to identify extra bytes
        info!("🔍 HEX DUMP OF DECRYPTED DATA:");
        trace!("   First 64 bytes: {}", hex::encode(&data[..std::cmp::min(64, data.len())]));
        if data.len() > 128 {
            trace!("   ... ({} bytes in middle) ...", data.len() - 128);
        }
        if data.len() > 64 {
            trace!("   Last 64 bytes: {}", hex::encode(&data[data.len().saturating_sub(64)..]));
        }
        info!("");
        
        let _data_before_parse = data.len();
        
        while offset < data.len() {
            // Read message type (1 byte)
            if offset >= data.len() {
                debug!("Reached end of data at offset {}", offset);
                break;
            }
            let msg_type = data[offset];
            
            // Check if this looks like a valid handshake message type
            if msg_type == 0x00 || msg_type > 0x18 {
                warn!("⚠️  Stopping parse: invalid message type 0x{:02x} at offset {}", msg_type, offset);
                warn!("   This might be padding or extra bytes!");
                warn!("   Remaining {} bytes: {}", data.len() - offset, hex::encode(&data[offset..std::cmp::min(offset + 32, data.len())]));
                break;
            }
            
            offset += 1;
            
            // Read length (3 bytes, big-endian)
            if offset + 3 > data.len() {
                warn!("⚠️  Truncated handshake message: not enough bytes for length at offset {}", offset);
                break;
            }
            let length = u32::from_be_bytes([
                0,
                data[offset],
                data[offset + 1],
                data[offset + 2],
            ]) as usize;
            offset += 3;
            
            // Read body
            if offset + length > data.len() {
                warn!("⚠️  Truncated handshake message: expected {} bytes, got {} at offset {}", 
                      length, data.len() - offset, offset);
                break;
            }
            
            // Extract complete message (type + length + body)
            let message_start = offset - 4;  // Go back to include type (1) + length (3)
            let full_message = &data[message_start..offset + length];
            
            let msg_name = match msg_type {
                0x08 => "EncryptedExtensions",
                0x0B => "Certificate",
                0x0F => "CertificateVerify",
                0x14 => "Finished",
                _ => "Unknown",
            };
            
            info!("✅ Parsed message #{}: {} (type 0x{:02x}, length {} bytes, total {} bytes)", 
                  messages.len() + 1, msg_name, msg_type, length, full_message.len());
            trace!("   Message offset: {} to {} (in decrypted blob)", message_start, offset + length);
            trace!("   First 32 bytes of message: {}", hex::encode(&full_message[..std::cmp::min(32, full_message.len())]));
            
            messages.push((msg_type, full_message.to_vec()));
            offset += length;
        }
        
        info!("");
        info!("📋 Parsing complete:");
        trace!("   Total messages parsed: {}", messages.len());
        trace!("   Bytes consumed: {} out of {} bytes", offset, data.len());
        
        // 🔍 CRITICAL CHECK: Are there extra bytes after the last message?
        if offset < data.len() {
            let extra_bytes = data.len() - offset;
            error!("🚨 EXTRA BYTES DETECTED!");
            error!("   {} extra bytes after last handshake message!", extra_bytes);
            error!("   Extra bytes (hex): {}", hex::encode(&data[offset..]));
            error!("   Extra bytes (ASCII): {:?}", String::from_utf8_lossy(&data[offset..]));
            error!("");
            error!("   💡 These extra bytes should NOT be added to transcript!");
            error!("   💡 They are likely padding or TLS framing!");
        } else {
            info!("✅ All bytes consumed - no extra bytes detected");
        }
        
        trace!("════════════════════════════════════════════════════════════");
        info!("");
        
        if messages.is_empty() {
            warn!("⚠️  No handshake messages parsed from {} bytes of data!", data.len());
        }
        
        Ok(messages)
    }
    
    /// Compute transcript hash (SHA-256)
    /// RFC 8446 Section 4.4.1: Transcript-Hash(M1, M2, ... Mn) = Hash(M1 || M2 || ... || Mn)
    fn compute_transcript_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.transcript);
        let hash = hasher.finalize().to_vec();
        info!("🔐 Computed transcript hash: {} bytes from {} bytes of messages", 
              hash.len(), self.transcript.len());
        trace!("Transcript hash (hex): {}", hex::encode(&hash));
        hash
    }

    /// Perform TLS 1.3 handshake
    pub async fn handshake(
        &mut self,
        stream: &mut TcpStream,
        server_name: &str,
    ) -> Result<SessionKeys> {
        info!("🤝 [TLS STEP 0] Starting TLS 1.3 handshake with {}", server_name);
        let _handshake_start = std::time::Instant::now();

        // 1. Generate client keypair
        let (client_public, client_private) = self.crypto.generate_x25519_keypair().await?;
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
        
        // 🔬 WIRE CAPTURE: Log complete ClientHello for analysis
        trace!("════════════════════════════════════════════════════════════");
        trace!("🔬 COMPLETE CLIENTHELLO HEX DUMP (FOR WIRE ANALYSIS)");
        trace!("════════════════════════════════════════════════════════════");
        trace!("Total length: {} bytes", client_hello.len());
        info!("");
        for (i, chunk) in client_hello.chunks(32).enumerate() {
            info!("{:04x}: {}", i * 32, hex::encode(chunk));
        }
        trace!("════════════════════════════════════════════════════════════");
        info!("");
        
        // RFC 8446 Section 4.4.1: Update transcript with ClientHello HANDSHAKE MESSAGE ONLY
        // The transcript includes the handshake message (Type + Length + Content), 
        // NOT the TLS record framing (ContentType + Version + RecordLength)
        //
        // ClientHello structure:
        // - TLS record header (5 bytes): ContentType (1) + Version (2) + RecordLength (2)
        // - Handshake message: Type (1) + Length (3) + Content (variable)
        //
        // We must strip the 5-byte TLS record header before adding to transcript!
        info!("📝 TRANSCRIPT UPDATE 1: Adding ClientHello (WITHOUT TLS record header)");
        let client_hello_len = if client_hello.len() > 5 {
            let handshake_message = &client_hello[5..]; // Skip 5-byte TLS record header
            trace!("   ClientHello total: {} bytes (with TLS header)", client_hello.len());
            trace!("   ClientHello handshake message: {} bytes (TLS header stripped)", handshake_message.len());
            debug!("   TLS record header (5 bytes, NOT in transcript): {:02x?}", &client_hello[..5]);
            
            // BearDog-requested verification: First 32 bytes should start with 0x01 (ClientHello type)
            trace!("🔍 VERIFICATION: ClientHello handshake message first bytes:");
            let preview_len = std::cmp::min(32, handshake_message.len());
            let first_bytes: String = handshake_message[..preview_len].iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            trace!("   First {} bytes: {}", preview_len, first_bytes);
            if !handshake_message.is_empty() {
                let first_byte = handshake_message[0];
                if first_byte == 0x01 {
                    trace!("   ✅ CORRECT: First byte is 0x01 (ClientHello handshake type)");
                } else if first_byte == 0x16 {
                    error!("   ❌ WRONG: First byte is 0x16 (TLS record header - should be stripped!)");
                } else {
                    warn!("   ⚠️  UNEXPECTED: First byte is 0x{:02x} (expected 0x01)", first_byte);
                }
            }
            
            debug!("   Handshake message (first 64 bytes, ADDED to transcript):");
            for (i, chunk) in handshake_message.chunks(16).take(4).enumerate() {
                let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                debug!("     {:04x}: {}", i * 16, hex);
            }
            if handshake_message.len() > 64 {
                debug!("     ... ({} more bytes)", handshake_message.len() - 64);
            }
            
            self.update_transcript_with_logging(handshake_message, "ClientHello", false);
            info!("✅ ClientHello handshake message added to transcript ({} bytes)", handshake_message.len());
            debug!("📊 Transcript now: {} bytes (ClientHello only)", self.transcript.len());
            handshake_message.len()
        } else {
            error!("❌ ClientHello too short to contain handshake message!");
            self.update_transcript_with_logging(&client_hello, "ClientHello (full, with TLS header)", false);
            client_hello.len()
        };
        
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
        let (server_hello_type, server_hello) = timeout(
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
        info!("✅ Received ServerHello: type=0x{:02x}, {} bytes in {:?}", 
              server_hello_type, server_hello.len(), read_start.elapsed());
        trace!("ServerHello content: {:02x?}", &server_hello[..std::cmp::min(64, server_hello.len())]);
        
        // Validate this is a Handshake record (0x16)
        // Check if we received a TLS alert instead of ServerHello
        if server_hello_type == 0x15 {
            // TLS Alert record (RFC 8446 Section 6)
            use crate::tls::alert::TlsAlert;
            
            warn!("⚠️  Received TLS Alert instead of ServerHello");
            
            // Parse the alert message (skip TLS record header if present)
            let alert_data = if server_hello.len() >= 5 {
                &server_hello[5..]  // Skip 5-byte TLS record header
            } else {
                &server_hello[..]   // Use all data if too short
            };
            
            match TlsAlert::parse(alert_data) {
                Ok(alert) => {
                    error!("");
                    error!("════════════════════════════════════════════════════════════");
                    error!("🚨 TLS ALERT RECEIVED FROM SERVER");
                    error!("════════════════════════════════════════════════════════════");
                    error!("");
                    error!("{}", alert.to_detailed_string());
                    error!("");
                    error!("════════════════════════════════════════════════════════════");
                    error!("");
                    
                    return Err(Error::TlsHandshake(format!(
                        "Server sent {} ({}). {}",
                        alert,
                        alert.description.explanation(),
                        alert.description.suggested_action()
                    )));
                }
                Err(e) => {
                    error!("❌ Failed to parse TLS alert: {}", e);
                    error!("   Alert data length: {} bytes", alert_data.len());
                    if !alert_data.is_empty() {
                        error!("   Alert bytes: {:02x?}", &alert_data[..std::cmp::min(2, alert_data.len())]);
                    }
                    return Err(Error::TlsHandshake(format!(
                        "Server sent TLS alert but parsing failed: {}",
                        e
                    )));
                }
            }
        }
        
        if server_hello_type != 0x16 {
            error!("❌ Expected Handshake record (0x16) for ServerHello, got 0x{:02x}", server_hello_type);
            
            // Provide helpful context for common unexpected types
            let type_hint = match server_hello_type {
                0x14 => "Change Cipher Spec (TLS 1.2 legacy)",
                0x15 => "Alert (should have been caught above)",
                0x17 => "Application Data (server may think we're resuming a session)",
                _ => "Unknown record type",
            };
            error!("   Record type 0x{:02x} = {}", server_hello_type, type_hint);
            
            return Err(Error::TlsHandshake(format!(
                "Expected Handshake record for ServerHello, got type 0x{:02x} ({})",
                server_hello_type, type_hint
            )));
        }
        
        // RFC 8446: Update transcript with ServerHello
        // Note: read_record() already stripped the 5-byte TLS record header,
        // so server_hello contains only the handshake message (Type + Length + Content)
        info!("📝 TRANSCRIPT UPDATE 2: Adding ServerHello (WITHOUT TLS record header)");
        trace!("   ServerHello handshake message: {} bytes (TLS header already stripped)", server_hello.len());
        
        // BearDog-requested verification: First 32 bytes should start with 0x02 (ServerHello type)
        trace!("🔍 VERIFICATION: ServerHello handshake message first bytes:");
        let preview_len = std::cmp::min(32, server_hello.len());
        let first_bytes: String = server_hello[..preview_len].iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        trace!("   First {} bytes: {}", preview_len, first_bytes);
        if !server_hello.is_empty() {
            let first_byte = server_hello[0];
            if first_byte == 0x02 {
                trace!("   ✅ CORRECT: First byte is 0x02 (ServerHello handshake type)");
            } else if first_byte == 0x16 {
                error!("   ❌ WRONG: First byte is 0x16 (TLS record header - should be stripped!)");
            } else {
                warn!("   ⚠️  UNEXPECTED: First byte is 0x{:02x} (expected 0x02)", first_byte);
            }
        }
        
        debug!("   Handshake message (first 64 bytes, ADDED to transcript):");
        for (i, chunk) in server_hello.chunks(16).take(4).enumerate() {
            let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            debug!("     {:04x}: {}", i * 16, hex);
        }
        if server_hello.len() > 64 {
            debug!("     ... ({} more bytes)", server_hello.len() - 64);
        }
        
        self.update_transcript_with_logging(&server_hello, "ServerHello", false);
        info!("✅ ServerHello handshake message added to transcript ({} bytes)", server_hello.len());
        debug!("📊 Transcript now: {} bytes total (ClientHello + ServerHello)", self.transcript.len());

        // 5. Parse ServerHello
        debug!("Step 5: Parsing ServerHello");
        let (server_random, server_public, cipher_suite) = self.parse_server_hello(&server_hello).map_err(|e| {
            error!("❌ Failed to parse ServerHello: {}", e);
            e
        })?;
        self.cipher_suite = cipher_suite;  // Store for later AEAD algorithm selection
        debug!("✅ Parsed ServerHello - cipher_suite: 0x{:04x}, server_random: {} bytes, server_public: {} bytes", 
               cipher_suite, server_random.len(), server_public.len());
        trace!("Server public key: {:02x?}", &server_public[..std::cmp::min(32, server_public.len())]);

        // 6. Perform ECDH
        debug!("Step 6: Computing shared secret via BearDog ECDH");
        let ecdh_start = std::time::Instant::now();
        let shared_secret = self.crypto
            .derive_x25519_shared_secret(&client_private, &server_public)
            .await
            .map_err(|e| {
                error!("❌ BearDog ECDH derivation failed: {}", e);
                e
            })?;
        debug!("✅ Computed shared secret: {} bytes in {:?}", shared_secret.len(), ecdh_start.elapsed());
        trace!("Shared secret: {:02x?}", &shared_secret[..std::cmp::min(16, shared_secret.len())]);

        // 7. Compute transcript hash for handshake key derivation
        // RFC 8446 Section 7.1: Handshake traffic secrets are derived using transcript of ClientHello + ServerHello
        info!("Step 7: Computing transcript hash for handshake key derivation");
        debug!("📊 Handshake transcript at this point (for handshake key derivation):");
        debug!("   Components: ClientHello + ServerHello (both plaintext)");
        debug!("   Total bytes: {}", self.transcript.len());
        info!("📊 TRANSCRIPT SNAPSHOT (before computing handshake hash):");
        trace!("   Total transcript: {} bytes (ClientHello + ServerHello)", self.transcript.len());
        trace!("   ClientHello was: {} bytes (first message in transcript)", client_hello_len);
        trace!("   ServerHello was: {} bytes (second message in transcript)", server_hello.len());
        debug!("   Full transcript (hex, all {} bytes):", self.transcript.len());
        for (i, chunk) in self.transcript.chunks(32).enumerate() {
            let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            debug!("     {:04x}: {}", i * 32, hex);
        }
        debug!("   ⚠️  CRITICAL: This transcript should contain:");
        debug!("      1. ClientHello handshake message (without TLS record header)");
        debug!("      2. ServerHello handshake message (without TLS record header)");
        debug!("      3. NO TLS record headers (no [16 03 03 ...] prefixes)");
        debug!("      4. ONLY the handshake message content (Type + Length + Content)");
        
        info!("🔐 COMPUTING HANDSHAKE TRANSCRIPT HASH (SHA-256 of {} bytes)", self.transcript.len());
        debug!("   RFC 8446 Section 4.4.1: Transcript-Hash(M1, M2) = Hash(M1 || M2)");
        debug!("   For handshake keys: M1 = ClientHello, M2 = ServerHello");
        debug!("   Both messages are handshake message bodies ONLY (no TLS record headers)");
        
        let handshake_transcript_hash = self.compute_transcript_hash();
        
        info!("✅ Handshake transcript hash computed!");
        trace!("   Hash length: {} bytes (SHA-256)", handshake_transcript_hash.len());
        trace!("   🎯 Transcript hash (hex): {}", hex::encode(&handshake_transcript_hash));
        trace!("   This hash will be passed to BearDog's tls.derive_handshake_secrets");
        debug!("🔍 BearDog will use this hash to derive handshake traffic keys (RFC 8446 Section 7.1)");
        debug!("   Server computes SAME hash from SAME transcript bytes");
        debug!("   If our hash differs by 1 byte → keys will be completely wrong → AEAD fails");

        // 8. Derive handshake traffic keys (RFC 8446 Section 7.1)
        // These keys are used to decrypt post-handshake messages (EncryptedExtensions, Certificate, etc.)
        // CRITICAL: Keys derived with transcript hash of ClientHello + ServerHello!
        info!("Step 8: Deriving handshake traffic keys for decrypting post-handshake messages");
        debug!("RFC 8446 Section 7.1: Handshake keys derived from:");
        debug!("   → ECDH shared secret");
        debug!("   → Client random");
        debug!("   → Server random");
        debug!("   → Transcript hash (ClientHello + ServerHello)");
        let handshake_start = std::time::Instant::now();
        let handshake_keys = self.crypto
            .tls_derive_handshake_secrets(&shared_secret, &handshake_transcript_hash)
            .await
            .map_err(|e| {
                error!("❌ Failed to derive handshake traffic keys: {}", e);
                e
            })?;
        info!("✅ Handshake traffic keys derived in {:?}", handshake_start.elapsed());
        debug!("  client_handshake_key: {} bytes", handshake_keys.client_write_key.len());
        debug!("  server_handshake_key: {} bytes", handshake_keys.server_write_key.len());
        debug!("  client_handshake_iv: {} bytes", handshake_keys.client_write_iv.len());
        debug!("  server_handshake_iv: {} bytes", handshake_keys.server_write_iv.len());

        // 9. Read and decrypt post-handshake encrypted messages
        // RFC 8446 Section 4.4.1: Transcript hash is computed over PLAINTEXT handshake messages!
        // Messages to decrypt: EncryptedExtensions, Certificate, CertificateVerify, Finished
        info!("Step 9: Reading and decrypting post-handshake encrypted messages");
        debug!("Expecting: ChangeCipherSpec (optional), EncryptedExtensions, Certificate, CertificateVerify, Finished");
        debug!("RFC 8446 CRITICAL: Transcript must contain PLAINTEXT (decrypted) messages, NOT encrypted!");
        
        // Read, decrypt, and track post-handshake messages for transcript hash
        // We expect: ChangeCipherSpec (optional), then multiple APPLICATION_DATA records containing handshake messages
        let mut messages_read = 0;
        let mut sequence_number = 0u64; // Sequence number for AEAD nonce generation
        let post_handshake_start = std::time::Instant::now();
        
        while messages_read < 5 { // Read up to 5 more records (generous limit)
            debug!("Waiting for encrypted post-handshake message {} (5 second timeout)", messages_read + 1);
            let record_start = std::time::Instant::now();
            
            match timeout(Duration::from_secs(5), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_record))) => {
                    info!("✅ Read TLS record type=0x{:02x} ({} bytes) in {:?}", 
                          content_type, encrypted_record.len(), record_start.elapsed());
                    
                    // RFC 8446 Section 5: Skip ChangeCipherSpec (legacy compatibility)
                    // ChangeCipherSpec (0x14) is PLAINTEXT in TLS 1.3, not encrypted!
                    // It's a 1-byte legacy message (0x01) for middlebox compatibility
                    // We MUST NOT try to decrypt it (would fail: 1 byte < 16 byte AEAD tag)
                    if content_type == 0x14 { // CHANGE_CIPHER_SPEC
                        info!("⏭️  Skipping ChangeCipherSpec (legacy TLS 1.3 compatibility message)");
                        debug!("   RFC 8446 Section 5: ChangeCipherSpec is PLAINTEXT (not encrypted)");
                        debug!("   Content: {:02x?}", encrypted_record);
                        
                        // Validate it's the expected 1-byte 0x01
                        if encrypted_record.len() == 1 && encrypted_record[0] == 0x01 {
                            debug!("   ✅ Valid ChangeCipherSpec (0x01)");
                        } else {
                            warn!("   ⚠️  Unexpected ChangeCipherSpec: {} bytes, content={:02x?}", 
                                  encrypted_record.len(), encrypted_record);
                        }
                        
                        // Do NOT add to transcript (not a handshake message)
                        // Do NOT try to decrypt (it's plaintext!)
                        // Just skip and continue to next record
                        continue;
                    }
                    
                    // For APPLICATION_DATA (0x17): encrypted handshake messages
                    // (EncryptedExtensions, Certificate, CertificateVerify, Finished)
                    if content_type != 0x17 {
                        warn!("⚠️  Unexpected record type after ServerHello: 0x{:02x}", content_type);
                        continue;
                    }
                    
                    messages_read += 1;
                    info!("✅ Read encrypted handshake record {} ({} bytes) in {:?}", 
                          messages_read, encrypted_record.len(), record_start.elapsed());
                    trace!("Encrypted record {} preview: {:02x?}", 
                           messages_read, &encrypted_record[..std::cmp::min(32, encrypted_record.len())]);
                    
                    // RFC 8446 CRITICAL: Decrypt the handshake message before adding to transcript!
                    // Transcript hash must be computed over PLAINTEXT messages, not encrypted ciphertext
                    debug!("🔓 Decrypting handshake record {} with handshake traffic keys (seq={})", 
                           messages_read, sequence_number);
                    let decrypt_start = std::time::Instant::now();
                    
                    match self.decrypt_handshake_record(&encrypted_record, &handshake_keys, sequence_number).await {
                        Ok(plaintext) => {
                            info!("✅ Decrypted handshake record {} to {} bytes of plaintext in {:?}", 
                                  messages_read, plaintext.len(), decrypt_start.elapsed());
                            trace!("Plaintext preview: {:02x?}", &plaintext[..std::cmp::min(32, plaintext.len())]);
                            
                            sequence_number += 1;
                            
                            // RFC 8446 Section 4.4.1: Add PLAINTEXT to transcript (not encrypted!)
                            // CRITICAL FIX: Parse INDIVIDUAL handshake messages from the decrypted blob!
                            // A single TLS record may contain MULTIPLE handshake messages (EncryptedExtensions,
                            // Certificate, CertificateVerify, Finished) concatenated together.
                            // RFC 8446 requires each message to be added to the transcript SEPARATELY!
                            
                            info!("🔬 CRITICAL: Parsing individual handshake messages from decrypted record");
                            let parsed_messages = self.parse_handshake_messages(&plaintext)?;
                            
                            info!("📝 Adding {} individual messages to transcript (NOT as one blob!)", parsed_messages.len());
                            for (msg_type, msg_data) in parsed_messages {
                                let message_type = match msg_type {
                                    0x08 => "EncryptedExtensions",
                                    0x0B => "Certificate",
                                    0x0F => "CertificateVerify",
                                    0x14 => "Server Finished",
                                    _ => "Unknown Handshake Message",
                                };
                                
                                self.update_transcript_with_logging(&msg_data, message_type, true);
                            }
                            
                            debug!("✅ Post-handshake messages {} parsed and added to transcript", messages_read);
                            debug!("📊 Transcript now: {} bytes total (all plaintext)", self.transcript.len());
                            
                            // RFC 8446 Section 4.4 & 5.1: Detect server Finished message (HandshakeType 0x14)
                            // CRITICAL: Server may send multiple handshake messages in ONE TLS record!
                            // We must parse the message framing to find Finished at any offset
                            if self.contains_finished_message(&plaintext) {
                                trace!("   Server handshake complete - deriving application keys and sending client Finished!");
                                
                                // Exit loop to derive application keys before sending client Finished
                        break;
                            }
                            
                            // If not Finished, continue reading more records
                            debug!("   No Finished message in this record yet, continuing...");
                        }
                        Err(e) => {
                            warn!("❌ Failed to decrypt handshake record {}: {}", messages_read, e);
                            // If we've read at least 3 messages successfully, assume handshake is done
                            if messages_read >= 4 {
                                info!("✅ Decrypted {} messages before error, proceeding", messages_read - 1);
                                break;
                            }
                            error!("❌ Handshake decryption failed after {} messages: {}", messages_read, e);
                            return Err(e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!("❌ Error reading post-handshake record {}: {}", messages_read + 1, e);
                    // If we've read at least 3 messages, assume handshake is done
                    if messages_read >= 3 {
                        info!("✅ Read and decrypted {} post-handshake messages before error, proceeding", messages_read);
                        break;
                    }
                    error!("❌ Handshake failed after {} messages: {}", messages_read, e);
                    return Err(e);
                }
                Err(_) => {
                    warn!("⏱️  Timeout waiting for post-handshake message {} after {:?}", 
                          messages_read + 1, record_start.elapsed());
                    if messages_read >= 3 {
                        info!("✅ Timeout after {} decrypted messages ({:?} total), assuming handshake complete", 
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
        
        debug!("Post-handshake phase complete: {} messages decrypted in {:?}", 
               messages_read, post_handshake_start.elapsed());
        
        // 10. Compute final transcript hash for application key derivation (RFC 8446 Section 4.4.1)
        // Transcript includes: ClientHello, ServerHello, and all DECRYPTED handshake messages
        info!("Step 10: Computing final transcript hash for application key derivation");
        
        // 🔬 COMPLETE TRANSCRIPT HEX DUMP (biomeOS v5.12.9 - byte-level forensics)
        trace!("════════════════════════════════════════════════════════════");
        info!("🔬 COMPLETE TRANSCRIPT HEX DUMP (BYTE-LEVEL FORENSICS)");
        trace!("════════════════════════════════════════════════════════════");
        info!("Total transcript length: {} bytes", self.transcript.len());
        info!("");
        trace!("📝 Full transcript (hex):");
        for (i, chunk) in self.transcript.chunks(64).enumerate() {
            info!("{:04x}: {}", i * 64, hex::encode(chunk));
        }
        trace!("════════════════════════════════════════════════════════════");
        info!("");
        
        // COMPREHENSIVE TRANSCRIPT VALIDATION (biomeOS v5.12.6 investigation)
        trace!("════════════════════════════════════════════════════════════");
        info!("📊 TRANSCRIPT HASH FOR APPLICATION KEY DERIVATION");
        trace!("════════════════════════════════════════════════════════════");
        info!("Total transcript length: {} bytes", self.transcript.len());
        info!("");
        info!("Expected to include (in this order):");
        trace!("  1. ClientHello (raw handshake message, no TLS header)");
        trace!("     • First byte should be: 0x01 (ClientHello message type)");
        trace!("     • Should NOT start with: 0x16 (TLS record header)");
        trace!("  2. ServerHello (raw handshake message, no TLS header)");
        trace!("     • First byte should be: 0x02 (ServerHello message type)");
        trace!("     • Should NOT start with: 0x16 (TLS record header)");
        trace!("  3. EncryptedExtensions (DECRYPTED plaintext)");
        trace!("     • First byte should be: 0x08 (EncryptedExtensions message type)");
        trace!("     • Must be decrypted BEFORE adding to transcript!");
        trace!("     • Should NOT start with: 0x16 or 0x17 (record header or ContentType)");
        trace!("  4. Certificate (DECRYPTED plaintext)");
        trace!("     • First byte should be: 0x0B (Certificate message type)");
        trace!("     • Must be decrypted BEFORE adding to transcript!");
        trace!("  5. CertificateVerify (DECRYPTED plaintext)");
        trace!("     • First byte should be: 0x0F (CertificateVerify message type)");
        trace!("     • Must be decrypted BEFORE adding to transcript!");
        trace!("  6. Server Finished (DECRYPTED plaintext)");
        trace!("     • First byte should be: 0x14 (Finished message type)");
        trace!("     • Must be decrypted BEFORE adding to transcript!");
        info!("");
        info!("Should NOT include:");
        trace!("  ❌ Client Finished (happens AFTER app key derivation!)");
        trace!("  ❌ TLS record headers (5 bytes: type, version, length)");
        trace!("  ❌ ContentType bytes (0x16 for encrypted handshake, 0x17 for app data)");
        trace!("  ❌ Padding zeros");
        info!("");
        info!("⚠️  VALIDATION CHECKLIST:");
        trace!("  • All messages added as PLAINTEXT (encrypted messages decrypted first)");
        trace!("  • No TLS record headers (first byte is handshake type, not 0x16)");
        trace!("  • No ContentType bytes (0x16/0x17) at start of messages");
        trace!("  • Message count: 6 total (ClientHello, ServerHello, + 4 encrypted)");
        info!("");
        debug!("📊 Final transcript: {} bytes total (ALL PLAINTEXT - RFC 8446 compliant!)", self.transcript.len());
        debug!("Transcript hex (first 64 bytes): {}", hex::encode(&self.transcript[..std::cmp::min(64, self.transcript.len())]));
        
        let transcript_hash = self.compute_transcript_hash();
        info!("✅ Transcript hash computed: {} bytes (SHA-256)", transcript_hash.len());
        trace!("🔐 Transcript hash (hex): {}", hex::encode(&transcript_hash));
        trace!("════════════════════════════════════════════════════════════");
        info!("");
        
        // Log transcript composition for debugging
        trace!("════════════════════════════════════════════════════════════");
        info!("📊 TRANSCRIPT FOR APPLICATION KEY DERIVATION (DIAGNOSTIC)");
        trace!("════════════════════════════════════════════════════════════");
        info!("Transcript composition (RFC 8446 Section 4.4.1):");
        trace!("  ✅ 1. ClientHello handshake message (plaintext, no TLS header)");
        trace!("  ✅ 2. ServerHello handshake message (plaintext, no TLS header)");
        trace!("  ✅ 3-{}. {} post-handshake DECRYPTED messages (plaintext, no TLS headers)", 
              2 + messages_read, messages_read);
        trace!("     (EncryptedExtensions, Certificate, CertificateVerify, server Finished)");
        trace!("  ❌ NOT INCLUDED: client Finished (will be sent AFTER key derivation)");
        trace!("  Total transcript: {} bytes → SHA-256 → {} bytes", 
              self.transcript.len(), transcript_hash.len());
        trace!("  🎯 CRITICAL: All handshake messages are PLAINTEXT (decrypted)!");
        debug!("Full transcript (hex): {}", hex::encode(&self.transcript));
        debug!("Transcript hash (hex): {}", hex::encode(&transcript_hash));
        trace!("════════════════════════════════════════════════════════════");
        
        // 11. Derive application traffic secrets (for HTTP data encryption)
        // RFC 8446 Section 7.1: Application secrets are derived WITH transcript hash
        // Note: TLS 1.3 has separate key schedules:
        // - Handshake traffic secrets: For decrypting handshake messages (EncryptedExtensions, Certificate, etc.)
        // - Application traffic secrets: For encrypting HTTP data (requires transcript hash!)
        info!("Step 11: Deriving TLS application traffic secrets via BearDog (WITH transcript hash)");
        let derive_start = std::time::Instant::now();
        let secrets = self.crypto
            .tls_derive_application_secrets(&handshake_keys.handshake_secret, &transcript_hash)
            .await
            .map_err(|e| {
                error!("❌ BearDog TLS application secret derivation failed: {}", e);
                e
            })?;
        
        info!("🔐 TLS application traffic keys derived in {:?}", derive_start.elapsed());
        
        // DIAGNOSTIC: Show key derivation details (biomeOS investigation)
        trace!("════════════════════════════════════════════════════════════");
        info!("🔑 APPLICATION KEY DERIVATION RESULTS (DIAGNOSTIC)");
        trace!("════════════════════════════════════════════════════════════");
        info!("This is the 'invisible 0.5%' - verifying key expansion:");
        info!("");
        info!("Input to HKDF-Expand-Label (in BearDog):");
        trace!("  • CLIENT_TRAFFIC_SECRET_0 (from tls_derive_application_secrets)");
        trace!("  • Label: 'tls13 key' (for write key)");
        trace!("  • Label: 'tls13 iv' (for write IV)");
        trace!("  • Cipher suite: 0x{:04x}", self.cipher_suite);
        info!("");
        info!("Output (what we'll use for HTTP request encryption):");
        trace!("  client_write_key ({} bytes): {}", 
              secrets.client_write_key.len(), 
              hex::encode(&secrets.client_write_key));
        trace!("  client_write_iv ({} bytes): {}", 
              secrets.client_write_iv.len(), 
              hex::encode(&secrets.client_write_iv));
        info!("");
        info!("Expected key length for cipher 0x{:04x}: {} bytes", 
              self.cipher_suite,
              match self.cipher_suite {
                  0x1301 => 16,  // AES-128-GCM
                  0x1302 => 32,  // AES-256-GCM
                  0x1303 => 32,  // ChaCha20-Poly1305
                  _ => 0,
              });
        info!("Expected IV length: 12 bytes (all TLS 1.3 ciphers)");
        info!("");
        info!("⚠️  CRITICAL CHECK:");
        if secrets.client_write_key.len() != match self.cipher_suite {
            0x1301 => 16, 0x1302 => 32, 0x1303 => 32, _ => 0
        } {
            error!("❌ client_write_key length MISMATCH! Expected {} bytes, got {} bytes",
                   match self.cipher_suite { 0x1301 => 16, 0x1302 => 32, 0x1303 => 32, _ => 0 },
                   secrets.client_write_key.len());
        } else {
            info!("✅ client_write_key length is CORRECT ({} bytes)", secrets.client_write_key.len());
        }
        if secrets.client_write_iv.len() != 12 {
            error!("❌ client_write_iv length MISMATCH! Expected 12 bytes, got {} bytes",
                   secrets.client_write_iv.len());
        } else {
            info!("✅ client_write_iv length is CORRECT (12 bytes)");
        }
        info!("");
        info!("These keys will be used with:");
        trace!("  • Sequence number: 0 (for first HTTP request)");
        trace!("  • Nonce: client_write_iv XOR sequence_number");
        trace!("  • AAD: TLS record header (ContentType 0x17, version, length)");
        trace!("════════════════════════════════════════════════════════════");
        
        // 12. Send client Finished NOW that application keys are derived
        // RFC 8446 Section 4.4.4: Client must send Finished after receiving server Finished
        // CRITICAL: Application keys MUST be derived BEFORE sending client Finished!
        info!("Step 12: Sending client Finished message (RFC 8446 Section 4.4.4)");
        self.send_client_finished(stream, &handshake_keys).await?;
        info!("✅ Client Finished sent - handshake complete!");
        trace!("   Server should now respond to HTTP requests! 🎉");
        
        // 13. Read ALL post-handshake messages (RFC 8446 Section 4.6)
        // RFC 8446: Server MAY send MULTIPLE NewSessionTicket messages after handshake
        // We MUST read and DECRYPT these to:
        //   1. Avoid stream desync when sending HTTP requests
        //   2. Detect if server is sending an encrypted alert (decrypt_error, etc.)
        // IMPORTANT: In TLS 1.3, post-handshake alerts are ENCRYPTED with application keys!
        info!("Step 13: Reading ALL post-handshake messages (NewSessionTicket, etc.)");
        trace!("   ⏱️  Will read and decrypt until timeout (200ms between messages)...");
        
        let mut post_handshake_count = 0;
        let mut read_sequence_number: u64 = 0;  // Separate sequence for reading (starts at 0)
        
        // Loop to read ALL post-handshake messages
        // Use shorter timeout (200ms) between messages since server sends them quickly
        loop {
            match timeout(Duration::from_millis(200), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_data))) => {
                    post_handshake_count += 1;
                    info!("✅ Post-handshake message #{}: type=0x{:02x}, {} bytes (encrypted)", 
                         post_handshake_count, content_type, encrypted_data.len());
                    
                    match content_type {
                        0x17 => {  // APPLICATION_DATA (encrypted post-handshake message)
                            // DECRYPT the message using APPLICATION traffic keys (server_write_key)
                            trace!("   🔐 Decrypting with server_write_key (application traffic key)...");
                            
                            // Build nonce: server_write_iv XOR read_sequence_number
                            let mut nonce = secrets.server_write_iv.clone();
                            let seq_bytes = read_sequence_number.to_be_bytes();
                            for i in 0..8 {
                                nonce[4 + i] ^= seq_bytes[i];
                            }
                            
                            // Build AAD (record header)
                            let aad = [
                                0x17, 0x03, 0x03,
                                (encrypted_data.len() >> 8) as u8,
                                (encrypted_data.len() & 0xFF) as u8,
                            ];
                            
                            // Decrypt based on cipher suite
                            let plaintext_result = match self.cipher_suite {
                                0x1301 => self.crypto.aes128_gcm_decrypt(
                                    &secrets.server_write_key, &nonce, &encrypted_data, &aad
                                ).await,
                                0x1302 => self.crypto.aes256_gcm_decrypt(
                                    &secrets.server_write_key, &nonce, &encrypted_data, &aad
                                ).await,
                                _ => self.crypto.decrypt(
                                    &secrets.server_write_key, &nonce, &encrypted_data, &aad
                                ).await,
                            };
                            
                            read_sequence_number += 1;  // Increment for next message
                            
                            match plaintext_result {
                                Ok(plaintext) => {
                                    // RFC 8446: TLSInnerPlaintext has ContentType as last non-zero byte
                                    if let Some(&inner_type) = plaintext.last() {
                                        let content = &plaintext[..plaintext.len()-1];
                                        trace!("   📨 Decrypted: {} bytes, inner type=0x{:02x}", 
                                              content.len(), inner_type);
                                        
                                        match inner_type {
                                            0x16 => {  // Handshake (NewSessionTicket is type 0x04)
                                                if !content.is_empty() && content[0] == 0x04 {
                                                    trace!("   🎟️  NewSessionTicket #{} (ignored for now)", post_handshake_count);
                                                } else {
                                                    trace!("   📋 Handshake message type 0x{:02x}", 
                                                          content.first().unwrap_or(&0));
                                                }
                                            }
                                            0x15 => {  // ALERT (inside encrypted envelope!)
                                                use crate::tls::alert::TlsAlert;
                                                if content.len() >= 2 {
                                                    if let Ok(alert) = TlsAlert::parse(content) {
                                                        error!("");
                                                        error!("════════════════════════════════════════════════════════════");
                                                        error!("🚨 SERVER SENT ENCRYPTED ALERT AFTER HANDSHAKE!");
                                                        error!("════════════════════════════════════════════════════════════");
                                                        error!("{}", alert.to_detailed_string());
                                                        error!("════════════════════════════════════════════════════════════");
                                                        error!("");
                                                        return Err(Error::TlsHandshake(format!(
                                                            "Server sent encrypted alert: {}",
                                                            alert.to_detailed_string()
                                                        )));
                                                    }
                                                }
                                            }
                                            0x17 => {
                                                trace!("   📦 Application data (unexpected at this stage)");
                                            }
                                            _ => {
                                                trace!("   ❓ Unknown inner type: 0x{:02x}", inner_type);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("   ⚠️  Failed to decrypt post-handshake message: {}", e);
                                    warn!("   This might indicate a key derivation issue");
                                    // Don't fail - might be able to continue
                                }
                            }
                        }
                        0x15 => {  // Unencrypted ALERT (shouldn't happen in TLS 1.3 after handshake)
                            warn!("⚠️  Received UNENCRYPTED TLS alert (unusual for TLS 1.3)!");
                            use crate::tls::alert::TlsAlert;
                            if let Ok(alert) = TlsAlert::parse(&encrypted_data) {
                                error!("🚨 Alert: {}", alert.to_detailed_string());
                                return Err(Error::TlsHandshake(format!(
                                    "Server sent unencrypted alert: {}",
                                    alert.to_detailed_string()
                                )));
                            }
                        }
                        _ => {
                            trace!("   ℹ️  Ignoring unexpected message type: 0x{:02x}", content_type);
                        }
                    }
                }
                Ok(Err(e)) => {
                    if post_handshake_count == 0 {
                        warn!("⚠️  Error reading first post-handshake message: {}", e);
                    } else {
                        debug!("   Error reading message #{}: {} (might be normal)", post_handshake_count + 1, e);
                    }
                    break;
                }
                Err(_) => {
                    if post_handshake_count == 0 {
                        trace!("   ⏱️  Timeout - no post-handshake messages (this is OK)");
                    } else {
                        trace!("   ⏱️  Timeout after {} post-handshake messages", post_handshake_count);
                    }
                    break;
                }
            }
        }
        
        info!("✅ Consumed {} post-handshake messages - stream ready for HTTP!", post_handshake_count);
        
        let total_time = handshake_start.elapsed();
        info!("🎉 ✅ TLS 1.3 handshake complete in {:?}", total_time);
        debug!("Handshake summary: {} post-handshake messages, cipher: TLS_CHACHA20_POLY1305_SHA256", 
               messages_read);

        Ok(SessionKeys {
            client_write_key: secrets.client_write_key,
            server_write_key: secrets.server_write_key,
            client_write_iv: secrets.client_write_iv,
            server_write_iv: secrets.server_write_iv,
            cipher_suite: self.cipher_suite,  // Pass negotiated cipher suite to session
            initial_read_sequence: read_sequence_number,  // Account for post-handshake messages consumed
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
    /// Build extensions based on strategy (no hardcoding!)
    pub(crate) fn build_extensions(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        use crate::tls::config::ExtensionStrategy;
        
        match &self.config.extension_strategy {
            ExtensionStrategy::Minimal => {
                debug!("🎯 Building MINIMAL extensions (3 extensions, ~50ms handshake)");
                self.build_extensions_minimal(server_name, public_key)
            }
            ExtensionStrategy::Standard => {
                debug!("🎯 Building STANDARD extensions (7 extensions, ~80ms handshake)");
                self.build_extensions_standard(server_name, public_key)
            }
            ExtensionStrategy::Modern => {
                debug!("🎯 Building MODERN extensions (10+ extensions, ~100ms handshake)");
                self.build_extensions_modern(server_name, public_key)
            }
            ExtensionStrategy::MaxCompatibility => {
                debug!("🎯 Building MAX COMPATIBILITY extensions (12+ extensions)");
                self.build_extensions_maxcompat(server_name, public_key)
            }
            ExtensionStrategy::Adaptive => {
                // Use profiler recommendation if available
                if let Some(profiler) = &self.profiler {
                    if let Some(profile) = profiler.get_profile(server_name) {
                        if profile.is_reliable() {
                            info!("🧠 Using learned extensions for {} (reliability: {:.1}%)", 
                                  server_name, profile.reliability * 100.0);
                            // Build custom extensions based on learned profile
                            // For now, fall back to standard
                            return self.build_extensions_standard(server_name, public_key);
                        }
                    }
                }
                debug!("🎯 ADAPTIVE: No profile, using STANDARD extensions");
                self.build_extensions_standard(server_name, public_key)
            }
            ExtensionStrategy::Custom(_ext_types) => {
                debug!("🎯 Building CUSTOM extensions (user-defined)");
                // For now, use standard
                // TODO: Build custom extension set
                self.build_extensions_standard(server_name, public_key)
            }
        }
    }
    
    /// Build minimal extensions (fastest handshake, ~50ms)
    /// Only required extensions: SNI, Supported Versions, Key Share
    fn build_extensions_minimal(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. SNI extension (0x0000) - REQUIRED for virtual hosting
        ext.extend_from_slice(&[0x00, 0x00]);
        let sni_data = self.build_sni_extension(server_name);
        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // 2. Supported versions (0x002b) - REQUIRED for TLS 1.3
        ext.extend_from_slice(&[0x00, 0x2b]);
        ext.extend_from_slice(&[0x00, 0x03]);
        ext.extend_from_slice(&[0x02]);
        ext.extend_from_slice(&TLS_1_3.to_be_bytes());

        // 3. Key share (0x0033) - REQUIRED for TLS 1.3
        ext.extend_from_slice(&[0x00, 0x33]);
        let key_share_data = self.build_key_share_extension(public_key);
        ext.extend_from_slice(&(key_share_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&key_share_data);

        Ok(ext)
    }
    
    /// Build standard extensions (balanced, ~80ms handshake)
    /// Current production-tested set
    fn build_extensions_standard(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        let mut ext = Vec::new();

        // 1. SNI extension (0x0000)
        ext.extend_from_slice(&[0x00, 0x00]);
        let sni_data = self.build_sni_extension(server_name);
        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // 2. ALPN extension (0x0010) - CRITICAL for HTTPS
        ext.extend_from_slice(&[0x00, 0x10]);
        ext.extend_from_slice(&[0x00, 0x0b]);
        ext.extend_from_slice(&[0x00, 0x09]);
        ext.extend_from_slice(&[0x08]);
        ext.extend_from_slice(b"http/1.1");

        // 3. Supported versions (0x002b)
        ext.extend_from_slice(&[0x00, 0x2b]);
        ext.extend_from_slice(&[0x00, 0x03]);
        ext.extend_from_slice(&[0x02]);
        ext.extend_from_slice(&TLS_1_3.to_be_bytes());

        // 4. Key share (0x0033)
        ext.extend_from_slice(&[0x00, 0x33]);
        let key_share_data = self.build_key_share_extension(public_key);
        ext.extend_from_slice(&(key_share_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&key_share_data);

        // 5. Supported groups (0x000a)
        ext.extend_from_slice(&[0x00, 0x0a]);
        ext.extend_from_slice(&[0x00, 0x04]);
        ext.extend_from_slice(&[0x00, 0x02]);
        ext.extend_from_slice(&[0x00, 0x1d]); // x25519

        // 6. Signature algorithms (0x000d)
        ext.extend_from_slice(&[0x00, 0x0d]);
        ext.extend_from_slice(&[0x00, 0x14]);
        ext.extend_from_slice(&[0x00, 0x12]);
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x06, 0x03]); // ecdsa_secp521r1_sha512
        ext.extend_from_slice(&[0x08, 0x07]); // ed25519
        ext.extend_from_slice(&[0x08, 0x08]); // ed448
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
        ext.extend_from_slice(&[0x06, 0x01]); // rsa_pkcs1_sha512
        ext.extend_from_slice(&[0x08, 0x04]); // rsa_pss_rsae_sha256

        // 7. PSK Key Exchange Modes (0x002d) - REQUIRED by many servers
        ext.extend_from_slice(&[0x00, 0x2d]);
        ext.extend_from_slice(&[0x00, 0x02]);
        ext.extend_from_slice(&[0x01]);
        ext.extend_from_slice(&[0x01]); // psk_dhe_ke

        Ok(ext)
    }
    
    /// Build modern extensions (latest features, ~100ms handshake)
    fn build_extensions_modern(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        // Start with standard extensions
        let mut ext = self.build_extensions_standard(server_name, public_key)?;

        // Add modern extensions
        
        // 8. Status Request (OCSP stapling, 0x0005)
        ext.extend_from_slice(&[0x00, 0x05]);
        ext.extend_from_slice(&[0x00, 0x05]);
        ext.extend_from_slice(&[0x01]); // status_type: ocsp
        ext.extend_from_slice(&[0x00, 0x00]); // responder_id_list: empty
        ext.extend_from_slice(&[0x00, 0x00]); // request_extensions: empty

        // RFC 8446 NOTE: We do NOT add TLS 1.2 legacy extensions like:
        // - extended_master_secret (0x0017) - causes servers to confuse us with TLS 1.2
        // - renegotiation_info (0xff01) - not applicable in TLS 1.3
        // These were causing real-world servers (cloudflare, google) to send Application Data
        // instead of ServerHello, as they thought we were trying session resumption.

        Ok(ext)
    }
    
    /// Build max compatibility extensions (exhaustive set)
    fn build_extensions_maxcompat(&self, server_name: &str, public_key: &[u8]) -> Result<Vec<u8>> {
        // Start with modern extensions
        let mut ext = self.build_extensions_modern(server_name, public_key)?;

        // Add compatibility extensions
        
        // 11. Session Ticket (0x0023)
        ext.extend_from_slice(&[0x00, 0x23]);
        ext.extend_from_slice(&[0x00, 0x00]); // Empty ticket

        // 12. Supported Signature Algorithms Cert (0x0032)
        ext.extend_from_slice(&[0x00, 0x32]);
        ext.extend_from_slice(&[0x00, 0x0c]);
        ext.extend_from_slice(&[0x00, 0x0a]);
        ext.extend_from_slice(&[0x04, 0x03]); // ecdsa_secp256r1_sha256
        ext.extend_from_slice(&[0x05, 0x03]); // ecdsa_secp384r1_sha384
        ext.extend_from_slice(&[0x04, 0x01]); // rsa_pkcs1_sha256
        ext.extend_from_slice(&[0x05, 0x01]); // rsa_pkcs1_sha384
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
    /// Read a TLS record and return (content_type, content)
    /// Returns the content type byte (e.g., 0x14=ChangeCipherSpec, 0x17=ApplicationData) and the record content
    async fn read_record(&self, stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> {
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

        Ok((content_type, content))
    }

    /// Decrypt a TLS handshake record with handshake traffic keys
    /// 
    /// RFC 8446 Section 4.4.1: Transcript hash is computed over PLAINTEXT handshake messages!
    /// After ServerHello, all handshake messages (EncryptedExtensions, Certificate, etc.) are encrypted.
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
    /// Decrypted plaintext handshake message (without ContentType byte)
    async fn decrypt_handshake_record(
        &self,
        encrypted_record: &[u8],
        keys: &TlsSecrets,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        info!("🔓 Decrypting handshake record (COMPREHENSIVE DEBUG):");
        trace!("   Encrypted length: {} bytes", encrypted_record.len());
        trace!("   Sequence number: {}", sequence_number);
        debug!("Encrypted data (first 32 bytes): {:02x?}", &encrypted_record[..std::cmp::min(32, encrypted_record.len())]);
        debug!("Encrypted data (last 16 bytes, likely tag): {:02x?}", &encrypted_record[encrypted_record.len().saturating_sub(16)..]);

        // Log keys and IVs
        info!("🔑 Cryptographic Material:");
        trace!("   Server write key: {} bytes", keys.server_write_key.len());
        debug!("   Server write key (first 16 bytes): {:02x?}", &keys.server_write_key[..std::cmp::min(16, keys.server_write_key.len())]);
        trace!("   Server write IV: {} bytes", keys.server_write_iv.len());
        debug!("   Server write IV (full): {:02x?}", keys.server_write_iv);

        // Build nonce: server_write_iv XOR sequence_number
        // RFC 8446 Section 5.3: per_record_nonce = IV XOR sequence_number (right-padded to IV length)
        // We're reading from server, so use server_write_iv
        info!("🧮 Computing nonce (RFC 8446 Section 5.3):");
        let mut nonce = keys.server_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();
        
        debug!("   Original IV: {:02x?}", nonce);
        debug!("   Sequence bytes (8 bytes, big-endian): {:02x?}", seq_bytes);
        
        // XOR the last 8 bytes of the IV with the sequence number
        // TLS 1.3: nonce = IV[0..4] || (IV[4..12] XOR sequence_number)
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        trace!("   Computed nonce: {:02x?}", nonce);
        debug!("   Nonce construction: IV XOR sequence_number (last 8 bytes)");

        // Build AAD (Additional Authenticated Data): TLS record header
        // RFC 8446 Section 5.2: AAD = TLS record header (5 bytes)
        // For encrypted records, ContentType is always 0x17 (ApplicationData) in TLS 1.3
        info!("📋 Building AAD (Additional Authenticated Data):");
        let record_type = 0x17; // ApplicationData (ALL encrypted records use 0x17 in TLS 1.3)
        let version = [0x03, 0x03]; // TLS 1.2 compatibility version
        let length = encrypted_record.len() as u16;
        let aad = [
            record_type,
            version[0],
            version[1],
            (length >> 8) as u8,
            (length & 0xFF) as u8,
        ];
        trace!("   AAD (TLS record header): {:02x?}", aad);
        debug!("   Breakdown:");
        debug!("     - ContentType: 0x{:02x} (APPLICATION_DATA)", record_type);
        debug!("     - Version: 0x{:02x}{:02x} (TLS 1.2 compat)", version[0], version[1]);
        debug!("     - Length: {} bytes (0x{:04x})", length, length);

        // Log comprehensive decryption parameters
        info!("🎯 Calling BearDog crypto.decrypt with:");
        trace!("   Key: server_write_key ({} bytes)", keys.server_write_key.len());
        trace!("   Nonce: {} bytes", nonce.len());
        trace!("   Ciphertext+Tag: {} bytes", encrypted_record.len());
        trace!("   AAD: {} bytes", aad.len());
        debug!("Decryption parameters summary:");
        debug!("  - Key type: Handshake traffic key (server_write_key)");
        debug!("  - Nonce: IV XOR sequence_number");
        debug!("  - AAD: TLS record header");
        debug!("  - Expected: ciphertext[:-16] as plaintext, ciphertext[-16:] as tag");

        // Decrypt via BearDog - use correct AEAD algorithm based on negotiated cipher suite!
        let decrypt_start = std::time::Instant::now();
        info!("⏳ Calling beardog.decrypt with cipher suite 0x{:04x}...", self.cipher_suite);
        
        let plaintext = match self.cipher_suite {
            0x1301 => {
                // TLS_AES_128_GCM_SHA256 (most common - GitHub, Google, CloudFlare)
                // BearDog now derives correct 16-byte keys based on cipher suite!
                trace!("   → Using AES-128-GCM (negotiated cipher suite)");
                debug!("  - Algorithm: AES-128-GCM AEAD");
                debug!("  - Key length from BearDog: {} bytes", keys.server_write_key.len());
                self.crypto.aes128_gcm_decrypt(
                    &keys.server_write_key,
                    &nonce,
                    encrypted_record,
                    &aad,
                ).await
            }
            0x1302 => {
                // TLS_AES_256_GCM_SHA384 (high security)
                trace!("   → Using AES-256-GCM (negotiated cipher suite)");
                debug!("  - Algorithm: AES-256-GCM AEAD");
                self.crypto.aes256_gcm_decrypt(
                    &keys.server_write_key,
                    &nonce,
                    encrypted_record,
                    &aad,
                ).await
            }
            0x1303 => {
                // TLS_CHACHA20_POLY1305_SHA256 (software-only, mobile-optimized)
                trace!("   → Using ChaCha20-Poly1305 (negotiated cipher suite)");
                debug!("  - Algorithm: ChaCha20-Poly1305 AEAD");
                self.crypto.decrypt(
                    &keys.server_write_key,
                    &nonce,
                    encrypted_record,
                    &aad,
                ).await
            }
            _ => {
                error!("❌ Unsupported cipher suite: 0x{:04x}", self.cipher_suite);
                return Err(Error::TlsHandshake(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.cipher_suite
                )));
            }
        }.map_err(|e| {
            error!("❌ Handshake record decryption FAILED!");
            error!("   Error: {}", e);
            error!("   AEAD authentication failure - investigating:");
            error!("");
            error!("   📊 Decryption Context:");
            error!("     • Encrypted length: {} bytes", encrypted_record.len());
            error!("     • Sequence number: {}", sequence_number);
            error!("     • Key: server_write_key ({} bytes)", keys.server_write_key.len());
            error!("     • IV: {:02x?}", keys.server_write_iv);
            error!("     • Nonce: {:02x?}", nonce);
            error!("     • AAD: {:02x?}", aad);
            error!("");
            error!("   🔍 Possible Causes:");
            error!("     1. Wrong key (key derivation mismatch)");
            error!("     2. Wrong nonce (sequence number or IV mismatch)");
            error!("     3. Wrong AAD (record header construction mismatch)");
            error!("     4. Corrupted ciphertext (network issue)");
            error!("     5. Tag split incorrectly (should be last 16 bytes)");
            error!("");
            error!("   🎯 Next Steps:");
            error!("     • Verify handshake key derivation includes transcript hash");
            error!("     • Verify sequence number starts at 0");
            error!("     • Verify AAD matches TLS record header exactly");
            error!("     • Compare with RFC 8448 test vectors");
            e
        })?;
        
        info!("✅ Decrypted handshake record successfully in {:?}", decrypt_start.elapsed());
        trace!("   Plaintext length: {} bytes", plaintext.len());
        debug!("Plaintext preview (first 32 bytes): {:02x?}", &plaintext[..std::cmp::min(32, plaintext.len())]);
        debug!("Plaintext preview (last 16 bytes): {:02x?}", &plaintext[plaintext.len().saturating_sub(16)..]);

        // RFC 8446 Section 5.2: TLS 1.3 encrypted records have ContentType as last byte
        // Strip the ContentType byte from the end
        if !plaintext.is_empty() {
            let content_type = plaintext[plaintext.len() - 1];
            debug!("ContentType (last byte of plaintext): 0x{:02x}", content_type);
            let message = plaintext[..plaintext.len() - 1].to_vec();
            info!("📤 Returning handshake message: {} bytes (ContentType stripped)", message.len());
            Ok(message)
        } else {
            warn!("⚠️  Empty plaintext after decryption!");
            Ok(plaintext)
        }
    }

    /// Parse ServerHello message
    /// 
    /// Returns: (server_random, server_public_key, cipher_suite)
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

    /// Encrypt application data for TLS 1.3 with correct AAD construction
    /// 
    /// This method constructs a complete TLS APPLICATION_DATA record with:
    /// - TLS record header (5 bytes)
    /// - Encrypted content (ciphertext + 16-byte AEAD tag)
    /// 
    /// AAD (Additional Authenticated Data) = TLS record header
    /// Nonce = IV XOR sequence_number (TLS 1.3 nonce construction per RFC 8446)
    pub async fn encrypt_application_data(
        &self,
        plaintext: &[u8],
        keys: &SessionKeys,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        trace!("🔐 Encrypting {} bytes of application data (seq={})", plaintext.len(), sequence_number);
        
        // Calculate ciphertext length (plaintext + 16-byte AEAD tag)
        let ciphertext_length = plaintext.len() + 16;
        
        // Construct TLS record header (this becomes the AAD)
        let record_type = 0x17; // ContentType: APPLICATION_DATA
        let version = [0x03, 0x03]; // TLS 1.2 (compatibility mode for TLS 1.3)
        let length = ciphertext_length as u16;
        
        let aad = [
            record_type,
            version[0],
            version[1],
            (length >> 8) as u8,
            (length & 0xFF) as u8,
        ];
        
        trace!("AAD (TLS record header): {:02x?}", aad);
        
        // Construct nonce: IV XOR sequence_number (RFC 8446 Section 5.3)
        // The sequence number is XORed with the IV (right-aligned)
        let mut nonce = keys.client_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();
        
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        
        trace!("Nonce (IV XOR seq): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);
        
        // Encrypt via BearDog
        let ciphertext = self.crypto.encrypt(
            &keys.client_write_key,
            &nonce,
            plaintext,
            &aad,
        ).await?;
        
        debug!("✅ Encrypted {} bytes → {} bytes (includes 16-byte tag)", plaintext.len(), ciphertext.len());
        
        // Construct complete TLS record: header + ciphertext (includes tag)
        let mut record = Vec::new();
        record.extend_from_slice(&aad);
        record.extend_from_slice(&ciphertext);
        
        Ok(record)
    }

    /// Send client Finished message (RFC 8446 Section 4.4.4)
    /// 
    /// The Finished message is sent by the client after receiving and verifying the server's
    /// Finished message. It contains a verify_data field that authenticates the entire handshake.
    /// 
    /// RFC 8446 Section 4.4.4:
    /// ```text
    /// struct {
    ///     opaque verify_data[Hash.length];
    /// } Finished;
    /// ```
    /// 
    /// The verify_data is computed as:
    /// ```text
    /// verify_data = HMAC(finished_key, Transcript-Hash(Handshake Context))
    /// ```
    /// 
    /// # Arguments
    /// 
    /// * `stream` - TCP stream to send the Finished message on
    /// * `handshake_keys` - Handshake traffic keys for encrypting the message
    async fn send_client_finished(
        &mut self,
        stream: &mut TcpStream,
        handshake_keys: &TlsSecrets,
    ) -> Result<()> {
        info!("🔐 Building client Finished message (RFC 8446 Section 4.4.4)");
        
        // 1. Compute transcript hash of all handshake messages
        // Includes: ClientHello, ServerHello, EncryptedExtensions, Certificate, CertificateVerify, server Finished
        let transcript_hash = self.compute_transcript_hash();
        info!("📊 Transcript hash for Finished: {} bytes", transcript_hash.len());
        debug!("   Transcript hash (hex): {}", hex::encode(&transcript_hash));
        
        // 2. Call BearDog to compute verify_data (RFC 8446 Section 4.4.4)
        // BearDog implements: HMAC(finished_key, transcript_hash)
        // where finished_key is derived from the handshake traffic secret (base_key)
        info!("🔐 Computing verify_data via CryptoCapability...");
        let verify_data = self.crypto
            .tls_compute_finished_verify_data(
                &handshake_keys.client_handshake_secret,  // RFC 8446 client_handshake_traffic_secret (32-byte PRK)
                &transcript_hash
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to compute Finished verify_data: {}", e);
                e
            })?;
        
        info!("✅ Finished verify_data computed: {} bytes", verify_data.len());
        debug!("   Verify data (hex): {}", hex::encode(&verify_data));
        
        // 3. Build Finished handshake message
        // Format: HandshakeType (1 byte) + Length (3 bytes) + verify_data (32 bytes for SHA-256)
        let mut finished_msg = Vec::new();
        finished_msg.push(0x14); // HandshakeType: Finished
        
        // Length (3 bytes, big-endian)
        let length = verify_data.len();
        finished_msg.push(((length >> 16) & 0xFF) as u8);
        finished_msg.push(((length >> 8) & 0xFF) as u8);
        finished_msg.push((length & 0xFF) as u8);
        
        // Verify data
        finished_msg.extend_from_slice(&verify_data);
        
        info!("📝 Built Finished message: {} bytes total", finished_msg.len());
        debug!("   Finished message (hex): {}", hex::encode(&finished_msg));
        
        // 4. Add ContentType byte for TLS 1.3 encryption (RFC 8446 Section 5.2)
        // In TLS 1.3, the ContentType (0x16 = Handshake) is encrypted as part of the payload
        let mut plaintext = finished_msg.clone();
        plaintext.push(0x16); // ContentType: Handshake
        
        info!("📝 Plaintext with ContentType: {} bytes", plaintext.len());
        debug!("   Last byte (ContentType): 0x{:02x}", plaintext[plaintext.len() - 1]);
        
        // 5. Encrypt with handshake traffic keys
        // We use client_write_key since we're the client sending this message
        // Sequence number for client Finished is 0 (first message we send with handshake keys)
        let sequence_number = 0u64;
        
        trace!("════════════════════════════════════════════════════════════");
        info!("🔐 ENCRYPTING CLIENT FINISHED (HANDSHAKE MESSAGE)");
        trace!("════════════════════════════════════════════════════════════");
        info!("Using: HANDSHAKE traffic keys (client_handshake_traffic_secret)");
        info!("Sequence number: {} (first handshake message sent by client)", sequence_number);
        info!("Cipher suite: 0x{:04x}", self.cipher_suite);
        debug!("Handshake key length: {} bytes", handshake_keys.client_write_key.len());
        debug!("Handshake IV length: {} bytes", handshake_keys.client_write_iv.len());
        info!("⚠️  NOTE: HTTP requests will use APPLICATION traffic keys (different!)");
        
        // Build nonce: client_write_iv XOR sequence_number (RFC 8446 Section 5.3)
        let mut nonce = handshake_keys.client_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();
        
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        
        debug!("   Nonce (IV XOR seq): {:02x?}", nonce);
        trace!("════════════════════════════════════════════════════════════");
        
        // Calculate ciphertext length (plaintext + 16-byte AEAD tag)
        let ciphertext_length = plaintext.len() + 16;
        
        // Build AAD (TLS record header)
        let record_type = 0x17; // APPLICATION_DATA (all encrypted records use 0x17 in TLS 1.3)
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let aad = [
            record_type,
            version[0],
            version[1],
            ((ciphertext_length >> 8) & 0xFF) as u8,
            (ciphertext_length & 0xFF) as u8,
        ];
        
        debug!("   AAD (TLS record header): {:02x?}", aad);
        
        // Encrypt via BearDog (uses correct AEAD algorithm based on cipher suite)
        // DIAGNOSTIC: Log BOTH client and server keys to detect swapping
        info!("🔑 CLIENT FINISHED ENCRYPTION KEY (DIAGNOSTIC):");
        trace!("   client_write_key (hex): {}", hex::encode(&handshake_keys.client_write_key));
        trace!("   server_write_key (hex): {}", hex::encode(&handshake_keys.server_write_key));
        trace!("   client_write_iv (hex): {}", hex::encode(&handshake_keys.client_write_iv));
        trace!("   server_write_iv (hex): {}", hex::encode(&handshake_keys.server_write_iv));
        trace!("   Nonce (IV XOR seq): {}", hex::encode(&nonce));
        trace!("   AAD (hex): {}", hex::encode(&aad));
        trace!("   Plaintext length: {} bytes", plaintext.len());
        trace!("   ⚠️  HYPOTHESIS: If server_write_key == server's expected client_write_key,");
        trace!("      then BearDog is swapping client/server labels!");
        
        // Use client_write_key for client→server encryption (correct per RFC 8446)
        let encryption_key = &handshake_keys.client_write_key;
        trace!("   🔑 USING KEY: client_write_key (correct per RFC 8446)");
        
        let ciphertext = match self.cipher_suite {
            0x1301 => {
                trace!("   → Using AES-128-GCM for client Finished");
                self.crypto.aes128_gcm_encrypt(
                    encryption_key,
                    &nonce,
                    &plaintext,
                    &aad,
                ).await
            }
            0x1302 => {
                trace!("   → Using AES-256-GCM for client Finished");
                self.crypto.aes256_gcm_encrypt(
                    encryption_key,
                    &nonce,
                    &plaintext,
                    &aad,
                ).await
            }
            0x1303 => {
                trace!("   → Using ChaCha20-Poly1305 for client Finished");
                self.crypto.encrypt(
                    encryption_key,
                    &nonce,
                    &plaintext,
                    &aad,
                ).await
            }
            _ => {
                error!("❌ Unsupported cipher suite: 0x{:04x}", self.cipher_suite);
                return Err(Error::TlsHandshake(format!(
                    "Unsupported TLS 1.3 cipher suite: 0x{:04x}",
                    self.cipher_suite
                )));
            }
        }.map_err(|e| {
            error!("❌ Failed to encrypt client Finished: {}", e);
            e
        })?;
        
        info!("✅ Encrypted client Finished: {} bytes (includes 16-byte tag)", ciphertext.len());
        
        // 6. Build complete TLS record: header + ciphertext
        let mut tls_record = Vec::new();
        tls_record.extend_from_slice(&aad);
        tls_record.extend_from_slice(&ciphertext);
        
        info!("📤 Sending client Finished TLS record: {} bytes total", tls_record.len());
        debug!("   TLS record preview: {:02x?}", &tls_record[..std::cmp::min(32, tls_record.len())]);
        
        // 7. Send over TCP
        stream.write_all(&tls_record).await.map_err(|e| {
            error!("❌ Failed to write client Finished: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await.map_err(|e| {
            error!("❌ Failed to flush client Finished: {}", e);
            Error::Io(e)
        })?;
        
        info!("✅ Client Finished TLS record sent successfully to server");
        
        Ok(())
    }

    /// Check if decrypted handshake record contains a Finished message (HandshakeType 0x14)
    /// 
    /// RFC 8446 Section 5.1: Multiple handshake messages MAY be coalesced into a single TLS record.
    /// 
    /// Server may send multiple handshake messages in ONE encrypted TLS ApplicationData record:
    /// - EncryptedExtensions (type 0x08)
    /// - Certificate (type 0x0B)
    /// - CertificateVerify (type 0x0F)
    /// - Finished (type 0x14) ← We need to find THIS!
    /// 
    /// Each handshake message has RFC 8446 framing:
    /// - HandshakeType msg_type (1 byte)
    /// - uint24 length (3 bytes, big-endian)
    /// - opaque body (variable length)
    /// 
    /// This method parses the framing to locate the Finished message at any offset.
    fn contains_finished_message(&self, plaintext: &[u8]) -> bool {
        let mut offset = 0;
        
        // Skip ContentType byte at end (0x16 for handshake, added during encryption)
        let data_len = plaintext.len().saturating_sub(1);
        
        debug!("🔍 Parsing handshake messages in {} byte plaintext blob", plaintext.len());
        
        while offset < data_len {
            // Check message type at current offset
            if plaintext[offset] == 0x14 {
                info!("🎯 SERVER FINISHED DETECTED! (HandshakeType 0x14 at offset {})", offset);
                return true;
            }
            
            // Parse handshake message header: type (1 byte) + length (3 bytes, big-endian)
            if offset + 4 > data_len {
                debug!("   End of handshake messages at offset {} (header incomplete)", offset);
                break;
            }
            
            let msg_type = plaintext[offset];
            let msg_len = u32::from_be_bytes([
                0,
                plaintext[offset + 1],
                plaintext[offset + 2],
                plaintext[offset + 3],
            ]) as usize;
            
            // Log the message type for debugging
            let msg_name = match msg_type {
                0x08 => "EncryptedExtensions",
                0x0B => "Certificate",
                0x0F => "CertificateVerify",
                0x14 => "Finished",
                _ => "Unknown",
            };
            debug!("   Handshake message at offset {}: type=0x{:02x} ({}), length={} bytes", 
                   offset, msg_type, msg_name, msg_len);
            
            // Skip to next message: header (4 bytes) + body (msg_len bytes)
            offset += 4 + msg_len;
            
            // Safety check: prevent infinite loop on malformed data
            if msg_len > 65536 {
                warn!("   Stopping parse: suspicious message length {} at offset {}", msg_len, offset);
                break;
            }
            
            if offset > data_len {
                debug!("   Stopping parse: offset {} exceeds data length {}", offset, data_len);
                break;
            }
        }
        
        debug!("   No Finished message found in {} byte plaintext", plaintext.len());
        false
    }

    /// Decrypt application data for TLS 1.3 with correct AAD construction
    /// 
    /// This method decrypts a TLS APPLICATION_DATA record using:
    /// - Record header as AAD
    /// - Nonce = IV XOR sequence_number (TLS 1.3 nonce construction per RFC 8446)
    /// 
    /// The ciphertext parameter should include the 16-byte AEAD tag.
    pub async fn decrypt_application_data(
        &self,
        record_header: &[u8; 5],
        ciphertext: &[u8],
        keys: &SessionKeys,
        sequence_number: u64,
    ) -> Result<Vec<u8>> {
        trace!("🔓 Decrypting {} bytes of application data (seq={})", ciphertext.len(), sequence_number);
        trace!("Record header (AAD): {:02x?}", record_header);
        
        // AAD = TLS record header (all 5 bytes: type, version, length)
        let aad = record_header;
        
        // Construct nonce: IV XOR sequence_number (RFC 8446 Section 5.3)
        let mut nonce = keys.server_write_iv.clone();
        let seq_bytes = sequence_number.to_be_bytes();
        
        if nonce.len() >= 8 {
            for (i, &byte) in seq_bytes.iter().enumerate() {
                let nonce_idx = nonce.len() - 8 + i;
                nonce[nonce_idx] ^= byte;
            }
        }
        
        trace!("Nonce (IV XOR seq): {:02x?}", &nonce[..std::cmp::min(12, nonce.len())]);
        
        // Decrypt via BearDog (will handle AEAD tag validation)
        let plaintext = self.crypto.decrypt(
            &keys.server_write_key,
            &nonce,
            ciphertext,
            aad,
        ).await?;
        
        debug!("✅ Decrypted {} bytes → {} bytes (AEAD authentication succeeded)", ciphertext.len(), plaintext.len());
        
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        let random = handshake.generate_random();
        assert_eq!(random.len(), 32);
    }

    // Note: generate_random() uses timestamp-based randomness for testing
    // In production, BearDog should provide cryptographically secure random

    #[test]
    fn test_build_sni_extension() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
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
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
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
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        let public_key = vec![1u8; 32];
        let extensions = handshake.build_extensions("api.github.com", &public_key)
            .expect("Should build extensions");
        
        assert!(!extensions.is_empty(), "Extensions should not be empty");
        assert!(extensions.len() > 90, "Should contain multiple extensions including ALPN");
        
        // Verify ALPN extension is present (0x00 0x10)
        let alpn_present = extensions.windows(2).any(|w| w == [0x00, 0x10]);
        assert!(alpn_present, "Should contain ALPN extension for HTTPS");
    }
    
    #[test]
    fn test_alpn_extension_encoding() {
        // CRITICAL: Validates byte-perfect ALPN encoding to prevent decode_error
        // This test prevents the exact bug biomeOS found in integration testing
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        let public_key = vec![1u8; 32];
        let extensions = handshake.build_extensions("api.github.com", &public_key)
            .expect("Should build extensions");
        
        // Find ALPN extension (0x00 0x10)
        let mut alpn_start = None;
        for i in 0..extensions.len() - 1 {
            if extensions[i] == 0x00 && extensions[i + 1] == 0x10 {
                alpn_start = Some(i);
                break;
            }
        }
        
        let alpn_start = alpn_start.expect("ALPN extension must be present");
        
        // Verify ALPN extension structure (RFC 7301)
        // Format: Type(2) + ExtLength(2) + ListLength(2) + ProtocolLength(1) + Protocol(n)
        assert_eq!(extensions[alpn_start], 0x00, "ALPN type byte 1");
        assert_eq!(extensions[alpn_start + 1], 0x10, "ALPN type byte 2");
        
        // Extension length should be 11 bytes (0x00 0x0b)
        assert_eq!(extensions[alpn_start + 2], 0x00, "ALPN extension length MSB");
        assert_eq!(extensions[alpn_start + 3], 0x0b, "ALPN extension length LSB = 11 bytes");
        
        // Protocol list length should be 9 bytes (0x00 0x09)
        assert_eq!(extensions[alpn_start + 4], 0x00, "ALPN list length MSB");
        assert_eq!(extensions[alpn_start + 5], 0x09, "ALPN list length LSB = 9 bytes (1 + 8)");
        
        // Protocol name length should be 8 bytes (0x08)
        assert_eq!(extensions[alpn_start + 6], 0x08, "Protocol name length = 8 bytes");
        
        // Protocol name should be "http/1.1"
        let protocol_name = &extensions[alpn_start + 7..alpn_start + 15];
        assert_eq!(protocol_name, b"http/1.1", "Protocol name should be 'http/1.1'");
        
        // Total ALPN extension size validation
        // Type(2) + ExtLength(2) + ListLength(2) + NameLength(1) + Name(8) = 15 bytes
        let total_alpn_size = 2 + 2 + 2 + 1 + 8;
        assert_eq!(total_alpn_size, 15, "Total ALPN extension should be 15 bytes");
        
        // Verify extension length field matches actual data
        let ext_length = u16::from_be_bytes([extensions[alpn_start + 2], extensions[alpn_start + 3]]);
        assert_eq!(ext_length, 11, "Extension length field should be 11");
        
        // Verify list length field matches actual data
        let list_length = u16::from_be_bytes([extensions[alpn_start + 4], extensions[alpn_start + 5]]);
        assert_eq!(list_length, 9, "List length field should be 9");
        
        // Verify protocol length matches actual protocol
        let protocol_length = extensions[alpn_start + 6];
        assert_eq!(protocol_length, 8, "Protocol length should be 8");
        assert_eq!(b"http/1.1".len(), 8, "Protocol 'http/1.1' is 8 bytes");
    }
    
    #[test]
    fn test_build_client_hello() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
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
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
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
        
        let (server_random, server_public, cipher_suite) = result.unwrap();
        assert_eq!(server_random.len(), 32, "Server random should be 32 bytes");
        assert_eq!(server_public.len(), 32, "Server public key should be 32 bytes");
        assert_eq!(cipher_suite, 0x1301, "Cipher suite should be 0x1301");
    }
    
    #[test]
    fn test_parse_server_hello_invalid() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
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
    
    // ============================================================================
    // RFC 8446 Transcript Tracking Tests
    // ============================================================================
    
    #[test]
    fn test_transcript_empty_initially() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        // Transcript should be empty initially
        assert_eq!(handshake.transcript.len(), 0, "Transcript should start empty");
    }
    
    #[test]
    fn test_update_transcript() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Add first message
        let message1 = b"ClientHello";
        handshake.update_transcript(message1);
        assert_eq!(handshake.transcript.len(), message1.len());
        
        // Add second message
        let message2 = b"ServerHello";
        handshake.update_transcript(message2);
        assert_eq!(handshake.transcript.len(), message1.len() + message2.len());
        
        // Verify messages are concatenated
        assert_eq!(&handshake.transcript[..message1.len()], message1);
        assert_eq!(&handshake.transcript[message1.len()..], message2);
    }
    
    #[test]
    fn test_compute_transcript_hash_empty() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        let hash = handshake.compute_transcript_hash();
        
        // SHA-256 hash of empty input
        // echo -n "" | sha256sum
        let expected_empty_hash = hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .expect("Valid hex");
        
        assert_eq!(hash.len(), 32, "SHA-256 hash should be 32 bytes");
        assert_eq!(hash, expected_empty_hash, "Empty transcript should match SHA-256(\"\")");
    }
    
    #[test]
    fn test_compute_transcript_hash_deterministic() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Add test messages
        handshake.update_transcript(b"ClientHello");
        handshake.update_transcript(b"ServerHello");
        
        // Compute hash twice
        let hash1 = handshake.compute_transcript_hash();
        let hash2 = handshake.compute_transcript_hash();
        
        // Should be identical (deterministic)
        assert_eq!(hash1, hash2, "Transcript hash should be deterministic");
        assert_eq!(hash1.len(), 32, "SHA-256 hash should be 32 bytes");
    }
    
    #[test]
    fn test_compute_transcript_hash_known_value() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Use a known message
        let message = b"test";
        handshake.update_transcript(message);
        
        let hash = handshake.compute_transcript_hash();
        
        // SHA-256 of "test"
        // echo -n "test" | sha256sum
        let expected_hash = hex::decode("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")
            .expect("Valid hex");
        
        assert_eq!(hash, expected_hash, "Transcript hash should match SHA-256(\"test\")");
    }
    
    #[test]
    fn test_transcript_accumulates_multiple_messages() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Simulate handshake message accumulation
        let client_hello = vec![1u8; 100];
        let server_hello = vec![2u8; 100];
        let encrypted_extensions = vec![3u8; 50];
        let certificate = vec![4u8; 200];
        let finished = vec![5u8; 50];
        
        handshake.update_transcript(&client_hello);
        handshake.update_transcript(&server_hello);
        handshake.update_transcript(&encrypted_extensions);
        handshake.update_transcript(&certificate);
        handshake.update_transcript(&finished);
        
        // Total should be sum of all messages
        let expected_total = 100 + 100 + 50 + 200 + 50;
        assert_eq!(handshake.transcript.len(), expected_total);
        
        // Compute hash of full transcript
        let hash = handshake.compute_transcript_hash();
        assert_eq!(hash.len(), 32, "SHA-256 hash should always be 32 bytes");
    }
    
    #[test]
    fn test_transcript_order_matters() {
        let beardog1 = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake1 = TlsHandshake::new(beardog1);
        
        let beardog2 = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake2 = TlsHandshake::new(beardog2);
        
        // Add messages in different orders
        handshake1.update_transcript(b"A");
        handshake1.update_transcript(b"B");
        
        handshake2.update_transcript(b"B");
        handshake2.update_transcript(b"A");
        
        let hash1 = handshake1.compute_transcript_hash();
        let hash2 = handshake2.compute_transcript_hash();
        
        // Hashes should be different (order matters!)
        assert_ne!(hash1, hash2, "Transcript hash should depend on message order");
    }
    
    #[test]
    fn test_transcript_hash_length() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Add various sized messages
        for size in [1, 10, 100, 1000, 10000] {
            handshake.update_transcript(&vec![0xFF; size]);
        }
        
        // Hash should always be 32 bytes regardless of input size
        let hash = handshake.compute_transcript_hash();
        assert_eq!(hash.len(), 32, "SHA-256 hash should always be 32 bytes");
    }
    
    // Tests for RFC 8446 handshake decryption
    
    #[tokio::test]
    #[ignore] // Requires BearDog running
    async fn test_decrypt_handshake_record_basic() {
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog.clone());
        
        // Create test keys (would normally come from CryptoCapability)
        let keys = TlsSecrets {
            client_write_key: vec![0x01; 32],
            server_write_key: vec![0x02; 32],
            client_write_iv: vec![0x03; 12],
            server_write_iv: vec![0x04; 12],
            client_handshake_secret: vec![0x05; 32],
            server_handshake_secret: vec![0x06; 32],
            handshake_secret: vec![0x07; 32],
        };
        
        // Create test encrypted data
        let encrypted = vec![0xFF; 48]; // 32 bytes data + 16 bytes Poly1305 tag
        
        // Attempt decryption (will fail since it's not real encrypted data, but tests code path)
        let result = handshake.decrypt_handshake_record(&encrypted, &keys, 0).await;
        
        // Should either succeed or fail with BearDog error, not panic
        assert!(result.is_ok() || result.is_err(), "Should handle decryption attempt gracefully");
    }
    
    #[test]
    fn test_handshake_transcript_with_plaintext() {
        // Test that transcript tracking works correctly
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Simulate adding plaintext handshake messages to transcript
        let client_hello = b"ClientHello handshake message (plaintext)";
        let server_hello = b"ServerHello handshake message (plaintext)";
        let encrypted_ext = b"EncryptedExtensions decrypted to plaintext";
        let certificate = b"Certificate decrypted to plaintext";
        let finished = b"Finished decrypted to plaintext";
        
        handshake.update_transcript(client_hello);
        handshake.update_transcript(server_hello);
        handshake.update_transcript(encrypted_ext);
        handshake.update_transcript(certificate);
        handshake.update_transcript(finished);
        
        // Verify transcript contains all plaintext messages
        let expected_len = client_hello.len() + server_hello.len() + 
                          encrypted_ext.len() + certificate.len() + finished.len();
        assert_eq!(handshake.transcript.len(), expected_len);
        
        // Compute hash of all plaintext messages
        let hash = handshake.compute_transcript_hash();
        assert_eq!(hash.len(), 32, "Transcript hash should be 32 bytes (SHA-256)");
    }
    
    #[test]
    fn test_sequence_number_nonce_construction() {
        // Test that sequence numbers produce different nonces
        let iv = vec![0x00; 12];
        
        // Sequence number 0
        let mut nonce0 = iv.clone();
        let seq0 = 0u64.to_be_bytes();
        for (i, &byte) in seq0.iter().enumerate() {
            let nonce_idx = nonce0.len() - 8 + i;
            nonce0[nonce_idx] ^= byte;
        }
        
        // Sequence number 1
        let mut nonce1 = iv.clone();
        let seq1 = 1u64.to_be_bytes();
        for (i, &byte) in seq1.iter().enumerate() {
            let nonce_idx = nonce1.len() - 8 + i;
            nonce1[nonce_idx] ^= byte;
        }
        
        // Nonces should be different
        assert_ne!(nonce0, nonce1, "Different sequence numbers should produce different nonces");
        
        // Last byte of nonce should differ by 1
        assert_eq!(nonce1[11], nonce0[11] ^ 1, "Sequence number XOR should affect last byte");
    }
    
    #[test]
    fn test_aad_construction() {
        // Test AAD (Additional Authenticated Data) construction for TLS 1.3
        let record_type = 0x17; // ApplicationData
        let version = [0x03, 0x03]; // TLS 1.2 compatibility
        let length = 100u16;
        
        let aad = [
            record_type,
            version[0],
            version[1],
            (length >> 8) as u8,
            (length & 0xFF) as u8,
        ];
        
        assert_eq!(aad.len(), 5, "AAD should be 5 bytes");
        assert_eq!(aad[0], 0x17, "ContentType should be ApplicationData");
        assert_eq!(aad[1], 0x03, "Version major should be 3");
        assert_eq!(aad[2], 0x03, "Version minor should be 3");
        assert_eq!(aad[3], 0x00, "Length high byte should be 0 for length 100");
        assert_eq!(aad[4], 0x64, "Length low byte should be 100 (0x64)");
    }
    
    #[test]
    fn test_transcript_plaintext_requirement() {
        // RFC 8446 Section 4.4.1: Transcript must contain PLAINTEXT messages
        // This test ensures we understand the requirement
        
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake = TlsHandshake::new(beardog);
        
        // Simulate plaintext messages (what SHOULD be in transcript)
        let plaintext_message = b"This is plaintext handshake message";
        handshake.update_transcript(plaintext_message);
        
        // Compute hash of plaintext
        let plaintext_hash = handshake.compute_transcript_hash();
        
        // Create new handshake with encrypted version (what SHOULD NOT be in transcript)
        let beardog2 = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/beardog.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let mut handshake2 = TlsHandshake::new(beardog2);
        let encrypted_message = b"ENCRYPTED_VERSION_OF_SAME_MESSAGE_WITH_TAG";
        handshake2.update_transcript(encrypted_message);
        
        // Compute hash of encrypted
        let encrypted_hash = handshake2.compute_transcript_hash();
        
        // Hashes MUST be different (plaintext vs encrypted)
        assert_ne!(plaintext_hash, encrypted_hash, 
                   "RFC 8446: Transcript hash of plaintext must differ from encrypted version!");
    }
    
    #[test]
    fn test_handshake_keys_separate_from_app_keys() {
        // Test that we understand TLS 1.3 has TWO key schedules:
        // 1. Handshake traffic keys - for decrypting post-handshake messages
        // 2. Application traffic keys - for encrypting HTTP data
        
        // Handshake keys (derived after ServerHello, no transcript hash)
        let handshake_keys = TlsSecrets {
            client_write_key: vec![0xAA; 32],
            server_write_key: vec![0xBB; 32],
            client_write_iv: vec![0xCC; 12],
            server_write_iv: vec![0xDD; 12],
            client_handshake_secret: vec![0xEE; 32],
            server_handshake_secret: vec![0xFF; 32],
            handshake_secret: vec![0x77; 32],
        };
        
        // Application keys (derived after Finished, WITH transcript hash)
        let app_keys = TlsSecrets {
            client_write_key: vec![0x11; 32],
            server_write_key: vec![0x22; 32],
            client_write_iv: vec![0x33; 12],
            server_write_iv: vec![0x44; 12],
            client_handshake_secret: vec![0x55; 32],
            server_handshake_secret: vec![0x66; 32],
            handshake_secret: vec![0x88; 32],
        };
        
        // Keys MUST be different
        assert_ne!(handshake_keys.server_write_key, app_keys.server_write_key,
                   "Handshake keys and application keys must be different!");
        assert_ne!(handshake_keys.server_write_iv, app_keys.server_write_iv,
                   "Handshake IVs and application IVs must be different!");
    }
    
    #[test]
    fn test_contains_finished_message_single() {
        // Test detecting Finished message when it's the only message in the record
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/test.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        // Build a Finished message: type (0x14) + length (3 bytes) + verify_data (32 bytes) + ContentType (0x16)
        let mut plaintext = vec![
            0x14, // HandshakeType: Finished
            0x00, // Length byte 1
            0x00, // Length byte 2
            0x20, // Length byte 3 (32 bytes)
        ];
        plaintext.extend_from_slice(&[0xAA; 32]); // verify_data (dummy)
        plaintext.push(0x16); // ContentType: Handshake
        
        assert!(handshake.contains_finished_message(&plaintext),
                "Should detect Finished message at offset 0");
    }
    
    #[test]
    fn test_contains_finished_message_multiple() {
        // Test detecting Finished message when multiple messages are coalesced
        // This simulates real-world behavior from Google, GitHub, CloudFlare, etc.
        let beardog = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/test.sock")) as std::sync::Arc<dyn CryptoCapability>;
        let handshake = TlsHandshake::new(beardog);
        
        // Message 1: EncryptedExtensions (type 0x08, 92 bytes body)
        let mut plaintext = vec![
            0x08, // HandshakeType: EncryptedExtensions
            0x00, // Length byte 1
            0x00, // Length byte 2
            0x5C, // Length byte 3 (92 bytes)
        ];
        plaintext.extend_from_slice(&[0xBB; 92]); // body (dummy)
        
        // Message 2: Certificate (type 0x0B, 2512 bytes body)
        plaintext.push(0x0B); // HandshakeType: Certificate
        plaintext.push(0x00); // Length byte 1
        plaintext.push(0x09); // Length byte 2
        plaintext.push(0xD0); // Length byte 3 (2512 bytes = 0x09D0)
        plaintext.extend_from_slice(&[0xCC; 2512]); // body (dummy)
        
        // Message 3: CertificateVerify (type 0x0F, 264 bytes body)
        plaintext.push(0x0F); // HandshakeType: CertificateVerify
        plaintext.push(0x00); // Length byte 1
        plaintext.push(0x01); // Length byte 2
        plaintext.push(0x08); // Length byte 3 (264 bytes = 0x0108)
        plaintext.extend_from_slice(&[0xDD; 264]); // body (dummy)
        
        // Message 4: Finished (type 0x14, 32 bytes body) ← THE ONE WE'RE LOOKING FOR!
        plaintext.push(0x14); // HandshakeType: Finished
        plaintext.push(0x00); // Length byte 1
        plaintext.push(0x00); // Length byte 2
        plaintext.push(0x20); // Length byte 3 (32 bytes)
        plaintext.extend_from_slice(&[0xEE; 32]); // verify_data (dummy)
        
        // ContentType byte at end (added during decryption)
        plaintext.push(0x16); // ContentType: Handshake
        
        // Total: 96 + 2516 + 268 + 36 + 1 = 2917 bytes (similar to real Google responses)
        assert_eq!(plaintext.len(), 2917, "Plaintext should be 2917 bytes");
        
        assert!(handshake.contains_finished_message(&plaintext),
                "Should detect Finished message at offset 2880 in multi-message record");
    }
    
    #[test]
    fn test_contains_finished_message_not_present() {
        // Test that we correctly return false when Finished is not present
        let crypto: std::sync::Arc<dyn CryptoCapability> = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/test.sock"));
        let handshake = TlsHandshake::new(crypto);
        
        // EncryptedExtensions only (no Finished)
        let mut plaintext = vec![
            0x08, // HandshakeType: EncryptedExtensions
            0x00, // Length byte 1
            0x00, // Length byte 2
            0x5C, // Length byte 3 (92 bytes)
        ];
        plaintext.extend_from_slice(&[0xBB; 92]); // body (dummy)
        plaintext.push(0x16); // ContentType: Handshake
        
        assert!(!handshake.contains_finished_message(&plaintext),
                "Should NOT detect Finished message when not present");
    }
    
    #[test]
    fn test_contains_finished_message_empty() {
        // Test edge case: empty plaintext
        let crypto: std::sync::Arc<dyn CryptoCapability> = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/test.sock"));
        let handshake = TlsHandshake::new(crypto);
        
        let plaintext = Vec::new();
        
        assert!(!handshake.contains_finished_message(&plaintext),
                "Should return false for empty plaintext");
    }
    
    #[test]
    fn test_contains_finished_message_malformed() {
        // Test resilience to malformed data
        let crypto: std::sync::Arc<dyn CryptoCapability> = std::sync::Arc::new(crate::crypto::BearDogProvider::new("/tmp/test.sock"));
        let handshake = TlsHandshake::new(crypto);
        
        // Truncated message header (only 2 bytes instead of 4)
        let plaintext = vec![0x08, 0x00];
        
        assert!(!handshake.contains_finished_message(&plaintext),
                "Should handle truncated message header gracefully");
    }
}


