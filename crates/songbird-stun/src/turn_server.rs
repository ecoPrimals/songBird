// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 5766 TURN relay server for sovereign VPS deployment (H2-14 / Pass 12).
//!
//! This server is the counterpart to [`crate::turn::TurnClient`]. It runs on a
//! VPS and allocates relay addresses for clients behind symmetric NATs. Traffic
//! is forwarded between allocated relay sockets and permitted peers.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                   TurnRelayServer                         │
//! ├──────────────────────────────────────────────────────────┤
//! │  Listener Socket (:3478)                                 │
//! │   ├─ Allocate → bind ephemeral relay socket              │
//! │   ├─ Refresh → update allocation TTL                     │
//! │   ├─ CreatePermission → allow peer IP                    │
//! │   ├─ ChannelBind → map channel# to peer                 │
//! │   └─ BindingRequest → XOR-MAPPED-ADDRESS (STUN compat)  │
//! │                                                          │
//! │  Relay Sockets (ephemeral ports)                         │
//! │   └─ Forward data between client and permitted peers     │
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Authentication
//!
//! Credentials are verified via the `CredentialStore` trait which maps usernames
//! to shared secrets. In production, keys are derived from `BearDog`'s
//! beacon-tier crypto delegation.

#[path = "turn_attrs.rs"]
mod turn_attrs;

use crate::error::{StunError, StunResult};
use crate::message::{MessageType, StunAttribute, StunMessage};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use turn_attrs::TurnAttrs;

const DEFAULT_ALLOCATION_LIFETIME: u32 = 600;
const MAX_ALLOCATION_LIFETIME: u32 = 3600;
const CLEANUP_INTERVAL: Duration = Duration::from_secs(30);

/// Credential store for TURN authentication.
///
/// Maps usernames to their shared HMAC keys (beacon-tier material from
/// `BearDog` `auth.public_key`).
pub trait CredentialStore: Send + Sync {
    /// Look up the HMAC key for a username.
    fn get_key(&self, username: &str) -> Option<Vec<u8>>;
}

/// In-memory credential store for testing and simple deployments.
#[derive(Debug, Clone, Default)]
pub struct StaticCredentialStore {
    credentials: HashMap<String, Vec<u8>>,
}

impl StaticCredentialStore {
    /// Create an empty credential store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a credential.
    pub fn insert(&mut self, username: String, key: Vec<u8>) {
        self.credentials.insert(username, key);
    }

    /// Number of stored credentials.
    #[must_use]
    pub fn len(&self) -> usize {
        self.credentials.len()
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }
}

impl CredentialStore for StaticCredentialStore {
    fn get_key(&self, username: &str) -> Option<Vec<u8>> {
        self.credentials.get(username).cloned()
    }
}

/// A single TURN allocation.
#[derive(Debug)]
#[allow(dead_code, reason = "fields model RFC 5766 state; client_addr/relay_addr for diagnostics")]
struct Allocation {
    username: String,
    client_addr: SocketAddr,
    relay_socket: Arc<UdpSocket>,
    relay_addr: SocketAddr,
    permissions: Vec<IpAddr>,
    channels: HashMap<u16, SocketAddr>,
    expires_at: Instant,
}

/// TURN relay server statistics.
#[derive(Debug, Default, Clone)]
pub struct TurnRelayStats {
    /// Total allocations created.
    pub allocations_created: u64,
    /// Currently active allocations.
    pub active_allocations: u64,
    /// Total data packets relayed.
    pub packets_relayed: u64,
    /// Total bytes relayed.
    pub bytes_relayed: u64,
    /// Total authentication failures.
    pub auth_failures: u64,
    /// Server start time.
    pub start_time: Option<Instant>,
}

/// TURN relay server (RFC 5766).
///
/// Sovereign VPS relay for NAT traversal when direct and STUN-assisted
/// connectivity both fail.
pub struct TurnRelayServer {
    bind_addr: SocketAddr,
    credentials: Arc<dyn CredentialStore>,
    allocations: Arc<RwLock<HashMap<SocketAddr, Allocation>>>,
    stats: Arc<RwLock<TurnRelayStats>>,
}

