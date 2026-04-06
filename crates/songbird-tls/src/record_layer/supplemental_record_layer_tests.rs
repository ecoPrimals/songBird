// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::RecordLayer;
use crate::MAX_RECORD_SIZE;
use crate::messages::ContentType;

#[test]
fn default_matches_new() {
    let a = RecordLayer::new();
    let b = RecordLayer::default();
    assert_eq!(a.write_sequence(), b.write_sequence());
    assert_eq!(a.read_sequence(), b.read_sequence());
}

#[test]
fn frame_plaintext_change_cipher_spec() {
    let mut rl = RecordLayer::new();
    let rec = rl.frame_plaintext(ContentType::ChangeCipherSpec, &[1]).unwrap();
    assert_eq!(rec[0], 20);
    assert_eq!(&rec[5..], &[1]);
}

#[test]
fn encrypt_record_does_not_increment_on_encrypt_failure() {
    let mut rl = RecordLayer::new();
    let seq_before = rl.write_sequence();
    let encrypt_fn =
        |_data: &[u8], _seq: u64| Err(crate::error::TlsError::CryptoError("fail".into()));
    let err = rl.encrypt_record(ContentType::Handshake, b"x", encrypt_fn).unwrap_err();
    assert!(matches!(err, crate::error::TlsError::CryptoError(_)));
    assert_eq!(rl.write_sequence(), seq_before);
}

#[test]
fn parse_record_legacy_version_bytes_preserved_in_wire() {
    let mut rl = RecordLayer::new();
    let mut buf = vec![0x16u8, 0x03, 0x01];
    buf.extend_from_slice(&[0x00, 0x02, 0xab, 0xcd]);
    let (ct, pl, n) = rl.parse_record(&buf).unwrap();
    assert_eq!(ct, ContentType::Handshake);
    assert_eq!(pl, vec![0xab, 0xcd]);
    assert_eq!(n, buf.len());
}

#[test]
fn decrypt_record_strips_trailing_zeros_after_content_type_byte() {
    let mut rl = RecordLayer::new();
    let decrypt_fn = |_data: &[u8], _seq: u64| {
        // TLSInnerPlaintext: content || ContentType || zero padding (RFC 8446)
        let mut v = vec![b'h', b'i', ContentType::Handshake as u8];
        v.push(0);
        v.push(0);
        Ok(v)
    };
    let (ct, plain) = rl.decrypt_record(&[0u8], decrypt_fn).unwrap();
    assert_eq!(ct, ContentType::Handshake);
    assert_eq!(plain, b"hi");
}

#[test]
fn frame_plaintext_rejects_len_overflow_for_u16() {
    let mut rl = RecordLayer::new();
    let payload = vec![0u8; MAX_RECORD_SIZE + 1];
    assert!(rl.frame_plaintext(ContentType::ApplicationData, &payload).is_err());
}
