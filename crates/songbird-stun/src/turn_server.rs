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
//! Credentials are verified via MESSAGE-INTEGRITY (HMAC-SHA1) using
//! `BearDog`-derived beacon-tier keys. The server holds a `CredentialStore`
//! that maps usernames to shared secrets.

use crate::error::{StunError, StunResult};
use crate::message::{MAGIC_COOKIE, MessageType, StunAttribute, StunMessage};
use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
}

impl CredentialStore for StaticCredentialStore {
    fn get_key(&self, username: &str) -> Option<Vec<u8>> {
        self.credentials.get(username).cloned()
    }
}

/// A single TURN allocation.
#[derive(Debug)]
#[allow(
    dead_code,
    reason = "fields keep relay resources alive and model RFC 5766 allocation state"
)]
struct Allocation {
    /// Username that owns this allocation.
    username: String,
    /// Client transport address (where the client sends from).
    client_addr: SocketAddr,
    /// Relay socket (bound to an ephemeral port, forwards to/from peers).
    relay_socket: Arc<UdpSocket>,
    /// Relay address (the public-facing address peers send to).
    relay_addr: SocketAddr,
    /// Permitted peer IP addresses.
    permissions: Vec<IpAddr>,
    /// Channel bindings: channel number → peer address.
    channels: HashMap<u16, SocketAddr>,
    /// When this allocation expires.
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

        // Spawn cleanup task
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
        // Try to decode as STUN/TURN message
        let msg = StunMessage::decode(data)?;

