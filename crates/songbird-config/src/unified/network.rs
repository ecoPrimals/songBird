//! Network configuration structures

use crate::config::constants::get_bind_address;
use serde::{Deserialize, Serialize};

use std::env;

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
    pub scheme: String, // http, https, tcp, etc.
    pub path: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl ServiceEndpoint {
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

    #[must_use]
    pub fn full_url(&self) -> String {
        let base = format!("{}://{}:{}", self.scheme, self.host, self.port);
        match &self.path {
            Some(path) => format!("{base}{path}"),
            None => base,
        }
    }

    #[must_use]
    pub fn from_env(env_prefix: &str) -> Option<Self> {
        let host_env = format!("{env_prefix}_HOST");
        let port_env = format!("{env_prefix}_PORT");
        let scheme_env = format!("{env_prefix}_SCHEME");

        let host = env::var(&host_env).ok()?;
        let port = env::var(&port_env).ok()?.parse().ok()?;
        let scheme = env::var(&scheme_env).unwrap_or_else(|_| "http".to_string());

        Some(Self::new(&host, port, &scheme))
    }
}

impl Default for ServiceEndpoint {
    fn default() -> Self {
        Self::new(&get_bind_address(), 
            std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080), 
            "http")
    }
}

/// Self-awareness configuration - each primal only knows itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAwareConfig {
    /// This primal's unique identifier
    pub id: String,
    /// This primal's endpoint information
    pub endpoint: ServiceEndpoint,
    /// Capabilities this primal provides
    pub capabilities: Vec<String>,
}

/// Universal adapter configuration for capability-based discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalDiscoveryConfig {
    /// Enable universal adapter for capability-based routing
    pub enabled: bool,
    /// Discovery methods to use for finding other primals
    pub discovery_methods: Vec<String>,
    /// Service discovery endpoints (infrastructure only, not specific primals)
    pub service_discovery: ServiceDiscoveryEndpoints,
}

impl Default for SelfAwareConfig {
    fn default() -> Self {
        Self {
            id: env::var("SONGBIRD_PRIMAL_ID").unwrap_or_else(|_| "songbird".to_string()),
            endpoint: ServiceEndpoint::from_env("SONGBIRD_SELF")
                .unwrap_or_else(|| ServiceEndpoint::new(&get_bind_address(), 8080, "http")),
            capabilities: env::var("SONGBIRD_CAPABILITIES")
                .unwrap_or_else(|_| "orchestration,service_discovery,load_balancing".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

impl Default for UniversalDiscoveryConfig {
    fn default() -> Self {
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

/// Service discovery endpoints
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
                ServiceEndpoint::new(&get_bind_address(), 8500, "http"),
                ServiceEndpoint::new("localhost", 8500, "http"),
            ],
            etcd: vec![
                ServiceEndpoint::new(&get_bind_address(), 2379, "http"),
                ServiceEndpoint::new(&get_bind_address(), 2380, "http"),
            ],
            kubernetes: vec![ServiceEndpoint::new(&get_bind_address(), 8080, "https")],
            docker: vec![
                ServiceEndpoint::new("localhost", 2375, "http"),
                ServiceEndpoint::new("localhost", 2376, "https"),
            ],
        }
    }
}

/// Unified network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNetworkConfig {
    pub bind_address: String,
    pub port: u16,
    pub max_connections: usize,
    pub keepalive_timeout_secs: u64,
    pub gaming: GamingNetworkConfig,
    pub reverse_proxy: ReverseProxyConfig,
    pub ssl: SslConfig,
    pub load_balancing: LoadBalancingConfig,
    pub rate_limiting: RateLimitingConfig,
    /// Self-awareness configuration (this primal only knows itself)
    pub self_config: SelfAwareConfig,
    /// Universal discovery configuration for capability-based routing
    pub universal_discovery: UniversalDiscoveryConfig,
    /// Connection pool configuration
    pub connection_pool: ConnectionPoolConfig,
    /// Network interface configuration
    pub interface: NetworkInterfaceConfig,
    /// TCP configuration
    pub tcp: TcpConfig,
    /// UDP configuration
    pub udp: UdpConfig,
    /// Legacy compatibility fields
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
}

fn default_enabled() -> bool {
    true
}

impl Default for UnifiedNetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: env::var("SONGBIRD_BIND_ADDRESS").map_or_else(
                |_| get_bind_address(),
                |addr| {
                    // Validate the IP address and fall back to default if invalid
                    match addr.parse::<std::net::IpAddr>() {
                        Ok(songbird_errors::evolved_success(_)) => addr,
                        Err(_) => get_bind_address(),
                    }
                },
            ),
            port: env::var("SONGBIRD_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            max_connections: 1000,
            keepalive_timeout_secs: 30,
            gaming: GamingNetworkConfig::default(),
            reverse_proxy: ReverseProxyConfig::default(),
            ssl: SslConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            self_config: SelfAwareConfig::default(),
            universal_discovery: UniversalDiscoveryConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
            interface: NetworkInterfaceConfig::default(),
            tcp: TcpConfig::default(),
            udp: UdpConfig::default(),
            // Legacy compatibility fields
            enabled: true,
            host: "localhost".to_string(),
        }
    }
}

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    pub enable_gaming_protocols: bool,
    pub directplay_port: Option<u16>,
    pub ipx_support: bool,
    pub netbios_support: bool,
    pub gaming_session_timeout_secs: u64,
    pub max_gaming_sessions: usize,
    pub packet_buffer_size: usize,
    // Add missing port_range field for backward compatibility
    pub port_range: (u16, u16),
}

