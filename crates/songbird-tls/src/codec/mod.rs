// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Wire format codec for TLS messages
//!
//! This module handles serialization and deserialization of TLS messages
//! to/from the wire format (byte streams) per RFC 8446.
//!
//! TLS uses big-endian byte order for all multi-byte integers.

pub mod messages;

use crate::error::{Result, TlsError};

/// Trait for encoding types to wire format
pub trait Encode {
    /// Encode this type to bytes
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails (e.g., invalid data, buffer overflow).
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()>;

    /// Get the encoded size in bytes
    fn encoded_size(&self) -> usize;
}

/// Trait for decoding types from wire format
pub trait Decode: Sized {
    /// Decode this type from bytes
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails (e.g., buffer underflow, invalid format).
    fn decode(buf: &[u8]) -> Result<(Self, usize)>;
}

/// Helper functions for encoding/decoding primitives
pub mod bytes {
    use super::{Result, TlsError};

    /// Write a u8 to buffer
    pub fn write_u8(buf: &mut Vec<u8>, value: u8) {
        buf.push(value);
    }

    /// Write a u16 (big-endian) to buffer
    pub fn write_u16(buf: &mut Vec<u8>, value: u16) {
        buf.extend_from_slice(&value.to_be_bytes());
    }

    /// Write a u24 (big-endian) to buffer
    pub fn write_u24(buf: &mut Vec<u8>, value: u32) {
        let bytes = value.to_be_bytes();
        buf.extend_from_slice(&bytes[1..4]); // Skip first byte for 24-bit
    }

    /// Write a u32 (big-endian) to buffer
    pub fn write_u32(buf: &mut Vec<u8>, value: u32) {
        buf.extend_from_slice(&value.to_be_bytes());
    }

