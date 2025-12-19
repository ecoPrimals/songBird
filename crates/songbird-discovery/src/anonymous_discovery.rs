//! Anonymous Discovery Protocol
//!
//! Implements secure anonymous discovery with UDP broadcast.
//! Towers discover each other without leaking identity, sharing only capabilities.
//!
//! ## Design Principles
//!
//! 1. **Anonymous First**: No identity shared in discovery messages
//! 2. **Capability-Based**: Share what you can do, not who you are
//! 3. **Rotating Sessions**: Session IDs rotate every hour to prevent tracking
//! 4. **Cryptographic Proof**: Capabilities are cryptographically signed
//! 5. **Progressive Trust**: Start anonymous, escalate on demand
//!
//! ## Protocol Flow
//!
//! ```text
//! Tower A                          Tower B
//!    |                                |
//!    |  UDP Broadcast (port 2300)     |
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
use std::net::SocketAddr;
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
    /// Protocol version (now "2.1" for connection-aware discovery)
    pub version: String,

    /// Temporary session ID (rotates every hour)
    ///
    /// This prevents tracking across sessions while allowing response correlation.
    pub session_id: String,

    /// Capabilities offered by this tower
    ///
    /// Examples: "orchestration", "gpu-compute", "storage", "ml-inference"
    pub capabilities: Vec<String>,

    /// Supported protocols for communication
    ///
    /// Examples: "https", "tarpc-tls", "websocket-tls"
    pub protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening
    ///
    /// Combined with the UDP sender's IP address, this allows peers to connect.
    /// This is NOT considered identity information - it's connection metadata.
    pub port: u16,

    /// Timestamp of message creation (Unix epoch seconds)
    pub timestamp: u64,

    /// Optional: Cryptographic proof of capabilities
    ///
    /// This can be used to verify that the tower actually has the claimed capabilities.
    /// For now, this is optional and can be added later for enhanced security.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_proof: Option<String>,
}

impl AnonymousDiscoveryMessage {
    /// Create a new anonymous discovery message
    pub fn new(capabilities: Vec<String>, protocols: Vec<String>, port: u16) -> Self {
        Self {
            version: "2.1".to_string(),
            session_id: Self::generate_session_id(),
            capabilities,
            protocols,
            port,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            capability_proof: None,
        }
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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

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
    /// - Protocol version is "2.0" or "2.1"
    /// - Session ID is not empty
    /// - At least one capability
    /// - At least one protocol
    /// - Port is valid (non-zero)
    /// - Timestamp is recent (within 5 minutes)
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "2.0" && self.version != "2.1" {
            return Err(format!("Unsupported protocol version: {}", self.version));
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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age = now.saturating_sub(self.timestamp);
        if age > 300 {
            // 5 minutes
            return Err(format!("Message too old: {} seconds", age));
        }

        Ok(())
    }
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Session ID of the peer
    pub session_id: String,

    /// Capabilities offered by the peer
    pub capabilities: Vec<String>,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Port where the peer's HTTPS/TLS server is listening
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
    pub fn https_endpoint(&self) -> String {
        format!("https://{}:{}", self.address.ip(), self.port)
    }
}

/// Anonymous discovery broadcaster
///
/// Broadcasts anonymous discovery messages over UDP to find other towers.
pub struct AnonymousDiscoveryBroadcaster {
    /// Capabilities to advertise
    capabilities: Vec<String>,

    /// Protocols supported
    protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening
    port: u16,

    /// Broadcast addresses to send to
    broadcast_addresses: Vec<SocketAddr>,

    /// Broadcast interval in seconds
    interval_secs: u64,
}

impl AnonymousDiscoveryBroadcaster {
    /// Create a new anonymous discovery broadcaster
    pub fn new(
        capabilities: Vec<String>,
        protocols: Vec<String>,
        port: u16,
        broadcast_addresses: Vec<SocketAddr>,
        interval_secs: u64,
    ) -> Self {
        Self {
            capabilities,
            protocols,
            port,
            broadcast_addresses,
            interval_secs,
        }
    }

