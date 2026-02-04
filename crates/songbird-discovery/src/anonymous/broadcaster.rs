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
use tracing::{debug, error, info, warn};

use super::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

/// Anonymous discovery broadcaster
///
/// Broadcasts anonymous discovery messages over UDP multicast to find other towers.
/// Supports both v2.1 (session-based) and v3.0 (node-identity-based) protocols.
///
/// Uses UDP multicast (224.0.0.251) by default for reliable cross-router discovery.
/// Falls back to broadcast and known peers for maximum compatibility.
pub struct AnonymousDiscoveryBroadcaster {
    /// Protocol version ("2.1" or "3.0")
    version: String,

    /// Stable node ID (v3.0 only)
    node_id: Option<String>,

    /// Node name (v3.0 only)
    node_name: Option<String>,

    /// All endpoints (v3.0 only)
    endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities to advertise
    capabilities: Vec<String>,

    /// Protocols supported (v2.1 fallback)
    protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening (v2.1 fallback)
    port: u16,

    /// Multicast/broadcast addresses to send to
    /// Includes multicast (224.0.0.251) + subnet broadcast fallback for cross-interface discovery
    broadcast_addresses: Vec<SocketAddr>,

    /// Known peer addresses for direct discovery (bypasses multicast)
    known_peers: Vec<SocketAddr>,

    /// Broadcast interval in seconds
    interval_secs: u64,

    /// Identity tags (v3.14.0 - tag-based identity)
    /// Opaque strings we broadcast. We don't interpret them!
    /// Format: `{provider}:{type}:{value}` (e.g., `beardog:family:nat0`)
    tags: Option<Vec<String>>,

    /// Identity attestations from security provider (CRITICAL FIX - Jan 3, 2026)
    ///
    /// These are provided by the security capability provider (e.g., `BearDog`) on startup
    /// and enable genetic lineage auto-trust. MUST be included for federation to work!
    identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// `BirdSong` encryption processor (optional) - NEW (Jan 3, 2026)
    birdsong: Option<Arc<crate::birdsong_integration::BirdSongProcessor>>,

    /// Statistics tracker for observability (optional) - NEW (Jan 5, 2026)
    stats: Option<Arc<crate::discovery_stats::DiscoveryStats>>,
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
            .unwrap_or(8080);

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
    /// after querying the security provider (e.g., `BearDog`) for our node's identity.
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
    /// Example: `beardog:family:nat0`
    ///
    /// Security providers (`BearDog`) interpret tag meaning.
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
    pub fn with_birdsong(
        mut self,
        processor: Arc<crate::birdsong_integration::BirdSongProcessor>,
    ) -> Self {
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

        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(self.interval_secs));

        loop {
            interval.tick().await;

            // Create discovery message (v2.1 or v3.0)
            let mut message = if self.version == "3.0" {
                AnonymousDiscoveryMessage::new_v3(
                    self.node_id.clone().unwrap(),
                    self.node_name.clone().unwrap(),
                    self.endpoints.clone().unwrap(),
                    self.capabilities.clone(),
                )
            } else {
                AnonymousDiscoveryMessage::new(
                    self.capabilities.clone(),
                    self.protocols.clone(),
                    self.port,
                )
            };

            // 🚨 CRITICAL FIX (Jan 3, 2026): Include identity attestations for genetic lineage auto-trust
            if let Some(ref attestations) = self.identity_attestations {
                message = message.with_identity_attestations(attestations.clone());
            }

            // ✅ CRITICAL FIX (v3.14.2 - Jan 7, 2026): Include identity tags!
            // THIS WAS THE BUG: Tags were in self.tags but never added to message!
            if let Some(ref tags) = self.tags {
                debug!("📋 Broadcasting {} identity tags: {:?}", tags.len(), tags);
                message = message.with_tags(tags.clone());
            } else {
                debug!("📋 No identity tags to broadcast");
            }

            // Serialize to bytes
            let bytes = match message.to_bytes() {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to serialize discovery message: {}", e);
                    continue;
                }
            };

