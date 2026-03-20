// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Transcript hash management for TLS 1.3 handshake
//!
//! RFC 8446 Section 4.4.1: The transcript hash is computed over all handshake messages.
//! CRITICAL: Transcript must contain PLAINTEXT messages, not encrypted versions.

use sha2::{Digest, Sha256};
use tracing::{debug, error, info, trace};

/// Transcript accumulator for RFC 8446 key derivation
///
/// Accumulates all handshake messages for transcript hash computation.
/// The transcript hash is used in key derivation and Finished message verification.
#[derive(Debug, Clone)]
pub struct Transcript {
    /// Accumulated handshake messages (plaintext only)
    messages: Vec<u8>,
}

impl Transcript {
    /// Create a new empty transcript
    #[must_use]
    pub const fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Get current transcript size in bytes
    #[must_use]
    pub const fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if transcript is empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Update transcript with a handshake message
    ///
    /// RFC 8446 Section 4.4.1: Transcript hash includes all handshake messages
    ///
    /// CRITICAL: This method expects handshake messages WITHOUT TLS record framing!
    /// - `ClientHello`: Must strip 5-byte TLS record header before calling
    /// - `ServerHello`: Already stripped by `read_record()`
    /// - Post-handshake messages: Already stripped by `read_record()`
    pub fn update(&mut self, message: &[u8]) {
        let before = self.messages.len();
        let after = before + message.len();
        trace!(
            "📝 Updating transcript: +{} bytes (total: {} → {} bytes)",
            message.len(),
            before,
            after
        );
        trace!("   Message preview: {:02x?}", &message[..std::cmp::min(16, message.len())]);
        self.messages.extend_from_slice(message);
    }

    /// Update transcript with comprehensive logging for debugging
    ///
    /// This enhanced version logs detailed information about each message
    /// to help diagnose transcript hash issues
    pub fn update_with_logging(&mut self, message: &[u8], message_type: &str, was_decrypted: bool) {
        let before = self.messages.len();

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

            // Enhanced hex dump: Show first/last bytes
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

            // Check length field in message (bytes 1-3 for handshake messages)
            if message.len() >= 4 {
                let declared_length =
                    u32::from_be_bytes([0, message[1], message[2], message[3]]) as usize;
                let actual_length = message.len() - 4; // Minus type (1) + length (3)
                info!("📏 Length validation:");
                trace!("   Declared length (bytes 1-3): {} bytes", declared_length);
                trace!("   Actual body length: {} bytes", actual_length);
                if declared_length != actual_length {
                    error!("🚨 LENGTH MISMATCH!");
                    error!("   Declared: {} bytes", declared_length);
                    error!("   Actual: {} bytes", actual_length);
                    error!(
                        "   Difference: {} bytes",
                        (actual_length as i64 - declared_length as i64).abs()
                    );
                    error!("   💡 This might be the source of transcript hash issues!");
                }
            }
        }

        // Add to transcript
        self.messages.extend_from_slice(message);

        let after = self.messages.len();
        info!("✅ Transcript updated: {} → {} bytes", before, after);
        debug!("Current transcript hash: {}", hex::encode(self.compute_hash()));
    }

    /// Compute transcript hash (SHA-256)
    ///
    /// RFC 8446 Section 4.4.1:
    /// ```text
    /// Transcript-Hash(M1, M2, ... Mn) = Hash(M1 || M2 || ... || Mn)
    /// ```
    #[must_use]
    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.messages);
        hasher.finalize().to_vec()
    }

    /// Get reference to accumulated messages (for testing)
    #[cfg(test)]
    pub fn messages(&self) -> &[u8] {
        &self.messages
    }
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_empty_initially() {
        let transcript = Transcript::new();
        assert!(transcript.is_empty());
        assert_eq!(transcript.len(), 0);
    }

    #[test]
    fn test_update_transcript() {
        let mut transcript = Transcript::new();
        let message = b"test message";

        transcript.update(message);

        assert_eq!(transcript.len(), message.len());
        assert_eq!(transcript.messages(), message);
    }

    #[test]
    fn test_compute_transcript_hash_empty() {
        let transcript = Transcript::new();
        let hash = transcript.compute_hash();

        // SHA-256 of empty string:
        // echo -n "" | sha256sum
        let expected =
            hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .expect("Valid hex");

        assert_eq!(hash, expected);
    }

    #[test]
    fn test_compute_transcript_hash_deterministic() {
        let mut t1 = Transcript::new();
        let mut t2 = Transcript::new();

        t1.update(b"test");
        t2.update(b"test");

        assert_eq!(t1.compute_hash(), t2.compute_hash());
    }

    #[test]
    fn test_compute_transcript_hash_known_value() {
        let mut transcript = Transcript::new();
        transcript.update(b"test");

        let hash = transcript.compute_hash();

        // SHA-256 of "test"
        // echo -n "test" | sha256sum
        let expected_hash =
            hex::decode("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")
                .expect("Valid hex");

        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_transcript_accumulates_multiple_messages() {
        let mut transcript = Transcript::new();

        let client_hello = vec![1u8; 100];
        let server_hello = vec![2u8; 100];
        let encrypted_extensions = vec![3u8; 50];
        let certificate = vec![4u8; 200];
        let finished = vec![5u8; 50];

        transcript.update(&client_hello);
        transcript.update(&server_hello);
        transcript.update(&encrypted_extensions);
        transcript.update(&certificate);
        transcript.update(&finished);

        let expected_total = 100 + 100 + 50 + 200 + 50;
        assert_eq!(transcript.len(), expected_total);

        let hash = transcript.compute_hash();
        assert_eq!(hash.len(), 32, "SHA-256 hash should always be 32 bytes");
    }

    #[test]
    fn test_transcript_order_matters() {
        let mut t1 = Transcript::new();
        let mut t2 = Transcript::new();

        // Add messages in different orders
        t1.update(b"A");
        t1.update(b"B");

        t2.update(b"B");
        t2.update(b"A");

        let hash1 = t1.compute_hash();
        let hash2 = t2.compute_hash();

        // Hashes should be different (order matters!)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_transcript_hash_length() {
        let mut transcript = Transcript::new();

        // Add various sized messages
        for size in [1, 10, 100, 1000, 10000] {
            transcript.update(&vec![0xFF; size]);
        }

        // Hash should always be 32 bytes regardless of input size
        let hash = transcript.compute_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_transcript_plaintext_vs_encrypted() {
        let mut t_plain = Transcript::new();
        let mut t_encrypted = Transcript::new();

        let plaintext = b"PLAINTEXT_MESSAGE";
        let encrypted = b"ENCRYPTED_VERSION_OF_SAME_MESSAGE_WITH_TAG";

        t_plain.update(plaintext);
        t_encrypted.update(encrypted);

        let hash_plain = t_plain.compute_hash();
        let hash_encrypted = t_encrypted.compute_hash();

        // RFC 8446: Transcript hash of plaintext must differ from encrypted version!
        assert_ne!(hash_plain, hash_encrypted);
    }
}
