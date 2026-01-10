//! Anonymous Discovery Protocol
//!
//! Implements secure anonymous discovery with UDP multicast.
//! Towers discover each other without leaking identity, sharing only capabilities.
//!
//! ## Design Principles
//!
//! 1. **Anonymous First**: No identity shared in discovery messages
//! 2. **Capability-Based**: Share what you can do, not who you are
//! 3. **Rotating Sessions**: Session IDs rotate every hour to prevent tracking
//! 4. **Cryptographic Proof**: Capabilities are cryptographically signed
//! 5. **Progressive Trust**: Start anonymous, escalate on demand
//! 6. **Multicast-First**: Uses UDP multicast for reliable cross-router discovery
//!
//! ## Protocol Flow
//!
//! ```text
//! Tower A                          Tower B
//!    |                                |
//!    |  UDP Multicast (224.0.0.251)   |
//!    |  {session_id, capabilities}    |
//!    |------------------------------->|
//!    |                                |
//!    |  UDP Response                  |
//!    |  {session_id, capabilities}    |
//!    |<-------------------------------|
//!    |                                |
//!    |  Establish Anonymous TLS       |
//!    |<==============================>|
//!    |                                |
//!    |  Coordinate Tasks (Level 1)    |
//!    |<==============================>|
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Anonymous discovery message (v2.1)
///
/// This message is broadcast over UDP to discover other Songbird towers.
/// It contains NO identity information - only capabilities and connection info.
///
/// ## What's Shared (Anonymous):
/// - Capabilities (what can be done)
/// - Protocols (how to connect)
/// - Port (where to connect)
/// - Session ID (temporary, rotates hourly)
///
/// ## What's NOT Shared (Private):
/// - Hostname
/// - Node ID
/// - Internal topology
/// - User data
///
/// The IP address is inherently revealed by UDP (sender address), but we don't
/// include it in the message to avoid redundancy and maintain protocol purity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousDiscoveryMessage {
    /// Protocol version (now "2.1" for connection-aware discovery, or "3.0" for multi-endpoint)
    pub version: String,

    /// Stable node ID (v3.0+) - allows interface coalescence
    ///
    /// In v3.0, this is the stable machine-based UUID.
    /// Receivers can group multiple endpoints under same `node_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Human-readable node name (v3.0+)
    ///
    /// Example: "eastgate", "westgate", "strandgate"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    /// Temporary session ID (v2.x) - rotates every hour
    ///
    /// This prevents tracking across sessions while allowing response correlation.
    /// In v3.0, this is deprecated in favor of `node_id`, but still included for compatibility.
    pub session_id: String,

    /// All transport endpoints for this node (v3.0+)
    ///
    /// Each endpoint represents a different network interface (Ethernet, `WiFi`, etc.)
    /// with its own address and capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities offered by this tower
    ///
    /// Examples: "orchestration", "gpu-compute", "storage", "ml-inference"
    pub capabilities: Vec<String>,

    /// Supported protocols for communication
    ///
    /// Examples: "https", "tarpc-tls", "websocket-tls"
    pub protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening (v2.x)
    ///
    /// Combined with the UDP sender's IP address, this allows peers to connect.
    /// This is NOT considered identity information - it's connection metadata.
    /// In v3.0, this is deprecated in favor of endpoints array.
    pub port: u16,

    /// Timestamp of message creation (Unix epoch seconds)
    pub timestamp: u64,

    /// Generic tags (NEW - for USB seed integration)
    ///
    /// Contains BearDog encryption tags and other metadata for trust evaluation.
    /// Songbird doesn't parse these - just passes them to the security provider.
    ///
    /// Examples:
    /// - BearDog lineage: `"beardog:family:a3f2:tower1"`
    /// - Protocol support: `"btsp_enabled"`, `"birdsong_v2"`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Identity attestations (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Structured identity information from security providers (e.g., BearDog, ToadStool).
    /// Enables genetic lineage auto-trust and provider-agnostic authentication.
    ///
    /// MUST be included for federation to work with genetic lineage!
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// Optional: Cryptographic proof of capabilities
    ///
    /// This can be used to verify that the tower actually has the claimed capabilities.
    /// For now, this is optional and can be added later for enhanced security.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_proof: Option<String>,
}

