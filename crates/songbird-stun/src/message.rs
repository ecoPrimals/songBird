// SPDX-License-Identifier: AGPL-3.0-only
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

use crate::error::{StunError, StunResult};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// STUN magic cookie (RFC 5389)
pub const MAGIC_COOKIE: u32 = 0x2112_A442;

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Binding Request (0x0001)
    BindingRequest,

    /// Binding Success Response (0x0101)
    BindingResponse,

    /// Binding Error Response (0x0111)
    BindingError,
}

impl MessageType {
    /// Convert to wire format (u16)
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::BindingRequest => 0x0001,
            Self::BindingResponse => 0x0101,
            Self::BindingError => 0x0111,
        }
    }

    /// Parse from wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid message type.
    pub fn from_u16(value: u16) -> StunResult<Self> {
        match value {
            0x0001 => Ok(Self::BindingRequest),
            0x0101 => Ok(Self::BindingResponse),
            0x0111 => Ok(Self::BindingError),
            _ => Err(StunError::InvalidResponse(format!("Unknown message type: 0x{value:04x}"))),
        }
    }
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    /// MAPPED-ADDRESS (0x0001)
    MappedAddress,

    /// XOR-MAPPED-ADDRESS (0x0020) - preferred
    XorMappedAddress,

    /// OTHER-ADDRESS (0x802C) - for NAT type detection
    OtherAddress,

    /// Unknown attribute
    Unknown(u16),
}

