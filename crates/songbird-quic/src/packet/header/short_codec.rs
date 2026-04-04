// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::{QuicError, Result};

use super::ShortHeader;

/// Decode a short header. Requires knowing the DCID length (from connection state).
///
/// # Errors
///
/// Returns [`QuicError::Stream`] on truncated or malformed wire data.
pub fn decode_short_header(buf: &[u8], dcid_len: usize) -> Result<(ShortHeader, usize)> {
    if buf.is_empty() {
        return Err(QuicError::Stream("Short header too short".into()));
    }

    let first = buf[0];
    if first & 0x80 != 0 {
        return Err(QuicError::Stream("Not a short header (form bit = 1)".into()));
    }

    let fixed_bit = first & 0x40 != 0;
    let spin_bit = first & 0x20 != 0;
    let reserved_bits = (first >> 3) & 0x03;
    let key_phase = first & 0x04 != 0;
    let pn_length = (first & 0x03) + 1;

    let mut offset = 1;

    if offset + dcid_len > buf.len() {
        return Err(QuicError::Stream("Truncated short header DCID".into()));
    }
    let dcid = buf[offset..offset + dcid_len].to_vec();
    offset += dcid_len;

    let pn_len = pn_length as usize;
    if offset + pn_len > buf.len() {
        return Err(QuicError::Stream("Truncated short header PN".into()));
    }
    let mut pn_bytes = [0u8; 4];
    pn_bytes[4 - pn_len..].copy_from_slice(&buf[offset..offset + pn_len]);
    let packet_number = u32::from_be_bytes(pn_bytes);
    offset += pn_len;

    Ok((
        ShortHeader {
            fixed_bit,
            spin_bit,
            reserved_bits,
            key_phase,
            pn_length,
            dcid,
            packet_number,
        },
        offset,
    ))
}

/// Encode a short header into the buffer. Returns bytes written.
///
/// # Errors
///
/// Returns [`QuicError::Stream`] if the encode buffer is too small.
pub fn encode_short_header(header: &ShortHeader, buf: &mut [u8]) -> Result<usize> {
    let mut offset = 0;

    let pn_len_bits = header.pn_length.saturating_sub(1) & 0x03;
    let first: u8 = if header.fixed_bit {
        0x40
    } else {
        0
    } | if header.spin_bit {
        0x20
    } else {
        0
    } | ((header.reserved_bits & 0x03) << 3)
        | if header.key_phase {
            0x04
        } else {
            0
        }
        | pn_len_bits;

    if buf.is_empty() {
        return Err(QuicError::Stream("Encode buffer too small".into()));
    }
    buf[offset] = first;
    offset += 1;

    if offset + header.dcid.len() > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for DCID".into()));
    }
    buf[offset..offset + header.dcid.len()].copy_from_slice(&header.dcid);
    offset += header.dcid.len();

    let pn_len = header.pn_length as usize;
    if offset + pn_len > buf.len() {
        return Err(QuicError::Stream("Encode buffer too small for PN".into()));
    }
    let pn_bytes = header.packet_number.to_be_bytes();
    buf[offset..offset + pn_len].copy_from_slice(&pn_bytes[4 - pn_len..]);
    offset += pn_len;

    Ok(offset)
}
