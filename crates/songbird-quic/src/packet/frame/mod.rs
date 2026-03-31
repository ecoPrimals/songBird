// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC frame types (RFC 9000 Section 12.4).
//!
//! Frames are the fundamental unit of data within QUIC packets.
//! Each packet can contain multiple frames.

mod decode;
mod encode;

/// QUIC frame type identifiers (RFC 9000 Table 3).
pub mod frame_type {
    /// PADDING frame type.
    pub const PADDING: u64 = 0x00;
    /// PING frame type.
    pub const PING: u64 = 0x01;
    /// ACK frame type.
    pub const ACK: u64 = 0x02;
    /// ACK with ECN counts frame type.
    pub const ACK_ECN: u64 = 0x03;
    /// `RESET_STREAM` frame type.
    pub const RESET_STREAM: u64 = 0x04;
    /// `STOP_SENDING` frame type.
    pub const STOP_SENDING: u64 = 0x05;
    /// CRYPTO frame type.
    pub const CRYPTO: u64 = 0x06;
    /// `NEW_TOKEN` frame type.
    pub const NEW_TOKEN: u64 = 0x07;
    /// STREAM frame base type (low 3 bits encode flags).
    pub const STREAM_BASE: u64 = 0x08;
    /// STREAM frame maximum type value.
    pub const STREAM_MAX: u64 = 0x0F;
    /// `MAX_DATA` frame type.
    pub const MAX_DATA: u64 = 0x10;
    /// `MAX_STREAM_DATA` frame type.
    pub const MAX_STREAM_DATA: u64 = 0x11;
    /// `MAX_STREAMS` (bidirectional) frame type.
    pub const MAX_STREAMS_BIDI: u64 = 0x12;
    /// `MAX_STREAMS` (unidirectional) frame type.
    pub const MAX_STREAMS_UNI: u64 = 0x13;
    /// `DATA_BLOCKED` frame type.
    pub const DATA_BLOCKED: u64 = 0x14;
    /// `STREAM_DATA_BLOCKED` frame type.
    pub const STREAM_DATA_BLOCKED: u64 = 0x15;
    /// `STREAMS_BLOCKED` (bidirectional) frame type.
    pub const STREAMS_BLOCKED_BIDI: u64 = 0x16;
    /// `STREAMS_BLOCKED` (unidirectional) frame type.
    pub const STREAMS_BLOCKED_UNI: u64 = 0x17;
    /// `NEW_CONNECTION_ID` frame type.
    pub const NEW_CONNECTION_ID: u64 = 0x18;
    /// `RETIRE_CONNECTION_ID` frame type.
    pub const RETIRE_CONNECTION_ID: u64 = 0x19;
    /// `PATH_CHALLENGE` frame type.
    pub const PATH_CHALLENGE: u64 = 0x1A;
    /// `PATH_RESPONSE` frame type.
    pub const PATH_RESPONSE: u64 = 0x1B;
    /// `CONNECTION_CLOSE` (QUIC layer) frame type.
    pub const CONNECTION_CLOSE_QUIC: u64 = 0x1C;
    /// `CONNECTION_CLOSE` (application layer) frame type.
    pub const CONNECTION_CLOSE_APP: u64 = 0x1D;
    /// `HANDSHAKE_DONE` frame type.
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
            | if self.has_offset {
                0x04
            } else {
                0
            }
            | if self.has_length {
                0x02
            } else {
                0
            }
            | if self.is_fin {
                0x01
            } else {
                0
            }
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
        /// First ACK range (number of contiguous packets before `largest_acked`).
        first_ack_range: u64,
        /// Additional ACK ranges.
        ack_ranges: Vec<AckRange>,
        /// ECN counts (present in `ACK_ECN` frames).
        ecn: Option<EcnCounts>,
    },

    /// `RESET_STREAM`: abruptly terminates the sending part of a stream.
    ResetStream {
        /// Stream identifier.
        stream_id: u64,
        /// Application protocol error code.
        application_error_code: u64,
        /// Final size of data sent on the stream.
        final_size: u64,
    },

    /// `STOP_SENDING`: request that a peer stop sending on a stream.
    StopSending {
        /// Stream identifier.
        stream_id: u64,
        /// Application protocol error code.
        application_error_code: u64,
    },

    /// CRYPTO: carries TLS handshake data.
    Crypto {
        /// Byte offset within the crypto stream.
        offset: u64,
        /// Handshake data.
        data: Vec<u8>,
    },

    /// `NEW_TOKEN`: provides a token for address validation on future connections.
    NewToken {
        /// Opaque token value.
        token: Vec<u8>,
    },

    /// STREAM: carries application data on a stream.
    Stream {
        /// Stream identifier.
        stream_id: u64,
        /// Byte offset within the stream.
        offset: u64,
        /// Application data.
        data: Vec<u8>,
        /// Stream frame flags (OFF, LEN, FIN).
        flags: StreamFlags,
    },

    /// `MAX_DATA`: informs peer of connection-level flow control limit.
    MaxData {
        /// Maximum data the peer may send on the connection.
        maximum_data: u64,
    },

    /// `MAX_STREAM_DATA`: informs peer of stream-level flow control limit.
    MaxStreamData {
        /// Stream identifier.
        stream_id: u64,
        /// Maximum data the peer may send on this stream.
        maximum_stream_data: u64,
    },

    /// `MAX_STREAMS` (bidirectional): limits the number of bidi streams.
    MaxStreamsBidi {
        /// Maximum number of bidirectional streams.
        maximum_streams: u64,
    },

    /// `MAX_STREAMS` (unidirectional): limits the number of uni streams.
    MaxStreamsUni {
        /// Maximum number of unidirectional streams.
        maximum_streams: u64,
    },

    /// `DATA_BLOCKED`: indicates connection-level flow control is blocking.
    DataBlocked {
        /// Connection-level limit that is blocking.
        maximum_data: u64,
    },

    /// `STREAM_DATA_BLOCKED`: indicates stream-level flow control is blocking.
    StreamDataBlocked {
        /// Stream identifier.
        stream_id: u64,
        /// Stream-level limit that is blocking.
        maximum_stream_data: u64,
    },

    /// `STREAMS_BLOCKED` (bidirectional).
    StreamsBlockedBidi {
        /// Bidirectional stream limit that is blocking.
        maximum_streams: u64,
    },

    /// `STREAMS_BLOCKED` (unidirectional).
    StreamsBlockedUni {
        /// Unidirectional stream limit that is blocking.
        maximum_streams: u64,
    },

    /// `NEW_CONNECTION_ID`: provides alternative connection IDs.
    NewConnectionId {
        /// Sequence number for ordering.
        sequence_number: u64,
        /// Retire all CIDs before this sequence number.
        retire_prior_to: u64,
        /// The new connection ID.
        connection_id: Vec<u8>,
        /// Stateless reset token for this CID.
        stateless_reset_token: [u8; 16],
    },

    /// `RETIRE_CONNECTION_ID`: indicates a connection ID is no longer used.
    RetireConnectionId {
        /// Sequence number of the CID being retired.
        sequence_number: u64,
    },

    /// `PATH_CHALLENGE`: verifies reachability on a path.
    PathChallenge {
        /// 8-byte challenge data.
        data: [u8; 8],
    },

    /// `PATH_RESPONSE`: response to `PATH_CHALLENGE`.
    PathResponse {
        /// 8-byte response data (must echo the challenge).
        data: [u8; 8],
    },

    /// `CONNECTION_CLOSE` (QUIC layer): closes the connection.
    ConnectionCloseQuic {
        /// QUIC transport error code.
        error_code: u64,
        /// Frame type that triggered the error (0 if unknown).
        frame_type: u64,
        /// Human-readable reason phrase.
        reason_phrase: Vec<u8>,
    },

    /// `CONNECTION_CLOSE` (application layer): closes the connection.
    ConnectionCloseApp {
        /// Application protocol error code.
        error_code: u64,
        /// Human-readable reason phrase.
        reason_phrase: Vec<u8>,
    },

    /// `HANDSHAKE_DONE`: server confirms handshake completion.
    HandshakeDone,
}

#[cfg(test)]
mod tests;
