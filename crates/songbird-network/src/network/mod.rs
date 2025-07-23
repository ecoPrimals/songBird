//! Network Layer Module
//!
//! Network configuration and management for the Songbird Orchestrator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use songbird_errors::{Result, SongbirdError};

pub mod discovery;
pub mod gaming;

/// Songbird Discovery Service - acts as the primary discovery coordinator
#[derive(Debug, Clone)]
pub struct SongbirdDiscoveryService {
    config: NetworkConfig,
    discovered_services: std::sync::Arc<tokio::sync::RwLock<HashMap<String, DiscoveredService>>>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub service_id: String,
    pub endpoint: String,
    pub service_type: String,
    pub capabilities: Vec<String>,
    pub last_seen: std::time::Instant,
    pub health_status: ServiceHealth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl SongbirdDiscoveryService {
    /// Create a new SongbirdDiscoveryService with the given configuration
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            discovered_services: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get the current network configuration
    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Update the network configuration
    pub fn update_config(&mut self, new_config: NetworkConfig) {
        self.config = new_config;
    }

    /// Register a service with the discovery system
    pub async fn register_service(&self, service: DiscoveredService) -> Result<()> {
        let mut services = self.discovered_services.write().await;
        services.insert(service.service_id.clone(), service);
        Ok(())
    }

    /// Discover services by capability
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        let services = self.discovered_services.read().await;
        let matching_services = services
            .values()
            .filter(|service| service.capabilities.contains(&capability.to_string()))
            .cloned()
            .collect();
        Ok(matching_services)
    }

    /// Get all healthy services
    pub async fn get_healthy_services(&self) -> Result<Vec<DiscoveredService>> {
        let services = self.discovered_services.read().await;
        let healthy_services = services
            .values()
            .filter(|service| service.health_status == ServiceHealth::Healthy)
            .cloned()
            .collect();
        Ok(healthy_services)
    }

    /// Perform ecosystem discovery (scan ../beardog, ../nestgate, etc.)
    pub async fn discover_ecosystem_services(&self) -> Result<Vec<DiscoveredService>> {
        let mut services = Vec::new();

        // Known ecosystem primals
        let ecosystem_services = vec![
            (
                "beardog",
                "8443",
                vec!["authentication", "security", "encryption"],
            ),
            (
                "nestgate",
                "8080",
                vec!["storage", "object_storage", "file_storage"],
            ),
            (
                "toadstool",
                "8082",
                vec!["compute", "serverless", "containers"],
            ),
            ("squirrel", "8084", vec!["ai", "llm", "machine_learning"]),
        ];

        for (service_name, default_port, capabilities) in ecosystem_services {
            // Check if ecosystem directory exists
            let ecosystem_path = format!("../{service_name}");
            if std::path::Path::new(&ecosystem_path).exists() {
                // Try to connect to the service
                let endpoints = vec![
                    format!("http://localhost:{}", default_port),
                    format!("http://127.0.0.1:{}", default_port),
                ];

                for endpoint in endpoints {
                    if self.test_endpoint_health(&endpoint).await? {
                        let service = DiscoveredService {
                            service_id: format!("{service_name}_ecosystem"),
                            endpoint,
                            service_type: service_name.to_string(),
                            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
                            last_seen: std::time::Instant::now(),
                            health_status: ServiceHealth::Healthy,
                        };
                        services.push(service);
                        break;
                    }
                }
            }
        }

        Ok(services)
    }

    async fn test_endpoint_health(&self, endpoint: &str) -> Result<bool> {
        let client = reqwest::Client::new();

        // Try common health endpoints
        let health_paths = vec!["/health", "/healthz", "/ping", "/status"];

        for path in health_paths {
            match client.get(format!("{endpoint}{path}")).send().await {
                Ok(response) if response.status().is_success() => return Ok(true),
                _ => continue,
            }
        }

        Ok(false)
    }
}

