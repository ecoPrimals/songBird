// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Anonymous Discovery Listener
//!
//! This module contains the listening logic for anonymous discovery via UDP multicast.
//!
//! ## Contents
//! - `AnonymousDiscoveryListener` - Listens for discovery messages
//! - Multicast group joining
//! - Message processing & peer registry
//! - Self-filtering logic
//! - Optional `BirdSong` decryption

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::messages::AnonymousDiscoveryMessage;
use super::peer::DiscoveredPeer;

/// Anonymous discovery listener
///
/// Listens for anonymous discovery messages from other towers.
/// Joins multicast group for reliable reception across routers.
///
/// **NEW (Jan 3, 2026)**: Optional `BirdSong` decryption for privacy-preserving discovery.
pub struct AnonymousDiscoveryListener {
    /// Port to listen on (typically 2300)
    port: u16,

    /// Multicast group to join (e.g., 224.0.0.251)
    multicast_addr: Option<Ipv4Addr>,

    /// Discovered peers (`session_id` -> peer info)
    peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,

    /// Peer timeout in seconds (default: 60)
    peer_timeout_secs: u64,

    /// `BirdSong` decryption processor (optional)
    birdsong: Option<Arc<crate::birdsong::BirdSongProcessor>>,

    /// Our own `node_id` for self-filtering (v3.10.2 - Jan 5, 2026)
    ///
    /// Used to filter out our own discovery broadcasts to prevent self-discovery.
    /// Critical for multi-instance deployments where multiple towers run on same machine.
    node_id: Option<String>,

    /// Statistics tracker for observability (optional) - NEW (Jan 5, 2026)
    stats: Option<Arc<crate::discovery_stats::DiscoveryStats>>,
}

impl AnonymousDiscoveryListener {
    /// Create a new anonymous discovery listener
    #[must_use]
    pub fn new(port: u16, peer_timeout_secs: u64) -> Self {
        Self {
            port,
            multicast_addr: Some(Ipv4Addr::new(224, 0, 0, 251)), // mDNS multicast group
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_timeout_secs,
            birdsong: None,
            stats: None,
            node_id: None,
        }
    }

    /// Create a listener without multicast (broadcast-only fallback)
    #[must_use]
    pub fn new_broadcast_only(port: u16, peer_timeout_secs: u64) -> Self {
        Self {
            port,
            multicast_addr: None,
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_timeout_secs,
            birdsong: None,
            stats: None,
            node_id: None,
        }
    }

    /// Set node ID for self-filtering (v3.10.2 - Jan 5, 2026)
    ///
    /// Enables filtering out our own discovery broadcasts to prevent self-discovery.
    /// Critical for multi-instance deployments (tower1, tower2, etc. on same machine).
    #[must_use]
    pub fn with_node_id(mut self, node_id: String) -> Self {
        self.node_id = Some(node_id);
        self
    }

    /// Returns `true` when this listener would skip `message` as its own broadcast (`node_id` match).
    ///
    /// Exposed for unit tests that assert self-filtering without UDP. Production receive path uses
    /// the same condition before inserting into `peers`.
    #[cfg(test)]
    #[must_use]
    pub fn would_skip_as_own_broadcast(
        &self,
        message: &super::messages::AnonymousDiscoveryMessage,
    ) -> bool {
        if let (Some(my_node_id), Some(peer_node_id)) = (&self.node_id, &message.node_id) {
            my_node_id == peer_node_id
        } else {
            false
        }
    }

    /// Enable `BirdSong` encrypted discovery (NEW - Jan 3, 2026)
    ///
    /// Adds `BirdSong` decryption for privacy-preserving discovery.
    /// Only same-family peers' packets will be decoded.
    #[must_use]
    pub fn with_birdsong(mut self, processor: Arc<crate::birdsong::BirdSongProcessor>) -> Self {
        info!("🎵 BirdSong decryption enabled for discovery listener");
        info!("   Status: {}", processor.status());
        self.birdsong = Some(processor);
        self
    }

    /// Enable statistics tracking for observability (NEW - Jan 5, 2026)
    #[must_use]
    pub fn with_stats(mut self, stats: Arc<crate::discovery_stats::DiscoveryStats>) -> Self {
        stats.set_listening(true);
        self.stats = Some(stats);
        self
    }