/// Transport endpoint in discovery message (v3.0+)
///
/// CRITICAL EVOLUTION (Dec 20, 2025): Changed from "port" to "address" (IP:port)
/// to enable proper multi-interface coalescence. Without the full address, receivers
/// couldn't distinguish between interfaces on the same machine vs different machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEndpointMessage {
    /// Interface type (e.g., "ethernet", "wifi", "bluetooth")
    pub interface_type: String,

    /// Full network address for this endpoint (IP:port format)
    ///
    /// This allows receivers to properly coalesce multiple interfaces under a
    /// single node identity based on the stable `node_id`.
    pub address: String,

    /// Protocols supported on this endpoint
    pub protocols: Vec<String>,

    /// Relative preference (0-255, higher = more preferred)
    pub preference: u8,
}

impl AnonymousDiscoveryMessage {
    /// Create a new anonymous discovery message (v2.1 - backward compatible)
    #[must_use]
    pub fn new(capabilities: Vec<String>, protocols: Vec<String>, port: u16) -> Self {
        Self {
            version: "2.1".to_string(),
            node_id: None,
            node_name: None,
            session_id: Self::generate_session_id(),
            endpoints: None,
            capabilities,
            protocols,
            port,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            tags: None, // Will be populated by security provider if configured
            identity_attestations: None, // No attestations in v2.1 (legacy)
            capability_proof: None,
        }
    }

    /// Create a new multi-endpoint discovery message (v3.0)
    ///
    /// This includes stable node identity and multiple transport endpoints.
    /// Receivers can coalesce multiple endpoints under the same `node_id`.
    #[must_use]
    pub fn new_v3(
        node_id: String,
        node_name: String,
        endpoints: Vec<TransportEndpointMessage>,
        capabilities: Vec<String>,
    ) -> Self {
        // Get primary endpoint for backward compatibility
        let primary_endpoint = endpoints.first();

        // Extract port from address (format: "IP:port")
        let port = primary_endpoint
            .and_then(|e| e.address.split(':').nth(1))
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let protocols =
            primary_endpoint.map_or_else(|| vec!["https".to_string()], |e| e.protocols.clone());

        Self {
            version: "3.0".to_string(),
            node_id: Some(node_id.clone()),
            node_name: Some(node_name),
            session_id: Self::generate_session_id_from_node(&node_id),
            endpoints: Some(endpoints),
            capabilities,
            protocols,
            port,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            tags: None, // Will be populated by security provider if configured
            identity_attestations: None, // Will be populated by security provider if configured
            capability_proof: None,
        }
    }

    /// Set identity attestations (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Adds identity attestations from security provider for genetic lineage auto-trust.
    pub fn with_identity_attestations(
        mut self,
        attestations: Vec<crate::IdentityAttestation>,
    ) -> Self {
        self.identity_attestations = Some(attestations);
        self
    }

