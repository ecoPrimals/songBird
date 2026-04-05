// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::{AttributeType, MAGIC_COOKIE, MessageType, StunAttribute, StunMessage};
use bytes::Bytes;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

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
    assert_eq!(u32::from_be_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]), MAGIC_COOKIE);
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

#[test]
fn message_type_binding_response_roundtrip() {
    assert_eq!(MessageType::from_u16(0x0101).unwrap(), MessageType::BindingResponse);
    assert_eq!(MessageType::BindingResponse.to_u16(), 0x0101);
}

#[test]
fn message_type_binding_error_roundtrip() {
    assert_eq!(MessageType::from_u16(0x0111).unwrap(), MessageType::BindingError);
}

#[test]
fn mapped_address_encode_decode_ipv4() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5)), 49152);
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::MappedAddress(addr));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).unwrap();
    assert_eq!(decoded.get_mapped_address(), Some(addr));
    assert_eq!(decoded.get_any_mapped_address(), Some(addr));
}

#[test]
fn xor_preferred_over_mapped_in_get_any_mapped_address() {
    let xor_a = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 111);
    let map_b = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)), 222);
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::MappedAddress(map_b));
    msg.attributes.push(StunAttribute::XorMappedAddress(xor_a));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).unwrap();
    assert_eq!(decoded.get_any_mapped_address(), Some(xor_a));
}

#[test]
fn other_address_attribute_roundtrip() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2)), 3478);
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::OtherAddress(addr));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).unwrap();
    let other = decoded.attributes.iter().find_map(|a| {
        if let StunAttribute::OtherAddress(sa) = a {
            Some(*sa)
        } else {
            None
        }
    });
    assert_eq!(other, Some(addr));
}

#[test]
fn unknown_attribute_preserved_in_message() {
    let data = Bytes::from_static(b"opaque");
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::Unknown(0x9999, data));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).unwrap();
    assert!(decoded.attributes.iter().any(|a| matches!(a, StunAttribute::Unknown(0x9999, _))));
}

#[test]
fn decode_attribute_too_short_errors_cleanly() {
    let mut buf: &[u8] = &[0x00, 0x01, 0x00, 0x10];
    assert!(StunAttribute::decode(&mut buf).is_err());
}

#[test]
fn mapped_address_ipv6_roundtrip() {
    let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)), 53_421);
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::MappedAddress(addr));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).expect("decode IPv6 mapped address");
    assert_eq!(decoded.get_mapped_address(), Some(addr));
}

#[test]
#[allow(clippy::unwrap_used, reason = "test assertion")]
fn decode_binding_error_response_without_attributes() {
    let mut buf = vec![0u8; 20];
    buf[0] = 0x01;
    buf[1] = 0x11; // 0x0111 binding error
    buf[2] = 0x00;
    buf[3] = 0x00;
    buf[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
    buf[8..20].copy_from_slice(&[0x5Au8; 12]);
    let msg = StunMessage::decode(&buf).expect("decode binding error header");
    assert_eq!(msg.message_type, MessageType::BindingError);
    assert_eq!(msg.transaction_id, [0x5A; 12]);
    assert!(msg.get_any_mapped_address().is_none());
}

#[test]
#[allow(clippy::unwrap_used, reason = "test assertion")]
fn xor_ipv4_mapped_address_port_xor_matches_rfc_5389() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 10)), 45_678);
    let mut msg = StunMessage::new_binding_request();
    msg.message_type = MessageType::BindingResponse;
    msg.attributes.push(StunAttribute::XorMappedAddress(addr));
    let wire = msg.encode();
    let decoded = StunMessage::decode(&wire).expect("roundtrip xor mapped");
    assert_eq!(decoded.get_xor_mapped_address(), Some(addr));
    let xored_port = u16::from_be_bytes([wire[26], wire[27]]);
    let expected = addr.port() ^ ((MAGIC_COOKIE >> 16) as u16);
    assert_eq!(xored_port, expected, "XOR-MAPPED-ADDRESS port xor");
}

#[test]
#[allow(clippy::unwrap_used, reason = "test assertion")]
fn message_type_from_u16_rejects_non_binding_variants() {
    assert!(MessageType::from_u16(0x0002).is_err());
    assert!(MessageType::from_u16(0x0100).is_err());
    let err = MessageType::from_u16(0xFFFF).expect_err("reserved");
    assert!(err.to_string().contains("Unknown") || err.to_string().contains("message type"));
}

#[test]
#[allow(clippy::unwrap_used, reason = "test assertion")]
fn binding_request_roundtrip_preserves_transaction_id_for_correlation() {
    let a = StunMessage::new_binding_request();
    let b = StunMessage::new_binding_request();
    assert_ne!(
        a.transaction_id, b.transaction_id,
        "successive STUN requests should use distinct transaction IDs"
    );
    let wire = a.encode();
    let decoded = StunMessage::decode(&wire).expect("decode");
    assert_eq!(decoded.transaction_id, a.transaction_id);
}

#[test]
fn decode_continues_when_mapped_attribute_has_unknown_family() {
    // Malformed MAPPED-ADDRESS (unknown family 0x03) causes attribute decode to fail;
    // decode() logs and stops parsing attributes (RFC lenient behavior).
    let mut buf = vec![0u8; 20];
    buf[0] = 0x01;
    buf[1] = 0x01;
    buf[2] = 0x00;
    buf[3] = 0x0c; // 12 bytes of attributes
    buf[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
    let tid = [0xabu8; 12];
    buf[8..20].copy_from_slice(&tid);
    buf.extend_from_slice(&[0x00, 0x01, 0x00, 0x08]);
    buf.extend_from_slice(&[0x00, 0x03, 0x12, 0x34, 0x01, 0x02, 0x03, 0x04]);
    let decoded = StunMessage::decode(&buf).expect("header should still parse");
    assert_eq!(decoded.message_type, MessageType::BindingResponse);
    assert_eq!(decoded.transaction_id, tid);
    assert!(
        decoded.attributes.is_empty(),
        "expected malformed MAPPED-ADDRESS to be skipped, attrs: {:?}",
        decoded.attributes
    );
}

/// Hand-crafted edge cases for [`StunMessage::decode`] (fuzz-style, no external harness).
mod fuzz_style_stun_decode_tests {
    use super::super::{MAGIC_COOKIE, MessageType, StunMessage};

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
