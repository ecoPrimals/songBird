// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC frame encoding (RFC 9000 Section 12.4).

use super::Frame;
use super::frame_type;
use crate::error::{QuicError, Result};
use crate::varint::VarInt;

impl Frame {
    /// Encode this frame into the buffer. Returns bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the buffer is too small to hold
    /// the encoded frame.
    #[expect(clippy::too_many_lines, reason = "exhaustive match over all QUIC frame variants")]
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Padding => encode_padding(buf),
            Self::Ping => encode_type_only(frame_type::PING, buf),
            Self::Ack {
                largest_acked,
                ack_delay,
                first_ack_range,
                ack_ranges,
                ecn,
            } => encode_ack(
                *largest_acked,
                *ack_delay,
                *first_ack_range,
                ack_ranges,
                ecn.as_ref(),
                buf,
            ),
            Self::ResetStream {
                stream_id,
                application_error_code,
                final_size,
            } => encode_reset_stream(*stream_id, *application_error_code, *final_size, buf),
            Self::StopSending {
                stream_id,
                application_error_code,
            } => encode_stop_sending(*stream_id, *application_error_code, buf),
            Self::Crypto {
                offset: crypto_offset,
                data,
            } => encode_crypto(*crypto_offset, data, buf),
            Self::NewToken {
                token,
            } => encode_new_token(token, buf),
            Self::Stream {
                stream_id,
                offset: stream_offset,
                data,
                flags,
            } => encode_stream(*stream_id, *stream_offset, data, *flags, buf),
            Self::MaxData {
                maximum_data,
            } => encode_varint_frame(frame_type::MAX_DATA, *maximum_data, buf),
            Self::MaxStreamData {
                stream_id,
                maximum_stream_data,
            } => encode_two_varint_frame(
                frame_type::MAX_STREAM_DATA,
                *stream_id,
                *maximum_stream_data,
                buf,
            ),
            Self::MaxStreamsBidi {
                maximum_streams,
            } => encode_varint_frame(frame_type::MAX_STREAMS_BIDI, *maximum_streams, buf),
            Self::MaxStreamsUni {
                maximum_streams,
            } => encode_varint_frame(frame_type::MAX_STREAMS_UNI, *maximum_streams, buf),
            Self::DataBlocked {
                maximum_data,
            } => encode_varint_frame(frame_type::DATA_BLOCKED, *maximum_data, buf),
            Self::StreamDataBlocked {
                stream_id,
                maximum_stream_data,
            } => encode_two_varint_frame(
                frame_type::STREAM_DATA_BLOCKED,
                *stream_id,
                *maximum_stream_data,
                buf,
            ),
            Self::StreamsBlockedBidi {
                maximum_streams,
            } => encode_varint_frame(frame_type::STREAMS_BLOCKED_BIDI, *maximum_streams, buf),
            Self::StreamsBlockedUni {
                maximum_streams,
            } => encode_varint_frame(frame_type::STREAMS_BLOCKED_UNI, *maximum_streams, buf),
            Self::NewConnectionId {
                sequence_number,
                retire_prior_to,
                connection_id,
                stateless_reset_token,
            } => encode_new_connection_id(
                *sequence_number,
                *retire_prior_to,
                connection_id,
                stateless_reset_token,
                buf,
            ),
            Self::RetireConnectionId {
                sequence_number,
            } => encode_varint_frame(frame_type::RETIRE_CONNECTION_ID, *sequence_number, buf),
            Self::PathChallenge {
                data,
            } => encode_path(frame_type::PATH_CHALLENGE, *data, buf),
            Self::PathResponse {
                data,
            } => encode_path(frame_type::PATH_RESPONSE, *data, buf),
            Self::ConnectionCloseQuic {
                error_code,
                frame_type: ft,
                reason_phrase,
            } => encode_connection_close_quic(*error_code, *ft, reason_phrase, buf),
            Self::ConnectionCloseApp {
                error_code,
                reason_phrase,
            } => encode_connection_close_app(*error_code, reason_phrase, buf),
            Self::HandshakeDone => encode_type_only(frame_type::HANDSHAKE_DONE, buf),
        }
    }
}

fn encode_padding(buf: &mut [u8]) -> Result<usize> {
    if buf.is_empty() {
        return Err(QuicError::Stream("Buffer too small".into()));
    }
    buf[0] = 0x00;
    Ok(1)
}

fn encode_type_only(ft: u64, buf: &mut [u8]) -> Result<usize> {
    #[expect(clippy::cast_possible_truncation, reason = "frame types fit in u32")]
    let n = VarInt::from_u32(ft as u32).encode(buf)?;
    Ok(n)
}

/// Encode a frame with a type byte and a single varint payload.
fn encode_varint_frame(ft: u64, val: u64, buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(ft, buf)?;
    offset += VarInt::new(val)?.encode(&mut buf[offset..])?;
    Ok(offset)
}

/// Encode a frame with a type byte and two varint payloads.
fn encode_two_varint_frame(ft: u64, v1: u64, v2: u64, buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(ft, buf)?;
    offset += VarInt::new(v1)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(v2)?.encode(&mut buf[offset..])?;
    Ok(offset)
}

