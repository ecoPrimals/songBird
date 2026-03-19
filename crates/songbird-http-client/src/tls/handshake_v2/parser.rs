//! TLS Handshake Message Parser Module
//!
//! This module handles parsing of RFC 8446 handshake messages from decrypted TLS records.
//! In TLS 1.3, multiple handshake messages can be coalesced into a single TLS record.
//!
//! ## RFC 8446 Compliance
//!
//! From RFC 8446 Section 5.1:
//! > "Multiple handshake messages MAY be coalesced into a single TLSPlaintext record"
//!
//! From RFC 8446 Section 4:
//! ```text
//! struct {
//!     HandshakeType msg_type;    // 1 byte
//!     uint24 length;             // 3 bytes (big-endian)
//!     opaque body<0..2^24-1>;   // variable
//! } Handshake;
//! ```
//!
//! ## Reusability
//!
//! This module is designed to be reusable by BOTH TLS client and server:
//! - Client: Parses server handshake messages (`EncryptedExtensions`, Certificate, etc.)
//! - Server: Parses client handshake messages (`ClientHello`, etc.)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let decrypted_record = decrypt_tls_record(&encrypted)?;
//! let messages = parse_handshake_messages(&decrypted_record)?;
//!
//! for (msg_type, msg_data) in messages {
//!     match msg_type {
//!         0x08 => handle_encrypted_extensions(&msg_data)?,
//!         0x0B => handle_certificate(&msg_data)?,
//!         // ...
//!     }
//! }
//! ```

use crate::error::Result;
use tracing::{debug, error, info, warn};

/// Parsed handshake message
///
/// Contains the complete message including type (1 byte) + length (3 bytes) + body.
#[derive(Debug, Clone)]
pub struct HandshakeMessage {
    /// Message type (RFC 8446 Section 4)
    pub msg_type: u8,

    /// Message length (body only, not including type + length header)
    pub length: usize,

    /// Complete message data (type + length + body)
    /// This is what should be added to the transcript.
    pub data: Vec<u8>,
}

impl HandshakeMessage {
    /// Get message type name
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self.msg_type {
            0x01 => "ClientHello",
            0x02 => "ServerHello",
            0x08 => "EncryptedExtensions",
            0x0B => "Certificate",
            0x0F => "CertificateVerify",
            0x14 => "Finished",
            0x04 => "NewSessionTicket",
            0x18 => "KeyUpdate",
            _ => "Unknown",
        }
    }

    /// Get message body (without type and length header)
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.data[4..] // Skip type (1) + length (3)
    }
}

/// Parse multiple handshake messages from a decrypted TLS record
///
/// In TLS 1.3, the server typically sends multiple handshake messages
/// coalesced into a single TLS record (e.g., `EncryptedExtensions`, Certificate,
/// `CertificateVerify`, and Finished all in one record).
///
/// This function parses the record according to RFC 8446 Section 4 framing:
/// - Type: 1 byte
/// - Length: 3 bytes (big-endian, body length only)
/// - Body: `length` bytes
///
/// # Arguments
/// * `data` - Decrypted TLS record containing one or more handshake messages
///
/// # Returns
/// * `Ok(Vec<HandshakeMessage>)` - Parsed messages (in order)
/// * `Err` - If parsing fails
///
/// # Example
/// ```rust,ignore
/// let decrypted = decrypt_record(&encrypted)?;
/// let messages = parse_handshake_messages(&decrypted)?;
///
/// for msg in messages {
///     println!("Message: {} ({} bytes)", msg.type_name(), msg.data.len());
/// }
/// ```
pub fn parse_handshake_messages(data: &[u8]) -> Result<Vec<HandshakeMessage>> {
    let mut messages = Vec::new();
    let mut offset = 0;

    info!("════════════════════════════════════════════════════════════");
    info!("📦 PARSING HANDSHAKE MESSAGES FROM DECRYPTED RECORD");
    info!("════════════════════════════════════════════════════════════");
    info!("Total decrypted data: {} bytes", data.len());
    info!("Parsing individual RFC 8446 handshake messages...");
    info!("");

    // 🔍 HEX DUMP: Show first/last bytes to identify extra bytes
    info!("🔍 HEX DUMP OF DECRYPTED DATA:");
    info!("   First 64 bytes: {}", hex::encode(&data[..std::cmp::min(64, data.len())]));
    if data.len() > 128 {
        info!("   ... ({} bytes in middle) ...", data.len() - 128);
    }
    if data.len() > 64 {
        info!("   Last 64 bytes: {}", hex::encode(&data[data.len().saturating_sub(64)..]));
    }
    info!("");

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
        let message_data = data[message_start..offset + length].to_vec();

        let msg = HandshakeMessage {
            msg_type,
            length,
            data: message_data,
        };

        info!(
            "✅ Parsed message #{}: {} (type 0x{:02x}, length {} bytes, total {} bytes)",
            messages.len() + 1,
            msg.type_name(),
            msg_type,
            length,
            msg.data.len()
        );
        info!("   Message offset: {} to {} (in decrypted blob)", message_start, offset + length);
        info!(
            "   First 32 bytes of message: {}",
            hex::encode(&msg.data[..std::cmp::min(32, msg.data.len())])
        );

        messages.push(msg);
        offset += length;
    }

    info!("");
    info!("📋 Parsing complete:");
    info!("   Total messages parsed: {}", messages.len());
    info!("   Bytes consumed: {} out of {} bytes", offset, data.len());

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

    info!("════════════════════════════════════════════════════════════");
    info!("");

    if messages.is_empty() {
        warn!("⚠️  No handshake messages parsed from {} bytes of data!", data.len());
    }

    Ok(messages)
}

