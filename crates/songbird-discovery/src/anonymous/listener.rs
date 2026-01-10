//! Anonymous Discovery Listener
//!
//! This module contains the listening logic for anonymous discovery via UDP multicast.
//!
//! ## Contents
//! - `AnonymousDiscoveryListener` - Listens for discovery messages
//! - Multicast group joining
//! - Message processing & peer registry
//! - Self-filtering logic
//! - Optional BirdSong decryption

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
/// **NEW (Jan 3, 2026)**: Optional BirdSong decryption for privacy-preserving discovery.
pub struct AnonymousDiscoveryListener {
    /// Port to listen on (typically 2300)
    port: u16,

    /// Multicast group to join (e.g., 224.0.0.251)
    multicast_addr: Option<Ipv4Addr>,

    /// Discovered peers (`session_id` -> peer info)
    peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,

    /// Peer timeout in seconds (default: 60)
    peer_timeout_secs: u64,

    /// BirdSong decryption processor (optional)
    birdsong: Option<Arc<crate::birdsong_integration::BirdSongProcessor>>,

    /// Our own node_id for self-filtering (v3.10.2 - Jan 5, 2026)
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

    /// Enable BirdSong encrypted discovery (NEW - Jan 3, 2026)
    ///
    /// Adds BirdSong decryption for privacy-preserving discovery.
    /// Only same-family peers' packets will be decoded.
    #[must_use]
    pub fn with_birdsong(
        mut self,
        processor: Arc<crate::birdsong_integration::BirdSongProcessor>,
    ) -> Self {
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

                    // 🎵 NEW (Jan 3, 2026): Optional BirdSong decryption
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
                            if let Some(ref my_node_id) = self.node_id {
                                if let Some(ref peer_node_id) = message.node_id {
                                    if my_node_id == peer_node_id {
                                        debug!("📭 Skipping own broadcast (self-discovery filtered: {})", my_node_id);
                                        continue;
                                    }
                                }
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

                            info!("🔍 Discovered peer: {} (v{}, capabilities: {:?}, HTTPS: https://{}:{})", 
                                peer_identifier, message.version, message.capabilities, addr.ip(), message.port);

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
}

#[cfg(test)]
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
}
