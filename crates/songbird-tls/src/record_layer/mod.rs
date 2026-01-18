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

use crate::error::{Result, TlsError};
use crate::messages::ContentType;
use crate::codec::bytes::*;
use crate::{TLS_VERSION_1_2, MAX_RECORD_SIZE};

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
    /// Create a new RecordLayer in plaintext mode
    pub fn new() -> Self {
        Self {
            write_sequence: 0,
            read_sequence: 0,
            encrypted: false,
        }
    }

    /// Enable encryption (called after handshake completion)
    pub fn enable_encryption(&mut self) {
        self.encrypted = true;
        // Note: Sequence numbers are NOT reset when enabling encryption
        // They continue from handshake phase
    }

    /// Get the current write sequence number
    pub fn write_sequence(&self) -> u64 {
        self.write_sequence
    }

    /// Get the current read sequence number
    pub fn read_sequence(&self) -> u64 {
        self.read_sequence
    }

    /// Increment write sequence number
    fn increment_write_sequence(&mut self) {
        self.write_sequence = self.write_sequence.wrapping_add(1);
    }

    /// Increment read sequence number
    fn increment_read_sequence(&mut self) {
        self.read_sequence = self.read_sequence.wrapping_add(1);
    }

    /// Frame a plaintext message into a TLS record
    ///
    /// This creates the 5-byte header + payload.
    /// Does NOT encrypt (encryption is handled separately).
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
        write_u16(&mut record, payload.len() as u16);

        // Payload
        record.extend_from_slice(payload);

        Ok(record)
    }

    /// Parse a TLS record from bytes
    ///
    /// Returns: (content_type, payload, bytes_consumed)
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
            return Err(TlsError::RecordTooLarge { size: length });
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
    /// The record content type is always ApplicationData (23).
    ///
    /// Format of encrypted payload:
    /// ```text
    /// struct {
    ///     opaque content[length];
    ///     ContentType type;        // Actual content type
    ///     uint8 zeros[length_of_padding];
    /// } TLSInnerPlaintext;
    /// ```
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
        let content_type = ContentType::from(inner.pop().unwrap());
        
        Ok((content_type, inner))
    }
}

impl Default for RecordLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
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
        
        let record = record_layer
            .frame_plaintext(ContentType::Handshake, payload)
            .unwrap();
        
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
        let record = record_layer
            .frame_plaintext(ContentType::ApplicationData, payload)
            .unwrap();
        
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
        let record = record_layer
            .frame_plaintext(ContentType::Handshake, original_payload)
            .unwrap();
        
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
}
