//! Canonical Network Configuration - Unified Modern Implementation
//!
//! This module provides the single, canonical `NetworkConfig` definition that replaces
//! all fragmented and deprecated network configuration structs across the codebase.
//!
//! ## Components
//! - `CanonicalNetworkConfig` - Main network configuration
//! - `PeerType` - Network topology and peer definitions
//! - `GamingNetworkConfig` - Gaming-specific configuration
//! - Supporting types: `PortRange`, `NetworkTimeouts`, `ConnectionLimits`, `CorsConfig`

use serde::{Deserialize, Serialize};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tracing::warn;

/// **CANONICAL**: Peer type in network topology
///
/// Unified from multiple definitions across:
/// - `songbird-config/src/lib.rs`
/// - `songbird-config/src/unified/network.rs`
/// - `songbird-network/src/network/discovery/types.rs`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerType {
    /// Client endpoint - initiates connections
    Client,
    /// Server endpoint - accepts connections
    Server,
    /// Peer-to-peer endpoint - both client and server
    Peer,
    /// Relay endpoint - forwards traffic between peers
    Relay,
    /// Gateway endpoint - protocol translation and routing
    Gateway,
    /// Unknown or unclassified peer type
    Unknown,
}

impl Default for PeerType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for PeerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Client => write!(f, "client"),
            Self::Server => write!(f, "server"),
            Self::Peer => write!(f, "peer"),
            Self::Relay => write!(f, "relay"),
            Self::Gateway => write!(f, "gateway"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for PeerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "client" => Ok(Self::Client),
            "server" => Ok(Self::Server),
            "peer" => Ok(Self::Peer),
            "relay" => Ok(Self::Relay),
            "gateway" => Ok(Self::Gateway),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("Unknown peer type: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_type_serialization() {
        let peer_type = PeerType::Gateway;
        let serialized = serde_json::to_string(&peer_type).expect(
            "PeerType should serialize successfully - this indicates a serde implementation issue",
        );
        let deserialized: PeerType = serde_json::from_str(&serialized)
            .expect("Serialized PeerType should deserialize successfully - this indicates a serde implementation issue");
        assert_eq!(peer_type, deserialized);
    }

    #[test]
    fn test_peer_type_display() {
        assert_eq!(PeerType::Client.to_string(), "client");
        assert_eq!(PeerType::Server.to_string(), "server");
        assert_eq!(PeerType::Gateway.to_string(), "gateway");
    }

    #[test]
    fn test_peer_type_from_str() {
        assert_eq!("client".parse::<PeerType>().unwrap(), PeerType::Client);
        assert_eq!("SERVER".parse::<PeerType>().unwrap(), PeerType::Server);
        assert_eq!("Gateway".parse::<PeerType>().unwrap(), PeerType::Gateway);
    }

    #[test]
    fn test_peer_type_default() {
        assert_eq!(PeerType::default(), PeerType::Unknown);
    }
}

// ============================================================================
// CANONICAL NETWORK CONFIGURATION
// ============================================================================

