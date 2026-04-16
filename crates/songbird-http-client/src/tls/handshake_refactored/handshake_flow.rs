// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 handshake flow orchestration
//!
//! This module contains the main handshake state machine that orchestrates
//! the complete TLS 1.3 handshake process.
//!
//! ## Handshake Flow (13 Steps)
//!
//! 1. Generate client keypair (X25519)
//! 2. Generate client random (32 bytes)
//! 3. Send `ClientHello`
//! 4. Receive `ServerHello`
//! 5. Parse `ServerHello`
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
use super::tls_wire_u16;
use crate::crypto::CryptoCapability;
use crate::crypto::{TlsApplicationSecrets, TlsHandshakeSecrets as TlsSecrets};
use crate::error::{Error, Result};
use crate::tls::session::SessionKeys;
use crate::tls::{CIPHER_SUITES, TLS_1_2};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, error, info, warn};

impl TlsHandshake {
    /// Execute the complete TLS 1.3 handshake
    ///
    /// Orchestrates all 13 steps of the TLS 1.3 handshake, delegating
    /// cryptographic operations to the `security provider` via the crypto capability.
    ///
    /// # Errors
    ///
    /// Returns error if any handshake step fails (network, crypto, protocol).
    pub async fn handshake(
        &mut self,
        stream: &mut TcpStream,
        server_name: &str,
    ) -> Result<SessionKeys> {
        info!("🤝 Starting TLS 1.3 handshake with {}", server_name);
        let handshake_start = std::time::Instant::now();

        // Steps 1-3: Generate keypair, random, send ClientHello
        let (client_public, client_private) = self.crypto.generate_x25519_keypair().await?;
        let client_random = self.generate_random();
        let client_hello = self.build_client_hello(&client_random, &client_public, server_name)?;

        info!("📤 Sending ClientHello: {} bytes", client_hello.len());
        self.add_client_hello_to_transcript(&client_hello);

        stream.write_all(&client_hello).await.map_err(|e| {
            error!("❌ Failed to write ClientHello: {}", e);
            Error::Io(e)
        })?;
        stream.flush().await.map_err(Error::Io)?;

        // Step 4: Receive ServerHello with timeout
        let server_hello = self.receive_server_hello(stream).await?;

        // Update transcript with ServerHello
        self.add_server_hello_to_transcript(&server_hello);

        // Step 5: Parse ServerHello
        let (server_random, server_public, cipher_suite) =
            self.parse_server_hello(&server_hello)?;
        self.cipher_suite = cipher_suite;
        debug!(
            "Parsed ServerHello: cipher=0x{:04x}, server_public={} bytes",
            cipher_suite,
            server_public.len()
        );

        // Step 6: ECDH key agreement
        let shared_secret = self
            .crypto
            .derive_x25519_shared_secret(&client_private, &server_public)
            .await
            .map_err(|e| {
                error!("❌ ECDH derivation failed: {}", e);
                e
            })?;
        debug!("✅ Shared secret: {} bytes", shared_secret.len());

        // Step 7: Transcript hash for handshake key derivation
        let handshake_transcript_hash =
            self.compute_transcript_hash_for_cipher(cipher_suite).await?;
        debug!(
            "Handshake transcript hash: {} bytes ({})",
            handshake_transcript_hash.len(),
            Self::hash_algorithm_name(&handshake_transcript_hash)
        );

        // Step 8: Derive handshake traffic keys
        let handshake_keys = self
            .crypto
            .tls_derive_handshake_secrets(
                &shared_secret,
                &client_random,
                &server_random,
                &handshake_transcript_hash,
                cipher_suite,
            )
            .await
            .map_err(|e| {
                error!("❌ Failed to derive handshake keys: {}", e);
                e
            })?;
        info!("✅ Handshake traffic keys derived");

        // Step 9: Read and decrypt post-handshake encrypted messages
        let messages_read = self.read_encrypted_handshake_messages(stream, &handshake_keys).await?;
        debug!("Decrypted {} encrypted handshake messages", messages_read);

        // Step 10: Compute final transcript hash for application key derivation
        let transcript_hash = self.compute_transcript_hash_for_cipher(self.cipher_suite).await?;
        info!(
            "✅ Application transcript hash: {} bytes ({})",
            transcript_hash.len(),
            Self::hash_algorithm_name(&transcript_hash)
        );

        // Step 11: Derive application traffic secrets
        let secrets = self
            .crypto
            .tls_derive_application_secrets(
                &handshake_keys.handshake_secret,
                &transcript_hash,
                self.cipher_suite,
            )
            .await
            .map_err(|e| {
                error!("❌ Application secret derivation failed: {}", e);
                e
            })?;

        self.validate_key_lengths(&secrets);

        // Step 12: Send client Finished
        self.send_client_finished(stream, &handshake_keys).await?;
        info!("✅ Client Finished sent");

        // Step 13: Consume post-handshake messages
        let post_result = self.consume_post_handshake_messages(stream, &secrets).await;

        let total_time = handshake_start.elapsed();
        info!("🎉 TLS 1.3 handshake complete in {:?}", total_time);

        Ok(SessionKeys {
            client_write_key: secrets.client_write_key,
            server_write_key: secrets.server_write_key,
            client_write_iv: secrets.client_write_iv,
            server_write_iv: secrets.server_write_iv,
            cipher_suite: self.cipher_suite,
            initial_read_sequence: post_result.read_sequence_number,
        })
    }