fn encode_ack(
    largest_acked: u64,
    ack_delay: u64,
    first_ack_range: u64,
    ack_ranges: &[super::AckRange],
    ecn: Option<&super::EcnCounts>,
    buf: &mut [u8],
) -> Result<usize> {
    let ft = if ecn.is_some() {
        frame_type::ACK_ECN
    } else {
        frame_type::ACK
    };
    let mut offset = encode_type_only(ft, buf)?;
    offset += VarInt::new(largest_acked)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(ack_delay)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(ack_ranges.len() as u64)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(first_ack_range)?.encode(&mut buf[offset..])?;
    for range in ack_ranges {
        offset += VarInt::new(range.gap)?.encode(&mut buf[offset..])?;
        offset += VarInt::new(range.ack_range)?.encode(&mut buf[offset..])?;
    }
    if let Some(ecn) = ecn {
        offset += VarInt::new(ecn.ect0)?.encode(&mut buf[offset..])?;
        offset += VarInt::new(ecn.ect1)?.encode(&mut buf[offset..])?;
        offset += VarInt::new(ecn.ecn_ce)?.encode(&mut buf[offset..])?;
    }
    Ok(offset)
}

fn encode_reset_stream(
    stream_id: u64,
    error_code: u64,
    final_size: u64,
    buf: &mut [u8],
) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::RESET_STREAM, buf)?;
    offset += VarInt::new(stream_id)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(error_code)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(final_size)?.encode(&mut buf[offset..])?;
    Ok(offset)
}

fn encode_stop_sending(stream_id: u64, error_code: u64, buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::STOP_SENDING, buf)?;
    offset += VarInt::new(stream_id)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(error_code)?.encode(&mut buf[offset..])?;
    Ok(offset)
}

fn encode_crypto(crypto_offset: u64, data: &[u8], buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::CRYPTO, buf)?;
    offset += VarInt::new(crypto_offset)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(data.len() as u64)?.encode(&mut buf[offset..])?;
    if offset + data.len() > buf.len() {
        return Err(QuicError::Stream("Buffer too small for CRYPTO data".into()));
    }
    buf[offset..offset + data.len()].copy_from_slice(data);
    offset += data.len();
    Ok(offset)
}

fn encode_new_token(token: &[u8], buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::NEW_TOKEN, buf)?;
    offset += VarInt::new(token.len() as u64)?.encode(&mut buf[offset..])?;
    if offset + token.len() > buf.len() {
        return Err(QuicError::Stream("Buffer too small for token".into()));
    }
    buf[offset..offset + token.len()].copy_from_slice(token);
    offset += token.len();
    Ok(offset)
}

fn encode_stream(
    stream_id: u64,
    stream_offset: u64,
    data: &[u8],
    flags: super::StreamFlags,
    buf: &mut [u8],
) -> Result<usize> {
    let ft = flags.to_type();
    let mut offset = 0;
    offset += VarInt::new(ft)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(stream_id)?.encode(&mut buf[offset..])?;
    if flags.has_offset {
        offset += VarInt::new(stream_offset)?.encode(&mut buf[offset..])?;
    }
    if flags.has_length {
        offset += VarInt::new(data.len() as u64)?.encode(&mut buf[offset..])?;
    }
    if offset + data.len() > buf.len() {
        return Err(QuicError::Stream("Buffer too small for STREAM data".into()));
    }
    buf[offset..offset + data.len()].copy_from_slice(data);
    offset += data.len();
    Ok(offset)
}

fn encode_new_connection_id(
    sequence_number: u64,
    retire_prior_to: u64,
    connection_id: &[u8],
    stateless_reset_token: &[u8; 16],
    buf: &mut [u8],
) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::NEW_CONNECTION_ID, buf)?;
    offset += VarInt::new(sequence_number)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(retire_prior_to)?.encode(&mut buf[offset..])?;
    if offset >= buf.len() {
        return Err(QuicError::Stream("Buffer too small for CID length".into()));
    }
    #[expect(clippy::cast_possible_truncation, reason = "CID length max is 20")]
    {
        buf[offset] = connection_id.len() as u8;
    }
    offset += 1;
    if offset + connection_id.len() + 16 > buf.len() {
        return Err(QuicError::Stream("Buffer too small for CID + token".into()));
    }
    buf[offset..offset + connection_id.len()].copy_from_slice(connection_id);
    offset += connection_id.len();
    buf[offset..offset + 16].copy_from_slice(stateless_reset_token);
    offset += 16;
    Ok(offset)
}

fn encode_path(ft: u64, data: [u8; 8], buf: &mut [u8]) -> Result<usize> {
    let mut offset = encode_type_only(ft, buf)?;
    if offset + 8 > buf.len() {
        return Err(QuicError::Stream("Buffer too small".into()));
    }
    buf[offset..offset + 8].copy_from_slice(&data);
    offset += 8;
    Ok(offset)
}

fn encode_connection_close_quic(
    error_code: u64,
    ft: u64,
    reason_phrase: &[u8],
    buf: &mut [u8],
) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::CONNECTION_CLOSE_QUIC, buf)?;
    offset += VarInt::new(error_code)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(ft)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(reason_phrase.len() as u64)?.encode(&mut buf[offset..])?;
    if offset + reason_phrase.len() > buf.len() {
        return Err(QuicError::Stream("Buffer too small for reason".into()));
    }
    buf[offset..offset + reason_phrase.len()].copy_from_slice(reason_phrase);
    offset += reason_phrase.len();
    Ok(offset)
}

fn encode_connection_close_app(
    error_code: u64,
    reason_phrase: &[u8],
    buf: &mut [u8],
) -> Result<usize> {
    let mut offset = encode_type_only(frame_type::CONNECTION_CLOSE_APP, buf)?;
    offset += VarInt::new(error_code)?.encode(&mut buf[offset..])?;
    offset += VarInt::new(reason_phrase.len() as u64)?.encode(&mut buf[offset..])?;
    if offset + reason_phrase.len() > buf.len() {
        return Err(QuicError::Stream("Buffer too small for reason".into()));
    }
    buf[offset..offset + reason_phrase.len()].copy_from_slice(reason_phrase);
    offset += reason_phrase.len();
    Ok(offset)
}
