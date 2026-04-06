// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC stream multiplexing (RFC 9000 Sections 2-3).
//!
//! Stream IDs encode the initiator and directionality:
//! - Bit 0: 0=client-initiated, 1=server-initiated
//! - Bit 1: 0=bidirectional, 1=unidirectional
//!
//! Streams are created lazily and managed by the `StreamManager`.

use crate::error::{QuicError, Result};
use std::collections::{BTreeMap, VecDeque};

/// Stream directionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    /// Bidirectional stream.
    Bidi,
    /// Unidirectional stream.
    Uni,
}

/// Stream initiator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Initiator {
    /// Client-initiated.
    Client,
    /// Server-initiated.
    Server,
}

/// Extract the stream type from a stream ID.
#[must_use]
pub const fn stream_type(id: u64) -> StreamType {
    if id & 0x02 == 0 {
        StreamType::Bidi
    } else {
        StreamType::Uni
    }
}

/// Extract the initiator from a stream ID.
#[must_use]
pub const fn stream_initiator(id: u64) -> Initiator {
    if id & 0x01 == 0 {
        Initiator::Client
    } else {
        Initiator::Server
    }
}

/// Stream state (simplified).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Stream is open for data.
    Open,
    /// Send side has been finished (FIN sent).
    SendFinished,
    /// Receive side has received FIN.
    RecvFinished,
    /// Both sides finished.
    Closed,
    /// Stream was reset.
    Reset,
}

/// A single QUIC stream's state and buffers.
#[derive(Debug)]
pub struct StreamEntry {
    /// Stream ID.
    pub id: u64,
    /// Current state.
    pub state: StreamState,
    /// Send buffer (data waiting to be sent).
    send_buf: VecDeque<u8>,
    /// Receive buffer (data received, not yet read by application).
    recv_buf: VecDeque<u8>,
    /// Total bytes sent on this stream.
    bytes_sent: u64,
    /// Total bytes received on this stream.
    bytes_received: u64,
    /// Send-side FIN has been queued.
    send_fin: bool,
    /// Receive-side FIN has been received.
    recv_fin: bool,
    /// Maximum data the peer allows us to send on this stream.
    max_stream_data_send: u64,
    /// Maximum data we allow the peer to send on this stream.
    _max_stream_data_recv: u64,
}

impl StreamEntry {
    fn new(id: u64, max_send: u64, max_recv: u64) -> Self {
        Self {
            id,
            state: StreamState::Open,
            send_buf: VecDeque::new(),
            recv_buf: VecDeque::new(),
            bytes_sent: 0,
            bytes_received: 0,
            send_fin: false,
            recv_fin: false,
            max_stream_data_send: max_send,
            _max_stream_data_recv: max_recv,
        }
    }

    /// Queue data for sending.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the stream is closed or send is finished.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if self.send_fin {
            return Err(QuicError::Stream("Cannot write after FIN".into()));
        }
        if matches!(self.state, StreamState::Closed | StreamState::Reset) {
            return Err(QuicError::Stream("Stream is closed".into()));
        }
        self.send_buf.extend(data);
        Ok(())
    }

    /// Read received data into the provided buffer.
    /// Returns bytes actually read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let n = buf.len().min(self.recv_buf.len());
        for (i, byte) in self.recv_buf.drain(..n).enumerate() {
            buf[i] = byte;
        }
        n
    }

    /// Receive data from the peer.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the stream is closed or reset.
    pub fn receive(&mut self, data: &[u8]) -> Result<()> {
        if matches!(self.state, StreamState::Closed | StreamState::Reset) {
            return Err(QuicError::Stream("Stream is closed".into()));
        }
        self.recv_buf.extend(data);
        self.bytes_received += data.len() as u64;
        Ok(())
    }

    /// Take pending send data (up to `max_len` bytes).
    pub fn take_send_data(&mut self, max_len: usize) -> Vec<u8> {
        let n = max_len.min(self.send_buf.len());
        let data: Vec<u8> = self.send_buf.drain(..n).collect();
        self.bytes_sent += data.len() as u64;
        data
    }

    /// Whether there is data waiting to be sent.
    #[must_use]
    pub fn has_pending_send(&self) -> bool {
        !self.send_buf.is_empty()
            || (self.send_fin
                && !matches!(self.state, StreamState::SendFinished | StreamState::Closed))
    }

    /// Whether there is received data available to read.
    #[must_use]
    pub fn has_readable_data(&self) -> bool {
        !self.recv_buf.is_empty()
    }

    /// Mark the send side as finished.
    pub fn finish_send(&mut self) {
        self.send_fin = true;
        if self.send_buf.is_empty() {
            self.state = if self.recv_fin {
                StreamState::Closed
            } else {
                StreamState::SendFinished
            };
        }
    }

    /// Mark the receive side as having received FIN.
    pub fn finish_recv(&mut self) {
        self.recv_fin = true;
        self.state = if self.send_fin {
            StreamState::Closed
        } else {
            StreamState::RecvFinished
        };
    }

    /// Reset the stream.
    pub fn reset(&mut self) {
        self.state = StreamState::Reset;
        self.send_buf.clear();
    }

    /// Available send window (bytes the peer allows minus bytes already sent).
    #[must_use]
    pub fn send_window(&self) -> u64 {
        self.max_stream_data_send.saturating_sub(self.bytes_sent)
    }
}

