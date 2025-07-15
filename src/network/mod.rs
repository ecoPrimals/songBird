//! Network Layer Module
//!
//! Network configuration and management for the Songbird Orchestrator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::errors::{Result, SongbirdError};
use crate::proxy::{ConnectionProxy, ProxyConfig};
use crate::traits::ServiceInfo;

pub mod gaming;
// FRAGO: BearDog Integration Module
// FRAGO: Network Discovery Engine Module
pub mod beardog_integration;
pub mod discovery_engine;

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
}

impl Default for NetworkConfig {
    fn default() -> Self {
        let env_config = crate::config::environment::EnvironmentConfig::default();

        Self {
            // Use environment configuration - NO MORE HARDCODING!
            bind_port: env_config.bind_port,
            bind_address: env_config.bind_address.parse().unwrap_or_else(|e| {
                let fallback_addr = crate::config::constants::network::default_bind_address();
                tracing::warn!(
                    "Invalid bind address {}, using {}: {}",
                    env_config.bind_address,
                    fallback_addr,
                    e
                );
                fallback_addr
                    .parse()
                    .unwrap_or_else(|_| std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
            }),

            // Configurable timeouts from environment
            health_check_interval: std::time::Duration::from_secs(
                env_config.health_check_interval_secs,
            ),
            connection_timeout: env_config.connection_timeout(),

            // Security and performance from environment
            max_connections: env_config.max_connections,
            enable_tls: env_config.require_tls,
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

/// SSL manager for handling TLS certificates and connections
#[derive(Debug)]
pub struct SslManager {
    config: SslConfig,
    certificates: Arc<RwLock<HashMap<String, Certificate>>>,
}

/// SSL certificate information
#[derive(Debug, Clone)]
pub struct Certificate {
    pub cert_data: Vec<u8>,
    pub key_data: Vec<u8>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub domains: Vec<String>,
}

impl SslManager {
    /// Create new SSL manager
    pub fn new(config: SslConfig) -> Result<Self> {
        Ok(Self {
            config,
            certificates: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load SSL certificate from files
    pub async fn load_certificate(&self, domain: &str) -> Result<()> {
        let cert_data =
            tokio::fs::read(&self.config.cert_path)
                .await
                .map_err(|e| SongbirdError::Config {
                    field: Some("ssl_cert_path".to_string()),
                    message: format!("Failed to read SSL certificate: {e}"),
                })?;

        let key_data =
            tokio::fs::read(&self.config.key_path)
                .await
                .map_err(|e| SongbirdError::Config {
                    field: Some("ssl_key_path".to_string()),
                    message: format!("Failed to read SSL private key: {e}"),
                })?;

        // For now, create a placeholder certificate
        // In a real implementation, we'd parse the X.509 certificate
        let certificate = Certificate {
            cert_data,
            key_data,
            expires_at: chrono::Utc::now() + chrono::Duration::days(365),
            domains: vec![domain.to_string()],
        };

        let mut certificates = self.certificates.write().await;
        certificates.insert(domain.to_string(), certificate);

        tracing::info!("Loaded SSL certificate for domain: {}", domain);
        Ok(())
    }

    /// Check if SSL is properly configured
    pub async fn is_configured(&self) -> bool {
        !self.certificates.read().await.is_empty()
    }

    /// Get certificate for domain
    pub async fn get_certificate(&self, domain: &str) -> Option<Certificate> {
        self.certificates.read().await.get(domain).cloned()
    }
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
    active_connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    proxy_stats: Arc<RwLock<HashMap<String, ProxyStats>>>,
    reverse_proxy: Option<Arc<ConnectionProxy>>,
    ssl_manager: Option<SslManager>,
    lan_discovery: Option<Arc<discovery_engine::NetworkDiscoveryEngine>>,
    running: Arc<RwLock<bool>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            proxy_stats: Arc::new(RwLock::new(HashMap::new())),
            reverse_proxy: None,
            ssl_manager: None,
            lan_discovery: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the network manager with all components
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing network manager...");

        // Initialize SSL manager if SSL is enabled
        if self.config.enable_tls {
            if let Some(ssl_config) = &self.config.ssl_config {
                let ssl_manager = SslManager::new(ssl_config.clone())?;

                // Load certificates for configured domains
                if let Some(domain_config) = &self.config.domain_config {
                    ssl_manager
                        .load_certificate(&domain_config.primary_domain)
                        .await?;

                    for domain in &domain_config.alternative_domains {
                        if let Err(e) = ssl_manager.load_certificate(domain).await {
                            tracing::warn!(
                                "Failed to load certificate for domain {}: {}",
                                domain,
                                e
                            );
                        }
                    }
                }

                self.ssl_manager = Some(ssl_manager);
                tracing::info!("SSL manager initialized");
            } else {
                tracing::warn!("SSL enabled but no SSL configuration provided");
            }
        }

        // Initialize reverse proxy
        let proxy_config = ProxyConfig {
            bind_address: self.config.bind_address.to_string(),
            port: self.config.bind_port,
            enable_ssl: self.config.enable_tls,
            ssl_cert_path: self
                .config
                .ssl_config
                .as_ref()
                .map(|ssl| ssl.cert_path.clone()),
            ssl_key_path: self
                .config
                .ssl_config
                .as_ref()
                .map(|ssl| ssl.key_path.clone()),
            request_timeout: self.config.connection_timeout.as_secs(),
            connection_timeout: self.config.connection_timeout.as_secs(),
            max_retries: 3,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: 60,
            enable_load_balancing: true,
            load_balancing_strategy: crate::proxy::LoadBalancingStrategy::RoundRobin,
            enable_logging: true,
            enable_compression: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
        };

        let reverse_proxy = Arc::new(ConnectionProxy::new(proxy_config));
        self.reverse_proxy = Some(reverse_proxy);
        tracing::info!("Reverse proxy initialized");

        // Initialize LAN discovery
        let discovery_config = discovery_engine::DiscoveryConfig {
            discovery_timeout: Duration::from_secs(5),
            peer_timeout: Duration::from_secs(30),
            topology_update_interval: self.config.health_check_interval,
            max_peers: 100,
            enable_upnp: true,
            enable_stun: true,
            enable_turn: false,
            gaming_optimized: true,
        };

        let lan_discovery = Arc::new(discovery_engine::NetworkDiscoveryEngine::new(
            discovery_config,
        ));
        self.lan_discovery = Some(lan_discovery);
        tracing::info!("LAN discovery initialized");

        tracing::info!("Network manager initialization completed");
        Ok(())
    }

    /// Start the network manager and all its components
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting network manager...");

        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Start reverse proxy
        if let Some(proxy) = &self.reverse_proxy {
            proxy.start().await.map_err(|e| SongbirdError::Network {
                service: "reverse_proxy".to_string(),
                message: format!("Failed to start reverse proxy: {e}"),
                details: None,
            })?;
            tracing::info!("Reverse proxy server started");
        }

        // Start LAN discovery
        if let Some(discovery) = &self.lan_discovery {
            tokio::spawn({
                let discovery = Arc::clone(discovery);
                async move {
                    let config =
                        crate::config::hardcoded_elimination::HardcodedEliminationConfig::new();
                    let discovery_interval = config.local_discovery_interval();

                    loop {
                        if let Err(e) = discovery.discover_peers().await {
                            tracing::warn!("LAN discovery error: {}", e);
                        }
                        tokio::time::sleep(discovery_interval).await;
                    }
                }
            });
            tracing::info!("LAN discovery started");
        }

        tracing::info!("Network manager started successfully");
        Ok(())
    }

    /// Stop the network manager and all its components
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping network manager...");

        {
            let mut running = self.running.write().await;
            if !*running {
                return Ok(());
            }
            *running = false;
        }

        // Stop reverse proxy
        if let Some(proxy) = &self.reverse_proxy {
            proxy.stop().await.map_err(|e| SongbirdError::Network {
                service: "reverse_proxy".to_string(),
                message: format!("Failed to stop reverse proxy: {e}"),
                details: None,
            })?;
            tracing::info!("Reverse proxy stopped");
        }

        tracing::info!("Network manager stopped successfully");
        Ok(())
    }

    /// Check if the network manager is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Update service registry for reverse proxy
    pub async fn update_services(&self, services: Vec<ServiceInfo>) -> Result<()> {
        if let Some(proxy) = &self.reverse_proxy {
            proxy
                .update_services(services)
                .await
                .map_err(|e| SongbirdError::Network {
                    service: "reverse_proxy".to_string(),
                    message: format!("Failed to update services: {e}"),
                    details: None,
                })?;
        }
        Ok(())
    }

    /// Get reverse proxy statistics
    pub async fn get_proxy_statistics(&self) -> Option<crate::proxy::ProxyStats> {
        if let Some(proxy) = &self.reverse_proxy {
            Some(proxy.get_stats().await)
        } else {
            None
        }
    }

    /// Discover LAN peers
    pub async fn discover_lan_peers(
        &self,
    ) -> Result<Vec<crate::network::beardog_integration::PeerCapabilities>> {
        if let Some(discovery) = &self.lan_discovery {
            discovery.discover_peers().await
        } else {
            Ok(vec![])
        }
    }

    /// Get network topology
    pub async fn get_network_topology(&self) -> Result<discovery_engine::NetworkTopology> {
        if let Some(discovery) = &self.lan_discovery {
            discovery.map_network_topology().await
        } else {
            Ok(discovery_engine::NetworkTopology {
                nodes: HashMap::new(),
                connections: vec![],
                measured_at: std::time::Instant::now(),
                quality_score: 0.0,
            })
        }
    }

    /// Get the bind address for the server
    pub fn get_bind_address(&self) -> SocketAddr {
        SocketAddr::new(self.config.bind_address, self.config.bind_port)
    }

    /// Add a new connection
    pub async fn add_connection(&self, connection_id: String, info: ConnectionInfo) {
        let mut connections = self.active_connections.write().await;
        connections.insert(connection_id, info);
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.active_connections.write().await;
        connections.remove(connection_id);
    }

    /// Get active connection count
    pub async fn get_active_connection_count(&self) -> usize {
        self.active_connections.read().await.len()
    }

    /// Get proxy statistics
    pub async fn get_proxy_stats(&self, route_path: &str) -> Option<ProxyStats> {
        self.proxy_stats.read().await.get(route_path).cloned()
    }

    /// Update proxy statistics
    pub async fn update_proxy_stats(&self, route_path: String, stats: ProxyStats) {
        let mut proxy_stats = self.proxy_stats.write().await;
        proxy_stats.insert(route_path, stats);
    }

    /// Check if we're at connection limit
    pub async fn is_at_connection_limit(&self) -> bool {
        self.active_connections.read().await.len() >= self.config.max_connections as usize
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
            });
        }

        if route.target.is_empty() {
            return Err(SongbirdError::Config {
                field: Some("proxy_route.target".to_string()),
                message: "Proxy route target cannot be empty".to_string(),
            });
        }

        // Validate target URL format
        if !route.target.starts_with("http://") && !route.target.starts_with("https://") {
            return Err(SongbirdError::Config {
                field: Some("proxy_route.target".to_string()),
                message: format!("Invalid proxy target URL: {}", route.target),
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

    /// Check if a host:port combination is reachable
    pub async fn is_reachable(host: &str, port: u16) -> bool {
        let addr = format!("{host}:{port}");
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
        let bind_addr = format!("{}:0", crate::config::constants::network::production_bind_address());
        let socket = std::net::UdpSocket::bind(&bind_addr).map_err(|e| {
            SongbirdError::NetworkDetection(format!("Failed to create socket: {e}"))
        })?;

        socket.connect("8.8.8.8:80").map_err(|e| {
            SongbirdError::NetworkDetection(format!("Failed to connect to determine local IP: {e}"))
        })?;

        let local_addr = socket.local_addr().map_err(|e| {
            SongbirdError::NetworkDetection(format!("Failed to get local address: {e}"))
        })?;

        Ok(local_addr.ip())
    }

    /// Validate an IP address string
    pub fn validate_ip_address(ip_str: &str) -> Result<IpAddr> {
        ip_str.parse().map_err(|e| SongbirdError::Config {
            field: Some("ip_address".to_string()),
            message: format!("Invalid IP address '{ip_str}': {e}"),
        })
    }

    /// Validate a port number
    pub fn validate_port(port: u16) -> Result<()> {
        if port == 0 {
            return Err(SongbirdError::Config {
                field: Some("port".to_string()),
                message: "Port cannot be 0".to_string(),
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
        let env_config = crate::config::environment::EnvironmentConfig::default();
        assert_eq!(config.bind_port, env_config.bind_port);
        assert_eq!(config.max_connections, 1000);
        assert!(config.ssl_config.is_none());
    }

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        assert_eq!(manager.get_active_connection_count().await, 0);
        assert!(!manager.is_at_connection_limit().await);
    }

    #[test]
    fn test_proxy_route_validation() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config);
        let env_config = crate::config::environment::EnvironmentConfig::default();

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

        let default_ip = crate::config::constants::default_bind_address();
        assert!(validate_ip_address(&default_ip).is_ok());
        assert!(validate_ip_address("::1").is_ok());
        assert!(validate_ip_address("invalid").is_err());
    }

    #[test]
    fn test_utils_validate_port() {
        use utils::*;

        let env_config = crate::config::environment::EnvironmentConfig::default();
        assert!(validate_port(env_config.bind_port).is_ok());
    }
}
