// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS 1.3 Record Layer
//!
//! Handles record framing, encryption, and decryption per RFC 8446 Section 5.
//!
//! ## Record Format
//!
//! ```text
//! struct {
//!     ContentType type;           // 1 byte
//!     ProtocolVersion legacy_record_version = 0x0303; // 2 bytes (TLS 1.2)
//!     uint16 length;              // 2 bytes
//!     opaque fragment[length];    // variable length (encrypted after handshake)
//! } TLSPlaintext;
//! ```

use crate::codec::bytes::{read_u8, read_u16, write_u8, write_u16};
use crate::error::{Result, TlsError};
use crate::messages::ContentType;
use crate::{MAX_RECORD_SIZE, TLS_VERSION_1_2};

/// TLS Record Layer
///
/// Handles framing, encryption, and decryption of TLS records.
pub struct RecordLayer {
    /// Sequence number for outgoing records (for nonce construction)
    write_sequence: u64,

    /// Sequence number for incoming records (for nonce construction)
    read_sequence: u64,

    /// Are we in encrypted mode? (after handshake)
    encrypted: bool,
}

impl RecordLayer {
    /// Create a new `RecordLayer` in plaintext mode
    #[must_use]
    pub const fn new() -> Self {
        Self {
            write_sequence: 0,
            read_sequence: 0,
            encrypted: false,
        }
    }

    /// Enable encryption (called after handshake completion)
    pub const fn enable_encryption(&mut self) {
        self.encrypted = true;
        // Note: Sequence numbers are NOT reset when enabling encryption
        // They continue from handshake phase
    }

    /// Get the current write sequence number
    #[must_use]
    pub const fn write_sequence(&self) -> u64 {
        self.write_sequence
    }

    /// Get the current read sequence number
    #[must_use]
    pub const fn read_sequence(&self) -> u64 {
        self.read_sequence
    }

    /// Increment write sequence number
    const fn increment_write_sequence(&mut self) {
        self.write_sequence = self.write_sequence.wrapping_add(1);
    }

    /// Increment read sequence number
    const fn increment_read_sequence(&mut self) {
        self.read_sequence = self.read_sequence.wrapping_add(1);
    }

    /// Frame a plaintext message into a TLS record
    ///
    /// This creates the 5-byte header + payload.
    /// Does NOT encrypt (encryption is handled separately).
    ///
    /// # Errors
    ///
    /// Returns an error if payload exceeds `MAX_RECORD_SIZE` or length truncation occurs.
    pub fn frame_plaintext(
        &mut self,
        content_type: ContentType,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        // Validate payload length
        if payload.len() > MAX_RECORD_SIZE {
            return Err(TlsError::RecordTooLarge {
                size: payload.len(),
            });
        }

        let mut record = Vec::with_capacity(5 + payload.len());

        // Content type (1 byte)
        write_u8(&mut record, content_type.into());

        // Legacy record version (2 bytes) - always 0x0303 (TLS 1.2) for compatibility
        write_u16(&mut record, TLS_VERSION_1_2);

        // Length (2 bytes)
        write_u16(
            &mut record,
            u16::try_from(payload.len()).map_err(|_| TlsError::RecordTooLarge {
                size: payload.len(),
            })?,
        );

        // Payload
        record.extend_from_slice(payload);

        Ok(record)
    }

    /// Parse a TLS record from bytes
    ///
    /// Returns: (`content_type`, payload, `bytes_consumed`)
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short, record is too large, or incomplete.
    pub fn parse_record(&mut self, buf: &[u8]) -> Result<(ContentType, Vec<u8>, usize)> {
        if buf.len() < 5 {
            return Err(TlsError::ProtocolError(
                "Record too short: need at least 5 bytes for header".to_string(),
            ));
        }

        let mut offset = 0;

        // Content type (1 byte)
        let content_type = ContentType::from(read_u8(buf, &mut offset)?);

        // Legacy record version (2 bytes) - we don't strictly validate this
        let _legacy_version = read_u16(buf, &mut offset)?;

        // Length (2 bytes)
        let length = read_u16(buf, &mut offset)? as usize;

        // Validate length
        if length > MAX_RECORD_SIZE {
            return Err(TlsError::RecordTooLarge {
                size: length,
            });
        }

        // Check if we have the full payload
        if offset + length > buf.len() {
            return Err(TlsError::ProtocolError(format!(
                "Incomplete record: need {} bytes, have {}",
                length,
                buf.len() - offset
            )));
        }

        // Extract payload
        let payload = buf[offset..offset + length].to_vec();
        offset += length;

        Ok((content_type, payload, offset))
    }

