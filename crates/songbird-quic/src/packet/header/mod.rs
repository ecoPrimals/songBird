// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC packet headers (RFC 9000 Sections 17.2 and 17.3).
//!
//! Long headers are used during the handshake (Initial, Handshake, 0-RTT, Retry).
//! Short headers (1-RTT) are used after the handshake is complete.

use crate::error::Result;

/// QUIC version 1 (RFC 9000).
pub const QUIC_VERSION_1: u32 = 0x0000_0001;

/// Long header packet types (RFC 9000 Section 17.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LongPacketType {
    /// Initial packet (handshake start, carries CRYPTO frames).
    Initial = 0x00,
    /// 0-RTT packet (early data).
    ZeroRtt = 0x01,
    /// Handshake packet (handshake continuation).
    Handshake = 0x02,
    /// Retry packet (address validation).
    Retry = 0x03,
}

impl LongPacketType {
    /// Decode from the two type bits in the first byte.
    ///
    /// # Errors
    ///
    /// Never returns `Err`; all two-bit patterns map to a variant.
    pub fn from_bits(bits: u8) -> Result<Self> {
        match bits & 0x03 {
            0x00 => Ok(Self::Initial),
            0x01 => Ok(Self::ZeroRtt),
            0x02 => Ok(Self::Handshake),
            0x03 => Ok(Self::Retry),
            _ => unreachable!(),
        }
    }
}

/// Parsed QUIC long header (RFC 9000 Section 17.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LongHeader {
    /// Fixed bit (must be 1 in QUIC v1).
    pub fixed_bit: bool,
    /// Packet type.
    pub packet_type: LongPacketType,
    /// Reserved bits (2 bits, must be 0 before header protection removal).
    pub reserved_bits: u8,
    /// Packet number length minus one (2 bits, 0..=3 → 1..=4 bytes).
    pub pn_length: u8,
    /// Version field.
    pub version: u32,
    /// Destination Connection ID.
    pub dcid: Vec<u8>,
    /// Source Connection ID.
    pub scid: Vec<u8>,
    /// Token (Initial packets only).
    pub token: Vec<u8>,
    /// Payload length (VarInt-encoded in wire format).
    pub payload_length: u64,
    /// Packet number (truncated, 1-4 bytes after header protection removal).
    pub packet_number: u32,
}

/// Parsed QUIC short header (1-RTT, RFC 9000 Section 17.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortHeader {
    /// Fixed bit (must be 1 in QUIC v1).
    pub fixed_bit: bool,
    /// Spin bit.
    pub spin_bit: bool,
    /// Reserved bits (2 bits).
    pub reserved_bits: u8,
    /// Key phase bit.
    pub key_phase: bool,
    /// Packet number length minus one (2 bits).
    pub pn_length: u8,
    /// Destination Connection ID.
    pub dcid: Vec<u8>,
    /// Packet number (truncated).
    pub packet_number: u32,
}

/// A decoded QUIC packet header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketHeader {
    /// Long header (Initial, 0-RTT, Handshake, Retry).
    Long(LongHeader),
    /// Short header (1-RTT).
    Short(ShortHeader),
}

impl PacketHeader {
    /// Returns `true` if this is a long header.
    #[must_use]
    pub const fn is_long(&self) -> bool {
        matches!(self, Self::Long(_))
    }

    /// Returns `true` if this is a short header.
    #[must_use]
    pub const fn is_short(&self) -> bool {
        matches!(self, Self::Short(_))
    }
}

/// Returns `true` if the first byte indicates a long header (form bit = 1).
#[must_use]
pub const fn is_long_header(first_byte: u8) -> bool {
    first_byte & 0x80 != 0
}

mod long_codec;
mod short_codec;

