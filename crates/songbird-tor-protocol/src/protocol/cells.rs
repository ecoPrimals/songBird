// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tor cell encoding/decoding
//!
//! **Status**: Phase 2B — cell types implemented, encryption pending `security provider` AES-128-CTR

use crate::error::{Error, Result};

/// Fixed cell size (512 bytes)
pub const CELL_LEN: usize = 512;

/// Cell command types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellCommand {
    /// Padding
    Padding = 0,
    /// Create circuit
    Create = 1,
    /// Circuit created
    Created = 2,
    /// Relay cell
    Relay = 3,
    /// Destroy circuit
    Destroy = 4,
    /// Create circuit (fast)
    CreateFast = 5,
    /// Circuit created (fast)
    CreatedFast = 6,
    /// Versions negotiation
    Versions = 7,
    /// Network info
    NetInfo = 8,
    /// Relay cell (early)
    RelayEarly = 9,
    /// Create circuit (v2, ntor)
    Create2 = 10,
    /// Circuit created (v2)
    Created2 = 11,
}

/// Tor cell (512 bytes fixed)
#[derive(Debug, Clone)]
pub struct Cell {
    /// Circuit ID (4 bytes)
    pub circ_id: u32,
    /// Command type
    pub command: CellCommand,
    /// Payload (507 bytes max)
    pub payload: Vec<u8>,
}

impl Cell {
    /// Encode cell to bytes
    #[must_use]
    pub fn encode(&self) -> [u8; CELL_LEN] {
        let mut buf = [0u8; CELL_LEN];
        buf[0..4].copy_from_slice(&self.circ_id.to_be_bytes());
        buf[4] = self.command as u8;

        let payload_len = self.payload.len().min(507);
        buf[5..5 + payload_len].copy_from_slice(&self.payload[..payload_len]);

        buf
    }

    /// Decode cell from bytes
    ///
    /// # Errors
    ///
    /// Returns error if command byte is invalid.
    pub fn decode(data: &[u8; CELL_LEN]) -> Result<Self> {
        let circ_id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let command = CellCommand::try_from(data[4])?;
        let payload = data[5..].to_vec();

        Ok(Self {
            circ_id,
            command,
            payload,
        })
    }
}

impl TryFrom<u8> for CellCommand {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Padding),
            1 => Ok(Self::Create),
            2 => Ok(Self::Created),
            3 => Ok(Self::Relay),
            4 => Ok(Self::Destroy),
            5 => Ok(Self::CreateFast),
            6 => Ok(Self::CreatedFast),
            7 => Ok(Self::Versions),
            8 => Ok(Self::NetInfo),
            9 => Ok(Self::RelayEarly),
            10 => Ok(Self::Create2),
            11 => Ok(Self::Created2),
            _ => Err(Error::Protocol(format!("Unknown cell command: {value}"))),
        }
    }
}

/// Relay command types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayCommand {
    /// Begin stream
    Begin = 1,
    /// Data
    Data = 2,
    /// End stream
    End = 3,
    /// Stream connected
    Connected = 4,
    /// Send more
    SendMe = 5,
    /// Extend circuit
    Extend = 6,
    /// Circuit extended
    Extended = 7,
    /// Introduce1
    Introduce1 = 32,
    /// Introduce2
    Introduce2 = 33,
    /// Rendezvous1
    Rendezvous1 = 34,
    /// Rendezvous2
    Rendezvous2 = 35,
}

/// Relay cell (inside RELAY cell payload)
#[derive(Debug, Clone)]
pub struct RelayCell {
    /// Relay command
    pub command: RelayCommand,
    /// Recognized (always 0 for valid)
    pub recognized: u16,
    /// Stream ID
    pub stream_id: u16,
    /// Digest (4 bytes)
    pub digest: [u8; 4],
    /// Payload length
    pub length: u16,
    /// Data
    pub data: Vec<u8>,
}