impl Default for GamingNetworkConfig {
    fn default() -> Self {
        Self {
            enable_gaming_protocols: env::var("SONGBIRD_ENABLE_GAMING")
                .map(|val| val.to_lowercase() == "true" || val == "1")
                .unwrap_or(true), // Default to enabled
            directplay_port: env::var("SONGBIRD_DIRECTPLAY_PORT")
                .ok()
                .and_then(|p| p.parse().ok()),
            ipx_support: true,
            netbios_support: true,
            gaming_session_timeout_secs: 300,
            max_gaming_sessions: 100,
            packet_buffer_size: 65536,
            port_range: (8000, 8100), // Add default port range
        }
    }
}

/// Reverse proxy configuration
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

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_SSL_ENABLED").is_ok(),
            cert_path: env::var("SONGBIRD_SSL_CERT").ok(),
            key_path: env::var("SONGBIRD_SSL_KEY").ok(),
            ca_path: env::var("SONGBIRD_SSL_CA").ok(),
        }
    }
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: String,
    pub health_check_interval_secs: u64,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: env::var("SONGBIRD_LB_STRATEGY")
                .unwrap_or_else(|_| "round_robin".to_string()),
            health_check_interval_secs: 30,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub max_requests_per_minute: usize,
    pub burst_size: usize,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_RATE_LIMIT_ENABLED").is_ok(),
            max_requests_per_minute: 1000,
            burst_size: 100,
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub recovery_timeout_seconds: u64,
}

impl Default for UnifiedCircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            timeout_seconds: 60,
            recovery_timeout_seconds: 30,
        }
    }
}

/// TURN relay configuration for NAT traversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TURNRelay {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

impl TURNRelay {
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

    /// Check if the TURN relay configuration has expired
    ///
    /// **ORCHESTRATION**: Configuration management is part of Songbird's role
    #[must_use]
    pub fn is_expired(&self) -> bool {
        // ✅ IMPLEMENTED: Simple expiration based on enabled status
        // In production, this would check against creation timestamp or last validation
        // For now, disabled relays are considered "expired"
        !self.enabled
    }
}

/// `UPnP` device configuration for local network discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UPnPDevice {
    pub device_id: String,
    pub friendly_name: String,
    pub device_type: String,
    pub enabled: bool,
}

impl UPnPDevice {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNetworkTopology {
    pub discovery_enabled: bool,
    pub topology_mapping: bool,
    pub peer_discovery_timeout: u64,
}

/// Network connection information for unified management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub connection_id: String,
    pub remote_addr: String,
    pub connection_type: String,
    pub established_at: u64,
}

/// Network measurement and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeasurement {
    pub latency_ms: u64,
    pub bandwidth_mbps: f64,
    pub packet_loss_rate: f64,
    pub jitter_ms: u64,
}

/// Unified connection information for network management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConnectionInfo {
    pub connection_id: String,
    pub remote_address: String,
    pub status: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Unified domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDomainConfig {
    pub domain_name: String,
    pub tls_enabled: bool,
    pub certificate_path: Option<String>,
}

/// Unified network manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNetworkManager {
    pub enabled: bool,
    pub management_port: u16,
    pub auto_discovery: bool,
}

/// Unified SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSslConfig {
    pub enabled: bool,
    pub certificate_path: String,
    pub private_key_path: String,
    pub ca_certificate_path: Option<String>,
}

/// Degradation severity levels for system health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradationSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Unified gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGamingConfig {
    pub enabled: bool,
    pub protocols: Vec<String>,
    pub port_range: (u16, u16),
    pub max_sessions: usize,
}

impl Default for UnifiedGamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            protocols: vec!["IPX".to_string(), "TCP".to_string(), "UDP".to_string()],
            port_range: (7000, 8000),
            max_sessions: 100,
        }
    }
}

/// Unified network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedNetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_active: usize,
    pub connections_total: u64,
    pub last_updated: u64,
}

/// Discovered peer information for network discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub address: String,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
    pub status: String,
}

impl DiscoveredPeer {
    #[must_use]
    pub fn new(peer_id: String, address: String) -> Self {
        Self {
            peer_id,
            address,
            capabilities: Vec::new(),
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: "active".to_string(),
        }
    }
}

/// Connection quality metrics for network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionQuality {
    pub latency_ms: u64,
    pub bandwidth_mbps: f64,
    pub packet_loss_rate: f64,
    pub stability_score: f64,
    pub last_measured: u64,
}

