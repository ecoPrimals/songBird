//! TLS Transcript Tracking Module
//!
//! This module handles transcript tracking for TLS 1.3 handshakes.
//! The transcript is a concatenation of all plaintext handshake messages,
//! used to derive application traffic keys via HKDF.
//!
//! ## RFC 8446 Compliance
//!
//! From RFC 8446 Section 4.4.1:
//! > The transcript hash is computed as:
//! > `Hash(ClientHello || ServerHello || EncryptedExtensions || Certificate || CertificateVerify || Finished)`
//!
//! **CRITICAL**: Each message must be added individually, not as a blob!
//!
//! ## Reusability
//!
//! This module is designed to be reusable by BOTH TLS client and server:
//! - Client: Tracks messages during handshake
//! - Server: Tracks messages (same logic, reversed flow)
//!
//! ## Usage
//!
//! ```rust,ignore
//! // In client or server
//! let mut transcript = Transcript::new();
//! transcript.update(client_hello, "ClientHello", false);
//! transcript.update(server_hello, "ServerHello", false);
//! let hash = transcript.compute_hash();
//! ```

use sha2::{Digest, Sha256};
use tracing::{debug, error, info};

/// Transcript tracker for TLS 1.3 handshakes
///
/// Accumulates all plaintext handshake messages for key derivation.
#[derive(Debug, Clone)]
pub struct Transcript {
    /// Accumulated transcript bytes
    messages: Vec<u8>,
}

impl Transcript {
    /// Create a new empty transcript
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Get current transcript length
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if transcript is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get reference to transcript bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.messages
    }

    /// Update transcript with a new message (simple version)
    ///
    /// **CRITICAL**: Only add PLAINTEXT messages, not encrypted!
    /// **CRITICAL**: Strip TLS record header (5 bytes) before adding!
    /// **CRITICAL**: Strip ContentType byte (1 byte) after AEAD decryption!
    ///
    /// # Arguments
    /// * `message` - Plaintext handshake message (type + length + body)
    ///
    /// # What to Include
    /// - ClientHello: Already stripped by read_record()
    /// - ServerHello: Already stripped by read_record()
    /// - Post-handshake messages: Already stripped by read_record()
    #[allow(dead_code)]
    pub fn update(&mut self, message: &[u8]) {
        let before = self.messages.len();
        let after = before + message.len();
        self.messages.extend_from_slice(message);

        debug!("Transcript updated: {} → {} bytes (+{} bytes)", before, after, message.len());
    }

    /// Update transcript with comprehensive logging
    ///
    /// This enhanced version logs detailed information about each message
    /// to help diagnose transcript hash issues.
    ///
    /// # Arguments
    /// * `message` - Plaintext handshake message
    /// * `message_type` - Human-readable message type (e.g., "ClientHello")
    /// * `was_decrypted` - Whether this message was AEAD-decrypted
    pub fn update_with_logging(&mut self, message: &[u8], message_type: &str, was_decrypted: bool) {
        let before = self.messages.len();

        // Log comprehensive details
        info!("════════════════════════════════════════════════════════════");
        info!("📝 TRANSCRIPT UPDATE: {}", message_type);
        info!("════════════════════════════════════════════════════════════");
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
            info!(
                "First 32 bytes (hex): {}",
                hex::encode(&message[..std::cmp::min(32, message.len())])
            );
            if message.len() > 64 {
                info!(
                    "Last 32 bytes (hex): {}",
                    hex::encode(&message[message.len().saturating_sub(32)..])
                );
            }

            // 🚨 CRITICAL VALIDATION: Check for common errors
            if first_byte == 0x16 {
                error!("⚠️  CRITICAL: TLS record header (0x16) detected!");
                error!("   This message should have the 5-byte TLS record header stripped!");
                error!("   Expected first byte: handshake message type (0x01, 0x02, 0x08, 0x0B, 0x0F, 0x14)");
            } else if first_byte == 0x17 {
                error!("⚠️  CRITICAL: ContentType byte (0x17) detected!");
                error!("   This should be stripped after AEAD decryption!");
            }
        }

        self.messages.extend_from_slice(message);
        let after = self.messages.len();

        info!(
            "Cumulative transcript length: {} bytes → {} bytes (+{} bytes)",
            before,
            after,
            message.len()
        );
        info!("════════════════════════════════════════════════════════════");
        info!("");
    }

    /// Compute SHA-256 hash of transcript
    ///
    /// This is used for key derivation in TLS 1.3.
    ///
    /// # Returns
    /// SHA-256 hash (32 bytes)
    pub fn compute_hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.messages);
        hasher.finalize().to_vec()
    }

    /// Compute and log SHA-256 hash of transcript
    ///
    /// Includes comprehensive logging for debugging.
    pub fn compute_hash_with_logging(&self) -> Vec<u8> {
        let hash = self.compute_hash();

        info!("🔐 Transcript Hash Computation");
        info!("   Transcript length: {} bytes", self.messages.len());
        info!("   SHA-256 hash: {}", hex::encode(&hash));

        hash
    }

    /// Log complete transcript as hex dump
    ///
    /// For byte-level forensics and comparison with external tools
    /// (Wireshark, OpenSSL, etc.)
    ///
    /// **Format**: 64 bytes per line with offset
    pub fn log_hex_dump(&self) {
        info!("════════════════════════════════════════════════════════════");
        info!("🔬 COMPLETE TRANSCRIPT HEX DUMP (BYTE-LEVEL FORENSICS)");
        info!("════════════════════════════════════════════════════════════");
        info!("Total transcript length: {} bytes", self.messages.len());
        info!("");
        info!("📝 Full transcript (hex):");
        for (i, chunk) in self.messages.chunks(64).enumerate() {
            info!("{:04x}: {}", i * 64, hex::encode(chunk));
        }
        info!("════════════════════════════════════════════════════════════");
        info!("");
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
    fn test_transcript_new() {
        let transcript = Transcript::new();
        assert_eq!(transcript.len(), 0);
        assert!(transcript.is_empty());
    }

    #[test]
    fn test_transcript_update() {
        let mut transcript = Transcript::new();

        transcript.update(b"ClientHello");
        assert_eq!(transcript.len(), 11);

        transcript.update(b"ServerHello");
        assert_eq!(transcript.len(), 22);
    }

    #[test]
    fn test_transcript_hash() {
        let mut transcript = Transcript::new();
        transcript.update(b"test message");

        let hash = transcript.compute_hash();
        assert_eq!(hash.len(), 32); // SHA-256 is 32 bytes
    }

    #[test]
    fn test_transcript_deterministic() {
        let mut t1 = Transcript::new();
        t1.update(b"message1");
        t1.update(b"message2");

        let mut t2 = Transcript::new();
        t2.update(b"message1");
        t2.update(b"message2");

        assert_eq!(t1.compute_hash(), t2.compute_hash());
    }
}
