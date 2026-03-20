// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Post-handshake message processing
//!
//! RFC 8446 Section 4.6: After the handshake completes, the server may send
//! `NewSessionTicket` messages and other post-handshake content. This module
//! reads and decrypts those messages to keep the stream in sync for HTTP.

use super::core::TlsHandshake;
use crate::crypto::TlsApplicationSecrets;
use crate::error::{Error, Result};
use crate::tls::alert::TlsAlert;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tracing::{error, info, trace, warn};

/// Post-handshake message processing result
pub struct PostHandshakeResult {
    /// Number of post-handshake messages consumed
    pub _message_count: u32,
    /// Final read sequence number (for session key initialization)
    pub read_sequence_number: u64,
}

impl TlsHandshake {
    /// Consume all post-handshake messages (`NewSessionTicket`, etc.)
    ///
    /// RFC 8446 Section 4.6: Server MAY send multiple `NewSessionTicket` messages
    /// after the handshake. We must read and decrypt them to avoid stream desync
    /// when sending HTTP requests.
    pub(crate) async fn consume_post_handshake_messages(
        &self,
        stream: &mut TcpStream,
        secrets: &TlsApplicationSecrets,
    ) -> PostHandshakeResult {
        info!("Step 13: Reading post-handshake messages (NewSessionTicket, etc.)");

        let mut count = 0u32;
        let mut read_seq: u64 = 0;

        loop {
            match timeout(Duration::from_millis(200), self.read_record(stream)).await {
                Ok(Ok((content_type, encrypted_data))) => {
                    count += 1;
                    trace!(
                        "Post-handshake #{}: type=0x{:02x}, {} bytes",
                        count,
                        content_type,
                        encrypted_data.len()
                    );

                    match content_type {
                        0x17 => {
                            // Encrypted post-handshake message
                            match self
                                .decrypt_post_handshake_record(&encrypted_data, secrets, read_seq)
                                .await
                            {
                                Ok(inner_type) => {
                                    read_seq += 1;
                                    if inner_type == PostHandshakeType::Alert {
                                        // Alert was already logged in decrypt method
                                        break;
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to decrypt post-handshake #{}: {}", count, e);
                                }
                            }
                        }
                        0x15 => {
                            // Unencrypted alert (unusual for TLS 1.3 after handshake)
                            warn!("Unencrypted TLS alert post-handshake!");
                            if let Ok(alert) = TlsAlert::parse(&encrypted_data) {
                                error!("🚨 Alert: {}", alert.to_detailed_string());
                            }
                            break;
                        }
                        _ => {
                            trace!("Ignoring unexpected type 0x{:02x}", content_type);
                        }
                    }
                }
                Ok(Err(e)) => {
                    if count == 0 {
                        warn!("Error reading first post-handshake message: {}", e);
                    }
                    break;
                }
                Err(_) => {
                    // Timeout - normal end of post-handshake messages
                    break;
                }
            }
        }

        info!("✅ Consumed {} post-handshake messages - stream ready for HTTP", count);

        PostHandshakeResult {
            _message_count: count,
            read_sequence_number: read_seq,
        }
    }

    /// Decrypt a single post-handshake record and identify its type
    async fn decrypt_post_handshake_record(
        &self,
        encrypted_data: &[u8],
        secrets: &TlsApplicationSecrets,
        read_seq: u64,
    ) -> Result<PostHandshakeType> {
        // Build nonce: server_write_iv XOR read_sequence_number
        let mut nonce = secrets.server_write_iv.clone();
        let seq_bytes = read_seq.to_be_bytes();
        for i in 0..8 {
            nonce[4 + i] ^= seq_bytes[i];
        }

        // Build AAD (record header)
        let aad = [
            0x17,
            0x03,
            0x03,
            u8::try_from(encrypted_data.len() >> 8).expect("length byte fits in u8"),
            u8::try_from(encrypted_data.len() & 0xFF).expect("length byte fits in u8"),
        ];

        // Decrypt based on cipher suite
        let plaintext = match self.cipher_suite {
            0x1301 => {
                self.crypto
                    .aes128_gcm_decrypt(&secrets.server_write_key, &nonce, encrypted_data, &aad)
                    .await
            }
            0x1302 => {
                self.crypto
                    .aes256_gcm_decrypt(&secrets.server_write_key, &nonce, encrypted_data, &aad)
                    .await
            }
            _ => self.crypto.decrypt(&secrets.server_write_key, &nonce, encrypted_data, &aad).await,
        }
        .map_err(|e| Error::TlsHandshake(format!("Post-handshake decryption failed: {e}")))?;

        // RFC 8446: TLSInnerPlaintext has ContentType as last non-zero byte
        if let Some(&inner_type) = plaintext.last() {
            let content = &plaintext[..plaintext.len() - 1];

            match inner_type {
                0x16 => {
                    // Handshake (e.g., NewSessionTicket type 0x04)
                    if !content.is_empty() && content[0] == 0x04 {
                        trace!("🎟️  NewSessionTicket (ignored)");
                    }
                    Ok(PostHandshakeType::Handshake)
                }
                0x15 => {
                    // Alert inside encrypted envelope
                    use crate::tls::alert::TlsAlert;
                    if content.len() >= 2
                        && let Ok(alert) = TlsAlert::parse(content)
                    {
                        error!("🚨 Encrypted post-handshake alert: {}", alert.to_detailed_string());
                        return Err(Error::TlsHandshake(format!(
                            "Server sent encrypted alert: {}",
                            alert.to_detailed_string()
                        )));
                    }
                    Ok(PostHandshakeType::Alert)
                }
                _ => {
                    trace!("Unknown inner type: 0x{:02x}", inner_type);
                    Ok(PostHandshakeType::Unknown)
                }
            }
        } else {
            Ok(PostHandshakeType::Unknown)
        }
    }
}

/// Types of post-handshake messages
#[derive(Debug, PartialEq, Eq)]
pub enum PostHandshakeType {
    /// Handshake message (e.g., `NewSessionTicket`)
    Handshake,
    /// TLS Alert
    Alert,
    /// Unknown message type
    Unknown,
}
