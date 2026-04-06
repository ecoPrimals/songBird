// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC Transport Parameters (RFC 9000 Section 18).
//!
//! Carried in the TLS handshake as extension type 0x39 (57).
//! Each parameter is encoded as a (`VarInt` id, `VarInt` length, value) tuple.

use crate::error::{QuicError, Result};
use crate::varint::VarInt;
use std::time::Duration;

/// QUIC Transport Parameters extension type in TLS (RFC 9000 Section 7.4).
pub const TRANSPORT_PARAMS_EXTENSION_TYPE: u16 = 0x0039;

/// Transport parameter IDs (RFC 9000 Section 18.2).
pub mod param_id {
    /// `original_destination_connection_id`.
    pub const ORIGINAL_DESTINATION_CONNECTION_ID: u64 = 0x00;
    /// `max_idle_timeout`.
    pub const MAX_IDLE_TIMEOUT: u64 = 0x01;
    /// `stateless_reset_token`.
    pub const STATELESS_RESET_TOKEN: u64 = 0x02;
    /// `max_udp_payload_size`.
    pub const MAX_UDP_PAYLOAD_SIZE: u64 = 0x03;
    /// `initial_max_data`.
    pub const INITIAL_MAX_DATA: u64 = 0x04;
    /// `initial_max_stream_data_bidi_local`.
    pub const INITIAL_MAX_STREAM_DATA_BIDI_LOCAL: u64 = 0x05;
    /// `initial_max_stream_data_bidi_remote`.
    pub const INITIAL_MAX_STREAM_DATA_BIDI_REMOTE: u64 = 0x06;
    /// `initial_max_stream_data_uni`.
    pub const INITIAL_MAX_STREAM_DATA_UNI: u64 = 0x07;
    /// `initial_max_streams_bidi`.
    pub const INITIAL_MAX_STREAMS_BIDI: u64 = 0x08;
    /// `initial_max_streams_uni`.
    pub const INITIAL_MAX_STREAMS_UNI: u64 = 0x09;
    /// `ack_delay_exponent`.
    pub const ACK_DELAY_EXPONENT: u64 = 0x0A;
    /// `max_ack_delay`.
    pub const MAX_ACK_DELAY: u64 = 0x0B;
    /// `disable_active_migration`.
    pub const DISABLE_ACTIVE_MIGRATION: u64 = 0x0C;
    /// `preferred_address`.
    pub const PREFERRED_ADDRESS: u64 = 0x0D;
    /// `active_connection_id_limit`.
    pub const ACTIVE_CONNECTION_ID_LIMIT: u64 = 0x0E;
    /// `initial_source_connection_id`.
    pub const INITIAL_SOURCE_CONNECTION_ID: u64 = 0x0F;
    /// `retry_source_connection_id`.
    pub const RETRY_SOURCE_CONNECTION_ID: u64 = 0x10;
}

/// QUIC transport parameters (RFC 9000 Section 18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportParams {
    /// Original Destination Connection ID (server only, from Initial packet).
    pub original_dcid: Option<Vec<u8>>,
    /// Maximum idle timeout in milliseconds (0 = disabled).
    pub max_idle_timeout: u64,
    /// Stateless reset token (server only, 16 bytes).
    pub stateless_reset_token: Option<[u8; 16]>,
    /// Maximum UDP payload size (default 65527).
    pub max_udp_payload_size: u64,
    /// Initial maximum data the peer may send on the connection.
    pub initial_max_data: u64,
    /// Initial max data on locally-initiated bidirectional streams.
    pub initial_max_stream_data_bidi_local: u64,
    /// Initial max data on remotely-initiated bidirectional streams.
    pub initial_max_stream_data_bidi_remote: u64,
    /// Initial max data on unidirectional streams.
    pub initial_max_stream_data_uni: u64,
    /// Maximum number of bidirectional streams the peer may initiate.
    pub initial_max_streams_bidi: u64,
    /// Maximum number of unidirectional streams the peer may initiate.
    pub initial_max_streams_uni: u64,
    /// ACK delay exponent (default 3, max 20).
    pub ack_delay_exponent: u64,
    /// Maximum ACK delay in milliseconds (default 25).
    pub max_ack_delay: u64,
    /// Disable active connection migration.
    pub disable_active_migration: bool,
    /// Active connection ID limit (default 2).
    pub active_connection_id_limit: u64,
    /// Initial Source Connection ID.
    pub initial_source_cid: Option<Vec<u8>>,
    /// Retry Source Connection ID (server only).
    pub retry_source_cid: Option<Vec<u8>>,
}

