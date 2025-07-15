//! Federation types and data structures

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

impl std::fmt::Display for NodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.addr)
    }
}

/// Federation node types based on network topology
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Tower node (basement server, compute node)
    Tower {
        location: String,
        capabilities: TowerCapabilities,
    },
    /// Edge node (laptop, mobile device)
    Edge { mobility: MobilityLevel },
    /// Gateway node (internet bridge, regional hub)
    Gateway { region: String, bandwidth_mbps: u32 },
    /// Relay node (worldwide mesh connector)
    Relay {
        tier: RelayTier,
        global_endpoints: Vec<String>,
    },
}

/// Tower capabilities for HPC federation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TowerCapabilities {
    /// CPU cores available
    pub cpu_cores: u32,
    /// Memory in GB
    pub memory_gb: u32,
    /// Storage in TB
    pub storage_tb: u32,
    /// GPU count and types
    pub gpus: Vec<GpuInfo>,
    /// Network bandwidth in Mbps
    pub network_bandwidth_mbps: u32,
    /// Specialized capabilities
    pub specializations: Vec<String>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuInfo {
    pub model: String,
    pub memory_gb: u32,
    pub compute_capability: String,
}

/// Node mobility level for routing optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MobilityLevel {
    /// Stationary (desktop, server)
    Stationary,
    /// Portable (laptop with power)
    Portable,
    /// Mobile (battery powered, changing networks)
    Mobile,
}

/// Relay tier for global mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelayTier {
    /// Regional relay (country/state level)
    Regional,
    /// Continental relay (cross-continent)
    Continental,
    /// Global relay (worldwide backbone)
    Global,
}

/// Network proximity levels for routing optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum NetworkProximity {
    /// Same machine (localhost)
    Localhost = 0,
    /// Same LAN (< 1ms)
    Local = 1,
    /// Same building/campus (< 5ms)  
    Campus = 2,
    /// Same city (< 20ms)
    City = 3,
    /// Same region/state (< 50ms)
    Regional = 4,
    /// Same country (< 100ms)
    National = 5,
    /// Same continent (< 200ms)
    Continental = 6,
    /// Worldwide (> 200ms)
    Global = 7,
}

/// Federation node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNode {
    /// Unique node identifier
    pub node_id: Uuid,
    /// Human-readable node name
    pub name: String,
    /// Node type and capabilities
    pub node_type: NodeType,
    /// Network addresses
    pub addresses: Vec<NodeAddress>,
    /// Current network proximity
    pub proximity: NetworkProximity,
    /// BearDog security session
    pub security_session: Option<SecuritySession>,
    /// Performance metrics
    pub metrics: NodeMetrics,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Node status
    pub status: NodeStatus,
}

/// Node network address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAddress {
    /// Socket address
    pub addr: SocketAddr,
    /// Address type (local, public, tunnel, etc.)
    pub addr_type: AddressType,
    /// Measured latency in milliseconds
    pub latency_ms: Option<u32>,
    /// Bandwidth in Mbps
    pub bandwidth_mbps: Option<u32>,
    /// Address preference score (higher = better)
    pub preference: u32,
}

/// Network address types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressType {
    /// Local LAN address
    Local,
    /// Public internet address
    Public,
    /// BearDog encrypted tunnel
    Tunnel,
    /// IPv6 address
    IPv6,
    /// Relay/proxy address
    Relay,
}

/// BearDog security session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySession {
    /// Session identifier
    pub session_id: String,
    /// Encryption key fingerprint
    pub key_fingerprint: String,
    /// Security level
    pub security_level: String,
    /// Session established time
    pub established_at: DateTime<Utc>,
    /// Session expires at
    pub expires_at: DateTime<Utc>,
}

