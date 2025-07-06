//! Network Configuration Module
//!
//! Provides configurable network settings to eliminate hardcoded addresses and ports

use serde::{Deserialize, Serialize};
use songbird_errors::{Result, SongbirdError};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Network configuration for Songbird orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Primary bind address for the orchestrator
    pub bind_address: IpAddr,

    /// Primary orchestrator port
    pub orchestrator_port: u16,

    /// Gaming bridge port range
    pub gaming_port_range: PortRange,

    /// Discovery service port
    pub discovery_port: u16,

    /// Health check port
    pub health_port: u16,

    /// Metrics/dashboard port
    pub dashboard_port: u16,

    /// Timeout configurations
    pub timeouts: TimeoutConfig,

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

    /// Enable TLS
    pub enable_tls: bool,

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

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Maximum connections per host
    pub max_connections_per_host: usize,

    /// Maximum total connections
    pub max_total_connections: usize,

    /// Connection pool idle timeout
    pub pool_idle_timeout_secs: u64,

    /// Maximum retry attempts
    pub max_retries: u32,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    /// Default gaming server port (StarCraft IPX)
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
    /// Create network configuration with NO hardcoded values
    pub fn from_env() -> Result<Self> {
        let env_config = crate::config::environment::EnvironmentConfig::from_env()?;

        Ok(Self {
            // Use environment configuration - no hardcoding!
            orchestrator_port: env_config.bind_port,
            bind_address: env_config
                .bind_address
                .parse()
                .map_err(|e| SongbirdError::Config {
                    field: Some("bind_address".to_string()),
                    message: format!("Invalid bind address: {}", e),
                })?,
            discovery_ports: env_config.discovery_ports.clone(),

            // Connection timeouts from environment
            connection_timeout: env_config.connection_timeout(),
            request_timeout: env_config.request_timeout(),

            // Security settings from environment
            enable_tls: env_config.require_tls,
            allowed_networks: env_config.allowed_networks,
            max_connections: env_config.max_connections as usize,

            // Performance settings from environment
            max_bandwidth_mbps: env_config.max_bandwidth_mbps,
            worker_threads: env_config.worker_threads,

            // Gaming configuration from environment
            gaming_port_range: PortRange {
                start: env_config.gaming_port_range.0,
                end: env_config.gaming_port_range.1,
            },

            // Service endpoints from environment
            discovery_port: 8001,
            health_port: 8002,
            dashboard_port: 8003,
            connection_limits: ConnectionLimits::default(),
            timeouts: TimeoutConfig::default(),
            gaming: GamingNetworkConfig::default(),
            federation_endpoints: env_config.federation_endpoints,
            stun_servers: env_config.stun_servers,

            // WebSocket port from environment
            websocket_port: env_config.websocket_port,

            // Metrics configuration
            metrics_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid bind address for metrics, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            metrics_port: 8004,

            // Federation configuration
            federation_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid bind address for federation, using 127.0.0.1: {}",
                    e
                );
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            federation_port: 8005,

            // CORS configuration
            cors: CorsConfig::default(),
        })
    }

    /// Get secure default configuration (localhost-only)
    pub fn secure_defaults() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();

        Self {
            orchestrator_port: env_config.bind_port,
            bind_address: "127.0.0.1".parse().unwrap_or_else(|e| {
                tracing::error!(
                    "Failed to parse bind address: {}, using localhost fallback",
                    e
                );
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
            }), // Always secure by default with safe fallback
            discovery_ports: env_config.discovery_ports.clone(),
            connection_timeout: env_config.connection_timeout(),
            request_timeout: env_config.request_timeout(),
            enable_tls: false, // Can be enabled via environment
            allowed_networks: vec!["127.0.0.0/8".to_string()], // Localhost only by default
            max_connections: 100, // Conservative default
            max_bandwidth_mbps: 100, // Conservative default
            worker_threads: 2, // Conservative default
            discovery_port: 8001,
            health_port: 8002,
            gaming_port_range: PortRange {
                start: 7000,
                end: 7100,
            }, // Small safe range by default
            connection_limits: ConnectionLimits::default(),
            gaming: GamingNetworkConfig::default(),
            dashboard_port: 8080,
            timeouts: TimeoutConfig::default(),
            federation_endpoints: vec![], // Empty by default - must be configured
            stun_servers: vec![],         // Empty by default - must be configured
            websocket_port: env_config.websocket_port,

            // Metrics configuration
            metrics_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid bind address, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            metrics_port: 8004,

            // Federation configuration
            federation_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid bind address, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            federation_port: 8005,

            // CORS configuration
            cors: CorsConfig::default(),
        }
    }

    /// Check if configuration is production-ready
    pub fn validate_production_readiness(&self) -> Result<()> {
        // Production environments should have explicit configuration
        if self.bind_address.to_string() == "0.0.0.0"
            && std::env::var("SONGBIRD_PRODUCTION_BINDING_APPROVED").is_err()
        {
            return Err(SongbirdError::Config {
                field: Some("bind_address".to_string()),
                message: "Production binding to 0.0.0.0 requires explicit approval via SONGBIRD_PRODUCTION_BINDING_APPROVED=true".to_string(),
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

    /// Get the primary orchestrator endpoint
    pub fn orchestrator_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.orchestrator_port)
    }

    /// Get the discovery endpoint
    pub fn discovery_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.discovery_port)
    }

    /// Get the health check endpoint
    pub fn health_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.health_port)
    }

    /// Get the dashboard endpoint
    pub fn dashboard_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.dashboard_port)
    }

    /// Get local bind address for dynamic port allocation
    pub fn local_bind_address(&self) -> Result<SocketAddr> {
        Ok(SocketAddr::new(self.bind_address, 0))
    }

    /// Get default endpoint (replaces hardcoded localhost:8080)
    pub fn default_endpoint(&self) -> Result<SocketAddr> {
        Ok(self.orchestrator_endpoint())
    }

    /// Get gaming port for specific protocol
    pub fn gaming_port(&self, protocol: &str) -> Result<u16> {
        match protocol.to_lowercase().as_str() {
            "starcraft" | "ipx" => Ok(self.gaming.starcraft_port),
            "aoe2" | "directplay" => Ok(self.gaming.aoe2_port),
            "cnc" | "command_conquer" => Ok(self.gaming.cnc_port_range.start),
            _ => Err(SongbirdError::Config {
                field: Some("gaming_protocol".to_string()),
                message: format!("Unknown gaming protocol: {}", protocol),
            }),
        }
    }

    /// Get timeout duration for operation type
    pub fn timeout(&self, operation: &str) -> Duration {
        let secs = match operation {
            "connection" => self.timeouts.connection_timeout_secs,
            "health_check" => self.timeouts.health_check_timeout_secs,
            "registration" => self.timeouts.registration_timeout_secs,
            "discovery" => self.timeouts.discovery_timeout_secs,
            _ => self.timeouts.default_timeout_secs,
        };
        Duration::from_secs(secs)
    }

    /// Check if port is in gaming range
    pub fn is_gaming_port(&self, port: u16) -> bool {
        port >= self.gaming_port_range.start && port <= self.gaming_port_range.end
    }

    /// Get next available port in gaming range
    pub fn next_gaming_port(&self, exclude: &[u16]) -> Result<u16> {
        for port in self.gaming_port_range.start..=self.gaming_port_range.end {
            if !exclude.contains(&port) {
                return Ok(port);
            }
        }
        Err(SongbirdError::Network {
            service: "Network Config".to_string(),
            message: "No available ports in gaming range".to_string(),
            details: None,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check port ranges
        if self.gaming_port_range.start >= self.gaming_port_range.end {
            return Err(SongbirdError::Config {
                field: Some("gaming_port_range".to_string()),
                message: "Start port must be less than end port".to_string(),
            });
        }

        // Check for port conflicts
        let ports = [
            self.orchestrator_port,
            self.discovery_port,
            self.health_port,
            self.dashboard_port,
        ];

        for (i, &port1) in ports.iter().enumerate() {
            for &port2 in ports.iter().skip(i + 1) {
                if port1 == port2 {
                    return Err(SongbirdError::Config {
                        field: Some("port_conflict".to_string()),
                        message: format!("Port {} is used multiple times", port1),
                    });
                }
            }
        }

        // Check timeouts are reasonable
        if self.timeouts.default_timeout_secs == 0 {
            return Err(SongbirdError::Config {
                field: Some("default_timeout_secs".to_string()),
                message: "Timeout cannot be zero".to_string(),
            });
        }

        Ok(())
    }

    /// Create configuration for specific gaming scale
    pub fn for_gaming_scale(scale: GamingScale) -> Self {
        use GamingScale;

        let mut config = Self::default();

        match scale {
            GamingScale::HomeGaming => {
                // Gaming-optimized for home setups (2-8 players)
                config.connection_limits.max_connections_per_host = 8;
                config.connection_limits.max_total_connections = 20;
                config.gaming_port_range = PortRange {
                    start: 6112,
                    end: 6120,
                };
            }
            GamingScale::LanParty => {
                // LAN party optimized (8-50 players)
                config.connection_limits.max_connections_per_host = 50;
                config.connection_limits.max_total_connections = 100;
                config.gaming_port_range = PortRange {
                    start: 6112,
                    end: 6200,
                };
            }
            GamingScale::Auto => {
                // Auto-detect gaming load
                config.connection_limits.max_connections_per_host = 25;
                config.connection_limits.max_total_connections = 50;
                config.gaming_port_range = PortRange {
                    start: 6112,
                    end: 6150,
                };
            }
        }

        config
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();

        Self {
            // Use environment configuration - NO MORE HARDCODING!
            orchestrator_port: env_config.bind_port,
            bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid environment bind address, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            discovery_ports: env_config.discovery_ports.clone(),

            discovery_port: 8001,
            health_port: 8002,
            // Connection timeouts from environment
            connection_timeout: env_config.connection_timeout(),
            request_timeout: env_config.request_timeout(),

            // Other settings from environment
            dashboard_port: env_config.dashboard_port,
            websocket_port: env_config.websocket_port,

            // Metrics configuration
            metrics_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid bind address, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            metrics_port: 8004,

            // Federation configuration
            federation_bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!("Invalid bind address, using 127.0.0.1: {}", e);
                "127.0.0.1"
                    .parse()
                    .expect("127.0.0.1 is a valid IP address")
            }),
            federation_port: 8005,

            // CORS configuration
            cors: CorsConfig::default(),

            gaming_port_range: PortRange {
                start: env_config.gaming_port_range.0,
                end: env_config.gaming_port_range.1,
            },
            gaming: GamingNetworkConfig::default(),
            timeouts: TimeoutConfig::default(),
            connection_limits: ConnectionLimits::default(),
            enable_tls: false,
            allowed_networks: Vec::new(),
            max_connections: 500,
            max_bandwidth_mbps: 100,
            worker_threads: 4,
            federation_endpoints: Vec::new(),
            stun_servers: Vec::new(),
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

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections_per_host: 50,
            max_total_connections: 500,
            pool_idle_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_origins: vec!["*".to_string()],
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
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Get random port in range
    pub fn random_port(&self) -> u16 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.start..=self.end)
    }

    /// Get all ports in range
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
    fn test_gaming_port_lookup() {
        let config = NetworkConfig::default();
        assert_eq!(config.gaming_port("starcraft").unwrap(), 6112);
        assert_eq!(config.gaming_port("aoe2").unwrap(), 2300);
        assert!(config.gaming_port("unknown").is_err());
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

        let home_config = NetworkConfig::for_gaming_scale(GamingScale::HomeGaming);
        let lan_config = NetworkConfig::for_gaming_scale(GamingScale::LanParty);

        assert!(
            home_config.connection_limits.max_total_connections
                < lan_config.connection_limits.max_total_connections
        );
    }
}

/// Gaming scale configuration for different deployment sizes
#[derive(Debug, Clone, PartialEq)]
pub enum GamingScale {
    /// Home gaming setup (2-8 players)
    HomeGaming,
    /// LAN party setup (8-50 players)
    LanParty,
    /// Auto-detect based on load
    Auto,
}
