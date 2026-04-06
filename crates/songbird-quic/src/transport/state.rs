// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC connection state machine (RFC 9000 Section 10).

use crate::error::{QuicError, Result};
use std::net::SocketAddr;
use std::time::Instant;

/// QUIC connection states (derived from RFC 9000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not yet started.
    Idle,
    /// Handshake in progress (sending/receiving `Initial` and `Handshake` packets).
    Handshaking,
    /// Handshake complete, application data can flow.
    Connected,
    /// Closing initiated (sending `CONNECTION_CLOSE`).
    Closing,
    /// Draining period (received `CONNECTION_CLOSE`, waiting before cleanup).
    Draining,
    /// Connection fully terminated.
    Closed,
}

/// Connection close reason.
#[derive(Debug, Clone)]
pub enum CloseReason {
    /// Local application requested close.
    Application {
        /// Application error code.
        error_code: u64,
        /// Application-defined reason bytes.
        reason: Vec<u8>,
    },
    /// Transport-level error.
    Transport {
        /// Transport error code.
        error_code: u64,
        /// Frame type that triggered the error, if applicable.
        frame_type: u64,
        /// Human-readable reason bytes.
        reason: Vec<u8>,
    },
    /// Idle timeout expired.
    IdleTimeout,
    /// Stateless reset received.
    StatelessReset,
}

/// Core connection state tracking.
#[derive(Debug)]
pub struct Connection {
    state: ConnectionState,
    /// Our role (client or server).
    is_server: bool,
    /// Remote peer address.
    remote_addr: SocketAddr,
    /// Local source connection ID.
    local_cid: Vec<u8>,
    /// Remote destination connection ID.
    remote_cid: Vec<u8>,
    /// When the connection was created.
    created_at: Instant,
    /// Last activity timestamp.
    last_activity: Instant,
    /// Close reason (if closing/draining/closed).
    close_reason: Option<CloseReason>,
    /// Next packet number to send (per-space in a full impl; simplified here).
    next_pn: u64,
    /// Largest received packet number.
    largest_recv_pn: Option<u64>,
}

impl Connection {
    /// Create a new connection in the Idle state.
    #[must_use]
    pub fn new(
        is_server: bool,
        remote_addr: SocketAddr,
        local_cid: Vec<u8>,
        remote_cid: Vec<u8>,
    ) -> Self {
        let now = Instant::now();
        Self {
            state: ConnectionState::Idle,
            is_server,
            remote_addr,
            local_cid,
            remote_cid,
            created_at: now,
            last_activity: now,
            close_reason: None,
            next_pn: 0,
            largest_recv_pn: None,
        }
    }

    /// Current connection state.
    #[must_use]
    pub const fn state(&self) -> ConnectionState {
        self.state
    }

    /// Whether we are the server.
    #[must_use]
    pub const fn is_server(&self) -> bool {
        self.is_server
    }

    /// Remote peer address.
    #[must_use]
    pub const fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Update remote address (connection migration).
    pub fn set_remote_addr(&mut self, addr: SocketAddr) {
        self.remote_addr = addr;
    }

    /// Local connection ID.
    #[must_use]
    pub fn local_cid(&self) -> &[u8] {
        &self.local_cid
    }

    /// Remote connection ID.
    #[must_use]
    pub fn remote_cid(&self) -> &[u8] {
        &self.remote_cid
    }

