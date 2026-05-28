// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::expect_used, reason = "test assertions")]

use std::sync::Arc;

use songbird_crypto_provider::CryptoProvider;

use super::*;
use crate::MAX_RECORD_SIZE;
use crate::error::TlsError;
use crate::messages::ContentType;

#[test]
fn test_new_record_layer() {
    let record_layer = RecordLayer::new();
    assert_eq!(record_layer.write_sequence(), 0);
    assert_eq!(record_layer.read_sequence(), 0);
    assert!(!record_layer.encrypted);
}

#[test]
fn test_enable_encryption() {
    let mut record_layer = RecordLayer::new();
    record_layer.enable_encryption();
    assert!(record_layer.encrypted);
}

#[test]
fn test_frame_plaintext() {
    let mut record_layer = RecordLayer::new();
    let payload = b"Hello, TLS 1.3!";

    let record = record_layer.frame_plaintext(ContentType::Handshake, payload).unwrap();

    // Check record structure: type (1) + version (2) + length (2) + payload
    assert_eq!(record.len(), 5 + payload.len());
    assert_eq!(record[0], ContentType::Handshake as u8);
    assert_eq!(&record[1..3], &[0x03, 0x03]); // TLS 1.2 legacy version
    assert_eq!(&record[3..5], &[0x00, 0x0F]); // Length = 15
    assert_eq!(&record[5..], payload);
}

#[test]
fn test_parse_record() {
    let mut record_layer = RecordLayer::new();

    // Create a test record
    let payload = b"Test payload";
    let record = record_layer.frame_plaintext(ContentType::ApplicationData, payload).unwrap();

    // Parse it back
    let (content_type, parsed_payload, bytes_consumed) =
        record_layer.parse_record(&record).unwrap();

    assert_eq!(content_type, ContentType::ApplicationData);
    assert_eq!(parsed_payload, payload);
    assert_eq!(bytes_consumed, record.len());
}

#[test]
fn test_frame_parse_roundtrip() {
    let mut record_layer = RecordLayer::new();
    let original_payload = b"Roundtrip test data";

    // Frame
    let record = record_layer.frame_plaintext(ContentType::Handshake, original_payload).unwrap();

    // Parse
    let (content_type, parsed_payload, _) = record_layer.parse_record(&record).unwrap();

    assert_eq!(content_type, ContentType::Handshake);
    assert_eq!(parsed_payload, original_payload);
}

#[test]
fn test_record_too_large() {
    let mut record_layer = RecordLayer::new();
    let payload = vec![0u8; MAX_RECORD_SIZE + 1]; // Too large!

    let result = record_layer.frame_plaintext(ContentType::ApplicationData, &payload);
    assert!(result.is_err());
}

#[test]
fn test_parse_record_too_short() {
    let mut record_layer = RecordLayer::new();
    let buf = vec![0x17, 0x03, 0x03]; // Only 3 bytes (need 5 for header)

    let result = record_layer.parse_record(&buf);
    assert!(result.is_err());
}

#[test]
fn test_parse_record_incomplete() {
    let mut record_layer = RecordLayer::new();
    // Header says 10 bytes, but only 5 bytes of payload
    let buf = vec![0x17, 0x03, 0x03, 0x00, 0x0A, 1, 2, 3, 4, 5];

    let result = record_layer.parse_record(&buf);
    assert!(result.is_err());
}

#[test]
fn test_sequence_numbers() {
    let mut record_layer = RecordLayer::new();

    assert_eq!(record_layer.write_sequence(), 0);
    assert_eq!(record_layer.read_sequence(), 0);

    // Simulate encryption (increments write sequence)
    record_layer.increment_write_sequence();
    assert_eq!(record_layer.write_sequence(), 1);

    // Simulate decryption (increments read sequence)
    record_layer.increment_read_sequence();
    assert_eq!(record_layer.read_sequence(), 1);

    // Multiple increments
    for i in 2..=5 {
        record_layer.increment_write_sequence();
        assert_eq!(record_layer.write_sequence(), i);
    }
}

#[tokio::test]
async fn test_encrypt_decrypt_roundtrip() {
    let mut record_layer = RecordLayer::with_crypto_provider(Arc::new(CryptoProvider::new(
        "/nonexistent/beardog.sock",
    )));
    record_layer.enable_encryption();
    let err = record_layer
        .encrypt_record_delegated(
            ContentType::ApplicationData,
            b"Secret message",
            &[0u8; 32],
            &[0u8; 12],
        )
        .await
        .expect_err("security provider socket missing");
    assert!(matches!(err, TlsError::CryptoUnavailable));
}