impl RelayCell {
    /// Encode relay cell
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(509);
        buf.push(self.command as u8);
        buf.extend_from_slice(&self.recognized.to_be_bytes());
        buf.extend_from_slice(&self.stream_id.to_be_bytes());
        buf.extend_from_slice(&self.digest);
        buf.extend_from_slice(&self.length.to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::Error;
    use crate::protocol::MAX_RELAY_PAYLOAD;
    use crate::protocol::constants::MAX_CELL_PAYLOAD;

    #[test]
    fn cell_roundtrip_relay_command() {
        let original = Cell {
            circ_id: 0xdeadbeef,
            command: CellCommand::Relay,
            payload: vec![1, 2, 3, 4, 5],
        };
        let encoded = original.encode();
        let decoded = Cell::decode(&encoded).expect("decode");
        assert_eq!(decoded.circ_id, original.circ_id);
        assert_eq!(decoded.command, CellCommand::Relay);
        assert_eq!(decoded.payload[..5], [1, 2, 3, 4, 5]);
    }

    #[test]
    fn cell_roundtrip_all_command_bytes() {
        for cmd in [
            CellCommand::Padding,
            CellCommand::Create,
            CellCommand::Created,
            CellCommand::Relay,
            CellCommand::Destroy,
            CellCommand::CreateFast,
            CellCommand::CreatedFast,
            CellCommand::Versions,
            CellCommand::NetInfo,
            CellCommand::RelayEarly,
            CellCommand::Create2,
            CellCommand::Created2,
        ] {
            let cell = Cell {
                circ_id: 42,
                command: cmd,
                payload: vec![],
            };
            let out = cell.encode();
            let back = Cell::decode(&out).expect("decode");
            assert_eq!(back.command, cmd, "roundtrip cmd {cmd:?}");
        }
    }

    #[test]
    fn cell_decode_rejects_unknown_command() {
        let mut buf = [0u8; CELL_LEN];
        buf[4] = 99;
        let err = Cell::decode(&buf).expect_err("unknown command");
        match err {
            Error::Protocol(s) => assert!(s.contains("Unknown cell command")),
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }

    #[test]
    fn cell_encode_truncates_payload_to_max_cell_payload() {
        let big = vec![0xabu8; MAX_CELL_PAYLOAD + 100];
        let cell = Cell {
            circ_id: 1,
            command: CellCommand::Relay,
            payload: big,
        };
        let enc = cell.encode();
        assert_eq!(enc[5..].len(), MAX_CELL_PAYLOAD);
        assert!(enc[5..].iter().all(|&b| b == 0xab));
    }

    #[test]
    fn cell_decode_restores_full_sized_payload_slice() {
        let mut payload = vec![0u8; MAX_CELL_PAYLOAD];
        for (i, b) in payload.iter_mut().enumerate() {
            *b = (i % 256) as u8;
        }
        let cell = Cell {
            circ_id: 0xcafe_babe,
            command: CellCommand::RelayEarly,
            payload: payload.clone(),
        };
        let enc = cell.encode();
        let decoded = Cell::decode(&enc).expect("decode");
        assert_eq!(decoded.payload.len(), MAX_CELL_PAYLOAD);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn relay_cell_encode_wire_layout() {
        let rc = RelayCell {
            command: RelayCommand::Extend,
            recognized: 0,
            stream_id: 0x0102,
            digest: [0xca, 0xfe, 0xba, 0xbe],
            length: 4,
            data: vec![1, 2, 3, 4],
        };
        let b = rc.encode();
        assert_eq!(b[0], RelayCommand::Extend as u8);
        assert_eq!(&b[1..3], &[0, 0]); // recognized BE
        assert_eq!(&b[3..5], &[0x01, 0x02]); // stream_id BE
        assert_eq!(&b[5..9], &[0xca, 0xfe, 0xba, 0xbe]);
        assert_eq!(&b[9..11], &[0, 4]); // length BE
        assert_eq!(&b[11..], &[1, 2, 3, 4]);
    }

    #[test]
    fn relay_cell_encode_empty_data() {
        let rc = RelayCell {
            command: RelayCommand::End,
            recognized: 0,
            stream_id: 0,
            digest: [0; 4],
            length: 0,
            data: vec![],
        };
        assert_eq!(rc.encode().len(), 11);
    }

    #[test]
    fn relay_command_discriminants_match_tor_wire_values() {
        assert_eq!(RelayCommand::Begin as u8, 1);
        assert_eq!(RelayCommand::Introduce1 as u8, 32);
        assert_eq!(RelayCommand::Rendezvous2 as u8, 35);
    }

    #[test]
    fn cell_command_try_from_maps_all_variants() {
        for (byte, expected) in [
            (0u8, CellCommand::Padding),
            (1, CellCommand::Create),
            (2, CellCommand::Created),
            (3, CellCommand::Relay),
            (4, CellCommand::Destroy),
            (5, CellCommand::CreateFast),
            (6, CellCommand::CreatedFast),
            (7, CellCommand::Versions),
            (8, CellCommand::NetInfo),
            (9, CellCommand::RelayEarly),
            (10, CellCommand::Create2),
            (11, CellCommand::Created2),
        ] {
            assert_eq!(CellCommand::try_from(byte).expect("ok"), expected);
        }
    }

    #[test]
    fn destroy_cell_roundtrip_preserves_reason_byte() {
        let cell = Cell {
            circ_id: 0x8000_0042,
            command: CellCommand::Destroy,
            payload: vec![0x03],
        };
        let enc = cell.encode();
        let dec = Cell::decode(&enc).expect("decode destroy");
        assert_eq!(dec.command, CellCommand::Destroy);
        assert_eq!(dec.circ_id, cell.circ_id);
        assert_eq!(dec.payload[0], 0x03);
    }

    #[test]
    fn cell_encode_empty_payload_yields_zeroed_tail() {
        let cell = Cell {
            circ_id: 0,
            command: CellCommand::Padding,
            payload: vec![],
        };
        let enc = cell.encode();
        assert!(enc[5..].iter().all(|&b| b == 0));
        let dec = Cell::decode(&enc).expect("decode padding");
        assert_eq!(dec.payload.len(), MAX_CELL_PAYLOAD);
    }

    #[test]
    fn relay_cell_length_field_matches_serialized_data_len() {
        let data = vec![0xCCu8; MAX_RELAY_PAYLOAD];
        let rc = RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id: 0xFFFF,
            digest: [0u8; 4],
            length: MAX_RELAY_PAYLOAD as u16,
            data: data.clone(),
        };
        let w = rc.encode();
        assert_eq!(w.len(), 11 + data.len());
        assert_eq!(u16::from_be_bytes([w[9], w[10]]) as usize, data.len());
        assert_eq!(&w[11..], data.as_slice());
    }
}