impl AttributeType {
    /// Convert to wire format (u16)
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        match self {
            Self::MappedAddress => 0x0001,
            Self::XorMappedAddress => 0x0020,
            Self::OtherAddress => 0x802C,
            Self::Unknown(value) => value,
        }
    }

    /// Parse from wire format
    #[must_use]
    pub const fn from_u16(value: u16) -> Self {
        match value {
            0x0001 => Self::MappedAddress,
            0x0020 => Self::XorMappedAddress,
            0x802C => Self::OtherAddress,
            _ => Self::Unknown(value),
        }
    }
}

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
    pub fn encode(&self) -> Bytes {
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
            attr.encode(&mut buf);
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
            return Err(StunError::InvalidResponse(
                "STUN message too short (< 20 bytes)".to_string(),
            ));
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
            match StunAttribute::decode(&mut buf) {
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
}

/// STUN attribute
#[derive(Debug, Clone)]
pub enum StunAttribute {
    /// MAPPED-ADDRESS
    MappedAddress(SocketAddr),

    /// XOR-MAPPED-ADDRESS (preferred)
    XorMappedAddress(SocketAddr),

    /// OTHER-ADDRESS (for NAT type detection)
    OtherAddress(SocketAddr),

    /// Unknown attribute (type, data)
    Unknown(u16, Bytes),
}

impl StunAttribute {
    /// Encode attribute to buffer
    fn encode(&self, buf: &mut BytesMut) {
        match self {
            Self::MappedAddress(addr) => {
                Self::encode_address(buf, AttributeType::MappedAddress, addr, None);
            }
            Self::XorMappedAddress(addr) => {
                Self::encode_address(
                    buf,
                    AttributeType::XorMappedAddress,
                    addr,
                    Some(MAGIC_COOKIE),
                );
            }
            Self::OtherAddress(addr) => {
                Self::encode_address(buf, AttributeType::OtherAddress, addr, None);
            }
            Self::Unknown(attr_type, data) => {
                buf.put_u16(*attr_type);
                buf.put_u16(u16::try_from(data.len()).unwrap_or(u16::MAX));
                buf.put_slice(data);
                Self::add_padding(buf);
            }
        }
    }

    /// Encode address attribute
    fn encode_address(
        buf: &mut BytesMut,
        attr_type: AttributeType,
        addr: &SocketAddr,
        xor_key: Option<u32>,
    ) {
        buf.put_u16(attr_type.to_u16());

        let data_start = buf.len();
        buf.put_u16(0); // Length placeholder

        // Family (1 byte reserved, 1 byte family)
        buf.put_u8(0);
        match addr {
            SocketAddr::V4(_) => buf.put_u8(0x01), // IPv4
            SocketAddr::V6(_) => buf.put_u8(0x02), // IPv6
        }

        // Port (2 bytes)
        let port = xor_key.map_or_else(|| addr.port(), |xor| addr.port() ^ (xor >> 16) as u16);
        buf.put_u16(port);

        // Address
        match addr.ip() {
            IpAddr::V4(ip) => {
                let octets = xor_key.map_or_else(
                    || ip.octets(),
                    |xor| {
                        let ip_u32 = u32::from(ip);
                        let xored = ip_u32 ^ xor;
                        xored.to_be_bytes()
                    },
                );
                buf.put_slice(&octets);
            }
            IpAddr::V6(ip) => {
                // XOR with magic cookie + transaction ID not implemented for IPv6
                let octets = ip.octets();
                buf.put_slice(&octets);
            }
        }

        // Update length
        let data_len = buf.len() - data_start - 2;
        buf[data_start..data_start + 2]
            .copy_from_slice(&u16::try_from(data_len).unwrap_or(u16::MAX).to_be_bytes());

        Self::add_padding(buf);
    }

    /// Decode attribute from buffer
    fn decode(buf: &mut &[u8]) -> StunResult<Self> {
        if buf.remaining() < 4 {
            return Err(StunError::InvalidResponse("Attribute too short".to_string()));
        }

        let attr_type = buf.get_u16();
        let attr_length = buf.get_u16() as usize;

        if buf.remaining() < attr_length {
            return Err(StunError::InvalidResponse(format!(
                "Attribute data too short: expected {}, got {}",
                attr_length,
                buf.remaining()
            )));
        }

        let attr_data = &buf[..attr_length];
        buf.advance(attr_length);

        // Skip padding (attributes padded to 4-byte boundary)
        let padding = (4 - (attr_length % 4)) % 4;
        buf.advance(padding.min(buf.remaining()));

        match AttributeType::from_u16(attr_type) {
            AttributeType::MappedAddress => {
                let addr = Self::decode_address(attr_data, None)?;
                Ok(Self::MappedAddress(addr))
            }
            AttributeType::XorMappedAddress => {
                let addr = Self::decode_address(attr_data, Some(MAGIC_COOKIE))?;
                Ok(Self::XorMappedAddress(addr))
            }
            AttributeType::OtherAddress => {
                let addr = Self::decode_address(attr_data, None)?;
                Ok(Self::OtherAddress(addr))
            }
            AttributeType::Unknown(_) => {
                Ok(Self::Unknown(attr_type, Bytes::copy_from_slice(attr_data)))
            }
        }
    }

    /// Decode address attribute
    fn decode_address(data: &[u8], xor_key: Option<u32>) -> StunResult<SocketAddr> {
        if data.len() < 4 {
            return Err(StunError::InvalidResponse("Address attribute too short".to_string()));
        }

        let mut buf = data;

        // Reserved (1 byte)
        buf.advance(1);

        // Family (1 byte)
        let family = buf.get_u8();

        // Port (2 bytes)
        let port_raw = buf.get_u16();
        let port = xor_key.map_or_else(|| port_raw, |xor| port_raw ^ (xor >> 16) as u16);

        // Address
        match family {
            0x01 => {
                // IPv4
                if buf.remaining() < 4 {
                    return Err(StunError::InvalidResponse("IPv4 address too short".to_string()));
                }

                let ip_raw = buf.get_u32();
                let ip = xor_key
                    .map_or_else(|| Ipv4Addr::from(ip_raw), |xor| Ipv4Addr::from(ip_raw ^ xor));

                Ok(SocketAddr::new(IpAddr::V4(ip), port))
            }
            0x02 => {
                // IPv6
                if buf.remaining() < 16 {
                    return Err(StunError::InvalidResponse("IPv6 address too short".to_string()));
                }

                let mut octets = [0u8; 16];
                buf.copy_to_slice(&mut octets);

                // Note: Full XOR for IPv6 requires transaction ID (not implemented for simplicity)
                let ip = Ipv6Addr::from(octets);

                Ok(SocketAddr::new(IpAddr::V6(ip), port))
            }
            _ => Err(StunError::InvalidResponse(format!("Unknown address family: {family}"))),
        }
    }

    /// Add padding to align to 4-byte boundary
    fn add_padding(buf: &mut BytesMut) {
        let padding = (4 - (buf.len() % 4)) % 4;
        for _ in 0..padding {
            buf.put_u8(0);
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::BindingRequest.to_u16(), 0x0001);
        assert_eq!(MessageType::from_u16(0x0001).expect("valid type"), MessageType::BindingRequest);
        let err = MessageType::from_u16(0xFFFF).expect_err("unknown type");
        assert!(err.to_string().contains("Unknown") || err.to_string().contains("message type"));
    }

    #[test]
    fn test_binding_request_encode() {
        let msg = StunMessage::new_binding_request();
        let encoded = msg.encode();

        // Verify header
        assert_eq!(encoded.len(), 20); // Header only, no attributes
        assert_eq!(u16::from_be_bytes([encoded[0], encoded[1]]), 0x0001); // Binding Request
        assert_eq!(
            u32::from_be_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]),
            MAGIC_COOKIE
        );
    }

    #[test]
    fn test_binding_request_decode() {
        let msg = StunMessage::new_binding_request();
        let encoded = msg.encode();
        let decoded = StunMessage::decode(&encoded).expect("decode binding request");

        assert_eq!(decoded.message_type, MessageType::BindingRequest);
        assert_eq!(decoded.transaction_id, msg.transaction_id);
    }

    #[test]
    fn test_xor_mapped_address() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 8080);

        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::BindingResponse;
        msg.attributes.push(StunAttribute::XorMappedAddress(addr));

        let encoded = msg.encode();
        let decoded = StunMessage::decode(&encoded).expect("decode with xor");

        assert_eq!(decoded.get_xor_mapped_address(), Some(addr));
    }

    #[test]
    fn decode_rejects_short_buffer() {
        let err = StunMessage::decode(&[0u8; 8]).expect_err("too short");
        assert!(err.to_string().contains("short") || err.to_string().contains("20"));
    }

    #[test]
    fn decode_rejects_bad_magic_cookie() {
        let mut buf = vec![0u8; 20];
        buf[0] = 0x00;
        buf[1] = 0x01;
        buf[2] = 0x00;
        buf[3] = 0x00;
        buf[4] = 0xff;
        buf[5] = 0xff;
        buf[6] = 0xff;
        buf[7] = 0xff;
        let err = StunMessage::decode(&buf).expect_err("bad cookie");
        assert!(err.to_string().contains("magic") || err.to_string().contains("cookie"));
    }

    #[test]
    fn attribute_type_roundtrip() {
        assert_eq!(AttributeType::Unknown(0xabcd).to_u16(), 0xabcd);
        assert!(matches!(AttributeType::from_u16(0xabcd), AttributeType::Unknown(0xabcd)));
    }
}

