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
    /// Universal Primal configuration patterns
    pub primals: PrimalConfig,
    /// Federation configuration patterns
    pub federation: FederationConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub version: String,
    pub base_url: String,
    pub health_endpoint: String,
    pub metrics_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub encryption_key_size: usize,
    pub session_timeout: Duration,
    pub beardog_endpoint: String,
    pub oauth_redirect_uri: String,
    pub tls_cert_path: String,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: IpAddr,
    pub production_bind_address: IpAddr,
    pub stun_servers: Vec<String>,
    pub port_ranges: HashMap<String, (u16, u16)>,
    pub orchestrator_endpoint: String,
    pub gaming_endpoint: String,
    pub federation_endpoint: String,
    pub dashboard_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub health_check_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub scaling_check_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub small_buffer_size: usize,
    pub large_buffer_size: usize,
    pub max_packet_size: usize,
    pub connection_pool_size: usize,
    pub cache_ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct PrimalConfig {
    pub beardog_endpoint: String,
    pub nestgate_endpoint: String,
    pub toadstool_endpoint: String,
    pub squirrel_endpoint: String,
    pub discovery_endpoints: Vec<String>,
    pub base_port: u16,
    pub port_range: (u16, u16),
}

#[derive(Debug, Clone)]
pub struct FederationConfig {
    pub cluster_endpoints: Vec<String>,
    pub heartbeat_endpoint: String,
    pub broadcast_ports: Vec<u16>,
    pub discovery_ports: Vec<u16>,
    pub default_cluster_id: String,
    pub auto_discovery_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_key_size: 256,
            session_timeout: Duration::from_secs(3600),
            beardog_endpoint: env_or_default("SONGBIRD_BEARDOG_ENDPOINT", "https://localhost:8443"),
            oauth_redirect_uri: env_or_default("SONGBIRD_OAUTH_REDIRECT", "http://localhost:8080/auth/callback"),
            tls_cert_path: env_or_default("SONGBIRD_TLS_CERT", "/etc/ssl/certs/songbird.crt"),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        let base_url = env_or_default("SONGBIRD_BASE_URL", "http://localhost:8080");
        Self {
            service_name: env_or_default("SONGBIRD_SERVICE_NAME", "songbird-orchestrator"),
            version: env_or_default("SONGBIRD_VERSION", "0.1.0"),
            base_url: base_url.clone(),
            health_endpoint: format!("{}/health", base_url),
            metrics_endpoint: format!("{}/metrics", base_url),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let bind_ip = env_or_default("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
        let orchestrator_port = env_or_default("SONGBIRD_ORCHESTRATOR_PORT", "8080");
        let gaming_port = env_or_default("SONGBIRD_GAMING_PORT", "8081");
        let federation_port = env_or_default("SONGBIRD_FEDERATION_PORT", "8082");
        let dashboard_port = env_or_default("SONGBIRD_DASHBOARD_PORT", "3000");

        Self {
            bind_address: bind_ip.parse().unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid SONGBIRD_BIND_ADDRESS, using default 127.0.0.1: {}",
                    e
                );
                "127.0.0.1".parse().expect("127.0.0.1 is a valid IP address")
            }),
            production_bind_address: env_or_default("SONGBIRD_PRODUCTION_BIND_ADDRESS", "0.0.0.0")
                .parse()
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Invalid SONGBIRD_PRODUCTION_BIND_ADDRESS, using default 0.0.0.0: {}",
                        e
                    );
                    "0.0.0.0".parse().expect("0.0.0.0 is a valid IP address")
                }),
            stun_servers: vec![
                env_or_default("SONGBIRD_STUN_SERVER_1", "stun.l.google.com:19302"),
                env_or_default("SONGBIRD_STUN_SERVER_2", "stun1.l.google.com:19302"),
            ],
            port_ranges: {
                let mut ranges = HashMap::new();
                ranges.insert("orchestrator".to_string(), (8080, 8090));
                ranges.insert("gaming".to_string(), (7000, 7100));
                ranges.insert("federation".to_string(), (8080, 8090));
                ranges.insert("primals".to_string(), (8080, 8090));
                ranges
            },
            orchestrator_endpoint: format!("http://{}:{}", bind_ip, orchestrator_port),
            gaming_endpoint: format!("http://{}:{}", bind_ip, gaming_port),
            federation_endpoint: format!("http://{}:{}", bind_ip, federation_port),
            dashboard_endpoint: format!("http://{}:{}", bind_ip, dashboard_port),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_CONNECTION_TIMEOUT", "30").parse().unwrap_or(30)
            ),
            request_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_REQUEST_TIMEOUT", "60").parse().unwrap_or(60)
            ),
            health_check_timeout: Duration::from_secs(
                env_or_default("SONGBIRD_HEALTH_CHECK_TIMEOUT", "5").parse().unwrap_or(5)
            ),
            heartbeat_interval: Duration::from_secs(
                env_or_default("SONGBIRD_HEARTBEAT_INTERVAL", "30").parse().unwrap_or(30)
            ),
            scaling_check_interval: Duration::from_secs(
                env_or_default("SONGBIRD_SCALING_CHECK_INTERVAL", "30").parse().unwrap_or(30)
            ),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            small_buffer_size: env_or_default("SONGBIRD_SMALL_BUFFER_SIZE", "1024").parse().unwrap_or(1024),
            large_buffer_size: env_or_default("SONGBIRD_LARGE_BUFFER_SIZE", "8192").parse().unwrap_or(8192),
            max_packet_size: env_or_default("SONGBIRD_MAX_PACKET_SIZE", "65536").parse().unwrap_or(65536),
            connection_pool_size: env_or_default("SONGBIRD_CONNECTION_POOL_SIZE", "10").parse().unwrap_or(10),
            cache_ttl: Duration::from_secs(
                env_or_default("SONGBIRD_CACHE_TTL", "300").parse().unwrap_or(300)
            ),
        }
    }
}

