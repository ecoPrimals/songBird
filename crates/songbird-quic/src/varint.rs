// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 9000 Section 16 variable-length integer encoding.
//!
//! QUIC variable-length integers use a two-bit length prefix:
//! - `0b00`: 6-bit value  (1 byte,  max 63)
//! - `0b01`: 14-bit value (2 bytes, max 16383)
//! - `0b10`: 30-bit value (4 bytes, max 1073741823)
//! - `0b11`: 62-bit value (8 bytes, max 4611686018427387903)

use crate::error::{QuicError, Result};

/// Maximum value representable in a QUIC variable-length integer (2^62 - 1).
pub const VARINT_MAX: u64 = (1 << 62) - 1;

/// A QUIC variable-length integer value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(u64);

impl VarInt {
    /// Create a new `VarInt` from a `u64`, returning an error if out of range.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Config`] if `value` exceeds [`VARINT_MAX`].
    pub fn new(value: u64) -> Result<Self> {
        if value > VARINT_MAX {
            return Err(QuicError::Config(format!(
                "VarInt value {value} exceeds maximum {VARINT_MAX}"
            )));
        }
        Ok(Self(value))
    }

    /// Create a `VarInt` from a `u32` (always in range).
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        Self(value as u64)
    }

    /// Return the underlying `u64` value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Number of bytes this value occupies when encoded.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        if self.0 < 64 {
            1
        } else if self.0 < 16384 {
            2
        } else if self.0 < 1_073_741_824 {
            4
        } else {
            8
        }
    }

    /// Encode into the provided buffer, returning the number of bytes written.
    ///
    /// The buffer must be at least `self.encoded_len()` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the buffer is too short.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(QuicError::Stream(format!(
                "VarInt encode buffer too small: need {len}, have {}",
                buf.len()
            )));
        }
        match len {
            1 => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "1-byte VarInt encodes values < 64"
                )]
                let b = self.0 as u8;
                buf[0] = b;
            }
            2 => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "2-byte VarInt encodes values < 16384"
                )]
                let val = (self.0 as u16) | 0x4000;
                buf[..2].copy_from_slice(&val.to_be_bytes());
            }
            4 => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "4-byte VarInt encodes values < 2^30"
                )]
                let val = (self.0 as u32) | 0x8000_0000;
                buf[..4].copy_from_slice(&val.to_be_bytes());
            }
            8 => {
                let val = self.0 | 0xC000_0000_0000_0000;
                buf[..8].copy_from_slice(&val.to_be_bytes());
            }
            _ => unreachable!(),
        }
        Ok(len)
    }

    /// Decode a `VarInt` from the beginning of the buffer.
    ///
    /// Returns the decoded value and the number of bytes consumed.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the buffer is empty or truncated.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        if buf.is_empty() {
            return Err(QuicError::Stream("VarInt decode: empty buffer".into()));
        }
        let prefix = buf[0] >> 6;
        let len = 1usize << prefix;
        if buf.len() < len {
            return Err(QuicError::Stream(format!(
                "VarInt decode: need {len} bytes, have {}",
                buf.len()
            )));
        }
        let value = match len {
            1 => u64::from(buf[0] & 0x3F),
            2 => {
                let mut tmp = [0u8; 2];
                tmp.copy_from_slice(&buf[..2]);
                u64::from(u16::from_be_bytes(tmp) & 0x3FFF)
            }
            4 => {
                let mut tmp = [0u8; 4];
                tmp.copy_from_slice(&buf[..4]);
                u64::from(u32::from_be_bytes(tmp) & 0x3FFF_FFFF)
            }
            8 => {
                let mut tmp = [0u8; 8];
                tmp.copy_from_slice(&buf[..8]);
                u64::from_be_bytes(tmp) & 0x3FFF_FFFF_FFFF_FFFF
            }
            _ => unreachable!(),
        };
        Ok((Self(value), len))
    }
}

impl From<VarInt> for u64 {
    fn from(v: VarInt) -> Self {
        v.0
    }
}

impl TryFrom<u64> for VarInt {
    type Error = QuicError;

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Display for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn roundtrip_1_byte() {
        for v in [0u64, 1, 37, 63] {
            let vi = VarInt::new(v).unwrap();
            assert_eq!(vi.encoded_len(), 1);
            let mut buf = [0u8; 8];
            let n = vi.encode(&mut buf).unwrap();
            assert_eq!(n, 1);
            let (decoded, consumed) = VarInt::decode(&buf[..n]).unwrap();
            assert_eq!(decoded.value(), v);
            assert_eq!(consumed, 1);
        }
    }

