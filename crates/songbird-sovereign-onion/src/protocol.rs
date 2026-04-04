// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Minimal onion protocol messages

use crate::error::{OnionError, Result};
use serde::{Deserialize, Serialize};

/// Protocol message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Key exchange message (handshake)
    KeyExchange = 0x01,
    /// Encrypted data message
    Data = 0x02,
    /// Close connection
    Close = 0x03,
}

impl TryFrom<u8> for MessageType {
    type Error = OnionError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(Self::KeyExchange),
            0x02 => Ok(Self::Data),
            0x03 => Ok(Self::Close),
            _ => Err(OnionError::InvalidMessage(format!("Unknown message type: 0x{value:02x}"))),
        }
    }
}

/// Key exchange message (client → server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeMessage {
    /// Protocol version (0x01)
    pub version: u8,
    /// X25519 public key (32 bytes)
    pub pubkey: [u8; 32],
    /// Random nonce (24 bytes)
    pub nonce: [u8; 24],
}

impl KeyExchangeMessage {
    /// Create new key exchange message
    #[must_use]
    pub const fn new(pubkey: [u8; 32], nonce: [u8; 24]) -> Self {
        Self {
            version: 0x01,
            pubkey,
            nonce,
        }
    }

    /// Encode to bytes
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 32 + 24);
        buf.push(self.version);
        buf.extend_from_slice(&self.pubkey);
        buf.extend_from_slice(&self.nonce);
        buf
    }

    /// Decode from bytes
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short or version is unsupported.
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short or version is unsupported.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 57 {
            return Err(OnionError::InvalidMessage("KeyExchange too short".into()));
        }

        let version = bytes[0];
        if version != 0x01 {
            return Err(OnionError::InvalidMessage(format!(
                "Unsupported protocol version: {version}"
            )));
        }

        let pubkey: [u8; 32] = bytes[1..33]
            .try_into()
            .map_err(|_| OnionError::InvalidMessage("pubkey slice mismatch".into()))?;
        let nonce: [u8; 24] = bytes[33..57]
            .try_into()
            .map_err(|_| OnionError::InvalidMessage("nonce slice mismatch".into()))?;

        Ok(Self {
            version,
            pubkey,
            nonce,
        })
    }
}

/// Data message (encrypted payload)
#[derive(Debug, Clone)]
pub struct DataMessage {
    /// Monotonic sequence number (for replay protection)
    pub sequence: u64,
    /// Encrypted payload (ChaCha20-Poly1305)
    pub encrypted_payload: Vec<u8>,
}

impl DataMessage {
    /// Create new data message
    #[must_use]
    pub const fn new(sequence: u64, encrypted_payload: Vec<u8>) -> Self {
        Self {
            sequence,
            encrypted_payload,
        }
    }

    /// Encode to bytes
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.encrypted_payload.len());
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.encrypted_payload);
        buf
    }

    /// Decode from bytes
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short.
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short for decoding.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(OnionError::InvalidMessage("Data message too short".into()));
        }

        let seq_bytes: [u8; 8] = bytes[..8]
            .try_into()
            .map_err(|_| OnionError::InvalidMessage("sequence slice mismatch".into()))?;
        let sequence = u64::from_be_bytes(seq_bytes);
        let encrypted_payload = bytes[8..].to_vec();

        Ok(Self {
            sequence,
            encrypted_payload,
        })
    }
}

/// Wire message (framed with length + type)
#[derive(Debug, Clone)]
pub enum WireMessage {
    /// Key exchange handshake message
    KeyExchange(KeyExchangeMessage),
    /// Encrypted data message
    Data(DataMessage),
    /// Close connection message
    Close,
}

