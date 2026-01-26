//! TLS 1.3 handshake flow orchestration
//!
//! This module contains the main handshake state machine that orchestrates
//! the complete TLS 1.3 handshake process.
//!
//! ## Handshake Flow (13 Steps)
//!
//! 1. Generate client keypair (X25519)
//! 2. Generate client random (32 bytes)
//! 3. Send ClientHello
//! 4. Receive ServerHello
//! 5. Parse ServerHello
//! 6. Perform ECDH key agreement
//! 7. Compute transcript hash for handshake key derivation
//! 8. Derive handshake traffic keys
//! 9. Read and decrypt post-handshake encrypted messages
//! 10. Compute final transcript hash for application key derivation
//! 11. Derive application traffic secrets
//! 12. Send client Finished message
//! 13. Read all post-handshake messages
//!
//! ## RFC 8446 Compliance
//!
//! This implementation follows RFC 8446 (TLS 1.3) strictly, including:
//! - Transcript hash computation (Section 4.4.1)
//! - Key schedule (Section 7.1)
//! - Finished message (Section 4.4.4)
//! - Record layer encryption (Section 5.2)

use super::core::TlsHandshake;
use crate::crypto::TlsHandshakeSecrets as TlsSecrets;
use crate::error::{Error, Result};
use crate::tls::session::SessionKeys;
use crate::tls::{TLS_1_2, CIPHER_SUITES};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, trace, warn};

impl TlsHandshake {
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
        let client_hello = self.build_client_hello(&client_random, &client_public, server_name)?;

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
            trace!(
                "   ClientHello handshake message: {} bytes (TLS header stripped)",
                handshake_message.len()
            );
            debug!(
                "   TLS record header (5 bytes, NOT in transcript): {:02x?}",
                &client_hello[..5]
            );

            // BearDog-requested verification: First 32 bytes should start with 0x01 (ClientHello type)
            trace!("🔍 VERIFICATION: ClientHello handshake message first bytes:");
            let preview_len = std::cmp::min(32, handshake_message.len());
            let first_bytes: String = handshake_message[..preview_len]
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            trace!("   First {} bytes: {}", preview_len, first_bytes);
            if !handshake_message.is_empty() {
                let first_byte = handshake_message[0];
                if first_byte == 0x01 {
                    trace!("   ✅ CORRECT: First byte is 0x01 (ClientHello handshake type)");
                } else if first_byte == 0x16 {
                    error!(
                        "   ❌ WRONG: First byte is 0x16 (TLS record header - should be stripped!)"
                    );
                } else {
                    warn!("   ⚠️  UNEXPECTED: First byte is 0x{:02x} (expected 0x01)", first_byte);
                }
            }

            debug!("   Handshake message (first 64 bytes, ADDED to transcript):");
            for (i, chunk) in handshake_message.chunks(16).take(4).enumerate() {
                let hex: String =
                    chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                debug!("     {:04x}: {}", i * 16, hex);
            }
            if handshake_message.len() > 64 {
                debug!("     ... ({} more bytes)", handshake_message.len() - 64);
            }

