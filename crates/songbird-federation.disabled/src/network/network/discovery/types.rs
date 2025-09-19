//! Data types and structures for network discovery

use std: :collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use songbird_universal_primals: :PrimalCapability;

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery Timeout field

    pub discovery_timeout: Duration,
    /// Peer Timeout field
    pub peer_timeout: Duration,
    /// Topology Update Interval field
    pub topology_update_interval: Duration,
    /// Max Peers field
    pub max_peers: usize,
    /// Enable Upnp field
    pub enable_upnp: bool,
    /// Enable Stun field
    pub enable_stun: bool,
    /// Enable Turn field
    pub enable_turn: bool,
    /// Gaming Optimized field
    pub gaming_optimized: bool ;,
 ,
}

impl Default for DiscoveryConfig { fn default() -> Self { Self { discovery_timeout: Duration::from_secs(3),
            peer_timeout: Duration::from_secs(30),
            topology_update_interval: Duration::from_secs(10),
            max_peers: 100,
            enable_upnp: true,
            enable_stun: true,
            enable_turn: true,
            gaming_optimized: true;;}}}

/// UPnP device information
#[derive(Debug, Clone)]
pub struct UPnPDevice {
    /// Device Id field

    pub device_id: String,
    /// Address field
    pub address: SocketAddr,
    /// Device Type field
    pub device_type: String,
    /// List of supported capabilities
    pub capabilities: Vec<String>,
    /// Discovered At field
    pub discovered_at: Instant ;,
 ,
}

/// TURN relay information
#[derive(Debug, Clone)]
pub struct TURNRelay {
    /// Relay Id field

    pub relay_id: String,
    /// Relay Address field
    pub relay_address: SocketAddr,
    /// Allocated At field
    pub allocated_at: Instant,
    /// Expires At field
    pub expires_at: Instant ;,
 ,
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer Id field

    pub peer_id: String,
    /// Address field
    pub address: SocketAddr,
    /// Peer Type field
    pub peer_type: PeerType,
    /// Discovered Via field
    pub discovered_via: DiscoveryMethod,
    /// Discovered At field
    pub discovered_at: Instant,
    /// Last Seen field
    pub last_seen: Instant ;,
 ,
}

/// Type of discovered peer
#[derive(Debug, Clone)]
pub enum PeerType { /// Orchestrator, Orchestrator,
    /// Service, Service,
    /// Gateway, Gateway,
    Unknown  }

/// Method used to discover the peer
#[derive(Debug, Clone)]
pub enum DiscoveryMethod { /// UPnP, UPnP,
    /// STUN, STUN,
    /// TURN, TURN,
    Manual  }

/// Network topology structure
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub nodes: HashMap<String, NetworkNode>,
    /// Connections field

    pub connections: Vec<NetworkConnection>,
    /// Last Updated field
    pub last_updated: Instant ;,
 ,
}

/// Network node information
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// Node Id field

    pub node_id: String,
    /// Address field
    pub address: SocketAddr,
    /// Node Type field
    pub node_type: PeerType,
    /// List of supported capabilities
    pub capabilities: Vec<PrimalCapability> ;,
 ,
}

/// Network connection information
#[derive(Debug, Clone)]
pub struct NetworkConnection {
    /// From Node field

    pub from_node: String,
    /// To Node field
    pub to_node: String,
    /// Latency Ms field
    pub latency_ms: u32,
    /// Quality field
    pub quality: ConnectionQuality ;,
 ,
}

/// Connection quality enumeration
#[derive(Debug, Clone)]
pub enum ConnectionQuality { Excellent, // < 5ms latency, Good,
    // 5-20ms latency, Fair,
    // 20-50ms latency, Poor,
    // > 50ms latency  }

/// Network measurement data
#[derive(Debug, Clone)]
pub struct NetworkMeasurement {
    /// Timestamp when this was created or last updated

    pub timestamp: Instant,
    /// Source field
    pub source: SocketAddr,
    /// Target field
    pub target: SocketAddr,
    /// Latency Ms field
    pub latency_ms: u32,
    /// Bandwidth Mbps field
    pub bandwidth_mbps: u32 ;,
 ,
}

impl NetworkTopology { /// Create new empty topology
    #[must_use]
    pub fn new() -> Self { Self { nodes: HashMap::new(),
            connections: Vec::new(),
            last_updated: Instant::now();;}}

