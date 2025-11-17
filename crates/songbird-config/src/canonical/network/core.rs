//! Core Network Configuration
//!
//! Core network structures including the main `CanonicalNetworkConfig`,
//! `PeerType`, and fundamental network settings.

use serde::{Deserialize, Serialize};
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tracing::warn;

use super::advanced::{
    DiscoveryNetworkTopology, NetworkMeasurement, ReverseProxyConfig, SelfAwareConfig, SslConfig,
    TURNRelay, UPnPDevice, UniversalDiscoveryConfig,
};
use super::cors::CorsConfig;
use super::gaming::GamingNetworkConfig;
use super::limits::ConnectionLimits;
use super::ports::PortRange;
use super::timeouts::NetworkTimeouts;

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

    // Phase 2B: Advanced features (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_config: Option<SelfAwareConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub universal_discovery: Option<UniversalDiscoveryConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse_proxy: Option<ReverseProxyConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl_config: Option<SslConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_relay: Option<TURNRelay>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub upnp_device: Option<UPnPDevice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_discovery: Option<DiscoveryNetworkTopology>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_metrics: Option<NetworkMeasurement>,
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
            production_bind_address: SafeEnv::get_or_default(
                "SONGBIRD_PRODUCTION_BIND_ADDRESS",
                "0.0.0.0",
            )
            .parse()
            .unwrap_or_else(|e| {
                warn!("Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using default 0.0.0.0: {}", e);
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            }),
            orchestrator_port: SafeEnv::get_port(
                "SONGBIRD_ORCHESTRATOR_PORT",
                SafeEnv::get_port("DEFAULT_HTTP_PORT", 8080),
            ),
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
    /// Returns an error if the protocol is not supported
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
            production_bind_address: "0.0.0.0"
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
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
            .expect("Serialized PeerType should deserialize successfully");
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