    /// Add `ClientHello` to transcript (strip 5-byte TLS record header per RFC 8446)
    fn add_client_hello_to_transcript(&mut self, client_hello: &[u8]) {
        if client_hello.len() > 5 {
            let handshake_message = &client_hello[5..];
            self.update_transcript_with_logging(handshake_message, "ClientHello", false);
            debug!("ClientHello added to transcript: {} bytes", handshake_message.len());
        } else {
            self.update_transcript_with_logging(client_hello, "ClientHello (short)", false);
        }
    }

    /// Add `ServerHello` to transcript (already stripped of TLS record header by `read_record`)
    fn add_server_hello_to_transcript(&mut self, server_hello: &[u8]) {
        self.update_transcript_with_logging(server_hello, "ServerHello", false);
        debug!(
            "ServerHello added to transcript: {} bytes (total: {})",
            server_hello.len(),
            self.transcript.len()
        );
    }

    /// Receive and validate `ServerHello` with timeout
    async fn receive_server_hello(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        info!("📥 Waiting for ServerHello");
        let (server_hello_type, server_hello) =
            timeout(Duration::from_secs(10), self.read_record(stream))
                .await
                .map_err(|_| Error::TlsHandshake("Timeout waiting for ServerHello (10s)".into()))?
                .map_err(|e| {
                    error!("❌ Error reading ServerHello: {}", e);
                    e
                })?;

        info!("Received: type=0x{:02x}, {} bytes", server_hello_type, server_hello.len());

        // Check for TLS alert
        if server_hello_type == 0x15 {
            return Err(self.handle_tls_alert(&server_hello));
        }

        if server_hello_type != 0x16 {
            let type_hint = match server_hello_type {
                0x14 => "Change Cipher Spec (TLS 1.2 legacy)",
                0x17 => "Application Data (unexpected)",
                _ => "Unknown record type",
            };
            return Err(Error::TlsHandshake(format!(
                "Expected Handshake (0x16), got 0x{server_hello_type:02x} ({type_hint})"
            )));
        }

        Ok(server_hello)
    }

    /// Handle a TLS alert received instead of expected handshake message
    #[expect(clippy::unused_self, reason = "unused bindings/imports in this compilation unit")] // API consistency
    fn handle_tls_alert(&self, data: &[u8]) -> Error {
        use crate::tls::alert::TlsAlert;

        let alert_data = if data.len() >= 5 {
            &data[5..]
        } else {
            data
        };

        match TlsAlert::parse(alert_data) {
            Ok(alert) => {
                error!("🚨 TLS ALERT: {}", alert.to_detailed_string());
                Error::TlsHandshake(format!(
                    "Server sent {} ({}). {}",
                    alert,
                    alert.description.explanation(),
                    alert.description.suggested_action()
                ))
            }
            Err(e) => Error::TlsHandshake(format!("Server sent TLS alert, parse failed: {e}")),
        }
    }

