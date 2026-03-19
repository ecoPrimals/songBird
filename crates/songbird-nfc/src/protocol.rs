//! Dark Forest NFC protocol implementation
//!
//! Wire format and message serialization/deserialization

use crate::error::{NfcError, Result};
use crate::{
    FRAME_OVERHEAD, MAX_PAYLOAD_SIZE, MSG_TYPE_GENESIS_REQUEST, MSG_TYPE_GENESIS_RESPONSE,
    NONCE_SIZE, PROTOCOL_VERSION, PUBLIC_KEY_SIZE, SIGNATURE_SIZE,
};

/// NFC message
#[derive(Debug, Clone)]
pub struct NfcMessage {
    /// Protocol version
    pub version: u8,

    /// Message type
    pub msg_type: u8,

    /// Ephemeral public key (X25519)
    pub public_key: [u8; PUBLIC_KEY_SIZE],

    /// Nonce (ChaCha20-Poly1305)
    pub nonce: [u8; NONCE_SIZE],

    /// Encrypted payload (with auth tag)
    pub encrypted_payload: Vec<u8>,

    /// Signature (ephemeral Ed25519)
    pub signature: [u8; SIGNATURE_SIZE],
}

impl NfcMessage {
    /// Create new message
    #[must_use]
    pub const fn new(
        msg_type: u8,
        public_key: [u8; PUBLIC_KEY_SIZE],
        nonce: [u8; NONCE_SIZE],
        encrypted_payload: Vec<u8>,
        signature: [u8; SIGNATURE_SIZE],
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type,
            public_key,
            nonce,
            encrypted_payload,
            signature,
        }
    }

    /// Serialize to wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the payload exceeds maximum size.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let payload_len = self.encrypted_payload.len();

        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(NfcError::PayloadTooLarge(payload_len, MAX_PAYLOAD_SIZE));
        }

        let total_len = FRAME_OVERHEAD + payload_len;
        let mut buf = Vec::with_capacity(total_len);

        // Header
        buf.push(self.version);
        buf.push(self.msg_type);
        buf.extend_from_slice(&u16::try_from(payload_len).unwrap_or(u16::MAX).to_be_bytes());

        // Ephemeral public key
        buf.extend_from_slice(&self.public_key);

        // Nonce
        buf.extend_from_slice(&self.nonce);

        // Encrypted payload
        buf.extend_from_slice(&self.encrypted_payload);

        // Signature
        buf.extend_from_slice(&self.signature);

        Ok(buf)
    }

    /// Deserialize from wire format
    ///
    /// # Errors
    ///
    /// Returns an error if the frame is malformed or too short.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < FRAME_OVERHEAD {
            return Err(NfcError::MalformedFrame(format!("Frame too short: {} bytes", data.len())));
        }

        let mut offset = 0;

        // Version
        let version = data[offset];
        offset += 1;

        if version != PROTOCOL_VERSION {
            return Err(NfcError::UnsupportedVersion(version));
        }

        // Message type
        let msg_type = data[offset];
        offset += 1;

        if msg_type != MSG_TYPE_GENESIS_REQUEST && msg_type != MSG_TYPE_GENESIS_RESPONSE {
            return Err(NfcError::InvalidMessageType(msg_type));
        }

        // Payload length
        let payload_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(NfcError::PayloadTooLarge(payload_len, MAX_PAYLOAD_SIZE));
        }

        // Public key
        let mut public_key = [0u8; PUBLIC_KEY_SIZE];
        public_key.copy_from_slice(&data[offset..offset + PUBLIC_KEY_SIZE]);
        offset += PUBLIC_KEY_SIZE;

        // Nonce
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&data[offset..offset + NONCE_SIZE]);
        offset += NONCE_SIZE;

        // Encrypted payload
        if data.len() < offset + payload_len + SIGNATURE_SIZE {
            return Err(NfcError::MalformedFrame(format!(
                "Incomplete frame: expected {} bytes",
                offset + payload_len + SIGNATURE_SIZE
            )));
        }

        let encrypted_payload = data[offset..offset + payload_len].to_vec();
        offset += payload_len;

        // Signature
        let mut signature = [0u8; SIGNATURE_SIZE];
        signature.copy_from_slice(&data[offset..offset + SIGNATURE_SIZE]);

        Ok(Self {
            version,
            msg_type,
            public_key,
            nonce,
            encrypted_payload,
            signature,
        })
    }
}

/// NFC protocol handler
#[derive(Debug)]
pub struct NfcProtocol {
    /// Configuration
    config: crate::config::NfcConfig,
}

impl NfcProtocol {
    /// Create new protocol handler
    #[must_use]
    pub const fn new(config: crate::config::NfcConfig) -> Self {
        Self {
            config,
        }
    }

    /// Get `BearDog` socket path
    #[must_use]
    pub fn beardog_socket(&self) -> &std::path::Path {
        &self.config.beardog_socket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_roundtrip() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [1u8; PUBLIC_KEY_SIZE],
            [2u8; NONCE_SIZE],
            vec![3u8; 128],
            [4u8; SIGNATURE_SIZE],
        );

        let bytes = msg.to_bytes().unwrap();
        let decoded = NfcMessage::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.version, PROTOCOL_VERSION);
        assert_eq!(decoded.msg_type, MSG_TYPE_GENESIS_REQUEST);
        assert_eq!(decoded.public_key, [1u8; PUBLIC_KEY_SIZE]);
        assert_eq!(decoded.nonce, [2u8; NONCE_SIZE]);
        assert_eq!(decoded.encrypted_payload.len(), 128);
        assert_eq!(decoded.signature, [4u8; SIGNATURE_SIZE]);
    }

    #[test]
    fn test_payload_too_large() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; MAX_PAYLOAD_SIZE + 1],
            [0u8; SIGNATURE_SIZE],
        );

        assert!(msg.to_bytes().is_err());
    }
}
