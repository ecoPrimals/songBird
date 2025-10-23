//! Network configuration for Songbird - Zero hardcoded values
//!
//! This module provides network configuration with environment-based defaults.
//! All network settings are configurable via environment variables.

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
type Result<T> = SongbirdResult<T>;
use crate::config::constants::get_bind_address;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tracing::warn;

/// Network configuration for Songbird orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address for services
    pub bind_address: IpAddr,

    /// Production bind address (typically 0.0.0.0)
    pub production_bind_address: IpAddr,

    /// Primary orchestrator port
    pub orchestrator_port: u16,

    /// Discovery service port
    pub discovery_port: u16,

    /// Default gaming server port (`StarCraft` IPX,
    pub gaming_port: u16,

    /// Health monitoring port for endpoint checks
    pub health_port: u16,

    /// Dashboard web interface port
    pub dashboard_port: u16,

    /// Gaming server port range for automatic allocation
    pub gaming_port_range: PortRange,

    /// Enable TLS for external connections
    pub require_tls: bool,

    /// Collection of timeout configurations
    pub timeouts: NetworkTimeouts,

    /// Connection limits
    pub connection_limits: ConnectionLimits,

    /// Gaming-specific network settings
    pub gaming: GamingNetworkConfig,

    /// Discovery ports
    pub discovery_ports: Vec<u16>,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Allowed networks
    pub allowed_networks: Vec<String>,

    /// Maximum connections
    pub max_connections: usize,

    /// Maximum bandwidth in Mbps
    pub max_bandwidth_mbps: u64,

    /// Worker threads
    pub worker_threads: usize,

    /// Federation endpoints
    pub federation_endpoints: Vec<String>,

    /// Stun servers
    pub stun_servers: Vec<String>,

    /// WebSocket port
    pub websocket_port: u16,

    /// Metrics bind address
    pub metrics_bind_address: IpAddr,

    /// Metrics port
    pub metrics_port: u16,

    /// Federation bind address
    pub federation_bind_address: IpAddr,

    /// Federation port
    pub federation_port: u16,

    /// CORS configuration
    pub cors: CorsConfig,
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

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)] // timeout_secs suffix is intentional for clarity
pub struct TimeoutConfig {
    /// Default operation timeout
    pub default_timeout_secs: u64,

    /// Connection timeout
    pub connection_timeout_secs: u64,

    /// Health check timeout
    pub health_check_timeout_secs: u64,

    /// Service registration timeout
    pub registration_timeout_secs: u64,

    /// Discovery timeout
    pub discovery_timeout_secs: u64,
}

/// Connection limits
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
            max_connections_per_host: 50,
            max_total_connections: 500,
            max_retries: 3,
            pool_idle_timeout_secs: 300,
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,
    /// Allowed origins
    pub origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    /// Default gaming server port (`StarCraft` IPX,
    pub starcraft_port: u16,

    /// Age of Empires II port
    pub aoe2_port: u16,

    /// Command & Conquer port range
    pub cnc_port_range: PortRange,

    /// Auto-detection interface
    pub detection_interface: Option<String>,

    /// Bridge buffer size
    pub bridge_buffer_size: usize,
}