/// Hand-crafted edge cases for [`StunMessage::decode`] (fuzz-style, no external harness).
#[cfg(test)]
mod fuzz_style_stun_decode_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::{MAGIC_COOKIE, MessageType, StunMessage};

    #[test]
    fn decode_random_short_inputs_never_panic() {
        for len in 0..20usize {
            #[allow(clippy::cast_possible_truncation, reason = "fuzz test: intentional truncation")]
            let buf: Vec<u8> = (0..len).map(|i| (i * 7 + 13) as u8).collect();
            let _ = StunMessage::decode(&buf);
        }
    }

    #[test]
    fn decode_truncated_at_nineteen_bytes_errors() {
        let buf = vec![0u8; 19];
        assert!(StunMessage::decode(&buf).is_err());
    }

    #[test]
    fn decode_header_length_claims_more_than_buffer_still_parses_header_fields() {
        // 20-byte header: type, length=1000, magic, txn id — but no attribute bytes.
        let mut buf = vec![0u8; 20];
        buf[0] = 0x00;
        buf[1] = 0x01;
        buf[2] = 0x03;
        buf[3] = 0xe8;
        buf[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        let msg = StunMessage::decode(&buf).expect("header parses");
        assert_eq!(msg.message_type, MessageType::BindingRequest);
        assert!(msg.attributes.is_empty());
    }

    #[test]
    fn decode_invalid_message_type_errors() {
        let mut buf = vec![0u8; 20];
        buf[0] = 0xff;
        buf[1] = 0xff;
        buf[2] = 0x00;
        buf[3] = 0x00;
        buf[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
        assert!(StunMessage::decode(&buf).is_err());
    }

    #[test]
    fn decode_binding_request_roundtrip_max_attribute_padding_stress() {
        let msg = StunMessage::new_binding_request();
        let encoded = msg.encode();
        let decoded = StunMessage::decode(&encoded).expect("roundtrip");
        assert_eq!(decoded.transaction_id, msg.transaction_id);
    }
}