/// Canonical Network Configuration - Single Source of Truth
///
/// This struct unifies all network configuration patterns across Songbird,
/// eliminating fragmentation and providing a modern, comprehensive solution.
///
/// ## Recent Additions (Phase 2B - Nov 2025)
/// - Self-awareness configuration for primal ecosystem
/// - Universal discovery for capability-based routing
/// - Advanced networking features (reverse proxy, SSL, TURN)
/// - Network topology and measurement capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalNetworkConfig {
    // Core network settings
    pub bind_address: IpAddr,
    pub production_bind_address: IpAddr,

    // Service ports
    pub orchestrator_port: u16,
    pub discovery_port: u16,
    pub health_port: u16,
    pub dashboard_port: u16,
    pub websocket_port: u16,
    pub metrics_port: u16,
    pub federation_port: u16,

    // Gaming configuration
    pub gaming: GamingNetworkConfig,
    pub gaming_port_range: PortRange,

    // Connection management
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub max_connections: usize,
    pub max_bandwidth_mbps: u64,
    pub worker_threads: usize,

    // Security and TLS
    pub require_tls: bool,
    pub cors: CorsConfig,

    // Discovery and federation
    pub discovery_ports: Vec<u16>,
    pub federation_endpoints: Vec<String>,
    pub stun_servers: Vec<String>,
    pub allowed_networks: Vec<String>,

    // Advanced configuration
    pub timeouts: NetworkTimeouts,
    pub connection_limits: ConnectionLimits,

    // Metrics and monitoring
    pub metrics_bind_address: IpAddr,
    pub federation_bind_address: IpAddr,

    // ========================================================================
    // PHASE 2B: ADVANCED FEATURES (merged from unified/network.rs)
    // ========================================================================
    
    /// Self-awareness configuration - this primal's identity and capabilities
    /// 
    /// Enables self-knowledge pattern where each primal knows only itself,
    /// not other primals (discovered dynamically via capabilities)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_config: Option<SelfAwareConfig>,

    /// Universal discovery configuration for capability-based routing
    /// 
    /// Enables discovering services by capability rather than hardcoded names,
    /// supporting dynamic ecosystem without tight coupling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub universal_discovery: Option<UniversalDiscoveryConfig>,

    /// Reverse proxy configuration for upstream routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse_proxy: Option<ReverseProxyConfig>,

    /// Advanced SSL/TLS configuration with certificate management
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_config: Option<SslConfig>,

    /// TURN relay configuration for NAT traversal
    /// 
    /// Enables peer-to-peer connections through NAT/firewalls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_relay: Option<TURNRelay>,

    /// UPnP device configuration for local network discovery
    /// 
    /// Enables automatic discovery of devices on local network
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upnp_device: Option<UPnPDevice>,

    /// Network topology discovery configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_discovery: Option<DiscoveryNetworkTopology>,

    /// Real-time network performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_metrics: Option<NetworkMeasurement>,
}

/// Gaming-specific network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    pub starcraft_port: u16,
    pub aoe2_port: u16,
    pub ipx_port: u16,
    pub udp_port: u16,
    pub enable_lan_discovery: bool,
    pub max_players_per_game: usize,
}

impl Default for GamingNetworkConfig {
    fn default() -> Self {
        Self {
            starcraft_port: 6112,
            aoe2_port: 6113,
            ipx_port: 6112,
            udp_port: 6114,
            enable_lan_discovery: true,
            max_players_per_game: 8,
        }
    }
}

/// **CANONICAL**: Gaming network scale configuration
///
/// Defines the scale and capacity of gaming network deployments.
///
/// **Consolidated from**: `config::network::GamingScale`  
/// **Purpose**: Unified gaming scale classification for network resource allocation
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::GamingScale;
///
/// let scale = GamingScale::Home;
/// assert_eq!(scale.max_players(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamingScale {
    /// Home gaming setup (1-4 players)
    Home,
    /// LAN party setup (5-16 players)
    LanParty,
    /// Tournament setup (17-64 players)
    Tournament,
    /// Professional setup (65+ players)
    Professional,
}

impl Default for GamingScale {
    fn default() -> Self {
        Self::Home
    }
}

impl GamingScale {
    /// Get the maximum recommended players for this scale
    #[must_use]
    pub const fn max_players(&self) -> usize {
        match self {
            Self::Home => 4,
            Self::LanParty => 16,
            Self::Tournament => 64,
            Self::Professional => 256,
        }
    }

    /// Get the recommended bandwidth in Mbps for this scale
    #[must_use]
    pub const fn recommended_bandwidth_mbps(&self) -> u64 {
        match self {
            Self::Home => 10,
            Self::LanParty => 50,
            Self::Tournament => 200,
            Self::Professional => 1000,
        }
    }

    /// Get the recommended concurrent connections for this scale
    #[must_use]
    pub const fn recommended_connections(&self) -> usize {
        match self {
            Self::Home => 10,
            Self::LanParty => 50,
            Self::Tournament => 200,
            Self::Professional => 1000,
        }
    }
}

impl std::fmt::Display for GamingScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Home => write!(f, "home"),
            Self::LanParty => write!(f, "lan-party"),
            Self::Tournament => write!(f, "tournament"),
            Self::Professional => write!(f, "professional"),
        }
    }
}

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl Default for PortRange {
    fn default() -> Self {
        Self {
            start: 7000,
            end: 7100,
        }
    }
}

/// Network timeout configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    pub connection: Duration,
    pub request: Duration,
    pub health_check: Duration,
    pub default: Duration,
}