    /// Start listening for discovery messages
    ///
    /// This runs indefinitely, processing incoming discovery messages.
    /// Joins multicast group if `multicast_addr` is set.
    pub async fn start_listening(&self) -> Result<(), std::io::Error> {
        info!("👂 Starting anonymous discovery listener on port {}", self.port);
        if let Some(multicast) = self.multicast_addr {
            info!("   Joining multicast group: {}", multicast);
        }

        // Create UDP socket with multicast support using socket2
        use socket2::{Domain, Protocol, Socket, Type};

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        // set_reuse_port is Unix-only (not available on Windows)
        #[cfg(unix)]
        socket.set_reuse_port(true)?;

        // Bind to port on all interfaces
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        socket.bind(&bind_addr.into())?;

        // Join multicast group if configured
        if let Some(multicast_addr) = self.multicast_addr {
            let interface = Ipv4Addr::UNSPECIFIED; // Join on all interfaces
            socket.join_multicast_v4(&multicast_addr, &interface).map_err(|e| {
                error!("Failed to join multicast group {}: {}", multicast_addr, e);
                e
            })?;
            info!("✅ Joined multicast group: {}", multicast_addr);
        }

        // Convert to tokio UdpSocket
        socket.set_nonblocking(true)?;
        let std_socket: std::net::UdpSocket = socket.into();
        let socket = UdpSocket::from_std(std_socket)?;

        info!("✅ Anonymous discovery listener started (multicast-enabled)");

        // Start peer cleanup task
        let peers_clone = Arc::clone(&self.peers);
        let timeout = self.peer_timeout_secs;
        tokio::spawn(async move {
            Self::cleanup_stale_peers(peers_clone, timeout).await;
        });

        // Buffer for incoming messages
        let mut buf = vec![0u8; 65536]; // 64KB buffer

        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    let data = &buf[..len];

                    // 🌲 NEW (Feb 3, 2026): Try Dark Forest beacon first (zero metadata leakage)
                    if let Some(ref birdsong) = self.birdsong
                        && birdsong.config().is_dark_forest_active()
                    {
                        // Try to parse as Dark Forest beacon
                        if let Ok(beacon) =
                            crate::dark_forest_beacon::DarkForestBeacon::from_bytes(data)
                            && beacon.version == 2
                        {
                            debug!(
                                "🌲 Received Dark Forest beacon from {} (size: {} bytes)",
                                addr,
                                data.len()
                            );

                            // Try to decrypt with all known beacon seeds
                            match birdsong.decrypt_dark_forest_beacon(&beacon).await {
                                Ok(Some((payload, beacon_id))) => {
                                    info!(
                                        "🌲✅ Decrypted Dark Forest beacon from {} (beacon_id: {})",
                                        payload.node_id,
                                        hex::encode(&beacon_id[..8.min(beacon_id.len())])
                                    );

                                    // Process Dark Forest beacon payload
                                    self.process_dark_forest_payload(payload, addr).await;

                                    if let Some(ref stats) = self.stats {
                                        stats.record_received();
                                        stats.record_peer_discovered();
                                    }

                                    continue;
                                }
                                Ok(None) => {
                                    // Different beacon family - just noise (EXPECTED)
                                    debug!(
                                        "🌲🔇 Dark Forest beacon from different beacon family (privacy working)"
                                    );
                                    continue;
                                }
                                Err(e) => {
                                    warn!("⚠️  Dark Forest beacon decryption error: {}", e);
                                    // Fall through to try legacy format
                                }
                            }
                        }

                        // Not Dark Forest or decryption failed
                        // Try legacy BirdSongPacket if allowed
                        if !birdsong.config().accept_legacy_format {
                            debug!("Rejecting non-Dark-Forest packet (accept_legacy_format=false)");
                            continue;
                        }
                    }

                    // 🎵 Legacy BirdSong decryption (has plaintext family_id)
                    let data = if let Some(ref birdsong) = self.birdsong {
                        match birdsong.decrypt_packet(data).await {
                            Ok(Some(plaintext)) => {
                                debug!(
                                    "🔓 BirdSong decrypted {} -> {} bytes",
                                    data.len(),
                                    plaintext.len()
                                );
                                if let Some(ref stats) = self.stats {
                                    stats.record_received();
                                }
                                plaintext
                            }
                            Ok(None) => {
                                // Different family - just noise, skip this packet
                                debug!("🔇 BirdSong noise from {} (different family)", addr);
                                continue;
                            }
                            Err(e) => {
                                warn!("⚠️  BirdSong decryption failed from {}: {}", addr, e);
                                if let Some(ref stats) = self.stats {
                                    stats.record_error();
                                }
                                continue;
                            }
                        }
                    } else {
                        if let Some(ref stats) = self.stats {
                            stats.record_received();
                        }
                        data.to_vec()
                    };

                    // Parse discovery message
                    match AnonymousDiscoveryMessage::from_bytes(&data) {
                        Ok(message) => {
                            // Validate message
                            if let Err(e) = message.validate() {
                                warn!("Invalid discovery message from {}: {}", addr, e);
                                if let Some(ref stats) = self.stats {
                                    stats.record_error();
                                }
                                continue;
                            }

                            // CRITICAL FIX (v3.10.2 - Jan 5, 2026): Filter out self-discovery
                            // Prevents towers from discovering their own broadcasts
                            // Critical for multi-instance deployments (tower1, tower2, etc.)
                            if let Some(ref my_node_id) = self.node_id
                                && let Some(ref peer_node_id) = message.node_id
                                && my_node_id == peer_node_id
                            {
                                debug!(
                                    "📭 Skipping own broadcast (self-discovery filtered: {})",
                                    my_node_id
                                );
                                continue;
                            }

                            debug!(
                                "📥 Received discovery from {} (session: {})",
                                addr, message.session_id
                            );

                            // Store peer info with v3.0 support
                            let peer = DiscoveredPeer {
                                session_id: message.session_id.clone(),
                                node_id: message.node_id.clone(),
                                node_name: message.node_name.clone(),
                                endpoints: message.endpoints.clone(),
                                capabilities: message.capabilities.clone(),
                                tags: message.tags.clone(),
                                timestamp: Some(message.timestamp),
                                identity_attestations: message.identity_attestations.clone(),
                                protocols: message.protocols.clone(),
                                port: message.port,
                                address: addr,
                                last_seen: SystemTime::now(),
                                version: message.version.clone(),
                            };

                            // Log with node name if available (v3.0), otherwise session ID (v2.x)
                            let peer_identifier =
                                message.node_name.as_ref().unwrap_or(&message.session_id);

                            info!(
                                "🔍 Discovered peer: {} (v{}, capabilities: {:?}, HTTPS: https://{}:{})",
                                peer_identifier,
                                message.version,
                                message.capabilities,
                                addr.ip(),
                                message.port
                            );

                            if let Some(ref stats) = self.stats {
                                stats.record_peer_discovered();
                            }

                            let mut peers = self.peers.write().await;
                            peers.insert(message.session_id, peer);
                        }
                        Err(e) => {
                            warn!("Failed to parse discovery message from {}: {}", addr, e);
                            if let Some(ref stats) = self.stats {
                                stats.record_error();
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive discovery message: {}", e);
                    if let Some(ref stats) = self.stats {
                        stats.record_error();
                    }
                }
            }
        }
    }