/// Network configuration for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: IpAddr,
    pub bind_port: u16,
    pub ssl_config: Option<SslConfig>,
    pub domain_config: Option<DomainConfig>,
    pub proxy_routes: Vec<ProxyRoute>,
    pub health_check_interval: Duration,
    pub connection_timeout: Duration,
    pub max_connections: u32,
    pub buffer_size: usize,
    pub enable_tls: bool,
    pub enable_ecosystem_discovery: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();

        Self {
            // Use environment configuration - NO MORE HARDCODING!
            bind_port: env_config.bind_port,
            bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid bind address {}, using 127.0.0.1: {}",
                    env_config.bind_address,
                    e
                );
                "127.0.0.1"
                    .parse()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            }),

            // Configurable timeouts from environment
            health_check_interval: std::time::Duration::from_secs(
                env_config.health_check_interval_secs,
            ),
            connection_timeout: env_config.connection_timeout(),

            // Security and performance from environment
            max_connections: env_config.max_connections as u32,
            enable_tls: env_config.require_tls,
            enable_ecosystem_discovery: true, // Enable by default
            ssl_config: None,
            domain_config: None,
            proxy_routes: Vec::new(),
            buffer_size: 8192,
        }
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
}

/// Domain configuration for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub primary_domain: String,
    pub alternative_domains: Vec<String>,
}

/// Proxy route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    pub path: String,
    pub target: String,
    pub proxy_type: ProxyType,
    pub health_check: Option<ProxyHealthCheck>,
}

/// Type of proxy routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Https,
    WebSocket,
}

/// Health check configuration for proxy routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyHealthCheck {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub path: String,
    pub expected_status: u16,
}

impl Default for ProxyHealthCheck {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            path: "/health".to_string(),
            expected_status: 200,
        }
    }
}

/// Network manager for handling connections and routing
pub struct NetworkManager {
    config: NetworkConfig,
    active_connections: HashMap<String, ConnectionInfo>,
    proxy_stats: HashMap<String, ProxyStats>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            active_connections: HashMap::new(),
            proxy_stats: HashMap::new(),
        }
    }

    /// Get the bind address for the server
    pub fn get_bind_address(&self) -> SocketAddr {
        SocketAddr::new(self.config.bind_address, self.config.bind_port)
    }

    /// Add a new connection
    pub fn add_connection(&mut self, connection_id: String, info: ConnectionInfo) {
        self.active_connections.insert(connection_id, info);
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, connection_id: &str) {
        self.active_connections.remove(connection_id);
    }

    /// Get active connection count
    pub fn get_active_connection_count(&self) -> usize {
        self.active_connections.len()
    }

    /// Get proxy statistics
    pub fn get_proxy_stats(&self, route_path: &str) -> Option<&ProxyStats> {
        self.proxy_stats.get(route_path)
    }

    /// Update proxy statistics
    pub fn update_proxy_stats(&mut self, route_path: String, stats: ProxyStats) {
        self.proxy_stats.insert(route_path, stats);
    }

    /// Check if we're at connection limit
    pub fn is_at_connection_limit(&self) -> bool {
        self.active_connections.len() >= self.config.max_connections as usize
    }

    /// Get network configuration
    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Update network configuration
    pub fn update_config(&mut self, config: NetworkConfig) {
        self.config = config;
    }

    /// Validate a proxy route
    pub fn validate_proxy_route(&self, route: &ProxyRoute) -> Result<()> {
        if route.path.is_empty() {
            return Err(SongbirdError::Config {
                field: Some("proxy_route.path".to_string()),
                message: "Proxy route path cannot be empty".to_string(),
                context: Some("network_configuration".to_string()),
                suggestion: Some("Check configuration values and network settings".to_string()),
            });
        }

        if route.target.is_empty() {
            return Err(SongbirdError::Config {
                field: Some("proxy_route.target".to_string()),
                message: "Proxy route target cannot be empty".to_string(),
                context: Some("network_configuration".to_string()),
                suggestion: Some("Check configuration values and network settings".to_string()),
            });
        }

        // Validate target URL format
        if !route.target.starts_with("http://") && !route.target.starts_with("https://") {
            return Err(SongbirdError::Config {
                field: Some("proxy_route.target".to_string()),
                message: format!("Invalid proxy target URL: {}", route.target),
                context: Some("proxy_route_validation".to_string()),
                suggestion: Some(
                    "Ensure proxy target URL starts with http:// or https://".to_string(),
                ),
            });
        }

        Ok(())
    }

    /// Get health check configuration for a route
    pub fn get_health_check_config(&self, route_path: &str) -> Option<&ProxyHealthCheck> {
        self.config
            .proxy_routes
            .iter()
            .find(|route| route.path == route_path)
            .and_then(|route| route.health_check.as_ref())
    }
}