impl Default for NetworkTimeouts {
    fn default() -> Self {
        Self {
            connection: Duration::from_secs(10),
            request: Duration::from_secs(60),
            health_check: Duration::from_secs(5),
            default: Duration::from_secs(30),
        }
    }
}

/// Connection limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_connections_per_host: usize,
    pub max_total_connections: usize,
    pub max_retries: u32,
    pub pool_idle_timeout_secs: u64,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_total_connections: 100,
            max_retries: 3,
            pool_idle_timeout_secs: 300,
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        }
    }
}

impl CanonicalNetworkConfig {
    /// Create canonical network configuration from environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if the bind address environment variable is invalid
    pub fn from_env() -> SongbirdResult<Self> {
        let bind_address = SafeEnv::get_or_default("SONGBIRD_BIND_ADDRESS", "0.0.0.0")
            .parse()
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Invalid bind address: {e}"),
                field: Some("bind_address".to_string()),
                suggestion: Some("Provide a valid IP address".to_string()),
            })?;

        let config = Self {
            bind_address,
            production_bind_address: SafeEnv::get_or_default("SONGBIRD_PRODUCTION_BIND_ADDRESS", "0.0.0.0")
                .parse()
                .unwrap_or_else(|e| {
                    warn!("Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using default 0.0.0.0: {}", e);
                    std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
                }),
            orchestrator_port: SafeEnv::get_port("SONGBIRD_ORCHESTRATOR_PORT",
                SafeEnv::get_port("DEFAULT_HTTP_PORT", 8080)),
            discovery_port: 8001,
            health_port: 8002,
            dashboard_port: 3000,
            websocket_port: 8080,
            metrics_port: 8004,
            federation_port: 8005,
            gaming: GamingNetworkConfig::default(),
            gaming_port_range: PortRange::default(),
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            max_connections: 100,
            max_bandwidth_mbps: 100,
            worker_threads: 2,
            require_tls: false,
            cors: CorsConfig::default(),
            discovery_ports: vec![8001],
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
            allowed_networks: vec!["127.0.0.0/8".to_string()],
            timeouts: NetworkTimeouts::default(),
            connection_limits: ConnectionLimits::default(),
            metrics_bind_address: bind_address,
            federation_bind_address: bind_address,
            
            // Phase 2B: Advanced features (optional, None by default for backward compatibility)
            self_config: None,
            universal_discovery: None,
            reverse_proxy: None,
            ssl_config: None,
            turn_relay: None,
            upnp_device: None,
            topology_discovery: None,
            network_metrics: None,
        };

        Ok(config)
    }

    /// Validate production readiness
    ///
    /// # Errors
    ///
    /// Returns an error if the bind address is set to localhost in production
    pub fn validate_production_readiness(&self) -> SongbirdResult<()> {
        let localhost_v4 = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
        if self.bind_address == localhost_v4 {
            return Err(SongbirdError::Configuration {
                message: "Production deployment should not use localhost bind address".to_string(),
                field: Some("bind_address".to_string()),
                suggestion: Some("Use 0.0.0.0 or a specific IP address for production".to_string()),
            });
        }
        Ok(())
    }

    /// Get local bind socket address
    ///
    /// # Errors
    ///
    /// Returns an error if the socket address cannot be parsed
    pub fn local_bind_address(&self) -> SongbirdResult<SocketAddr> {
        let addr = format!("{}:{}", self.bind_address, self.orchestrator_port);
        let socket_addr = addr.parse::<SocketAddr>().map_err(|e| SongbirdError::Configuration {
            message: format!("Invalid address: {e}"),
            field: Some("address".to_string()),
            suggestion: Some("Provide a valid IP and port format".to_string()),
        })?;
        Ok(socket_addr)
    }

    /// Get gaming port for specific protocol
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is not supported (must be 'starcraft', 'ipx', 'aoe2', or 'udp')
    pub fn gaming_port(&self, protocol: &str) -> SongbirdResult<u16> {
        let port = match protocol {
            "starcraft" | "ipx" => self.gaming.starcraft_port,
            "aoe2" | "udp" => self.gaming.aoe2_port,
            _ => {
                return Err(SongbirdError::Configuration {
                    message: format!("Unknown protocol: {protocol}"),
                    field: Some("protocol".to_string()),
                    suggestion: Some("Use 'starcraft', 'aoe2', or 'udp'".to_string()),
                });
            }
        };
        Ok(port)
    }

    /// Get orchestrator endpoint
    #[must_use]
    pub fn orchestrator_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.orchestrator_port)
    }

    /// Get discovery endpoint
    #[must_use]
    pub fn discovery_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.discovery_port)
    }

    /// Get metrics endpoint
    #[must_use]
    pub fn metrics_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.metrics_bind_address, self.metrics_port)
    }

    /// Get federation endpoint
    #[must_use]
    pub fn federation_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.federation_bind_address, self.federation_port)
    }
}

