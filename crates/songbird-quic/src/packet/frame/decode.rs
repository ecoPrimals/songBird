// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC frame decoding (RFC 9000 Section 12.4).

use super::frame_type;
use super::{AckRange, EcnCounts, Frame, StreamFlags};
use crate::error::{QuicError, Result};
use crate::varint::VarInt;

impl Frame {
    /// Decode one frame from the buffer. Returns the frame and bytes consumed.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the buffer is empty, truncated, or
    /// contains an unknown frame type.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        if buf.is_empty() {
            return Err(QuicError::Stream("Frame decode: empty buffer".into()));
        }

        if buf[0] == 0x00 {
            return Ok((Self::Padding, 1));
        }

        let (frame_type, offset) = VarInt::decode(buf)?;
        let ft = frame_type.value();

        match ft {
            frame_type::PING => Ok((Self::Ping, offset)),
            frame_type::ACK | frame_type::ACK_ECN => decode_ack(buf, ft, offset),
            frame_type::RESET_STREAM => decode_reset_stream(buf, offset),
            frame_type::STOP_SENDING => decode_stop_sending(buf, offset),
            frame_type::CRYPTO => decode_crypto(buf, offset),
            frame_type::NEW_TOKEN => decode_new_token(buf, offset),
            ft if (frame_type::STREAM_BASE..=frame_type::STREAM_MAX).contains(&ft) => {
                decode_stream(buf, ft, offset)
            }
            frame_type::MAX_DATA => decode_single_varint(buf, offset, Self::max_data),
            frame_type::MAX_STREAM_DATA => {
                decode_stream_data_pair(buf, offset, Self::max_stream_data)
            }
            frame_type::MAX_STREAMS_BIDI => {
                decode_single_varint(buf, offset, Self::max_streams_bidi)
            }
            frame_type::MAX_STREAMS_UNI => decode_single_varint(buf, offset, Self::max_streams_uni),
            frame_type::DATA_BLOCKED => decode_single_varint(buf, offset, Self::data_blocked),
            frame_type::STREAM_DATA_BLOCKED => {
                decode_stream_data_pair(buf, offset, Self::stream_data_blocked)
            }
            frame_type::STREAMS_BLOCKED_BIDI => {
                decode_single_varint(buf, offset, Self::streams_blocked_bidi)
            }
            frame_type::STREAMS_BLOCKED_UNI => {
                decode_single_varint(buf, offset, Self::streams_blocked_uni)
            }
            frame_type::NEW_CONNECTION_ID => decode_new_connection_id(buf, offset),
            frame_type::RETIRE_CONNECTION_ID => {
                decode_single_varint(buf, offset, Self::retire_connection_id)
            }
            frame_type::PATH_CHALLENGE => decode_path_data(buf, offset, |d| Self::PathChallenge {
                data: d,
            }),
            frame_type::PATH_RESPONSE => decode_path_data(buf, offset, |d| Self::PathResponse {
                data: d,
            }),
            frame_type::CONNECTION_CLOSE_QUIC => decode_connection_close_quic(buf, offset),
            frame_type::CONNECTION_CLOSE_APP => decode_connection_close_app(buf, offset),
            frame_type::HANDSHAKE_DONE => Ok((Self::HandshakeDone, offset)),
            _ => Err(QuicError::Stream(format!("Unknown frame type: {ft:#x}"))),
        }
    }

    fn max_data(v: u64) -> Self {
        Self::MaxData {
            maximum_data: v,
        }
    }
    fn max_streams_bidi(v: u64) -> Self {
        Self::MaxStreamsBidi {
            maximum_streams: v,
        }
    }
    fn max_streams_uni(v: u64) -> Self {
        Self::MaxStreamsUni {
            maximum_streams: v,
        }
    }
    fn data_blocked(v: u64) -> Self {
        Self::DataBlocked {
            maximum_data: v,
        }
    }
    fn streams_blocked_bidi(v: u64) -> Self {
        Self::StreamsBlockedBidi {
            maximum_streams: v,
        }
    }
    fn streams_blocked_uni(v: u64) -> Self {
        Self::StreamsBlockedUni {
            maximum_streams: v,
        }
    }
    fn retire_connection_id(v: u64) -> Self {
        Self::RetireConnectionId {
            sequence_number: v,
        }
    }
    fn max_stream_data(id: u64, val: u64) -> Self {
        Self::MaxStreamData {
            stream_id: id,
            maximum_stream_data: val,
        }
    }
    fn stream_data_blocked(id: u64, val: u64) -> Self {
        Self::StreamDataBlocked {
            stream_id: id,
            maximum_stream_data: val,
        }
    }
}