/// Node performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// CPU utilization percentage
    pub cpu_usage: f32,
    /// Memory utilization percentage
    pub memory_usage: f32,
    /// Network latency in milliseconds
    pub network_latency_ms: u32,
    /// Bandwidth utilization in Mbps
    pub bandwidth_usage_mbps: u32,
    /// Active BYOB deployments
    pub active_deployments: u32,
    /// Load score (higher = more loaded)
    pub load_score: f32,
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    /// Node is online and available
    Online,
    /// Node is online but busy/overloaded
    Busy,
    /// Node is degraded performance
    Degraded,
    /// Node is offline
    Offline,
    /// Node is unreachable
    Unreachable,
    /// Node status is unknown
    Unknown,
}

/// Discovery protocols supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryProtocol {
    /// Local network mDNS/DNS-SD
    MDNS,
    /// UPnP SSDP discovery
    UPnP,
    /// STUN/TURN for NAT traversal
    STUN,
    /// BearDog secure discovery
    BearDog,
    /// Manual configuration
    Manual,
}

/// Route optimization strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteStrategy {
    /// Minimize latency (gaming)
    LowLatency,
    /// Maximize bandwidth (data transfer)
    HighBandwidth,
    /// Minimize cost (cloud usage)
    LowCost,
    /// Balance performance and cost
    Balanced,
}

/// Network topology cache
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    /// Network graph (node -> connected nodes)
    pub graph: HashMap<Uuid, HashSet<Uuid>>,
    /// Proximity matrix (cached for performance)
    pub proximity_matrix: HashMap<(Uuid, Uuid), NetworkProximity>,
    /// Route table (source -> destination -> best path)
    pub route_table: HashMap<(Uuid, Uuid), Vec<Uuid>>,
    /// Last topology update
    pub last_updated: Instant,
}

/// Discovery intervals configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryIntervals {
    /// Local LAN discovery interval
    pub local_discovery: Duration,
    /// Regional discovery interval
    pub regional_discovery: Duration,
    /// Global discovery interval
    pub global_discovery: Duration,
    /// Topology refresh interval
    pub topology_refresh: Duration,
}

/// Discovery result
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Discovered nodes
    pub nodes: Vec<FederationNode>,
    /// Discovery method used
    pub method: DiscoveryProtocol,
    /// Discovery timestamp
    pub discovered_at: Instant,
    /// Result confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Route information
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// Route path (sequence of node IDs)
    pub path: Vec<Uuid>,
    /// Expected latency in milliseconds
    pub expected_latency_ms: u32,
    /// Expected bandwidth in Mbps
    pub expected_bandwidth_mbps: u32,
    /// Route quality score (0.0 - 1.0)
    pub quality_score: f32,
    /// Last measured at
    pub measured_at: Instant,
}

/// Performance snapshot for route optimization
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Timestamp
    pub timestamp: Instant,
    /// Latency in milliseconds
    pub latency_ms: u32,
    /// Bandwidth in Mbps
    pub bandwidth_mbps: u32,
    /// Packet loss percentage
    pub packet_loss: f32,
    /// Jitter in milliseconds
    pub jitter_ms: u32,
}

/// Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Local node configuration
    pub local_node: LocalNodeConfig,
    /// Discovery configuration
    pub discovery: DiscoveryConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
    /// Federation limits
    pub limits: FederationLimits,
}

/// Local node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNodeConfig {
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Listening addresses
    pub listen_addresses: Vec<SocketAddr>,
    /// Public addresses (for internet connectivity)
    pub public_addresses: Vec<SocketAddr>,
    /// Location information
    pub location: Option<String>,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enabled discovery protocols
    pub enabled_protocols: Vec<DiscoveryProtocol>,
    /// Discovery intervals
    pub intervals: DiscoveryIntervals,
    /// Maximum discovery range
    pub max_range: NetworkProximity,
    /// Bootstrap nodes for initial discovery
    pub bootstrap_nodes: Vec<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable BearDog encryption
    pub enable_beardog: bool,
    /// Security level requirement
    pub required_security_level: String,
    /// Trusted node certificates
    pub trusted_nodes: Vec<String>,
    /// Security session timeout
    pub session_timeout: Duration,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Route optimization strategy
    pub route_strategy: RouteStrategy,
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
    /// Route cache TTL
    pub route_cache_ttl: Duration,
    /// Maximum hops for routes
    pub max_route_hops: u32,
}