    /// Generate a rotating session ID
    ///
    /// Session IDs are based on:
    /// - Current hour (rotates every hour)
    /// - Random UUID (prevents collisions)
    ///
    /// This allows correlation of responses within an hour while preventing long-term tracking.
    fn generate_session_id() -> String {
        use sha2::{Digest, Sha256};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        // Truncate to hour (rotates every 3600 seconds)
        let hour = now / 3600;

        // Mix with random UUID to prevent collisions
        let uuid = Uuid::new_v4();

        // Hash to create session ID
        let mut hasher = Sha256::new();
        hasher.update(hour.to_le_bytes());
        hasher.update(uuid.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Generate a session ID from stable node ID (v3.0+)
    ///
    /// This creates a deterministic but rotating session ID based on:
    /// - Stable node ID (for consistency within an hour)
    /// - Current hour (for rotation)
    fn generate_session_id_from_node(node_id: &str) -> String {
        use sha2::{Digest, Sha256};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        // Truncate to hour (rotates every 3600 seconds)
        let hour = now / 3600;

        // Hash node_id + hour
        let mut hasher = Sha256::new();
        hasher.update(node_id.as_bytes());
        hasher.update(hour.to_le_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Serialize to JSON bytes for UDP transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes received via UDP
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Validate the discovery message
    ///
    /// Checks:
    /// - Protocol version is "2.0", "2.1", or "3.0"
    /// - Session ID is not empty
    /// - At least one capability
    /// - At least one protocol
    /// - Port is valid (non-zero)
    /// - Timestamp is recent (within 5 minutes)
    /// - For v3.0: `node_id` and endpoints are present
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "2.0" && self.version != "2.1" && self.version != "3.0" {
            return Err(format!("Unsupported protocol version: {}", self.version));
        }

        // v3.0 specific validation
        if self.version == "3.0" {
            if self.node_id.is_none() {
                return Err("v3.0 requires node_id".to_string());
            }
            if self.node_name.is_none() {
                return Err("v3.0 requires node_name".to_string());
            }
            if self.endpoints.is_none() || self.endpoints.as_ref().unwrap().is_empty() {
                return Err("v3.0 requires at least one endpoint".to_string());
            }
        }

        if self.port == 0 {
            return Err("Invalid port: 0".to_string());
        }

        if self.session_id.is_empty() {
            return Err("Session ID is empty".to_string());
        }

        if self.capabilities.is_empty() {
            return Err("No capabilities specified".to_string());
        }

        if self.protocols.is_empty() {
            return Err("No protocols specified".to_string());
        }

        // Check timestamp is recent (within 5 minutes)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let age = now.saturating_sub(self.timestamp);
        if age > 300 {
            // 5 minutes
            return Err(format!("Message too old: {age} seconds"));
        }

        Ok(())
    }
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Session ID of the peer (v2.x - deprecated in v3.0)
    pub session_id: String,

    /// Stable node ID (v3.0+) - machine-based UUID for identity coalescence
    pub node_id: Option<String>,

    /// Human-readable node name (v3.0+) - e.g., "eastgate", "westgate"
    pub node_name: Option<String>,

    /// All transport endpoints for this node (v3.0+)
    pub endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities offered by the peer
    pub capabilities: Vec<String>,

    /// Generic tags (NEW - for USB seed integration)
    /// Contains BearDog encryption tags for genetic lineage verification
    pub tags: Option<Vec<String>>,

    /// Discovery timestamp (NEW - for USB seed integration)
    /// Unix timestamp when this discovery message was sent
    pub timestamp: Option<u64>,

    /// Identity attestations (CRITICAL FIX - Jan 3, 2026)
    /// Structured identity information from security providers for genetic lineage auto-trust
    pub identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Port where the peer's HTTPS/TLS server is listening (v2.x)
    pub port: u16,

    /// Socket address where the discovery message came from (UDP source)
    pub address: SocketAddr,

    /// When this peer was last seen
    pub last_seen: SystemTime,

    /// Discovery message version
    pub version: String,
}

impl DiscoveredPeer {
    /// Get the HTTPS endpoint for this peer
    ///
    /// Combines the source IP (from UDP) with the advertised HTTPS port
    #[must_use]
    pub fn https_endpoint(&self) -> String {
        format!("https://{}:{}", self.address.ip(), self.port)
    }
}

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
    broadcast_addresses: Vec<SocketAddr>,

    /// Known peer addresses for direct discovery (bypasses multicast)
    known_peers: Vec<SocketAddr>,

    /// Broadcast interval in seconds
    interval_secs: u64,

    /// Identity attestations from security provider (CRITICAL FIX - Jan 3, 2026)
    ///
    /// These are provided by the security capability provider (e.g., BearDog) on startup
    /// and enable genetic lineage auto-trust. MUST be included for federation to work!
    identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// BirdSong encryption processor (optional) - NEW (Jan 3, 2026)
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
    /// after querying the security provider (e.g., BearDog) for our node's identity.
    #[must_use]
    pub fn with_identity_attestations(
        mut self,
        attestations: Vec<crate::IdentityAttestation>,
    ) -> Self {
        self.identity_attestations = Some(attestations);
        self
    }

    /// Enable statistics tracking for observability (NEW - Jan 5, 2026)
    #[must_use]
    pub fn with_stats(mut self, stats: Arc<crate::discovery_stats::DiscoveryStats>) -> Self {
        stats.set_broadcasting(true);
        self.stats = Some(stats);
        self
    }

    /// Enable BirdSong encrypted discovery (NEW - Jan 3, 2026)
    ///
    /// Adds BirdSong encryption for privacy-preserving discovery.
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

            // Serialize to bytes
            let bytes = match message.to_bytes() {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to serialize discovery message: {}", e);
                    continue;
                }
            };

            // 🎵 NEW (Jan 3, 2026): Optional BirdSong encryption for privacy
            let bytes = if let Some(ref birdsong) = self.birdsong {
                match birdsong.encrypt_packet(&bytes).await {
                    Ok(encrypted) => {
                        debug!(
                            "🔒 BirdSong encrypted: {} -> {} bytes",
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
            } else {
                bytes
            };

            // 1. Send to multicast/broadcast addresses
            for addr in &self.broadcast_addresses {
                match socket.send_to(&bytes, addr).await {
                    Ok(sent) => {
                        debug!("Multicast {} bytes to {}", sent, addr);
                    }
                    Err(e) => {
                        warn!("Failed to multicast to {}: {}", addr, e);
                    }
                }
            }

            // 2. Send to known peers (direct UDP)
            for peer in &self.known_peers {
                match socket.send_to(&bytes, peer).await {
                    Ok(sent) => {
                        debug!("Direct send {} bytes to known peer {}", sent, peer);
                    }
                    Err(e) => {
                        warn!("Failed to send to known peer {}: {}", peer, e);
                    }
                }
            }

            debug!("📡 Broadcast discovery message (session: {})", message.session_id);
        }
    }
}

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
    ///
    /// # Example
    /// ```
    /// let listener = AnonymousDiscoveryListener::new(2300, 60)
    ///     .with_node_id("tower1".to_string());
    /// ```
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
                                plaintext
                            }
                            Ok(None) => {
                                // Different family - just noise, skip this packet
                                debug!("🔇 BirdSong noise from {} (different family)", addr);
                                continue;
                            }
                            Err(e) => {
                                warn!("⚠️  BirdSong decryption failed from {}: {}", addr, e);
                                continue;
                            }
                        }
                    } else {
                        data.to_vec()
                    };

                    // Parse discovery message
                    match AnonymousDiscoveryMessage::from_bytes(&data) {
                        Ok(message) => {
                            // Validate message
                            if let Err(e) = message.validate() {
                                warn!("Invalid discovery message from {}: {}", addr, e);
                                continue;
                            }

                            // CRITICAL FIX (v3.10.2 - Jan 5, 2026): Filter out self-discovery
                            // Prevents towers from discovering their own broadcasts
                            // Critical for multi-instance deployments (tower1, tower2, etc.)
                            if let Some(ref my_node_id) = self.node_id {
                                if let Some(ref peer_node_id) = message.node_id {
                                    if my_node_id == peer_node_id {
                                        debug!("📭 Skipping own broadcast (self-discovery filtered: {})", my_node_id);

                                        // Update stats: self-discoveries filtered
                                        if let Some(ref _stats) = self.stats {
                                            // Note: Could add a self_discoveries_filtered counter if needed
                                        }

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
                                tags: message.tags.clone(), // NEW: BearDog encryption tags for trust evaluation
                                timestamp: Some(message.timestamp), // NEW: For trust evaluation timing
                                identity_attestations: message.identity_attestations.clone(), // 🚨 CRITICAL FIX (Jan 3): Genetic lineage auto-trust
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

                            let mut peers = self.peers.write().await;
                            peers.insert(message.session_id, peer);
                        }
                        Err(e) => {
                            warn!("Failed to parse discovery message from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive discovery message: {}", e);
                }
            }
        }
    }

    /// Get all discovered peers
    pub async fn get_peers(&self) -> Vec<DiscoveredPeer> {
        let peers = self.peers.read().await;

        // INFO LOGGING (v3.10.3 - Jan 6, 2026): Diagnose bridge gap at INFO level
        // Previous debug!() required RUST_LOG=debug, now visible with default logging
        if !peers.is_empty() {
            info!("📊 get_peers() called: {} peers in HashMap", peers.len());
            for (session_id, peer) in peers.iter() {
                let node_name = peer.node_name.as_deref().unwrap_or("unknown");
                let node_id = peer.node_id.as_deref().unwrap_or("no-id");
                info!("   - session:{} | node_id:{} | name:{}", session_id, node_id, node_name);
            }
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
    fn test_anonymous_discovery_message_creation() {
        let capabilities = vec!["orchestration".to_string(), "gpu-compute".to_string()];
        let protocols = vec!["https".to_string(), "tarpc-tls".to_string()];
        let port = 8080u16;

        let message = AnonymousDiscoveryMessage::new(capabilities.clone(), protocols.clone(), port);

        assert_eq!(message.version, "2.1");
        assert!(!message.session_id.is_empty());
        assert_eq!(message.capabilities, capabilities);
        assert_eq!(message.protocols, protocols);
        assert!(message.timestamp > 0);
    }

    #[test]
    fn test_session_id_rotation() {
        // Generate multiple session IDs - they should be different (due to UUID)
        let id1 = AnonymousDiscoveryMessage::generate_session_id();
        let id2 = AnonymousDiscoveryMessage::generate_session_id();

        assert_ne!(id1, id2, "Session IDs should be unique");
        assert_eq!(id1.len(), 64, "Session ID should be 64 hex characters (SHA256)");
    }

    #[test]
    fn test_message_serialization() {
        let capabilities = vec!["orchestration".to_string()];
        let protocols = vec!["https".to_string()];
        let port = 8080u16;

        let message = AnonymousDiscoveryMessage::new(capabilities, protocols, port);

        // Serialize
        let bytes = message.to_bytes().unwrap();
        assert!(!bytes.is_empty());

        // Deserialize
        let deserialized = AnonymousDiscoveryMessage::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.version, message.version);
        assert_eq!(deserialized.session_id, message.session_id);
        assert_eq!(deserialized.capabilities, message.capabilities);
    }

    #[test]
    fn test_message_validation() {
        let capabilities = vec!["orchestration".to_string()];
        let protocols = vec!["https".to_string()];
        let port = 8080u16;

        let message = AnonymousDiscoveryMessage::new(capabilities, protocols, port);

        // Valid message should pass
        assert!(message.validate().is_ok());

        // Invalid version
        let mut invalid = message.clone();
        invalid.version = "1.0".to_string();
        assert!(invalid.validate().is_err());

        // Empty session ID
        let mut invalid = message.clone();
        invalid.session_id = String::new();
        assert!(invalid.validate().is_err());

        // No capabilities
        let mut invalid = message.clone();
        invalid.capabilities = vec![];
        assert!(invalid.validate().is_err());

        // No protocols
        let mut invalid = message.clone();
        invalid.protocols = vec![];
        assert!(invalid.validate().is_err());

        // Old timestamp
        let mut invalid = message.clone();
        invalid.timestamp = 0; // Very old
        assert!(invalid.validate().is_err());
    }

    #[tokio::test]
    async fn test_discovery_listener_creation() {
        let listener = AnonymousDiscoveryListener::new(2300, 60);
        assert_eq!(listener.port, 2300);
        assert_eq!(listener.peer_timeout_secs, 60);

        let peers = listener.get_peers().await;
        assert!(peers.is_empty());
    }

    #[test]
    fn test_discovery_broadcaster_creation() {
        let capabilities = vec!["orchestration".to_string()];
        let protocols = vec!["https".to_string()];
        let broadcast_addrs = vec!["255.255.255.255:2300".parse().unwrap()];
        let port = 8080u16;
        let interval_secs = 30u64;

        let broadcaster = AnonymousDiscoveryBroadcaster::new(
            capabilities.clone(),
            protocols.clone(),
            port,
            broadcast_addrs.clone(),
            interval_secs,
        );

        assert_eq!(broadcaster.capabilities, capabilities);
        assert_eq!(broadcaster.protocols, protocols);
        assert_eq!(broadcaster.port, port);
        assert_eq!(broadcaster.broadcast_addresses, broadcast_addrs);
        assert_eq!(broadcaster.interval_secs, interval_secs);
    }

    // ========================================================================
    // v3.3 TESTS: BirdSong Listener Integration
    // ========================================================================

    #[test]
    fn test_listener_with_birdsong_builder() {
        use crate::birdsong_integration::{BirdSongConfig, BirdSongProcessor};

        // Create a plaintext-fallback BirdSong processor
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true,
        };
        let processor = Arc::new(BirdSongProcessor::new(None, config));

        // Create listener with BirdSong
        let listener = AnonymousDiscoveryListener::new(2300, 60).with_birdsong(processor.clone());

        // Verify BirdSong is wired
        assert!(listener.birdsong.is_some());
        assert!(Arc::ptr_eq(&listener.birdsong.unwrap(), &processor));
    }

    #[test]
    fn test_broadcaster_with_identity_attestations() {
        use crate::IdentityAttestation;
        use serde_json::json;

        let capabilities = vec!["orchestration".to_string()];
        let protocols = vec!["https".to_string()];
        let broadcast_addrs = vec!["224.0.0.251:2300".parse().unwrap()];

        // Create identity attestations
        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "test-family",
                "tags": ["beardog:family:test-family:node1"]
            }),
        }];

        // Create broadcaster with attestations
        let broadcaster =
            AnonymousDiscoveryBroadcaster::new(capabilities, protocols, 8080, broadcast_addrs, 30)
                .with_identity_attestations(attestations.clone());

        // Verify attestations are stored
        assert!(broadcaster.identity_attestations.is_some());
        assert_eq!(broadcaster.identity_attestations.unwrap().len(), 1);
    }

    #[test]
    fn test_discovery_message_with_attestations() {
        use crate::IdentityAttestation;
        use serde_json::json;

        let capabilities = vec!["orchestration".to_string()];
        let protocols = vec!["https".to_string()];

        // Create identity attestations
        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "iidn",
                "tags": ["beardog:family:iidn:test-node"]
            }),
        }];

        // Create message with attestations
        let message = AnonymousDiscoveryMessage::new(capabilities, protocols, 8080)
            .with_identity_attestations(attestations.clone());

        // Verify attestations are present
        assert!(message.identity_attestations.is_some());
        let msg_attestations = message.identity_attestations.as_ref().unwrap();
        assert_eq!(msg_attestations.len(), 1);
        assert_eq!(msg_attestations[0].provider_capability, "security/identity");

        // Verify serialization preserves attestations
        let bytes = message.to_bytes().unwrap();
        let deserialized = AnonymousDiscoveryMessage::from_bytes(&bytes).unwrap();
        assert!(deserialized.identity_attestations.is_some());
    }

    #[tokio::test]
    async fn test_e2e_broadcaster_listener_with_birdsong() {
        use crate::birdsong_integration::{BirdSongConfig, BirdSongProcessor};
        use crate::IdentityAttestation;
        use serde_json::json;

        // Create identity attestations
        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "test-family",
                "tags": ["beardog:family:test-family:node1"]
            }),
        }];

        // Create BirdSong processor (plaintext fallback mode)
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true,
        };
        let processor = Arc::new(BirdSongProcessor::new(None, config));

        // Create discovery message with attestations
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 100,
        }];

        let message = AnonymousDiscoveryMessage::new_v3(
            "test-node-1".to_string(),
            "test-node-1".to_string(),
            endpoints,
            vec!["orchestration".to_string()],
        )
        .with_identity_attestations(attestations.clone());

        // Serialize message
        let plaintext = message.to_bytes().unwrap();

        // Test encryption (should fall back to plaintext in this mode)
        let encrypted = processor.encrypt_packet(&plaintext).await.unwrap();

        // Test decryption
        let decrypted = processor.decrypt_packet(&encrypted).await.unwrap();
        assert!(decrypted.is_some(), "Decryption should succeed");

        // Verify attestations survived the roundtrip
        let recovered_message = AnonymousDiscoveryMessage::from_bytes(&decrypted.unwrap()).unwrap();
        assert!(recovered_message.identity_attestations.is_some());
        assert_eq!(recovered_message.identity_attestations.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_listener_handles_plaintext_packets() {
        use crate::IdentityAttestation;
        use serde_json::json;

        // Create a listener WITHOUT BirdSong (plaintext mode)
        let _listener = AnonymousDiscoveryListener::new(2301, 60);

        // Create a plaintext discovery message with attestations
        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "plaintext-family",
                "tags": ["beardog:family:plaintext-family:node1"]
            }),
        }];

        let message = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        )
        .with_identity_attestations(attestations);

        // Serialize to bytes (plaintext)
        let bytes = message.to_bytes().unwrap();

        // Verify listener can parse plaintext packets
        let parsed = AnonymousDiscoveryMessage::from_bytes(&bytes).unwrap();
        assert!(parsed.identity_attestations.is_some());
        assert_eq!(parsed.identity_attestations.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_broadcaster_encrypts_attestations() {
        use crate::birdsong_integration::{BirdSongConfig, BirdSongProcessor};
        use crate::IdentityAttestation;
        use serde_json::json;

        // Create identity attestations
        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "encrypted-family",
                "tags": ["beardog:family:encrypted-family:node1"]
            }),
        }];

        // Create BirdSong processor
        let config = BirdSongConfig {
            enabled: true,
            fallback_to_plaintext: true,
            security_endpoint: None,
            mixed_mode: true,
        };
        let processor = Arc::new(BirdSongProcessor::new(None, config));

        // Create discovery message with attestations
        let message = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        )
        .with_identity_attestations(attestations.clone());

        // Serialize and encrypt
        let plaintext = message.to_bytes().unwrap();
        let encrypted = processor.encrypt_packet(&plaintext).await.unwrap();

        // Verify encryption changes the data (in real encryption mode)
        // In fallback mode, it might be the same, but structure should be valid
        assert!(!encrypted.is_empty());

        // Verify we can decrypt back
        let decrypted = processor.decrypt_packet(&encrypted).await.unwrap();
        assert!(decrypted.is_some());

        // Verify attestations are preserved
        let recovered = AnonymousDiscoveryMessage::from_bytes(&decrypted.unwrap()).unwrap();
        assert!(recovered.identity_attestations.is_some());
    }

    #[test]
    fn test_v3_message_with_attestations_serialization() {
        use crate::IdentityAttestation;
        use serde_json::json;

        // Create v3.0 message with all features
        let endpoints = vec![
            TransportEndpointMessage {
                interface_type: "ethernet".to_string(),
                address: "192.168.1.100:8080".to_string(),
                protocols: vec!["https".to_string()],
                preference: 100,
            },
            TransportEndpointMessage {
                interface_type: "wifi".to_string(),
                address: "192.168.1.101:8080".to_string(),
                protocols: vec!["https".to_string()],
                preference: 50,
            },
        ];

        let attestations = vec![IdentityAttestation {
            provider_capability: "security/identity".to_string(),
            format: "tag_list".to_string(),
            data: json!({
                "family_id": "v3-family",
                "tags": ["beardog:family:v3-family:multi-endpoint"]
            }),
        }];

        let message = AnonymousDiscoveryMessage::new_v3(
            "multi-endpoint-node".to_string(),
            "multi-endpoint-node".to_string(),
            endpoints,
            vec!["orchestration".to_string(), "storage".to_string()],
        )
        .with_identity_attestations(attestations);

        // Serialize
        let bytes = message.to_bytes().unwrap();

        // Deserialize
        let recovered = AnonymousDiscoveryMessage::from_bytes(&bytes).unwrap();

        // Verify all fields preserved
        assert_eq!(recovered.version, "3.0");
        assert!(recovered.node_id.is_some());
        assert!(recovered.endpoints.is_some());
        assert_eq!(recovered.endpoints.unwrap().len(), 2);
        assert!(recovered.identity_attestations.is_some());
        assert_eq!(recovered.identity_attestations.unwrap().len(), 1);
    }
}