        match msg.message_type {
            MessageType::BindingRequest => self.handle_binding(socket, &msg, src_addr).await,
            MessageType::Allocate => self.handle_allocate(socket, &msg, src_addr).await,
            MessageType::Refresh => self.handle_refresh(socket, &msg, src_addr).await,
            MessageType::CreatePermission => {
                self.handle_create_permission(socket, &msg, src_addr).await
            }
            MessageType::ChannelBind => self.handle_channel_bind(socket, &msg, src_addr).await,
            _ => {
                debug!("TURN: ignoring message type {:?} from {src_addr}", msg.message_type);
                Ok(())
            }
        }
    }

    /// STUN Binding compatibility — respond with XOR-MAPPED-ADDRESS.
    async fn handle_binding(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        let mut response = StunMessage {
            message_type: MessageType::BindingResponse,
            transaction_id: request.transaction_id,
            attributes: vec![
                StunAttribute::XorMappedAddress(src_addr),
                StunAttribute::MappedAddress(src_addr),
            ],
        };
        let _ = &mut response;
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN binding response send: {e}")))?;
        Ok(())
    }

    /// Handle Allocate request — create a relay allocation.
    async fn handle_allocate(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        // Authenticate
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

        // Check for existing allocation
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

        // Parse requested lifetime
        let lifetime = Self::parse_lifetime(request).unwrap_or(DEFAULT_ALLOCATION_LIFETIME);
        let lifetime = lifetime.min(MAX_ALLOCATION_LIFETIME);

        // Bind relay socket
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

        // Spawn relay forwarder
        let fwd_socket = Arc::clone(socket);
        let fwd_relay = Arc::clone(&relay_socket);
        let fwd_client = src_addr;
        let fwd_allocs = Arc::clone(&self.allocations);
        let fwd_stats = Arc::clone(&self.stats);
        tokio::spawn(async move {
            Self::relay_forward_loop(fwd_relay, fwd_socket, fwd_client, fwd_allocs, fwd_stats)
                .await;
        });

        // Store allocation
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

        // Build success response
        let server_addr = socket.local_addr().unwrap_or(self.bind_addr);
        let response =
            Self::build_allocate_success(request, src_addr, relay_addr, server_addr, lifetime);
        let wire = response.encode();
        socket
            .send_to(&wire, src_addr)
            .await
            .map_err(|e| StunError::Network(format!("allocate response send: {e}")))?;

        Ok(())
    }

    /// Handle Refresh — extend or release allocation.
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
                MessageType::RefreshSuccess,
                401,
                "Unauthorized",
            )
            .await?;
            return Ok(());
        }

        let lifetime = Self::parse_lifetime(request).unwrap_or(DEFAULT_ALLOCATION_LIFETIME);

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

        let response = Self::build_lifetime_response(
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

    /// Handle CreatePermission — allow a peer IP through the relay.
    async fn handle_create_permission(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if self.authenticate(request).is_err() {
            return Ok(());
        }

        // Extract XOR-PEER-ADDRESS from Unknown(0x0012, ...)
        let peer_ip = request.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(0x0012, data) = attr {
                StunAttribute::decode_address(data, Some(MAGIC_COOKIE), &request.transaction_id)
                    .ok()
                    .map(|addr| addr.ip())
            } else {
                None
            }
        });

        if let Some(ip) = peer_ip {
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

    /// Handle ChannelBind — map a channel number to a peer address.
    async fn handle_channel_bind(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        src_addr: SocketAddr,
    ) -> StunResult<()> {
        if self.authenticate(request).is_err() {
            return Ok(());
        }

        let channel = request.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(0x000C, data) = attr {
                if data.len() >= 2 {
                    Some(u16::from_be_bytes([data[0], data[1]]))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let peer_addr = request.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(0x0012, data) = attr {
                StunAttribute::decode_address(data, Some(MAGIC_COOKIE), &request.transaction_id)
                    .ok()
            } else {
                None
            }
        });

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

    // --- Internal helpers ---

    /// Verify MESSAGE-INTEGRITY against the credential store.
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

    fn parse_lifetime(msg: &StunMessage) -> Option<u32> {
        msg.attributes.iter().find_map(|attr| {
            if let StunAttribute::Unknown(0x000D, data) = attr {
                if data.len() >= 4 {
                    Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn build_allocate_success(
        request: &StunMessage,
        client_addr: SocketAddr,
        relay_addr: SocketAddr,
        server_addr: SocketAddr,
        lifetime: u32,
    ) -> StunMessage {
        let mut attrs: Vec<StunAttribute> = Vec::new();

        // XOR-MAPPED-ADDRESS (client's reflexive address)
        attrs.push(StunAttribute::XorMappedAddress(client_addr));

        // XOR-RELAYED-ADDRESS (0x0016)
        let mut relay_buf = BytesMut::new();
        relay_buf.put_u8(0); // reserved
        match relay_addr {
            SocketAddr::V4(_) => relay_buf.put_u8(0x01),
            SocketAddr::V6(_) => relay_buf.put_u8(0x02),
        }
        let port = relay_addr.port() ^ (MAGIC_COOKIE >> 16) as u16;
        relay_buf.put_u16(port);
        match relay_addr.ip() {
            IpAddr::V4(ip) => {
                let xored = u32::from(ip) ^ MAGIC_COOKIE;
                relay_buf.put_u32(xored);
            }
            IpAddr::V6(ip) => {
                let mut xor_pad = [0u8; 16];
                xor_pad[..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
                xor_pad[4..].copy_from_slice(&request.transaction_id);
                let raw = ip.octets();
                for i in 0..16 {
                    relay_buf.put_u8(raw[i] ^ xor_pad[i]);
                }
            }
        }
        attrs.push(StunAttribute::Unknown(0x0016, relay_buf.freeze()));

        // LIFETIME (0x000D)
        let mut lt_buf = BytesMut::with_capacity(4);
        lt_buf.put_u32(lifetime);
        attrs.push(StunAttribute::Unknown(0x000D, lt_buf.freeze()));

        let _ = server_addr; // reserved for SOFTWARE attribute if needed

        StunMessage {
            message_type: MessageType::AllocateSuccess,
            transaction_id: request.transaction_id,
            attributes: attrs,
        }
    }

    fn build_lifetime_response(
        request: &StunMessage,
        msg_type: MessageType,
        lifetime: u32,
    ) -> StunMessage {
        let mut lt_buf = BytesMut::with_capacity(4);
        lt_buf.put_u32(lifetime);

        StunMessage {
            message_type: msg_type,
            transaction_id: request.transaction_id,
            attributes: vec![StunAttribute::Unknown(0x000D, lt_buf.freeze())],
        }
    }

    async fn send_error(
        &self,
        socket: &Arc<UdpSocket>,
        request: &StunMessage,
        dst: SocketAddr,
        error_type: MessageType,
        _code: u16,
        _reason: &str,
    ) -> StunResult<()> {
        let response = StunMessage {
            message_type: error_type,
            transaction_id: request.transaction_id,
            attributes: Vec::new(),
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
            let permitted = {
                let allocs = allocations.read().await;
                allocs.get(&client_addr).is_some_and(|a| a.permissions.contains(&peer_addr.ip()))
            };

            if !permitted {
                debug!("TURN: dropping packet from unpermitted peer {peer_addr}");
                continue;
            }

            if main_socket.send_to(&buf[..len], client_addr).await.is_ok() {
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
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::message::StunMessage;
    use crate::turn::TurnClient;
    use crate::types::StunCredentials;

    fn test_credentials() -> Arc<dyn CredentialStore> {
        let mut store = StaticCredentialStore::new();
        store.insert("testuser".to_string(), b"testkey123".to_vec());
        Arc::new(store)
    }

    #[tokio::test]
    async fn server_creation() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let server = TurnRelayServer::new(addr, test_credentials());
        let stats = server.stats().await;
        assert_eq!(stats.allocations_created, 0);
    }

    #[tokio::test]
    async fn static_credential_store_get_key() {
        let mut store = StaticCredentialStore::new();
        store.insert("user1".to_string(), b"key1".to_vec());
        assert!(store.get_key("user1").is_some());
        assert!(store.get_key("missing").is_none());
    }

    #[tokio::test]
    async fn turn_server_allocate_and_refresh() {
        let creds = test_credentials();
        let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

        let (tx, rx) = tokio::sync::oneshot::channel();
        let server_handle = tokio::spawn(async move {
            let _ = server.run_with_ready(tx).await;
        });

        let server_addr = rx.await.expect("server ready");

        // Create TURN client
        let client_creds = StunCredentials {
            username: "testuser".to_string(),
            key: b"testkey123".to_vec(),
        };
        let client = TurnClient::new(server_addr, client_creds);

        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Allocate
        let alloc = client.allocate(&socket).await.expect("allocate should succeed");
        assert!(alloc.relay_addr.port() > 0);
        assert!(alloc.lifetime_secs > 0);

        // Refresh
        let new_lifetime = client.refresh(&socket, 300).await.expect("refresh should succeed");
        assert!(new_lifetime > 0);

        server_handle.abort();
    }

    #[tokio::test]
    async fn turn_server_binding_request() {
        let creds = test_credentials();
        let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

        let (tx, rx) = tokio::sync::oneshot::channel();
        let server_handle = tokio::spawn(async move {
            let _ = server.run_with_ready(tx).await;
        });

        let server_addr = rx.await.expect("server ready");

        // Send a STUN Binding Request (no auth needed)
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let request = StunMessage::new_binding_request();
        let wire = request.encode();
        socket.send_to(&wire, server_addr).await.unwrap();

        let mut buf = [0u8; 1500];
        let (len, _) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf))
            .await
            .expect("timeout")
            .expect("recv");

        let response = StunMessage::decode(&buf[..len]).expect("decode");
        assert_eq!(response.message_type, MessageType::BindingResponse);
        assert!(response.get_xor_mapped_address().is_some());

        server_handle.abort();
    }

    #[tokio::test]
    async fn turn_server_rejects_unknown_user() {
        let creds = test_credentials();
        let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), creds);

        let (tx, rx) = tokio::sync::oneshot::channel();
        let server_handle = tokio::spawn(async move {
            let _ = server.run_with_ready(tx).await;
        });

        let server_addr = rx.await.expect("server ready");

        let bad_creds = StunCredentials {
            username: "baduser".to_string(),
            key: b"wrongkey".to_vec(),
        };
        let client = TurnClient::new(server_addr, bad_creds);
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let result = client.allocate(&socket).await;
        assert!(result.is_err(), "should reject unknown user");

        server_handle.abort();
    }

    #[tokio::test]
    async fn turn_relay_stats_initial() {
        let stats = TurnRelayStats::default();
        assert_eq!(stats.allocations_created, 0);
        assert_eq!(stats.active_allocations, 0);
        assert_eq!(stats.packets_relayed, 0);
        assert_eq!(stats.bytes_relayed, 0);
        assert_eq!(stats.auth_failures, 0);
        assert!(stats.start_time.is_none());
    }
}
