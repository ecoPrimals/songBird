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
    pub fn new(stream_id: u16, circuit_id: u32) -> Self {
        Self {
            stream_id,
            state: StreamState::Connecting,
            circuit_id,
            send_window: 500, // Initial window
            recv_window: 500,
        }
    }

    /// Check if stream can send data
    pub fn can_send(&self) -> bool {
        self.state == StreamState::Open && self.send_window > 0
    }

    /// Decrease send window
    pub fn decrease_send_window(&mut self, amount: u16) {
        self.send_window = self.send_window.saturating_sub(amount);
    }

    /// Increase send window (from SENDME)
    pub fn increase_send_window(&mut self, amount: u16) {
        self.send_window = self.send_window.saturating_add(amount);
    }

    /// Mark stream as connected
    pub fn mark_connected(&mut self) {
        self.state = StreamState::Open;
    }

    /// Mark stream as closed
    pub fn mark_closed(&mut self) {
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
    pub fn new(circuit_id: u32) -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            next_stream_id: Arc::new(RwLock::new(1)),
            circuit_id,
        }
    }

    /// Allocate new stream ID
    pub fn allocate_stream(&self) -> Result<u16> {
        let mut next_id = self.next_stream_id.write()
            .map_err(|_| Error::Protocol("Failed to acquire stream ID lock".to_string()))?;
        
        let stream_id = *next_id;
        *next_id = next_id.wrapping_add(1);
        
        // Create stream
        let stream = Stream::new(stream_id, self.circuit_id);
        
        // Store stream
        let mut streams = self.streams.write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams.insert(stream_id, stream);
        
        Ok(stream_id)
    }

    /// Get stream
    pub fn get_stream(&self, stream_id: u16) -> Result<Stream> {
        let streams = self.streams.read()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams.get(&stream_id)
            .cloned()
            .ok_or_else(|| Error::Stream(format!("Stream {} not found", stream_id)))
    }

    /// Update stream state
    pub fn update_stream<F>(&self, stream_id: u16, f: F) -> Result<()>
    where
        F: FnOnce(&mut Stream),
    {
        let mut streams = self.streams.write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        
        let stream = streams.get_mut(&stream_id)
            .ok_or_else(|| Error::Stream(format!("Stream {} not found", stream_id)))?;
        
        f(stream);
        Ok(())
    }

    /// Remove stream
    pub fn remove_stream(&self, stream_id: u16) -> Result<()> {
        let mut streams = self.streams.write()
            .map_err(|_| Error::Protocol("Failed to acquire streams lock".to_string()))?;
        streams.remove(&stream_id);
        Ok(())
    }

    /// Get stream count
    pub fn stream_count(&self) -> usize {
        self.streams.read()
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

/// Stream protocol handler
pub struct StreamProtocol;

impl StreamProtocol {
    /// Create RELAY_BEGIN cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID for this connection
    /// * `address` - Target address (e.g., "example.com:80")
    ///
    /// # Returns
    /// * RelayCell with BEGIN command
    pub fn create_begin(stream_id: u16, address: &str) -> RelayCell {
        let mut data = address.as_bytes().to_vec();
        data.push(0); // Null terminator

        RelayCell {
            command: RelayCommand::Begin,
            recognized: 0,
            stream_id,
            digest: [0u8; 4], // TODO: Calculate digest
            length: data.len() as u16,
            data,
        }
    }

    /// Create RELAY_DATA cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID
    /// * `data` - Data to send (max 498 bytes per cell)
    ///
    /// # Returns
    /// * RelayCell with DATA command
    pub fn create_data(stream_id: u16, data: &[u8]) -> RelayCell {
        RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id,
            digest: [0u8; 4], // TODO: Calculate digest
            length: data.len() as u16,
            data: data.to_vec(),
        }
    }

    /// Create RELAY_END cell
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID to close
    /// * `reason` - Reason code (0 = normal close)
    ///
    /// # Returns
    /// * RelayCell with END command
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

    /// Create RELAY_SENDME cell (flow control)
    ///
    /// # Arguments
    /// * `stream_id` - Stream ID (0 for circuit-level)
    ///
    /// # Returns
    /// * RelayCell with SENDME command
    pub fn create_sendme(stream_id: u16) -> RelayCell {
        RelayCell {
            command: RelayCommand::SendMe,
            recognized: 0,
            stream_id,
            digest: [0u8; 4],
            length: 0,
            data: Vec::new(),
        }
    }

    /// Parse RELAY_CONNECTED cell
    ///
    /// # Arguments
    /// * `cell` - RelayCell to parse
    ///
    /// # Returns
    /// * Ok if connected, Error otherwise
    pub fn parse_connected(cell: &RelayCell) -> Result<()> {
        if cell.command != RelayCommand::Connected {
            return Err(Error::Protocol(format!(
                "Expected CONNECTED, got {:?}",
                cell.command
            )));
        }
        Ok(())
    }

    /// Parse RELAY_END cell
    ///
    /// # Arguments
    /// * `cell` - RelayCell to parse
    ///
    /// # Returns
    /// * Reason code for closure
    pub fn parse_end(cell: &RelayCell) -> Result<u8> {
        if cell.command != RelayCommand::End {
            return Err(Error::Protocol(format!(
                "Expected END, got {:?}",
                cell.command
            )));
        }
        
        if cell.data.is_empty() {
            return Ok(0); // Normal close
        }
        
        Ok(cell.data[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_begin_cell() {
        let cell = StreamProtocol::create_begin(42, "example.com:80");
        assert_eq!(cell.command, RelayCommand::Begin);
        assert_eq!(cell.stream_id, 42);
        assert!(cell.data.ends_with(&[0])); // Null terminator
    }

    #[test]
    fn test_data_cell() {
        let data = b"Hello, Tor!";
        let cell = StreamProtocol::create_data(42, data);
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
}
