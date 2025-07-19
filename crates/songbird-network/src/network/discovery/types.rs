//! Data types and structures for network discovery

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::super::beardog_integration::PeerCapabilities;

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub discovery_timeout: Duration,
    pub peer_timeout: Duration,
    pub topology_update_interval: Duration,
    pub max_peers: usize,
    pub enable_upnp: bool,
    pub enable_stun: bool,
    pub enable_turn: bool,
    pub gaming_optimized: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: Duration::from_secs(3),
            peer_timeout: Duration::from_secs(30),
            topology_update_interval: Duration::from_secs(10),
            max_peers: 100,
            enable_upnp: true,
            enable_stun: true,
            enable_turn: true,
            gaming_optimized: true,
        }
    }
}

/// UPnP device information
#[derive(Debug, Clone)]
pub struct UPnPDevice {
    pub device_id: String,
    pub address: SocketAddr,
    pub device_type: String,
    pub capabilities: Vec<String>,
    pub discovered_at: Instant,
}

/// TURN relay information
#[derive(Debug, Clone)]
pub struct TURNRelay {
    pub relay_id: String,
    pub relay_address: SocketAddr,
    pub allocated_at: Instant,
    pub expires_at: Instant,
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub address: SocketAddr,
    pub peer_type: PeerType,
    pub discovered_via: DiscoveryMethod,
    pub discovered_at: Instant,
    pub last_seen: Instant,
}

/// Type of discovered peer
#[derive(Debug, Clone)]
pub enum PeerType {
    Orchestrator,
    Service,
    Gateway,
    Unknown,
}

/// Method used to discover the peer
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    UPnP,
    STUN,
    TURN,
    Manual,
}

/// Network topology structure
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub nodes: HashMap<String, NetworkNode>,
    pub connections: Vec<NetworkConnection>,
    pub last_updated: Instant,
}

/// Network node information
#[derive(Debug, Clone)]
pub struct NetworkNode {
    pub node_id: String,
    pub address: SocketAddr,
    pub node_type: PeerType,
    pub capabilities: PeerCapabilities,
}

/// Network connection information
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub from_node: String,
    pub to_node: String,
    pub latency_ms: u32,
    pub quality: ConnectionQuality,
}

/// Connection quality enumeration
#[derive(Debug, Clone)]
pub enum ConnectionQuality {
    Excellent, // < 5ms latency
    Good,      // 5-20ms latency
    Fair,      // 20-50ms latency
    Poor,      // > 50ms latency
}

/// Network measurement data
#[derive(Debug, Clone)]
pub struct NetworkMeasurement {
    pub timestamp: Instant,
    pub source: SocketAddr,
    pub target: SocketAddr,
    pub latency_ms: u32,
    pub bandwidth_mbps: u32,
}

impl NetworkTopology {
    /// Create new empty topology
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            last_updated: Instant::now(),
        }
    }

    /// Add node to topology
    pub fn add_node(&mut self, node: NetworkNode) {
        self.nodes.insert(node.node_id.clone(), node);
        self.last_updated = Instant::now();
    }

    /// Add connection to topology
    pub fn add_connection(&mut self, connection: NetworkConnection) {
        // Remove existing connection between same nodes
        self.connections.retain(|c| {
            !(c.from_node == connection.from_node && c.to_node == connection.to_node)
        });
        
        self.connections.push(connection);
        self.last_updated = Instant::now();
    }

    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&NetworkNode> {
        self.nodes.get(node_id)
    }

    /// Get connections from a node
    pub fn get_connections_from(&self, node_id: &str) -> Vec<&NetworkConnection> {
        self.connections
            .iter()
            .filter(|c| c.from_node == node_id)
            .collect()
    }

    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: &PeerType) -> Vec<&NetworkNode> {
        self.nodes
            .values()
            .filter(|node| std::mem::discriminant(&node.node_type) == std::mem::discriminant(node_type))
            .collect()
    }
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkNode {
    /// Create new network node
    pub fn new(node_id: String, address: SocketAddr, node_type: PeerType, capabilities: PeerCapabilities) -> Self {
        Self {
            node_id,
            address,
            node_type,
            capabilities,
        }
    }
}

impl NetworkConnection {
    /// Create new network connection
    pub fn new(from_node: String, to_node: String, latency_ms: u32) -> Self {
        let quality = match latency_ms {
            0..=5 => ConnectionQuality::Excellent,
            6..=20 => ConnectionQuality::Good,
            21..=50 => ConnectionQuality::Fair,
            _ => ConnectionQuality::Poor,
        };

        Self {
            from_node,
            to_node,
            latency_ms,
            quality,
        }
    }
}

impl NetworkMeasurement {
    /// Create new network measurement
    pub fn new(source: SocketAddr, target: SocketAddr, latency_ms: u32, bandwidth_mbps: u32) -> Self {
        Self {
            timestamp: Instant::now(),
            source,
            target,
            latency_ms,
            bandwidth_mbps,
        }
    }
}

impl DiscoveredPeer {
    /// Create new discovered peer
    pub fn new(
        peer_id: String,
        address: SocketAddr,
        peer_type: PeerType,
        discovered_via: DiscoveryMethod,
    ) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            address,
            peer_type,
            discovered_via,
            discovered_at: now,
            last_seen: now,
        }
    }

    /// Update last seen time
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Check if peer has timed out
    pub fn has_timed_out(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
}

impl UPnPDevice {
    /// Create new UPnP device
    pub fn new(device_id: String, address: SocketAddr, device_type: String) -> Self {
        Self {
            device_id,
            address,
            device_type,
            capabilities: Vec::new(),
            discovered_at: Instant::now(),
        }
    }

    /// Add capability to device
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }
}

impl TURNRelay {
    /// Create new TURN relay
    pub fn new(relay_id: String, relay_address: SocketAddr, duration: Duration) -> Self {
        let now = Instant::now();
        Self {
            relay_id,
            relay_address,
            allocated_at: now,
            expires_at: now + duration,
        }
    }

    /// Check if relay has expired
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    /// Get remaining time before expiration
    pub fn remaining_time(&self) -> Option<Duration> {
        if self.is_expired() {
            None
        } else {
            Some(self.expires_at.duration_since(Instant::now()))
        }
    }
} 