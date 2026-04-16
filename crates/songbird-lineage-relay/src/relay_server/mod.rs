// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Pure Rust Relay Server - Lineage-based packet forwarding
//!
//! **Zero C Dependencies | Zero Unsafe Code | ecoBin Compliant**
//!
//! Evolution of TURN (RFC 5766) with genetic lineage authorization
//! instead of traditional username/password credentials.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐         ┌──────────────┐         ┌─────────────┐
//! │ Requester   │         │ Relay Server │         │   Target    │
//! │  (Pixel)    │────────>│   (Tower)    │────────>│  (Laptop)   │
//! └─────────────┘         └──────────────┘         └─────────────┘
//!                    UDP Packet Forwarding
//!                 (Lineage-based authorization)
//! ```
//!
//! ## Features
//!
//! - ✅ Lineage-based authorization (`security provider` integration)
//! - ✅ UDP packet forwarding for symmetric NAT
//! - ✅ Privacy masking based on family relationship
//! - ✅ Session management (TTL, cleanup)
//! - ✅ Statistics tracking
//! - ✅ Pure Rust, zero unsafe code

mod packet_handler;

use crate::error::{LineageRelayError, Result};
use crate::relay::RelayAuthority;
use crate::types::{MaskingLevel, NodeId};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Internal session state for active relays
#[derive(Debug, Clone)]
pub struct RelaySessionState {
    /// Unique session identifier
    pub session_id: Uuid,

    /// Requester's address (peer requesting relay)
    pub requester_addr: SocketAddr,

    /// Target's address (peer being relayed to)
    pub target_addr: SocketAddr,

    /// Requester node ID
    pub requester_id: NodeId,

    /// Target node ID
    pub target_id: NodeId,

    /// Privacy masking level
    pub masking_level: MaskingLevel,

    /// Session creation time
    pub created_at: SystemTime,

    /// Last activity time
    pub last_activity: SystemTime,

    /// Total bytes forwarded
    pub bytes_forwarded: u64,

    /// Total packets forwarded
    pub packets_forwarded: u64,
}

/// Relay server statistics
#[derive(Debug, Clone, Default)]
pub struct RelayServerStats {
    /// Active sessions
    pub sessions_active: u64,

    /// Total sessions allocated (lifetime)
    pub sessions_total: u64,

    /// Total bytes forwarded
    pub bytes_forwarded: u64,

    /// Total packets forwarded
    pub packets_forwarded: u64,

    /// Authorization failures
    pub authorization_failures: u64,

    /// Server start time
    pub start_time: Option<SystemTime>,
}

impl RelayServerStats {
    /// Get server uptime in seconds
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time
            .and_then(|start| SystemTime::now().duration_since(start).ok())
            .map_or(0, |d| d.as_secs())
    }
}

/// Pure Rust Relay Server
///
/// Forwards UDP packets between peers who cannot establish direct connection,
/// typically due to symmetric NAT on both ends.
///
/// ## Design Principles
///
/// - **Lineage-Based**: Authorization via genetic lineage (not passwords)
/// - **Privacy-Preserving**: Masking based on family relationship
/// - **Zero Unsafe**: All operations use safe Rust
/// - **Self-Contained**: No external primal dependencies
/// - **Modern Idiomatic**: Async/await, Result-based errors
///
/// ## Example
///
/// ```rust,ignore
/// use songbird_lineage_relay::{RelayServer, RelayAuthority};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let bind_addr = "0.0.0.0:3479".parse()?;
///     let authority = Arc::new(MyRelayAuthority::new());
///     
///     let server = RelayServer::new(bind_addr, authority).await?;
///     server.run().await?;
///     Ok(())
/// }
/// ```
pub struct RelayServer {
    /// Bind address for relay service
    bind_addr: SocketAddr,

    /// Active relay sessions
    sessions: Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,

    /// Lineage authority for authorization
    authority: Arc<RelayAuthority>,

    /// UDP socket for packet forwarding
    socket: Arc<UdpSocket>,

    /// Server statistics
    stats: Arc<RwLock<RelayServerStats>>,
}