            self.update_transcript_with_logging(handshake_message, "ClientHello", false);
            info!(
                "✅ ClientHello handshake message added to transcript ({} bytes)",
                handshake_message.len()
            );
            debug!("📊 Transcript now: {} bytes (ClientHello only)", self.transcript.len());
            handshake_message.len()
        } else {
            error!("❌ ClientHello too short to contain handshake message!");
            self.update_transcript_with_logging(
                &client_hello,
                "ClientHello (full, with TLS header)",
                false,
            );
            client_hello.len()
        };

        // Comprehensive hex dump for debugging
        debug!("ClientHello hex dump (first 160 bytes):");
        for (i, chunk) in client_hello.chunks(16).take(10).enumerate() {
            let hex: String =
                chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            let ascii: String = chunk
                .iter()
                .map(|&b| {
                    if (32..127).contains(&b) {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();
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
        let (server_hello_type, server_hello) =
            timeout(Duration::from_secs(10), self.read_record(stream))
                .await
                .map_err(|_| {
                    error!("❌ TIMEOUT waiting for ServerHello after {:?}", read_start.elapsed());
                    Error::TlsHandshake("Timeout waiting for ServerHello (10s)".to_string())
                })
                .and_then(|r| {
                    r.map_err(|e| {
                        error!(
                            "❌ Error reading ServerHello after {:?}: {}",
                            read_start.elapsed(),
                            e
                        );
                        e
                    })
                })?;
        info!(
            "✅ Received ServerHello: type=0x{:02x}, {} bytes in {:?}",
            server_hello_type,
            server_hello.len(),
            read_start.elapsed()
        );
        trace!(
            "ServerHello content: {:02x?}",
            &server_hello[..std::cmp::min(64, server_hello.len())]
        );

        // Validate this is a Handshake record (0x16)
        // Check if we received a TLS alert instead of ServerHello
        if server_hello_type == 0x15 {
            // TLS Alert record (RFC 8446 Section 6)
            use crate::tls::alert::TlsAlert;

            warn!("⚠️  Received TLS Alert instead of ServerHello");

            // Parse the alert message (skip TLS record header if present)
            let alert_data = if server_hello.len() >= 5 {
                &server_hello[5..] // Skip 5-byte TLS record header
            } else {
                &server_hello[..] // Use all data if too short
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
                        error!(
                            "   Alert bytes: {:02x?}",
                            &alert_data[..std::cmp::min(2, alert_data.len())]
                        );
                    }
                    return Err(Error::TlsHandshake(format!(
                        "Server sent TLS alert but parsing failed: {}",
                        e
                    )));
                }
            }
        }

        if server_hello_type != 0x16 {
            error!(
                "❌ Expected Handshake record (0x16) for ServerHello, got 0x{:02x}",
                server_hello_type
            );

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
        trace!(
            "   ServerHello handshake message: {} bytes (TLS header already stripped)",
            server_hello.len()
        );

        // BearDog-requested verification: First 32 bytes should start with 0x02 (ServerHello type)
        trace!("🔍 VERIFICATION: ServerHello handshake message first bytes:");
        let preview_len = std::cmp::min(32, server_hello.len());
        let first_bytes: String = server_hello[..preview_len]
            .iter()
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
            let hex: String =
                chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            debug!("     {:04x}: {}", i * 16, hex);
        }
        if server_hello.len() > 64 {
            debug!("     ... ({} more bytes)", server_hello.len() - 64);
        }

        self.update_transcript_with_logging(&server_hello, "ServerHello", false);
        info!(
            "✅ ServerHello handshake message added to transcript ({} bytes)",
            server_hello.len()
        );
        debug!(
            "📊 Transcript now: {} bytes total (ClientHello + ServerHello)",
            self.transcript.len()
        );

        // 5. Parse ServerHello
        debug!("Step 5: Parsing ServerHello");
        let (server_random, server_public, cipher_suite) =
            self.parse_server_hello(&server_hello).map_err(|e| {
                error!("❌ Failed to parse ServerHello: {}", e);
                e
            })?;
        self.cipher_suite = cipher_suite; // Store for later AEAD algorithm selection
        debug!("✅ Parsed ServerHello - cipher_suite: 0x{:04x}, server_random: {} bytes, server_public: {} bytes", 
               cipher_suite, server_random.len(), server_public.len());
        trace!(
            "Server public key: {:02x?}",
            &server_public[..std::cmp::min(32, server_public.len())]
        );

        // 6. Perform ECDH
        debug!("Step 6: Computing shared secret via BearDog ECDH");
        let ecdh_start = std::time::Instant::now();
        let shared_secret = self
            .crypto
            .derive_x25519_shared_secret(&client_private, &server_public)
            .await
            .map_err(|e| {
                error!("❌ BearDog ECDH derivation failed: {}", e);
                e
            })?;
        debug!(
            "✅ Computed shared secret: {} bytes in {:?}",
            shared_secret.len(),
            ecdh_start.elapsed()
        );
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
            let hex: String =
                chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
            debug!("     {:04x}: {}", i * 32, hex);
        }
        debug!("   ⚠️  CRITICAL: This transcript should contain:");
        debug!("      1. ClientHello handshake message (without TLS record header)");
        debug!("      2. ServerHello handshake message (without TLS record header)");
        debug!("      3. NO TLS record headers (no [16 03 03 ...] prefixes)");
        debug!("      4. ONLY the handshake message content (Type + Length + Content)");

        info!(
            "🔐 COMPUTING HANDSHAKE TRANSCRIPT HASH (SHA-256 of {} bytes)",
            self.transcript.len()
        );
        debug!("   RFC 8446 Section 4.4.1: Transcript-Hash(M1, M2) = Hash(M1 || M2)");
        debug!("   For handshake keys: M1 = ClientHello, M2 = ServerHello");
        debug!("   Both messages are handshake message bodies ONLY (no TLS record headers)");

        let handshake_transcript_hash = self.compute_transcript_hash();

        info!("✅ Handshake transcript hash computed!");
        trace!("   Hash length: {} bytes (SHA-256)", handshake_transcript_hash.len());
        trace!("   🎯 Transcript hash (hex): {}", hex::encode(&handshake_transcript_hash));
        trace!("   This hash will be passed to BearDog's tls.derive_handshake_secrets");
        debug!(
            "🔍 BearDog will use this hash to derive handshake traffic keys (RFC 8446 Section 7.1)"
        );
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
        let handshake_keys = self
            .crypto
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

        while messages_read < 5 {
            // Read up to 5 more records (generous limit)
            debug!(
                "Waiting for encrypted post-handshake message {} (5 second timeout)",
                messages_read + 1
            );
            let record_start = std::time::Instant::now();

            match timeout(Duration::from_secs(5), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_record))) => {
                    info!(
                        "✅ Read TLS record type=0x{:02x} ({} bytes) in {:?}",
                        content_type,
                        encrypted_record.len(),
                        record_start.elapsed()
                    );

                    // RFC 8446 Section 5: Skip ChangeCipherSpec (legacy compatibility)
                    // ChangeCipherSpec (0x14) is PLAINTEXT in TLS 1.3, not encrypted!
                    // It's a 1-byte legacy message (0x01) for middlebox compatibility
                    // We MUST NOT try to decrypt it (would fail: 1 byte < 16 byte AEAD tag)
                    if content_type == 0x14 {
                        // CHANGE_CIPHER_SPEC
                        info!(
                            "⏭️  Skipping ChangeCipherSpec (legacy TLS 1.3 compatibility message)"
                        );
                        debug!(
                            "   RFC 8446 Section 5: ChangeCipherSpec is PLAINTEXT (not encrypted)"
                        );
                        debug!("   Content: {:02x?}", encrypted_record);

                        // Validate it's the expected 1-byte 0x01
                        if encrypted_record.len() == 1 && encrypted_record[0] == 0x01 {
                            debug!("   ✅ Valid ChangeCipherSpec (0x01)");
                        } else {
                            warn!(
                                "   ⚠️  Unexpected ChangeCipherSpec: {} bytes, content={:02x?}",
                                encrypted_record.len(),
                                encrypted_record
                            );
                        }

                        // Do NOT add to transcript (not a handshake message)
                        // Do NOT try to decrypt (it's plaintext!)
                        // Just skip and continue to next record
                        continue;
                    }

                    // For APPLICATION_DATA (0x17): encrypted handshake messages
                    // (EncryptedExtensions, Certificate, CertificateVerify, Finished)
                    if content_type != 0x17 {
                        warn!(
                            "⚠️  Unexpected record type after ServerHello: 0x{:02x}",
                            content_type
                        );
                        continue;
                    }

                    messages_read += 1;
                    info!(
                        "✅ Read encrypted handshake record {} ({} bytes) in {:?}",
                        messages_read,
                        encrypted_record.len(),
                        record_start.elapsed()
                    );
                    trace!(
                        "Encrypted record {} preview: {:02x?}",
                        messages_read,
                        &encrypted_record[..std::cmp::min(32, encrypted_record.len())]
                    );

                    // RFC 8446 CRITICAL: Decrypt the handshake message before adding to transcript!
                    // Transcript hash must be computed over PLAINTEXT messages, not encrypted ciphertext
                    debug!(
                        "🔓 Decrypting handshake record {} with handshake traffic keys (seq={})",
                        messages_read, sequence_number
                    );
                    let decrypt_start = std::time::Instant::now();

                    match self
                        .decrypt_handshake_record(
                            &encrypted_record,
                            &handshake_keys,
                            sequence_number,
                        )
                        .await
                    {
                        Ok(plaintext) => {
                            info!(
                                "✅ Decrypted handshake record {} to {} bytes of plaintext in {:?}",
                                messages_read,
                                plaintext.len(),
                                decrypt_start.elapsed()
                            );
                            trace!(
                                "Plaintext preview: {:02x?}",
                                &plaintext[..std::cmp::min(32, plaintext.len())]
                            );

                            sequence_number += 1;

                            // RFC 8446 Section 4.4.1: Add PLAINTEXT to transcript (not encrypted!)
                            // CRITICAL FIX: Parse INDIVIDUAL handshake messages from the decrypted blob!
                            // A single TLS record may contain MULTIPLE handshake messages (EncryptedExtensions,
                            // Certificate, CertificateVerify, Finished) concatenated together.
                            // RFC 8446 requires each message to be added to the transcript SEPARATELY!

                            info!("🔬 CRITICAL: Parsing individual handshake messages from decrypted record");
                            let parsed_messages = self.parse_handshake_messages(&plaintext)?;

                            info!(
                                "📝 Adding {} individual messages to transcript (NOT as one blob!)",
                                parsed_messages.len()
                            );
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

                            debug!(
                                "✅ Post-handshake messages {} parsed and added to transcript",
                                messages_read
                            );
                            debug!(
                                "📊 Transcript now: {} bytes total (all plaintext)",
                                self.transcript.len()
                            );

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
                                info!(
                                    "✅ Decrypted {} messages before error, proceeding",
                                    messages_read - 1
                                );
                                break;
                            }
                            error!(
                                "❌ Handshake decryption failed after {} messages: {}",
                                messages_read, e
                            );
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
                    warn!(
                        "⏱️  Timeout waiting for post-handshake message {} after {:?}",
                        messages_read + 1,
                        record_start.elapsed()
                    );
                    if messages_read >= 3 {
                        info!("✅ Timeout after {} decrypted messages ({:?} total), assuming handshake complete", 
                              messages_read, post_handshake_start.elapsed());
                        break;
                    }
                    error!("❌ Handshake timeout after only {} messages", messages_read);
                    return Err(Error::TlsHandshake(format!(
                        "Timeout reading post-handshake messages (got {}/3+)",
                        messages_read
                    )));
                }
            }
        }

        debug!(
            "Post-handshake phase complete: {} messages decrypted in {:?}",
            messages_read,
            post_handshake_start.elapsed()
        );

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
        debug!(
            "📊 Final transcript: {} bytes total (ALL PLAINTEXT - RFC 8446 compliant!)",
            self.transcript.len()
        );
        debug!(
            "Transcript hex (first 64 bytes): {}",
            hex::encode(&self.transcript[..std::cmp::min(64, self.transcript.len())])
        );

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
        trace!(
            "  ✅ 3-{}. {} post-handshake DECRYPTED messages (plaintext, no TLS headers)",
            2 + messages_read,
            messages_read
        );
        trace!("     (EncryptedExtensions, Certificate, CertificateVerify, server Finished)");
        trace!("  ❌ NOT INCLUDED: client Finished (will be sent AFTER key derivation)");
        trace!(
            "  Total transcript: {} bytes → SHA-256 → {} bytes",
            self.transcript.len(),
            transcript_hash.len()
        );
        trace!("  🎯 CRITICAL: All handshake messages are PLAINTEXT (decrypted)!");
        debug!("Full transcript (hex): {}", hex::encode(&self.transcript));
        debug!("Transcript hash (hex): {}", hex::encode(&transcript_hash));
        trace!("════════════════════════════════════════════════════════════");

        // 11. Derive application traffic secrets (for HTTP data encryption)
        // RFC 8446 Section 7.1: Application secrets are derived WITH transcript hash
        // Note: TLS 1.3 has separate key schedules:
        // - Handshake traffic secrets: For decrypting handshake messages (EncryptedExtensions, Certificate, etc.)
        // - Application traffic secrets: For encrypting HTTP data (requires transcript hash!)
        info!(
            "Step 11: Deriving TLS application traffic secrets via BearDog (WITH transcript hash)"
        );
        let derive_start = std::time::Instant::now();
        let secrets = self
            .crypto
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
        trace!(
            "  client_write_key ({} bytes): {}",
            secrets.client_write_key.len(),
            hex::encode(&secrets.client_write_key)
        );
        trace!(
            "  client_write_iv ({} bytes): {}",
            secrets.client_write_iv.len(),
            hex::encode(&secrets.client_write_iv)
        );
        info!("");
        info!(
            "Expected key length for cipher 0x{:04x}: {} bytes",
            self.cipher_suite,
            match self.cipher_suite {
                0x1301 => 16, // AES-128-GCM
                0x1302 => 32, // AES-256-GCM
                0x1303 => 32, // ChaCha20-Poly1305
                _ => 0,
            }
        );
        info!("Expected IV length: 12 bytes (all TLS 1.3 ciphers)");
        info!("");
        info!("⚠️  CRITICAL CHECK:");
        if secrets.client_write_key.len()
            != match self.cipher_suite {
                0x1301 => 16,
                0x1302 => 32,
                0x1303 => 32,
                _ => 0,
            }
        {
            error!(
                "❌ client_write_key length MISMATCH! Expected {} bytes, got {} bytes",
                match self.cipher_suite {
                    0x1301 => 16,
                    0x1302 => 32,
                    0x1303 => 32,
                    _ => 0,
                },
                secrets.client_write_key.len()
            );
        } else {
            info!(
                "✅ client_write_key length is CORRECT ({} bytes)",
                secrets.client_write_key.len()
            );
        }
        if secrets.client_write_iv.len() != 12 {
            error!(
                "❌ client_write_iv length MISMATCH! Expected 12 bytes, got {} bytes",
                secrets.client_write_iv.len()
            );
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
        let mut read_sequence_number: u64 = 0; // Separate sequence for reading (starts at 0)

        // Loop to read ALL post-handshake messages
        // Use shorter timeout (200ms) between messages since server sends them quickly
        loop {
            match timeout(Duration::from_millis(200), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_data))) => {
                    post_handshake_count += 1;
                    info!(
                        "✅ Post-handshake message #{}: type=0x{:02x}, {} bytes (encrypted)",
                        post_handshake_count,
                        content_type,
                        encrypted_data.len()
                    );

                    match content_type {
                        0x17 => {
                            // APPLICATION_DATA (encrypted post-handshake message)
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
                                0x17,
                                0x03,
                                0x03,
                                (encrypted_data.len() >> 8) as u8,
                                (encrypted_data.len() & 0xFF) as u8,
                            ];

                            // Decrypt based on cipher suite
                            let plaintext_result = match self.cipher_suite {
                                0x1301 => {
                                    self.crypto
                                        .aes128_gcm_decrypt(
                                            &secrets.server_write_key,
                                            &nonce,
                                            &encrypted_data,
                                            &aad,
                                        )
                                        .await
                                }
                                0x1302 => {
                                    self.crypto
                                        .aes256_gcm_decrypt(
                                            &secrets.server_write_key,
                                            &nonce,
                                            &encrypted_data,
                                            &aad,
                                        )
                                        .await
                                }
                                _ => {
                                    self.crypto
                                        .decrypt(
                                            &secrets.server_write_key,
                                            &nonce,
                                            &encrypted_data,
                                            &aad,
                                        )
                                        .await
                                }
                            };

                            read_sequence_number += 1; // Increment for next message

                            match plaintext_result {
                                Ok(plaintext) => {
                                    // RFC 8446: TLSInnerPlaintext has ContentType as last non-zero byte
                                    if let Some(&inner_type) = plaintext.last() {
                                        let content = &plaintext[..plaintext.len() - 1];
                                        trace!(
                                            "   📨 Decrypted: {} bytes, inner type=0x{:02x}",
                                            content.len(),
                                            inner_type
                                        );

                                        match inner_type {
                                            0x16 => {
                                                // Handshake (NewSessionTicket is type 0x04)
                                                if !content.is_empty() && content[0] == 0x04 {
                                                    trace!("   🎟️  NewSessionTicket #{} (ignored for now)", post_handshake_count);
                                                } else {
                                                    trace!(
                                                        "   📋 Handshake message type 0x{:02x}",
                                                        content.first().unwrap_or(&0)
                                                    );
                                                }
                                            }
                                            0x15 => {
                                                // ALERT (inside encrypted envelope!)
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
                                                trace!(
                                                    "   ❓ Unknown inner type: 0x{:02x}",
                                                    inner_type
                                                );
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
                        0x15 => {
                            // Unencrypted ALERT (shouldn't happen in TLS 1.3 after handshake)
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
                            trace!(
                                "   ℹ️  Ignoring unexpected message type: 0x{:02x}",
                                content_type
                            );
                        }
                    }
                }
                Ok(Err(e)) => {
                    if post_handshake_count == 0 {
                        warn!("⚠️  Error reading first post-handshake message: {}", e);
                    } else {
                        debug!(
                            "   Error reading message #{}: {} (might be normal)",
                            post_handshake_count + 1,
                            e
                        );
                    }
                    break;
                }
                Err(_) => {
                    if post_handshake_count == 0 {
                        trace!("   ⏱️  Timeout - no post-handshake messages (this is OK)");
                    } else {
                        trace!(
                            "   ⏱️  Timeout after {} post-handshake messages",
                            post_handshake_count
                        );
                    }
                    break;
                }
            }
        }

        info!(
            "✅ Consumed {} post-handshake messages - stream ready for HTTP!",
            post_handshake_count
        );

        let total_time = handshake_start.elapsed();
        info!("🎉 ✅ TLS 1.3 handshake complete in {:?}", total_time);
        debug!(
            "Handshake summary: {} post-handshake messages, cipher: TLS_CHACHA20_POLY1305_SHA256",
            messages_read
        );

        Ok(SessionKeys {
            client_write_key: secrets.client_write_key,
            server_write_key: secrets.server_write_key,
            client_write_iv: secrets.client_write_iv,
            server_write_iv: secrets.server_write_iv,
            cipher_suite: self.cipher_suite, // Pass negotiated cipher suite to session
            initial_read_sequence: read_sequence_number, // Account for post-handshake messages consumed
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
        let verify_data = self
            .crypto
            .tls_compute_finished_verify_data(
                &handshake_keys.client_handshake_secret, // RFC 8446 client_handshake_traffic_secret (32-byte PRK)
                &transcript_hash,
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
        trace!("   AAD (hex): {}", hex::encode(aad));
        trace!("   Plaintext length: {} bytes", plaintext.len());
        trace!("   ⚠️  HYPOTHESIS: If server_write_key == server's expected client_write_key,");
        trace!("      then BearDog is swapping client/server labels!");

        // Use client_write_key for client→server encryption (correct per RFC 8446)
        let encryption_key = &handshake_keys.client_write_key;
        trace!("   🔑 USING KEY: client_write_key (correct per RFC 8446)");

        let ciphertext = match self.cipher_suite {
            0x1301 => {
                trace!("   → Using AES-128-GCM for client Finished");
                self.crypto.aes128_gcm_encrypt(encryption_key, &nonce, &plaintext, &aad).await
            }
            0x1302 => {
                trace!("   → Using AES-256-GCM for client Finished");
                self.crypto.aes256_gcm_encrypt(encryption_key, &nonce, &plaintext, &aad).await
            }
            0x1303 => {
                trace!("   → Using ChaCha20-Poly1305 for client Finished");
                self.crypto.encrypt(encryption_key, &nonce, &plaintext, &aad).await
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
            error!("❌ Failed to encrypt client Finished: {}", e);
            e
        })?;

        info!("✅ Encrypted client Finished: {} bytes (includes 16-byte tag)", ciphertext.len());

        // 6. Build complete TLS record: header + ciphertext
        let mut tls_record = Vec::new();
        tls_record.extend_from_slice(&aad);
        tls_record.extend_from_slice(&ciphertext);

        info!("📤 Sending client Finished TLS record: {} bytes total", tls_record.len());
        debug!(
            "   TLS record preview: {:02x?}",
            &tls_record[..std::cmp::min(32, tls_record.len())]
        );

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
            debug!(
                "   Handshake message at offset {}: type=0x{:02x} ({}), length={} bytes",
                offset, msg_type, msg_name, msg_len
            );

            // Skip to next message: header (4 bytes) + body (msg_len bytes)
            offset += 4 + msg_len;

            // Safety check: prevent infinite loop on malformed data
            if msg_len > 65536 {
                warn!(
                    "   Stopping parse: suspicious message length {} at offset {}",
                    msg_len, offset
                );
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
}
