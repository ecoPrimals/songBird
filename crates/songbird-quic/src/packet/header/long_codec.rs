// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{QuicError, Result};
use crate::varint::VarInt;

use super::LongHeader;
use super::LongPacketType;

/// Decode a long header from the buffer. Returns the header and the offset
/// where the packet payload begins (after the packet number).
///
/// # Errors
///
/// Returns [`QuicError::Stream`] on truncated or malformed wire data.
pub fn decode_long_header(buf: &[u8]) -> Result<(LongHeader, usize)> {
    if buf.len() < 7 {
        return Err(QuicError::Stream("Long header too short".into()));
    }

    let first = buf[0];
    if first & 0x80 == 0 {
        return Err(QuicError::Stream("Not a long header (form bit = 0)".into()));
    }

    let fixed_bit = first & 0x40 != 0;
    let type_bits = (first >> 4) & 0x03;
    let packet_type = LongPacketType::from_bits(type_bits)?;
    let reserved_bits = (first >> 2) & 0x03;
    let pn_length = (first & 0x03) + 1;

    let version = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);

    let mut offset = 5;

    // DCID length (1 byte) + DCID
    if offset >= buf.len() {
        return Err(QuicError::Stream("Truncated DCID length".into()));
    }
    let dcid_len = buf[offset] as usize;
    offset += 1;
    if offset + dcid_len > buf.len() {
        return Err(QuicError::Stream("Truncated DCID".into()));
    }
    let dcid = buf[offset..offset + dcid_len].to_vec();
    offset += dcid_len;

    // SCID length (1 byte) + SCID
    if offset >= buf.len() {
        return Err(QuicError::Stream("Truncated SCID length".into()));
    }
    let scid_len = buf[offset] as usize;
    offset += 1;
    if offset + scid_len > buf.len() {
        return Err(QuicError::Stream("Truncated SCID".into()));
    }
    let scid = buf[offset..offset + scid_len].to_vec();
    offset += scid_len;

    // Token (Initial packets only)
    let token = if packet_type == LongPacketType::Initial {
        let (token_len, consumed) = VarInt::decode(&buf[offset..])?;
        offset += consumed;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Initial token length from VarInt; bounds-checked against buffer"
        )]
        let tlen = token_len.value() as usize;
        if offset + tlen > buf.len() {
            return Err(QuicError::Stream("Truncated token".into()));
        }
        let t = buf[offset..offset + tlen].to_vec();
        offset += tlen;
        t
    } else {
        Vec::new()
    };

    // Retry packets don't have Length or PN fields
    if packet_type == LongPacketType::Retry {
        return Ok((
            LongHeader {
                fixed_bit,
                packet_type,
                reserved_bits,
                pn_length,
                version,
                dcid,
                scid,
                token,
                payload_length: 0,
                packet_number: 0,
            },
            offset,
        ));
    }

    // Payload length
    let (payload_length, consumed) = VarInt::decode(&buf[offset..])?;
    offset += consumed;

    // Packet number (pn_length bytes, big-endian)
    let pn_len = pn_length as usize;
    if offset + pn_len > buf.len() {
        return Err(QuicError::Stream("Truncated packet number".into()));
    }
    let mut pn_bytes = [0u8; 4];
    pn_bytes[4 - pn_len..].copy_from_slice(&buf[offset..offset + pn_len]);
    let packet_number = u32::from_be_bytes(pn_bytes);
    offset += pn_len;

    Ok((
        LongHeader {
            fixed_bit,
            packet_type,
            reserved_bits,
            pn_length,
            version,
            dcid,
            scid,
            token,
            payload_length: payload_length.value(),
            packet_number,
        },
        offset,
    ))
}

/// Encode a long header into the buffer. Returns bytes written.
///
/// Does NOT include the packet payload — only the header through the packet number.
///
/// # Errors
///
/// Returns [`QuicError::Stream`] if the encode buffer is too small.
pub fn encode_long_header(header: &LongHeader, buf: &mut [u8]) -> Result<usize> {
    let mut offset = 0;

    // First byte
    let pn_len_bits = header.pn_length.saturating_sub(1) & 0x03;
    let first: u8 = 0x80 // long header form bit
        | if header.fixed_bit { 0x40 } else { 0 }
        | ((header.packet_type as u8) << 4)
        | ((header.reserved_bits & 0x03) << 2)
        | pn_len_bits;

    if buf.is_empty() {
        return Err(QuicError::Stream("Encode buffer too small".into()));
    }
    buf[offset] = first;
    offset += 1;

    // Version
    if offset + 4 > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for version".into()));
    }
    buf[offset..offset + 4].copy_from_slice(&header.version.to_be_bytes());
    offset += 4;

    // DCID
    if offset + 1 + header.dcid.len() > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for DCID".into()));
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "QUIC CID length is at most 20 bytes per RFC 9000"
    )]
    let dcid_len_u8 = header.dcid.len() as u8;
    buf[offset] = dcid_len_u8;
    offset += 1;
    buf[offset..offset + header.dcid.len()].copy_from_slice(&header.dcid);
    offset += header.dcid.len();

    // SCID
    if offset + 1 + header.scid.len() > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for SCID".into()));
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "QUIC CID length is at most 20 bytes per RFC 9000"
    )]
    let scid_len_u8 = header.scid.len() as u8;
    buf[offset] = scid_len_u8;
    offset += 1;
    buf[offset..offset + header.scid.len()].copy_from_slice(&header.scid);
    offset += header.scid.len();

    // Token (Initial only)
    if header.packet_type == LongPacketType::Initial {
        let token_len = VarInt::new(header.token.len() as u64)?;
        let n = token_len.encode(&mut buf[offset..])?;
        offset += n;
        if offset + header.token.len() > buf.len() {
            return Err(QuicError::Stream("Encode buffer too small for token".into()));
        }
        buf[offset..offset + header.token.len()].copy_from_slice(&header.token);
        offset += header.token.len();
    }

    // Retry packets don't encode Length or PN
    if header.packet_type == LongPacketType::Retry {
        return Ok(offset);
    }

    // Payload length
    let pl = VarInt::new(header.payload_length)?;
    let n = pl.encode(&mut buf[offset..])?;
    offset += n;

    // Packet number
    let pn_len = header.pn_length as usize;
    if offset + pn_len > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for PN".into()));
    }
    let pn_bytes = header.packet_number.to_be_bytes();
    buf[offset..offset + pn_len].copy_from_slice(&pn_bytes[4 - pn_len..]);
    offset += pn_len;

    Ok(offset)
}