    /// Encrypt a TLS record (Application Data)
    ///
    /// In TLS 1.3, the actual content type is hidden inside the encrypted payload.
    /// The record content type is always `ApplicationData` (23).
    ///
    /// Format of encrypted payload:
    /// ```text
    /// struct {
    ///     opaque content[length];
    ///     ContentType type;        // Actual content type
    ///     uint8 zeros[length_of_padding];
    /// } TLSInnerPlaintext;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if encryption or framing fails.
    pub fn encrypt_record(
        &mut self,
        content_type: ContentType,
        plaintext: &[u8],
        encrypt_fn: impl FnOnce(&[u8], u64) -> Result<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        // Build inner plaintext: content + content_type + padding
        let mut inner = Vec::with_capacity(plaintext.len() + 1);
        inner.extend_from_slice(plaintext);
        inner.push(content_type.into()); // Actual content type
        // No padding for now (can be added later for traffic analysis resistance)

        // Encrypt the inner plaintext
        let ciphertext = encrypt_fn(&inner, self.write_sequence)?;

        // Increment sequence number
        self.increment_write_sequence();

        // Frame as ApplicationData record
        self.frame_plaintext(ContentType::ApplicationData, &ciphertext)
    }

    /// Decrypt a TLS record (Application Data)
    ///
    /// Extracts the hidden content type from the end of the decrypted payload.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails or payload is invalid.
    pub fn decrypt_record(
        &mut self,
        ciphertext: &[u8],
        decrypt_fn: impl FnOnce(&[u8], u64) -> Result<Vec<u8>>,
    ) -> Result<(ContentType, Vec<u8>)> {
        // Decrypt the ciphertext
        let mut inner = decrypt_fn(ciphertext, self.read_sequence)?;

        // Increment sequence number
        self.increment_read_sequence();

        // Extract content type from the end (remove padding zeros first)
        while !inner.is_empty() && inner[inner.len() - 1] == 0 {
            inner.pop();
        }

        if inner.is_empty() {
            return Err(TlsError::DecryptError);
        }

        // Last byte is the actual content type
        let content_type_byte = inner.pop().ok_or(TlsError::DecryptError)?;
        let content_type = ContentType::from(content_type_byte);

        Ok((content_type, inner))
    }
}

impl Default for RecordLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

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
        let record =
            record_layer.frame_plaintext(ContentType::Handshake, original_payload).unwrap();

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

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut record_layer = RecordLayer::new();
        record_layer.enable_encryption();

        let plaintext = b"Secret message";

        // Mock encryption: just append sequence number and reverse bytes
        let encrypt_fn = |data: &[u8], seq: u64| {
            let mut encrypted = data.to_vec();
            encrypted.reverse();
            encrypted.extend_from_slice(&seq.to_be_bytes());
            Ok(encrypted)
        };

        // Mock decryption: remove sequence number and reverse bytes
        let decrypt_fn = |data: &[u8], _seq: u64| {
            let mut decrypted = data[..data.len() - 8].to_vec();
            decrypted.reverse();
            Ok(decrypted)
        };

        // Encrypt
        let encrypted_record = record_layer
            .encrypt_record(ContentType::ApplicationData, plaintext, encrypt_fn)
            .unwrap();

        // Parse the encrypted record
        let (content_type, ciphertext, _) = record_layer.parse_record(&encrypted_record).unwrap();
        assert_eq!(content_type, ContentType::ApplicationData);

        // Decrypt
        let (decrypted_type, decrypted_plaintext) =
            record_layer.decrypt_record(&ciphertext, decrypt_fn).unwrap();

        assert_eq!(decrypted_type, ContentType::ApplicationData);
        assert_eq!(decrypted_plaintext, plaintext);
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
            let record =
                record_layer.frame_plaintext(ContentType::ApplicationData, &payload).unwrap();

            // Verify record size
            assert_eq!(record.len(), 5 + size);

            // Verify length field in header
            let length = u16::from_be_bytes([record[3], record[4]]);
            assert_eq!(length as usize, size);
        }
    }

    #[test]
    fn test_encryption_with_sequence_increment() {
        let mut record_layer = RecordLayer::new();
        record_layer.enable_encryption();

        let initial_seq = record_layer.write_sequence();

        // Mock encryption function
        let encrypt_fn = |data: &[u8], seq: u64| {
            let mut result = data.to_vec();
            result.extend_from_slice(&seq.to_be_bytes());
            Ok(result)
        };

        // Encrypt a record
        let _encrypted =
            record_layer.encrypt_record(ContentType::ApplicationData, b"test", encrypt_fn).unwrap();

        // Sequence should have incremented
        assert_eq!(record_layer.write_sequence(), initial_seq + 1);
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
}

/// Hand-crafted edge cases for [`RecordLayer::parse_record`] (fuzz-style, no external harness).
#[cfg(test)]
mod fuzz_style_record_parsing_tests {
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
    #[expect(
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
}