    /// Write a length-prefixed vector (u8 length)
    ///
    /// # Errors
    ///
    /// Returns an error if data length exceeds 255 bytes.
    pub fn write_vec8(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 255 {
            return Err(TlsError::InvalidParameter(format!(
                "Vec8 too long: {} bytes (max 255)",
                data.len()
            )));
        }
        write_u8(
            buf,
            u8::try_from(data.len())
                .map_err(|_| TlsError::InvalidParameter("Vec8 length overflow".to_string()))?,
        );
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Write a length-prefixed vector (u16 length)
    ///
    /// # Errors
    ///
    /// Returns an error if data length exceeds 65535 bytes.
    pub fn write_vec16(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 65535 {
            return Err(TlsError::InvalidParameter(format!(
                "Vec16 too long: {} bytes (max 65535)",
                data.len()
            )));
        }
        write_u16(
            buf,
            u16::try_from(data.len())
                .map_err(|_| TlsError::InvalidParameter("Vec16 length overflow".to_string()))?,
        );
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Write a length-prefixed vector (u24 length)
    ///
    /// # Errors
    ///
    /// Returns an error if data length exceeds 16777215 bytes.
    pub fn write_vec24(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 0x00FF_FFFF {
            return Err(TlsError::InvalidParameter(format!(
                "Vec24 too long: {} bytes (max 16777215)",
                data.len()
            )));
        }
        write_u24(
            buf,
            u32::try_from(data.len())
                .map_err(|_| TlsError::InvalidParameter("Vec24 length overflow".to_string()))?,
        );
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Read a u8 from buffer
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn read_u8(buf: &[u8], offset: &mut usize) -> Result<u8> {
        if *offset >= buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u8".to_string()));
        }
        let value = buf[*offset];
        *offset += 1;
        Ok(value)
    }

    /// Read a u16 (big-endian) from buffer
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn read_u16(buf: &[u8], offset: &mut usize) -> Result<u16> {
        if *offset + 2 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u16".to_string()));
        }
        let bytes = [buf[*offset], buf[*offset + 1]];
        *offset += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    /// Read a u24 (big-endian) from buffer
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn read_u24(buf: &[u8], offset: &mut usize) -> Result<u32> {
        if *offset + 3 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u24".to_string()));
        }
        let bytes = [0, buf[*offset], buf[*offset + 1], buf[*offset + 2]];
        *offset += 3;
        Ok(u32::from_be_bytes(bytes))
    }

    /// Read a u32 (big-endian) from buffer
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn read_u32(buf: &[u8], offset: &mut usize) -> Result<u32> {
        if *offset + 4 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u32".to_string()));
        }
        let bytes = [buf[*offset], buf[*offset + 1], buf[*offset + 2], buf[*offset + 3]];
        *offset += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    /// Read a length-prefixed vector (u8 length)
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted or truncated.
    pub fn read_vec8(buf: &[u8], offset: &mut usize) -> Result<Vec<u8>> {
        let len = read_u8(buf, offset)? as usize;
        if *offset + len > buf.len() {
            return Err(TlsError::ProtocolError(format!(
                "Buffer underflow reading vec8: need {} bytes, have {}",
                len,
                buf.len() - *offset
            )));
        }
        let data = buf[*offset..*offset + len].to_vec();
        *offset += len;
        Ok(data)
    }

    /// Read a length-prefixed vector (u16 length)
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted or truncated.
    pub fn read_vec16(buf: &[u8], offset: &mut usize) -> Result<Vec<u8>> {
        let len = read_u16(buf, offset)? as usize;
        if *offset + len > buf.len() {
            return Err(TlsError::ProtocolError(format!(
                "Buffer underflow reading vec16: need {} bytes, have {}",
                len,
                buf.len() - *offset
            )));
        }
        let data = buf[*offset..*offset + len].to_vec();
        *offset += len;
        Ok(data)
    }

    /// Read a length-prefixed vector (u24 length)
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted or truncated.
    pub fn read_vec24(buf: &[u8], offset: &mut usize) -> Result<Vec<u8>> {
        let len = read_u24(buf, offset)? as usize;
        if *offset + len > buf.len() {
            return Err(TlsError::ProtocolError(format!(
                "Buffer underflow reading vec24: need {} bytes, have {}",
                len,
                buf.len() - *offset
            )));
        }
        let data = buf[*offset..*offset + len].to_vec();
        *offset += len;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::bytes::*;

    #[test]
    fn test_write_read_u8() {
        let mut buf = Vec::new();
        write_u8(&mut buf, 42);
        assert_eq!(buf, vec![42]);

        let mut offset = 0;
        let value = read_u8(&buf, &mut offset).unwrap();
        assert_eq!(value, 42);
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_write_read_u16() {
        let mut buf = Vec::new();
        write_u16(&mut buf, 0x1234);
        assert_eq!(buf, vec![0x12, 0x34]); // Big-endian

        let mut offset = 0;
        let value = read_u16(&buf, &mut offset).unwrap();
        assert_eq!(value, 0x1234);
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_write_read_u24() {
        let mut buf = Vec::new();
        write_u24(&mut buf, 0x12_3456);
        assert_eq!(buf, vec![0x12, 0x34, 0x56]); // Big-endian, 3 bytes

        let mut offset = 0;
        let value = read_u24(&buf, &mut offset).unwrap();
        assert_eq!(value, 0x12_3456);
        assert_eq!(offset, 3);
    }

    #[test]
    fn test_write_read_u32() {
        let mut buf = Vec::new();
        write_u32(&mut buf, 0x1234_5678);
        assert_eq!(buf, vec![0x12, 0x34, 0x56, 0x78]); // Big-endian

        let mut offset = 0;
        let value = read_u32(&buf, &mut offset).unwrap();
        assert_eq!(value, 0x1234_5678);
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_write_read_vec8() {
        let mut buf = Vec::new();
        let data = vec![1, 2, 3, 4, 5];
        write_vec8(&mut buf, &data).unwrap();
        assert_eq!(buf, vec![5, 1, 2, 3, 4, 5]); // Length prefix + data

        let mut offset = 0;
        let read_data = read_vec8(&buf, &mut offset).unwrap();
        assert_eq!(read_data, data);
        assert_eq!(offset, 6);
    }

    #[test]
    fn test_write_read_vec16() {
        let mut buf = Vec::new();
        let data = vec![1, 2, 3, 4, 5];
        write_vec16(&mut buf, &data).unwrap();
        assert_eq!(buf, vec![0, 5, 1, 2, 3, 4, 5]); // Length prefix (u16) + data

        let mut offset = 0;
        let read_data = read_vec16(&buf, &mut offset).unwrap();
        assert_eq!(read_data, data);
        assert_eq!(offset, 7);
    }

    #[test]
    fn test_write_read_vec24() {
        let mut buf = Vec::new();
        let data = vec![1, 2, 3, 4, 5];
        write_vec24(&mut buf, &data).unwrap();
        assert_eq!(buf, vec![0, 0, 5, 1, 2, 3, 4, 5]); // Length prefix (u24) + data

        let mut offset = 0;
        let read_data = read_vec24(&buf, &mut offset).unwrap();
        assert_eq!(read_data, data);
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_vec8_too_long() {
        let mut buf = Vec::new();
        let data = vec![0u8; 256]; // Too long for u8 length
        let result = write_vec8(&mut buf, &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u8_underflow() {
        let buf = vec![];
        let mut offset = 0;
        let result = read_u8(&buf, &mut offset);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_u16_underflow() {
        let buf = vec![0x12]; // Only 1 byte
        let mut offset = 0;
        let result = read_u16(&buf, &mut offset);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_vec8_underflow() {
        let buf = vec![5, 1, 2]; // Says 5 bytes, but only 2 available
        let mut offset = 0;
        let result = read_vec8(&buf, &mut offset);
        assert!(result.is_err());
    }

    #[test]
    fn write_vec16_rejects_oversized_payload() {
        let mut buf = Vec::new();
        let data = vec![0u8; 65536];
        assert!(write_vec16(&mut buf, &data).is_err());
    }

    #[test]
    fn write_vec24_rejects_oversized_payload() {
        let mut buf = Vec::new();
        let data = vec![0u8; 0x01_00_00_00];
        assert!(write_vec24(&mut buf, &data).is_err());
    }

    #[test]
    fn read_vec16_truncated_payload_errors() {
        let mut buf = vec![0x00, 0x05];
        buf.extend_from_slice(&[1, 2, 3]); // need 5, have 3
        let mut offset = 0;
        assert!(read_vec16(&buf, &mut offset).is_err());
    }

    #[test]
    fn read_vec24_truncated_payload_errors() {
        let mut buf = vec![0x00, 0x00, 0x04];
        buf.extend_from_slice(&[1, 2]); // need 4, have 2
        let mut offset = 0;
        assert!(read_vec24(&buf, &mut offset).is_err());
    }

    #[test]
    fn write_u24_max_three_byte_value() {
        let mut buf = Vec::new();
        write_u24(&mut buf, 0xFF_FFFF);
        assert_eq!(buf, vec![0xff, 0xff, 0xff]);
        let mut off = 0;
        assert_eq!(read_u24(&buf, &mut off).unwrap(), 0xFF_FFFF);
    }

    #[test]
    fn read_u32_at_buffer_end() {
        let buf = [0x12, 0x34, 0x56, 0x78];
        let mut off = 0;
        assert_eq!(read_u32(&buf, &mut off).unwrap(), 0x1234_5678);
        assert_eq!(off, 4);
    }
}