impl Default for ConnectionQuality {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            bandwidth_mbps: 0.0,
            packet_loss_rate: 0.0,
            stability_score: 1.0,
            last_measured: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Proxy configuration for network routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub bind_port: u16,
    pub target_address: String,
    pub target_port: u16,
    pub connection_timeout_ms: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: get_bind_address().to_string(),
            bind_port: 8080,
            target_address: get_bind_address().to_string(),
            target_port: 8081,
            connection_timeout_ms: 5000,
        }
    }
}

/// Proxy statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyStats {
    pub total_connections: u64,
    pub active_connections: usize,
    pub bytes_transferred: u64,
    pub requests_processed: u64,
    pub errors_count: u64,
}

// HttpRequest - use your preferred HTTP crate's Request type directly

// ============================================================================
// ADVANCED NETWORK CONFIGURATION - CONSOLIDATED FROM UNIVERSAL-PRIMALS
// ============================================================================

/// Connection pool configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,
    /// Minimum number of connections
    pub min_connections: usize,
    /// Connection timeout (in seconds)
    pub connection_timeout_seconds: u64,
    /// Idle timeout for connections (in seconds)
    pub idle_timeout_seconds: u64,
    /// Maximum lifetime of a connection (in seconds)
    pub max_lifetime_seconds: u64,
    /// Connection validation query
    pub validation_query: Option<String>,
    /// Whether to validate connections on borrow
    pub validate_on_borrow: bool,
    /// Whether to validate connections on return
    pub validate_on_return: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: env::var("SONGBIRD_POOL_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            min_connections: env::var("SONGBIRD_POOL_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            connection_timeout_seconds: env::var("SONGBIRD_POOL_CONNECTION_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            idle_timeout_seconds: env::var("SONGBIRD_POOL_IDLE_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(600),
            max_lifetime_seconds: env::var("SONGBIRD_POOL_MAX_LIFETIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            validation_query: env::var("SONGBIRD_POOL_VALIDATION_QUERY").ok(),
            validate_on_borrow: env::var("SONGBIRD_POOL_VALIDATE_ON_BORROW")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            validate_on_return: env::var("SONGBIRD_POOL_VALIDATE_ON_RETURN")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}

/// Network interface configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceConfig {
    /// Bind address for services
    pub bind_address: String,
    /// Network interface to bind to
    pub interface: Option<String>,
    /// Whether to enable IPv6
    pub ipv6_enabled: bool,
    /// Socket buffer sizes
    pub socket_buffer: SocketBufferConfig,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            bind_address: env::var("SONGBIRD_INTERFACE_BIND")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            interface: env::var("SONGBIRD_NETWORK_INTERFACE").ok(),
            ipv6_enabled: env::var("SONGBIRD_IPV6_ENABLED")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            socket_buffer: SocketBufferConfig::default(),
        }
    }
}

/// Socket buffer configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketBufferConfig {
    /// Receive buffer size
    pub receive_buffer_size: usize,
    /// Send buffer size
    pub send_buffer_size: usize,
}

impl Default for SocketBufferConfig {
    fn default() -> Self {
        Self {
            receive_buffer_size: env::var("SONGBIRD_SOCKET_RECV_BUFFER")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8192),
            send_buffer_size: env::var("SONGBIRD_SOCKET_SEND_BUFFER")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8192),
        }
    }
}

/// TCP configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// Whether to enable TCP keep-alive
    pub keepalive_enabled: bool,
    /// TCP keep-alive configuration
    pub keepalive: TcpKeepAliveConfig,
    /// Whether to enable Nagle's algorithm
    pub nodelay: bool,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            keepalive_enabled: env::var("SONGBIRD_TCP_KEEPALIVE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            keepalive: TcpKeepAliveConfig::default(),
            nodelay: env::var("SONGBIRD_TCP_NODELAY")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        }
    }
}

/// TCP keep-alive configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpKeepAliveConfig {
    /// Time before sending keep-alive probes (in seconds)
    pub time: u64,
    /// Interval between keep-alive probes (in seconds)
    pub interval: u64,
    /// Number of keep-alive probes
    pub probes: u32,
}

impl Default for TcpKeepAliveConfig {
    fn default() -> Self {
        Self {
            time: env::var("SONGBIRD_TCP_KEEPALIVE_TIME")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(7200),
            interval: env::var("SONGBIRD_TCP_KEEPALIVE_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(75),
            probes: env::var("SONGBIRD_TCP_KEEPALIVE_PROBES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(9),
        }
    }
}

/// UDP configuration (consolidated from songbird-universal-primals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    /// Whether to enable broadcast
    pub broadcast_enabled: bool,
    /// Whether to enable multicast
    pub multicast_enabled: bool,
    /// Multicast TTL
    pub multicast_ttl: u32,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            broadcast_enabled: env::var("SONGBIRD_UDP_BROADCAST")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            multicast_enabled: env::var("SONGBIRD_UDP_MULTICAST")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            multicast_ttl: env::var("SONGBIRD_UDP_MULTICAST_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
        }
    }
}
