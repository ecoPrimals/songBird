/// Production LAN Gaming Configuration
///
/// This module contains all configuration structures for the production LAN gaming system.
/// Each configuration section is focused and well-documented.
use serde::{Deserialize, Serialize};

/// Production configuration for LAN gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionLanConfig {
    /// Discovery configuration
    pub discovery: DiscoveryConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Self-healing settings
    pub healing: HealingConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Ports to use for discovery (will try multiple)
    pub discovery_ports: Vec<u16>,
    /// Discovery broadcast interval
    pub broadcast_interval_ms: u64,
    /// Discovery timeout
    pub discovery_timeout_ms: u64,
    /// Maximum sessions to track
    pub max_sessions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable session encryption
    pub enable_encryption: bool,
    /// Maximum players per session
    pub max_players_per_session: u8,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Rate limiting for discovery
    pub max_discovery_requests_per_minute: u32,
    /// Allowed network interfaces (empty = all)
    pub allowed_interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Port range for game traffic
    pub game_port_range: (u16, u16),
    /// Buffer sizes for packet processing
    pub packet_buffer_size: usize,
    /// Maximum packet size
    pub max_packet_size: usize,
    /// Network interface preference order
    pub interface_preference: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingConfig {
    /// Enable automatic recovery
    pub enable_auto_recovery: bool,
    /// Health check interval
    pub health_check_interval_ms: u64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry backoff multiplier
    pub retry_backoff_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable traffic monitoring
    pub enable_traffic_monitoring: bool,
    /// Metrics collection interval
    pub metrics_interval_ms: u64,
    /// Log level for gaming operations
    pub log_level: String,
}

impl Default for ProductionLanConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();

        Self {
            discovery: DiscoveryConfig {
                discovery_ports: env_config.discovery_ports,
                broadcast_interval_ms: env_config.health_check_interval_secs * 1000 / 10,
                discovery_timeout_ms: env_config.discovery_timeout_secs * 1000,
                max_sessions: std::env::var("SONGBIRD_MAX_SESSIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100),
            },
            security: SecurityConfig {
                enable_encryption: env_config.enable_encryption,
                max_players_per_session: std::env::var("SONGBIRD_MAX_PLAYERS_PER_SESSION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(16),
                session_timeout_seconds: env_config.session_timeout_secs,
                max_discovery_requests_per_minute: std::env::var("SONGBIRD_MAX_DISCOVERY_REQUESTS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
                allowed_interfaces: vec![],
            },
            network: NetworkConfig {
                game_port_range: env_config.gaming_port_range,
                packet_buffer_size: std::env::var("SONGBIRD_PACKET_BUFFER_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(65536),
                max_packet_size: std::env::var("SONGBIRD_MAX_PACKET_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1500),
                interface_preference: std::env::var("SONGBIRD_INTERFACE_PREFERENCE")
                    .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_else(|_| vec!["eth0".to_string(), "wlan0".to_string()]),
            },
            healing: HealingConfig {
                enable_auto_recovery: std::env::var("SONGBIRD_ENABLE_AUTO_RECOVERY")
                    .map(|v| v.parse().unwrap_or(true))
                    .unwrap_or(true),
                health_check_interval_ms: env_config.health_check_interval_secs * 1000,
                max_retry_attempts: std::env::var("SONGBIRD_MAX_RETRY_ATTEMPTS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                retry_backoff_multiplier: std::env::var("SONGBIRD_RETRY_BACKOFF_MULTIPLIER")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(2.0),
            },
            monitoring: MonitoringConfig {
                enable_performance_monitoring: std::env::var(
                    "SONGBIRD_ENABLE_PERFORMANCE_MONITORING",
                )
                .map(|v| v.parse().unwrap_or(true))
                .unwrap_or(true),
                enable_traffic_monitoring: std::env::var("SONGBIRD_ENABLE_TRAFFIC_MONITORING")
                    .map(|v| v.parse().unwrap_or(true))
                    .unwrap_or(true),
                metrics_interval_ms: env_config.metrics_interval_secs * 1000,
                log_level: env_config.log_level,
            },
        }
    }
}