    /// How long the connection has been alive.
    #[must_use]
    pub fn uptime(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Duration since last activity.
    #[must_use]
    pub fn idle_duration(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }

    /// Record activity (packet sent or received).
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Allocate the next packet number.
    pub fn next_packet_number(&mut self) -> u64 {
        let pn = self.next_pn;
        self.next_pn += 1;
        pn
    }

    /// Record a received packet number.
    pub fn record_received(&mut self, pn: u64) {
        self.largest_recv_pn = Some(self.largest_recv_pn.map_or(pn, |prev| prev.max(pn)));
        self.touch();
    }

    /// Largest received packet number.
    #[must_use]
    pub const fn largest_received_pn(&self) -> Option<u64> {
        self.largest_recv_pn
    }

    /// Close reason (if connection is closing or closed).
    #[must_use]
    pub fn close_reason(&self) -> Option<&CloseReason> {
        self.close_reason.as_ref()
    }

    /// Whether the connection is usable for sending application data.
    #[must_use]
    pub const fn is_established(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    /// Whether the connection is fully terminated.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        matches!(self.state, ConnectionState::Closed)
    }

    // --- State transitions ---

    /// Transition: Idle → Handshaking.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::ConnectionClosed`] if the connection is not in [`ConnectionState::Idle`].
    pub fn start_handshake(&mut self) -> Result<()> {
        self.require_state(ConnectionState::Idle)?;
        self.state = ConnectionState::Handshaking;
        self.touch();
        Ok(())
    }

    /// Transition: Handshaking → Connected.
    ///
    /// # Errors
    ///
    /// Returns [`QuicError::ConnectionClosed`] if the connection is not in [`ConnectionState::Handshaking`].
    pub fn handshake_complete(&mut self) -> Result<()> {
        self.require_state(ConnectionState::Handshaking)?;
        self.state = ConnectionState::Connected;
        self.touch();
        Ok(())
    }

    /// Transition: Connected → Closing (initiated by local).
    ///
    /// # Errors
    ///
    /// Never returns `Err`; included for API consistency with other transitions.
    pub fn initiate_close(&mut self, reason: CloseReason) -> Result<()> {
        if matches!(self.state, ConnectionState::Closed | ConnectionState::Draining) {
            return Ok(());
        }
        self.state = ConnectionState::Closing;
        self.close_reason = Some(reason);
        self.touch();
        Ok(())
    }

    /// Transition: Connected|Handshaking → Draining (received `CONNECTION_CLOSE`).
    ///
    /// # Errors
    ///
    /// Never returns `Err`; included for API consistency with other transitions.
    pub fn enter_draining(&mut self, reason: CloseReason) -> Result<()> {
        if matches!(self.state, ConnectionState::Closed | ConnectionState::Draining) {
            return Ok(());
        }
        self.state = ConnectionState::Draining;
        self.close_reason = Some(reason);
        self.touch();
        Ok(())
    }

    /// Transition: Closing|Draining → Closed.
    pub fn finish_close(&mut self) {
        self.state = ConnectionState::Closed;
    }

    fn require_state(&self, expected: ConnectionState) -> Result<()> {
        if self.state != expected {
            return Err(QuicError::ConnectionClosed(format!(
                "Expected state {:?}, but in {:?}",
                expected, self.state
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        Connection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![0x01, 0x02],
            vec![0x03, 0x04],
        )
    }

    #[test]
    fn initial_state_is_idle() {
        let conn = test_conn();
        assert_eq!(conn.state(), ConnectionState::Idle);
        assert!(!conn.is_established());
        assert!(!conn.is_closed());
        assert!(!conn.is_server());
    }

    #[test]
    fn normal_lifecycle() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        assert_eq!(conn.state(), ConnectionState::Handshaking);

        conn.handshake_complete().unwrap();
        assert_eq!(conn.state(), ConnectionState::Connected);
        assert!(conn.is_established());

        conn.initiate_close(CloseReason::Application {
            error_code: 0,
            reason: b"bye".to_vec(),
        })
        .unwrap();
        assert_eq!(conn.state(), ConnectionState::Closing);
        assert!(conn.close_reason().is_some());

        conn.finish_close();
        assert_eq!(conn.state(), ConnectionState::Closed);
        assert!(conn.is_closed());
    }

    #[test]
    fn draining_from_connected() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.handshake_complete().unwrap();
        conn.enter_draining(CloseReason::Transport {
            error_code: 0x0A,
            frame_type: 0,
            reason: b"flow control".to_vec(),
        })
        .unwrap();
        assert_eq!(conn.state(), ConnectionState::Draining);
    }

    #[test]
    fn invalid_transition_errors() {
        let mut conn = test_conn();
        assert!(conn.handshake_complete().is_err());
        conn.start_handshake().unwrap();
        assert!(conn.start_handshake().is_err());
    }

    #[test]
    fn packet_number_increments() {
        let mut conn = test_conn();
        assert_eq!(conn.next_packet_number(), 0);
        assert_eq!(conn.next_packet_number(), 1);
        assert_eq!(conn.next_packet_number(), 2);
    }

    #[test]
    fn record_received_tracks_largest() {
        let mut conn = test_conn();
        assert_eq!(conn.largest_received_pn(), None);
        conn.record_received(5);
        assert_eq!(conn.largest_received_pn(), Some(5));
        conn.record_received(3);
        assert_eq!(conn.largest_received_pn(), Some(5));
        conn.record_received(10);
        assert_eq!(conn.largest_received_pn(), Some(10));
    }

    #[test]
    fn connection_ids() {
        let conn = test_conn();
        assert_eq!(conn.local_cid(), &[0x01, 0x02]);
        assert_eq!(conn.remote_cid(), &[0x03, 0x04]);
    }

    #[test]
    fn connection_migration() {
        let mut conn = test_conn();
        let new_addr: SocketAddr = "192.168.1.100:5000".parse().unwrap();
        conn.set_remote_addr(new_addr);
        assert_eq!(conn.remote_addr(), new_addr);
    }

    #[test]
    fn server_flag() {
        let conn = Connection::new(true, "127.0.0.1:4433".parse().unwrap(), vec![], vec![]);
        assert!(conn.is_server());
    }

    #[test]
    fn idle_timeout_close() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.handshake_complete().unwrap();
        conn.initiate_close(CloseReason::IdleTimeout).unwrap();
        assert_eq!(conn.state(), ConnectionState::Closing);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn initiate_close_is_idempotent_after_closed() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.handshake_complete().unwrap();
        conn.initiate_close(CloseReason::StatelessReset).unwrap();
        conn.finish_close();
        assert!(conn.is_closed());
        conn.initiate_close(CloseReason::IdleTimeout).unwrap();
        assert!(conn.is_closed());
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn enter_draining_is_idempotent_when_already_draining() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.handshake_complete().unwrap();
        conn.enter_draining(CloseReason::IdleTimeout).unwrap();
        assert_eq!(conn.state(), ConnectionState::Draining);
        conn.enter_draining(CloseReason::StatelessReset).unwrap();
        assert_eq!(conn.state(), ConnectionState::Draining);
    }

    #[test]
    fn handshake_complete_fails_when_not_handshaking() {
        let mut conn = test_conn();
        assert!(conn.handshake_complete().is_err());
        conn.start_handshake().unwrap();
        conn.handshake_complete().unwrap();
        assert!(conn.handshake_complete().is_err());
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn finish_close_from_handshaking_sets_closed_without_connected() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.finish_close();
        assert_eq!(conn.state(), ConnectionState::Closed);
    }

    #[test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    fn enter_draining_from_handshaking_skips_connected() {
        let mut conn = test_conn();
        conn.start_handshake().unwrap();
        conn.enter_draining(CloseReason::Application {
            error_code: 1,
            reason: b"nope".to_vec(),
        })
        .unwrap();
        assert_eq!(conn.state(), ConnectionState::Draining);
    }
}
