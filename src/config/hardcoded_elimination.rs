//! Hardcoding Elimination Infrastructure
//!
//! Systematic replacement of hardcoded values with configurable alternatives.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Central configuration for eliminating hardcoded values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardcodedEliminationConfig {
    pub network: NetworkDefaults,
    pub timeouts: TimeoutDefaults,
    pub security: SecurityDefaults,
    pub discovery: DiscoveryDefaults,
    pub gaming: GamingDefaults,
    pub federation: FederationDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDefaults {
    pub default_bind_address: String,
    pub default_port: u16,
    pub dashboard_port: u16,
    pub reverse_proxy_port: u16,
    pub alternative_ports: Vec<u16>,
    pub common_discovery_ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutDefaults {
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub discovery_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub stun_timeout_ms: u64,
    pub dht_timeout_ms: u64,
    pub deployment_timeout_ms: u64,
    pub health_check_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDefaults {
    pub session_timeout_hours: u64,
    pub key_rotation_interval_hours: u64,
    pub audit_log_retention_days: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDefaults {
    pub local_discovery_interval_seconds: u64,
    pub regional_discovery_interval_seconds: u64,
    pub global_discovery_interval_seconds: u64,
    pub topology_refresh_interval_seconds: u64,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingDefaults {
    pub max_latency_microseconds: u64,
    pub gaming_timeout_ms: u64,
    pub family_timeout_ms: u64,
    pub tunnel_expiry_hours: u64,
    pub auto_renewal_window_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDefaults {
    pub bootstrap_query_timeout_seconds: u64,
    pub peer_discovery_timeout_seconds: u64,
    pub route_cache_ttl_seconds: u64,
    pub health_monitoring_interval_seconds: u64,
    pub max_route_hops: u32,
    pub max_concurrent_connections: u32,
}

impl Default for NetworkDefaults {
    fn default() -> Self {
        Self {
            default_bind_address: std::env::var("SONGBIRD_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            default_port: std::env::var("SONGBIRD_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            dashboard_port: std::env::var("SONGBIRD_DASHBOARD_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            reverse_proxy_port: std::env::var("SONGBIRD_PROXY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            alternative_ports: vec![8080, 9090, 3000, 4000, 5000, 8000],
            common_discovery_ports: vec![8080, 9090, 3000, 4000, 5000],
        }
    }
}

impl Default for TimeoutDefaults {
    fn default() -> Self {
        Self {
            connection_timeout_ms: std::env::var("SONGBIRD_CONNECTION_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            request_timeout_ms: std::env::var("SONGBIRD_REQUEST_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            discovery_timeout_ms: std::env::var("SONGBIRD_DISCOVERY_TIMEOUT_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            bootstrap_timeout_ms: std::env::var("SONGBIRD_BOOTSTRAP_TIMEOUT_MS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            stun_timeout_ms: std::env::var("SONGBIRD_STUN_TIMEOUT_MS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .unwrap_or(2000),
            dht_timeout_ms: std::env::var("SONGBIRD_DHT_TIMEOUT_MS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .unwrap_or(2000),
            deployment_timeout_ms: std::env::var("SONGBIRD_DEPLOYMENT_TIMEOUT_MS")
                .unwrap_or_else(|_| "300000".to_string())
                .parse()
                .unwrap_or(300000), // 5 minutes
            health_check_timeout_ms: std::env::var("SONGBIRD_HEALTH_CHECK_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
        }
    }
}

impl Default for SecurityDefaults {
    fn default() -> Self {
        Self {
            session_timeout_hours: std::env::var("SONGBIRD_SESSION_TIMEOUT_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            key_rotation_interval_hours: std::env::var("SONGBIRD_KEY_ROTATION_HOURS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            audit_log_retention_days: std::env::var("SONGBIRD_AUDIT_RETENTION_DAYS")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .unwrap_or(90),
            max_failed_attempts: std::env::var("SONGBIRD_MAX_FAILED_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            lockout_duration_minutes: std::env::var("SONGBIRD_LOCKOUT_DURATION_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
        }
    }
}

impl Default for DiscoveryDefaults {
    fn default() -> Self {
        Self {
            local_discovery_interval_seconds: std::env::var(
                "SONGBIRD_LOCAL_DISCOVERY_INTERVAL_SECONDS",
            )
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
            regional_discovery_interval_seconds: std::env::var(
                "SONGBIRD_REGIONAL_DISCOVERY_INTERVAL_SECONDS",
            )
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60),
            global_discovery_interval_seconds: std::env::var(
                "SONGBIRD_GLOBAL_DISCOVERY_INTERVAL_SECONDS",
            )
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300),
            topology_refresh_interval_seconds: std::env::var(
                "SONGBIRD_TOPOLOGY_REFRESH_INTERVAL_SECONDS",
            )
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30),
            cache_ttl_seconds: std::env::var("SONGBIRD_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
        }
    }
}

impl Default for GamingDefaults {
    fn default() -> Self {
        Self {
            max_latency_microseconds: std::env::var("SONGBIRD_MAX_LATENCY_MICROSECONDS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            gaming_timeout_ms: std::env::var("SONGBIRD_GAMING_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            family_timeout_ms: std::env::var("SONGBIRD_FAMILY_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            tunnel_expiry_hours: std::env::var("SONGBIRD_TUNNEL_EXPIRY_HOURS")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .unwrap_or(8),
            auto_renewal_window_minutes: std::env::var("SONGBIRD_AUTO_RENEWAL_WINDOW_MINUTES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}

impl Default for FederationDefaults {
    fn default() -> Self {
        Self {
            bootstrap_query_timeout_seconds: std::env::var(
                "SONGBIRD_BOOTSTRAP_QUERY_TIMEOUT_SECONDS",
            )
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10),
            peer_discovery_timeout_seconds: std::env::var(
                "SONGBIRD_PEER_DISCOVERY_TIMEOUT_SECONDS",
            )
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30),
            route_cache_ttl_seconds: std::env::var("SONGBIRD_ROUTE_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            health_monitoring_interval_seconds: std::env::var(
                "SONGBIRD_HEALTH_MONITORING_INTERVAL_SECONDS",
            )
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60),
            max_route_hops: std::env::var("SONGBIRD_MAX_ROUTE_HOPS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            max_concurrent_connections: std::env::var("SONGBIRD_MAX_CONCURRENT_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
        }
    }
}

impl HardcodedEliminationConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connection_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.connection_timeout_ms)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.request_timeout_ms)
    }

    pub fn discovery_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.discovery_timeout_ms)
    }

    pub fn bootstrap_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.bootstrap_timeout_ms)
    }

    pub fn stun_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.stun_timeout_ms)
    }

    pub fn dht_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.dht_timeout_ms)
    }

    pub fn deployment_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.deployment_timeout_ms)
    }

    pub fn health_check_timeout(&self) -> Duration {
        Duration::from_millis(self.timeouts.health_check_timeout_ms)
    }

    pub fn local_discovery_interval(&self) -> Duration {
        Duration::from_secs(self.discovery.local_discovery_interval_seconds)
    }

    pub fn regional_discovery_interval(&self) -> Duration {
        Duration::from_secs(self.discovery.regional_discovery_interval_seconds)
    }

    pub fn global_discovery_interval(&self) -> Duration {
        Duration::from_secs(self.discovery.global_discovery_interval_seconds)
    }

    pub fn topology_refresh_interval(&self) -> Duration {
        Duration::from_secs(self.discovery.topology_refresh_interval_seconds)
    }

    pub fn gaming_timeout(&self) -> Duration {
        Duration::from_millis(self.gaming.gaming_timeout_ms)
    }

    pub fn family_timeout(&self) -> Duration {
        Duration::from_millis(self.gaming.family_timeout_ms)
    }

    pub fn session_timeout(&self) -> Duration {
        Duration::from_secs(self.security.session_timeout_hours * 3600)
    }

    pub fn key_rotation_interval(&self) -> Duration {
        Duration::from_secs(self.security.key_rotation_interval_hours * 3600)
    }

    pub fn audit_log_retention(&self) -> Duration {
        Duration::from_secs(self.security.audit_log_retention_days * 24 * 3600)
    }

    pub fn bootstrap_query_timeout(&self) -> Duration {
        Duration::from_secs(self.federation.bootstrap_query_timeout_seconds)
    }

    pub fn peer_discovery_timeout(&self) -> Duration {
        Duration::from_secs(self.federation.peer_discovery_timeout_seconds)
    }

    pub fn route_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.federation.route_cache_ttl_seconds)
    }

    pub fn health_monitoring_interval(&self) -> Duration {
        Duration::from_secs(self.federation.health_monitoring_interval_seconds)
    }
}