            // 🎵 NEW (Jan 3, 2026): Optional BirdSong encryption for privacy
            // 🌲 EVOLVED (Feb 3, 2026): Support Dark Forest beacons (zero metadata leakage)
            let bytes = if let Some(ref birdsong) = self.birdsong {
                // Check if Dark Forest mode is enabled
                if birdsong.config().is_dark_forest_active() {
                    // Dark Forest mode - broadcast fully encrypted beacon
                    match self.create_and_broadcast_dark_forest_beacon(&socket, birdsong).await {
                        Ok(()) => {
                            debug!("🌲 Dark Forest beacon broadcasted");

                            // If dual broadcast enabled, also send legacy format
                            if birdsong.config().dual_broadcast {
                                debug!("📢 Dual broadcast: also sending legacy format");
                                // Encrypt with legacy BirdSong (has plaintext family_id)
                                match birdsong.encrypt_packet(&bytes).await {
                                    Ok(encrypted) => encrypted,
                                    Err(e) => {
                                        warn!("⚠️  Legacy encryption failed: {}", e);
                                        bytes.clone()
                                    }
                                }
                            } else {
                                // Dark Forest only - skip legacy broadcast
                                continue;
                            }
                        }
                        Err(e) => {
                            warn!(
                                "⚠️  Dark Forest broadcast failed: {}, falling back to legacy",
                                e
                            );
                            // Fallback to legacy BirdSong
                            match birdsong.encrypt_packet(&bytes).await {
                                Ok(encrypted) => {
                                    debug!(
                                        "🔒 BirdSong encrypted (legacy fallback): {} -> {} bytes",
                                        bytes.len(),
                                        encrypted.len()
                                    );
                                    encrypted
                                }
                                Err(e) => {
                                    warn!(
                                        "⚠️  Legacy encryption also failed: {}, using plaintext",
                                        e
                                    );
                                    bytes
                                }
                            }
                        }
                    }
                } else {
                    // Legacy BirdSong mode (has plaintext family_id header)
                    match birdsong.encrypt_packet(&bytes).await {
                        Ok(encrypted) => {
                            debug!(
                                "🔒 BirdSong encrypted (legacy): {} -> {} bytes",
                                bytes.len(),
                                encrypted.len()
                            );
                            encrypted
                        }
                        Err(e) => {
                            warn!("⚠️  BirdSong encryption failed: {}, using plaintext", e);
                            bytes
                        }
                    }
                }
            } else {
                bytes
            };