/// Federation limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationLimits {
    /// Maximum federated nodes
    pub max_nodes: u32,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Maximum route length
    pub max_route_length: u32,
    /// Rate limiting
    pub rate_limits: RateLimits,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Discovery requests per minute
    pub discovery_per_minute: u32,
    /// Route requests per minute
    pub route_requests_per_minute: u32,
    /// Data transfer rate in Mbps
    pub max_transfer_rate_mbps: u32,
}

/// Federation events
#[derive(Debug, Clone)]
pub enum FederationEvent {
    /// Node discovered
    NodeDiscovered { node: Box<FederationNode> },
    /// Node lost/disconnected
    NodeLost { node_id: String },
    /// Data synchronization event
    DataSync { event: DataSyncEvent },
    /// Resource usage alert
    ResourceAlert { alert: ResourceAlert },
    /// Security event
    SecurityEvent { event: String },
    /// Performance alert
    PerformanceAlert { alert: PerformanceAlert },
}

/// Topology change events
#[derive(Debug, Clone)]
pub enum TopologyChange {
    /// Node added to topology
    NodeAdded { node_id: Uuid },
    /// Node removed from topology
    NodeRemoved { node_id: Uuid },
    /// Link added between nodes
    LinkAdded { from: Uuid, to: Uuid },
    /// Link removed between nodes
    LinkRemoved { from: Uuid, to: Uuid },
    /// Route updated
    RouteUpdated { route: RouteInfo },
}

/// Data synchronization events
#[derive(Debug, Clone)]
pub enum DataSyncEvent {
    /// Data replication started
    ReplicationStarted { source: String, target: String },
    /// Data replication completed
    ReplicationCompleted { source: String, target: String },
    /// Data conflict detected
    ConflictDetected {
        source: String,
        target: String,
        conflict_type: String,
    },
}

/// Resource alert information
#[derive(Debug, Clone)]
pub struct ResourceAlert {
    pub resource_type: String,
    pub threshold: f64,
    pub current_value: f64,
    pub severity: String,
}

/// Performance alert information
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub metric: String,
    pub threshold: f64,
    pub current_value: f64,
    pub node_id: String,
}

/// External IP information from STUN
#[derive(Debug, Clone)]
pub struct ExternalIPInfo {
    /// External IP address
    pub external_ip: String,
    /// External port
    pub external_port: u16,
    /// STUN server that provided this information
    pub server: String,
    /// NAT type detected
    pub nat_type: NATType,
    /// Inferred geographic region
    pub region: String,
}

/// Bootstrap peer information
#[derive(Debug, Clone)]
pub struct BootstrapPeerInfo {
    /// Peer address
    pub address: String,
    /// Peer region
    pub region: String,
    /// Peer bandwidth in Mbps
    pub bandwidth_mbps: Option<u32>,
    /// Peer latency in ms
    pub latency_ms: Option<u32>,
    /// Peer preference score
    pub preference: Option<u32>,
    /// Peer capabilities
    pub capabilities: Vec<String>,
}

/// NAT type detected during STUN
#[derive(Debug, Clone)]
pub enum NATType {
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT
    SymmetricNAT,
    /// Symmetric NAT (alias for compatibility)
    Symmetric,
    /// No NAT (public IP)
    None,
    /// Unknown NAT type
    Unknown,
    /// NAT detection timeout
    Timeout,
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// Interface IP address
    pub ip: String,
    /// Broadcast IP address
    pub broadcast_ip: String,
    /// Subnet mask
    pub subnet_mask: String,
}

/// Service information for discovery
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service identifier
    pub service_id: String,
    /// Service type
    pub service_type: String,
    /// Service name (alias for service_type for compatibility)
    pub service_name: String,
    /// Service endpoint
    pub endpoint: String,
    /// Service endpoints (for multiple endpoints)
    pub endpoints: Vec<String>,
    /// Service status
    pub status: String,
    /// Service health status
    pub health_status: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service version
    pub version: String,
    /// Service location
    pub location: Option<String>,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Topology analysis result