pub use long_codec::{decode_long_header, encode_long_header};
pub use short_codec::{decode_short_header, encode_short_header};

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    fn make_initial_header() -> LongHeader {
        LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::Initial,
            reserved_bits: 0,
            pn_length: 1,
            version: QUIC_VERSION_1,
            dcid: vec![0x83, 0x94, 0xc8, 0xf0, 0x3e, 0x51, 0x57, 0x08],
            scid: vec![],
            token: vec![],
            payload_length: 1162,
            packet_number: 0,
        }
    }

    #[test]
    fn long_header_roundtrip() {
        let header = make_initial_header();
        let mut buf = [0u8; 256];
        let written = encode_long_header(&header, &mut buf).unwrap();
        let (decoded, consumed) = decode_long_header(&buf[..written]).unwrap();
        assert_eq!(consumed, written);
        assert_eq!(decoded.packet_type, LongPacketType::Initial);
        assert_eq!(decoded.version, QUIC_VERSION_1);
        assert_eq!(decoded.dcid, header.dcid);
        assert_eq!(decoded.scid, header.scid);
        assert_eq!(decoded.token, header.token);
        assert_eq!(decoded.payload_length, header.payload_length);
        assert_eq!(decoded.packet_number, header.packet_number);
    }

    #[test]
    fn handshake_header_roundtrip() {
        let header = LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::Handshake,
            reserved_bits: 0,
            pn_length: 2,
            version: QUIC_VERSION_1,
            dcid: vec![0x01, 0x02, 0x03, 0x04],
            scid: vec![0x05, 0x06],
            token: vec![],
            payload_length: 500,
            packet_number: 1,
        };
        let mut buf = [0u8; 256];
        let written = encode_long_header(&header, &mut buf).unwrap();
        let (decoded, _) = decode_long_header(&buf[..written]).unwrap();
        assert_eq!(decoded.packet_type, LongPacketType::Handshake);
        assert_eq!(decoded.pn_length, 2);
        assert_eq!(decoded.packet_number, 1);
    }

    #[test]
    fn short_header_roundtrip() {
        let header = ShortHeader {
            fixed_bit: true,
            spin_bit: false,
            reserved_bits: 0,
            key_phase: false,
            pn_length: 2,
            dcid: vec![0xAA, 0xBB, 0xCC, 0xDD],
            packet_number: 42,
        };
        let mut buf = [0u8; 64];
        let written = encode_short_header(&header, &mut buf).unwrap();
        let (decoded, consumed) = decode_short_header(&buf[..written], 4).unwrap();
        assert_eq!(consumed, written);
        assert_eq!(decoded.dcid, header.dcid);
        assert_eq!(decoded.packet_number, 42);
        assert!(!decoded.spin_bit);
        assert!(!decoded.key_phase);
    }

    #[test]
    fn is_long_header_detection() {
        assert!(is_long_header(0xC0));
        assert!(is_long_header(0x80));
        assert!(!is_long_header(0x40));
        assert!(!is_long_header(0x00));
    }

    #[test]
    fn packet_type_from_bits() {
        assert_eq!(LongPacketType::from_bits(0).unwrap(), LongPacketType::Initial);
        assert_eq!(LongPacketType::from_bits(1).unwrap(), LongPacketType::ZeroRtt);
        assert_eq!(LongPacketType::from_bits(2).unwrap(), LongPacketType::Handshake);
        assert_eq!(LongPacketType::from_bits(3).unwrap(), LongPacketType::Retry);
    }

    #[test]
    fn retry_header_has_no_pn_or_length() {
        let header = LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::Retry,
            reserved_bits: 0,
            pn_length: 1,
            version: QUIC_VERSION_1,
            dcid: vec![0x01],
            scid: vec![0x02],
            token: vec![],
            payload_length: 0,
            packet_number: 0,
        };
        let mut buf = [0u8; 64];
        let written = encode_long_header(&header, &mut buf).unwrap();
        let (decoded, _) = decode_long_header(&buf[..written]).unwrap();
        assert_eq!(decoded.packet_type, LongPacketType::Retry);
        assert_eq!(decoded.payload_length, 0);
        assert_eq!(decoded.packet_number, 0);
    }

    #[test]
    fn truncated_long_header_errors() {
        assert!(decode_long_header(&[0xC0, 0x00]).is_err());
    }

    #[test]
    fn truncated_short_header_errors() {
        assert!(decode_short_header(&[], 4).is_err());
        assert!(decode_short_header(&[0x40], 4).is_err());
    }

    #[test]
    fn packet_header_enum_helpers() {
        let long = PacketHeader::Long(make_initial_header());
        assert!(long.is_long());
        assert!(!long.is_short());

        let short = PacketHeader::Short(ShortHeader {
            fixed_bit: true,
            spin_bit: false,
            reserved_bits: 0,
            key_phase: false,
            pn_length: 1,
            dcid: vec![],
            packet_number: 0,
        });
        assert!(short.is_short());
        assert!(!short.is_long());
    }

    #[test]
    fn initial_with_token_roundtrip() {
        let header = LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::Initial,
            reserved_bits: 0,
            pn_length: 2,
            version: QUIC_VERSION_1,
            dcid: vec![0x01, 0x02],
            scid: vec![0x03],
            token: vec![0xDE, 0xAD, 0xBE, 0xEF],
            payload_length: 100,
            packet_number: 5,
        };
        let mut buf = [0u8; 256];
        let written = encode_long_header(&header, &mut buf).unwrap();
        let (decoded, _) = decode_long_header(&buf[..written]).unwrap();
        assert_eq!(decoded.token, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn four_byte_packet_number() {
        let header = LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::Handshake,
            reserved_bits: 0,
            pn_length: 4,
            version: QUIC_VERSION_1,
            dcid: vec![],
            scid: vec![],
            token: vec![],
            payload_length: 50,
            packet_number: 0x00FF_AABB,
        };
        let mut buf = [0u8; 128];
        let written = encode_long_header(&header, &mut buf).unwrap();
        let (decoded, _) = decode_long_header(&buf[..written]).unwrap();
        assert_eq!(decoded.packet_number, 0x00FF_AABB);
        assert_eq!(decoded.pn_length, 4);
    }

    #[test]
    fn decode_long_rejects_short_form_first_byte() {
        let err = decode_long_header(&[0x40, 0, 0, 0, 1, 0, 0])
            .expect_err("short form must not decode as long header");
        assert!(err.to_string().contains("Not a long header"), "unexpected error: {err}");
    }

    #[test]
    fn decode_short_rejects_long_form_first_byte() {
        let err = decode_short_header(&[0xC0], 0).expect_err("long form must not decode as short");
        assert!(err.to_string().contains("Not a short header"), "unexpected error: {err}");
    }

    #[test]
    fn zero_rtt_long_header_roundtrip() {
        let header = LongHeader {
            fixed_bit: true,
            packet_type: LongPacketType::ZeroRtt,
            reserved_bits: 0,
            pn_length: 1,
            version: QUIC_VERSION_1,
            dcid: vec![0x01, 0x02],
            scid: vec![0x03],
            token: vec![],
            payload_length: 200,
            packet_number: 9,
        };
        let mut buf = [0u8; 128];
        let written = encode_long_header(&header, &mut buf).expect("encode 0-RTT long header");
        let (decoded, _) = decode_long_header(&buf[..written]).expect("decode 0-RTT long header");
        assert_eq!(decoded.packet_type, LongPacketType::ZeroRtt, "packet type must stay 0-RTT");
        assert!(decoded.token.is_empty(), "0-RTT must not carry Initial token");
        assert_eq!(decoded.payload_length, 200);
        assert_eq!(decoded.packet_number, 9);
    }

    #[test]
    fn encode_long_header_buffer_too_small_for_version() {
        let header = make_initial_header();
        let mut buf = [0u8; 1];
        let err = encode_long_header(&header, &mut buf).expect_err("buffer must be too small");
        assert!(
            err.to_string().contains("version") || err.to_string().contains("small"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn encode_short_header_buffer_too_small_for_pn() {
        let header = ShortHeader {
            fixed_bit: true,
            spin_bit: false,
            reserved_bits: 0,
            key_phase: false,
            pn_length: 4,
            dcid: vec![0x01],
            packet_number: 0x1122_3344,
        };
        let mut buf = [0u8; 3];
        let err =
            encode_short_header(&header, &mut buf).expect_err("must fail for undersized buffer");
        assert!(
            err.to_string().contains("PN") || err.to_string().contains("small"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn short_header_spin_and_key_phase_roundtrip() {
        let header = ShortHeader {
            fixed_bit: true,
            spin_bit: true,
            reserved_bits: 0b11,
            key_phase: true,
            pn_length: 1,
            dcid: vec![0xAB; 8],
            packet_number: 7,
        };
        let mut buf = [0u8; 32];
        let written = encode_short_header(&header, &mut buf).expect("encode short header");
        let (decoded, _) = decode_short_header(&buf[..written], 8).expect("decode short header");
        assert!(decoded.spin_bit, "spin bit must round-trip");
        assert!(decoded.key_phase, "key phase must round-trip");
        assert_eq!(decoded.reserved_bits, 0b11, "reserved bits must round-trip");
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn decode_initial_packet_truncated_token_errors() {
        // Long header Initial: after SCID, VarInt token length claims 4 bytes but only 1 byte follows.
        let buf = [
            0xC0, // long + fixed + Initial + PN length 1
            0x00, 0x00, 0x00, 0x01, // version 1
            0x00, // DCID len 0
            0x00, // SCID len 0
            0x04, // token length VarInt = 4
            0xAA, // only one byte of token
        ];
        let err = decode_long_header(&buf).expect_err("truncated Initial token must fail");
        assert!(
            err.to_string().contains("token") || err.to_string().contains("Truncated"),
            "unexpected: {err}"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn decode_long_header_truncated_packet_number_errors() {
        // Handshake: version 1, empty CIDs, payload length 0 (one-byte varint), then PN missing.
        let buf = [
            0xE0, // long + fixed + Handshake (type 0b10) + PN length 1
            0x00, 0x00, 0x00, 0x01, // version 1
            0x00, // DCID len 0
            0x00, // SCID len 0
            0x00, // payload length VarInt = 0
        ];
        let err = decode_long_header(&buf).expect_err("truncated PN must fail");
        assert!(
            err.to_string().contains("packet number") || err.to_string().contains("Truncated"),
            "unexpected: {err}"
        );
    }
}