impl Default for CanonicalNetworkConfig {
    fn default() -> Self {
        let bind_address = "0.0.0.0".parse().unwrap_or_else(|_| {
            warn!("Failed to parse bind address, using development default");
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        });

        Self {
            bind_address,
            production_bind_address: "0.0.0.0".parse().unwrap_or({
                // This should never fail as 0.0.0.0 is always valid, but handle gracefully
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            }),
            orchestrator_port: 8080,
            discovery_port: 8001,
            health_port: 8002,
            dashboard_port: 3000,
            websocket_port: 8080,
            metrics_port: 8004,
            federation_port: 8005,
            gaming: GamingNetworkConfig::default(),
            gaming_port_range: PortRange::default(),
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            max_connections: 50,
            max_bandwidth_mbps: 50,
            worker_threads: 2,
            require_tls: false,
            cors: CorsConfig {
                enabled: true,
                origins: vec!["http://localhost:3000".to_string()],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ],
                allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            },
            discovery_ports: vec![8001],
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
            allowed_networks: vec![
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
                "192.168.0.0/16".to_string(),
            ],
            timeouts: NetworkTimeouts::default(),
            connection_limits: ConnectionLimits::default(),
            metrics_bind_address: bind_address,
            federation_bind_address: bind_address,
            
            // Phase 2B: Advanced features (optional, None by default for backward compatibility)
            self_config: None,
            universal_discovery: None,
            reverse_proxy: None,
            ssl_config: None,
            turn_relay: None,
            upnp_device: None,
            topology_discovery: None,
            network_metrics: None,
        }
    }
}

/// Type alias for backwards compatibility during migration
pub type NetworkConfig = CanonicalNetworkConfig;

// ============================================================================
// EXTENDED NETWORK CONFIG TYPES (merged from unified/network.rs)
// ============================================================================

/// Load balancing configuration for distributing traffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Enable load balancing
    pub enabled: bool,
    /// Load balancing strategy (round_robin, least_connections, random)
    pub strategy: String,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Backend servers
    pub backends: Vec<String>,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: "round_robin".to_string(),
            health_check_interval_secs: 30,
            backends: Vec::new(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second limit
    pub requests_per_second: u32,
    /// Burst size (max requests in short burst)
    pub burst_size: u32,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

/// Connection pool configuration for efficient connection reuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,
    /// Minimum number of idle connections to maintain
    pub min_idle: usize,
    /// Maximum lifetime of a connection (in seconds)
    pub max_lifetime_secs: u64,
    /// Idle timeout (in seconds)
    pub idle_timeout_secs: u64,
    /// Connection timeout (in seconds)
    pub connection_timeout_secs: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            min_idle: 10,
            max_lifetime_secs: 1800, // 30 minutes
            idle_timeout_secs: 600,  // 10 minutes
            connection_timeout_secs: 30,
        }
    }
}

/// TCP-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// Enable TCP keep-alive
    pub keepalive: bool,
    /// Keep-alive configuration
    pub keepalive_config: TcpKeepAliveConfig,
    /// Enable TCP_NODELAY (disable Nagle's algorithm)
    pub nodelay: bool,
    /// Socket buffer sizes
    pub buffer_config: SocketBufferConfig,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            keepalive: true,
            keepalive_config: TcpKeepAliveConfig::default(),
            nodelay: true,
            buffer_config: SocketBufferConfig::default(),
        }
    }
}

