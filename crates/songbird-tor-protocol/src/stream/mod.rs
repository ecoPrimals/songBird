// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Stream protocol - RELAY cells and stream management
//!
//! **Phase 2C**: Onion Client

mod onion_address;

pub use onion_address::OnionAddress;

use crate::error::{Error, Result};
use crate::protocol::{RelayCell, RelayCommand};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Stream state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Connecting
    Connecting,
    /// Connected and open
    Open,
    /// Closed
    Closed,
}

/// Stream for multiplexed data over a circuit
#[derive(Debug, Clone)]
pub struct Stream {
    /// Stream ID (unique per circuit)
    pub stream_id: u16,
    /// Current state
    pub state: StreamState,
    /// Circuit ID this stream belongs to
    pub circuit_id: u32,
    /// Send window (flow control)
    pub send_window: u16,
    /// Receive window (flow control)
    pub recv_window: u16,
}

impl Stream {
    /// Create new stream
    #[must_use]
    pub const fn new(stream_id: u16, circuit_id: u32) -> Self {
        Self {
            stream_id,
            state: StreamState::Connecting,
            circuit_id,
            send_window: 500, // Initial window
            recv_window: 500,
        }
    }

    /// Check if stream can send data
    #[must_use]
    pub fn can_send(&self) -> bool {
        self.state == StreamState::Open && self.send_window > 0
    }

    /// Decrease send window
    pub const fn decrease_send_window(&mut self, amount: u16) {
        self.send_window = self.send_window.saturating_sub(amount);
    }

    /// Increase send window (from SENDME)
    pub const fn increase_send_window(&mut self, amount: u16) {
        self.send_window = self.send_window.saturating_add(amount);
    }

    /// Mark stream as connected
    pub const fn mark_connected(&mut self) {
        self.state = StreamState::Open;
    }

    /// Mark stream as closed
    pub const fn mark_closed(&mut self) {
        self.state = StreamState::Closed;
    }
}

/// Stream manager for a circuit
pub struct StreamManager {
    /// Active streams
    streams: Arc<RwLock<HashMap<u16, Stream>>>,
    /// Next stream ID
    next_stream_id: Arc<RwLock<u16>>,
    /// Circuit ID
    circuit_id: u32,
}

impl StreamManager {
    /// Create new stream manager for a circuit
    #[must_use]
    pub fn new(circuit_id: u32) -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            next_stream_id: Arc::new(RwLock::new(1)),
            circuit_id,
        }
    }

    /// Allocate new stream ID
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails.
    pub fn allocate_stream(&self) -> Result<u16> {
        let mut next_id = self
            .next_stream_id
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire stream ID lock".to_string()))?;

        let stream_id = *next_id;
        *next_id = next_id.wrapping_add(1);
        drop(next_id);

        // Create stream
        let stream = Stream::new(stream_id, self.circuit_id);

        // Store stream
        let mut streams = self
            .streams
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams.insert(stream_id, stream);
        drop(streams);

        Ok(stream_id)
    }

    /// Get stream
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails or stream not found.
    pub fn get_stream(&self, stream_id: u16) -> Result<Stream> {
        let streams = self
            .streams
            .read()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams
            .get(&stream_id)
            .cloned()
            .ok_or_else(|| Error::Stream(format!("Stream {stream_id} not found")))
    }

    /// Update stream state
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails or stream not found.
    pub fn update_stream<F>(&self, stream_id: u16, f: F) -> Result<()>
    where
        F: FnOnce(&mut Stream),
    {
        let mut streams = self
            .streams
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;

        let stream = streams
            .get_mut(&stream_id)
            .ok_or_else(|| Error::Stream(format!("Stream {stream_id} not found")))?;

        f(stream);
        drop(streams);
        Ok(())
    }

    /// Remove stream
    ///
    /// # Errors
    ///
    /// Returns error if lock acquisition fails.
    pub fn remove_stream(&self, stream_id: u16) -> Result<()> {
        let mut streams = self
            .streams
            .write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams.remove(&stream_id);
        drop(streams);
        Ok(())
    }

    /// Get stream count
    #[must_use]
    pub fn stream_count(&self) -> usize {
        self.streams.read().map(|s| s.len()).unwrap_or(0)
    }
}

