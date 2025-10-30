//! Network configuration types

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

/// Port range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    /// Check if a port is within this range
    #[must_use]
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Generate a random port within this range
    #[must_use]
    pub fn random_port(&self) -> u16 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.start..=self.end)
    }

    /// Get all ports in the range
    #[must_use]
    pub fn all_ports(&self) -> Vec<u16> {
        (self.start..=self.end).collect()
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

impl NetworkTimeouts {
    /// Create network timeouts from environment
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            connection: Duration::from_secs(
                env::var("CONNECTION_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            ),
            request: Duration::from_secs(
                env::var("REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
            health_check: Duration::from_secs(
                env::var("HEALTH_CHECK_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
            ),
            default: Duration::from_secs(
                env::var("DEFAULT_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
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

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
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
            max_retries: env::var("MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
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

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingNetworkConfig {
    /// StarCraft IPX port
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

