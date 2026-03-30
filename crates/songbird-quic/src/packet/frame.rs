// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC frame types (RFC 9000 Section 12.4).
//!
//! Frames are the fundamental unit of data within QUIC packets.
//! Each packet can contain multiple frames.

use crate::error::{QuicError, Result};
use crate::varint::VarInt;

/// QUIC frame type identifiers (RFC 9000 Table 3).
pub mod frame_type {
    pub const PADDING: u64 = 0x00;
    pub const PING: u64 = 0x01;
    pub const ACK: u64 = 0x02;
    pub const ACK_ECN: u64 = 0x03;
    pub const RESET_STREAM: u64 = 0x04;
    pub const STOP_SENDING: u64 = 0x05;
    pub const CRYPTO: u64 = 0x06;
    pub const NEW_TOKEN: u64 = 0x07;
    pub const STREAM_BASE: u64 = 0x08;
    pub const STREAM_MAX: u64 = 0x0F;
    pub const MAX_DATA: u64 = 0x10;
    pub const MAX_STREAM_DATA: u64 = 0x11;
    pub const MAX_STREAMS_BIDI: u64 = 0x12;
    pub const MAX_STREAMS_UNI: u64 = 0x13;
    pub const DATA_BLOCKED: u64 = 0x14;
    pub const STREAM_DATA_BLOCKED: u64 = 0x15;
    pub const STREAMS_BLOCKED_BIDI: u64 = 0x16;
    pub const STREAMS_BLOCKED_UNI: u64 = 0x17;
    pub const NEW_CONNECTION_ID: u64 = 0x18;
    pub const RETIRE_CONNECTION_ID: u64 = 0x19;
    pub const PATH_CHALLENGE: u64 = 0x1A;
    pub const PATH_RESPONSE: u64 = 0x1B;
    pub const CONNECTION_CLOSE_QUIC: u64 = 0x1C;
    pub const CONNECTION_CLOSE_APP: u64 = 0x1D;
    pub const HANDSHAKE_DONE: u64 = 0x1E;
}

/// ACK range entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckRange {
    /// Gap before this range (number of unacknowledged packets minus 1).
    pub gap: u64,
    /// Length of this contiguous acknowledged range minus 1.
    pub ack_range: u64,
}

/// ECN counts for ACK frames.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EcnCounts {
    /// ECT(0) count.
    pub ect0: u64,
    /// ECT(1) count.
    pub ect1: u64,
    /// ECN-CE count.
    pub ecn_ce: u64,
}

/// Stream frame flags (encoded in the low 3 bits of the frame type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StreamFlags {
    /// OFF bit: offset field is present.
    pub has_offset: bool,
    /// LEN bit: length field is present.
    pub has_length: bool,
    /// FIN bit: this is the final data on the stream.
    pub is_fin: bool,
}

impl StreamFlags {
    /// Decode from the low 3 bits of the STREAM frame type byte.
    #[must_use]
    pub const fn from_type_bits(type_val: u64) -> Self {
        Self {
            has_offset: type_val & 0x04 != 0,
            has_length: type_val & 0x02 != 0,
            is_fin: type_val & 0x01 != 0,
        }
    }

    /// Encode to the STREAM frame type byte.
    #[must_use]
    pub const fn to_type(self) -> u64 {
        frame_type::STREAM_BASE
            | if self.has_offset { 0x04 } else { 0 }
            | if self.has_length { 0x02 } else { 0 }
            | if self.is_fin { 0x01 } else { 0 }
    }
}