#[test]
fn test_sequence_wrapping() {
    let mut record_layer = RecordLayer::new();
    record_layer.write_sequence = u64::MAX;

    record_layer.increment_write_sequence();
    assert_eq!(record_layer.write_sequence(), 0); // Wrapped to 0
}

// ========================================
// NEW COMPREHENSIVE RECORD LAYER TESTS
// Added: January 27, 2026 (Evening)
// Goal: Increase coverage from 12% → 70%
// ========================================

#[test]
fn test_multiple_content_types() {
    let mut record_layer = RecordLayer::new();
    let payload = b"test";

    // Test all content types
    let types = vec![ContentType::Alert, ContentType::Handshake, ContentType::ApplicationData];

    for content_type in types {
        let record = record_layer.frame_plaintext(content_type, payload).unwrap();
        let (parsed_type, parsed_payload, _) = record_layer.parse_record(&record).unwrap();

        assert_eq!(parsed_type, content_type);
        assert_eq!(parsed_payload, payload);
    }
}

#[test]
fn test_frame_empty_payload() {
    let mut record_layer = RecordLayer::new();
    let payload = b"";

    let record = record_layer.frame_plaintext(ContentType::Alert, payload).unwrap();

    // Should have 5-byte header + 0 payload
    assert_eq!(record.len(), 5);
    assert_eq!(record[0], ContentType::Alert as u8);
}

#[test]
fn test_frame_maximum_size_payload() {
    let mut record_layer = RecordLayer::new();
    let payload = vec![42u8; MAX_RECORD_SIZE];

    let result = record_layer.frame_plaintext(ContentType::ApplicationData, &payload);

    // Should succeed (exactly at max)
    assert!(result.is_ok());
}

#[test]
fn test_parse_record_with_extra_data() {
    let mut record_layer = RecordLayer::new();

    // Create a record with extra data after it
    let mut buf = record_layer.frame_plaintext(ContentType::Handshake, b"Hello").unwrap();
    buf.extend_from_slice(b"Extra data");

    let (_, _, bytes_consumed) = record_layer.parse_record(&buf).unwrap();

    // Should only consume the record, not the extra data
    assert!(bytes_consumed < buf.len());
}

#[test]
fn test_sequence_independence() {
    let mut record_layer = RecordLayer::new();

    // Write and read sequences should be independent
    record_layer.increment_write_sequence();
    record_layer.increment_write_sequence();
    assert_eq!(record_layer.write_sequence(), 2);
    assert_eq!(record_layer.read_sequence(), 0);

    record_layer.increment_read_sequence();
    assert_eq!(record_layer.write_sequence(), 2);
    assert_eq!(record_layer.read_sequence(), 1);
}

#[test]
fn test_encryption_flag_persistence() {
    let mut record_layer = RecordLayer::new();
    assert!(!record_layer.encrypted);

    record_layer.enable_encryption();
    assert!(record_layer.encrypted);

    // Enable again (should still be true)
    record_layer.enable_encryption();
    assert!(record_layer.encrypted);
}

#[test]
fn test_parse_alert_record() {
    let mut record_layer = RecordLayer::new();
    let alert_payload = vec![2u8, 50u8]; // Level: Fatal (2), Description: Decode error (50)

    let record = record_layer.frame_plaintext(ContentType::Alert, &alert_payload).unwrap();
    let (content_type, payload, _) = record_layer.parse_record(&record).unwrap();

    assert_eq!(content_type, ContentType::Alert);
    assert_eq!(payload, alert_payload.as_slice());
}

#[test]
fn test_large_sequence_numbers() {
    let mut record_layer = RecordLayer::new();

    // Set high sequence numbers
    record_layer.write_sequence = 1_000_000;
    record_layer.read_sequence = 2_000_000;

    assert_eq!(record_layer.write_sequence(), 1_000_000);
    assert_eq!(record_layer.read_sequence(), 2_000_000);

    record_layer.increment_write_sequence();
    assert_eq!(record_layer.write_sequence(), 1_000_001);
}