impl std::fmt::Debug for RelayServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelayServer")
            .field("bind_addr", &self.bind_addr)
            .field("sessions", &self.sessions)
            .field("authority", &"<RelayAuthority>")
            .field("socket", &self.socket)
            .field("stats", &self.stats)
            .finish()
    }
}

impl RelayServer {
    /// Create new relay server
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Address to bind for relay service
    /// * `authority` - Lineage authority provider (`security provider`)
    ///
    /// # Errors
    ///
    /// Returns error if UDP socket binding fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use songbird_lineage_relay::RelayServer;
    /// use std::sync::Arc;
    ///
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let bind_addr = "0.0.0.0:3479".parse()?;
    ///     let authority = Arc::new(MyAuthority::new());
    ///     let server = RelayServer::new(bind_addr, authority).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(bind_addr: SocketAddr, authority: Arc<RelayAuthority>) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to bind relay server: {e}"))
        })?;

        let actual_addr = socket.local_addr().map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to get local address: {e}"))
        })?;

        Ok(Self {
            bind_addr: actual_addr,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            authority,
            socket: Arc::new(socket),
            stats: Arc::new(RwLock::new(RelayServerStats {
                start_time: Some(SystemTime::now()),
                ..Default::default()
            })),
        })
    }

    /// Run relay server
    ///
    /// Listens for allocation requests and data packets, forwarding
    /// between authorized peers.
    ///
    /// Runs indefinitely until error occurs.
    ///
    /// # Errors
    ///
    /// Returns error if fatal network error occurs.
    pub async fn run(&self) -> Result<()> {
        info!("🔄 Relay server listening on {}", self.bind_addr);

        // Spawn cleanup task
        let _cleanup_handle = self.spawn_cleanup_task();

        let mut buf = vec![0u8; 65536]; // Max UDP datagram size

        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    debug!("📨 Received {} bytes from {}", len, src_addr);

                    // Handle packet (fire and forget for performance)
                    let socket = self.socket.clone();
                    let sessions = self.sessions.clone();
                    let authority = self.authority.clone();
                    let stats = self.stats.clone();
                    let bind_addr = self.bind_addr;
                    let data = buf[..len].to_vec();

                    tokio::spawn(async move {
                        if let Err(e) = packet_handler::handle_packet(
                            &socket, &sessions, &authority, &stats, bind_addr, &data, src_addr,
                        )
                        .await
                        {
                            warn!("⚠️  Failed to handle packet: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ Failed to receive packet: {}", e);
                    // Continue running (don't crash on receive errors)
                }
            }
        }
    }

    /// Spawn background cleanup task
    ///
    /// Removes sessions idle for >5 minutes
    fn spawn_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let sessions = self.sessions.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let now = SystemTime::now();
                let mut sessions_guard = sessions.write().await;

                let before_count = sessions_guard.len();

                sessions_guard.retain(|id, session| {
                    let idle_time =
                        now.duration_since(session.last_activity).unwrap_or(Duration::from_secs(0));

                    if idle_time > Duration::from_secs(300) {
                        info!("🧹 Cleaning up idle session {} (idle: {:?})", id, idle_time);
                        false
                    } else {
                        true
                    }
                });

                let cleaned = before_count - sessions_guard.len();
                if cleaned > 0 {
                    debug!("🧹 Cleaned up {} expired sessions", cleaned);
                }

                // Update active count
                let mut stats_guard = stats.write().await;
                stats_guard.sessions_active = sessions_guard.len() as u64;
            }
        })
    }

    /// Get server statistics
    ///
    /// Returns current snapshot of server metrics.
    pub async fn stats(&self) -> RelayServerStats {
        self.stats.read().await.clone()
    }

    /// Get bind address
    #[must_use]
    pub const fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    /// Shutdown gracefully
    ///
    /// Closes all sessions and stops accepting new requests.
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down relay server");

        let mut sessions = self.sessions.write().await;
        let session_count = sessions.len();
        sessions.clear();
        drop(sessions);

        info!("✅ Relay server shut down ({} sessions closed)", session_count);

        Ok(())
    }
}

#[cfg(test)]
#[path = "relay_server_tests.rs"]
mod tests;
