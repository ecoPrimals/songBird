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
    fn encode(&self, buf: &mut Vec<u8>) -> Result<()>;

    /// Get the encoded size in bytes
    fn encoded_size(&self) -> usize;
}

/// Trait for decoding types from wire format
pub trait Decode: Sized {
    /// Decode this type from bytes
    fn decode(buf: &[u8]) -> Result<(Self, usize)>;
}

/// Helper functions for encoding/decoding primitives
pub mod bytes {
    use super::*;

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
    pub fn write_vec8(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 255 {
            return Err(TlsError::InvalidParameter(format!(
                "Vec8 too long: {} bytes (max 255)",
                data.len()
            )));
        }
        write_u8(buf, data.len() as u8);
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Write a length-prefixed vector (u16 length)
    pub fn write_vec16(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 65535 {
            return Err(TlsError::InvalidParameter(format!(
                "Vec16 too long: {} bytes (max 65535)",
                data.len()
            )));
        }
        write_u16(buf, data.len() as u16);
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Write a length-prefixed vector (u24 length)
    pub fn write_vec24(buf: &mut Vec<u8>, data: &[u8]) -> Result<()> {
        if data.len() > 0xFFFFFF {
            return Err(TlsError::InvalidParameter(format!(
                "Vec24 too long: {} bytes (max 16777215)",
                data.len()
            )));
        }
        write_u24(buf, data.len() as u32);
        buf.extend_from_slice(data);
        Ok(())
    }

    /// Read a u8 from buffer
    pub fn read_u8(buf: &[u8], offset: &mut usize) -> Result<u8> {
        if *offset >= buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u8".to_string()));
        }
        let value = buf[*offset];
        *offset += 1;
        Ok(value)
    }

    /// Read a u16 (big-endian) from buffer
    pub fn read_u16(buf: &[u8], offset: &mut usize) -> Result<u16> {
        if *offset + 2 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u16".to_string()));
        }
        let bytes = [buf[*offset], buf[*offset + 1]];
        *offset += 2;
        Ok(u16::from_be_bytes(bytes))
    }

    /// Read a u24 (big-endian) from buffer
    pub fn read_u24(buf: &[u8], offset: &mut usize) -> Result<u32> {
        if *offset + 3 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u24".to_string()));
        }
        let bytes = [0, buf[*offset], buf[*offset + 1], buf[*offset + 2]];
        *offset += 3;
        Ok(u32::from_be_bytes(bytes))
    }

    /// Read a u32 (big-endian) from buffer
    pub fn read_u32(buf: &[u8], offset: &mut usize) -> Result<u32> {
        if *offset + 4 > buf.len() {
            return Err(TlsError::ProtocolError("Buffer underflow reading u32".to_string()));
        }
        let bytes = [buf[*offset], buf[*offset + 1], buf[*offset + 2], buf[*offset + 3]];
        *offset += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    /// Read a length-prefixed vector (u8 length)
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
        write_u24(&mut buf, 0x123456);
        assert_eq!(buf, vec![0x12, 0x34, 0x56]); // Big-endian, 3 bytes

        let mut offset = 0;
        let value = read_u24(&buf, &mut offset).unwrap();
        assert_eq!(value, 0x123456);
        assert_eq!(offset, 3);
    }

    #[test]
    fn test_write_read_u32() {
        let mut buf = Vec::new();
        write_u32(&mut buf, 0x12345678);
        assert_eq!(buf, vec![0x12, 0x34, 0x56, 0x78]); // Big-endian

        let mut offset = 0;
        let value = read_u32(&buf, &mut offset).unwrap();
        assert_eq!(value, 0x12345678);
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
}
