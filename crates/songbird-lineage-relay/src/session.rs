// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Connection session abstraction
//!
//! Handles both direct and relayed connections transparently

use crate::error::Result;
use crate::relay::RelaySession;
use crate::types::{ConnectionStats, ConnectionType, NodeId};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tracing::debug;

/// Connection session (direct, lineage-relayed, or TURN-relayed)
pub enum ConnectionSession {
    /// Direct connection (no relay)
    Direct(DirectConnection),
    /// Relayed connection through ancestor
    Relayed(RelayedConnection),
    /// Relayed via sovereign TURN server (RFC 5766)
    TurnRelayed(TurnRelayedConnection),
}

impl ConnectionSession {
    /// Send data through connection
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        match self {
            Self::Direct(conn) => conn.send(data).await,
            Self::Relayed(conn) => conn.send(data).await,
            Self::TurnRelayed(conn) => conn.send(data).await,
        }
    }

    /// Get connection type
    #[must_use]
    pub const fn connection_type(&self) -> ConnectionType {
        match self {
            Self::Direct(_) => ConnectionType::Direct,
            Self::Relayed(_) => ConnectionType::Relayed,
            Self::TurnRelayed(_) => ConnectionType::TurnRelayed,
        }
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        match self {
            Self::Direct(conn) => conn.stats().await,
            Self::Relayed(conn) => conn.stats().await,
            Self::TurnRelayed(conn) => conn.stats().await,
        }
    }

    /// Attempt to upgrade relayed connection to direct
    ///
    /// # Errors
    ///
    /// Returns error if upgrade fails
    pub async fn attempt_upgrade(&mut self) -> Result<bool> {
        match self {
            Self::Relayed(_) | Self::TurnRelayed(_) => {
                debug!("Attempting to upgrade relayed connection to direct");
                Ok(false)
            }
            Self::Direct(_) => Ok(true),
        }
    }
}

/// Direct connection (no relay)
pub struct DirectConnection {
    peer: NodeId,
    /// Remote address (used when relay mode fully implemented)
    _address: SocketAddr,
    stats: Arc<Mutex<ConnectionStats>>,
}

impl DirectConnection {
    /// Create new direct connection
    #[must_use]
    pub fn new(peer: NodeId, address: SocketAddr) -> Self {
        Self {
            peer,
            _address: address,
            stats: Arc::new(Mutex::new(ConnectionStats {
                established_at: Some(SystemTime::now()),
                connection_type: Some(ConnectionType::Direct),
                ..Default::default()
            })),
        }
    }

    /// Send data directly
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes directly to {}", data.len(), self.peer);

        let mut stats = self.stats.lock().await;
        stats.bytes_sent += data.len() as u64;
        stats.packets_sent += 1;
        drop(stats);

        // In real implementation, would send through UDP/TCP socket
        Ok(())
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        self.stats.lock().await.clone()
    }
}

/// Relayed connection through ancestor
pub struct RelayedConnection {
    relay_session: Arc<RelaySession>,
    stats: Arc<Mutex<ConnectionStats>>,
}

impl RelayedConnection {
    /// Create new relayed connection
    #[must_use]
    pub fn new(relay_session: Arc<RelaySession>) -> Self {
        Self {
            relay_session,
            stats: Arc::new(Mutex::new(ConnectionStats {
                established_at: Some(SystemTime::now()),
                connection_type: Some(ConnectionType::Relayed),
                ..Default::default()
            })),
        }
    }

    /// Send data through relay
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes through relay {}", data.len(), self.relay_session.relay_node);

        self.relay_session.send(data).await?;

        let mut stats = self.stats.lock().await;
        stats.bytes_sent += data.len() as u64;
        stats.packets_sent += 1;
        drop(stats);

        Ok(())
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        let mut stats = self.stats.lock().await.clone();
        // Add relay-specific stats
        stats.bytes_sent = self.relay_session.stats();
        stats
    }
}

/// TURN-relayed connection via sovereign VPS relay (RFC 5766).
///
/// Wraps a [`songbird_turn_client::TurnSession`] with stats tracking and
/// automatic keepalive. Created when the `ConnectionFallbackChain` reaches
/// Tier 4 (TURN relay).
pub struct TurnRelayedConnection {
    session: Arc<songbird_turn_client::TurnSession>,
    stats: Arc<Mutex<ConnectionStats>>,
    keepalive_handle: tokio::task::JoinHandle<()>,
}