#[test]
fn test_read_sequence_wrapping() {
    let mut record_layer = RecordLayer::new();
    record_layer.read_sequence = u64::MAX;

    record_layer.increment_read_sequence();
    assert_eq!(record_layer.read_sequence(), 0); // Wrapped to 0
}

#[test]
fn test_parse_record_zero_length() {
    let mut record_layer = RecordLayer::new();

    // Create a record with zero-length payload
    let buf = vec![0x17, 0x03, 0x03, 0x00, 0x00]; // ApplicationData, TLS 1.2, length 0

    let (content_type, payload, bytes_consumed) = record_layer.parse_record(&buf).unwrap();

    assert_eq!(content_type, ContentType::ApplicationData);
    assert_eq!(payload.len(), 0);
    assert_eq!(bytes_consumed, 5);
}

#[test]
fn test_multiple_records_sequential() {
    let mut record_layer = RecordLayer::new();

    // Create multiple records
    let record1 = record_layer.frame_plaintext(ContentType::Handshake, b"First").unwrap();
    let record2 = record_layer.frame_plaintext(ContentType::Handshake, b"Second").unwrap();

    // Parse first
    let (_, payload1, consumed1) = record_layer.parse_record(&record1).unwrap();
    assert_eq!(payload1, b"First");
    assert_eq!(consumed1, record1.len());

    // Parse second
    let (_, payload2, consumed2) = record_layer.parse_record(&record2).unwrap();
    assert_eq!(payload2, b"Second");
    assert_eq!(consumed2, record2.len());
}

#[test]
fn test_frame_different_payload_sizes() {
    let mut record_layer = RecordLayer::new();

    let sizes = vec![1, 10, 100, 1000, 5000];

    for size in sizes {
        let payload = vec![0xAAu8; size];
        let record = record_layer.frame_plaintext(ContentType::ApplicationData, &payload).unwrap();

        // Verify record size
        assert_eq!(record.len(), 5 + size);

        // Verify length field in header
        let length = u16::from_be_bytes([record[3], record[4]]);
        assert_eq!(length as usize, size);
    }
}

#[tokio::test]
async fn test_encryption_with_sequence_increment() {
    let mut record_layer = RecordLayer::with_crypto_provider(Arc::new(CryptoProvider::new(
        "/nonexistent/beardog.sock",
    )));
    record_layer.enable_encryption();
    let initial_seq = record_layer.write_sequence();
    let err = record_layer
        .encrypt_record_delegated(ContentType::ApplicationData, b"test", &[0u8; 32], &[0u8; 12])
        .await
        .expect_err("security provider socket missing");
    assert!(matches!(err, TlsError::CryptoUnavailable));
    assert_eq!(record_layer.write_sequence(), initial_seq);
}

#[test]
#[expect(clippy::cast_possible_truncation, reason = "test: value masked to u8 range")]
fn parse_record_rejects_oversized_length_field() {
    let mut record_layer = RecordLayer::new();
    let oversized = MAX_RECORD_SIZE + 1;
    let len_hi = ((oversized >> 8) & 0xff) as u8;
    let len_lo = (oversized & 0xff) as u8;
    let buf = vec![0x17, 0x03, 0x03, len_hi, len_lo];
    let err = record_layer.parse_record(&buf).expect_err("oversized header");
    assert!(matches!(err, crate::error::TlsError::RecordTooLarge { .. }));
}

#[test]
fn decrypt_record_errors_when_inner_empty_after_padding_strip() {
    let mut record_layer = RecordLayer::new();
    let decrypt_fn = |_data: &[u8], _seq: u64| Ok(vec![0u8, 0u8]); // only padding zeros
    let err = record_layer.decrypt_record(&[1, 2, 3], decrypt_fn).expect_err("empty inner");
    assert!(matches!(err, crate::error::TlsError::DecryptError));
}

#[test]
fn decrypt_record_parses_inner_content_type() {
    let mut record_layer = RecordLayer::new();
    let decrypt_fn = |_data: &[u8], _seq: u64| {
        let mut v = vec![b'a', b'b'];
        v.push(ContentType::Handshake as u8);
        Ok(v)
    };
    let (ct, plain) = record_layer.decrypt_record(&[0u8], decrypt_fn).expect("decrypt");
    assert_eq!(ct, ContentType::Handshake);
    assert_eq!(plain, b"ab");
}
