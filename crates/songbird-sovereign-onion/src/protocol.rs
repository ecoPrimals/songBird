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
    /// # Panics
    ///
    /// Panics if byte slice lengths are incorrect (caller must ensure valid input).
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

        let pubkey: [u8; 32] = bytes[1..33].try_into().expect("slice length is 32");
        let nonce: [u8; 24] = bytes[33..57].try_into().expect("slice length is 24");

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
    /// # Panics
    ///
    /// Panics if byte slice is too short for sequence (caller must ensure valid input).
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(OnionError::InvalidMessage("Data message too short".into()));
        }

        let sequence = u64::from_be_bytes(bytes[..8].try_into().expect("slice length is 8"));
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
    pub fn encode(&self) -> Vec<u8> {
        let (msg_type, payload) = match self {
            Self::KeyExchange(msg) => (MessageType::KeyExchange, msg.encode()),
            Self::Data(msg) => (MessageType::Data, msg.encode()),
            Self::Close => (MessageType::Close, vec![]),
        };

        let length = u32::try_from(1 + payload.len()).expect("payload length fits in u32"); // type byte + payload
        let mut buf = Vec::with_capacity(4 + 1 + payload.len());
        buf.extend_from_slice(&length.to_be_bytes());
        buf.push(msg_type as u8);
        buf.extend_from_slice(&payload);

        buf
    }

    /// Decode from wire format
    ///
    /// # Errors
    ///
    /// Returns error if bytes are too short, length mismatch, or payload decode fails.
    ///
    /// # Panics
    ///
    /// Panics if length header slice is wrong size (caller must ensure valid input).
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(OnionError::InvalidMessage("Wire message too short".into()));
        }

        let length = u32::from_be_bytes(bytes[..4].try_into().expect("slice length is 4")) as usize;
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
        let encoded = wire_msg.encode();
        let decoded = WireMessage::decode(&encoded).unwrap();

        match decoded {
            WireMessage::KeyExchange(msg) => {
                assert_eq!(msg.pubkey, ke_msg.pubkey);
                assert_eq!(msg.nonce, ke_msg.nonce);
            }
            _ => panic!("Expected KeyExchange message"),
        }
    }

    #[test]
    fn test_wire_message_data() {
        let data_msg = DataMessage::new(123, vec![1, 2, 3]);

        let wire_msg = WireMessage::Data(data_msg.clone());
        let encoded = wire_msg.encode();
        let decoded = WireMessage::decode(&encoded).unwrap();

        match decoded {
            WireMessage::Data(msg) => {
                assert_eq!(msg.sequence, data_msg.sequence);
                assert_eq!(msg.encrypted_payload, data_msg.encrypted_payload);
            }
            _ => panic!("Expected Data message"),
        }
    }

    #[test]
    fn test_wire_message_close() {
        let wire_msg = WireMessage::Close;
        let encoded = wire_msg.encode();
        let decoded = WireMessage::decode(&encoded).unwrap();

        assert!(matches!(decoded, WireMessage::Close));
    }
}