/// Manages all streams for a QUIC connection.
#[derive(Debug)]
pub struct StreamManager {
    streams: BTreeMap<u64, StreamEntry>,
    _is_server: bool,
    next_bidi_id: u64,
    next_uni_id: u64,
    max_bidi_streams: u64,
    max_uni_streams: u64,
    default_max_stream_data: u64,
}

impl StreamManager {
    /// Create a new stream manager.
    #[must_use]
    pub fn new(is_server: bool, max_bidi: u64, max_uni: u64, default_max_stream_data: u64) -> Self {
        let (bidi_base, uni_base) = if is_server {
            (1, 3)
        } else {
            (0, 2)
        };
        Self {
            streams: BTreeMap::new(),
            _is_server: is_server,
            next_bidi_id: bidi_base,
            next_uni_id: uni_base,
            max_bidi_streams: max_bidi,
            max_uni_streams: max_uni,
            default_max_stream_data,
        }
    }

    /// Open a new locally-initiated bidirectional stream.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the bidirectional stream limit is reached.
    pub fn open_bidi(&mut self) -> Result<u64> {
        let id = self.next_bidi_id;
        if self.bidi_count() >= self.max_bidi_streams {
            return Err(QuicError::Stream("Max bidi streams exceeded".into()));
        }
        self.streams.insert(
            id,
            StreamEntry::new(id, self.default_max_stream_data, self.default_max_stream_data),
        );
        self.next_bidi_id += 4;
        Ok(id)
    }

    /// Open a new locally-initiated unidirectional stream.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::Stream`] if the unidirectional stream limit is reached.
    pub fn open_uni(&mut self) -> Result<u64> {
        let id = self.next_uni_id;
        if self.uni_count() >= self.max_uni_streams {
            return Err(QuicError::Stream("Max uni streams exceeded".into()));
        }
        self.streams.insert(
            id,
            StreamEntry::new(id, self.default_max_stream_data, self.default_max_stream_data),
        );
        self.next_uni_id += 4;
        Ok(id)
    }

    /// Accept a remotely-initiated stream (creates the entry if new).
    ///
    /// # Errors
    ///
    /// Never returns `Err`; [`Result`] is reserved for future validation.
    pub fn accept_remote(&mut self, stream_id: u64) -> Result<&mut StreamEntry> {
        Ok(self.streams.entry(stream_id).or_insert_with(|| {
            StreamEntry::new(stream_id, self.default_max_stream_data, self.default_max_stream_data)
        }))
    }

    /// Get a stream entry by ID.
    #[must_use]
    pub fn get(&self, stream_id: u64) -> Option<&StreamEntry> {
        self.streams.get(&stream_id)
    }

    /// Get a mutable stream entry by ID.
    pub fn get_mut(&mut self, stream_id: u64) -> Option<&mut StreamEntry> {
        self.streams.get_mut(&stream_id)
    }

    /// Number of active bidirectional streams.
    #[must_use]
    pub fn bidi_count(&self) -> u64 {
        self.streams
            .keys()
            .filter(|id| {
                stream_type(**id) == StreamType::Bidi
                    && !matches!(self.streams[id].state, StreamState::Closed | StreamState::Reset)
            })
            .count() as u64
    }

    /// Number of active unidirectional streams.
    #[must_use]
    pub fn uni_count(&self) -> u64 {
        self.streams
            .keys()
            .filter(|id| {
                stream_type(**id) == StreamType::Uni
                    && !matches!(self.streams[id].state, StreamState::Closed | StreamState::Reset)
            })
            .count() as u64
    }

    /// Total number of open streams.
    #[must_use]
    pub fn total_open(&self) -> usize {
        self.streams
            .values()
            .filter(|s| !matches!(s.state, StreamState::Closed | StreamState::Reset))
            .count()
    }

    /// IDs of streams with pending send data.
    #[must_use]
    pub fn streams_with_pending_data(&self) -> Vec<u64> {
        self.streams.iter().filter(|(_, s)| s.has_pending_send()).map(|(id, _)| *id).collect()
    }