impl Default for TransportParams {
    fn default() -> Self {
        Self {
            original_dcid: None,
            max_idle_timeout: 0,
            stateless_reset_token: None,
            max_udp_payload_size: 65527,
            initial_max_data: 0,
            initial_max_stream_data_bidi_local: 0,
            initial_max_stream_data_bidi_remote: 0,
            initial_max_stream_data_uni: 0,
            initial_max_streams_bidi: 0,
            initial_max_streams_uni: 0,
            ack_delay_exponent: 3,
            max_ack_delay: 25,
            disable_active_migration: false,
            active_connection_id_limit: 2,
            initial_source_cid: None,
            retry_source_cid: None,
        }
    }
}

impl TransportParams {
    /// Create default parameters appropriate for a Songbird QUIC endpoint.
    #[must_use]
    pub fn songbird_defaults() -> Self {
        Self {
            max_idle_timeout: 30_000,
            max_udp_payload_size: 1200,
            initial_max_data: 1_048_576,                 // 1 MiB
            initial_max_stream_data_bidi_local: 262_144, // 256 KiB
            initial_max_stream_data_bidi_remote: 262_144,
            initial_max_stream_data_uni: 262_144,
            initial_max_streams_bidi: 100,
            initial_max_streams_uni: 100,
            active_connection_id_limit: 4,
            ..Self::default()
        }
    }