impl TurnRelayedConnection {
    /// Create from a connected `TurnSession`, spawning a keepalive task.
    pub fn new(session: Arc<songbird_turn_client::TurnSession>) -> Self {
        let keepalive = session.spawn_keepalive();
        Self {
            session,
            stats: Arc::new(Mutex::new(ConnectionStats {
                established_at: Some(SystemTime::now()),
                connection_type: Some(ConnectionType::TurnRelayed),
                ..Default::default()
            })),
            keepalive_handle: keepalive,
        }
    }

    /// Send data through the TURN relay.
    ///
    /// # Errors
    ///
    /// Returns error if the TURN session send fails.
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        debug!("Sending {} bytes through TURN relay", data.len());

        self.session.send(data).await.map_err(|e| {
            crate::error::LineageRelayError::NetworkError(format!("TURN send: {e}"))
        })?;

        let mut stats = self.stats.lock().await;
        stats.bytes_sent += data.len() as u64;
        stats.packets_sent += 1;
        Ok(())
    }

    /// Receive data from the TURN relay.
    ///
    /// # Errors
    ///
    /// Returns error on timeout or I/O failure.
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        let n = self.session.recv(buf).await.map_err(|e| {
            crate::error::LineageRelayError::NetworkError(format!("TURN recv: {e}"))
        })?;
        let mut stats = self.stats.lock().await;
        stats.bytes_received += n as u64;
        Ok(n)
    }

    /// Get connection statistics.
    pub async fn stats(&self) -> ConnectionStats {
        self.stats.lock().await.clone()
    }

    /// The relay address allocated by the TURN server.
    #[must_use]
    pub fn relay_addr(&self) -> SocketAddr {
        self.session.relay_addr()
    }
}

impl Drop for TurnRelayedConnection {
    fn drop(&mut self) {
        self.keepalive_handle.abort();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::types::MaskingLevel;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_direct_connection() {
        let conn = DirectConnection::new(NodeId::from("peer-1"), "127.0.0.1:8080".parse().unwrap());

        conn.send(b"test data").await.unwrap();

        let stats = conn.stats().await;
        assert_eq!(stats.bytes_sent, 9);
        assert_eq!(stats.packets_sent, 1);
        assert_eq!(stats.connection_type, Some(ConnectionType::Direct));
    }

    #[tokio::test]
    async fn test_relayed_connection() {
        // Bind a server first
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let relay_session = RelaySession::new(
            NodeId::from("relay-1"),
            server_addr,
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        )
        .await
        .unwrap();

        let conn = RelayedConnection::new(Arc::new(relay_session));

        conn.send(b"test data").await.unwrap();

        let stats = conn.stats().await;
        assert_eq!(stats.bytes_sent, 9);
        assert_eq!(stats.connection_type, Some(ConnectionType::Relayed));
    }

    #[tokio::test]
    async fn test_connection_session_enum() {
        let direct =
            DirectConnection::new(NodeId::from("peer-1"), "127.0.0.1:8080".parse().unwrap());
        let session = ConnectionSession::Direct(direct);

        assert_eq!(session.connection_type(), ConnectionType::Direct);

        session.send(b"hello").await.unwrap();

        let stats = session.stats().await;
        assert_eq!(stats.bytes_sent, 5);
    }

    #[tokio::test]
    async fn attempt_upgrade_direct_reports_already_direct() {
        let mut session = ConnectionSession::Direct(DirectConnection::new(
            NodeId::from("peer-1"),
            "127.0.0.1:8080".parse().unwrap(),
        ));
        assert!(session.attempt_upgrade().await.expect("upgrade"));
    }

    #[tokio::test]
    async fn attempt_upgrade_relayed_returns_false_without_socket_upgrade() {
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();
        let relay_session = RelaySession::new(
            NodeId::from("relay-1"),
            server_addr,
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        )
        .await
        .unwrap();
        let mut session =
            ConnectionSession::Relayed(RelayedConnection::new(Arc::new(relay_session)));
        assert!(!session.attempt_upgrade().await.expect("upgrade attempt"));
    }

    #[tokio::test]
    async fn relayed_stats_reflects_session_counter_not_local_mutex_after_send() {
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();
        let relay_session = Arc::new(
            RelaySession::new(
                NodeId::from("relay-1"),
                server_addr,
                NodeId::from("requester"),
                NodeId::from("target"),
                MaskingLevel::None,
            )
            .await
            .unwrap(),
        );
        relay_session.send(b"abc").await.unwrap();
        let conn = RelayedConnection::new(relay_session.clone());
        let stats = conn.stats().await;
        assert_eq!(stats.bytes_sent, relay_session.stats());
        assert_eq!(stats.bytes_sent, 3);
    }
}