/// Parse a single handshake message
///
/// Use this when you know there's only one message in the data.
///
/// # Arguments
/// * `data` - Handshake message data (type + length + body)
///
/// # Returns
/// * `Ok(HandshakeMessage)` - Parsed message
/// * `Err` - If parsing fails or multiple messages found
pub fn parse_single_handshake_message(data: &[u8]) -> Result<HandshakeMessage> {
    let messages = parse_handshake_messages(data)?;

    match messages.len() {
        1 => {
            // Safe: We just verified length is exactly 1
            Ok(messages.into_iter().next().expect("BUG: messages.len() == 1 but no first element"))
        }
        n => Err(crate::error::Error::TlsHandshake(format!(
            "Expected 1 handshake message, found {n}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_message() {
        // Handshake message: type 0x14 (Finished), length 0x000020 (32 bytes)
        let mut data = vec![0x14, 0x00, 0x00, 0x20];
        data.extend_from_slice(&[0u8; 32]); // 32 bytes of body

        let messages = parse_handshake_messages(&data).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].msg_type, 0x14);
        assert_eq!(messages[0].length, 32);
        assert_eq!(messages[0].data.len(), 36); // 4 header + 32 body
    }

    #[test]
    fn test_parse_multiple_messages() {
        let mut data = Vec::new();

        // Message 1: type 0x08, length 10
        data.extend_from_slice(&[0x08, 0x00, 0x00, 0x0A]);
        data.extend_from_slice(&[1u8; 10]);

        // Message 2: type 0x14, length 32
        data.extend_from_slice(&[0x14, 0x00, 0x00, 0x20]);
        data.extend_from_slice(&[2u8; 32]);

        let messages = parse_handshake_messages(&data).unwrap();
        assert_eq!(messages.len(), 2);

        assert_eq!(messages[0].msg_type, 0x08);
        assert_eq!(messages[0].length, 10);

        assert_eq!(messages[1].msg_type, 0x14);
        assert_eq!(messages[1].length, 32);
    }

    #[test]
    fn test_message_type_name() {
        let msg = HandshakeMessage {
            msg_type: 0x14,
            length: 32,
            data: vec![0x14, 0x00, 0x00, 0x20],
        };

        assert_eq!(msg.type_name(), "Finished");
    }

    #[test]
    fn test_message_body() {
        let mut data = vec![0x14, 0x00, 0x00, 0x04];
        data.extend_from_slice(b"test");

        let msg = HandshakeMessage {
            msg_type: 0x14,
            length: 4,
            data: data.clone(),
        };

        assert_eq!(msg.body(), b"test");
    }

    #[test]
    fn test_parse_with_invalid_type() {
        // Invalid message type 0xFF
        let data = vec![0xFF, 0x00, 0x00, 0x10];

        let messages = parse_handshake_messages(&data).unwrap();
        // Should stop parsing at invalid type
        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_parse_truncated_message() {
        // Message says it's 100 bytes but only 10 bytes provided
        let mut data = vec![0x14, 0x00, 0x00, 0x64]; // length = 100
        data.extend_from_slice(&[0u8; 10]); // only 10 bytes

        let messages = parse_handshake_messages(&data).unwrap();
        // Should stop at truncated message
        assert_eq!(messages.len(), 0);
    }
}