    #[test]
    fn roundtrip_2_byte() {
        for v in [64u64, 255, 16383] {
            let vi = VarInt::new(v).unwrap();
            assert_eq!(vi.encoded_len(), 2);
            let mut buf = [0u8; 8];
            let n = vi.encode(&mut buf).unwrap();
            assert_eq!(n, 2);
            let (decoded, consumed) = VarInt::decode(&buf[..n]).unwrap();
            assert_eq!(decoded.value(), v);
            assert_eq!(consumed, 2);
        }
    }

    #[test]
    fn roundtrip_4_byte() {
        for v in [16384u64, 65535, 1_073_741_823] {
            let vi = VarInt::new(v).unwrap();
            assert_eq!(vi.encoded_len(), 4);
            let mut buf = [0u8; 8];
            let n = vi.encode(&mut buf).unwrap();
            assert_eq!(n, 4);
            let (decoded, consumed) = VarInt::decode(&buf[..n]).unwrap();
            assert_eq!(decoded.value(), v);
            assert_eq!(consumed, 4);
        }
    }

    #[test]
    fn roundtrip_8_byte() {
        for v in [1_073_741_824u64, u64::from(u32::MAX), VARINT_MAX] {
            let vi = VarInt::new(v).unwrap();
            assert_eq!(vi.encoded_len(), 8);
            let mut buf = [0u8; 8];
            let n = vi.encode(&mut buf).unwrap();
            assert_eq!(n, 8);
            let (decoded, consumed) = VarInt::decode(&buf[..n]).unwrap();
            assert_eq!(decoded.value(), v);
            assert_eq!(consumed, 8);
        }
    }

    #[test]
    fn rfc_test_vectors() {
        // RFC 9000 Appendix A examples
        let cases: &[(u64, &[u8])] = &[
            (151_288_809_941_952_652, &[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c]),
            (494_878_333, &[0x9d, 0x7f, 0x3e, 0x7d]),
            (15_293, &[0x7b, 0xbd]),
            (37, &[0x25]),
        ];
        for &(val, expected) in cases {
            let vi = VarInt::new(val).unwrap();
            let mut buf = [0u8; 8];
            let n = vi.encode(&mut buf).unwrap();
            assert_eq!(&buf[..n], expected, "encode mismatch for {val}");
            let (decoded, consumed) = VarInt::decode(expected).unwrap();
            assert_eq!(decoded.value(), val, "decode mismatch for {val}");
            assert_eq!(consumed, expected.len());
        }
    }

    #[test]
    fn exceeds_max_returns_error() {
        assert!(VarInt::new(VARINT_MAX + 1).is_err());
    }

    #[test]
    fn from_u32_always_valid() {
        let vi = VarInt::from_u32(u32::MAX);
        assert_eq!(vi.value(), u64::from(u32::MAX));
    }

    #[test]
    fn empty_buffer_decode_error() {
        assert!(VarInt::decode(&[]).is_err());
    }

    #[test]
    fn truncated_buffer_decode_error() {
        // 2-byte prefix but only 1 byte available
        assert!(VarInt::decode(&[0x40]).is_err());
    }

    #[test]
    fn encode_buffer_too_small() {
        let vi = VarInt::new(16384).unwrap(); // 4-byte encoding
        let mut buf = [0u8; 2];
        assert!(vi.encode(&mut buf).is_err());
    }

    #[test]
    fn display_impl() {
        let vi = VarInt::new(42).unwrap();
        assert_eq!(format!("{vi}"), "42");
    }

    #[test]
    fn try_from_u64() {
        let vi: VarInt = 100u64.try_into().unwrap();
        assert_eq!(vi.value(), 100);
        assert!(VarInt::try_from(VARINT_MAX + 1).is_err());
    }

    #[test]
    fn into_u64() {
        let vi = VarInt::new(999).unwrap();
        let val: u64 = vi.into();
        assert_eq!(val, 999);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn truncated_four_byte_varint_decode_errors() {
        // Prefix 0b10 => 4 bytes required; provide 3.
        let err = VarInt::decode(&[0x80, 0x80, 0x80]).expect_err("need 4 bytes");
        assert!(
            err.to_string().contains('4') || err.to_string().contains("bytes"),
            "unexpected: {err}"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn truncated_eight_byte_varint_decode_errors() {
        // Prefix 0b11 => 8 bytes required; provide 5.
        let err = VarInt::decode(&[0xC0, 1, 2, 3, 4]).expect_err("need 8 bytes");
        assert!(
            err.to_string().contains('8') || err.to_string().contains("bytes"),
            "unexpected: {err}"
        );
    }
}
