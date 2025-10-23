//! Canonical Network Configuration - Unified Modern Implementation
//!
//! This module provides the single, canonical `NetworkConfig` definition that replaces
//! all fragmented and deprecated network configuration structs across the codebase.

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tracing::warn;
/// Canonical Network Configuration - Single Source of Truth
///
/// This struct unifies all network configuration patterns across Songbird,
/// eliminating fragmentation and providing a modern, comprehensive solution.
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
            origins: vec!["http://crate::constants::network::DEFAULT_HOST:3000".to_string()],
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
    pub async fn from_env() -> SongbirdResult<Self> {
        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0".to_string())
            .parse()
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Invalid bind address: {e}"),
                field: Some("bind_address".to_string()),
                suggestion: Some("Provide a valid IP address".to_string()),
            })?;

        let config = Self {
            bind_address,
            production_bind_address: std::env::var("SONGBIRD_PRODUCTION_BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string())
                .parse()
                .unwrap_or_else(|e| {
                    warn!("Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using default 0.0.0.0: {}", e);
                    std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
                }),
            orchestrator_port: std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
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
        };

        Ok(config)
    }

    /// Validate production readiness
    ///
    /// # Errors
    ///
    /// Returns an error if the bind address is set to localhost in production
    pub async fn validate_production_readiness(&self) -> SongbirdResult<()> {
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
    pub async fn local_bind_address(&self) -> SongbirdResult<SocketAddr> {
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
    pub async fn gaming_port(&self, protocol: &str) -> SongbirdResult<u16> {
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
                origins: vec!["http://crate::constants::network::DEFAULT_HOST:3000".to_string()],
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
        }
    }
}

// Type alias for backwards compatibility during migration
pub type NetworkConfig = CanonicalNetworkConfig;