    /// Set max idle timeout from a Duration.
    #[must_use]
    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Idle timeout in ms fits u64 for practical QUIC use."
        )]
        let ms = timeout.as_millis() as u64;
        self.max_idle_timeout = ms;
        self
    }

    /// Encode transport parameters into wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if a value cannot be encoded as a [`VarInt`].
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(128);

        if let Some(ref dcid) = self.original_dcid {
            encode_param_bytes(&mut buf, param_id::ORIGINAL_DESTINATION_CONNECTION_ID, dcid)?;
        }
        if self.max_idle_timeout > 0 {
            encode_param_varint(&mut buf, param_id::MAX_IDLE_TIMEOUT, self.max_idle_timeout)?;
        }
        if let Some(ref token) = self.stateless_reset_token {
            encode_param_bytes(&mut buf, param_id::STATELESS_RESET_TOKEN, token)?;
        }
        encode_param_varint(&mut buf, param_id::MAX_UDP_PAYLOAD_SIZE, self.max_udp_payload_size)?;
        encode_param_varint(&mut buf, param_id::INITIAL_MAX_DATA, self.initial_max_data)?;
        encode_param_varint(
            &mut buf,
            param_id::INITIAL_MAX_STREAM_DATA_BIDI_LOCAL,
            self.initial_max_stream_data_bidi_local,
        )?;
        encode_param_varint(
            &mut buf,
            param_id::INITIAL_MAX_STREAM_DATA_BIDI_REMOTE,
            self.initial_max_stream_data_bidi_remote,
        )?;
        encode_param_varint(
            &mut buf,
            param_id::INITIAL_MAX_STREAM_DATA_UNI,
            self.initial_max_stream_data_uni,
        )?;
        encode_param_varint(
            &mut buf,
            param_id::INITIAL_MAX_STREAMS_BIDI,
            self.initial_max_streams_bidi,
        )?;
        encode_param_varint(
            &mut buf,
            param_id::INITIAL_MAX_STREAMS_UNI,
            self.initial_max_streams_uni,
        )?;
        encode_param_varint(&mut buf, param_id::ACK_DELAY_EXPONENT, self.ack_delay_exponent)?;
        encode_param_varint(&mut buf, param_id::MAX_ACK_DELAY, self.max_ack_delay)?;
        if self.disable_active_migration {
            encode_param_empty(&mut buf, param_id::DISABLE_ACTIVE_MIGRATION)?;
        }
        encode_param_varint(
            &mut buf,
            param_id::ACTIVE_CONNECTION_ID_LIMIT,
            self.active_connection_id_limit,
        )?;
        if let Some(ref cid) = self.initial_source_cid {
            encode_param_bytes(&mut buf, param_id::INITIAL_SOURCE_CONNECTION_ID, cid)?;
        }
        if let Some(ref cid) = self.retry_source_cid {
            encode_param_bytes(&mut buf, param_id::RETRY_SOURCE_CONNECTION_ID, cid)?;
        }

        Ok(buf)
    }

    /// Decode transport parameters from wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is truncated, varints are invalid, or a parameter value is ill-formed.
    pub fn decode(mut data: &[u8]) -> Result<Self> {
        let mut params = Self::default();

        while !data.is_empty() {
            let (id, consumed) = VarInt::decode(data)?;
            data = &data[consumed..];
            let (length, consumed) = VarInt::decode(data)?;
            data = &data[consumed..];
            #[expect(
                clippy::cast_possible_truncation,
                reason = "QUIC transport parameter lengths are bounded by wire format."
            )]
            let len = length.value() as usize;
            if data.len() < len {
                return Err(QuicError::Config("Transport param value truncated".into()));
            }
            let value = &data[..len];
            data = &data[len..];

            match id.value() {
                param_id::ORIGINAL_DESTINATION_CONNECTION_ID => {
                    params.original_dcid = Some(value.to_vec());
                }
                param_id::MAX_IDLE_TIMEOUT => {
                    params.max_idle_timeout = decode_varint_param(value)?;
                }
                param_id::STATELESS_RESET_TOKEN => {
                    if value.len() != 16 {
                        return Err(QuicError::Config(
                            "Stateless reset token must be 16 bytes".into(),
                        ));
                    }
                    let mut token = [0u8; 16];
                    token.copy_from_slice(value);
                    params.stateless_reset_token = Some(token);
                }
                param_id::MAX_UDP_PAYLOAD_SIZE => {
                    params.max_udp_payload_size = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_DATA => {
                    params.initial_max_data = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_STREAM_DATA_BIDI_LOCAL => {
                    params.initial_max_stream_data_bidi_local = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_STREAM_DATA_BIDI_REMOTE => {
                    params.initial_max_stream_data_bidi_remote = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_STREAM_DATA_UNI => {
                    params.initial_max_stream_data_uni = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_STREAMS_BIDI => {
                    params.initial_max_streams_bidi = decode_varint_param(value)?;
                }
                param_id::INITIAL_MAX_STREAMS_UNI => {
                    params.initial_max_streams_uni = decode_varint_param(value)?;
                }
                param_id::ACK_DELAY_EXPONENT => {
                    params.ack_delay_exponent = decode_varint_param(value)?;
                }
                param_id::MAX_ACK_DELAY => {
                    params.max_ack_delay = decode_varint_param(value)?;
                }
                param_id::DISABLE_ACTIVE_MIGRATION => {
                    params.disable_active_migration = true;
                }
                param_id::ACTIVE_CONNECTION_ID_LIMIT => {
                    params.active_connection_id_limit = decode_varint_param(value)?;
                }
                param_id::INITIAL_SOURCE_CONNECTION_ID => {
                    params.initial_source_cid = Some(value.to_vec());
                }
                param_id::RETRY_SOURCE_CONNECTION_ID => {
                    params.retry_source_cid = Some(value.to_vec());
                }
                _ => {
                    // Unknown parameters are ignored per RFC 9000.
                }
            }
        }

        Ok(params)
    }
}

fn encode_param_varint(buf: &mut Vec<u8>, id: u64, value: u64) -> Result<()> {
    let mut tmp = [0u8; 16];
    let n = VarInt::new(id)?.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    let vi = VarInt::new(value)?;
    let value_len = vi.encoded_len();
    let n = VarInt::new(value_len as u64)?.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    let n = vi.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    Ok(())
}

fn encode_param_bytes(buf: &mut Vec<u8>, id: u64, value: &[u8]) -> Result<()> {
    let mut tmp = [0u8; 8];
    let n = VarInt::new(id)?.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    let n = VarInt::new(value.len() as u64)?.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    buf.extend_from_slice(value);
    Ok(())
}

fn encode_param_empty(buf: &mut Vec<u8>, id: u64) -> Result<()> {
    let mut tmp = [0u8; 8];
    let n = VarInt::new(id)?.encode(&mut tmp)?;
    buf.extend_from_slice(&tmp[..n]);
    buf.push(0); // length = 0
    Ok(())
}

fn decode_varint_param(data: &[u8]) -> Result<u64> {
    if data.is_empty() {
        return Ok(0);
    }
    let (vi, _) = VarInt::decode(data)?;
    Ok(vi.value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params() {
        let p = TransportParams::default();
        assert_eq!(p.max_idle_timeout, 0);
        assert_eq!(p.max_udp_payload_size, 65527);
        assert_eq!(p.ack_delay_exponent, 3);
        assert_eq!(p.max_ack_delay, 25);
        assert_eq!(p.active_connection_id_limit, 2);
    }

    #[test]
    fn songbird_defaults() {
        let p = TransportParams::songbird_defaults();
        assert_eq!(p.max_idle_timeout, 30_000);
        assert_eq!(p.initial_max_data, 1_048_576);
        assert_eq!(p.initial_max_streams_bidi, 100);
        assert_eq!(p.active_connection_id_limit, 4);
    }

    #[test]
    fn roundtrip_defaults() {
        let params = TransportParams::songbird_defaults();
        let encoded = params.encode().unwrap();
        let decoded = TransportParams::decode(&encoded).unwrap();
        assert_eq!(decoded.max_idle_timeout, params.max_idle_timeout);
        assert_eq!(decoded.initial_max_data, params.initial_max_data);
        assert_eq!(decoded.initial_max_streams_bidi, params.initial_max_streams_bidi);
        assert_eq!(decoded.initial_max_streams_uni, params.initial_max_streams_uni);
        assert_eq!(decoded.active_connection_id_limit, params.active_connection_id_limit);
    }

    #[test]
    fn roundtrip_with_cids() {
        let params = TransportParams {
            original_dcid: Some(vec![0x01, 0x02, 0x03]),
            initial_source_cid: Some(vec![0x04, 0x05]),
            retry_source_cid: Some(vec![0x06]),
            ..TransportParams::songbird_defaults()
        };
        let encoded = params.encode().unwrap();
        let decoded = TransportParams::decode(&encoded).unwrap();
        assert_eq!(decoded.original_dcid, params.original_dcid);
        assert_eq!(decoded.initial_source_cid, params.initial_source_cid);
        assert_eq!(decoded.retry_source_cid, params.retry_source_cid);
    }

    #[test]
    fn roundtrip_disable_migration() {
        let params = TransportParams {
            disable_active_migration: true,
            ..TransportParams::default()
        };
        let encoded = params.encode().unwrap();
        let decoded = TransportParams::decode(&encoded).unwrap();
        assert!(decoded.disable_active_migration);
    }

    #[test]
    fn roundtrip_stateless_reset_token() {
        let params = TransportParams {
            stateless_reset_token: Some([0xAA; 16]),
            ..TransportParams::default()
        };
        let encoded = params.encode().unwrap();
        let decoded = TransportParams::decode(&encoded).unwrap();
        assert_eq!(decoded.stateless_reset_token, Some([0xAA; 16]));
    }

    #[test]
    fn decode_unknown_params_ignored() {
        let mut buf = Vec::new();
        // Unknown param ID 0xFF with 3 bytes of data
        let mut tmp = [0u8; 8];
        let n = VarInt::from_u32(0xFF).encode(&mut tmp).unwrap();
        buf.extend_from_slice(&tmp[..n]);
        let n = VarInt::from_u32(3).encode(&mut tmp).unwrap();
        buf.extend_from_slice(&tmp[..n]);
        buf.extend_from_slice(&[0x01, 0x02, 0x03]);

        let decoded = TransportParams::decode(&buf).unwrap();
        assert_eq!(decoded, TransportParams::default());
    }

    #[test]
    fn empty_buffer_returns_defaults() {
        let decoded = TransportParams::decode(&[]).unwrap();
        assert_eq!(decoded, TransportParams::default());
    }

    #[test]
    fn with_idle_timeout_builder() {
        let p = TransportParams::default().with_idle_timeout(Duration::from_secs(60));
        assert_eq!(p.max_idle_timeout, 60_000);
    }
}