impl Default for PrimalConfig {
    fn default() -> Self {
        let base_ip = env_or_default("SONGBIRD_PRIMAL_BASE_IP", "127.0.0.1");
        let base_port: u16 = env_or_default("SONGBIRD_PRIMAL_BASE_PORT", "8080").parse().unwrap_or(8080);
        
        Self {
            beardog_endpoint: env_or_default(
                "SONGBIRD_BEARDOG_ENDPOINT", 
                &format!("https://{}:8443", base_ip)
            ),
            nestgate_endpoint: env_or_default(
                "SONGBIRD_NESTGATE_ENDPOINT", 
                &format!("http://{}:{}/storage", base_ip, base_port)
            ),
            toadstool_endpoint: env_or_default(
                "SONGBIRD_TOADSTOOL_ENDPOINT", 
                &format!("http://{}:8082", base_ip)
            ),
            squirrel_endpoint: env_or_default(
                "SONGBIRD_SQUIRREL_ENDPOINT", 
                &format!("http://{}:8083", base_ip)
            ),
            discovery_endpoints: vec![
                env_or_default("SONGBIRD_DISCOVERY_ENDPOINT_1", &format!("http://{}:{}/discovery", base_ip, base_port)),
                env_or_default("SONGBIRD_DISCOVERY_ENDPOINT_2", &format!("http://{}:8081/discovery", base_ip)),
            ],
            base_port,
            port_range: (
                env_or_default("SONGBIRD_PRIMAL_PORT_START", "8080").parse().unwrap_or(8080),
                env_or_default("SONGBIRD_PRIMAL_PORT_END", "8090").parse().unwrap_or(8090)
            ),
        }
    }
}

