// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Anonymous Discovery Broadcaster
//!
//! This module contains the broadcasting logic for anonymous discovery via UDP multicast.
//!
//! ## Contents
//! - `AnonymousDiscoveryBroadcaster` - Broadcasts discovery messages
//! - Multicast setup and network interface detection
//! - V2.1 and v3.0 protocol support
//! - Optional `BirdSong` encryption integration

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use super::messages::TransportEndpointMessage;
use super::protocol;
use super::scheduling;

/// Anonymous discovery broadcaster
///
/// Broadcasts anonymous discovery messages over UDP multicast to find other towers.
/// Supports both v2.1 (session-based) and v3.0 (node-identity-based) protocols.
///
/// Uses UDP multicast (224.0.0.251) by default for reliable cross-router discovery.
/// Falls back to broadcast and known peers for maximum compatibility.
pub struct AnonymousDiscoveryBroadcaster {
    /// Protocol version ("2.1" or "3.0")
    pub(crate) version: String,

    /// Stable node ID (v3.0 only)
    pub(crate) node_id: Option<String>,

    /// Node name (v3.0 only)
    pub(crate) node_name: Option<String>,

    /// All endpoints (v3.0 only)
    pub(crate) endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities to advertise
    pub(crate) capabilities: Vec<String>,

    /// Protocols supported (v2.1 fallback)
    pub(crate) protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening (v2.1 fallback)
    pub(crate) port: u16,

    /// Multicast/broadcast addresses to send to
    /// Includes multicast (224.0.0.251) + subnet broadcast fallback for cross-interface discovery
    pub(crate) broadcast_addresses: Vec<SocketAddr>,

    /// Known peer addresses for direct discovery (bypasses multicast)
    pub(crate) known_peers: Vec<SocketAddr>,

    /// Broadcast interval in seconds
    pub(crate) interval_secs: u64,

    /// Identity tags (v3.14.0 - tag-based identity)
    /// Opaque strings we broadcast. We don't interpret them!
    /// Format: `{provider}:{type}:{value}` (e.g., `crypto:family:my-family`)
    pub(crate) tags: Option<Vec<String>>,

    /// Identity attestations from security provider (CRITICAL FIX - Jan 3, 2026)
    ///
    /// These are provided by the security capability provider (e.g., `security provider`) on startup
    /// and enable genetic lineage auto-trust. MUST be included for federation to work!
    pub(crate) identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// `BirdSong` encryption processor (optional) - NEW (Jan 3, 2026)
    pub(crate) birdsong: Option<Arc<crate::birdsong::BirdSongProcessor>>,

    /// Statistics tracker for observability (optional) - NEW (Jan 5, 2026)
    pub(crate) stats: Option<Arc<crate::discovery_stats::DiscoveryStats>>,
}

impl AnonymousDiscoveryBroadcaster {
    /// Create a new anonymous discovery broadcaster (v2.1 - backward compatible)
    #[must_use]
    pub fn new(
        capabilities: Vec<String>,
        protocols: Vec<String>,
        port: u16,
        broadcast_addresses: Vec<SocketAddr>,
        interval_secs: u64,
    ) -> Self {
        Self {
            version: "2.1".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities,
            protocols,
            port,
            broadcast_addresses,
            known_peers: Vec::new(),
            interval_secs,
            tags: None, // NEW (v3.14.0)
            identity_attestations: None,
            birdsong: None,
            stats: None,
        }
    }

    /// Create a new v3.0 broadcaster with node identity and multiple endpoints
    #[must_use]
    pub fn new_v3(
        node_id: String,
        node_name: String,
        endpoints: Vec<TransportEndpointMessage>,
        capabilities: Vec<String>,
        broadcast_addresses: Vec<SocketAddr>,
        interval_secs: u64,
    ) -> Self {
        // Extract primary endpoint for v2.1 fallback
        let primary = endpoints.first();

        // Extract port from address (format: "IP:port")
        let port = primary
            .and_then(|e| e.address.split(':').nth(1))
            .and_then(|p| p.parse().ok())
            .unwrap_or_else(protocol::default_v3_fallback_port);

        let protocols = primary.map_or_else(|| vec!["https".to_string()], |e| e.protocols.clone());

        Self {
            version: "3.0".to_string(),
            node_id: Some(node_id),
            node_name: Some(node_name),
            endpoints: Some(endpoints),
            capabilities,
            protocols,
            port,
            broadcast_addresses,
            known_peers: Vec::new(),
            interval_secs,
            tags: None, // NEW (v3.14.0)
            identity_attestations: None,
            birdsong: None,
            stats: None,
        }
    }

