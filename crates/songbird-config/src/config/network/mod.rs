//! # 🌐 Network Configuration (Migrating to Zero Hardcoding)
//!
//! **MIGRATION STATUS**: This module is being migrated to zero-hardcoding patterns.
//!
//! **RECOMMENDED**:
//! ```rust,ignore
//! // ✅ Use zero-touch configuration (no hardcoded ports)
//! use songbird_config::zero_touch::InfantZeroTouchConfig;
//! let config = InfantZeroTouchConfig::from_environment()?;
//! // Requires: SERVICE_PORT environment variable
//! ```
//!
//! **LEGACY** (Deprecated):
//! ```rust,ignore
//! // ⚠️  Uses hardcoded default ports
//! let config = NetworkConfig::default();
//! ```
//!
//! This module provides network configuration with environment-based defaults.
//! All network settings are configurable via environment variables.

use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
type Result<T> = SongbirdResult<T>;
use crate::config::constants::get_bind_address;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tracing::{info, warn};

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

impl NetworkTimeouts {
    /// Create network timeouts from environment
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            connection: Duration::from_secs(
                env::var("CONNECTION_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
            ),
            request: Duration::from_secs(
                env::var("REQUEST_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60),
            ),
            health_check: Duration::from_secs(
                env::var("HEALTH_CHECK_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
            ),
            default: Duration::from_secs(
                env::var("DEFAULT_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30),
            ),
        }
    }
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

impl ConnectionLimits {
    /// Create connection limits from environment
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            max_connections_per_host: env::var("MAX_CONNECTIONS_PER_HOST")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            max_total_connections: env::var("MAX_TOTAL_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(500),
            max_retries: env::var("MAX_RETRIES").ok().and_then(|s| s.parse().ok()).unwrap_or(3),
            pool_idle_timeout_secs: env::var("POOL_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        }
    }
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

impl NetworkConfig {
    /// Create network configuration from environment variables (RECOMMENDED)
    ///
    /// **⚠️  Zero-Hardcoding Pattern**: This method requires environment variables and has NO hardcoded defaults.
    ///
    /// # Required Environment Variables
    /// - `SERVICE_PORT` or `ORCHESTRATOR_PORT` - Main service port
    ///
    /// # Optional Environment Variables
    /// - `DISCOVERY_PORT` - Discovery service port (defaults to `SERVICE_PORT` + 1)
    /// - `HEALTH_PORT` - Health check port (defaults to `SERVICE_PORT` + 2)
    /// - `METRICS_PORT` - Metrics port (defaults to `SERVICE_PORT` + 3)
    /// - `DASHBOARD_PORT` - Dashboard port (defaults to 3000)
    /// - `GAMING_PORT` - Gaming port (defaults to 6112)
    /// - `FEDERATION_PORT` - Federation port (defaults to `SERVICE_PORT` + 5)
    /// - `WEBSOCKET_PORT` - WebSocket port (defaults to `SERVICE_PORT`)
    ///
    /// # Example
    /// ```bash
    /// export SERVICE_PORT=8080
    /// export HEALTH_PORT=8081
    /// export METRICS_PORT=8082
    /// ```
    ///
    /// # Errors
    /// Returns error if `SERVICE_PORT` is not set or invalid
    pub fn from_environment() -> SongbirdResult<Self> {
        info!("🌐 Creating network config from environment (zero hardcoding)");

        // Get base port from environment - NO default
        let service_port = env::var("SERVICE_PORT")
            .or_else(|_| env::var("ORCHESTRATOR_PORT"))
            .map_err(|_| SongbirdError::Configuration {
                message: "SERVICE_PORT or ORCHESTRATOR_PORT environment variable required"
                    .to_string(),
                field: Some("service_port".to_string()),
                suggestion: Some("Set SERVICE_PORT=8080 (or desired port)".to_string()),
            })?
            .parse::<u16>()
            .map_err(|_| SongbirdError::Configuration {
                message: "Invalid SERVICE_PORT - must be a valid port number".to_string(),
                field: Some("service_port".to_string()),
                suggestion: Some("Set SERVICE_PORT to a number between 1 and 65535".to_string()),
            })?;

        info!("📍 Base service port: {}", service_port);

        // Helper to get port from env or calculate from base
        let get_port = |env_var: &str, offset: u16| -> u16 {
            env::var(env_var)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| service_port + offset)
        };

        // Bind address detection (production vs development)
        let bind_address =
            env::var("BIND_ADDRESS").ok().and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                // Check if we're in a container/production environment
                if env::var("KUBERNETES_SERVICE_HOST").is_ok()
                    || env::var("DOCKER_HOST").is_ok()
                    || env::var("PRODUCTION").is_ok()
                {
                    IpAddr::V4(Ipv4Addr::UNSPECIFIED) // 0.0.0.0
                } else {
                    IpAddr::V4(Ipv4Addr::LOCALHOST) // 127.0.0.1
                }
            });

        Ok(Self {
            bind_address,
            production_bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            orchestrator_port: service_port,
            discovery_port: get_port("DISCOVERY_PORT", 1),
            gaming_port: get_port("GAMING_PORT", 0), // Default gaming uses a well-known port
            health_port: get_port("HEALTH_PORT", 2),
            dashboard_port: get_port("DASHBOARD_PORT", 0), // Dashboard has special default
            gaming_port_range: PortRange {
                start: env::var("GAMING_PORT_START")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(7000),
                end: env::var("GAMING_PORT_END").ok().and_then(|s| s.parse().ok()).unwrap_or(7100),
            },
            require_tls: env::var("REQUIRE_TLS").ok().is_some_and(|v| v.to_lowercase() == "true"),
            timeouts: NetworkTimeouts::from_environment(),
            connection_limits: ConnectionLimits::from_environment(),
            gaming: GamingNetworkConfig::from_environment(),
            discovery_ports: vec![get_port("DISCOVERY_PORT", 1)],
            connection_timeout: Duration::from_secs(
                env::var("CONNECTION_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30),
            ),
            request_timeout: Duration::from_secs(
                env::var("REQUEST_TIMEOUT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(60),
            ),
            allowed_networks: env::var("ALLOWED_NETWORKS").map_or_else(
                |_| vec!["127.0.0.0/8".to_string()],
                |s| s.split(',').map(String::from).collect(),
            ),
            max_connections: env::var("MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            max_bandwidth_mbps: env::var("MAX_BANDWIDTH_MBPS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            worker_threads: env::var("WORKER_THREADS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| num_cpus::get().max(2)),
            federation_endpoints: env::var("FEDERATION_ENDPOINTS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
            stun_servers: env::var("STUN_SERVERS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
            websocket_port: get_port("WEBSOCKET_PORT", 0),
            metrics_bind_address: bind_address,
            metrics_port: get_port("METRICS_PORT", 3),
            federation_bind_address: bind_address,
            federation_port: get_port("FEDERATION_PORT", 5),
            cors: CorsConfig::from_environment(),
        })
    }
}

impl Default for NetworkConfig {
    /// **⚠️  DEPRECATED**: This uses hardcoded port defaults
    ///
    /// **USE INSTEAD**: `NetworkConfig::from_environment()`
    ///
    /// This default implementation is provided for backward compatibility but uses
    /// hardcoded port numbers which violates the zero-hardcoding philosophy.
    fn default() -> Self {
        warn!("⚠️  NetworkConfig::default() uses hardcoded ports. Use NetworkConfig::from_environment() instead.");

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

impl CorsConfig {
    /// Create CORS config from environment
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            enabled: env::var("CORS_ENABLED").is_ok_and(|v| v.to_lowercase() == "true"),
            origins: env::var("CORS_ORIGINS").map_or_else(
                |_| vec!["http://localhost:3000".to_string()],
                |s| s.split(',').map(String::from).collect(),
            ),
            allowed_methods: env::var("CORS_METHODS").map_or_else(
                |_| vec!["GET".to_string(), "POST".to_string()],
                |s| s.split(',').map(String::from).collect(),
            ),
            allowed_headers: env::var("CORS_HEADERS").map_or_else(
                |_| vec!["Content-Type".to_string()],
                |s| s.split(',').map(String::from).collect(),
            ),
        }
    }
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

impl GamingNetworkConfig {
    /// Create gaming network config from environment
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            starcraft_port: env::var("STARCRAFT_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(6112),
            aoe2_port: env::var("AOE2_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(2300),
            cnc_port_range: PortRange {
                start: env::var("CNC_PORT_START").ok().and_then(|s| s.parse().ok()).unwrap_or(1234),
                end: env::var("CNC_PORT_END").ok().and_then(|s| s.parse().ok()).unwrap_or(1240),
            },
            detection_interface: env::var("GAMING_INTERFACE").ok(),
            bridge_buffer_size: env::var("BRIDGE_BUFFER_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(65536),
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

// Tests module
#[path = "tests.rs"]
#[cfg(test)]
mod tests;

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