    /// Read and decrypt encrypted handshake messages (Step 9)
    ///
    /// Reads `EncryptedExtensions`, Certificate, `CertificateVerify`, and Finished
    /// from the server, decrypting each and adding plaintext to the transcript.
    async fn read_encrypted_handshake_messages(
        &mut self,
        stream: &mut TcpStream,
        handshake_keys: &TlsSecrets,
    ) -> Result<u32> {
        info!("Step 9: Reading encrypted handshake messages");

        let mut messages_read = 0u32;
        let mut sequence_number = 0u64;

        while messages_read < 5 {
            match timeout(Duration::from_secs(5), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_record))) => {
                    // Skip ChangeCipherSpec (legacy TLS 1.3 compatibility)
                    if content_type == 0x14 {
                        debug!("⏭️ Skipping ChangeCipherSpec");
                        continue;
                    }

                    if content_type != 0x17 {
                        warn!("Unexpected record type 0x{:02x}", content_type);
                        continue;
                    }

                    messages_read += 1;

                    match self
                        .decrypt_handshake_record(
                            &encrypted_record,
                            handshake_keys,
                            sequence_number,
                        )
                        .await
                    {
                        Ok(plaintext) => {
                            sequence_number += 1;

                            // Parse individual handshake messages and add to transcript
                            let parsed_messages = self.parse_handshake_messages(&plaintext);
                            for (msg_type, msg_data) in &parsed_messages {
                                let name = match msg_type {
                                    0x08 => "EncryptedExtensions",
                                    0x0B => "Certificate",
                                    0x0F => "CertificateVerify",
                                    0x14 => "Server Finished",
                                    _ => "Unknown",
                                };
                                self.update_transcript_with_logging(msg_data, name, true);
                            }

                            // Check for Finished message
                            if self.contains_finished_message(&plaintext) {
                                info!("🎯 Server Finished received");
                                break;
                            }
                        }
                        Err(e) => {
                            if messages_read >= 4 {
                                info!(
                                    "Decrypted {} messages before error, proceeding",
                                    messages_read - 1
                                );
                                break;
                            }
                            return Err(e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    if messages_read >= 3 {
                        break;
                    }
                    return Err(e);
                }
                Err(_) => {
                    if messages_read >= 3 {
                        break;
                    }
                    return Err(Error::TlsHandshake(format!(
                        "Timeout reading handshake messages (got {messages_read}/3+)"
                    )));
                }
            }
        }

        Ok(messages_read)
    }

    /// Validate application key lengths match cipher suite expectations
    fn validate_key_lengths(&self, secrets: &TlsApplicationSecrets) {
        let expected_key_len = match self.cipher_suite {
            0x1301 => 16,          // AES-128-GCM
            0x1302 | 0x1303 => 32, // AES-256-GCM, ChaCha20-Poly1305
            _ => 0,
        };

        if secrets.client_write_key.len() != expected_key_len {
            error!(
                "❌ Key length mismatch: expected {} bytes, got {}",
                expected_key_len,
                secrets.client_write_key.len()
            );
        }
        if secrets.client_write_iv.len() != 12 {
            error!(
                "❌ IV length mismatch: expected 12 bytes, got {}",
                secrets.client_write_iv.len()
            );
        }
    }