    /// Add known peer addresses for direct discovery
    #[must_use]
    pub fn with_known_peers(mut self, peers: Vec<SocketAddr>) -> Self {
        self.known_peers = peers;
        self
    }

    /// Set identity attestations from security provider (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Adds identity attestations for genetic lineage auto-trust. This should be called
    /// after querying the security provider (e.g., `security provider`) for our node's identity.
    #[must_use]
    pub fn with_identity_attestations(
        mut self,
        attestations: Vec<crate::IdentityAttestation>,
    ) -> Self {
        self.identity_attestations = Some(attestations);
        self
    }

    /// Set identity tags (v3.14.0 - tag-based identity)
    ///
    /// Tags are opaque strings we broadcast. We don't interpret them!
    /// Format: `{provider}:{type}:{value}`
    /// Example: `crypto:family:my-family`
    ///
    /// Security providers (`security provider`) interpret tag meaning.
    #[must_use]
    pub fn with_identity_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = if tags.is_empty() {
            None
        } else {
            Some(tags)
        };
        self
    }

    /// Enable statistics tracking for observability (NEW - Jan 5, 2026)
    #[must_use]
    pub fn with_stats(mut self, stats: Arc<crate::discovery_stats::DiscoveryStats>) -> Self {
        stats.set_broadcasting(true);
        self.stats = Some(stats);
        self
    }

    /// Enable `BirdSong` encrypted discovery (NEW - Jan 3, 2026)
    ///
    /// Adds `BirdSong` encryption for privacy-preserving discovery.
    /// Only same-family peers can decrypt discovery packets.
    #[must_use]
    pub fn with_birdsong(mut self, processor: Arc<crate::birdsong::BirdSongProcessor>) -> Self {
        info!("🎵 BirdSong encryption enabled for discovery broadcaster");
        info!("   Status: {}", processor.status());
        self.birdsong = Some(processor);
        self
    }

    /// Start broadcasting discovery messages
    ///
    /// This runs indefinitely, broadcasting every `interval_secs` seconds.
    /// Sends v2.1 or v3.0 messages based on broadcaster configuration.
    ///
    /// Uses UDP multicast for reliable cross-router discovery, with fallback to:
    /// - Broadcast addresses (for local subnet)
    /// - Known peers (for guaranteed delivery)
    #[allow(
        clippy::too_many_lines,
        reason = "sequential multicast, broadcast, and peer fallback discovery loop"
    )]
    pub async fn start_broadcasting(&self) -> Result<(), std::io::Error> {
        info!("🌐 Starting anonymous discovery broadcaster");
        info!("   Version: {}", self.version);
        if let Some(ref node_id) = self.node_id {
            info!("   Node ID: {}", node_id);
        }
        if let Some(ref node_name) = self.node_name {
            info!("   Node Name: {}", node_name);
        }
        if let Some(ref endpoints) = self.endpoints {
            info!("   Endpoints: {} transport paths", endpoints.len());
            for (i, endpoint) in endpoints.iter().enumerate() {
                info!(
                    "     {}. {} ({}, preference {})",
                    i + 1,
                    endpoint.interface_type,
                    endpoint.address,
                    endpoint.preference
                );
            }
        }
        info!("   Capabilities: {:?}", self.capabilities);
        info!("   Protocols: {:?}", self.protocols);
        info!("   Multicast/broadcast addresses: {:?}", self.broadcast_addresses);
        if !self.known_peers.is_empty() {
            info!("   Known peers: {:?}", self.known_peers);
        }

        // ✅ v3.14.2: Log identity tags for verification
        if let Some(ref tags) = self.tags {
            info!("   Identity Tags: {} tags configured", tags.len());
            for tag in tags {
                info!("     📋 {}", tag);
            }
        } else {
            info!("   Identity Tags: None (peers won't see our family)");
        }
        info!("   Interval: {}s", self.interval_secs);

        // Create UDP socket with multicast support using socket2
        use socket2::{Domain, Protocol, Socket, Type};

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_broadcast(true)?; // Keep for fallback to broadcast
        socket.set_multicast_ttl_v4(255)?; // Multicast TTL for cross-router
        socket.set_nonblocking(true)?;

        // Convert to tokio UdpSocket
        let std_socket: std::net::UdpSocket = socket.into();
        let socket = UdpSocket::from_std(std_socket)?;

        info!("✅ Anonymous discovery broadcaster started (multicast-enabled)");

        let mut interval = scheduling::broadcast_interval(self.interval_secs);

        loop {
            interval.tick().await;

            let prepared = match protocol::build_discovery_plaintext(
                &self.version,
                self.node_id.clone(),
                self.node_name.clone(),
                self.endpoints.clone(),
                self.capabilities.clone(),
                self.protocols.clone(),
                self.port,
                self.identity_attestations.clone(),
                self.tags.clone(),
            ) {
                Ok(p) => p,
                Err(e) => {
                    protocol::log_serialize_error(&e);
                    continue;
                }
            };

            let session_id = prepared.session_id;
            let mut bytes = prepared.bytes;

            if let Some(ref birdsong) = self.birdsong {
                if birdsong.config().is_dark_forest_active() {
                    match protocol::build_dark_forest_beacon_bytes(
                        self.node_id.clone(),
                        self.endpoints.as_ref(),
                        self.port,
                        &self.capabilities,
                        birdsong,
                    )
                    .await
                    {
                        Ok(beacon_bytes) => {
                            self.send_udp_payload(&socket, &beacon_bytes).await;
                            debug!("🌲 Dark Forest beacon broadcasted");
                            protocol::log_dark_forest_beacon_sent(beacon_bytes.len());

                            if birdsong.config().dual_broadcast {
                                debug!("📢 Dual broadcast: also sending legacy format");
                                bytes =
                                    protocol::encrypt_birdsong_dual_legacy(&bytes, birdsong).await;
                            } else {
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "⚠️  Dark Forest broadcast failed: {}, falling back to legacy",
                                e
                            );
                            bytes = protocol::encrypt_birdsong_after_dark_forest_failure(
                                &bytes, birdsong,
                            )
                            .await;
                        }
                    }
                } else {
                    bytes = protocol::encrypt_birdsong_legacy(&bytes, birdsong).await;
                }
            }

            self.send_udp_payload(&socket, &bytes).await;
            debug!("📡 Broadcast discovery message (session: {})", session_id);
        }
    }

    async fn send_udp_payload(&self, socket: &UdpSocket, bytes: &[u8]) {
        for addr in &self.broadcast_addresses {
            match socket.send_to(bytes, addr).await {
                Ok(sent) => {
                    debug!("Multicast {} bytes to {}", sent, addr);
                    if let Some(ref stats) = self.stats {
                        stats.record_broadcast();
                    }
                }
                Err(e) => {
                    warn!("Failed to multicast to {}: {}", addr, e);
                    if let Some(ref stats) = self.stats {
                        stats.record_error();
                    }
                }
            }
        }

        for peer in &self.known_peers {
            match socket.send_to(bytes, peer).await {
                Ok(sent) => {
                    debug!("Direct send {} bytes to known peer {}", sent, peer);
                    if let Some(ref stats) = self.stats {
                        stats.record_broadcast();
                    }
                }
                Err(e) => {
                    warn!("Failed to send to known peer {}: {}", peer, e);
                    if let Some(ref stats) = self.stats {
                        stats.record_error();
                    }
                }
            }
        }
    }
}