fn decode_ack(buf: &[u8], ft: u64, mut offset: usize) -> Result<(Frame, usize)> {
    let (largest_acked, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (ack_delay, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (ack_range_count, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (first_ack_range, n) = VarInt::decode(&buf[offset..])?;
    offset += n;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "ACK range count is bounded by QUIC frame size; allocation uses usize"
    )]
    let ack_cap = ack_range_count.value() as usize;
    let mut ack_ranges = Vec::with_capacity(ack_cap);
    for _ in 0..ack_range_count.value() {
        let (gap, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        let (ack_range, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        ack_ranges.push(AckRange {
            gap: gap.value(),
            ack_range: ack_range.value(),
        });
    }

    let ecn = if ft == frame_type::ACK_ECN {
        let (ect0, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        let (ect1, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        let (ecn_ce, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        Some(EcnCounts {
            ect0: ect0.value(),
            ect1: ect1.value(),
            ecn_ce: ecn_ce.value(),
        })
    } else {
        None
    };

    Ok((
        Frame::Ack {
            largest_acked: largest_acked.value(),
            ack_delay: ack_delay.value(),
            first_ack_range: first_ack_range.value(),
            ack_ranges,
            ecn,
        },
        offset,
    ))
}

fn decode_reset_stream(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (stream_id, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (error_code, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (final_size, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    Ok((
        Frame::ResetStream {
            stream_id: stream_id.value(),
            application_error_code: error_code.value(),
            final_size: final_size.value(),
        },
        offset,
    ))
}

fn decode_stop_sending(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (stream_id, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (error_code, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    Ok((
        Frame::StopSending {
            stream_id: stream_id.value(),
            application_error_code: error_code.value(),
        },
        offset,
    ))
}

fn decode_crypto(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (crypto_offset, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (length, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "CRYPTO payload length from VarInt; bounds-checked against buffer"
    )]
    let len = length.value() as usize;
    if offset + len > buf.len() {
        return Err(QuicError::Stream("CRYPTO frame data truncated".into()));
    }
    let data = buf[offset..offset + len].to_vec();
    offset += len;
    Ok((
        Frame::Crypto {
            offset: crypto_offset.value(),
            data,
        },
        offset,
    ))
}

fn decode_new_token(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (length, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "token length from VarInt; bounds-checked against buffer"
    )]
    let len = length.value() as usize;
    if offset + len > buf.len() {
        return Err(QuicError::Stream("NEW_TOKEN data truncated".into()));
    }
    let token = buf[offset..offset + len].to_vec();
    offset += len;
    Ok((
        Frame::NewToken {
            token,
        },
        offset,
    ))
}

fn decode_stream(buf: &[u8], ft: u64, mut offset: usize) -> Result<(Frame, usize)> {
    let flags = StreamFlags::from_type_bits(ft);
    let (stream_id, n) = VarInt::decode(&buf[offset..])?;
    offset += n;

    let stream_offset = if flags.has_offset {
        let (o, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        o.value()
    } else {
        0
    };

    let data_len = if flags.has_length {
        let (l, n) = VarInt::decode(&buf[offset..])?;
        offset += n;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "STREAM length from VarInt; bounds-checked against buffer"
        )]
        let stream_len = l.value() as usize;
        stream_len
    } else {
        buf.len() - offset
    };

    if offset + data_len > buf.len() {
        return Err(QuicError::Stream("STREAM frame data truncated".into()));
    }
    let data = buf[offset..offset + data_len].to_vec();
    offset += data_len;

    Ok((
        Frame::Stream {
            stream_id: stream_id.value(),
            offset: stream_offset,
            data,
            flags,
        },
        offset,
    ))
}

/// Decode a frame that contains a single varint field.
fn decode_single_varint(
    buf: &[u8],
    mut offset: usize,
    ctor: fn(u64) -> Frame,
) -> Result<(Frame, usize)> {
    let (val, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    Ok((ctor(val.value()), offset))
}

/// Decode a frame that contains a stream ID + a single varint field.
fn decode_stream_data_pair(
    buf: &[u8],
    mut offset: usize,
    ctor: fn(u64, u64) -> Frame,
) -> Result<(Frame, usize)> {
    let (stream_id, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (val, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    Ok((ctor(stream_id.value(), val.value()), offset))
}

fn decode_new_connection_id(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (seq, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (retire, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    if offset >= buf.len() {
        return Err(QuicError::Stream("NEW_CONNECTION_ID: missing CID length".into()));
    }
    let cid_len = buf[offset] as usize;
    offset += 1;
    if cid_len > 20 || offset + cid_len + 16 > buf.len() {
        return Err(QuicError::Stream("NEW_CONNECTION_ID: invalid CID or token".into()));
    }
    let connection_id = buf[offset..offset + cid_len].to_vec();
    offset += cid_len;
    let mut token = [0u8; 16];
    token.copy_from_slice(&buf[offset..offset + 16]);
    offset += 16;
    Ok((
        Frame::NewConnectionId {
            sequence_number: seq.value(),
            retire_prior_to: retire.value(),
            connection_id,
            stateless_reset_token: token,
        },
        offset,
    ))
}

fn decode_path_data(
    buf: &[u8],
    mut offset: usize,
    ctor: impl FnOnce([u8; 8]) -> Frame,
) -> Result<(Frame, usize)> {
    if offset + 8 > buf.len() {
        return Err(QuicError::Stream("PATH frame: truncated data".into()));
    }
    let mut data = [0u8; 8];
    data.copy_from_slice(&buf[offset..offset + 8]);
    offset += 8;
    Ok((ctor(data), offset))
}

fn decode_connection_close_quic(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (error_code, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (ft_val, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (reason_len, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "reason length from VarInt; bounds-checked against buffer"
    )]
    let rlen = reason_len.value() as usize;
    if offset + rlen > buf.len() {
        return Err(QuicError::Stream("CONNECTION_CLOSE reason truncated".into()));
    }
    let reason_phrase = buf[offset..offset + rlen].to_vec();
    offset += rlen;
    Ok((
        Frame::ConnectionCloseQuic {
            error_code: error_code.value(),
            frame_type: ft_val.value(),
            reason_phrase,
        },
        offset,
    ))
}

fn decode_connection_close_app(buf: &[u8], mut offset: usize) -> Result<(Frame, usize)> {
    let (error_code, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    let (reason_len, n) = VarInt::decode(&buf[offset..])?;
    offset += n;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "reason length from VarInt; bounds-checked against buffer"
    )]
    let rlen = reason_len.value() as usize;
    if offset + rlen > buf.len() {
        return Err(QuicError::Stream("CONNECTION_CLOSE reason truncated".into()));
    }
    let reason_phrase = buf[offset..offset + rlen].to_vec();
    offset += rlen;
    Ok((
        Frame::ConnectionCloseApp {
            error_code: error_code.value(),
            reason_phrase,
        },
        offset,
    ))
}
