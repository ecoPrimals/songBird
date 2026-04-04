// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

    /// Get security provider socket path
    #[must_use]
    pub fn security_provider_socket(&self) -> &std::path::Path {
        &self.config.security_provider_socket
    }

    /// Deprecated alias for [`NfcProtocol::security_provider_socket`].
    #[deprecated(note = "use security_provider_socket")]
    #[must_use]
    pub fn beardog_socket(&self) -> &std::path::Path {
        self.security_provider_socket()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
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

    #[test]
    fn empty_payload_roundtrips() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [0xabu8; PUBLIC_KEY_SIZE],
            [0xbcu8; NONCE_SIZE],
            Vec::new(),
            [0xdeu8; SIGNATURE_SIZE],
        );
        let bytes = msg.to_bytes().expect("empty payload is valid");
        assert_eq!(
            bytes.len(),
            FRAME_OVERHEAD,
            "wire size should equal frame overhead for empty ciphertext"
        );
        let decoded = NfcMessage::from_bytes(&bytes).expect("parse empty-payload frame");
        assert_eq!(decoded.encrypted_payload.len(), 0);
        assert_eq!(decoded.msg_type, MSG_TYPE_GENESIS_RESPONSE);
    }

    #[test]
    fn from_bytes_rejects_frame_shorter_than_overhead() {
        let short = vec![0u8; FRAME_OVERHEAD - 1];
        let err = NfcMessage::from_bytes(&short).expect_err("truncated frame should error");
        assert!(matches!(err, NfcError::MalformedFrame(_)), "expected MalformedFrame, got {err:?}");
    }

    #[test]
    fn from_bytes_rejects_unsupported_version() {
        let mut msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![1u8],
            [0u8; SIGNATURE_SIZE],
        );
        msg.version = 0x99;
        let bytes = msg.to_bytes().expect("valid message with nonstandard version byte");
        let err = NfcMessage::from_bytes(&bytes).expect_err("wrong version should error");
        assert!(
            matches!(err, NfcError::UnsupportedVersion(0x99)),
            "expected UnsupportedVersion, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_rejects_invalid_message_type() {
        let base = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![],
            [0u8; SIGNATURE_SIZE],
        );
        let mut bytes = base.to_bytes().expect("valid base");
        bytes[1] = 0xff;
        let err = NfcMessage::from_bytes(&bytes).expect_err("unknown msg type should error");
        assert!(
            matches!(err, NfcError::InvalidMessageType(0xff)),
            "expected InvalidMessageType, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_rejects_oversized_declared_payload() {
        let declared = u16::try_from(MAX_PAYLOAD_SIZE + 1).expect("1025 fits in u16");
        let mut frame = vec![0u8; FRAME_OVERHEAD];
        frame[0] = PROTOCOL_VERSION;
        frame[1] = MSG_TYPE_GENESIS_REQUEST;
        frame[2..4].copy_from_slice(&declared.to_be_bytes());
        let err = NfcMessage::from_bytes(&frame).expect_err("oversized length field should error");
        assert!(
            matches!(err, NfcError::PayloadTooLarge(n, MAX_PAYLOAD_SIZE) if n == MAX_PAYLOAD_SIZE + 1),
            "expected PayloadTooLarge, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_rejects_incomplete_trailing_frame() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [1u8; PUBLIC_KEY_SIZE],
            [2u8; NONCE_SIZE],
            vec![3u8; 10],
            [4u8; SIGNATURE_SIZE],
        );
        let mut bytes = msg.to_bytes().expect("valid message");
        bytes.truncate(bytes.len() - 1);
        let err = NfcMessage::from_bytes(&bytes).expect_err("truncated body should error");
        assert!(
            matches!(err, NfcError::MalformedFrame(_)),
            "expected MalformedFrame for incomplete frame, got {err:?}"
        );
    }

    #[test]
    fn nfc_protocol_exposes_config_socket() {
        use std::path::PathBuf;
        let socket = PathBuf::from("/tmp/test-nfc-security.sock");
        let cfg = crate::NfcConfig::default().with_security_provider_socket(socket.clone());
        let proto = NfcProtocol::new(cfg);
        assert_eq!(
            proto.security_provider_socket(),
            socket.as_path(),
            "socket path should match config"
        );
    }

    #[test]
    fn to_bytes_accepts_max_payload_boundary() {
        let payload = vec![0xabu8; MAX_PAYLOAD_SIZE];
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            payload,
            [0u8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("max payload serializes");
        assert_eq!(wire.len(), FRAME_OVERHEAD + MAX_PAYLOAD_SIZE);
        let back = NfcMessage::from_bytes(&wire).expect("roundtrip");
        assert_eq!(back.encrypted_payload.len(), MAX_PAYLOAD_SIZE);
    }

    #[test]
    fn to_bytes_rejects_payload_one_byte_over_max() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [1u8; PUBLIC_KEY_SIZE],
            [2u8; NONCE_SIZE],
            vec![0u8; MAX_PAYLOAD_SIZE + 1],
            [3u8; SIGNATURE_SIZE],
        );
        let err = msg.to_bytes().expect_err("oversized payload");
        assert!(
            matches!(err, NfcError::PayloadTooLarge(n, MAX_PAYLOAD_SIZE) if n == MAX_PAYLOAD_SIZE + 1),
            "expected PayloadTooLarge, got {err:?}"
        );
    }

    #[test]
    fn from_bytes_accepts_both_genesis_message_types() {
        for ty in [MSG_TYPE_GENESIS_REQUEST, MSG_TYPE_GENESIS_RESPONSE] {
            let msg = NfcMessage::new(
                ty,
                [7u8; PUBLIC_KEY_SIZE],
                [8u8; NONCE_SIZE],
                vec![9, 10],
                [6u8; SIGNATURE_SIZE],
            );
            let bytes = msg.to_bytes().expect("serialize");
            let parsed = NfcMessage::from_bytes(&bytes).expect("parse");
            assert_eq!(parsed.msg_type, ty);
        }
    }

    #[test]
    fn wire_length_field_is_big_endian() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0xff; 256],
            [0u8; SIGNATURE_SIZE],
        );
        let bytes = msg.to_bytes().expect("serialize");
        assert_eq!(bytes[2], 0x01);
        assert_eq!(bytes[3], 0x00);
    }

    #[test]
    fn from_bytes_preserves_declared_protocol_version_field() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0xabu8; PUBLIC_KEY_SIZE],
            [0xbcu8; NONCE_SIZE],
            vec![1],
            [0xdeu8; SIGNATURE_SIZE],
        );
        assert_eq!(msg.version, PROTOCOL_VERSION);
        let bytes = msg.to_bytes().expect("serialize");
        let parsed = NfcMessage::from_bytes(&bytes).expect("parse");
        assert_eq!(parsed.version, PROTOCOL_VERSION);
    }

    #[test]
    fn malformed_frame_display_explains_incomplete_frame() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; 5],
            [0u8; SIGNATURE_SIZE],
        );
        let mut bytes = msg.to_bytes().expect("serialize");
        bytes.truncate(bytes.len() - SIGNATURE_SIZE / 2);
        let err = NfcMessage::from_bytes(&bytes).expect_err("truncated signature");
        let text = err.to_string();
        assert!(
            text.contains("Incomplete") || text.contains("bytes"),
            "error should explain truncation: {text}"
        );
    }

    #[test]
    fn from_bytes_accepts_trailing_garbage_after_signature() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_RESPONSE,
            [0x11u8; PUBLIC_KEY_SIZE],
            [0x22u8; NONCE_SIZE],
            vec![0x33, 0x44],
            [0x55u8; SIGNATURE_SIZE],
        );
        let mut bytes = msg.to_bytes().expect("serialize");
        bytes.extend_from_slice(b"EXTRA_TRAILING_BYTES");
        let parsed = NfcMessage::from_bytes(&bytes).expect("wire format does not require EOF");
        assert_eq!(parsed.msg_type, MSG_TYPE_GENESIS_RESPONSE);
        assert_eq!(parsed.encrypted_payload, vec![0x33, 0x44]);
    }

    #[test]
    fn from_bytes_rejects_truncated_signature_only() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [1u8; PUBLIC_KEY_SIZE],
            [2u8; NONCE_SIZE],
            vec![],
            [3u8; SIGNATURE_SIZE],
        );
        let bytes = msg.to_bytes().expect("serialize");
        let truncated = &bytes[..bytes.len().saturating_sub(1)];
        let err = NfcMessage::from_bytes(truncated).expect_err("missing final signature byte");
        assert!(matches!(err, NfcError::MalformedFrame(_)), "got {err:?}");
    }

    #[test]
    fn from_bytes_rejects_message_type_zero() {
        let base = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![],
            [0u8; SIGNATURE_SIZE],
        );
        let mut bytes = base.to_bytes().expect("serialize");
        bytes[1] = 0;
        let err = NfcMessage::from_bytes(&bytes).expect_err("msg type 0 is invalid");
        assert!(matches!(err, NfcError::InvalidMessageType(0)), "got {err:?}");
    }

    #[test]
    fn from_bytes_rejects_wrong_protocol_version_byte() {
        let mut bytes = vec![0u8; FRAME_OVERHEAD];
        bytes[0] = PROTOCOL_VERSION.wrapping_add(1);
        bytes[1] = MSG_TYPE_GENESIS_REQUEST;
        bytes[2..4].copy_from_slice(&0u16.to_be_bytes());
        let err = NfcMessage::from_bytes(&bytes).expect_err("version mismatch");
        assert!(
            matches!(err, NfcError::UnsupportedVersion(v) if v == PROTOCOL_VERSION.wrapping_add(1)),
            "got {err:?}"
        );
    }

    #[test]
    fn to_bytes_length_field_matches_payload_at_max() {
        let msg = NfcMessage::new(
            MSG_TYPE_GENESIS_REQUEST,
            [0u8; PUBLIC_KEY_SIZE],
            [0u8; NONCE_SIZE],
            vec![0u8; MAX_PAYLOAD_SIZE],
            [0u8; SIGNATURE_SIZE],
        );
        let wire = msg.to_bytes().expect("at-limit payload");
        let len = u16::from_be_bytes([wire[2], wire[3]]) as usize;
        assert_eq!(len, MAX_PAYLOAD_SIZE);
    }
}