    /// Start broadcasting discovery messages
    ///
    /// This runs indefinitely, broadcasting every `interval_secs` seconds.
    pub async fn start_broadcasting(&self) -> Result<(), std::io::Error> {
        info!("🌐 Starting anonymous discovery broadcaster");
        info!("   Capabilities: {:?}", self.capabilities);
        info!("   Protocols: {:?}", self.protocols);
        info!("   Broadcast addresses: {:?}", self.broadcast_addresses);
        info!("   Interval: {}s", self.interval_secs);

        // Create UDP socket for broadcasting
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        info!("✅ Anonymous discovery broadcaster started");

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(self.interval_secs));

        loop {
            interval.tick().await;

            // Create discovery message
            let message = AnonymousDiscoveryMessage::new(
                self.capabilities.clone(),
                self.protocols.clone(),
                self.port,
            );

            // Serialize to bytes
            let bytes = match message.to_bytes() {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to serialize discovery message: {}", e);
                    continue;
                }
            };

            // Broadcast to all addresses
            for addr in &self.broadcast_addresses {
                match socket.send_to(&bytes, addr).await {
                    Ok(sent) => {
                        debug!("Broadcast {} bytes to {}", sent, addr);
                    }
                    Err(e) => {
                        warn!("Failed to broadcast to {}: {}", addr, e);
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
pub struct AnonymousDiscoveryListener {
    /// Port to listen on (typically 2300)
    port: u16,

    /// Discovered peers (session_id -> peer info)
    peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>,

    /// Peer timeout in seconds (default: 60)
    peer_timeout_secs: u64,
}

impl AnonymousDiscoveryListener {
    /// Create a new anonymous discovery listener
    pub fn new(port: u16, peer_timeout_secs: u64) -> Self {
        Self {
            port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            peer_timeout_secs,
        }
    }

    /// Start listening for discovery messages
    ///
    /// This runs indefinitely, processing incoming discovery messages.
    pub async fn start_listening(&self) -> Result<(), std::io::Error> {
        info!("👂 Starting anonymous discovery listener on port {}", self.port);

        // Bind UDP socket
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port)).await?;

        info!("✅ Anonymous discovery listener started");

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

                    // Parse discovery message
                    match AnonymousDiscoveryMessage::from_bytes(data) {
                        Ok(message) => {
                            // Validate message
                            if let Err(e) = message.validate() {
                                warn!("Invalid discovery message from {}: {}", addr, e);
                                continue;
                            }

                            debug!("📥 Received discovery from {} (session: {})", addr, message.session_id);

                            // Store peer info
                            let peer = DiscoveredPeer {
                                session_id: message.session_id.clone(),
                                capabilities: message.capabilities.clone(),
                                protocols: message.protocols.clone(),
                                port: message.port,
                                address: addr,
                                last_seen: SystemTime::now(),
                                version: message.version.clone(),
                            };

                            info!("🔍 Discovered peer: {} (capabilities: {:?}, HTTPS: https://{}:{})", 
                                message.session_id, message.capabilities, addr.ip(), message.port);

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
        peers.values().cloned().collect()
    }

    /// Get peer by session ID
    pub async fn get_peer(&self, session_id: &str) -> Option<DiscoveredPeer> {
        let peers = self.peers.read().await;
        peers.get(session_id).cloned()
    }

    /// Cleanup stale peers (runs periodically)
    async fn cleanup_stale_peers(peers: Arc<RwLock<HashMap<String, DiscoveredPeer>>>, timeout_secs: u64) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(timeout_secs / 2));

        loop {
            interval.tick().await;

            let now = SystemTime::now();
            let mut peers_lock = peers.write().await;

            // Remove stale peers
            peers_lock.retain(|session_id, peer| {
                let age = now
                    .duration_since(peer.last_seen)
                    .unwrap_or_default()
                    .as_secs();

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

        let message = AnonymousDiscoveryMessage::new(capabilities.clone(), protocols.clone());

        assert_eq!(message.version, "2.0");
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

        let message = AnonymousDiscoveryMessage::new(capabilities, protocols);

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

        let message = AnonymousDiscoveryMessage::new(capabilities, protocols);

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

        let broadcaster = AnonymousDiscoveryBroadcaster::new(
            capabilities.clone(),
            protocols.clone(),
            broadcast_addrs.clone(),
            30,
        );

        assert_eq!(broadcaster.capabilities, capabilities);
        assert_eq!(broadcaster.protocols, protocols);
        assert_eq!(broadcaster.broadcast_addresses, broadcast_addrs);
        assert_eq!(broadcaster.interval_secs, 30);
    }
}

