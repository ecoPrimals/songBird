// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Hand-crafted edge cases for [`RecordLayer::parse_record`] (fuzz-style, no external harness).

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::RecordLayer;
use crate::MAX_RECORD_SIZE;
use crate::messages::ContentType;

fn parse_or_err(rl: &mut RecordLayer, buf: &[u8]) {
    let _ = rl.parse_record(buf);
}

#[test]
fn parse_record_random_one_byte_headers_never_panic() {
    let mut rl = RecordLayer::new();
    for b in 0u8..=255 {
        parse_or_err(&mut rl, &[b]);
    }
}

#[test]
fn parse_record_malformed_headers_handcrafted() {
    let mut rl = RecordLayer::new();
    let samples: &[&[u8]] = &[
        &[0xFF, 0x03, 0x03, 0x00, 0x00],
        &[0x00, 0x00, 0x00, 0x00, 0x00],
        &[23, 0x03, 0x04, 0xFF, 0xFF],
        &[22, 0x01, 0x02, 0x00, 0x01, 0x00],
    ];
    for buf in samples {
        let _ = rl.parse_record(buf);
    }
}

#[test]
fn parse_record_invalid_content_type_maps_to_invalid_enum() {
    let mut rl = RecordLayer::new();
    let buf = vec![0xFF, 0x03, 0x03, 0x00, 0x00];
    let (ct, payload, n) = rl.parse_record(&buf).unwrap();
    assert_eq!(ct, ContentType::Invalid);
    assert!(payload.is_empty());
    assert_eq!(n, 5);
}

#[test]
fn parse_record_truncated_after_header_various_lengths() {
    let mut rl = RecordLayer::new();
    for len in 0u8..5 {
        let mut v = vec![0x17u8, 0x03, 0x03, 0x00, 0x10];
        v.truncate(len as usize);
        assert!(rl.parse_record(&v).is_err());
    }
    // Header claims 10 bytes payload, buffer ends at 7
    let buf = vec![0x17, 0x03, 0x03, 0x00, 0x0A, 1, 2, 3];
    assert!(rl.parse_record(&buf).is_err());
}

#[test]
#[expect(clippy::cast_possible_truncation, reason = "test: MAX_RECORD_SIZE fits in u16")]
fn parse_record_exact_max_length_succeeds_when_buffer_complete() {
    let mut rl = RecordLayer::new();
    let mut buf = vec![0x16u8, 0x03, 0x03];
    let len = MAX_RECORD_SIZE;
    buf.extend_from_slice(&(len as u16).to_be_bytes());
    buf.extend(std::iter::repeat_n(0u8, len));
    assert_eq!(buf.len(), 5 + len);
    let (ct, payload, consumed) = rl.parse_record(&buf).unwrap();
    assert_eq!(ct, ContentType::Handshake);
    assert_eq!(payload.len(), len);
    assert_eq!(consumed, buf.len());
}

#[test]
#[allow(
    clippy::cast_possible_truncation,
    reason = "test: wire length encodes oversized record (truncation intentional)"
)]
fn parse_record_length_max_plus_one_errors_even_with_enough_bytes() {
    let mut rl = RecordLayer::new();
    let len = MAX_RECORD_SIZE + 1;
    let mut buf = vec![0x17u8, 0x03, 0x03];
    buf.extend_from_slice(&(len as u16).to_be_bytes());
    buf.extend(std::iter::repeat_n(0u8, len));
    let err = rl.parse_record(&buf).unwrap_err();
    assert!(matches!(err, crate::error::TlsError::RecordTooLarge { .. }));
}

#[test]
fn parse_record_empty_application_data_payload() {
    let mut rl = RecordLayer::new();
    let buf = vec![0x17, 0x03, 0x03, 0x00, 0x00];
    let (ct, pl, n) = rl.parse_record(&buf).unwrap();
    assert_eq!(ct, ContentType::ApplicationData);
    assert!(pl.is_empty());
    assert_eq!(n, 5);
}
