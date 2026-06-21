// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::MAGIC_COOKIE;
use super::types::AttributeType;
use crate::error::{StunError, StunResult};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// RFC 5389 FINGERPRINT XOR constant.
const FINGERPRINT_XOR: u32 = 0x5354_554E;

/// STUN attribute
#[derive(Debug, Clone)]
pub enum StunAttribute {
    /// MAPPED-ADDRESS
    MappedAddress(SocketAddr),

    /// XOR-MAPPED-ADDRESS (preferred)
    XorMappedAddress(SocketAddr),

    /// USERNAME (RFC 5389 short-term or long-term credentials)
    Username(String),

    /// MESSAGE-INTEGRITY (RFC 5389) — HMAC-SHA1 over the message
    MessageIntegrity([u8; 20]),

    /// FINGERPRINT (RFC 5389) — CRC32 XOR 0x5354554E over the message
    Fingerprint(u32),

    /// OTHER-ADDRESS (for NAT type detection)
    OtherAddress(SocketAddr),

    /// Unknown attribute (type, data)
    Unknown(u16, Bytes),
}

impl StunAttribute {
    #[allow(
        dead_code,
        reason = "convenience fallback for tests/server without transaction context"
    )]
    pub(crate) fn encode(&self, buf: &mut BytesMut) {
        self.encode_with_tid(buf, &[0u8; 12]);
    }

    pub(crate) fn encode_with_tid(&self, buf: &mut BytesMut, transaction_id: &[u8; 12]) {
        match self {
            Self::MappedAddress(addr) => {
                Self::encode_address(buf, AttributeType::MappedAddress, addr, None, transaction_id);
            }
            Self::XorMappedAddress(addr) => {
                Self::encode_address(
                    buf,
                    AttributeType::XorMappedAddress,
                    addr,
                    Some(MAGIC_COOKIE),
                    transaction_id,
                );
            }
            Self::Username(name) => {
                let name_bytes = name.as_bytes();
                buf.put_u16(AttributeType::Username.to_u16());
                buf.put_u16(u16::try_from(name_bytes.len()).unwrap_or(u16::MAX));
                buf.put_slice(name_bytes);
                Self::add_padding(buf);
            }
            Self::MessageIntegrity(hmac) => {
                buf.put_u16(AttributeType::MessageIntegrity.to_u16());
                buf.put_u16(20);
                buf.put_slice(hmac);
            }
            Self::Fingerprint(crc) => {
                buf.put_u16(AttributeType::Fingerprint.to_u16());
                buf.put_u16(4);
                buf.put_u32(*crc);
            }
            Self::OtherAddress(addr) => {
                Self::encode_address(buf, AttributeType::OtherAddress, addr, None, transaction_id);
            }
            Self::Unknown(attr_type, data) => {
                buf.put_u16(*attr_type);
                buf.put_u16(u16::try_from(data.len()).unwrap_or(u16::MAX));
                buf.put_slice(data);
                Self::add_padding(buf);
            }
        }
    }

    pub(crate) fn encode_address(
        buf: &mut BytesMut,
        attr_type: AttributeType,
        addr: &SocketAddr,
        xor_key: Option<u32>,
        transaction_id: &[u8; 12],
    ) {
        buf.put_u16(attr_type.to_u16());

        let data_start = buf.len();
        buf.put_u16(0); // Length placeholder

        // Family (1 byte reserved, 1 byte family)
        buf.put_u8(0);
        match addr {
            SocketAddr::V4(_) => buf.put_u8(0x01), // IPv4
            SocketAddr::V6(_) => buf.put_u8(0x02), // IPv6
        }

        // Port (2 bytes) — XOR with high 16 bits of magic cookie
        let port = xor_key.map_or_else(|| addr.port(), |xor| addr.port() ^ (xor >> 16) as u16);
        buf.put_u16(port);

        // Address
        match addr.ip() {
            IpAddr::V4(ip) => {
                let octets = xor_key.map_or_else(
                    || ip.octets(),
                    |xor| {
                        let ip_u32 = u32::from(ip);
                        let xored = ip_u32 ^ xor;
                        xored.to_be_bytes()
                    },
                );
                buf.put_slice(&octets);
            }
            IpAddr::V6(ip) => {
                if xor_key.is_some() {
                    // RFC 5389 §15.2: XOR with magic_cookie (4 bytes) || transaction_id (12 bytes)
                    let mut xor_pad = [0u8; 16];
                    xor_pad[..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
                    xor_pad[4..].copy_from_slice(transaction_id);
                    let raw = ip.octets();
                    let mut xored = [0u8; 16];
                    for i in 0..16 {
                        xored[i] = raw[i] ^ xor_pad[i];
                    }
                    buf.put_slice(&xored);
                } else {
                    buf.put_slice(&ip.octets());
                }
            }
        }

        // Update length
        let data_len = buf.len() - data_start - 2;
        buf[data_start..data_start + 2]
            .copy_from_slice(&u16::try_from(data_len).unwrap_or(u16::MAX).to_be_bytes());

        Self::add_padding(buf);
    }

    #[allow(dead_code, reason = "convenience fallback for tests without transaction context")]
    pub(crate) fn decode(buf: &mut &[u8]) -> StunResult<Self> {
        Self::decode_with_tid(buf, &[0u8; 12])
    }

    pub(crate) fn decode_with_tid(buf: &mut &[u8], transaction_id: &[u8; 12]) -> StunResult<Self> {
        if buf.remaining() < 4 {
            return Err(StunError::InvalidResponse(String::from("Attribute too short")));
        }

        let attr_type = buf.get_u16();
        let attr_length = buf.get_u16() as usize;

        if buf.remaining() < attr_length {
            return Err(StunError::InvalidResponse(format!(
                "Attribute data too short: expected {}, got {}",
                attr_length,
                buf.remaining()
            )));
        }

        let attr_data = &buf[..attr_length];
        buf.advance(attr_length);

        // Skip padding (attributes padded to 4-byte boundary)
        let padding = (4 - (attr_length % 4)) % 4;
        buf.advance(padding.min(buf.remaining()));

        match AttributeType::from_u16(attr_type) {
            AttributeType::MappedAddress => {
                let addr = Self::decode_address(attr_data, None, transaction_id)?;
                Ok(Self::MappedAddress(addr))
            }
            AttributeType::Username => {
                let name = std::str::from_utf8(attr_data)
                    .map_err(|e| {
                        StunError::InvalidResponse(format!("Invalid USERNAME UTF-8: {e}"))
                    })?
                    .to_string();
                Ok(Self::Username(name))
            }
            AttributeType::XorMappedAddress => {
                let addr = Self::decode_address(attr_data, Some(MAGIC_COOKIE), transaction_id)?;
                Ok(Self::XorMappedAddress(addr))
            }
            AttributeType::MessageIntegrity => {
                if attr_data.len() != 20 {
                    return Err(StunError::InvalidResponse(format!(
                        "MESSAGE-INTEGRITY must be 20 bytes, got {}",
                        attr_data.len()
                    )));
                }
                let mut hmac_val = [0u8; 20];
                hmac_val.copy_from_slice(attr_data);
                Ok(Self::MessageIntegrity(hmac_val))
            }
            AttributeType::Fingerprint => {
                if attr_data.len() != 4 {
                    return Err(StunError::InvalidResponse(format!(
                        "FINGERPRINT must be 4 bytes, got {}",
                        attr_data.len()
                    )));
                }
                let crc =
                    u32::from_be_bytes([attr_data[0], attr_data[1], attr_data[2], attr_data[3]]);
                Ok(Self::Fingerprint(crc))
            }
            AttributeType::OtherAddress => {
                let addr = Self::decode_address(attr_data, None, transaction_id)?;
                Ok(Self::OtherAddress(addr))
            }
            AttributeType::Realm | AttributeType::Nonce | AttributeType::Unknown(_) => {
                Ok(Self::Unknown(attr_type, Bytes::copy_from_slice(attr_data)))
            }
        }
    }

    /// Decode an address attribute value (MAPPED-ADDRESS or XOR-MAPPED-ADDRESS).
    ///
    /// When `xor_key` is `Some(MAGIC_COOKIE)`, the address and port are XOR-decoded
    /// per RFC 5389 §15.2.
    pub fn decode_address(
        data: &[u8],
        xor_key: Option<u32>,
        transaction_id: &[u8; 12],
    ) -> StunResult<SocketAddr> {
        if data.len() < 4 {
            return Err(StunError::InvalidResponse(String::from("Address attribute too short")));
        }

        let mut buf = data;

        // Reserved (1 byte)
        buf.advance(1);

        // Family (1 byte)
        let family = buf.get_u8();

        // Port (2 bytes)
        let port_raw = buf.get_u16();
        let port = xor_key.map_or_else(|| port_raw, |xor| port_raw ^ (xor >> 16) as u16);

        // Address
        match family {
            0x01 => {
                // IPv4
                if buf.remaining() < 4 {
                    return Err(StunError::InvalidResponse(String::from("IPv4 address too short")));
                }

                let ip_raw = buf.get_u32();
                let ip = xor_key
                    .map_or_else(|| Ipv4Addr::from(ip_raw), |xor| Ipv4Addr::from(ip_raw ^ xor));

                Ok(SocketAddr::new(IpAddr::V4(ip), port))
            }
            0x02 => {
                // IPv6
                if buf.remaining() < 16 {
                    return Err(StunError::InvalidResponse(String::from("IPv6 address too short")));
                }

                let mut octets = [0u8; 16];
                buf.copy_to_slice(&mut octets);

                if xor_key.is_some() {
                    // RFC 5389 §15.2: XOR with magic_cookie (4 bytes) || transaction_id (12 bytes)
                    let mut xor_pad = [0u8; 16];
                    xor_pad[..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
                    xor_pad[4..].copy_from_slice(transaction_id);
                    for i in 0..16 {
                        octets[i] ^= xor_pad[i];
                    }
                }

                let ip = Ipv6Addr::from(octets);
                Ok(SocketAddr::new(IpAddr::V6(ip), port))
            }
            _ => Err(StunError::InvalidResponse(format!("Unknown address family: {family}"))),
        }
    }

    /// Verify a FINGERPRINT attribute value against a message buffer.
    /// `message_up_to_fingerprint` is all bytes before the FINGERPRINT attribute TLV.
    #[must_use]
    pub fn verify_fingerprint(message_up_to_fingerprint: &[u8], received_crc: u32) -> bool {
        let computed = crc32fast::hash(message_up_to_fingerprint) ^ FINGERPRINT_XOR;
        computed == received_crc
    }

    /// Compute FINGERPRINT value for a message buffer (all bytes before the FINGERPRINT TLV).
    #[must_use]
    pub fn compute_fingerprint(message_up_to_fingerprint: &[u8]) -> u32 {
        crc32fast::hash(message_up_to_fingerprint) ^ FINGERPRINT_XOR
    }

    /// Compute MESSAGE-INTEGRITY HMAC-SHA1 for a message buffer.
    /// `message_up_to_integrity` is all bytes before the MESSAGE-INTEGRITY attribute TLV,
    /// with the message length adjusted to include the MESSAGE-INTEGRITY attribute (24 bytes).
    #[must_use]
    pub fn compute_message_integrity(message_up_to_integrity: &[u8], key: &[u8]) -> [u8; 20] {
        use hmac::{Hmac, Mac};
        type HmacSha1 = Hmac<sha1::Sha1>;

        // HMAC-SHA1 new_from_slice is infallible for any key length (SHA1 block size handles all).
        let Ok(mut mac) = HmacSha1::new_from_slice(key) else {
            unreachable!()
        };
        mac.update(message_up_to_integrity);
        let result = mac.finalize();
        let mut out = [0u8; 20];
        out.copy_from_slice(&result.into_bytes());
        out
    }

    /// Verify a MESSAGE-INTEGRITY attribute value.
    #[must_use]
    pub fn verify_message_integrity(
        message_up_to_integrity: &[u8],
        key: &[u8],
        received_hmac: &[u8; 20],
    ) -> bool {
        let computed = Self::compute_message_integrity(message_up_to_integrity, key);
        computed == *received_hmac
    }

    pub(crate) fn add_padding(buf: &mut BytesMut) {
        let padding = (4 - (buf.len() % 4)) % 4;
        for _ in 0..padding {
            buf.put_u8(0);
        }
    }
}