/// TCP keep-alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpKeepAliveConfig {
    /// Time before sending keep-alive probes (in seconds)
    pub time_secs: u64,
    /// Interval between keep-alive probes (in seconds)
    pub interval_secs: u64,
    /// Number of keep-alive probes before giving up
    pub probes: u32,
}

impl Default for TcpKeepAliveConfig {
    fn default() -> Self {
        Self {
            time_secs: 60,
            interval_secs: 10,
            probes: 5,
        }
    }
}

/// Socket buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketBufferConfig {
    /// Receive buffer size (bytes)
    pub recv_buffer_size: usize,
    /// Send buffer size (bytes)
    pub send_buffer_size: usize,
}

impl Default for SocketBufferConfig {
    fn default() -> Self {
        Self {
            recv_buffer_size: 65536,  // 64KB
            send_buffer_size: 65536,   // 64KB
        }
    }
}

/// UDP-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    /// Enable broadcast
    pub broadcast: bool,
    /// Enable multicast
    pub multicast: bool,
    /// Multicast TTL
    pub multicast_ttl: u8,
    /// Buffer configuration
    pub buffer_config: SocketBufferConfig,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            broadcast: false,
            multicast: false,
            multicast_ttl: 1,
            buffer_config: SocketBufferConfig::default(),
        }
    }
}

/// Network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceConfig {
    /// Bind address for services
    pub bind_address: String,
    /// Bind to specific network interface (e.g., "eth0")
    pub interface_name: Option<String>,
    /// Enable IPv6
    pub ipv6_enabled: bool,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            interface_name: None,
            ipv6_enabled: true,
        }
    }
}

/// Circuit breaker configuration
///
/// **CONSOLIDATED**: Re-export of canonical version (Week 2, Nov 10 2025).
/// Field mapping: timeout_secs (u64) → timeout (Duration)
pub use super::resilience::CircuitBreakerConfig;

// Default implementation now provided by canonical/resilience

// ============================================================================
// ADVANCED FEATURES - Merged from unified/network.rs
// ============================================================================

/// Service endpoint configuration for service discovery
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Detailed service endpoint information for discovery and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
    pub scheme: String, // http, https, tcp, etc.
    pub path: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    #[must_use]
    pub fn new(host: &str, port: u16, scheme: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            scheme: scheme.to_string(),
            path: None,
            timeout_secs: None,
        }
    }

    /// Get the full URL for this endpoint
    #[must_use]
    pub fn full_url(&self) -> String {
        let base = format!("{}://{}:{}", self.scheme, self.host, self.port);
        match &self.path {
            Some(path) => format!("{base}{path}"),
            None => base,
        }
    }

    /// Create endpoint from environment variables
    ///
    /// Expects: `{env_prefix}_HOST`, `{env_prefix}_PORT`, `{env_prefix}_SCHEME`
    #[must_use]
    pub fn from_env(env_prefix: &str) -> Option<Self> {
        let host_env = format!("{env_prefix}_HOST");
        let port_env = format!("{env_prefix}_PORT");
        let scheme_env = format!("{env_prefix}_SCHEME");

        let host = std::env::var(&host_env).ok()?;
        let port = std::env::var(&port_env).ok()?.parse().ok()?;
        let scheme = std::env::var(&scheme_env).unwrap_or_else(|_| "http".to_string());

        Some(Self::new(&host, port, &scheme))
    }
}

impl Default for ServiceEndpoint {
    fn default() -> Self {
        Self::new("127.0.0.1", 8080, "http")
    }
}

/// Self-awareness configuration - each primal knows itself
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Self-knowledge pattern for primal ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAwareConfig {
    /// This primal's unique identifier
    pub id: String,
    /// This primal's endpoint information
    pub endpoint: ServiceEndpoint,
    /// Capabilities this primal provides
    pub capabilities: Vec<String>,
}

