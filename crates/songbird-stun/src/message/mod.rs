// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! STUN message encoding/decoding (RFC 5389)
//!
//! **Pure Rust Implementation | Zero Unsafe Code**
//!
//! STUN messages are binary-encoded with:
//! - 20-byte header (type, length, magic cookie, transaction ID)
//! - Variable attributes (type-length-value)
//!
//! ## Message Format (RFC 5389)
//!
//! ```text
//!  0                   1                   2                   3
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |0 0|     STUN Message Type     |         Message Length        |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                         Magic Cookie                          |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! |                                                               |
//! |                     Transaction ID (96 bits)                  |
//! |                                                               |
//! +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//! ```

mod attributes;
mod types;

pub use attributes::StunAttribute;
pub use types::{AttributeType, MessageType};

use crate::error::{StunError, StunResult};
use bytes::{Buf, BufMut, BytesMut};
use std::net::SocketAddr;

/// STUN magic cookie (RFC 5389)
pub const MAGIC_COOKIE: u32 = 0x2112_A442;

/// STUN message
#[derive(Debug, Clone)]
pub struct StunMessage {
    /// Message type
    pub message_type: MessageType,

    /// Transaction ID (96 bits = 12 bytes)
    pub transaction_id: [u8; 12],

    /// Attributes
    pub attributes: Vec<StunAttribute>,
}

impl StunMessage {
    /// Create a new STUN binding request
    #[must_use]
    pub fn new_binding_request() -> Self {
        // Generate random transaction ID
        let mut transaction_id = [0u8; 12];
        for byte in &mut transaction_id {
            *byte = rand::random();
        }

        Self {
            message_type: MessageType::BindingRequest,
            transaction_id,
            attributes: Vec::new(),
        }
    }

    /// Encode message to bytes
    #[must_use]
    pub fn encode(&self) -> bytes::Bytes {
        let mut buf = BytesMut::new();

        // Header: Message Type (2 bytes)
        buf.put_u16(self.message_type.to_u16());

        // Header: Message Length (2 bytes) - will be filled later
        let length_offset = buf.len();
        buf.put_u16(0);

        // Header: Magic Cookie (4 bytes)
        buf.put_u32(MAGIC_COOKIE);

        // Header: Transaction ID (12 bytes)
        buf.put_slice(&self.transaction_id);

        // Attributes
        for attr in &self.attributes {
            attr.encode_with_tid(&mut buf, &self.transaction_id);
        }

        // Update message length (total bytes after header)
        let message_length = buf.len() - 20; // 20 bytes = header size
        buf[length_offset..length_offset + 2]
            .copy_from_slice(&u16::try_from(message_length).unwrap_or(u16::MAX).to_be_bytes());

        buf.freeze()
    }

    /// Decode message from bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the data is too short or malformed.
    pub fn decode(data: &[u8]) -> StunResult<Self> {
        if data.len() < 20 {
            return Err(StunError::InvalidResponse(String::from(
                "STUN message too short (< 20 bytes)",
            )));
        }

        let mut buf = data;

        // Parse header: Message Type (2 bytes)
        let message_type_raw = buf.get_u16();
        let message_type = MessageType::from_u16(message_type_raw)?;

        // Parse header: Message Length (2 bytes)
        let _message_length = buf.get_u16() as usize;

        // Parse header: Magic Cookie (4 bytes)
        let magic_cookie = buf.get_u32();
        if magic_cookie != MAGIC_COOKIE {
            return Err(StunError::InvalidResponse(format!(
                "Invalid magic cookie: 0x{magic_cookie:08x} (expected 0x{MAGIC_COOKIE:08x})"
            )));
        }

        // Parse header: Transaction ID (12 bytes)
        let mut transaction_id = [0u8; 12];
        buf.copy_to_slice(&mut transaction_id);

        // Parse attributes
        let mut attributes = Vec::new();
        while buf.remaining() >= 4 {
            match StunAttribute::decode_with_tid(&mut buf, &transaction_id) {
                Ok(attr) => attributes.push(attr),
                Err(e) => {
                    tracing::debug!("Failed to decode attribute: {}", e);
                    break;
                }
            }
        }

        Ok(Self {
            message_type,
            transaction_id,
            attributes,
        })
    }

    /// Find XOR-MAPPED-ADDRESS attribute (preferred)
    #[must_use]
    pub fn get_xor_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::XorMappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }

    /// Find MAPPED-ADDRESS attribute (fallback)
    #[must_use]
    pub fn get_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::MappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }

    /// Get any mapped address (XOR preferred, then MAPPED)
    #[must_use]
    pub fn get_any_mapped_address(&self) -> Option<SocketAddr> {
        self.get_xor_mapped_address().or_else(|| self.get_mapped_address())
    }

    /// Encode with MESSAGE-INTEGRITY and FINGERPRINT (RFC 5389 authenticated message).
    ///
    /// The message is encoded normally, then:
    /// 1. MESSAGE-INTEGRITY is computed over the header+attrs (with length adjusted
    ///    to include the MI attribute) and appended.
    /// 2. FINGERPRINT is computed over the header+attrs+MI (with length adjusted
    ///    to include the FP attribute) and appended.
    #[must_use]
    pub fn encode_authenticated(&self, key: &[u8]) -> bytes::Bytes {
        let mut buf = BytesMut::new();

        // Header
        buf.put_u16(self.message_type.to_u16());
        buf.put_u16(0); // length placeholder
        buf.put_u32(MAGIC_COOKIE);
        buf.put_slice(&self.transaction_id);

        // Attributes (excluding MI and FP — we compute them)
        for attr in &self.attributes {
            attr.encode_with_tid(&mut buf, &self.transaction_id);
        }

        // --- MESSAGE-INTEGRITY ---
        // Adjust length to include MI attr (4 header + 20 value = 24 bytes)
        let mi_length = (buf.len() - 20) + 24;
        buf[2..4].copy_from_slice(&u16::try_from(mi_length).unwrap_or(u16::MAX).to_be_bytes());
        let hmac_val = StunAttribute::compute_message_integrity(&buf, key);
        StunAttribute::MessageIntegrity(hmac_val).encode_with_tid(&mut buf, &self.transaction_id);

        // --- FINGERPRINT ---
        // Adjust length to include FP attr (4 header + 4 value = 8 bytes)
        let fp_length = (buf.len() - 20) + 8;
        buf[2..4].copy_from_slice(&u16::try_from(fp_length).unwrap_or(u16::MAX).to_be_bytes());
        let fp_val = StunAttribute::compute_fingerprint(&buf);
        StunAttribute::Fingerprint(fp_val).encode_with_tid(&mut buf, &self.transaction_id);

        // Final length (includes MI + FP)
        let final_length = buf.len() - 20;
        buf[2..4].copy_from_slice(&u16::try_from(final_length).unwrap_or(u16::MAX).to_be_bytes());

        buf.freeze()
    }
}

#[cfg(test)]
mod tests;
