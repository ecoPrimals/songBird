// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Transcript management for TLS 1.3 handshake
//!
//! RFC 8446 Section 4.4.1: The transcript hash is computed by hashing the
//! concatenation of all handshake messages (in plaintext) up to that point.

use super::core::TlsHandshake;
use crate::crypto::CryptoCapability;
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, trace, warn};

impl TlsHandshake {
    /// Update transcript with handshake message
    /// RFC 8446 Section 4.4.1: Transcript hash includes all handshake messages
    ///
    /// CRITICAL: This method expects handshake messages WITHOUT TLS record framing!
    /// - `ClientHello`: Must strip 5-byte TLS record header before calling
    /// - `ServerHello`: Already stripped by `read_record()`
    /// - Post-handshake messages: Already stripped by `read_record()`
    #[allow(
        dead_code,
        reason = "sync transcript helper; production path uses update_transcript_with_logging"
    )]
    pub(super) fn update_transcript(&mut self, message: &[u8]) {
        let before = self.transcript.len();
        let after = before + message.len();
        trace!(
            "📝 Updating transcript: +{} bytes (total: {} → {} bytes)",
            message.len(),
            before,
            after
        );
        trace!("   Message preview: {:02x?}", &message[..std::cmp::min(16, message.len())]);
        self.transcript.extend_from_slice(message);
    }

    /// Update transcript with comprehensive logging for debugging
    ///
    /// This enhanced version logs detailed information about each message
    /// to help diagnose transcript hash issues (biomeOS v5.12.6 investigation)
    pub(super) fn update_transcript_with_logging(
        &mut self,
        message: &[u8],
        message_type: &str,
        was_decrypted: bool,
    ) {
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
            info!(
                "First byte: 0x{:02x} ({})",
                first_byte,
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
                }
            );

            // 🔍 ENHANCED HEX DUMP: Show first/last bytes to identify extra bytes
            trace!(
                "First 32 bytes (hex): {}",
                hex::encode(&message[..std::cmp::min(32, message.len())])
            );
            if message.len() > 64 {
                trace!(
                    "Last 32 bytes (hex): {}",
                    hex::encode(&message[message.len().saturating_sub(32)..])
                );
            }

            // 🔍 CHECK: Length field in message (bytes 1-3 for handshake messages)
            if message.len() >= 4 {
                let declared_length =
                    u32::from_be_bytes([0, message[1], message[2], message[3]]) as usize;
                let actual_length = message.len() - 4; // Minus type (1) + length (3)
                info!("📏 Length validation:");
                trace!("   Declared length (bytes 1-3): {} bytes", declared_length);
                trace!("   Actual body length: {} bytes", actual_length);
                if declared_length == actual_length {
                    trace!("   ✅ Length match - message is correct size");
                } else {
                    error!("🚨 LENGTH MISMATCH!");
                    error!("   Declared: {} bytes", declared_length);
                    error!("   Actual: {} bytes", actual_length);
                    #[expect(
                        clippy::cast_possible_wrap,
                        reason = "intentional pattern; clippy false positive for this API"
                    )] // Handshake message lengths are < 16MB
                    let diff = (actual_length as i64 - declared_length as i64).abs();
                    error!("   Difference: {} bytes", diff);
                    error!("   💡 This might be the source of the 2-byte discrepancy!");
                }
            }

            // Warn if TLS record header or ContentType byte detected
            if first_byte == 0x16 {
                error!("⚠️  CRITICAL: TLS record header (0x16) detected!");
                error!("   This message should have the 5-byte TLS record header stripped!");
                error!(
                    "   Expected first byte: handshake message type (0x01, 0x02, 0x08, 0x0B, 0x0F, 0x14)"
                );
            } else if first_byte == 0x17 {
                error!("⚠️  CRITICAL: ContentType byte (0x17) detected!");
                error!("   This should be stripped after AEAD decryption!");
            }
        }

        // Add to transcript
        self.transcript.extend_from_slice(message);
        let after = self.transcript.len();

        info!(
            "Cumulative transcript length: {} bytes → {} bytes (+{} bytes)",
            before,
            after,
            message.len()
        );
        trace!("════════════════════════════════════════════════════════════");
        info!("");
    }

    /// Parse multiple handshake messages from a decrypted TLS record
    ///
    /// RFC 8446 Section 4: Handshake messages have the format:
    /// - `HandshakeType` `msg_type` (1 byte)
    /// - uint24 length (3 bytes, big-endian)
    /// - opaque body[length]
    ///
    /// A single TLS record may contain MULTIPLE handshake messages concatenated together!
    /// This function parses them individually so they can be added to the transcript separately.
    #[expect(clippy::unused_self, reason = "unused bindings/imports in this compilation unit")] // API consistency
    #[expect(
        clippy::too_many_lines,
        reason = "intentional pattern; clippy false positive for this API"
    )] // Handshake parsing has many validation branches
    pub(super) fn parse_handshake_messages(&self, data: &[u8]) -> Vec<(u8, Vec<u8>)> {
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
                warn!(
                    "⚠️  Stopping parse: invalid message type 0x{:02x} at offset {}",
                    msg_type, offset
                );
                warn!("   This might be padding or extra bytes!");
                warn!(
                    "   Remaining {} bytes: {}",
                    data.len() - offset,
                    hex::encode(&data[offset..std::cmp::min(offset + 32, data.len())])
                );
                break;
            }

            offset += 1;

            // Read length (3 bytes, big-endian)
            if offset + 3 > data.len() {
                warn!(
                    "⚠️  Truncated handshake message: not enough bytes for length at offset {}",
                    offset
                );
                break;
            }
            let length =
                u32::from_be_bytes([0, data[offset], data[offset + 1], data[offset + 2]]) as usize;
            offset += 3;

            // Read body
            if offset + length > data.len() {
                warn!(
                    "⚠️  Truncated handshake message: expected {} bytes, got {} at offset {}",
                    length,
                    data.len() - offset,
                    offset
                );
                break;
            }

            // Extract complete message (type + length + body)
            let message_start = offset - 4; // Go back to include type (1) + length (3)
            let full_message = &data[message_start..offset + length];

            let msg_name = match msg_type {
                0x08 => "EncryptedExtensions",
                0x0B => "Certificate",
                0x0F => "CertificateVerify",
                0x14 => "Finished",
                _ => "Unknown",
            };

            info!(
                "✅ Parsed message #{}: {} (type 0x{:02x}, length {} bytes, total {} bytes)",
                messages.len() + 1,
                msg_name,
                msg_type,
                length,
                full_message.len()
            );
            trace!(
                "   Message offset: {} to {} (in decrypted blob)",
                message_start,
                offset + length
            );
            trace!(
                "   First 32 bytes of message: {}",
                hex::encode(&full_message[..std::cmp::min(32, full_message.len())])
            );

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

        messages
    }

    /// Compute transcript hash (SHA-256) - legacy/sync version
    /// RFC 8446 Section 4.4.1: Transcript-Hash(M1, M2, ... Mn) = Hash(M1 || M2 || ... || Mn)
    ///
    /// NOTE: This uses local SHA-256 only. For cipher-aware hashing (SHA-384 for 0x1302),
    /// use `compute_transcript_hash_for_cipher` instead.
    #[allow(dead_code, reason = "SHA-256-only helper kept for tests and legacy callers")]
    pub(super) fn compute_transcript_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.transcript);
        let hash = hasher.finalize().to_vec();
        info!(
            "🔐 Computed transcript hash: {} bytes from {} bytes of messages",
            hash.len(),
            self.transcript.len()
        );
        trace!("   Transcript hash (hex): {}", hex::encode(&hash));
        hash
    }

    /// Compute cipher-aware transcript hash via the `crypto provider`
    ///
    /// RFC 8446 Section 4.4.1: Different cipher suites use different hash algorithms:
    /// - 0x1301 (`TLS_AES_128_GCM_SHA256)`: SHA-256 (32 bytes)
    /// - 0x1302 (`TLS_AES_256_GCM_SHA384)`: SHA-384 (48 bytes)
    /// - 0x1303 (`TLS_CHACHA20_POLY1305_SHA256)`: SHA-256 (32 bytes)
    ///
    /// This method delegates to the provider's `crypto.hash_for_cipher` capability.
    pub(super) async fn compute_transcript_hash_for_cipher(
        &self,
        cipher_suite: u16,
    ) -> crate::error::Result<Vec<u8>> {
        info!(
            "🔐 Computing cipher-aware transcript hash for 0x{:04x} from {} bytes",
            cipher_suite,
            self.transcript.len()
        );

        let hash = self.crypto.hash_for_cipher(&self.transcript, cipher_suite).await?;

        info!(
            "   → Hash: {} bytes ({})",
            hash.len(),
            if hash.len() == 32 {
                "SHA-256"
            } else if hash.len() == 48 {
                "SHA-384"
            } else {
                "Unknown"
            }
        );
        trace!("   Hash (hex): {}", hex::encode(&hash));

        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_transcript_empty_initially() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let handshake = TlsHandshake::new(crypto);

        // Transcript should be empty initially
        assert_eq!(handshake.transcript.len(), 0, "Transcript should start empty");
    }

    #[test]
    fn test_update_transcript() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

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
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let handshake = TlsHandshake::new(crypto);

        let hash = handshake.compute_transcript_hash();

        // SHA-256 hash of empty input
        // echo -n "" | sha256sum
        let expected_empty_hash =
            hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .expect("Valid hex");

        assert_eq!(hash.len(), 32, "SHA-256 hash should be 32 bytes");
        assert_eq!(hash, expected_empty_hash, "Empty transcript should match SHA-256(\"\")");
    }

    #[test]
    fn test_compute_transcript_hash_deterministic() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

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
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

        // Use a known message
        let message = b"test";
        handshake.update_transcript(message);

        let hash = handshake.compute_transcript_hash();

        // SHA-256 of "test"
        // echo -n "test" | sha256sum
        let expected_hash =
            hex::decode("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")
                .expect("Valid hex");

        assert_eq!(hash, expected_hash, "Transcript hash should match SHA-256(\"test\")");
    }

    #[test]
    fn test_transcript_accumulates_multiple_messages() {
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

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
        let crypto1 = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake1 = TlsHandshake::new(crypto1);

        let crypto2 = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake2 = TlsHandshake::new(crypto2);

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
        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

        // Add various sized messages
        for size in [1, 10, 100, 1000, 10000] {
            handshake.update_transcript(&vec![0xFF; size]);
        }

        // Hash should always be 32 bytes regardless of input size
        let hash = handshake.compute_transcript_hash();
        assert_eq!(hash.len(), 32, "SHA-256 hash should always be 32 bytes");
    }

    #[test]
    fn test_transcript_plaintext_requirement() {
        // RFC 8446 Section 4.4.1: Transcript must contain PLAINTEXT messages
        // This test ensures we understand the requirement

        let crypto = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake = TlsHandshake::new(crypto);

        // Simulate plaintext messages (what SHOULD be in transcript)
        let plaintext_message = b"This is plaintext handshake message";
        handshake.update_transcript(plaintext_message);

        // Compute hash of plaintext
        let plaintext_hash = handshake.compute_transcript_hash();

        // Create new handshake with encrypted version (what SHOULD NOT be in transcript)
        let crypto2 = std::sync::Arc::new(crate::crypto::SecurityCryptoProvider::new(
            "/tmp/security-provider.sock",
        ));
        let mut handshake2 = TlsHandshake::new(crypto2);
        let encrypted_message = b"ENCRYPTED_VERSION_OF_SAME_MESSAGE_WITH_TAG";
        handshake2.update_transcript(encrypted_message);

        // Compute hash of encrypted
        let encrypted_hash = handshake2.compute_transcript_hash();

        // Hashes MUST be different (plaintext vs encrypted)
        assert_ne!(
            plaintext_hash, encrypted_hash,
            "RFC 8446: Transcript hash of plaintext must differ from encrypted version!"
        );
    }
}