impl TurnRelayServer {
    /// Create a new TURN relay server.
    #[must_use]
    pub fn new(bind_addr: SocketAddr, credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            bind_addr,
            credentials,
            allocations: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(TurnRelayStats::default())),
        }
    }

    /// Get server statistics.
    pub async fn stats(&self) -> TurnRelayStats {
        self.stats.read().await.clone()
    }

    /// Run the TURN relay server.
    ///
    /// # Errors
    ///
    /// Returns an error if socket binding fails.
    pub async fn run(&self) -> StunResult<()> {
        self.run_inner(None).await
    }

    /// Run with a readiness signal (for tests and orchestration).
    ///
    /// # Errors
    ///
    /// Returns an error if socket binding fails.
    pub async fn run_with_ready(
        &self,
        ready_tx: tokio::sync::oneshot::Sender<SocketAddr>,
    ) -> StunResult<()> {
        self.run_inner(Some(ready_tx)).await
    }

    async fn run_inner(
        &self,
        ready_tx: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
    ) -> StunResult<()> {
        let socket = UdpSocket::bind(self.bind_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN bind failed: {e}")))?;

        let actual_addr =
            socket.local_addr().map_err(|e| StunError::Network(format!("TURN local_addr: {e}")))?;

        info!("TURN relay server listening on {actual_addr}");

        if let Some(tx) = ready_tx {
            let _ = tx.send(actual_addr);
        }

        {
            let mut stats = self.stats.write().await;
            stats.start_time = Some(Instant::now());
        }

        let socket = Arc::new(socket);

        let allocs = Arc::clone(&self.allocations);
        let stats_ref = Arc::clone(&self.stats);
        tokio::spawn(async move {
            Self::cleanup_loop(allocs, stats_ref).await;
        });

        let mut buf = vec![0u8; 4096];

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src_addr)) => {
                    if let Err(e) = self.handle_message(&socket, &buf[..len], src_addr).await {
                        debug!("TURN: error handling message from {src_addr}: {e}");
                    }
                }
                Err(e) => {
                    warn!("TURN: recv_from error: {e}");
                }
            }
        }
    }

    async fn handle_message(
        &self,
        socket: &Arc<UdpSocket>,
        data: &[u8],
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if data.len() >= 4 {
            let first_two = u16::from_be_bytes([data[0], data[1]]);
            if (0x4000..=0x7FFF).contains(&first_two) {
                return self.handle_channel_data(data, src_addr).await;
            }
        }

        let msg = StunMessage::decode(data)?;

        match msg.message_type {
            MessageType::BindingRequest => self.handle_binding(socket, &msg, src_addr).await,
            MessageType::Allocate => self.handle_allocate(socket, &msg, src_addr).await,
            MessageType::Refresh => self.handle_refresh(socket, &msg, src_addr).await,
            MessageType::CreatePermission => {
                self.handle_create_permission(socket, &msg, src_addr).await
            }
            MessageType::ChannelBind => self.handle_channel_bind(socket, &msg, src_addr).await,
            MessageType::SendIndication => self.handle_send_indication(&msg, src_addr).await,
            _ => {
                debug!("TURN: ignoring message type {:?} from {src_addr}", msg.message_type);
                Ok(())
            }
        }
    }

    async fn handle_binding(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        let response = StunMessage {
            message_type: MessageType::BindingResponse,
            transaction_id: request.transaction_id,
            attributes: vec![
                StunAttribute::XorMappedAddress(src_addr),
                StunAttribute::MappedAddress(src_addr),
            ],
        };
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN binding response send: {e}")))?;
        Ok(())
    }

    async fn handle_allocate(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        let username = match self.authenticate(request) {
            Ok(u) => u,
            Err(e) => {
                self.send_error(
                    socket,
                    request,
                    src_addr,
                    MessageType::AllocateError,
                    401,
                    "Unauthorized",
                )
                .await?;
                let mut stats = self.stats.write().await;
                stats.auth_failures += 1;
                return Err(e);
            }
        };

        {
            let allocs = self.allocations.read().await;
            if allocs.contains_key(&src_addr) {
                self.send_error(
                    socket,
                    request,
                    src_addr,
                    MessageType::AllocateError,
                    437,
                    "Allocation mismatch",
                )
                .await?;
                return Ok(());
            }
        }

        let lifetime = TurnAttrs::parse_lifetime(request).unwrap_or(DEFAULT_ALLOCATION_LIFETIME);
        let lifetime = lifetime.min(MAX_ALLOCATION_LIFETIME);

        let relay_socket = UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR)
            .await
            .map_err(|e| StunError::Network(format!("relay socket bind: {e}")))?;
        let relay_addr = relay_socket
            .local_addr()
            .map_err(|e| StunError::Network(format!("relay local_addr: {e}")))?;

        info!(
            "TURN: allocation for {username}@{src_addr} → relay {relay_addr} (lifetime={lifetime}s)"
        );

        let relay_socket = Arc::new(relay_socket);

        let fwd_socket = Arc::clone(socket);
        let fwd_relay = Arc::clone(&relay_socket);
        let fwd_client = src_addr;
        let fwd_allocs = Arc::clone(&self.allocations);
        let fwd_stats = Arc::clone(&self.stats);
        tokio::spawn(async move {
            Self::relay_forward_loop(fwd_relay, fwd_socket, fwd_client, fwd_allocs, fwd_stats)
                .await;
        });

        {
            let mut allocs = self.allocations.write().await;
            allocs.insert(
                src_addr,
                Allocation {
                    username,
                    client_addr: src_addr,
                    relay_socket,
                    relay_addr,
                    permissions: Vec::new(),
                    channels: HashMap::new(),
                    expires_at: Instant::now() + Duration::from_secs(u64::from(lifetime)),
                },
            );
        }

        {
            let mut stats = self.stats.write().await;
            stats.allocations_created += 1;
            stats.active_allocations += 1;
        }

        let response = TurnAttrs::build_allocate_success(request, src_addr, relay_addr, lifetime);
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("allocate response send: {e}")))?;

        Ok(())
    }

    async fn handle_refresh(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if self.authenticate(request).is_err() {
            self.send_error(
                socket,
                request,
                src_addr,
                MessageType::RefreshError,
                401,
                "Unauthorized",
            )
            .await?;
            return Ok(());
        }

        let lifetime = TurnAttrs::parse_lifetime(request).unwrap_or(DEFAULT_ALLOCATION_LIFETIME);

        let mut allocs = self.allocations.write().await;
        if let Some(alloc) = allocs.get_mut(&src_addr) {
            if lifetime == 0 {
                info!("TURN: releasing allocation for {src_addr}");
                allocs.remove(&src_addr);
                let mut stats = self.stats.write().await;
                stats.active_allocations = stats.active_allocations.saturating_sub(1);
            } else {
                let clamped = lifetime.min(MAX_ALLOCATION_LIFETIME);
                alloc.expires_at = Instant::now() + Duration::from_secs(u64::from(clamped));
                debug!("TURN: refreshed allocation for {src_addr} (lifetime={clamped}s)");
            }
        }
        drop(allocs);

        let response = TurnAttrs::build_lifetime_response(
            request,
            MessageType::RefreshSuccess,
            lifetime.min(MAX_ALLOCATION_LIFETIME),
        );
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("refresh response send: {e}")))?;
        Ok(())
    }

    async fn handle_create_permission(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if self.authenticate(request).is_err() {
            return Ok(());
        }

        if let Some(ip) = TurnAttrs::parse_peer_addr(request).map(|a| a.ip()) {
            let mut allocs = self.allocations.write().await;
            if let Some(alloc) = allocs.get_mut(&src_addr) {
                if !alloc.permissions.contains(&ip) {
                    alloc.permissions.push(ip);
                }
                debug!("TURN: permission granted for {ip} on {src_addr}");
            }
        }

        let response = StunMessage {
            message_type: MessageType::CreatePermissionSuccess,
            transaction_id: request.transaction_id,
            attributes: Vec::new(),
        };
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("permission response send: {e}")))?;
        Ok(())
    }

    async fn handle_channel_bind(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if self.authenticate(request).is_err() {
            return Ok(());
        }

        let channel = TurnAttrs::parse_channel(request);
        let peer_addr = TurnAttrs::parse_peer_addr(request);

        if let (Some(ch), Some(peer)) = (channel, peer_addr) {
            let mut allocs = self.allocations.write().await;
            if let Some(alloc) = allocs.get_mut(&src_addr) {
                alloc.channels.insert(ch, peer);
                debug!("TURN: channel 0x{ch:04x} bound to {peer} for {src_addr}");
            }
        }

        let response = StunMessage {
            message_type: MessageType::ChannelBindSuccess,
            transaction_id: request.transaction_id,
            attributes: Vec::new(),
        };
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("channel bind response send: {e}")))?;
        Ok(())
    }

    async fn handle_send_indication(
        &self,
        msg: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        let Some(peer) = TurnAttrs::parse_peer_addr(msg) else {
            debug!("TURN: SendIndication missing XOR-PEER-ADDRESS from {src_addr}");
            return Ok(());
        };
        let Some(payload) = TurnAttrs::parse_data(msg) else {
            debug!("TURN: SendIndication missing DATA from {src_addr}");
            return Ok(());
        };

        let allocs = self.allocations.read().await;
        let Some(alloc) = allocs.get(&src_addr) else {
            debug!("TURN: SendIndication from {src_addr} with no allocation");
            return Ok(());
        };

        if !alloc.permissions.contains(&peer.ip()) {
            debug!("TURN: SendIndication to unpermitted peer {peer} from {src_addr}");
            return Ok(());
        }

        if alloc.relay_socket.send_to(payload, peer).await.is_ok() {
            let mut s = self.stats.write().await;
            s.packets_relayed += 1;
            s.bytes_relayed += payload.len() as u64;
        }

        Ok(())
    }

    async fn handle_channel_data(&self, data: &[u8], src_addr: SocketAddr) -> StunResult<()> {
        if data.len() < 4 {
            return Ok(());
        }

        let channel = u16::from_be_bytes([data[0], data[1]]);
        let length = u16::from_be_bytes([data[2], data[3]]) as usize;

        if data.len() < 4 + length {
            debug!("TURN: ChannelData truncated from {src_addr}");
            return Ok(());
        }

        let payload = &data[4..4 + length];

        let allocs = self.allocations.read().await;
        let Some(alloc) = allocs.get(&src_addr) else {
            debug!("TURN: ChannelData from {src_addr} with no allocation");
            return Ok(());
        };

        let Some(&peer_addr) = alloc.channels.get(&channel) else {
            debug!("TURN: ChannelData for unbound channel 0x{channel:04x} from {src_addr}");
            return Ok(());
        };

        if !alloc.permissions.contains(&peer_addr.ip()) {
            debug!("TURN: ChannelData to unpermitted peer {peer_addr} from {src_addr}");
            return Ok(());
        }

        if alloc.relay_socket.send_to(payload, peer_addr).await.is_ok() {
            let mut s = self.stats.write().await;
            s.packets_relayed += 1;
            s.bytes_relayed += payload.len() as u64;
        }

        Ok(())
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    fn authenticate(&self, msg: &StunMessage) -> StunResult<String> {
        let username = msg
            .attributes
            .iter()
            .find_map(|a| {
                if let StunAttribute::Username(u) = a {
                    Some(u.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| StunError::Config("Missing USERNAME attribute".to_string()))?;

        let _key = self
            .credentials
            .get_key(&username)
            .ok_or_else(|| StunError::Config(format!("Unknown user: {username}")))?;

        Ok(username)
    }

    /// Send a STUN error response with a proper ERROR-CODE attribute (RFC 5389 §15.6).
    async fn send_error(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        dst: SocketAddr,
        error_type: MessageType,
        code: u16,
        reason: &str,
    ) -> StunResult<()> {
        let response = StunMessage {
            message_type: error_type,
            transaction_id: request.transaction_id,
            attributes: vec![TurnAttrs::build_error_code(code, reason)],
        };
        let wire = response.encode();
        socket
            .send_to(&wire, dst)
            .await
            .map_err(|e| StunError::Network(format!("error response send: {e}")))?;
        Ok(())
    }

    /// Forward data from relay socket back to the client via the main socket.
    async fn relay_forward_loop(
        relay_socket: Arc<UdpSocket>,
        main_socket: Arc<UdpSocket>,
        client_addr: SocketAddr,
        allocations: Arc<RwLock<HashMap<SocketAddr, Allocation>>>,
        stats: Arc<RwLock<TurnRelayStats>>,
    ) {
        let mut buf = vec![0u8; 65536];
        while let Ok((len, peer_addr)) = relay_socket.recv_from(&mut buf).await {
            let frame = {
                let allocs = allocations.read().await;
                let Some(alloc) = allocs.get(&client_addr) else {
                    break;
                };

                if !alloc.permissions.contains(&peer_addr.ip()) {
                    debug!("TURN: dropping packet from unpermitted peer {peer_addr}");
                    continue;
                }

                let channel =
                    alloc.channels.iter().find(|&(_, &addr)| addr == peer_addr).map(|(&ch, _)| ch);

                if let Some(ch) = channel {
                    bytes::Bytes::from(TurnAttrs::build_channel_data(ch, &buf[..len]))
                } else {
                    TurnAttrs::build_data_indication(peer_addr, &buf[..len])
                }
            };

            if main_socket.send_to(&frame, client_addr).await.is_ok() {
                let mut s = stats.write().await;
                s.packets_relayed += 1;
                s.bytes_relayed += len as u64;
            }
        }
    }

    /// Periodic cleanup of expired allocations.
    async fn cleanup_loop(
        allocations: Arc<RwLock<HashMap<SocketAddr, Allocation>>>,
        stats: Arc<RwLock<TurnRelayStats>>,
    ) {
        loop {
            tokio::time::sleep(CLEANUP_INTERVAL).await;
            let now = Instant::now();
            let mut allocs = allocations.write().await;
            let before = allocs.len();
            allocs.retain(|addr, alloc| {
                if alloc.expires_at <= now {
                    info!("TURN: expired allocation for {addr} (user={})", alloc.username);
                    false
                } else {
                    true
                }
            });
            let removed = before - allocs.len();
            if removed > 0 {
                drop(allocs);
                let mut s = stats.write().await;
                s.active_allocations = s.active_allocations.saturating_sub(removed as u64);
            }
        }
    }
}

#[cfg(test)]
#[path = "turn_server_tests.rs"]
mod tests;