impl Default for SelfAwareConfig {
    fn default() -> Self {
        use std::env;
        Self {
            id: env::var("SONGBIRD_PRIMAL_ID").unwrap_or_else(|_| "songbird".to_string()),
            endpoint: ServiceEndpoint::from_env("SONGBIRD_SELF")
                .unwrap_or_else(|| ServiceEndpoint::new("127.0.0.1", 8080, "http")),
            capabilities: env::var("SONGBIRD_CAPABILITIES")
                .unwrap_or_else(|_| "orchestration,service_discovery,load_balancing".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

/// Universal discovery configuration for capability-based routing
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Capability-based service discovery (no hardcoded primal names)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalDiscoveryConfig {
    /// Enable universal adapter for capability-based routing
    pub enabled: bool,
    /// Discovery methods to use for finding other primals
    pub discovery_methods: Vec<String>,
    /// Service discovery endpoints (infrastructure only, not specific primals)
    pub service_discovery: ServiceDiscoveryEndpoints,
}

impl Default for UniversalDiscoveryConfig {
    fn default() -> Self {
        use std::env;
        Self {
            enabled: env::var("SONGBIRD_UNIVERSAL_DISCOVERY_ENABLED").is_ok(),
            discovery_methods: env::var("SONGBIRD_DISCOVERY_METHODS")
                .unwrap_or_else(|_| "network_scan,service_registry,mdns".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            service_discovery: ServiceDiscoveryEndpoints::default(),
        }
    }
}

/// Service discovery endpoints for infrastructure backends
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Multi-backend service discovery (Consul, etcd, Kubernetes, Docker)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryEndpoints {
    /// Consul endpoints
    pub consul: Vec<ServiceEndpoint>,
    /// etcd endpoints
    pub etcd: Vec<ServiceEndpoint>,
    /// Kubernetes API endpoints
    pub kubernetes: Vec<ServiceEndpoint>,
    /// Docker endpoints
    pub docker: Vec<ServiceEndpoint>,
}

impl Default for ServiceDiscoveryEndpoints {
    fn default() -> Self {
        Self {
            consul: vec![
                ServiceEndpoint::new("127.0.0.1", 8500, "http"),
                ServiceEndpoint::new("localhost", 8500, "http"),
            ],
            etcd: vec![
                ServiceEndpoint::new("127.0.0.1", 2379, "http"),
                ServiceEndpoint::new("127.0.0.1", 2380, "http"),
            ],
            kubernetes: vec![ServiceEndpoint::new("127.0.0.1", 8080, "https")],
            docker: vec![
                ServiceEndpoint::new("localhost", 2375, "http"),
                ServiceEndpoint::new("localhost", 2376, "https"),
            ],
        }
    }
}

/// Reverse proxy configuration
///
/// **Merged from**: `unified/network.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyConfig {
    pub enabled: bool,
    pub upstream_timeout_secs: u64,
    pub max_upstream_connections: usize,
}

impl Default for ReverseProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            upstream_timeout_secs: 30,
            max_upstream_connections: 100,
        }
    }
}

/// Advanced SSL/TLS configuration
///
/// **Merged from**: `unified/network.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        use std::env;
        Self {
            enabled: env::var("SONGBIRD_SSL_ENABLED").is_ok(),
            cert_path: env::var("SONGBIRD_SSL_CERT").ok(),
            key_path: env::var("SONGBIRD_SSL_KEY").ok(),
            ca_path: env::var("SONGBIRD_SSL_CA").ok(),
        }
    }
}

// Note: LoadBalancingConfig and RateLimitingConfig already defined above (lines 457-498)

/// TURN relay configuration for NAT traversal
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: TURN/STUN relay configuration for NAT traversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TURNRelay {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

impl TURNRelay {
    /// Create a new TURN relay configuration
    #[must_use]
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            enabled: true,
        }
    }

    /// Check if the TURN relay configuration is expired (disabled)
    #[must_use]
    pub fn is_expired(&self) -> bool {
        !self.enabled
    }
}

/// UPnP device configuration for local network discovery
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Universal Plug and Play device discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UPnPDevice {
    pub device_id: String,
    pub friendly_name: String,
    pub device_type: String,
    pub enabled: bool,
}

impl UPnPDevice {
    /// Create a new UPnP device configuration
    #[must_use]
    pub fn new(device_id: String, friendly_name: String, device_type: String) -> Self {
        Self {
            device_id,
            friendly_name,
            device_type,
            enabled: true,
        }
    }
}

/// Network topology discovery configuration
///
/// **Merged from**: `unified/network.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNetworkTopology {
    pub discovery_enabled: bool,
    pub topology_mapping: bool,
    pub peer_discovery_timeout: u64,
}

impl Default for DiscoveryNetworkTopology {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            topology_mapping: false,
            peer_discovery_timeout: 30,
        }
    }
}