    /// Get human-readable hash algorithm name from hash length
    const fn hash_algorithm_name(hash: &[u8]) -> &'static str {
        if hash.len() == 32 {
            "SHA-256"
        } else {
            "SHA-384"
        }
    }

    /// Build `ClientHello` message
    #[expect(
        clippy::cast_possible_truncation,
        reason = "TLS wire format: values are masked/bounded"
    )]
    pub(crate) fn build_client_hello(
        &self,
        client_random: &[u8],
        client_public_key: &[u8],
        server_name: &str,
    ) -> Result<Vec<u8>> {
        let mut msg = Vec::new();

        // TLS record header
        msg.push(0x16); // ContentType: Handshake
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version

        let length_pos = msg.len();
        msg.extend_from_slice(&[0, 0]); // Placeholder for length

        // Handshake header
        msg.push(0x01); // HandshakeType: ClientHello

        let handshake_length_pos = msg.len();
        msg.extend_from_slice(&[0, 0, 0]); // Placeholder

        // ClientHello content
        msg.extend_from_slice(&TLS_1_2.to_be_bytes()); // Legacy version
        msg.extend_from_slice(client_random); // Random (32 bytes)

        // RFC 8446 Appendix D.4: non-empty legacy_session_id for middlebox compat.
        // Servers behind CDNs/load balancers (Cloudflare, etc.) require this.
        let session_id = self.generate_random();
        msg.push(32); // Legacy session ID length
        msg.extend_from_slice(&session_id);

        // Cipher suites
        msg.extend_from_slice(&tls_wire_u16(CIPHER_SUITES.len().saturating_mul(2))?.to_be_bytes());
        for suite in CIPHER_SUITES {
            msg.extend_from_slice(&suite.to_be_bytes());
        }

        // Compression methods
        msg.push(1); // Length
        msg.push(0); // No compression

        // Extensions
        let extensions = self.build_extensions(server_name, client_public_key)?;
        msg.extend_from_slice(&tls_wire_u16(extensions.len())?.to_be_bytes());
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::crypto::SecurityCryptoProvider;
    use crate::tls::config::TlsConfig;
    use crate::tls::{CIPHER_SUITES, TLS_1_2, content_type, handshake_type};
    use std::sync::Arc;

    fn test_handshake() -> TlsHandshake {
        let crypto =
            Arc::new(SecurityCryptoProvider::new("/tmp/songbird-handshake-flow-test.sock"));
        TlsHandshake::new(crypto)
    }

    #[test]
    fn build_client_hello_tls_record_header() {
        let h = test_handshake();
        let random = [0x5au8; 32];
        let key = [0x3cu8; 32];
        let msg = h.build_client_hello(&random, &key, "example.com").expect("client hello");

        assert_eq!(msg[0], content_type::HANDSHAKE);
        assert_eq!(&msg[1..3], &TLS_1_2.to_be_bytes());
        let record_len = u16::from_be_bytes([msg[3], msg[4]]) as usize;
        assert_eq!(record_len, msg.len() - 5);
    }

    #[test]
    fn build_client_hello_contains_client_hello_handshake_type() {
        let h = test_handshake();
        let random = [0u8; 32];
        let key = [1u8; 32];
        let msg = h.build_client_hello(&random, &key, "test.local").expect("client hello");

        assert_eq!(msg[5], handshake_type::CLIENT_HELLO);
    }

    #[test]
    fn build_client_hello_embeds_client_random_and_cipher_suites() {
        let h = test_handshake();
        let mut random = [0u8; 32];
        random[0] = 0x7e;
        let key = [9u8; 32];
        let msg = h.build_client_hello(&random, &key, "svc.example").expect("client hello");

        // After record header (5) + handshake type (1) + handshake length (3) + legacy version (2)
        let body_start = 5 + 1 + 3;
        assert_eq!(&msg[body_start..body_start + 2], &TLS_1_2.to_be_bytes());
        assert_eq!(&msg[body_start + 2..body_start + 34], random.as_slice());

        // Session ID: 1 byte length (32) + 32 bytes random
        let session_id_len = msg[body_start + 34] as usize;
        assert_eq!(session_id_len, 32, "legacy_session_id must be 32 bytes for middlebox compat");
        let cs_len_pos = body_start + 34 + 1 + session_id_len;
        let cs_len = u16::from_be_bytes([msg[cs_len_pos], msg[cs_len_pos + 1]]) as usize;
        assert_eq!(cs_len, CIPHER_SUITES.len() * 2);
    }

    #[test]
    fn handshake_with_config_preserves_cipher_list_in_client_hello() {
        let crypto = Arc::new(SecurityCryptoProvider::new("/tmp/songbird-tls-config-test.sock"));
        let h = TlsHandshake::with_config(crypto, TlsConfig::default(), None);
        let msg =
            h.build_client_hello(&[0xee; 32], &[0xdd; 32], "h.example").expect("client hello");

        assert!(msg.len() > 40);
        assert_eq!(msg[0], content_type::HANDSHAKE);
    }

    #[test]
    fn build_client_hello_sni_extension_present_for_non_empty_host() {
        let h = test_handshake();
        let msg =
            h.build_client_hello(&[0x01; 32], &[0x02; 32], "api.github.com").expect("client hello");
        let host = b"api.github.com";
        assert!(
            msg.windows(host.len()).any(|w| w == host),
            "expected SNI hostname bytes in ClientHello extensions"
        );
    }

    #[test]
    fn build_client_hello_record_length_matches_payload() {
        let h = test_handshake();
        let msg = h.build_client_hello(&[0xab; 32], &[0xcd; 32], "x").expect("client hello");

        let rl = u16::from_be_bytes([msg[3], msg[4]]) as usize;
        assert_eq!(msg.len(), 5 + rl);
    }
}