/// Stream protocol handler
///
/// Creates relay cells for Tor stream operations. The `digest` field
/// is a running SHA-1 hash that provides integrity checking across
/// circuit hops. When `security provider` crypto is available, pass the running
/// digest state; otherwise zeros are used (suitable for testing).
pub struct StreamProtocol;

impl StreamProtocol {
    /// Create `RELAY_BEGIN` cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID for this connection
    /// * `address` - Target address (e.g., "example.com:80")
    ///
    /// # Returns
    /// * `RelayCell` with BEGIN command
    ///
    /// # Errors
    ///
    /// Returns an error if the address payload length does not fit in `u16`.
    pub fn create_begin(stream_id: u16, address: &str) -> Result<RelayCell> {
        let mut data = address.as_bytes().to_vec();
        data.push(0); // Null terminator

        let length = u16::try_from(data.len()).map_err(|_| {
            Error::Protocol(format!(
                "RELAY_BEGIN address too long: {} bytes (max {})",
                data.len(),
                u16::MAX
            ))
        })?;

        Ok(RelayCell {
            command: RelayCommand::Begin,
            recognized: 0,
            stream_id,
            digest: [0u8; 4], // Populated by onion layer before encryption
            length,
            data,
        })
    }

    /// Create `RELAY_DATA` cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID
    /// * `data` - Data to send (max 498 bytes per cell)
    ///
    /// # Returns
    /// * `RelayCell` with DATA command
    ///
    /// # Errors
    ///
    /// Returns an error if the payload length does not fit in `u16`.
    pub fn create_data(stream_id: u16, data: &[u8]) -> Result<RelayCell> {
        let length = u16::try_from(data.len()).map_err(|_| {
            Error::Protocol(format!(
                "RELAY_DATA payload too long: {} bytes (max {})",
                data.len(),
                u16::MAX
            ))
        })?;

        Ok(RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id,
            digest: [0u8; 4], // Populated by onion layer before encryption
            length,
            data: data.to_vec(),
        })
    }

    /// Create `RELAY_END` cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID to close
    /// * `reason` - Reason code (0 = normal close)
    ///
    /// # Returns
    /// * `RelayCell` with END command
    #[must_use]
    pub fn create_end(stream_id: u16, reason: u8) -> RelayCell {
        RelayCell {
            command: RelayCommand::End,
            recognized: 0,
            stream_id,
            digest: [0u8; 4],
            length: 1,
            data: vec![reason],
        }
    }

    /// Create `RELAY_SENDME` cell (flow control)
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID (0 for circuit-level)
    ///
    /// # Returns
    /// * `RelayCell` with SENDME command
    #[must_use]
    pub const fn create_sendme(stream_id: u16) -> RelayCell {
        RelayCell {
            command: RelayCommand::SendMe,
            recognized: 0,
            stream_id,
            digest: [0u8; 4],
            length: 0,
            data: Vec::new(),
        }
    }

    /// Parse `RELAY_CONNECTED` cell
    ///
    /// # Arguments
    /// * `cell` - `RelayCell` to parse
    ///
    /// # Returns
    /// * Ok if connected, Error otherwise
    ///
    /// # Errors
    ///
    /// Returns error if cell command is not CONNECTED.
    pub fn parse_connected(cell: &RelayCell) -> Result<()> {
        if cell.command != RelayCommand::Connected {
            return Err(Error::Protocol(format!("Expected CONNECTED, got {:?}", cell.command)));
        }
        Ok(())
    }

    /// Parse `RELAY_END` cell
    ///
    /// # Arguments
    /// * `cell` - `RelayCell` to parse
    ///
    /// # Returns
    /// * Reason code for closure
    ///
    /// # Errors
    ///
    /// Returns error if cell command is not END.
    pub fn parse_end(cell: &RelayCell) -> Result<u8> {
        if cell.command != RelayCommand::End {
            return Err(Error::Protocol(format!("Expected END, got {:?}", cell.command)));
        }

        if cell.data.is_empty() {
            return Ok(0); // Normal close
        }

        Ok(cell.data[0])
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::error::Error;
    use crate::protocol::{MAX_RELAY_PAYLOAD, RelayCell, RelayCommand};

    #[test]
    fn test_stream_creation() {
        let stream = Stream::new(42, 1234);
        assert_eq!(stream.stream_id, 42);
        assert_eq!(stream.circuit_id, 1234);
        assert_eq!(stream.state, StreamState::Connecting);
        assert_eq!(stream.send_window, 500);
    }

    #[test]
    fn test_stream_flow_control() {
        let mut stream = Stream::new(1, 1);
        stream.mark_connected(); // Must be open to send

        assert!(stream.can_send());

        stream.decrease_send_window(100);
        assert_eq!(stream.send_window, 400);
        assert!(stream.can_send());

        stream.increase_send_window(50);
        assert_eq!(stream.send_window, 450);

        // Exhaust window
        stream.decrease_send_window(450);
        assert_eq!(stream.send_window, 0);
        assert!(!stream.can_send());
    }

    #[test]
    fn test_stream_state_transitions() {
        let mut stream = Stream::new(1, 1);

        assert_eq!(stream.state, StreamState::Connecting);

        stream.mark_connected();
        assert_eq!(stream.state, StreamState::Open);

        stream.mark_closed();
        assert_eq!(stream.state, StreamState::Closed);
    }

    #[test]
    fn test_stream_manager() {
        let manager = StreamManager::new(1);

        let stream_id = manager.allocate_stream().expect("Failed to allocate");
        assert_eq!(stream_id, 1);
        assert_eq!(manager.stream_count(), 1);

        let stream = manager.get_stream(stream_id).expect("Stream not found");
        assert_eq!(stream.stream_id, 1);

        manager.remove_stream(stream_id).expect("Failed to remove");
        assert_eq!(manager.stream_count(), 0);
    }

    #[test]
    fn stream_connecting_cannot_send_even_with_window() {
        let s = Stream::new(1, 1);
        assert_eq!(s.state, StreamState::Connecting);
        assert!(!s.can_send());
    }

    #[test]
    fn stream_recv_window_default_matches_send() {
        let s = Stream::new(0, 0);
        assert_eq!(s.recv_window, 500);
        assert_eq!(s.send_window, 500);
    }

    #[test]
    fn parse_connected_accepts_connected_command() {
        let cell = RelayCell {
            command: RelayCommand::Connected,
            recognized: 0,
            stream_id: 9,
            digest: [0u8; 4],
            length: 0,
            data: vec![],
        };
        StreamProtocol::parse_connected(&cell).expect("connected");
    }

    #[test]
    fn parse_connected_rejects_wrong_command() {
        let cell = RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id: 1,
            digest: [0u8; 4],
            length: 0,
            data: vec![],
        };
        let err = StreamProtocol::parse_connected(&cell).expect_err("expected protocol error");
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[test]
    fn parse_end_accepts_end_and_reason() {
        let cell = RelayCell {
            command: RelayCommand::End,
            recognized: 0,
            stream_id: 2,
            digest: [0u8; 4],
            length: 1,
            data: vec![0x07],
        };
        assert_eq!(StreamProtocol::parse_end(&cell).expect("reason"), 0x07);
    }

    #[test]
    fn parse_end_empty_payload_means_normal_close() {
        let cell = RelayCell {
            command: RelayCommand::End,
            recognized: 0,
            stream_id: 2,
            digest: [0u8; 4],
            length: 0,
            data: vec![],
        };
        assert_eq!(StreamProtocol::parse_end(&cell).expect("zero"), 0);
    }

    #[test]
    fn parse_end_rejects_non_end_command() {
        let cell = RelayCell {
            command: RelayCommand::SendMe,
            recognized: 0,
            stream_id: 0,
            digest: [0u8; 4],
            length: 0,
            data: vec![],
        };
        let err = StreamProtocol::parse_end(&cell).expect_err("wrong cmd");
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[test]
    fn stream_manager_get_missing_returns_error() {
        let manager = StreamManager::new(42);
        let err = manager.get_stream(999).expect_err("missing");
        assert!(matches!(err, Error::Stream(_)));
    }

    #[test]
    fn stream_manager_update_missing_returns_error() {
        let manager = StreamManager::new(42);
        let err = manager.update_stream(7, Stream::mark_connected).expect_err("missing stream");
        assert!(matches!(err, Error::Stream(_)));
    }

    #[test]
    fn stream_manager_allocate_sequential_stream_ids() {
        let manager = StreamManager::new(100);
        assert_eq!(manager.allocate_stream().expect("a"), 1);
        assert_eq!(manager.allocate_stream().expect("b"), 2);
        assert_eq!(manager.allocate_stream().expect("c"), 3);
        assert_eq!(manager.stream_count(), 3);
    }

    #[test]
    fn test_begin_cell() {
        let cell = StreamProtocol::create_begin(42, "example.com:80").unwrap();
        assert_eq!(cell.command, RelayCommand::Begin);
        assert_eq!(cell.stream_id, 42);
        assert!(cell.data.ends_with(&[0])); // Null terminator
    }

    #[test]
    fn test_data_cell() {
        let data = b"Hello, Tor!";
        let cell = StreamProtocol::create_data(42, data).unwrap();
        assert_eq!(cell.command, RelayCommand::Data);
        assert_eq!(cell.stream_id, 42);
        assert_eq!(cell.data, data);
    }

    #[test]
    fn test_end_cell() {
        let cell = StreamProtocol::create_end(42, 0);
        assert_eq!(cell.command, RelayCommand::End);
        assert_eq!(cell.stream_id, 42);
        assert_eq!(cell.data, vec![0]);
    }

    #[test]
    fn test_sendme_cell() {
        let cell = StreamProtocol::create_sendme(42);
        assert_eq!(cell.command, RelayCommand::SendMe);
        assert_eq!(cell.stream_id, 42);
        assert_eq!(cell.data.len(), 0);
    }

    #[test]
    fn create_begin_empty_address_still_has_null_terminator() {
        let cell = StreamProtocol::create_begin(3, "").expect("begin");
        assert_eq!(cell.command, RelayCommand::Begin);
        assert_eq!(cell.data, vec![0u8]);
        assert_eq!(cell.length, 1);
    }

    #[test]
    fn create_begin_rejects_when_address_exceeds_u16_max_with_null() {
        let addr = vec![b'a'; usize::from(u16::MAX)];
        let s = String::from_utf8(addr).expect("utf8");
        let err = StreamProtocol::create_begin(1, &s).expect_err("too long");
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[test]
    fn create_data_accepts_max_relay_payload() {
        let payload = vec![0x7Fu8; MAX_RELAY_PAYLOAD];
        let cell = StreamProtocol::create_data(9, &payload).expect("max data");
        assert_eq!(cell.length as usize, payload.len());
        assert_eq!(cell.data.len(), MAX_RELAY_PAYLOAD);
    }

    #[test]
    fn create_data_rejects_payload_when_len_exceeds_u16() {
        // `create_data` bounds length by `u16` (Tor relay data length field); it does not
        // additionally clamp to `MAX_RELAY_PAYLOAD` — exercise the `u16` guard.
        let payload = vec![0u8; usize::from(u16::MAX) + 1];
        let err = StreamProtocol::create_data(1, &payload).expect_err("overflow");
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[test]
    fn relay_cell_encode_roundtrip_by_manual_parse() {
        let original = RelayCell {
            command: RelayCommand::Connected,
            recognized: 0x0102,
            stream_id: 0x0304,
            digest: [0x0A, 0x0B, 0x0C, 0x0D],
            length: 2,
            data: vec![0xEE, 0xFF],
        };
        let wire = original.encode();
        assert_eq!(wire[0], RelayCommand::Connected as u8);
        assert_eq!(&wire[1..3], &[0x01, 0x02]);
        assert_eq!(&wire[3..5], &[0x03, 0x04]);
        assert_eq!(&wire[5..9], &[0x0A, 0x0B, 0x0C, 0x0D]);
        assert_eq!(u16::from_be_bytes([wire[9], wire[10]]), 2);
        assert_eq!(&wire[11..], &[0xEE, 0xFF]);
    }

    #[test]
    fn stream_manager_remove_unknown_stream_succeeds_noop() {
        let manager = StreamManager::new(7);
        manager.remove_stream(999).expect("remove missing");
        assert_eq!(manager.stream_count(), 0);
    }

    #[test]
    fn stream_manager_update_stream_applies_closure() {
        let manager = StreamManager::new(0xCAFE);
        let sid = manager.allocate_stream().expect("alloc");
        manager.update_stream(sid, Stream::mark_connected).expect("update");
        let stream = manager.get_stream(sid).expect("get");
        assert_eq!(stream.state, StreamState::Open);
        assert!(stream.can_send());
    }
}