    /// Update the peer's max stream data for a specific stream.
    pub fn update_max_stream_data(&mut self, stream_id: u64, max_data: u64) {
        if let Some(s) = self.streams.get_mut(&stream_id) {
            s.max_stream_data_send = max_data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_id_encoding() {
        // Client bidi: 0, 4, 8, ...
        assert_eq!(stream_type(0), StreamType::Bidi);
        assert_eq!(stream_initiator(0), Initiator::Client);

        // Server bidi: 1, 5, 9, ...
        assert_eq!(stream_type(1), StreamType::Bidi);
        assert_eq!(stream_initiator(1), Initiator::Server);

        // Client uni: 2, 6, 10, ...
        assert_eq!(stream_type(2), StreamType::Uni);
        assert_eq!(stream_initiator(2), Initiator::Client);

        // Server uni: 3, 7, 11, ...
        assert_eq!(stream_type(3), StreamType::Uni);
        assert_eq!(stream_initiator(3), Initiator::Server);
    }

    #[test]
    fn stream_manager_client_ids() {
        let mut mgr = StreamManager::new(false, 100, 100, 65536);
        let id1 = mgr.open_bidi().unwrap();
        let id2 = mgr.open_bidi().unwrap();
        assert_eq!(id1, 0); // Client bidi starts at 0
        assert_eq!(id2, 4);

        let uid1 = mgr.open_uni().unwrap();
        let uid2 = mgr.open_uni().unwrap();
        assert_eq!(uid1, 2); // Client uni starts at 2
        assert_eq!(uid2, 6);
    }

    #[test]
    fn stream_manager_server_ids() {
        let mut mgr = StreamManager::new(true, 100, 100, 65536);
        let id1 = mgr.open_bidi().unwrap();
        assert_eq!(id1, 1); // Server bidi starts at 1
        let uid1 = mgr.open_uni().unwrap();
        assert_eq!(uid1, 3); // Server uni starts at 3
    }

    #[test]
    fn stream_write_and_read() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        entry.write(b"hello").unwrap();
        assert!(entry.has_pending_send());

        let data = entry.take_send_data(5);
        assert_eq!(data, b"hello");
        assert!(!entry.has_pending_send());
    }

    #[test]
    fn stream_receive_and_read() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        entry.receive(b"world").unwrap();
        assert!(entry.has_readable_data());

        let mut buf = [0u8; 10];
        let n = entry.read(&mut buf);
        assert_eq!(n, 5);
        assert_eq!(&buf[..n], b"world");
        assert!(!entry.has_readable_data());
    }

    #[test]
    fn stream_finish_send_and_recv() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        assert_eq!(entry.state, StreamState::Open);

        entry.finish_send();
        assert_eq!(entry.state, StreamState::SendFinished);

        entry.finish_recv();
        assert_eq!(entry.state, StreamState::Closed);
    }

    #[test]
    fn stream_finish_recv_then_send() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        entry.finish_recv();
        assert_eq!(entry.state, StreamState::RecvFinished);

        entry.finish_send();
        assert_eq!(entry.state, StreamState::Closed);
    }

    #[test]
    fn stream_reset() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        entry.write(b"data").unwrap();
        entry.reset();
        assert_eq!(entry.state, StreamState::Reset);
        assert!(!entry.has_pending_send());
    }

    #[test]
    fn write_after_fin_errors() {
        let mut entry = StreamEntry::new(0, 65536, 65536);
        entry.finish_send();
        assert!(entry.write(b"more").is_err());
    }

    #[test]
    fn max_bidi_streams_enforced() {
        let mut mgr = StreamManager::new(false, 2, 100, 65536);
        mgr.open_bidi().unwrap();
        mgr.open_bidi().unwrap();
        assert!(mgr.open_bidi().is_err());
    }

    #[test]
    fn accept_remote_creates_entry() {
        let mut mgr = StreamManager::new(false, 100, 100, 65536);
        let entry = mgr.accept_remote(1).unwrap(); // Server-initiated bidi
        assert_eq!(entry.id, 1);
        assert_eq!(entry.state, StreamState::Open);
    }

    #[test]
    fn total_open_counts() {
        let mut mgr = StreamManager::new(false, 100, 100, 65536);
        assert_eq!(mgr.total_open(), 0);
        mgr.open_bidi().unwrap();
        mgr.open_uni().unwrap();
        assert_eq!(mgr.total_open(), 2);
    }

    #[test]
    fn send_window_tracks_usage() {
        let mut entry = StreamEntry::new(0, 100, 65536);
        assert_eq!(entry.send_window(), 100);
        entry.write(b"hello").unwrap();
        entry.take_send_data(5);
        assert_eq!(entry.send_window(), 95);
    }

    #[test]
    fn streams_with_pending_data() {
        let mut mgr = StreamManager::new(false, 100, 100, 65536);
        let id1 = mgr.open_bidi().unwrap();
        let id2 = mgr.open_bidi().unwrap();
        mgr.get_mut(id1).unwrap().write(b"data").unwrap();
        let pending = mgr.streams_with_pending_data();
        assert_eq!(pending, vec![id1]);
        assert!(!pending.contains(&id2));
    }
}