impl WireMessage {
    /// Encode to wire format
    ///
    /// Format: \[length: 4 bytes BE\] \[type: 1 byte\] \[payload\]
    ///
    /// Encode to wire format
    ///
    /// Format: \[length: 4 bytes BE\] \[type: 1 byte\] \[payload\]
    ///
    /// # Errors
    ///
    /// Returns error if the payload length exceeds `u32::MAX`.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let (msg_type, payload) = match self {
            Self::KeyExchange(msg) => (MessageType::KeyExchange, msg.encode()),
            Self::Data(msg) => (MessageType::Data, msg.encode()),
            Self::Close => (MessageType::Close, vec![]),
        };

        let length = u32::try_from(1 + payload.len())
            .map_err(|_| OnionError::InvalidMessage("payload exceeds u32 length limit".into()))?;
        let mut buf = Vec::with_capacity(4 + 1 + payload.len());
        buf.extend_from_slice(&length.to_be_bytes());
        buf.push(msg_type as u8);
        buf.extend_from_slice(&payload);

        Ok(buf)
    }

    /// Decode from wire format
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short, length mismatch, or payload decode fails.
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short, length mismatch, or payload decode fails.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(OnionError::InvalidMessage("Wire message too short".into()));
        }

        let len_bytes: [u8; 4] = bytes[..4]
            .try_into()
            .map_err(|_| OnionError::InvalidMessage("length header slice mismatch".into()))?;
        let length = u32::from_be_bytes(len_bytes) as usize;
        let msg_type = MessageType::try_from(bytes[4])?;
        let payload = &bytes[5..];

        if payload.len() + 1 != length {
            return Err(OnionError::InvalidMessage(format!(
                "Length mismatch: expected {length}, got {}",
                payload.len() + 1
            )));
        }

        match msg_type {
            MessageType::KeyExchange => {
                let msg = KeyExchangeMessage::decode(payload)?;
                Ok(Self::KeyExchange(msg))
            }
            MessageType::Data => {
                let msg = DataMessage::decode(payload)?;
                Ok(Self::Data(msg))
            }
            MessageType::Close => Ok(Self::Close),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn message_type_try_from_unknown() {
        let r = MessageType::try_from(0xff);
        assert!(matches!(r, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn key_exchange_decode_too_short() {
        let r = KeyExchangeMessage::decode(&[0u8; 10]);
        assert!(matches!(r, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn data_message_decode_too_short() {
        let r = DataMessage::decode(&[1, 2, 3]);
        assert!(matches!(r, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn wire_decode_too_short() {
        let r = WireMessage::decode(&[0u8; 3]);
        assert!(matches!(r, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn test_key_exchange_encode_decode() {
        let pubkey = [0x42u8; 32];
        let nonce = [0x01u8; 24];

        let msg = KeyExchangeMessage::new(pubkey, nonce);
        let encoded = msg.encode();
        let decoded = KeyExchangeMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.version, msg.version);
        assert_eq!(decoded.pubkey, msg.pubkey);
        assert_eq!(decoded.nonce, msg.nonce);
    }

    #[test]
    fn test_data_message_encode_decode() {
        let sequence = 42;
        let payload = vec![1, 2, 3, 4, 5];

        let msg = DataMessage::new(sequence, payload.clone());
        let encoded = msg.encode();
        let decoded = DataMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.sequence, sequence);
        assert_eq!(decoded.encrypted_payload, payload);
    }

    #[test]
    fn test_wire_message_key_exchange() {
        let pubkey = [0x42u8; 32];
        let nonce = [0x01u8; 24];
        let ke_msg = KeyExchangeMessage::new(pubkey, nonce);

        let wire_msg = WireMessage::KeyExchange(ke_msg.clone());
        let encoded = wire_msg.encode().unwrap();
        let decoded = WireMessage::decode(&encoded).unwrap();

        assert!(
            matches!(&decoded, WireMessage::KeyExchange(msg) if msg.pubkey == ke_msg.pubkey && msg.nonce == ke_msg.nonce),
            "Expected KeyExchange message"
        );
    }

    #[test]
    fn test_wire_message_data() {
        let data_msg = DataMessage::new(123, vec![1, 2, 3]);

        let wire_msg = WireMessage::Data(data_msg.clone());
        let encoded = wire_msg.encode().unwrap();
        let decoded = WireMessage::decode(&encoded).unwrap();

        assert!(
            matches!(&decoded, WireMessage::Data(msg) if msg.sequence == data_msg.sequence && msg.encrypted_payload == data_msg.encrypted_payload),
            "Expected Data message"
        );
    }

    #[test]
    fn test_wire_message_close() {
        let wire_msg = WireMessage::Close;
        let encoded = wire_msg.encode().unwrap();
        let decoded = WireMessage::decode(&encoded).unwrap();

        assert!(matches!(decoded, WireMessage::Close));
    }

    #[test]
    fn key_exchange_decode_bad_version() {
        let mut buf = vec![0x02]; // unsupported version
        buf.extend_from_slice(&[0u8; 56]); // pad to required length
        let result = KeyExchangeMessage::decode(&buf);
        assert!(matches!(result, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn key_exchange_decode_valid() {
        let mut buf = vec![0x01];
        buf.extend_from_slice(&[0xAA; 32]); // pubkey
        buf.extend_from_slice(&[0xBB; 24]); // nonce
        let msg = KeyExchangeMessage::decode(&buf).unwrap();
        assert_eq!(msg.version, 0x01);
        assert_eq!(msg.pubkey, [0xAA; 32]);
        assert_eq!(msg.nonce, [0xBB; 24]);
    }

    #[test]
    fn data_message_roundtrip_empty_payload() {
        let msg = DataMessage::new(0, vec![]);
        let encoded = msg.encode();
        let decoded = DataMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.sequence, 0);
        assert!(decoded.encrypted_payload.is_empty());
    }

    #[test]
    fn wire_decode_length_mismatch() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&100u32.to_be_bytes()); // claim 100 bytes
        buf.push(MessageType::Close as u8);
        // no payload but claimed 100
        let result = WireMessage::decode(&buf);
        assert!(matches!(result, Err(OnionError::InvalidMessage(_))));
    }

    #[test]
    fn message_type_all_variants() {
        assert_eq!(MessageType::try_from(0x01).unwrap(), MessageType::KeyExchange);
        assert_eq!(MessageType::try_from(0x02).unwrap(), MessageType::Data);
        assert_eq!(MessageType::try_from(0x03).unwrap(), MessageType::Close);
    }

    #[test]
    fn message_type_equality_and_copy() {
        let a = MessageType::KeyExchange;
        let b = MessageType::KeyExchange;
        assert_eq!(a, b, "KeyExchange should equal itself");
        assert_ne!(MessageType::Data, MessageType::Close, "distinct variants");
    }

    #[test]
    fn wire_decode_truncated_after_header() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&10u32.to_be_bytes());
        buf.push(MessageType::Close as u8);
        let r = WireMessage::decode(&buf);
        assert!(
            matches!(r, Err(OnionError::InvalidMessage(_))),
            "expected length mismatch, got {r:?}"
        );
    }

    #[test]
    fn wire_decode_payload_short_for_key_exchange() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&2u32.to_be_bytes());
        buf.push(MessageType::KeyExchange as u8);
        buf.push(0x01);
        let r = WireMessage::decode(&buf);
        assert!(
            matches!(r, Err(OnionError::InvalidMessage(_))),
            "KeyExchange payload too short: {r:?}"
        );
    }

    #[test]
    fn key_exchange_new_const_matches_encode_layout() {
        let msg = KeyExchangeMessage::new([3u8; 32], [4u8; 24]);
        assert_eq!(msg.version, 0x01, "protocol version");
        let enc = msg.encode();
        assert_eq!(enc.len(), 57, "encoded key exchange length");
    }

    #[test]
    fn data_message_large_sequence_roundtrips() {
        let seq = u64::MAX;
        let msg = DataMessage::new(seq, vec![0xAB; 3]);
        let decoded = DataMessage::decode(&msg.encode()).expect("decode");
        assert_eq!(decoded.sequence, seq, "u64::MAX sequence");
    }

    #[test]
    fn wire_message_close_length_is_one() {
        let w = WireMessage::Close;
        let enc = w.encode().expect("encode close");
        assert_eq!(
            u32::from_be_bytes([enc[0], enc[1], enc[2], enc[3]]),
            1,
            "length field should cover type byte only"
        );
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn wire_decode_rejects_unknown_message_type_byte() {
        let mut buf = vec![0u8; 6];
        buf[0..4].copy_from_slice(&5u32.to_be_bytes());
        buf[4] = 0xFF;
        let r = WireMessage::decode(&buf);
        assert!(matches!(r, Err(crate::error::OnionError::InvalidMessage(_))));
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn key_exchange_decode_ignores_trailing_payload_bytes() {
        let mut v = KeyExchangeMessage::new([1u8; 32], [2u8; 24]).encode();
        v.extend_from_slice(&[0xAB, 0xCD]);
        let m = KeyExchangeMessage::decode(&v).expect("leading 57 bytes must decode");
        assert_eq!(m.pubkey, [1u8; 32]);
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn wire_message_data_rejects_length_too_short_for_declared_payload() {
        let mut buf = vec![0u8; 8];
        buf[0..4].copy_from_slice(&100u32.to_be_bytes());
        buf[4] = MessageType::Data as u8;
        buf.push(0x00);
        let r = WireMessage::decode(&buf);
        assert!(matches!(r, Err(crate::error::OnionError::InvalidMessage(_))));
    }

    #[test]
    #[expect(clippy::unwrap_used, reason = "test assertion")]
    fn message_type_try_from_boundary_values() {
        assert!(MessageType::try_from(0x00).is_err());
        assert!(MessageType::try_from(0x04).is_err());
    }
}