#[derive(Debug, Clone)]
pub struct TopologyAnalysis {
    /// Number of nodes in the topology
    pub node_count: usize,
    /// Number of edges in the topology
    pub edge_count: usize,
    /// Health percentage (0.0 - 1.0)
    pub health_percentage: f32,
    /// Whether the topology needs optimization
    pub needs_optimization: bool,
}

/// Federated deployment requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedDeploymentRequirements {
    /// Required node count
    pub node_count: u32,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Geographic constraints
    pub geographic_constraints: Option<GeographicConstraints>,
    /// Performance requirements
    pub performance_requirements: PerformanceRequirements,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
}

/// Geographic constraints for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConstraints {
    /// Preferred regions
    pub preferred_regions: Vec<String>,
    /// Maximum latency between nodes
    pub max_inter_node_latency_ms: u32,
    /// Data residency requirements
    pub data_residency: Option<String>,
}

/// Performance requirements for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Minimum CPU cores per node
    pub min_cpu_cores: u32,
    /// Minimum memory per node in GB
    pub min_memory_gb: u32,
    /// Minimum bandwidth between nodes in Mbps
    pub min_bandwidth_mbps: u32,
    /// Maximum acceptable latency in ms
    pub max_latency_ms: u32,
}

/// Security requirements for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Required security level
    pub security_level: String,
    /// Require BearDog encryption
    pub require_beardog: bool,
    /// Trusted node certificates
    pub trusted_certificates: Vec<String>,
}

/// Federated deployment result
#[derive(Debug, Clone)]
pub struct FederatedDeploymentResult {
    /// Deployment identifier
    pub deployment_id: Uuid,
    /// Team identifier
    pub team_id: String,
    /// Nodes where deployment was placed
    pub deployed_nodes: Vec<Uuid>,
    /// Deployment status
    pub deployment_status: String,
    /// Service endpoints
    pub endpoints: HashMap<String, String>,
    /// Deployment created timestamp
    pub created_at: DateTime<Utc>,
}

/// Federation status information
#[derive(Debug, Clone)]
pub struct FederationStatus {
    /// Local node information
    pub local_node: FederationNode,
    /// Total federated nodes
    pub total_nodes: u32,
    /// Online nodes count
    pub online_nodes: u32,
    /// Topology edges count
    pub topology_edges: u32,
    /// Overall federation health (0.0 - 1.0)
    pub federation_health: f32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

// Additional deployment-related types
#[derive(Debug, Clone)]
pub struct NodeDeploymentTask {
    pub node_id: Uuid,
    pub deployment_id: Uuid,
    pub team_id: String,
    pub config: DeploymentConfig,
    pub security_session: SecuritySession,
}

#[derive(Debug, Clone)]
pub struct NodeDeploymentResult {
    pub node_id: Uuid,
    pub deployment_id: Uuid,
    pub success: bool,
    pub endpoints: HashMap<String, String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub services: Vec<ServiceDefinition>,
    pub networking: NetworkingConfig,
    pub security: SecurityConfig,
    pub resources: ResourceConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    pub name: String,
    pub image: String,
    pub ports: Vec<u16>,
    pub env_vars: HashMap<String, String>,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub disk_gb: u32,
}

#[derive(Debug, Clone)]
pub struct NetworkingConfig {
    pub subnet: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub cpu_limit: f32,
    pub memory_limit_mb: u32,
    pub disk_limit_gb: u32,
}

#[derive(Debug, Clone)]
pub struct SecureConnection {
    pub node_id: Uuid,
    pub session: SecuritySession,
    pub endpoint: String,
}

// Default implementations
impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            proximity_matrix: HashMap::new(),
            route_table: HashMap::new(),
            last_updated: Instant::now(),
        }
    }
}

impl Default for DiscoveryIntervals {
    fn default() -> Self {
        Self {
            local_discovery: Duration::from_secs(30),
            regional_discovery: Duration::from_secs(300),
            global_discovery: Duration::from_secs(3600),
            topology_refresh: Duration::from_secs(60),
        }
    }
}