    /// Get all discovered peers
    pub async fn get_peers(&self) -> Vec<DiscoveredPeer> {
        let peers = self.peers.read().await;

        // INFO LOGGING (v3.10.3 - Jan 6, 2026): Diagnose bridge gap at INFO level
        if !peers.is_empty() {
            info!("📊 get_peers() called: {} peers in HashMap", peers.len());
            for (session_id, peer) in peers.iter() {
                let node_name = peer.node_name.as_deref().unwrap_or("unknown");
                let node_id = peer.node_id.as_deref().unwrap_or("no-id");
                info!("   - session:{} | node_id:{} | name:{}", session_id, node_id, node_name);
            }
        }

        if let Some(ref stats) = self.stats {
            stats.set_peers_active(peers.len() as u64);
        }

        peers.values().cloned().collect()
    }

    /// Get peer by session ID
    pub async fn get_peer(&self, session_id: &str) -> Option<DiscoveredPeer> {
        let peers = self.peers.read().await;
        peers.get(session_id).cloned()
    }

    /// Cleanup stale peers (runs periodically)
    async fn cleanup_stale_peers(
        peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,
        timeout_secs: u64,
    ) {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(timeout_secs / 2));

        loop {
            interval.tick().await;

            let now = SystemTime::now();
            let mut peers_lock = peers.write().await;

            // Remove stale peers
            peers_lock.retain(|session_id, peer| {
                let age = now.duration_since(peer.last_seen).unwrap_or_default().as_secs();

                if age > timeout_secs {
                    debug!("🗑️  Removing stale peer: {} (age: {}s)", session_id, age);
                    false
                } else {
                    true
                }
            });
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Dark Forest Beacon Processing (NEW - Feb 3, 2026)
    // ═══════════════════════════════════════════════════════════════════════

    /// Process Dark Forest beacon payload (after successful decryption)
    ///
    /// Converts `BeaconPayload` to `DiscoveredPeer` and stores in peer registry.
    ///
    /// # Arguments
    ///
    /// * `payload` - Decrypted beacon payload
    /// * `addr` - Source address of beacon
    ///
    /// # Privacy Note
    ///
    /// This method is only called AFTER successful decryption, meaning
    /// we share beacon genetics with the sender. Failed decryptions never
    /// reach this point (different beacon family = noise, filtered out).
    async fn process_dark_forest_payload(
        &self,
        payload: crate::dark_forest_beacon::BeaconPayload,
        addr: SocketAddr,
    ) {
        use super::messages::TransportEndpointMessage;

        // Convert endpoints to TransportEndpointMessage format
        let endpoints: Vec<TransportEndpointMessage> = payload
            .endpoints
            .iter()
            .enumerate()
            .map(|(idx, ep)| {
                // Parse endpoint (format: "protocol:address")
                let parts: Vec<&str> = ep.split(':').collect();
                let (interface_type, address) = if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1..].join(":"))
                } else {
                    ("tcp".to_string(), ep.clone())
                };

                TransportEndpointMessage {
                    interface_type,
                    address,
                    protocols: vec!["https".to_string()],
                    preference: u8::try_from(idx).unwrap_or(u8::MAX),
                }
            })
            .collect();

        // Create DiscoveredPeer from Dark Forest beacon payload
        let peer = DiscoveredPeer {
            session_id: payload.session_id.clone(),
            node_id: Some(payload.node_id.clone()),
            node_name: Some(payload.node_id.clone()),
            endpoints: Some(endpoints),
            capabilities: Vec::new(), // Capabilities hash only, full list exchanged later
            tags: None,
            timestamp: Some(payload.created_at),
            identity_attestations: None, // Exchange after trust establishment
            protocols: vec!["https".to_string()],
            port: 8080, // Default
            address: addr,
            last_seen: SystemTime::now(),
            version: "dark_forest_v2".to_string(),
        };

        info!(
            "🌲 Registered peer from Dark Forest: {} (session: {}, {} endpoints)",
            payload.node_id,
            payload.session_id,
            payload.endpoints.len()
        );

        // Store in peer registry
        let mut peers = self.peers.write().await;
        peers.insert(payload.session_id, peer);
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_listener_new() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        assert_eq!(listener.port, 2300);
        assert_eq!(listener.peer_timeout_secs, 60);
        assert!(listener.multicast_addr.is_some());
        assert!(listener.node_id.is_none());
    }

    #[test]
    fn test_listener_broadcast_only() {
        let listener = AnonymousDiscoveryListener::new_broadcast_only(2300, 60);
        assert_eq!(listener.port, 2300);
        assert!(listener.multicast_addr.is_none());
    }

    #[test]
    fn test_listener_with_node_id() {
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_node_id("tower1".to_string());
        assert_eq!(listener.node_id, Some("tower1".to_string()));
    }

    #[tokio::test]
    async fn test_listener_get_peers_empty() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        let peers = listener.get_peers().await;
        assert_eq!(peers.len(), 0);
    }

    #[tokio::test]
    async fn test_listener_get_peer_not_found() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        let peer = listener.get_peer("nonexistent").await;
        assert!(peer.is_none());
    }

    #[test]
    fn anonymous_message_from_bytes_matches_constructed() {
        use crate::anonymous::messages::AnonymousDiscoveryMessage;
        let msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".into()],
            vec!["https".into()],
            8443,
        );
        let bytes = msg.to_bytes().expect("serialize");
        let parsed = AnonymousDiscoveryMessage::from_bytes(&bytes).expect("parse");
        assert_eq!(parsed.port, 8443);
        assert_eq!(parsed.version, "2.1");
        assert!(parsed.validate().is_ok());
    }
}
