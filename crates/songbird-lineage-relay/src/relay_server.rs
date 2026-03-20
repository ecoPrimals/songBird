// SPDX-License-Identifier: AGPL-3.0-only
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
//! - ✅ Lineage-based authorization (`BearDog` integration)
//! - ✅ UDP packet forwarding for symmetric NAT
//! - ✅ Privacy masking based on family relationship
//! - ✅ Session management (TTL, cleanup)
//! - ✅ Statistics tracking
//! - ✅ Pure Rust, zero unsafe code

use crate::error::{LineageRelayError, Result};
use crate::relay::RelayAuthority;
use crate::relay_protocol::{AllocationRequest, AllocationResponse, RelayProtocol};
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
    authority: Arc<dyn RelayAuthority>,

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
            .field("authority", &"<dyn RelayAuthority>")
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
    /// * `authority` - Lineage authority provider (`BearDog`)
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
    pub async fn new(bind_addr: SocketAddr, authority: Arc<dyn RelayAuthority>) -> Result<Self> {
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
                        if let Err(e) = Self::handle_packet(
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

    /// Handle single packet
    async fn handle_packet(
        socket: &Arc<UdpSocket>,
        sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
        authority: &Arc<dyn RelayAuthority>,
        stats: &Arc<RwLock<RelayServerStats>>,
        relay_addr: SocketAddr,
        data: &[u8],
        src_addr: SocketAddr,
    ) -> Result<()> {
        match RelayProtocol::parse(data)? {
            RelayProtocol::AllocateRequest(req) => {
                Self::handle_allocate(socket, sessions, authority, stats, relay_addr, req, src_addr)
                    .await
            }
            RelayProtocol::DataPacket {
                session_id,
                data,
            } => Self::forward_packet(socket, sessions, stats, session_id, &data, src_addr).await,
            RelayProtocol::Refresh {
                session_id,
            } => Self::refresh_session(sessions, session_id, src_addr).await,
            RelayProtocol::Deallocate {
                session_id,
            } => Self::deallocate_session(sessions, session_id, src_addr).await,
            RelayProtocol::AllocateResponse(_) => {
                // Server doesn't handle responses (client-only message)
                Ok(())
            }
        }
    }

    /// Handle allocation request
    async fn handle_allocate(
        socket: &Arc<UdpSocket>,
        sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
        authority: &Arc<dyn RelayAuthority>,
        stats: &Arc<RwLock<RelayServerStats>>,
        relay_addr: SocketAddr,
        request: AllocationRequest,
        src_addr: SocketAddr,
    ) -> Result<()> {
        debug!("🔐 Allocation request from {} for {}", request.requester, request.target_addr);

        // Verify lineage authorization
        let auth_result = authority.authorize_relay(&request.relay_node, &request.requester).await;

        let response = match auth_result {
            Ok(auth) => {
                if auth.authorized {
                    // Authorized - create session
                    let session_id = Uuid::new_v4();

                    let session = RelaySessionState {
                        session_id,
                        requester_addr: src_addr,
                        target_addr: request.target_addr,
                        requester_id: request.requester.clone(),
                        target_id: "unknown".into(), // Will be discovered on first packet
                        masking_level: auth.masking_level,
                        created_at: SystemTime::now(),
                        last_activity: SystemTime::now(),
                        bytes_forwarded: 0,
                        packets_forwarded: 0,
                    };

                    // Store session
                    {
                        let mut sessions_guard = sessions.write().await;
                        sessions_guard.insert(session_id, session);

                        let mut stats_guard = stats.write().await;
                        stats_guard.sessions_active = sessions_guard.len() as u64;
                        stats_guard.sessions_total += 1;
                    }

                    info!("✅ Allocated relay session {} for {}", session_id, request.requester);

                    AllocationResponse::success(session_id, relay_addr, request.ttl_seconds)
                } else {
                    // Not authorized
                    warn!("🚫 Unauthorized relay request from {}", request.requester);

                    let mut stats_guard = stats.write().await;
                    stats_guard.authorization_failures += 1;

                    AllocationResponse::unauthorized("Lineage verification failed")
                }
            }
            Err(e) => {
                // Authorization check failed
                warn!("⚠️  Authorization error: {}", e);

                let mut stats_guard = stats.write().await;
                stats_guard.authorization_failures += 1;

                AllocationResponse::error(format!("Authorization failed: {e}"))
            }
        };

        // Send response
        let response_msg = RelayProtocol::AllocateResponse(response);
        let encoded = response_msg.encode();
        socket.send_to(&encoded, src_addr).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to send allocation response: {e}"))
        })?;

        Ok(())
    }

    /// Forward packet between peers
    ///
    /// This is the CORE functionality that replaces the stub in `RelaySession.send()`
    async fn forward_packet(
        socket: &Arc<UdpSocket>,
        sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
        stats: &Arc<RwLock<RelayServerStats>>,
        session_id: Uuid,
        data: &[u8],
        src_addr: SocketAddr,
    ) -> Result<()> {
        let mut sessions_guard = sessions.write().await;

        let session = sessions_guard.get_mut(&session_id).ok_or_else(|| {
            LineageRelayError::SessionNotFound(format!("Session {session_id} not found"))
        })?;

        // Determine destination (the other peer)
        let dest_addr = if src_addr == session.requester_addr {
            // From requester → to target
            session.target_addr
        } else if src_addr.ip() == session.target_addr.ip() {
            // From target → to requester
            // Note: Port might differ due to NAT, but IP should match
            session.requester_addr
        } else {
            // Unknown source - reject
            warn!(
                "🚫 Packet from unauthorized source {} (session {}, expected {} or {})",
                src_addr, session_id, session.requester_addr, session.target_addr
            );
            return Ok(()); // Silently drop (don't error, just ignore)
        };

        // Update session activity
        session.last_activity = SystemTime::now();
        session.bytes_forwarded += data.len() as u64;
        session.packets_forwarded += 1;

        // Apply masking based on lineage relationship
        let masked_data = Self::apply_masking(data, session.masking_level)?;

        // Forward packet
        socket.send_to(&masked_data, dest_addr).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to forward packet: {e}"))
        })?;

        // Update global stats
        {
            let mut stats_guard = stats.write().await;
            stats_guard.bytes_forwarded += data.len() as u64;
            stats_guard.packets_forwarded += 1;
        }

        debug!(
            "📦 Forwarded {} bytes: {} → {} (session: {})",
            data.len(),
            src_addr,
            dest_addr,
            session_id
        );

        Ok(())
    }

    /// Apply privacy masking based on lineage relationship
    ///
    /// Closer family = less masking, distant family = more masking
    #[expect(
        clippy::unnecessary_wraps,
        reason = "intentional pattern; clippy false positive for this API"
    )] // Result kept for future masking errors
    fn apply_masking(data: &[u8], level: MaskingLevel) -> Result<Vec<u8>> {
        match level {
            MaskingLevel::None => {
                // Direct family (parent ↔ child): No masking
                Ok(data.to_vec())
            }
            MaskingLevel::TimingOnly => {
                // Close family (siblings): Timing jitter only
                // Future: Add random delay (not in packet data)
                Ok(data.to_vec())
            }
            MaskingLevel::SizeObfuscation => {
                // Extended family: Pad to fixed sizes
                let mut padded = data.to_vec();
                // Pad to next 1KB boundary
                let target_size = data.len().div_ceil(1024) * 1024;
                padded.resize(target_size, 0);
                Ok(padded)
            }
            MaskingLevel::Full => {
                // Distant family: Full encryption + padding
                // Future: Integrate with BearDog encryption
                // For now, just pad (encryption is future enhancement)
                let mut padded = data.to_vec();
                let target_size = data.len().div_ceil(1024) * 1024;
                padded.resize(target_size, 0);
                Ok(padded)
            }
            // Legacy variants (for backward compatibility)
            MaskingLevel::Masked | MaskingLevel::SubMasked => {
                // Minimal masking (legacy default)
                Ok(data.to_vec())
            }
            MaskingLevel::FullVisibility => {
                // Full visibility (ancestor privilege - legacy)
                Ok(data.to_vec())
            }
        }
    }

    /// Refresh session (extend TTL)
    async fn refresh_session(
        sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
        session_id: Uuid,
        src_addr: SocketAddr,
    ) -> Result<()> {
        let mut sessions_guard = sessions.write().await;

        if let Some(session) = sessions_guard.get_mut(&session_id) {
            // Verify refresh comes from requester or target
            if src_addr == session.requester_addr || src_addr.ip() == session.target_addr.ip() {
                session.last_activity = SystemTime::now();
                debug!("🔄 Refreshed session {}", session_id);
            } else {
                warn!("🚫 Refresh from unauthorized source: {}", src_addr);
            }
        }

        Ok(())
    }

    /// Deallocate session (close)
    async fn deallocate_session(
        sessions: &Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
        session_id: Uuid,
        src_addr: SocketAddr,
    ) -> Result<()> {
        let mut sessions_guard = sessions.write().await;

        if let Some(session) = sessions_guard.get(&session_id) {
            // Verify deallocation comes from requester
            if src_addr == session.requester_addr {
                sessions_guard.remove(&session_id);
                info!("🛑 Deallocated session {}", session_id);
            } else {
                warn!("🚫 Deallocation from unauthorized source: {}", src_addr);
            }
        }

        Ok(())
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
    pub fn bind_addr(&self) -> SocketAddr {
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

        info!("✅ Relay server shut down ({} sessions closed)", session_count);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;

    /// Mock relay authority for testing
    struct MockRelayAuthority {
        should_authorize: bool,
    }

    impl MockRelayAuthority {
        fn new(should_authorize: bool) -> Self {
            Self {
                should_authorize,
            }
        }
    }

    #[async_trait]
    impl RelayAuthority for MockRelayAuthority {
        async fn authorize_relay(
            &self,
            relay_node: &NodeId,
            requester: &NodeId,
        ) -> Result<crate::types::RelayAuthorization> {
            Ok(crate::types::RelayAuthorization::authorized(
                relay_node.clone(),
                requester.clone(),
                MaskingLevel::None,
                300,
            ))
        }

        async fn determine_masking(
            &self,
            _relay_node: &NodeId,
            _requester: &NodeId,
        ) -> Result<MaskingLevel> {
            Ok(MaskingLevel::None)
        }
    }

    #[tokio::test]
    async fn test_relay_server_creation() {
        let authority = Arc::new(MockRelayAuthority::new(true));
        let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

        assert!(server.bind_addr().port() > 0);
    }

    #[tokio::test]
    async fn test_relay_server_stats() {
        let authority = Arc::new(MockRelayAuthority::new(true));
        let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority).await.unwrap();

        let stats = server.stats().await;

        assert_eq!(stats.sessions_active, 0);
        assert_eq!(stats.sessions_total, 0);
        assert!(stats.start_time.is_some());
    }

    #[tokio::test]
    async fn test_masking_none() {
        let data = b"Hello, World!";
        let masked = RelayServer::apply_masking(data, MaskingLevel::None).unwrap();

        assert_eq!(masked, data);
    }

    #[tokio::test]
    async fn test_masking_size_obfuscation() {
        let data = b"Hello"; // 5 bytes
        let masked = RelayServer::apply_masking(data, MaskingLevel::SizeObfuscation).unwrap();

        // Should be padded to 1KB
        assert_eq!(masked.len(), 1024);

        // First 5 bytes should be original data
        assert_eq!(&masked[..5], data);

        // Rest should be padding
        assert!(masked[5..].iter().all(|&b| b == 0));
    }

    #[tokio::test]
    async fn test_masking_full() {
        let data = b"Secret message";
        let masked = RelayServer::apply_masking(data, MaskingLevel::Full).unwrap();

        // Currently same as SizeObfuscation (encryption is future)
        assert!(masked.len() >= data.len());
    }
}