/// All QUIC frame types (RFC 9000 Section 19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// PADDING frame: fills packet to desired size.
    Padding,

    /// PING frame: keeps connection alive / elicits ACK.
    Ping,

    /// ACK frame: acknowledges received packets.
    Ack {
        /// Largest packet number acknowledged.
        largest_acked: u64,
        /// ACK delay (in microseconds after decoding).
        ack_delay: u64,
        /// First ACK range (number of contiguous packets before largest_acked).
        first_ack_range: u64,
        /// Additional ACK ranges.
        ack_ranges: Vec<AckRange>,
        /// ECN counts (present in ACK_ECN frames).
        ecn: Option<EcnCounts>,
    },

    /// RESET_STREAM: abruptly terminates the sending part of a stream.
    ResetStream {
        stream_id: u64,
        application_error_code: u64,
        final_size: u64,
    },

    /// STOP_SENDING: request that a peer stop sending on a stream.
    StopSending {
        stream_id: u64,
        application_error_code: u64,
    },

    /// CRYPTO: carries TLS handshake data.
    Crypto {
        offset: u64,
        data: Vec<u8>,
    },

    /// NEW_TOKEN: provides a token for address validation on future connections.
    NewToken {
        token: Vec<u8>,
    },

    /// STREAM: carries application data on a stream.
    Stream {
        stream_id: u64,
        offset: u64,
        data: Vec<u8>,
        flags: StreamFlags,
    },

    /// MAX_DATA: informs peer of connection-level flow control limit.
    MaxData {
        maximum_data: u64,
    },

    /// MAX_STREAM_DATA: informs peer of stream-level flow control limit.
    MaxStreamData {
        stream_id: u64,
        maximum_stream_data: u64,
    },

    /// MAX_STREAMS (bidirectional): limits the number of bidi streams.
    MaxStreamsBidi {
        maximum_streams: u64,
    },

    /// MAX_STREAMS (unidirectional): limits the number of uni streams.
    MaxStreamsUni {
        maximum_streams: u64,
    },

    /// DATA_BLOCKED: indicates connection-level flow control is blocking.
    DataBlocked {
        maximum_data: u64,
    },

    /// STREAM_DATA_BLOCKED: indicates stream-level flow control is blocking.
    StreamDataBlocked {
        stream_id: u64,
        maximum_stream_data: u64,
    },

    /// STREAMS_BLOCKED (bidirectional).
    StreamsBlockedBidi {
        maximum_streams: u64,
    },

    /// STREAMS_BLOCKED (unidirectional).
    StreamsBlockedUni {
        maximum_streams: u64,
    },

    /// NEW_CONNECTION_ID: provides alternative connection IDs.
    NewConnectionId {
        sequence_number: u64,
        retire_prior_to: u64,
        connection_id: Vec<u8>,
        stateless_reset_token: [u8; 16],
    },

    /// RETIRE_CONNECTION_ID: indicates a connection ID is no longer used.
    RetireConnectionId {
        sequence_number: u64,
    },

    /// PATH_CHALLENGE: verifies reachability on a path.
    PathChallenge {
        data: [u8; 8],
    },

    /// PATH_RESPONSE: response to PATH_CHALLENGE.
    PathResponse {
        data: [u8; 8],
    },

    /// CONNECTION_CLOSE (QUIC layer): closes the connection.
    ConnectionCloseQuic {
        error_code: u64,
        frame_type: u64,
        reason_phrase: Vec<u8>,
    },

    /// CONNECTION_CLOSE (application layer): closes the connection.
    ConnectionCloseApp {
        error_code: u64,
        reason_phrase: Vec<u8>,
    },

    /// HANDSHAKE_DONE: server confirms handshake completion.
    HandshakeDone,
}