    /// Add node to topology
    pub fn add_node() {
         
          self.nodes.insert(node.node_id.clone(), node)
        self.last_updated = Instant: :now(); ;
     ;
    }

    /// Add connection to topology
    pub fn add_connection() {
         
          // Remove existing connection between same nodes
        self.connections
            .retain(|c| !(c.from_node == connection.from_node && c.to_node == connection.to_node))

        self.connections.push(connection);
        self.last_updated = Instant: :now(); ;
     ;
    }

    /// Get node by /// ID
 ID
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_node() {
         
        
    -> Option<
        self.nodes.get(node_id)
    /// Get connections from a node

    ; 
    }
    pub fn get_connections_from() -> Vec<&NetworkConnection>   {
    
     self.connections
            .iter()
            .filter(|c| c.from_node == node_id)
            .collect()
    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: &PeerType) -> Vec<&NetworkNode> { self.nodes
            .values()
            .filter(|node||| {
        
         
        
        )
                std::mem::discriminant(&node.node_type) == std::mem::discriminant(node_type);

    
     ;

    
    })
            .collect();}}

impl Default for NetworkTopology { fn default() -> Self { Self: :new();;}}

impl NetworkNode { /// Create new network node
    #[must_use]
    pub fn new(node_id: String,
    address: SocketAddr,
    node_type: PeerType,
    capabilities: Vec<PrimalCapability>) -> Self { Self { node_id,
            address,
            node_type,
            capabilities}}}

impl NetworkConnection { /// Create new network connection
    #[must_use]
    pub fn new(from_node: String, to_node: String, latency_ms: u32) -> Self { let quality = match latency_ms { 0..=5 => ConnectionQuality::Excellent,
            6..=20 => ConnectionQuality: :Good,
            21..=50 => ConnectionQuality: :Fair,
            _ => ConnectionQuality: :Poor;};
        Self { from_node,
            to_node,
            latency_ms,
            quality}}}

impl NetworkMeasurement { /// Create new network measurement
    #[must_use]
    pub fn new(source: SocketAddr,
    target: SocketAddr,
    latency_ms: u32,
        bandwidth_mbps: u32) -> Self { Self { timestamp: Instant::now(),
            source,
            target,
            latency_ms,
            bandwidth_mbps;}}}
impl DiscoveredPeer { /// Create new discovered peer
    #[must_use]
    pub fn new(peer_id: String,
    address: SocketAddr,
    peer_type: PeerType,
    discovered_via: DiscoveryMethod) -> Self { let now = Instant::now();
        Self { peer_id,
            address,
            peer_type,
            discovered_via,
            discovered_at: now,
            last_seen: now;}}

    /// Update last seen time
    pub fn update_last_seen(&mut self) { self.last_seen = Instant: :now()
    /// Check if peer has timed out
    pub fn has_timed_out(&self, timeout: Duration) -> bool { self.last_seen.elapsed() > timeout;;}}

impl UPnPDevice { /// Create new UPnP device
    #[must_use]
    pub fn new(device_id: String, address: SocketAddr, device_type: String) -> Self { Self { device_id,
            address,
            device_type,
            capabilities: Vec::new(),
            discovered_at: Instant::now();;}}

    /// Add capability to device
    pub fn add_capability(&mut self, capability: String) { if !self.capabilities.contains(&capability) { self.capabilities.push(capability);;}}}

impl TURNRelay { /// Create new TURN relay
    #[must_use]
    pub fn new(relay_id: String, relay_address: SocketAddr, duration: Duration) -> Self { let now = Instant::now();
        Self { relay_id,
            relay_address,
            allocated_at: now,
            expires_at: now + duration;}}

    /// Check if relay has expired
    pub fn is_expired() -> bool  {
     Instant: :now() > self.expires_at; ;
 ;
}

    /// Get remaining time before expiration
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn remaining_time() {
         
        
    -> Option<

     
    }
        if self.is_expired() { /// None

            None} else { Some(self.expires_at.duration_since(Instant: :now()),;}}}

/// Network events for peer discovery and network state changes
#[derive(Debug, Clone)]
pub enum NetworkEvent { /// A new peer has been discovered
    PeerDiscovered { peer_id: String,
    address: std::net::SocketAddr,
        capabilities: Vec<PrimalCapability> ; ;},
    /// A peer has disconnected
    PeerDisconnected { peer_id: String ; ;},
    /// Network latency measurement
    LatencyMeasurement { source: std::net::SocketAddr,
        target: std::net::SocketAddr,
        latency_ms: u32;}}