/// Network connection information for unified management
///
/// **Merged from**: `unified/network.rs`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub connection_id: String,
    pub remote_addr: String,
    pub connection_type: String,
    pub established_at: u64,
}

/// Network measurement and metrics
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Real-time network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeasurement {
    pub latency_ms: u64,
    pub bandwidth_mbps: f64,
    pub packet_loss_rate: f64,
    pub jitter_ms: u64,
}

impl Default for NetworkMeasurement {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            bandwidth_mbps: 0.0,
            packet_loss_rate: 0.0,
            jitter_ms: 0,
        }
    }
}

// Note: ConnectionPoolConfig already defined above (lines 502-527)

// ============================================================================
// ADDITIONAL NETWORK CONFIGURATION STRUCTS
// ============================================================================

/// Network timeout configuration
///
/// **Merged from**: `config/network/mod.rs`  
/// **Purpose**: Centralized timeout configuration for all network operations
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::TimeoutConfig;
///
/// let timeouts = TimeoutConfig {
///     default_timeout_secs: 30,
///     connection_timeout_secs: 10,
///     health_check_timeout_secs: 5,
///     registration_timeout_secs: 15,
///     discovery_timeout_secs: 30,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimeoutConfig {
    /// Default operation timeout in seconds
    pub default_timeout_secs: u64,
    
    /// Connection establishment timeout in seconds
    pub connection_timeout_secs: u64,
    
    /// Health check timeout in seconds
    pub health_check_timeout_secs: u64,
    
    /// Service registration timeout in seconds
    pub registration_timeout_secs: u64,
    
    /// Service discovery timeout in seconds
    pub discovery_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            connection_timeout_secs: 10,
            health_check_timeout_secs: 5,
            registration_timeout_secs: 15,
            discovery_timeout_secs: 30,
        }
    }
}

/// Domain name configuration
///
/// **Merged from**: `unified/network.rs` (UnifiedDomainConfig)  
/// **Purpose**: Multi-domain deployment configuration with TLS support
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::DomainConfig;
///
/// let domain = DomainConfig {
///     domain_name: "api.example.com".to_string(),
///     tls_enabled: true,
///     certificate_path: Some("/etc/certs/cert.pem".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DomainConfig {
    /// Primary domain name
    pub domain_name: String,
    
    /// Enable TLS/SSL for this domain
    pub tls_enabled: bool,
    
    /// Path to TLS certificate file (optional)
    pub certificate_path: Option<String>,
}

impl Default for DomainConfig {
    fn default() -> Self {
        use std::env;
        Self {
            domain_name: env::var("SONGBIRD_DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
            tls_enabled: env::var("SONGBIRD_TLS_ENABLED").is_ok(),
            certificate_path: env::var("SONGBIRD_TLS_CERT").ok(),
        }
    }
}

/// HTTP/HTTPS proxy configuration
///
/// **Merged from**: `unified/network.rs`  
/// **Purpose**: Reverse proxy and forward proxy configuration
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::ProxyConfig;
///
/// let proxy = ProxyConfig {
///     enabled: true,
///     bind_address: "0.0.0.0".to_string(),
///     bind_port: 8080,
///     target_address: "127.0.0.1".to_string(),
///     target_port: 9000,
///     connection_timeout_ms: 5000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProxyConfig {
    /// Enable proxy functionality
    pub enabled: bool,
    
    /// Address to bind the proxy server
    pub bind_address: String,
    
    /// Port to bind the proxy server
    pub bind_port: u16,
    
    /// Target backend address
    pub target_address: String,
    
    /// Target backend port
    pub target_port: u16,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        use std::env;
        Self {
            enabled: env::var("SONGBIRD_PROXY_ENABLED").ok().and_then(|v| v.parse().ok()).unwrap_or(false),
            bind_address: env::var("SONGBIRD_PROXY_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
            bind_port: env::var("SONGBIRD_PROXY_BIND_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            target_address: env::var("SONGBIRD_PROXY_TARGET_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
            target_port: env::var("SONGBIRD_PROXY_TARGET_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8000),
            connection_timeout_ms: 5000,
        }
    }
}

// ============================================================================
// END OF MERGED FEATURES FROM unified/network.rs
// ============================================================================