            // 1. Send to multicast/broadcast addresses
            for addr in &self.broadcast_addresses {
                match socket.send_to(&bytes, addr).await {
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

            // 2. Send to known peers (direct UDP)
            for peer in &self.known_peers {
                match socket.send_to(&bytes, peer).await {
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

            debug!("📡 Broadcast discovery message (session: {})", message.session_id);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Dark Forest Beacon Broadcasting (NEW - Feb 3, 2026)
    // ═══════════════════════════════════════════════════════════════════════

    /// Create and broadcast Dark Forest beacon (zero metadata leakage)
    ///
    /// Unlike legacy `BirdSongPacket` which has plaintext `family_id`,
    /// Dark Forest beacons are FULLY encrypted. Observers see only noise.
    ///
    /// # Arguments
    ///
    /// * `socket` - UDP socket for broadcasting
    /// * `birdsong` - `BirdSong` processor for encryption
    ///
    /// # Returns
    ///
    /// Ok if successfully created and broadcasted Dark Forest beacon
    ///
    /// # Privacy Guarantee
    ///
    /// Network observers see ONLY:
    /// - Encrypted blob (random-looking data)
    /// - Nonce (public, reveals nothing)
    /// - Timestamp (replay protection, reveals nothing)
    ///
    /// NO metadata leakage: family, capabilities, endpoints all encrypted.
    async fn create_and_broadcast_dark_forest_beacon(
        &self,
        socket: &UdpSocket,
        birdsong: &crate::birdsong_integration::BirdSongProcessor,
    ) -> Result<(), anyhow::Error> {
        use crate::dark_forest_beacon::BeaconPayload;

        // Get our beacon ID (or generate placeholder if not available yet)
        let beacon_id = match birdsong.encryption_provider() {
            Some(enc) if enc.is_available() => {
                enc.get_beacon_id().await?.unwrap_or_else(|| {
                    // Placeholder if beacon ID not yet available
                    vec![0u8; 16]
                })
            }
            _ => vec![0u8; 16], // Placeholder
        };

        // Build endpoints list from our configuration
        let endpoints: Vec<String> = if let Some(ref eps) = self.endpoints {
            eps.iter().map(|e| format!("{}:{}", e.interface_type, e.address)).collect()
        } else {
            vec![format!("tcp:0.0.0.0:{}", self.port)]
        };

        // Create beacon payload
        let payload = BeaconPayload::new(
            beacon_id,
            self.node_id.clone().unwrap_or_else(|| "unknown".to_string()),
            endpoints,
            &self.capabilities,
            None,                       // cluster_id - TODO: Add cluster support
            self.generate_session_id(), // Session ID for rotation
        );

        // Encrypt payload to create Dark Forest beacon
        let beacon = birdsong
            .encrypt_dark_forest_beacon(&payload)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to encrypt Dark Forest beacon: {e}"))?;

        // Serialize beacon to bytes
        let beacon_bytes = beacon
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Failed to serialize Dark Forest beacon: {e}"))?;

        // Broadcast to all multicast/broadcast addresses
        for addr in &self.broadcast_addresses {
            match socket.send_to(&beacon_bytes, addr).await {
                Ok(sent) => {
                    debug!("🌲 Dark Forest beacon sent {} bytes to {}", sent, addr);
                    if let Some(ref stats) = self.stats {
                        stats.record_broadcast();
                    }
                }
                Err(e) => {
                    warn!("Failed to send Dark Forest beacon to {}: {}", addr, e);
                    if let Some(ref stats) = self.stats {
                        stats.record_error();
                    }
                }
            }
        }

        // Send to known peers
        for peer in &self.known_peers {
            match socket.send_to(&beacon_bytes, peer).await {
                Ok(sent) => {
                    debug!("🌲 Dark Forest beacon sent {} bytes to known peer {}", sent, peer);
                    if let Some(ref stats) = self.stats {
                        stats.record_broadcast();
                    }
                }
                Err(e) => {
                    warn!("Failed to send Dark Forest beacon to peer {}: {}", peer, e);
                    if let Some(ref stats) = self.stats {
                        stats.record_error();
                    }
                }
            }
        }

        info!(
            "🌲 Broadcasted Dark Forest beacon (size: {} bytes, NO metadata leakage)",
            beacon_bytes.len()
        );

        Ok(())
    }

    /// Generate session ID for beacon rotation
    ///
    /// Creates a unique session ID that rotates periodically.
    /// Prevents long-term tracking by changing session identifiers.
    ///
    /// Current implementation: timestamp-based (rotates every ~hour)
    /// Production: Should rotate every 24 hours
    fn generate_session_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Session ID rotates every hour (3600 seconds)
        // Production: Change to 86400 for daily rotation
        let session_slot = timestamp / 3600;

        format!("session-{session_slot}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_new_v2() {
        let broadcaster = AnonymousDiscoveryBroadcaster::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
            vec!["224.0.0.251:2300".parse().unwrap()],
            30,
        );

        assert_eq!(broadcaster.version, "2.1");
        assert!(broadcaster.node_id.is_none());
        assert_eq!(broadcaster.capabilities.len(), 1);
        assert_eq!(broadcaster.interval_secs, 30);
    }

    #[test]
    fn test_broadcaster_new_v3() {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 255,
        }];

        let broadcaster = AnonymousDiscoveryBroadcaster::new_v3(
            "node-123".to_string(),
            "testnode".to_string(),
            endpoints,
            vec!["orchestration".to_string()],
            vec!["224.0.0.251:2300".parse().unwrap()],
            30,
        );

        assert_eq!(broadcaster.version, "3.0");
        assert_eq!(broadcaster.node_id, Some("node-123".to_string()));
        assert_eq!(broadcaster.node_name, Some("testnode".to_string()));
        assert!(broadcaster.endpoints.is_some());
    }

    #[test]
    fn test_broadcaster_with_known_peers() {
        let broadcaster = AnonymousDiscoveryBroadcaster::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
            vec!["224.0.0.251:2300".parse().unwrap()],
            30,
        )
        .with_known_peers(vec!["192.168.1.10:2300".parse().unwrap()]);

        assert_eq!(broadcaster.known_peers.len(), 1);
    }

    #[test]
    fn test_broadcaster_with_identity_attestations() {
        use crate::IdentityAttestation;

        let attestation = IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: serde_json::json!({"family_id": "test-family"}),
        };

        let broadcaster = AnonymousDiscoveryBroadcaster::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
            vec!["224.0.0.251:2300".parse().unwrap()],
            30,
        )
        .with_identity_attestations(vec![attestation]);

        assert!(broadcaster.identity_attestations.is_some());
        assert_eq!(broadcaster.identity_attestations.unwrap().len(), 1);
    }
}