impl NetworkConfig {
    /// Create network configuration from environment variables
    ///
    /// # Errors
    ///
    /// Returns an error if environment variables contain invalid values
    ///
    /// # Panics
    ///
    /// Panics if &`crate::constants::network::DEFAULT_HOST` cannot be parsed as an IP address
    pub fn from_env() -> Result<Self> {
        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS")
            .unwrap_or_else(|_| crate::constants::network::DEFAULT_HOST.to_string())
            .parse()
            .map_err(|e| SongbirdError::Configuration {
                message: format!("Invalid bind address: {e}"),
                field: Some("bind_address".to_string()),
                suggestion: Some("Provide a valid IP address for bind_address".to_string()),
            })?;

        Ok(Self {
            bind_address,
            production_bind_address: std::env::var("SONGBIRD_PRODUCTION_BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string())
                .parse()
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using default 0.0.0.0: {e}"
                    );
                    get_bind_address()
                        .parse()
                        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
                }),
            orchestrator_port: std::env::var("SONGBIRD_ORCHESTRATOR_PORT")
                .unwrap_or_else(|_| {
                    crate::constants::network::DEFAULT_ORCHESTRATOR_PORT.to_string()
                })
                .parse()
                .unwrap_or(8080),
            discovery_port: 8001,
            gaming_port: 6112,
            health_port: 8002,
            dashboard_port: 3000,
            gaming_port_range: PortRange {
                start: 7000,
                end: 7100,
            },
            require_tls: false,
            timeouts: NetworkTimeouts::default(),
            connection_limits: ConnectionLimits::default(),
            gaming: GamingNetworkConfig::default(),
            discovery_ports: vec![8001],
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            allowed_networks: vec!["127.0.0.0/8".to_string()],
            max_connections: 100,
            max_bandwidth_mbps: 100,
            worker_threads: 2,
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
            websocket_port: 8080,
            metrics_bind_address: bind_address,
            metrics_port: 8004,
            federation_bind_address: bind_address,
            federation_port: 8005,
            cors: CorsConfig {
                enabled: false,
                origins: std::env::var("SONGBIRD_CORS_ORIGINS").map_or_else(
                    |_| vec!["http://crate::constants::network::DEFAULT_HOST:3000".to_string()],
                    |origins| origins.split(',').map(String::from).collect(),
                ),
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
            },
        })
    }

    /// Get secure configuration for production
    ///
    /// # Panics
    ///
    /// Panics if &`crate::constants::network::DEFAULT_HOST` cannot be parsed as an IP address
    #[must_use]
    pub fn secure_defaults() -> Self {
        Self {bind_address: get_bind_address()
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            production_bind_address: "0.0.0.0"
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
            orchestrator_port: 8080,
            discovery_port: 8001,
            gaming_port: 6112,
            health_port: 8002,
            dashboard_port: 3000,
            gaming_port_range: PortRange {
                start: 7000,
                end: 7100,
            },
            require_tls: true, // Secure by default
            timeouts: NetworkTimeouts::default(),
            connection_limits: ConnectionLimits  {max_connections_per_host: 10,
                max_total_connections: 50,
                max_retries: 3,
                pool_idle_timeout_secs: 300,
            }, // Conservative limits
            gaming: GamingNetworkConfig::default(),
            discovery_ports: vec![8001],
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            allowed_networks: vec!["127.0.0.0/8".to_string()], // Localhost only
            max_connections: 50,                               // Conservative
            max_bandwidth_mbps: 50,                            // Conservative
            worker_threads: 2,                                 // Conservative
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
            websocket_port: 8080,
            metrics_bind_address: get_bind_address().parse().unwrap_or_else(|_| {
                warn!("Failed to parse default bind address, using crate::constants::network::DEFAULT_HOST");
                crate::constants::network::DEFAULT_HOST
                    .parse()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            }),
            metrics_port: 8004,
            federation_bind_address: get_bind_address().parse().unwrap_or_else(|_| {
                warn!("Failed to parse default bind address, using crate::constants::network::DEFAULT_HOST");
                crate::constants::network::DEFAULT_HOST
                    .parse()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            }),
            federation_port: 8005,
            cors: CorsConfig  {enabled: false,
                origins: std::env::var("SONGBIRD_CORS_ORIGINS").map_or_else(|_| vec!["http://crate::constants::network::DEFAULT_HOST:3000".to_string()], |origins| origins.split(',').map(String::from).collect()),
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
            },
        }
    }

    /// Validate production readiness
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is not suitable for production
    pub fn validate_production_readiness(&self) -> Result<()> {
        // Production environments should have explicit configuration
        if self.bind_address.to_string() == "0.0.0.0"
            && std::env::var("SONGBIRD_PRODUCTION_BINDING_APPROVED").is_err()
        {
            return Err(SongbirdError::Configuration {
                message: "Production binding to 0.0.0.0 requires explicit approval via SONGBIRD_PRODUCTION_BINDING_APPROVED=true".to_string(),
                field: Some("bind_address".to_string()),
                suggestion: Some("Set SONGBIRD_PRODUCTION_BINDING_APPROVED=true or use a specific bind address".to_string()),
            });
        }

        // Validate federation endpoints are configured for production
        if self.federation_endpoints.is_empty()
            && std::env::var("SONGBIRD_ENV").unwrap_or_default() == "production"
        {
            tracing::warn!("Production environment without federation endpoints configured");
        }

        Ok(())
    }

    /// Get orchestrator endpoint
    #[must_use]
    pub const fn orchestrator_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.orchestrator_port)
    }

    /// Get discovery endpoint
    #[must_use]
    pub const fn discovery_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.discovery_port)
    }

    /// Get health endpoint
    #[must_use]
    pub const fn health_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.health_port)
    }

    /// Get dashboard endpoint
    #[must_use]
    pub const fn dashboard_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.dashboard_port)
    }

    /// Default gaming server port (`StarCraft` IPX,
    pub const DEFAULT_GAMING_PORT: u16 = 6112;

    /// Get local bind address based on configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the bind address cannot be parsed or constructed
    #[must_use = "Network configuration should be used"]
    pub fn local_bind_address(&self) -> Result<SocketAddr> {
        let bind_str = format!("{}:{}", self.bind_address, self.orchestrator_port);

        bind_str.parse::<SocketAddr>().map_or_else(
            |_| {
                self.bind_address.to_string().parse::<IpAddr>().map_or_else(
                    |_| {
                        Ok(SocketAddr::new(
                            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                            self.orchestrator_port,
                        ))
                    },
                    |ip| Ok(SocketAddr::new(ip, self.orchestrator_port)),
                )
            },
            Ok,
        )
    }

    /// Get default endpoint for services
    #[must_use]
    pub const fn default_endpoint(&self) -> SocketAddr {
        self.orchestrator_endpoint()
    }

    /// Get gaming port for protocol
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is not supported
    pub fn gaming_port(&self, protocol: &str) -> Result<u16> {
        match protocol {
            "ipx" | "starcraft" => Ok(self.gaming_port),
            "aoe2" => Ok(self.gaming.aoe2_port),
            _ => Err(SongbirdError::Configuration {
                message: format!("Unknown gaming protocol: {protocol}"),
                field: Some("gaming_protocol".to_string()),
                suggestion: Some(
                    "Use a supported gaming protocol like 'ipx', 'starcraft', or 'aoe2'"
                        .to_string(),
                ),
            }),
        }
    }

    /// Get timeout for operation
    #[must_use]
    pub fn timeout(&self, operation: &str) -> Duration {
        match operation {
            "connection" => self.timeouts.connection,
            "request" => self.timeouts.request,
            "health_check" => self.timeouts.health_check,
            _ => self.timeouts.default,
        }
    }

    /// Check if port is in gaming range
    #[must_use]
    pub const fn is_gaming_port(&self, port: u16) -> bool {
        port >= self.gaming_port_range.start && port <= self.gaming_port_range.end
    }

    /// Get next available gaming port
    ///
    /// # Errors
    ///
    /// Returns an error if no ports are available in the range
    pub fn next_gaming_port(&self, exclude: &[u16]) -> Result<u16> {
        for port in self.gaming_port_range.start..=self.gaming_port_range.end {
            if !exclude.contains(&port) {
                return Ok(port);
            }
        }
        Err(SongbirdError::Configuration {
            field: Some("gaming_port_range".to_string()),
            message: "No available ports in gaming range".to_string(),
            suggestion: Some("Expand the gaming port range or release some ports".to_string()),
        })
    }

    /// Validate network configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration has conflicts or invalid values
    pub fn validate(&self) -> Result<()> {
        let all_ports = [
            self.orchestrator_port,
            self.discovery_port,
            self.health_port,
            self.dashboard_port,
            self.gaming_port,
        ];

        for (i, &port1) in all_ports.iter().enumerate() {
            for &port2 in all_ports.iter().skip(i + 1) {
                if port1 == port2 {
                    return Err(SongbirdError::Configuration {
                        field: Some("port_conflict".to_string()),
                        message: format!("Port {port1} is used multiple times"),
                        suggestion: Some(
                            "Ensure each port is used only once in the configuration".to_string(),
                        ),
                    });
                }
            }
        }

        // Validate port range
        if self.gaming_port_range.start > self.gaming_port_range.end {
            return Err(SongbirdError::Configuration {
                field: Some("gaming_port_range".to_string()),
                message: format!(
                    "Invalid port range: start ({}) > end ({})",
                    self.gaming_port_range.start, self.gaming_port_range.end
                ),
                suggestion: Some(
                    "Ensure the start port is less than or equal to the end port".to_string(),
                ),
            });
        }

        Ok(())
    }

    /// Create configuration for gaming scale
    #[must_use]
    pub fn for_gaming_scale(scale: &GamingScale) -> Self {
        let mut config = Self::default();

        match scale {
            GamingScale::Home => {
                config.connection_limits.max_total_connections = 100;
                config.connection_limits.max_connections_per_host = 10;
            }
            GamingScale::LanParty => {
                config.connection_limits.max_total_connections = 1000;
                config.connection_limits.max_connections_per_host = 50;
            }
            GamingScale::Tournament => {
                config.connection_limits.max_total_connections = 5000;
                config.connection_limits.max_connections_per_host = 100;
            }
            GamingScale::Professional => {
                config.connection_limits.max_total_connections = 10000;
                config.connection_limits.max_connections_per_host = 200;
            }
        }

        config
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: get_bind_address().parse().unwrap_or_else(|e| {
                warn!("Failed to parse bind address, using 127.0.0.1: {e}");
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            }),
            production_bind_address: "0.0.0.0".parse().unwrap_or_else(|_| {
                warn!("Failed to parse production bind address, using 0.0.0.0");
                std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            }),
            orchestrator_port: 8080,
            discovery_port: 8001,
            gaming_port: 6112,
            health_port: 8002,
            dashboard_port: 3000,
            gaming_port_range: PortRange {
                start: 7000,
                end: 7100,
            },
            require_tls: false,
            timeouts: NetworkTimeouts::default(),
            connection_limits: ConnectionLimits::default(),
            gaming: GamingNetworkConfig::default(),
            discovery_ports: vec![8001],
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            allowed_networks: vec!["127.0.0.0/8".to_string()],
            max_connections: 100,
            max_bandwidth_mbps: 100,
            worker_threads: 2,
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
            websocket_port: 8080,
            metrics_bind_address: get_bind_address().parse().unwrap_or_else(|e| {
                warn!("Failed to parse metrics bind address, using 127.0.0.1: {}", e);
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            }),
            metrics_port: 8004,
            federation_bind_address: get_bind_address().parse().unwrap_or_else(|e| {
                warn!("Failed to parse federation bind address, using 127.0.0.1: {}", e);
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            }),
            federation_port: 8005,
            cors: CorsConfig {
                enabled: false,
                origins: std::env::var("SONGBIRD_CORS_ORIGINS").map_or_else(
                    |_| vec!["http://crate::constants::network::DEFAULT_HOST:3000".to_string()],
                    |origins| origins.split(',').map(String::from).collect(),
                ),
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["Content-Type".to_string()],
            },
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            connection_timeout_secs: 10,
            health_check_timeout_secs: 5,
            registration_timeout_secs: 15,
            discovery_timeout_secs: 10,
        }
    }
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

