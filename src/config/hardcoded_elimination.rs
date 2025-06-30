//! Hardcoding Elimination Infrastructure
//!
//! Systematic replacement of hardcoded values with configurable alternatives.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

/// Central configuration for eliminating hardcoded values
#[derive(Debug, Clone, Default)]
pub struct HardcodingEliminationConfig {
    /// Network configuration patterns
    pub network: NetworkConfig,
    /// Service configuration patterns
    pub service: ServiceConfig,
    /// Security configuration patterns
    pub security: SecurityConfig,
    /// Timeout configuration patterns
    pub timeouts: TimeoutConfig,
    /// Performance configuration patterns
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub encryption_key_size: usize,
    pub session_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: IpAddr,
    pub production_bind_address: IpAddr,
    pub stun_servers: Vec<String>,
    pub port_ranges: HashMap<String, (u16, u16)>,
}

#[derive(Debug, Clone)] 
pub struct TimeoutConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub health_check_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub small_buffer_size: usize,
    pub large_buffer_size: usize,
    pub max_packet_size: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_key_size: 256,
            session_timeout: Duration::from_secs(3600),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            service_name: env_or_default("SONGBIRD_SERVICE_NAME", "songbird-orchestrator"),
            version: env_or_default("SONGBIRD_VERSION", "0.1.0"),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: env_or_default("SONGBIRD_BIND_ADDRESS", "127.0.0.1").parse().unwrap_or_else(|_| "127.0.0.1".parse().unwrap()),
            production_bind_address: env_or_default("SONGBIRD_PRODUCTION_BIND_ADDRESS", "0.0.0.0").parse().unwrap_or_else(|_| "0.0.0.0".parse().unwrap()),
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
            port_ranges: {
                let mut ranges = HashMap::new();
                ranges.insert("orchestrator".to_string(), (8080, 8090));
                ranges.insert("gaming".to_string(), (7000, 7100));
                ranges
            },
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            health_check_timeout: Duration::from_secs(5),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            small_buffer_size: 1024,
            large_buffer_size: 8192,
            max_packet_size: 65536,
        }
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Thread-safe global configuration using OnceLock (idiomatic Rust)
use std::sync::OnceLock;
static GLOBAL_CONFIG: OnceLock<HardcodingEliminationConfig> = OnceLock::new();

/// Get global configuration (thread-safe, idiomatic)
pub fn get_config() -> &'static HardcodingEliminationConfig {
    GLOBAL_CONFIG.get_or_init(HardcodingEliminationConfig::default)
}

/// Convenience functions for replacing hardcoded values
pub mod replace {
    use super::*;

    /// Replace hardcoded "127.0.0.1"
    pub fn bind_address() -> IpAddr {
        get_config().network.bind_address
    }

    /// Replace hardcoded Duration::from_secs(30)
    pub fn connection_timeout() -> Duration {
        get_config().timeouts.connection_timeout
    }

    /// Replace hardcoded 8192
    pub fn large_buffer_size() -> usize {
        get_config().performance.large_buffer_size
    }

    /// Replace hardcoded STUN servers
    pub fn stun_servers() -> Vec<String> {
        get_config().network.stun_servers.clone()
    }
}
