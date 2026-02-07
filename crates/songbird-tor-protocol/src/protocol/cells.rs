//! Tor cell encoding/decoding
//!
//! **Status**: Phase 2B (TODO)

use crate::error::{Result, Error};

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
    pub fn encode(&self) -> [u8; CELL_LEN] {
        let mut buf = [0u8; CELL_LEN];
        buf[0..4].copy_from_slice(&self.circ_id.to_be_bytes());
        buf[4] = self.command as u8;
        
        let payload_len = self.payload.len().min(507);
        buf[5..5 + payload_len].copy_from_slice(&self.payload[..payload_len]);
        
        buf
    }
    
    /// Decode cell from bytes
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
            0 => Ok(CellCommand::Padding),
            1 => Ok(CellCommand::Create),
            2 => Ok(CellCommand::Created),
            3 => Ok(CellCommand::Relay),
            4 => Ok(CellCommand::Destroy),
            5 => Ok(CellCommand::CreateFast),
            6 => Ok(CellCommand::CreatedFast),
            7 => Ok(CellCommand::Versions),
            8 => Ok(CellCommand::NetInfo),
            9 => Ok(CellCommand::RelayEarly),
            10 => Ok(CellCommand::Create2),
            11 => Ok(CellCommand::Created2),
            _ => Err(Error::Protocol(format!("Unknown cell command: {}", value))),
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