impl Default for FederationConfig {
    fn default() -> Self {
        let base_ip = env_or_default("SONGBIRD_FEDERATION_BASE_IP", "127.0.0.1");
        let base_port = env_or_default("SONGBIRD_FEDERATION_BASE_PORT", "8080");
        
        Self {
            cluster_endpoints: vec![
                env_or_default("SONGBIRD_CLUSTER_ENDPOINT_1", &format!("http://{}:{}", base_ip, base_port)),
                env_or_default("SONGBIRD_CLUSTER_ENDPOINT_2", &format!("http://{}:8081", base_ip)),
            ],
            heartbeat_endpoint: env_or_default(
                "SONGBIRD_HEARTBEAT_ENDPOINT", 
                &format!("http://{}:{}/federation/heartbeat", base_ip, base_port)
            ),
            broadcast_ports: vec![8080, 8081, 8082, 8090],
            discovery_ports: vec![8080, 8000, 3000, 5000],
            default_cluster_id: env_or_default("SONGBIRD_CLUSTER_ID", "songbird-cluster"),
            auto_discovery_enabled: env_or_default("SONGBIRD_AUTO_DISCOVERY", "true") == "true",
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

    /// Replace hardcoded "localhost:8080"
    pub fn orchestrator_endpoint() -> String {
        get_config().network.orchestrator_endpoint.clone()
    }

    /// Replace hardcoded "localhost:8081"
    pub fn gaming_endpoint() -> String {
        get_config().network.gaming_endpoint.clone()
    }

    /// Replace hardcoded "localhost:8443"
    pub fn beardog_endpoint() -> String {
        get_config().primals.beardog_endpoint.clone()
    }

    /// Replace hardcoded "localhost:8080/storage"
    pub fn nestgate_endpoint() -> String {
        get_config().primals.nestgate_endpoint.clone()
    }

    /// Replace hardcoded Duration::from_secs(30)
    pub fn connection_timeout() -> Duration {
        get_config().timeouts.connection_timeout
    }

    /// Replace hardcoded Duration::from_secs(60)
    pub fn request_timeout() -> Duration {
        get_config().timeouts.request_timeout
    }

    /// Replace hardcoded Duration::from_secs(5)
    pub fn health_check_timeout() -> Duration {
        get_config().timeouts.health_check_timeout
    }

    /// Replace hardcoded 8192
    pub fn large_buffer_size() -> usize {
        get_config().performance.large_buffer_size
    }

    /// Replace hardcoded STUN servers
    pub fn stun_servers() -> Vec<String> {
        get_config().network.stun_servers.clone()
    }

    /// Replace hardcoded federation endpoints
    pub fn federation_endpoints() -> Vec<String> {
        get_config().federation.cluster_endpoints.clone()
    }

    /// Replace hardcoded primal discovery endpoints
    pub fn primal_discovery_endpoints() -> Vec<String> {
        get_config().primals.discovery_endpoints.clone()
    }

    /// Replace hardcoded broadcast ports
    pub fn federation_broadcast_ports() -> Vec<u16> {
        get_config().federation.broadcast_ports.clone()
    }

    /// Replace hardcoded discovery ports
    pub fn federation_discovery_ports() -> Vec<u16> {
        get_config().federation.discovery_ports.clone()
    }

    /// Get production-ready bind address (0.0.0.0 vs 127.0.0.1)
    pub fn production_bind_address() -> IpAddr {
        if std::env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default() == "production" {
            get_config().network.production_bind_address
        } else {
            get_config().network.bind_address
        }
    }

    /// Format endpoint with configurable IP and port
    pub fn format_endpoint(service: &str, port_override: Option<u16>) -> String {
        let config = get_config();
        let ip = if std::env::var("SONGBIRD_ENVIRONMENT").unwrap_or_default() == "production" {
            config.network.production_bind_address
        } else {
            config.network.bind_address
        };

        let port = port_override.unwrap_or(
            match service {
                "orchestrator" => 8080,
                "gaming" => 8081,
                "federation" => 8082,
                "beardog" => 8443,
                "nestgate" => 8080,
                "toadstool" => 8082,
                "squirrel" => 8083,
                _ => 8080,
            }
        );

        let protocol = if port == 8443 { "https" } else { "http" };
        format!("{}://{}:{}", protocol, ip, port)
    }

    /// Format service endpoint with path
    pub fn format_service_endpoint(service: &str, path: &str, port_override: Option<u16>) -> String {
        let base = format_endpoint(service, port_override);
        format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
    }
}
