// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared TURN attribute parsing and construction helpers.
//!
//! Centralizes RFC 5766 attribute type codes, XOR address encoding, and
//! ERROR-CODE construction so that `turn_server.rs` and any future TURN
//! code shares a single source of truth for wire format.

use crate::message::{MAGIC_COOKIE, MessageType, StunAttribute, StunMessage};
use crate::turn::encode_xor_peer_address;
use bytes::{BufMut, BytesMut};
use std::net::SocketAddr;

/// TURN attribute type codes (RFC 5766).
mod attr_type {
    pub const CHANNEL_NUMBER: u16 = 0x000C;
    pub const LIFETIME: u16 = 0x000D;
    pub const XOR_PEER_ADDRESS: u16 = 0x0012;
    pub const DATA: u16 = 0x0013;
    pub const XOR_RELAYED_ADDRESS: u16 = 0x0016;
    /// ERROR-CODE attribute (RFC 5389 §15.6)
    pub const ERROR_CODE: u16 = 0x0009;
}

/// Namespace for TURN attribute helpers (stateless, no `self`).
pub struct TurnAttrs;

impl TurnAttrs {
    // ── Parsing ──────────────────────────────────────────────────────────

    /// Extract XOR-PEER-ADDRESS from a STUN message.
    pub fn parse_peer_addr(msg: &StunMessage) -> Option<SocketAddr> {
        msg.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(attr_type::XOR_PEER_ADDRESS, data) = attr {
                StunAttribute::decode_address(data, Some(MAGIC_COOKIE), &msg.transaction_id).ok()
            } else {
                None
            }
        })
    }

    /// Extract LIFETIME attribute value (seconds).
    pub fn parse_lifetime(msg: &StunMessage) -> Option<u32> {
        msg.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(attr_type::LIFETIME, data) = attr {
                (data.len() >= 4).then(|| u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
            } else {
                None
            }
        })
    }

    /// Extract CHANNEL-NUMBER attribute value.
    pub fn parse_channel(msg: &StunMessage) -> Option<u16> {
        msg.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(attr_type::CHANNEL_NUMBER, data) = attr {
                (data.len() >= 2).then(|| u16::from_be_bytes([data[0], data[1]]))
            } else {
                None
            }
        })
    }

    /// Extract DATA attribute payload.
    pub fn parse_data(msg: &StunMessage) -> Option<&[u8]> {
        msg.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(attr_type::DATA, data) = attr {
                Some(data.as_ref())
            } else {
                None
            }
        })
    }

    // ── Construction ─────────────────────────────────────────────────────

    /// Build an Allocate Success response with XOR-MAPPED-ADDRESS,
    /// XOR-RELAYED-ADDRESS, and LIFETIME.
    pub fn build_allocate_success(
        request: &StunMessage,
        client_addr: SocketAddr,
        relay_addr: SocketAddr,
        lifetime: u32,
    ) -> StunMessage {
        let relay_encoded = encode_xor_peer_address(&relay_addr, &request.transaction_id);

        let mut lt_buf = BytesMut::with_capacity(4);
        lt_buf.put_u32(lifetime);

        StunMessage {
            message_type: MessageType::AllocateSuccess,
            transaction_id: request.transaction_id,
            attributes: vec![
                StunAttribute::XorMappedAddress(client_addr),
                StunAttribute::Unknown(attr_type::XOR_RELAYED_ADDRESS, relay_encoded),
                StunAttribute::Unknown(attr_type::LIFETIME, lt_buf.freeze()),
            ],
        }
    }

    /// Build a response carrying only a LIFETIME attribute.
    pub fn build_lifetime_response(
        request: &StunMessage,
        msg_type: MessageType,
        lifetime: u32,
    ) -> StunMessage {
        let mut lt_buf = BytesMut::with_capacity(4);
        lt_buf.put_u32(lifetime);

        StunMessage {
            message_type: msg_type,
            transaction_id: request.transaction_id,
            attributes: vec![StunAttribute::Unknown(attr_type::LIFETIME, lt_buf.freeze())],
        }
    }

    /// Build a `ChannelData` frame: `[2B channel][2B length][payload]`
    pub fn build_channel_data(channel: u16, data: &[u8]) -> Vec<u8> {
        let len = u16::try_from(data.len()).unwrap_or(u16::MAX);
        let mut frame = Vec::with_capacity(4 + data.len());
        frame.extend_from_slice(&channel.to_be_bytes());
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(data);
        frame
    }

    /// Build a Data Indication (RFC 5766 §10): XOR-PEER-ADDRESS + DATA.
    pub fn build_data_indication(peer_addr: SocketAddr, data: &[u8]) -> bytes::Bytes {
        let mut tid = [0u8; 12];
        for byte in &mut tid {
            *byte = rand::random();
        }

        let peer_encoded = encode_xor_peer_address(&peer_addr, &tid);

        let msg = StunMessage {
            message_type: MessageType::DataIndication,
            transaction_id: tid,
            attributes: vec![
                StunAttribute::Unknown(attr_type::XOR_PEER_ADDRESS, peer_encoded),
                StunAttribute::Unknown(attr_type::DATA, bytes::Bytes::copy_from_slice(data)),
            ],
        };
        msg.encode()
    }

    /// Build an ERROR-CODE attribute (RFC 5389 §15.6).
    ///
    /// Format: `[2B reserved][1B class (hundreds)][1B number (tens+units)][reason phrase]`
    pub fn build_error_code(code: u16, reason: &str) -> StunAttribute {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "RFC error codes are always 3-digit (300-699)"
        )]
        let class = (code / 100) as u8;
        #[allow(clippy::cast_possible_truncation, reason = "modulo 100 always fits u8")]
        let number = (code % 100) as u8;
        let reason_bytes = reason.as_bytes();
        let mut buf = BytesMut::with_capacity(4 + reason_bytes.len());
        buf.put_u16(0); // reserved
        buf.put_u8(class);
        buf.put_u8(number);
        buf.put_slice(reason_bytes);
        StunAttribute::Unknown(attr_type::ERROR_CODE, buf.freeze())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::types::StunCredentials;

    #[test]
    fn parse_lifetime_extracts_value() {
        let mut msg = StunMessage::new_binding_request();
        let mut lt_buf = BytesMut::with_capacity(4);
        lt_buf.put_u32(300);
        msg.attributes.push(StunAttribute::Unknown(attr_type::LIFETIME, lt_buf.freeze()));

        assert_eq!(TurnAttrs::parse_lifetime(&msg), Some(300));
    }

    #[test]
    fn parse_lifetime_returns_none_when_missing() {
        let msg = StunMessage::new_binding_request();
        assert_eq!(TurnAttrs::parse_lifetime(&msg), None);
    }

    #[test]
    fn parse_channel_extracts_value() {
        let mut msg = StunMessage::new_binding_request();
        let mut ch_buf = BytesMut::with_capacity(4);
        ch_buf.put_u16(0x4001);
        ch_buf.put_u16(0); // RFFU
        msg.attributes.push(StunAttribute::Unknown(attr_type::CHANNEL_NUMBER, ch_buf.freeze()));

        assert_eq!(TurnAttrs::parse_channel(&msg), Some(0x4001));
    }

    #[test]
    fn parse_data_extracts_payload() {
        let mut msg = StunMessage::new_binding_request();
        msg.attributes
            .push(StunAttribute::Unknown(attr_type::DATA, bytes::Bytes::from_static(b"hello")));

        assert_eq!(TurnAttrs::parse_data(&msg), Some(b"hello".as_slice()));
    }

    #[test]
    fn parse_peer_addr_roundtrips() {
        let addr: SocketAddr = "203.0.113.5:9200".parse().unwrap();
        let mut msg = StunMessage::new_binding_request();
        let encoded = encode_xor_peer_address(&addr, &msg.transaction_id);
        msg.attributes.push(StunAttribute::Unknown(attr_type::XOR_PEER_ADDRESS, encoded));

        let parsed = TurnAttrs::parse_peer_addr(&msg).unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn build_error_code_format() {
        let attr = TurnAttrs::build_error_code(401, "Unauthorized");
        if let StunAttribute::Unknown(0x0009, data) = attr {
            assert_eq!(data[0], 0); // reserved
            assert_eq!(data[1], 0); // reserved
            assert_eq!(data[2], 4); // class = 4 (4xx)
            assert_eq!(data[3], 1); // number = 01
            assert_eq!(&data[4..], b"Unauthorized");
        } else {
            panic!("expected Unknown(ERROR_CODE, ...)");
        }
    }

    #[test]
    fn build_allocate_success_has_required_attrs() {
        let req = StunMessage::new_binding_request();
        let client: SocketAddr = "192.0.2.1:12345".parse().unwrap();
        let relay: SocketAddr = "198.51.100.5:49152".parse().unwrap();

        let resp = TurnAttrs::build_allocate_success(&req, client, relay, 600);
        assert_eq!(resp.message_type, MessageType::AllocateSuccess);
        assert_eq!(resp.transaction_id, req.transaction_id);
        assert_eq!(resp.attributes.len(), 3);
    }

    #[test]
    fn channel_data_roundtrip() {
        let data = b"test payload data";
        let frame = TurnAttrs::build_channel_data(0x4000, data);
        assert_eq!(frame.len(), 4 + data.len());
        assert_eq!(u16::from_be_bytes([frame[0], frame[1]]), 0x4000);
        assert_eq!(u16::from_be_bytes([frame[2], frame[3]]), data.len() as u16);
        assert_eq!(&frame[4..], data);
    }

    #[test]
    fn data_indication_parses_back() {
        let peer: SocketAddr = "10.0.0.5:8080".parse().unwrap();
        let payload = b"relayed data";
        let wire = TurnAttrs::build_data_indication(peer, payload);

        let msg = StunMessage::decode(&wire).unwrap();
        assert_eq!(msg.message_type, MessageType::DataIndication);

        let parsed_data = TurnAttrs::parse_data(&msg).unwrap();
        assert_eq!(parsed_data, payload);
    }

    #[test]
    fn new_error_types_wire_values() {
        assert_eq!(MessageType::RefreshError.to_u16(), 0x0114);
        assert_eq!(MessageType::CreatePermissionError.to_u16(), 0x0118);
        assert_eq!(MessageType::ChannelBindError.to_u16(), 0x0119);

        assert_eq!(MessageType::from_u16(0x0114).unwrap(), MessageType::RefreshError);
        assert_eq!(MessageType::from_u16(0x0118).unwrap(), MessageType::CreatePermissionError);
        assert_eq!(MessageType::from_u16(0x0119).unwrap(), MessageType::ChannelBindError);
    }
}
