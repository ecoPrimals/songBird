// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::MAGIC_COOKIE;
use super::types::AttributeType;
use crate::error::{StunError, StunResult};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// STUN attribute
#[derive(Debug, Clone)]
pub enum StunAttribute {
    /// MAPPED-ADDRESS
    MappedAddress(SocketAddr),

    /// XOR-MAPPED-ADDRESS (preferred)
    XorMappedAddress(SocketAddr),

    /// USERNAME (RFC 5389 short-term or long-term credentials)
    Username(String),

    /// OTHER-ADDRESS (for NAT type detection)
    OtherAddress(SocketAddr),

    /// Unknown attribute (type, data)
    Unknown(u16, Bytes),
}

impl StunAttribute {
    pub(crate) fn encode(&self, buf: &mut BytesMut) {
        match self {
            Self::MappedAddress(addr) => {
                Self::encode_address(buf, AttributeType::MappedAddress, addr, None);
            }
            Self::XorMappedAddress(addr) => {
                Self::encode_address(
                    buf,
                    AttributeType::XorMappedAddress,
                    addr,
                    Some(MAGIC_COOKIE),
                );
            }
            Self::Username(name) => {
                let name_bytes = name.as_bytes();
                buf.put_u16(AttributeType::Username.to_u16());
                buf.put_u16(u16::try_from(name_bytes.len()).unwrap_or(u16::MAX));
                buf.put_slice(name_bytes);
                Self::add_padding(buf);
            }
            Self::OtherAddress(addr) => {
                Self::encode_address(buf, AttributeType::OtherAddress, addr, None);
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

        // Port (2 bytes)
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
                // XOR with magic cookie + transaction ID not implemented for IPv6
                let octets = ip.octets();
                buf.put_slice(&octets);
            }
        }

        // Update length
        let data_len = buf.len() - data_start - 2;
        buf[data_start..data_start + 2]
            .copy_from_slice(&u16::try_from(data_len).unwrap_or(u16::MAX).to_be_bytes());

        Self::add_padding(buf);
    }

    pub(crate) fn decode(buf: &mut &[u8]) -> StunResult<Self> {
        if buf.remaining() < 4 {
            return Err(StunError::InvalidResponse("Attribute too short".to_string()));
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
                let addr = Self::decode_address(attr_data, None)?;
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
                let addr = Self::decode_address(attr_data, Some(MAGIC_COOKIE))?;
                Ok(Self::XorMappedAddress(addr))
            }
            AttributeType::OtherAddress => {
                let addr = Self::decode_address(attr_data, None)?;
                Ok(Self::OtherAddress(addr))
            }
            AttributeType::Realm | AttributeType::Nonce | AttributeType::Unknown(_) => {
                Ok(Self::Unknown(attr_type, Bytes::copy_from_slice(attr_data)))
            }
        }
    }

    pub(crate) fn decode_address(data: &[u8], xor_key: Option<u32>) -> StunResult<SocketAddr> {
        if data.len() < 4 {
            return Err(StunError::InvalidResponse("Address attribute too short".to_string()));
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
                    return Err(StunError::InvalidResponse("IPv4 address too short".to_string()));
                }

                let ip_raw = buf.get_u32();
                let ip = xor_key
                    .map_or_else(|| Ipv4Addr::from(ip_raw), |xor| Ipv4Addr::from(ip_raw ^ xor));

                Ok(SocketAddr::new(IpAddr::V4(ip), port))
            }
            0x02 => {
                // IPv6
                if buf.remaining() < 16 {
                    return Err(StunError::InvalidResponse("IPv6 address too short".to_string()));
                }

                let mut octets = [0u8; 16];
                buf.copy_to_slice(&mut octets);

                // Note: Full XOR for IPv6 requires transaction ID (not implemented for simplicity)
                let ip = Ipv6Addr::from(octets);

                Ok(SocketAddr::new(IpAddr::V6(ip), port))
            }
            _ => Err(StunError::InvalidResponse(format!("Unknown address family: {family}"))),
        }
    }

    pub(crate) fn add_padding(buf: &mut BytesMut) {
        let padding = (4 - (buf.len() % 4)) % 4;
        for _ in 0..padding {
            buf.put_u8(0);
        }
    }
}