impl Frame {
    /// Decode one frame from the buffer. Returns the frame and bytes consumed.
    pub fn decode(buf: &[u8]) -> Result<(Self, usize)> {
        if buf.is_empty() {
            return Err(QuicError::Stream("Frame decode: empty buffer".into()));
        }

        // PADDING is special: type byte is 0x00 and consumes exactly 1 byte
        if buf[0] == 0x00 {
            return Ok((Frame::Padding, 1));
        }

        let (frame_type, mut offset) = VarInt::decode(buf)?;
        let ft = frame_type.value();

        match ft {
            frame_type::PING => Ok((Frame::Ping, offset)),

            frame_type::ACK | frame_type::ACK_ECN => {
                let (largest_acked, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (ack_delay, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (ack_range_count, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (first_ack_range, n) = VarInt::decode(&buf[offset..])?;
                offset += n;

                let mut ack_ranges = Vec::with_capacity(ack_range_count.value() as usize);
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

            frame_type::RESET_STREAM => {
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

            frame_type::STOP_SENDING => {
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

            frame_type::CRYPTO => {
                let (crypto_offset, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (length, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
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

            frame_type::NEW_TOKEN => {
                let (length, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let len = length.value() as usize;
                if offset + len > buf.len() {
                    return Err(QuicError::Stream("NEW_TOKEN data truncated".into()));
                }
                let token = buf[offset..offset + len].to_vec();
                offset += len;
                Ok((Frame::NewToken { token }, offset))
            }

            ft if (frame_type::STREAM_BASE..=frame_type::STREAM_MAX).contains(&ft) => {
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
                    l.value() as usize
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

            frame_type::MAX_DATA => {
                let (max_data, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::MaxData { maximum_data: max_data.value() }, offset))
            }

            frame_type::MAX_STREAM_DATA => {
                let (stream_id, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (max_data, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((
                    Frame::MaxStreamData {
                        stream_id: stream_id.value(),
                        maximum_stream_data: max_data.value(),
                    },
                    offset,
                ))
            }

            frame_type::MAX_STREAMS_BIDI => {
                let (max_streams, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::MaxStreamsBidi { maximum_streams: max_streams.value() }, offset))
            }

            frame_type::MAX_STREAMS_UNI => {
                let (max_streams, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::MaxStreamsUni { maximum_streams: max_streams.value() }, offset))
            }

            frame_type::DATA_BLOCKED => {
                let (max_data, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::DataBlocked { maximum_data: max_data.value() }, offset))
            }

            frame_type::STREAM_DATA_BLOCKED => {
                let (stream_id, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (max_data, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((
                    Frame::StreamDataBlocked {
                        stream_id: stream_id.value(),
                        maximum_stream_data: max_data.value(),
                    },
                    offset,
                ))
            }

            frame_type::STREAMS_BLOCKED_BIDI => {
                let (max_streams, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::StreamsBlockedBidi { maximum_streams: max_streams.value() }, offset))
            }

            frame_type::STREAMS_BLOCKED_UNI => {
                let (max_streams, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::StreamsBlockedUni { maximum_streams: max_streams.value() }, offset))
            }

            frame_type::NEW_CONNECTION_ID => {
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

            frame_type::RETIRE_CONNECTION_ID => {
                let (seq, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                Ok((Frame::RetireConnectionId { sequence_number: seq.value() }, offset))
            }

            frame_type::PATH_CHALLENGE => {
                if offset + 8 > buf.len() {
                    return Err(QuicError::Stream("PATH_CHALLENGE: truncated data".into()));
                }
                let mut data = [0u8; 8];
                data.copy_from_slice(&buf[offset..offset + 8]);
                offset += 8;
                Ok((Frame::PathChallenge { data }, offset))
            }

            frame_type::PATH_RESPONSE => {
                if offset + 8 > buf.len() {
                    return Err(QuicError::Stream("PATH_RESPONSE: truncated data".into()));
                }
                let mut data = [0u8; 8];
                data.copy_from_slice(&buf[offset..offset + 8]);
                offset += 8;
                Ok((Frame::PathResponse { data }, offset))
            }

            frame_type::CONNECTION_CLOSE_QUIC => {
                let (error_code, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (ft_val, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (reason_len, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
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

            frame_type::CONNECTION_CLOSE_APP => {
                let (error_code, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
                let (reason_len, n) = VarInt::decode(&buf[offset..])?;
                offset += n;
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

            frame_type::HANDSHAKE_DONE => Ok((Frame::HandshakeDone, offset)),

            _ => Err(QuicError::Stream(format!("Unknown frame type: {ft:#x}"))),
        }
    }

    /// Encode this frame into the buffer. Returns bytes written.
    pub fn encode(&self, buf: &mut [u8]) -> Result<usize> {
        let mut offset = 0;

        match self {
            Frame::Padding => {
                if buf.is_empty() {
                    return Err(QuicError::Stream("Buffer too small".into()));
                }
                buf[0] = 0x00;
                Ok(1)
            }

            Frame::Ping => {
                offset += VarInt::from_u32(frame_type::PING as u32).encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::Ack {
                largest_acked,
                ack_delay,
                first_ack_range,
                ack_ranges,
                ecn,
            } => {
                let ft = if ecn.is_some() { frame_type::ACK_ECN } else { frame_type::ACK };
                offset += VarInt::from_u32(ft as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*largest_acked)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*ack_delay)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(ack_ranges.len() as u64)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*first_ack_range)?.encode(&mut buf[offset..])?;
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

            Frame::ResetStream { stream_id, application_error_code, final_size } => {
                offset += VarInt::from_u32(frame_type::RESET_STREAM as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*stream_id)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*application_error_code)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*final_size)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::StopSending { stream_id, application_error_code } => {
                offset += VarInt::from_u32(frame_type::STOP_SENDING as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*stream_id)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*application_error_code)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::Crypto { offset: crypto_offset, data } => {
                offset += VarInt::from_u32(frame_type::CRYPTO as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*crypto_offset)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(data.len() as u64)?.encode(&mut buf[offset..])?;
                if offset + data.len() > buf.len() {
                    return Err(QuicError::Stream("Buffer too small for CRYPTO data".into()));
                }
                buf[offset..offset + data.len()].copy_from_slice(data);
                offset += data.len();
                Ok(offset)
            }

            Frame::NewToken { token } => {
                offset += VarInt::from_u32(frame_type::NEW_TOKEN as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(token.len() as u64)?.encode(&mut buf[offset..])?;
                if offset + token.len() > buf.len() {
                    return Err(QuicError::Stream("Buffer too small for token".into()));
                }
                buf[offset..offset + token.len()].copy_from_slice(token);
                offset += token.len();
                Ok(offset)
            }

            Frame::Stream { stream_id, offset: stream_offset, data, flags } => {
                let ft = flags.to_type();
                offset += VarInt::new(ft)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*stream_id)?.encode(&mut buf[offset..])?;
                if flags.has_offset {
                    offset += VarInt::new(*stream_offset)?.encode(&mut buf[offset..])?;
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

            Frame::MaxData { maximum_data } => {
                offset += VarInt::from_u32(frame_type::MAX_DATA as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_data)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::MaxStreamData { stream_id, maximum_stream_data } => {
                offset += VarInt::from_u32(frame_type::MAX_STREAM_DATA as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*stream_id)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_stream_data)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::MaxStreamsBidi { maximum_streams } => {
                offset += VarInt::from_u32(frame_type::MAX_STREAMS_BIDI as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_streams)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::MaxStreamsUni { maximum_streams } => {
                offset += VarInt::from_u32(frame_type::MAX_STREAMS_UNI as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_streams)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::DataBlocked { maximum_data } => {
                offset += VarInt::from_u32(frame_type::DATA_BLOCKED as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_data)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::StreamDataBlocked { stream_id, maximum_stream_data } => {
                offset += VarInt::from_u32(frame_type::STREAM_DATA_BLOCKED as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*stream_id)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_stream_data)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::StreamsBlockedBidi { maximum_streams } => {
                offset += VarInt::from_u32(frame_type::STREAMS_BLOCKED_BIDI as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_streams)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::StreamsBlockedUni { maximum_streams } => {
                offset += VarInt::from_u32(frame_type::STREAMS_BLOCKED_UNI as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*maximum_streams)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::NewConnectionId {
                sequence_number,
                retire_prior_to,
                connection_id,
                stateless_reset_token,
            } => {
                offset += VarInt::from_u32(frame_type::NEW_CONNECTION_ID as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*sequence_number)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*retire_prior_to)?.encode(&mut buf[offset..])?;
                if offset >= buf.len() {
                    return Err(QuicError::Stream("Buffer too small for CID length".into()));
                }
                buf[offset] = connection_id.len() as u8;
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

            Frame::RetireConnectionId { sequence_number } => {
                offset += VarInt::from_u32(frame_type::RETIRE_CONNECTION_ID as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*sequence_number)?.encode(&mut buf[offset..])?;
                Ok(offset)
            }

            Frame::PathChallenge { data } => {
                offset += VarInt::from_u32(frame_type::PATH_CHALLENGE as u32).encode(&mut buf[offset..])?;
                if offset + 8 > buf.len() {
                    return Err(QuicError::Stream("Buffer too small".into()));
                }
                buf[offset..offset + 8].copy_from_slice(data);
                offset += 8;
                Ok(offset)
            }

            Frame::PathResponse { data } => {
                offset += VarInt::from_u32(frame_type::PATH_RESPONSE as u32).encode(&mut buf[offset..])?;
                if offset + 8 > buf.len() {
                    return Err(QuicError::Stream("Buffer too small".into()));
                }
                buf[offset..offset + 8].copy_from_slice(data);
                offset += 8;
                Ok(offset)
            }

            Frame::ConnectionCloseQuic { error_code, frame_type: ft, reason_phrase } => {
                offset += VarInt::from_u32(frame_type::CONNECTION_CLOSE_QUIC as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*error_code)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(*ft)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(reason_phrase.len() as u64)?.encode(&mut buf[offset..])?;
                if offset + reason_phrase.len() > buf.len() {
                    return Err(QuicError::Stream("Buffer too small for reason".into()));
                }
                buf[offset..offset + reason_phrase.len()].copy_from_slice(reason_phrase);
                offset += reason_phrase.len();
                Ok(offset)
            }

            Frame::ConnectionCloseApp { error_code, reason_phrase } => {
                offset += VarInt::from_u32(frame_type::CONNECTION_CLOSE_APP as u32).encode(&mut buf[offset..])?;
                offset += VarInt::new(*error_code)?.encode(&mut buf[offset..])?;
                offset += VarInt::new(reason_phrase.len() as u64)?.encode(&mut buf[offset..])?;
                if offset + reason_phrase.len() > buf.len() {
                    return Err(QuicError::Stream("Buffer too small for reason".into()));
                }
                buf[offset..offset + reason_phrase.len()].copy_from_slice(reason_phrase);
                offset += reason_phrase.len();
                Ok(offset)
            }

            Frame::HandshakeDone => {
                offset += VarInt::from_u32(frame_type::HANDSHAKE_DONE as u32).encode(&mut buf[offset..])?;
                Ok(offset)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(frame: &Frame) {
        let mut buf = [0u8; 1024];
        let written = frame.encode(&mut buf).unwrap();
        let (decoded, consumed) = Frame::decode(&buf[..written]).unwrap();
        assert_eq!(consumed, written, "consumed != written for {frame:?}");
        assert_eq!(&decoded, frame, "roundtrip mismatch for {frame:?}");
    }

    #[test]
    fn padding_roundtrip() {
        roundtrip(&Frame::Padding);
    }

    #[test]
    fn ping_roundtrip() {
        roundtrip(&Frame::Ping);
    }

    #[test]
    fn ack_simple_roundtrip() {
        roundtrip(&Frame::Ack {
            largest_acked: 42,
            ack_delay: 100,
            first_ack_range: 5,
            ack_ranges: vec![],
            ecn: None,
        });
    }

    #[test]
    fn ack_with_ranges_roundtrip() {
        roundtrip(&Frame::Ack {
            largest_acked: 1000,
            ack_delay: 500,
            first_ack_range: 10,
            ack_ranges: vec![
                AckRange { gap: 2, ack_range: 5 },
                AckRange { gap: 0, ack_range: 3 },
            ],
            ecn: None,
        });
    }

    #[test]
    fn ack_ecn_roundtrip() {
        roundtrip(&Frame::Ack {
            largest_acked: 100,
            ack_delay: 0,
            first_ack_range: 0,
            ack_ranges: vec![],
            ecn: Some(EcnCounts { ect0: 10, ect1: 20, ecn_ce: 1 }),
        });
    }

    #[test]
    fn reset_stream_roundtrip() {
        roundtrip(&Frame::ResetStream {
            stream_id: 4,
            application_error_code: 0x0100,
            final_size: 2048,
        });
    }

    #[test]
    fn stop_sending_roundtrip() {
        roundtrip(&Frame::StopSending {
            stream_id: 8,
            application_error_code: 42,
        });
    }

    #[test]
    fn crypto_roundtrip() {
        roundtrip(&Frame::Crypto {
            offset: 0,
            data: vec![0x01, 0x00, 0x00, 0xF1, 0x03, 0x03],
        });
    }

    #[test]
    fn new_token_roundtrip() {
        roundtrip(&Frame::NewToken {
            token: vec![0xDE, 0xAD, 0xBE, 0xEF],
        });
    }

    #[test]
    fn stream_all_flags_roundtrip() {
        roundtrip(&Frame::Stream {
            stream_id: 0,
            offset: 100,
            data: b"hello quic".to_vec(),
            flags: StreamFlags {
                has_offset: true,
                has_length: true,
                is_fin: true,
            },
        });
    }

    #[test]
    fn stream_no_offset_no_length() {
        let frame = Frame::Stream {
            stream_id: 4,
            offset: 0,
            data: b"data".to_vec(),
            flags: StreamFlags {
                has_offset: false,
                has_length: false,
                is_fin: false,
            },
        };
        let mut buf = [0u8; 64];
        let written = frame.encode(&mut buf).unwrap();
        let (decoded, consumed) = Frame::decode(&buf[..written]).unwrap();
        assert_eq!(consumed, written);
        if let Frame::Stream { stream_id, data, .. } = decoded {
            assert_eq!(stream_id, 4);
            assert_eq!(data, b"data");
        } else {
            panic!("expected Stream frame");
        }
    }

    #[test]
    fn max_data_roundtrip() {
        roundtrip(&Frame::MaxData { maximum_data: 1_000_000 });
    }

    #[test]
    fn max_stream_data_roundtrip() {
        roundtrip(&Frame::MaxStreamData { stream_id: 4, maximum_stream_data: 65536 });
    }

    #[test]
    fn max_streams_roundtrip() {
        roundtrip(&Frame::MaxStreamsBidi { maximum_streams: 100 });
        roundtrip(&Frame::MaxStreamsUni { maximum_streams: 50 });
    }

    #[test]
    fn blocked_frames_roundtrip() {
        roundtrip(&Frame::DataBlocked { maximum_data: 9999 });
        roundtrip(&Frame::StreamDataBlocked { stream_id: 12, maximum_stream_data: 5000 });
        roundtrip(&Frame::StreamsBlockedBidi { maximum_streams: 10 });
        roundtrip(&Frame::StreamsBlockedUni { maximum_streams: 5 });
    }

    #[test]
    fn new_connection_id_roundtrip() {
        roundtrip(&Frame::NewConnectionId {
            sequence_number: 1,
            retire_prior_to: 0,
            connection_id: vec![0x01, 0x02, 0x03, 0x04],
            stateless_reset_token: [0xAA; 16],
        });
    }

    #[test]
    fn retire_connection_id_roundtrip() {
        roundtrip(&Frame::RetireConnectionId { sequence_number: 5 });
    }

    #[test]
    fn path_challenge_response_roundtrip() {
        roundtrip(&Frame::PathChallenge { data: [1, 2, 3, 4, 5, 6, 7, 8] });
        roundtrip(&Frame::PathResponse { data: [8, 7, 6, 5, 4, 3, 2, 1] });
    }

    #[test]
    fn connection_close_quic_roundtrip() {
        roundtrip(&Frame::ConnectionCloseQuic {
            error_code: 0x0A,
            frame_type: 0x06,
            reason_phrase: b"flow control error".to_vec(),
        });
    }

    #[test]
    fn connection_close_app_roundtrip() {
        roundtrip(&Frame::ConnectionCloseApp {
            error_code: 42,
            reason_phrase: b"shutting down".to_vec(),
        });
    }

    #[test]
    fn handshake_done_roundtrip() {
        roundtrip(&Frame::HandshakeDone);
    }

    #[test]
    fn stream_flags_bits() {
        let flags = StreamFlags { has_offset: true, has_length: true, is_fin: true };
        assert_eq!(flags.to_type(), 0x0F);
        let decoded = StreamFlags::from_type_bits(0x0F);
        assert_eq!(decoded, flags);

        let none = StreamFlags::from_type_bits(0x08);
        assert!(!none.has_offset);
        assert!(!none.has_length);
        assert!(!none.is_fin);
    }

    #[test]
    fn multiple_frames_in_sequence() {
        let frames = vec![
            Frame::Padding,
            Frame::Ping,
            Frame::Crypto { offset: 0, data: vec![0x01] },
            Frame::Ack {
                largest_acked: 10,
                ack_delay: 0,
                first_ack_range: 0,
                ack_ranges: vec![],
                ecn: None,
            },
        ];
        let mut buf = [0u8; 256];
        let mut total = 0;
        for f in &frames {
            total += f.encode(&mut buf[total..]).unwrap();
        }
        let mut offset = 0;
        for expected in &frames {
            let (decoded, consumed) = Frame::decode(&buf[offset..total]).unwrap();
            assert_eq!(&decoded, expected);
            offset += consumed;
        }
        assert_eq!(offset, total);
    }

    #[test]
    fn empty_buffer_errors() {
        assert!(Frame::decode(&[]).is_err());
    }

    #[test]
    fn unknown_frame_type_errors() {
        let mut buf = [0u8; 4];
        let n = VarInt::from_u32(0xFF).encode(&mut buf).unwrap();
        assert!(Frame::decode(&buf[..n]).is_err());
    }
}