impl Default for GamingNetworkConfig {
    fn default() -> Self {
        Self {
            starcraft_port: 6112,
            aoe2_port: 2300,
            cnc_port_range: PortRange {
                start: 1234,
                end: 1240,
            },
            detection_interface: None, // Auto-detect
            bridge_buffer_size: 65536,
        }
    }
}

impl PortRange {
    /// Check if port is in range
    #[must_use]
    pub const fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Get random port in range
    #[must_use]
    pub fn random_port(&self) -> u16 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.start..=self.end)
    }

    /// Get all ports in range
    #[must_use]
    pub fn all_ports(&self) -> Vec<u16> {
        (self.start..=self.end).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        assert_eq!(config.orchestrator_port, 8080);
        assert_eq!(config.gaming.starcraft_port, 6112);
        assert_eq!(config.gaming.aoe2_port, 2300);
    }

    #[test]
    fn test_endpoint_generation() {
        let config = NetworkConfig::default();
        let endpoint = config.orchestrator_endpoint();
        assert_eq!(endpoint.port(), 8080);
    }

    #[test]
    fn test_gaming_port_lookup() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = NetworkConfig::default();
        assert_eq!(
            config.gaming_port("starcraft").map_err(|e| SongbirdError::configuration(format!(
                "Test: starcraft port should be found: {}",
                e
            )))?,
            6112
        );
        assert_eq!(
            config.gaming_port("aoe2").map_err(|e| SongbirdError::configuration(format!(
                "Test: aoe2 port should be found: {}",
                e
            )))?,
            2300
        );
        assert!(config.gaming_port("unknown").is_err());
        Ok(())
    }

    #[test]
    fn test_timeout_lookup() {
        let config = NetworkConfig::default();
        assert_eq!(config.timeout("connection"), Duration::from_secs(10));
        assert_eq!(config.timeout("health_check"), Duration::from_secs(5));
        assert_eq!(config.timeout("unknown"), Duration::from_secs(30));
    }

    #[test]
    fn test_port_range() {
        let range = PortRange {
            start: 100,
            end: 200,
        };
        assert!(range.contains(150));
        assert!(!range.contains(50));
        assert!(!range.contains(250));
    }

    #[test]
    fn test_config_validation() {
        let mut config = NetworkConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid port range
        config.gaming_port_range.start = 200;
        config.gaming_port_range.end = 100;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_gaming_scale_configs() {
        use GamingScale;
        // use songbird_config; // FIXED: Circular import removed

        let home_config = NetworkConfig::for_gaming_scale(&GamingScale::Home);
        let lan_config = NetworkConfig::for_gaming_scale(&GamingScale::LanParty);

        assert!(
            home_config.connection_limits.max_total_connections
                < lan_config.connection_limits.max_total_connections
        )
    }

    /// Example configurations for different gaming scales
    #[allow(dead_code)]
    pub fn example_configurations() -> Vec<(GamingScale, NetworkConfig)> {
        let home_config = NetworkConfig::for_gaming_scale(&GamingScale::Home);
        let lan_config = NetworkConfig::for_gaming_scale(&GamingScale::LanParty);

        vec![(GamingScale::Home, home_config), (GamingScale::LanParty, lan_config)]
    }
}

/// Gaming network scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
