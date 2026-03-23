// SPDX-License-Identifier: AGPL-3.0-only
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

/// Connection session (direct or relayed)
pub enum ConnectionSession {
    /// Direct connection (no relay)
    Direct(DirectConnection),
    /// Relayed connection through ancestor
    Relayed(RelayedConnection),
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
        }
    }

    /// Get connection type
    #[must_use]
    pub const fn connection_type(&self) -> ConnectionType {
        match self {
            Self::Direct(_) => ConnectionType::Direct,
            Self::Relayed(_) => ConnectionType::Relayed,
        }
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        match self {
            Self::Direct(conn) => conn.stats().await,
            Self::Relayed(conn) => conn.stats().await,
        }
    }

    /// Attempt to upgrade relayed connection to direct
    ///
    /// # Errors
    ///
    /// Returns error if upgrade fails
    pub async fn attempt_upgrade(&mut self) -> Result<bool> {
        match self {
            Self::Relayed(_conn) => {
                debug!("Attempting to upgrade relayed connection to direct");
                // In real implementation, would attempt direct connection
                // For now, just return false (no upgrade)
                Ok(false)
            }
            Self::Direct(_) => Ok(true), // Already direct
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

#[cfg(test)]
mod tests {
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
}
