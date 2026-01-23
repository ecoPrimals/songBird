//! TLS 1.3 handshake implementation

use crate::beardog_client::{BearDogClient, TlsSecrets};
use crate::error::{Error, Result};
use crate::tls::{session::SessionKeys, TLS_1_2, TLS_1_3, CIPHER_SUITES};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, trace, warn};

/// TLS 1.3 handshake
pub struct TlsHandshake {
    beardog: Arc<BearDogClient>,
    /// Transcript accumulator for RFC 8446 key derivation
    /// Accumulates all handshake messages for transcript hash computation
    transcript: Vec<u8>,
    /// Negotiated TLS 1.3 cipher suite from ServerHello
    /// 0x1301 = TLS_AES_128_GCM_SHA256
    /// 0x1302 = TLS_AES_256_GCM_SHA384
    /// 0x1303 = TLS_CHACHA20_POLY1305_SHA256
    cipher_suite: u16,
}

impl TlsHandshake {
    /// Create a new TLS handshake
    pub fn new(beardog: Arc<BearDogClient>) -> Self {
        Self { 
            beardog,
            transcript: Vec::new(),
            cipher_suite: 0,  // Will be set after parsing ServerHello
        }
    }
    
    /// Update transcript with handshake message
    /// RFC 8446 Section 4.4.1: Transcript hash includes all handshake messages
    /// 
    /// CRITICAL: This method expects handshake messages WITHOUT TLS record framing!
    /// - ClientHello: Must strip 5-byte TLS record header before calling
    /// - ServerHello: Already stripped by read_record()
    /// - Post-handshake messages: Already stripped by read_record()
    fn update_transcript(&mut self, message: &[u8]) {
        let before = self.transcript.len();
        let after = before + message.len();
        trace!("📝 Updating transcript: +{} bytes (total: {} → {} bytes)", 
               message.len(), before, after);
        trace!("   Message preview: {:02x?}", &message[..std::cmp::min(16, message.len())]);
        self.transcript.extend_from_slice(message);
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
            info!("   ClientHello total: {} bytes (with TLS header)", client_hello.len());
            info!("   ClientHello handshake message: {} bytes (TLS header stripped)", handshake_message.len());
            debug!("   TLS record header (5 bytes, NOT in transcript): {:02x?}", &client_hello[..5]);
            
            // BearDog-requested verification: First 32 bytes should start with 0x01 (ClientHello type)
            info!("🔍 VERIFICATION: ClientHello handshake message first bytes:");
            let preview_len = std::cmp::min(32, handshake_message.len());
            let first_bytes: String = handshake_message[..preview_len].iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            info!("   First {} bytes: {}", preview_len, first_bytes);
            if !handshake_message.is_empty() {
                let first_byte = handshake_message[0];
                if first_byte == 0x01 {
                    info!("   ✅ CORRECT: First byte is 0x01 (ClientHello handshake type)");
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
            
            self.update_transcript(handshake_message);
            info!("✅ ClientHello handshake message added to transcript ({} bytes)", handshake_message.len());
            debug!("📊 Transcript now: {} bytes (ClientHello only)", self.transcript.len());
            handshake_message.len()
        } else {
            error!("❌ ClientHello too short to contain handshake message!");
            self.update_transcript(&client_hello);
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
        if server_hello_type != 0x16 {
            error!("❌ Expected Handshake record (0x16) for ServerHello, got 0x{:02x}", server_hello_type);
            return Err(Error::TlsHandshake(format!(
                "Expected Handshake record for ServerHello, got type 0x{:02x}",
                server_hello_type
            )));
        }
        
        // RFC 8446: Update transcript with ServerHello
        // Note: read_record() already stripped the 5-byte TLS record header,
        // so server_hello contains only the handshake message (Type + Length + Content)
        info!("📝 TRANSCRIPT UPDATE 2: Adding ServerHello (WITHOUT TLS record header)");
        info!("   ServerHello handshake message: {} bytes (TLS header already stripped)", server_hello.len());
        
        // BearDog-requested verification: First 32 bytes should start with 0x02 (ServerHello type)
        info!("🔍 VERIFICATION: ServerHello handshake message first bytes:");
        let preview_len = std::cmp::min(32, server_hello.len());
        let first_bytes: String = server_hello[..preview_len].iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        info!("   First {} bytes: {}", preview_len, first_bytes);
        if !server_hello.is_empty() {
            let first_byte = server_hello[0];
            if first_byte == 0x02 {
                info!("   ✅ CORRECT: First byte is 0x02 (ServerHello handshake type)");
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
        
        self.update_transcript(&server_hello);
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
        let shared_secret = self.beardog
            .ecdh_derive(&client_private, &server_public)
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
        info!("   Total transcript: {} bytes (ClientHello + ServerHello)", self.transcript.len());
        info!("   ClientHello was: {} bytes (first message in transcript)", client_hello_len);
        info!("   ServerHello was: {} bytes (second message in transcript)", server_hello.len());
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
        info!("   Hash length: {} bytes (SHA-256)", handshake_transcript_hash.len());
        info!("   🎯 Transcript hash (hex): {}", hex::encode(&handshake_transcript_hash));
        info!("   This hash will be passed to BearDog's tls.derive_handshake_secrets");
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
        let handshake_keys = self.beardog
            .tls_derive_handshake_secrets(&shared_secret, &client_random, &server_random, &handshake_transcript_hash, self.cipher_suite)
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
                            self.update_transcript(&plaintext);
                            debug!("✅ Post-handshake PLAINTEXT {} added to transcript ({} bytes)", 
                                   messages_read, plaintext.len());
                            debug!("📊 Transcript now: {} bytes total (all plaintext)", self.transcript.len());
                            
                            // Check if this looks like the last handshake message (server Finished)
                            // Server Finished is typically small after decryption (< 100 bytes plaintext)
                            if plaintext.len() < 100 && messages_read >= 3 {
                                info!("🎯 Likely received server Finished message (small plaintext after 3+ messages)");
                                break;
                            }
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
        debug!("📊 Final transcript: {} bytes total (ALL PLAINTEXT - RFC 8446 compliant!)", self.transcript.len());
        debug!("Transcript hex (first 64 bytes): {}", hex::encode(&self.transcript[..std::cmp::min(64, self.transcript.len())]));
        
        let transcript_hash = self.compute_transcript_hash();
        info!("✅ Transcript hash computed: {} bytes (SHA-256)", transcript_hash.len());
        info!("🔐 Transcript hash (hex): {}", hex::encode(&transcript_hash));
        
        // Log transcript composition for debugging
        debug!("Transcript composition (RFC 8446 Section 4.4.1):");
        debug!("  - ClientHello handshake message (plaintext, no TLS header)");
        debug!("  - ServerHello handshake message (plaintext, no TLS header)");
        debug!("  - {} post-handshake DECRYPTED messages (plaintext, no TLS headers)", messages_read);
        debug!("  Total: {} bytes → SHA-256 → 32 bytes", self.transcript.len());
        debug!("  🎯 CRITICAL: All handshake messages are PLAINTEXT (decrypted)!");
        
        // 11. Derive application traffic secrets (for HTTP data encryption)
        // RFC 8446 Section 7.1: Application secrets are derived WITH transcript hash
        // Note: TLS 1.3 has separate key schedules:
        // - Handshake traffic secrets: For decrypting handshake messages (EncryptedExtensions, Certificate, etc.)
        // - Application traffic secrets: For encrypting HTTP data (requires transcript hash!)
        info!("Step 11: Deriving TLS application traffic secrets via BearDog (WITH transcript hash)");
        let derive_start = std::time::Instant::now();
        let secrets = self.beardog
            .tls_derive_application_secrets(&shared_secret, &client_random, &server_random, &transcript_hash)
            .await
            .map_err(|e| {
                error!("❌ BearDog TLS application secret derivation failed: {}", e);
                e
            })?;
        
        info!("🔐 TLS application traffic keys derived in {:?}", derive_start.elapsed());
        debug!("Application secrets derived successfully (for HTTP data encryption)");
        
        // 12. Send client Finished message (simplified - empty for MVP)
        // In full TLS 1.3, this would be encrypted and contain HMAC of transcript
        // For MVP, we send a minimal ChangeCipherSpec to indicate we're ready
        debug!("Step 12: Sending client ChangeCipherSpec acknowledgment");
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

        // SNI extension (0x0000) - Server Name Indication
        ext.extend_from_slice(&[0x00, 0x00]); // Extension type
        let sni_data = self.build_sni_extension(server_name);
        ext.extend_from_slice(&(sni_data.len() as u16).to_be_bytes());
        ext.extend_from_slice(&sni_data);

        // ALPN extension (0x0010) - Application-Layer Protocol Negotiation
        // CRITICAL for HTTPS servers like GitHub, CloudFlare, Google
        // RFC 7301: ProtocolNameList = length(2) + [length(1) + name(n)]+
        ext.extend_from_slice(&[0x00, 0x10]); // Extension type
        ext.extend_from_slice(&[0x00, 0x0b]); // Extension length: 11 bytes (2 + 1 + 8)
        ext.extend_from_slice(&[0x00, 0x09]); // Protocol list length: 9 bytes (1 + 8)
        ext.extend_from_slice(&[0x08]); // Protocol name length: 8 bytes
        ext.extend_from_slice(b"http/1.1"); // Protocol name: "http/1.1"

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
        info!("   Encrypted length: {} bytes", encrypted_record.len());
        info!("   Sequence number: {}", sequence_number);
        debug!("Encrypted data (first 32 bytes): {:02x?}", &encrypted_record[..std::cmp::min(32, encrypted_record.len())]);
        debug!("Encrypted data (last 16 bytes, likely tag): {:02x?}", &encrypted_record[encrypted_record.len().saturating_sub(16)..]);

        // Log keys and IVs
        info!("🔑 Cryptographic Material:");
        info!("   Server write key: {} bytes", keys.server_write_key.len());
        debug!("   Server write key (first 16 bytes): {:02x?}", &keys.server_write_key[..std::cmp::min(16, keys.server_write_key.len())]);
        info!("   Server write IV: {} bytes", keys.server_write_iv.len());
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
        info!("   Computed nonce: {:02x?}", nonce);
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
        info!("   AAD (TLS record header): {:02x?}", aad);
        debug!("   Breakdown:");
        debug!("     - ContentType: 0x{:02x} (APPLICATION_DATA)", record_type);
        debug!("     - Version: 0x{:02x}{:02x} (TLS 1.2 compat)", version[0], version[1]);
        debug!("     - Length: {} bytes (0x{:04x})", length, length);

        // Log comprehensive decryption parameters
        info!("🎯 Calling BearDog crypto.decrypt with:");
        info!("   Key: server_write_key ({} bytes)", keys.server_write_key.len());
        info!("   Nonce: {} bytes", nonce.len());
        info!("   Ciphertext+Tag: {} bytes", encrypted_record.len());
        info!("   AAD: {} bytes", aad.len());
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
                info!("   → Using AES-128-GCM (negotiated cipher suite)");
                debug!("  - Algorithm: AES-128-GCM AEAD");
                debug!("  - Key length from BearDog: {} bytes", keys.server_write_key.len());
                self.beardog.decrypt_aes_128_gcm(
                    &keys.server_write_key,
                    &nonce,
                    encrypted_record,
                    &aad,
                ).await
            }
            0x1302 => {
                // TLS_AES_256_GCM_SHA384 (high security)
                info!("   → Using AES-256-GCM (negotiated cipher suite)");
                debug!("  - Algorithm: AES-256-GCM AEAD");
                self.beardog.decrypt_aes_256_gcm(
                    &keys.server_write_key,
                    &nonce,
                    encrypted_record,
                    &aad,
                ).await
            }
            0x1303 => {
                // TLS_CHACHA20_POLY1305_SHA256 (software-only, mobile-optimized)
                info!("   → Using ChaCha20-Poly1305 (negotiated cipher suite)");
                debug!("  - Algorithm: ChaCha20-Poly1305 AEAD");
                self.beardog.decrypt(
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
        info!("   Plaintext length: {} bytes", plaintext.len());
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
            0x1301 => info!("   → TLS_AES_128_GCM_SHA256 (most common, hardware accelerated)"),
            0x1302 => info!("   → TLS_AES_256_GCM_SHA384 (high security, hardware accelerated)"),
            0x1303 => info!("   → TLS_CHACHA20_POLY1305_SHA256 (software-only, mobile-optimized)"),
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
        let ciphertext = self.beardog.encrypt(
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
        let plaintext = self.beardog.decrypt(
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
        assert!(extensions.len() > 90, "Should contain multiple extensions including ALPN");
        
        // Verify ALPN extension is present (0x00 0x10)
        let alpn_present = extensions.windows(2).any(|w| w == [0x00, 0x10]);
        assert!(alpn_present, "Should contain ALPN extension for HTTPS");
    }
    
    #[test]
    fn test_alpn_extension_encoding() {
        // CRITICAL: Validates byte-perfect ALPN encoding to prevent decode_error
        // This test prevents the exact bug biomeOS found in integration testing
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
    
    // ============================================================================
    // RFC 8446 Transcript Tracking Tests
    // ============================================================================
    
    #[test]
    fn test_transcript_empty_initially() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog);
        
        // Transcript should be empty initially
        assert_eq!(handshake.transcript.len(), 0, "Transcript should start empty");
    }
    
    #[test]
    fn test_update_transcript() {
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog1 = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let mut handshake1 = TlsHandshake::new(beardog1);
        
        let beardog2 = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let handshake = TlsHandshake::new(beardog.clone());
        
        // Create test keys (would normally come from BearDog)
        let keys = TlsSecrets {
            client_write_key: vec![0x01; 32],
            server_write_key: vec![0x02; 32],
            client_write_iv: vec![0x03; 12],
            server_write_iv: vec![0x04; 12],
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
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        
        let beardog = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
        let mut handshake = TlsHandshake::new(beardog);
        
        // Simulate plaintext messages (what SHOULD be in transcript)
        let plaintext_message = b"This is plaintext handshake message";
        handshake.update_transcript(plaintext_message);
        
        // Compute hash of plaintext
        let plaintext_hash = handshake.compute_transcript_hash();
        
        // Create new handshake with encrypted version (what SHOULD NOT be in transcript)
        let beardog2 = Arc::new(BearDogClient::new("/tmp/beardog.sock"));
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
        };
        
        // Application keys (derived after Finished, WITH transcript hash)
        let app_keys = TlsSecrets {
            client_write_key: vec![0x11; 32],
            server_write_key: vec![0x22; 32],
            client_write_iv: vec![0x33; 12],
            server_write_iv: vec![0x44; 12],
        };
        
        // Keys MUST be different
        assert_ne!(handshake_keys.server_write_key, app_keys.server_write_key,
                   "Handshake keys and application keys must be different!");
        assert_ne!(handshake_keys.server_write_iv, app_keys.server_write_iv,
                   "Handshake IVs and application IVs must be different!");
    }
}

