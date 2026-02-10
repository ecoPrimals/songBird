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
            0x01 => Ok(MessageType::KeyExchange),
            0x02 => Ok(MessageType::Data),
            0x03 => Ok(MessageType::Close),
            _ => Err(OnionError::InvalidMessage(format!("Unknown message type: 0x{:02x}", value))),
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
    pub fn new(pubkey: [u8; 32], nonce: [u8; 24]) -> Self {
        Self {
            version: 0x01,
            pubkey,
            nonce,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 32 + 24);
        buf.push(self.version);
        buf.extend_from_slice(&self.pubkey);
        buf.extend_from_slice(&self.nonce);
        buf
    }

    /// Decode from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 57 {
            return Err(OnionError::InvalidMessage("KeyExchange too short".into()));
        }

        let version = bytes[0];
        if version != 0x01 {
            return Err(OnionError::InvalidMessage(format!(
                "Unsupported protocol version: {}",
                version
            )));
        }

        let pubkey: [u8; 32] = bytes[1..33].try_into().unwrap();
        let nonce: [u8; 24] = bytes[33..57].try_into().unwrap();

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
    pub fn new(sequence: u64, encrypted_payload: Vec<u8>) -> Self {
        Self {
            sequence,
            encrypted_payload,
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.encrypted_payload.len());
        buf.extend_from_slice(&self.sequence.to_be_bytes());
        buf.extend_from_slice(&self.encrypted_payload);
        buf
    }

    /// Decode from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(OnionError::InvalidMessage("Data message too short".into()));
        }

        let sequence = u64::from_be_bytes(bytes[..8].try_into().unwrap());
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
            WireMessage::KeyExchange(msg) => (MessageType::KeyExchange, msg.encode()),
            WireMessage::Data(msg) => (MessageType::Data, msg.encode()),
            WireMessage::Close => (MessageType::Close, vec![]),
        };

        let length = (1 + payload.len()) as u32; // type byte + payload
        let mut buf = Vec::with_capacity(4 + 1 + payload.len());
        buf.extend_from_slice(&length.to_be_bytes());
        buf.push(msg_type as u8);
        buf.extend_from_slice(&payload);

        buf
    }

    /// Decode from wire format
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(OnionError::InvalidMessage("Wire message too short".into()));
        }

        let length = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
        let msg_type = MessageType::try_from(bytes[4])?;
        let payload = &bytes[5..];

        if payload.len() + 1 != length {
            return Err(OnionError::InvalidMessage(format!(
                "Length mismatch: expected {}, got {}",
                length,
                payload.len() + 1
            )));
        }

        match msg_type {
            MessageType::KeyExchange => {
                let msg = KeyExchangeMessage::decode(payload)?;
                Ok(WireMessage::KeyExchange(msg))
            }
            MessageType::Data => {
                let msg = DataMessage::decode(payload)?;
                Ok(WireMessage::Data(msg))
            }
            MessageType::Close => Ok(WireMessage::Close),
        }
    }
}

#[cfg(all(test, feature = "standalone"))]
mod tests {
    use super::*;

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