/// Information about an active connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub remote_addr: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_type: String,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            remote_addr: "unknown".to_string(),
            connected_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
            connection_type: "http".to_string(),
        }
    }
}

/// Statistics for proxy routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub health_check_status: bool,
}

impl Default for ProxyStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_millis(0),
            last_health_check: None,
            health_check_status: false,
        }
    }
}

/// Network utilities
pub mod utils {
    use super::*;
    use std::net::ToSocketAddrs;
    use tracing::debug;

    /// Check if a host:port combination is reachable
    pub async fn is_reachable(host: &str, port: u16) -> bool {
        let addr = format!("{host}:{port}");
        debug!("Testing reachability for {}:{}", host, port);

        match addr.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(socket_addr) = addrs.next() {
                    tokio::net::TcpStream::connect(socket_addr).await.is_ok()
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Get the local IP address
    pub fn get_local_ip() -> Result<IpAddr> {
        // Try to connect to a remote address to determine local IP
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| {
            SongbirdError::NetworkDetection {
                message: format!("Failed to create socket: {e}").to_string(),
                interface: None,
                suggestion: Some("Check network permissions and socket availability".to_string()),
            }
        })?;

        socket
            .connect("8.8.8.8:80")
            .map_err(|e| SongbirdError::NetworkDetection {
                message: format!("Failed to connect to determine local IP: {e}").to_string(),
                interface: None,
                suggestion: Some("Check network connectivity and DNS resolution".to_string()),
            })?;

        let local_addr = socket
            .local_addr()
            .map_err(|e| SongbirdError::NetworkDetection {
                message: format!("Failed to get local address: {e}").to_string(),
                interface: None,
                suggestion: Some("Check socket binding and network interface status".to_string()),
            })?;

        Ok(local_addr.ip())
    }

    /// Validate an IP address string
    pub fn validate_ip_address(ip_str: &str) -> Result<IpAddr> {
        ip_str.parse().map_err(|e| SongbirdError::Config {
            field: Some("ip_address".to_string()),
            message: format!("Invalid IP address '{ip_str}': {e}").to_string(),
            context: Some("ip_address_validation".to_string()),
            suggestion: Some("Ensure IP address format is valid (e.g., 192.168.1.1)".to_string()),
        })
    }

    /// Validate a port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(SongbirdError::Config {
                field: Some("port".to_string()),
                message: "Port cannot be 0".to_string(),
                context: Some("network_configuration".to_string()),
                suggestion: Some("Check configuration values and network settings".to_string()),
            });
        }
        if port < 1024 {
            tracing::warn!(
                "Using privileged port {}, may require elevated permissions",
                port
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();
        assert_eq!(config.bind_port, env_config.bind_port);
        assert_eq!(config.max_connections, 1000);
        assert!(config.ssl_config.is_none());
    }

    #[test]
    fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        assert_eq!(manager.get_active_connection_count(), 0);
        assert!(!manager.is_at_connection_limit());
    }

    #[test]
    fn test_proxy_route_validation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();

        let valid_route = ProxyRoute {
            path: "/api".to_string(),
            target: format!(
                "http://{}:{}",
                env_config.bind_address, env_config.dashboard_port
            )
            .to_string(),
            proxy_type: ProxyType::Http,
            health_check: None,
        };

        assert!(manager.validate_proxy_route(&valid_route).is_ok());

        let invalid_route = ProxyRoute {
            path: "".to_string(),
            target: format!(
                "http://{}:{}",
                env_config.bind_address, env_config.dashboard_port
            )
            .to_string(),
            proxy_type: ProxyType::Http,
            health_check: None,
        };

        assert!(manager.validate_proxy_route(&invalid_route).is_err());
    }

    #[tokio::test]
    async fn test_utils_validate_ip() {
        use utils::*;

        let default_ip = songbird_config::config::constants::default_bind_address();
        assert!(validate_ip_address(&default_ip).is_ok());
        assert!(validate_ip_address("::1").is_ok());
        assert!(validate_ip_address("invalid").is_err());
    }

    #[test]
    fn test_utils_validate_port() {
        use utils::*;

        let env_config = songbird_config::config::environment::EnvironmentConfig::default();
        assert!(validate_port(env_config.bind_port).is_ok());
    }
}
