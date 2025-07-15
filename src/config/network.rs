//! Network Configuration using OS Substrate
//!
//! Network configuration that uses toadstool and biomeOS for platform-agnostic
//! network operations instead of hardcoded values.

use crate::config::constants;
use crate::errors::SongbirdError;
use crate::substrate::{NetworkOperation, NetworkRequest, OSSubstrate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tracing::{debug, warn};

/// Network configuration using OS substrate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Port for orchestrator service
    pub bind_port: u16,

    /// Bind address for services
    pub bind_address: IpAddr,

    /// Health check configuration
    pub health_check_interval: Duration,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// TLS configuration
    pub enable_tls: bool,

    /// SSL/TLS configuration
    pub ssl_config: Option<SslConfig>,

    /// Domain configuration
    pub domain_config: Option<DomainConfig>,

    /// Proxy routes configuration
    pub proxy_routes: Vec<ProxyRoute>,

    /// Buffer size for network operations
    pub buffer_size: usize,

    /// Federation configuration
    pub federation_bind_address: IpAddr,
    pub federation_port: u16,

    /// CORS configuration
    pub cors: CorsConfig,

    /// Network endpoints discovered through substrate
    pub discovered_endpoints: HashMap<String, String>,

    /// Available network interfaces from substrate
    pub available_interfaces: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::new_fallback()
    }
}

impl NetworkConfig {
    /// Create new network configuration using OS substrate
    pub async fn new() -> crate::errors::Result<Self> {
        let substrate = crate::substrate::get_substrate().await;

        debug!("🌐 Configuring network through OS substrate (toadstool/biomeOS)");

        // Get network interface from substrate
        let network_interface = substrate.get_network_interface().await?;
        let bind_address = network_interface
            .ip_address
            .parse()
            .or_else(|_| constants::network::DEFAULT_BIND_ADDRESS.parse())
            .map_err(|e| SongbirdError::Config {
                message: format!("Failed to parse bind address: {}", e),
                field: Some("bind_address".to_string()),
                context: Some("Network configuration initialization".to_string()),
                suggestion: Some("Check your network configuration and DEFAULT_BIND_ADDRESS constant".to_string()),
            })?;

        // Get available port from substrate
        let bind_port = substrate.get_available_port().await?;

        // Get federation port (offset by 1000)
        let federation_port = substrate
            .get_available_port()
            .await
            .map(|p| p + 1000)
            .unwrap_or(constants::network::DEFAULT_FEDERATION_PORT);

        // Get system capabilities
        let system_info = substrate.get_system_info().await?;

        // Discover network endpoints through substrate
        let mut discovered_endpoints = HashMap::new();
        if let Ok(response) = substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::ResolveName,
                target: "services".to_string(),
                parameters: HashMap::new(),
            })
            .await
        {
            if let Some(endpoints) = response.get("endpoints").and_then(|v| v.as_object()) {
                for (name, endpoint) in endpoints {
                    if let Some(endpoint_str) = endpoint.as_str() {
                        discovered_endpoints.insert(name.clone(), endpoint_str.to_string());
                    }
                }
            }
        }

        // Get available interfaces
        let available_interfaces = system_info
            .network_interfaces
            .iter()
            .map(|iface| iface.name.clone())
            .collect();

        Ok(Self {
            bind_port,
            bind_address,
            health_check_interval: constants::network::DEFAULT_HEALTH_CHECK_INTERVAL,
            connection_timeout: constants::network::DEFAULT_CONNECTION_TIMEOUT,
            max_connections: constants::network::MAX_CONNECTIONS,
            enable_tls: false, // Will be configured by BearDog
            ssl_config: None,
            domain_config: None,
            proxy_routes: Vec::new(),
            buffer_size: constants::network::DEFAULT_BUFFER_SIZE,
            federation_bind_address: bind_address,
            federation_port,
            cors: CorsConfig::default(),
            discovered_endpoints,
            available_interfaces,
        })
    }

    /// Create fallback network configuration when substrate is unavailable
    pub fn new_fallback() -> Self {
        warn!("🔄 Using fallback network configuration (substrate unavailable)");

        let bind_address = std::env::var("SONGBIRD_BIND_ADDRESS")
            .and_then(|addr| addr.parse().ok())
            .or_else(|| constants::network::DEFAULT_BIND_ADDRESS.parse().ok())
            .unwrap_or_else(|| {
                warn!("Failed to parse bind address, using fallback");
                "127.0.0.1".parse().expect("Hardcoded IP should be valid")
            });

        let bind_port = std::env::var("SONGBIRD_BIND_PORT")
            .and_then(|port| port.parse().ok())
            .unwrap_or(constants::network::DEFAULT_ORCHESTRATOR_PORT);

        Self {
            bind_port,
            bind_address,
            health_check_interval: constants::network::DEFAULT_HEALTH_CHECK_INTERVAL,
            connection_timeout: constants::network::DEFAULT_CONNECTION_TIMEOUT,
            max_connections: constants::network::MAX_CONNECTIONS,
            enable_tls: false,
            ssl_config: None,
            domain_config: None,
            proxy_routes: Vec::new(),
            buffer_size: constants::network::DEFAULT_BUFFER_SIZE,
            federation_bind_address: bind_address,
            federation_port: constants::network::DEFAULT_FEDERATION_PORT,
            cors: CorsConfig::default(),
            discovered_endpoints: HashMap::new(),
            available_interfaces: vec!["default".to_string()],
        }
    }

    /// Refresh network configuration through substrate
    pub async fn refresh(&mut self) -> crate::errors::Result<()> {
        let substrate = crate::substrate::get_substrate().await;

        // Refresh network interface
        let network_interface = substrate.get_network_interface().await?;
        self.bind_address = network_interface
            .ip_address
            .parse()
            .unwrap_or(self.bind_address);

        // Refresh available port
        if let Ok(port) = substrate.get_available_port().await {
            self.bind_port = port;
        }

        // Refresh discovered endpoints
        if let Ok(response) = substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::ResolveName,
                target: "services".to_string(),
                parameters: HashMap::new(),
            })
            .await
        {
            if let Some(endpoints) = response.get("endpoints").and_then(|v| v.as_object()) {
                self.discovered_endpoints.clear();
                for (name, endpoint) in endpoints {
                    if let Some(endpoint_str) = endpoint.as_str() {
                        self.discovered_endpoints
                            .insert(name.clone(), endpoint_str.to_string());
                    }
                }
            }
        }

        debug!("🔄 Network configuration refreshed through substrate");
        Ok(())
    }

    /// Get endpoint for a service through substrate
    pub async fn get_service_endpoint(&self, service_name: &str) -> Option<String> {
        // Check discovered endpoints first
        if let Some(endpoint) = self.discovered_endpoints.get(service_name) {
            return Some(endpoint.clone());
        }

        // Try to discover through substrate
        let substrate = crate::substrate::get_substrate().await;
        if let Ok(response) = substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::ResolveName,
                target: service_name.to_string(),
                parameters: HashMap::new(),
            })
            .await
        {
            if let Some(endpoint) = response.get("endpoint").and_then(|v| v.as_str()) {
                return Some(endpoint.to_string());
            }
        }

        None
    }

    /// Configure firewall through substrate
    pub async fn configure_firewall(&self, rules: Vec<FirewallRule>) -> crate::errors::Result<()> {
        let substrate = crate::substrate::get_substrate().await;

        let mut parameters = HashMap::new();
        parameters.insert("rules".to_string(), serde_json::to_value(rules)?);

        substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::ConfigureFirewall,
                target: "songbird".to_string(),
                parameters,
            })
            .await?;

        debug!("🔥 Firewall configured through substrate");
        Ok(())
    }

    /// Check connectivity through substrate
    pub async fn check_connectivity(&self, target: &str) -> bool {
        let substrate = crate::substrate::get_substrate().await;

        if let Ok(response) = substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::CheckConnectivity,
                target: target.to_string(),
                parameters: HashMap::new(),
            })
            .await
        {
            if let Some(connected) = response.get("connected").and_then(|v| v.as_bool()) {
                return connected;
            }
        }

        false
    }

    /// Get orchestrator endpoint
    pub fn orchestrator_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.bind_address, self.bind_port)
    }

    /// Get federation endpoint
    pub fn federation_endpoint(&self) -> SocketAddr {
        SocketAddr::new(self.federation_bind_address, self.federation_port)
    }

    /// Validate network configuration
    pub async fn validate(&self) -> crate::errors::Result<()> {
        // Validate through substrate
        let substrate = crate::substrate::get_substrate().await;

        // Check if bind address is available
        if let Ok(response) = substrate
            .network_operation(NetworkRequest {
                operation: NetworkOperation::CheckConnectivity,
                target: self.bind_address.to_string(),
                parameters: HashMap::new(),
            })
            .await
        {
            if let Some(available) = response.get("available").and_then(|v| v.as_bool()) {
                if !available {
                    return Err(crate::errors::SongbirdError::Config {
                        field: Some("bind_address".to_string()),
                        message: format!("Bind address {} is not available", self.bind_address),
                    });
                }
            }
        }

        // Validate port ranges
        if self.bind_port < constants::validation::MIN_PORT
            || self.bind_port > constants::validation::MAX_PORT
        {
            return Err(crate::errors::SongbirdError::Config {
                field: Some("bind_port".to_string()),
                message: format!(
                    "Port {} is outside valid range {}-{}",
                    self.bind_port,
                    constants::validation::MIN_PORT,
                    constants::validation::MAX_PORT
                ),
            });
        }

        debug!("✅ Network configuration validated through substrate");
        Ok(())
    }
}

/// SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
}

/// Domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub domain: String,
    pub subdomain: Option<String>,
}

/// Proxy route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRoute {
    pub path: String,
    pub target: String,
    pub rewrite: Option<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            max_age: Some(3600),
        }
    }
}

/// Firewall rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub action: String,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
}

/// Get the best available network configuration
pub async fn get_network_config() -> crate::errors::Result<NetworkConfig> {
    // Try to use substrate first
    match NetworkConfig::new().await {
        Ok(config) => {
            debug!("✅ Using substrate-based network configuration");
            Ok(config)
        }
        Err(e) => {
            warn!(
                "⚠️ Substrate network configuration failed: {}, using fallback",
                e
            );
            Ok(NetworkConfig::new_fallback())
        }
    }
}

/// Initialize network configuration for a service
pub async fn initialize_service_network(
    service_name: &str,
) -> crate::errors::Result<ServiceNetworkConfig> {
    let substrate = crate::substrate::get_substrate().await;

    // Get service-specific network interface
    let network_interface = substrate.get_network_interface().await?;
    let service_port = substrate.get_available_port().await?;

    Ok(ServiceNetworkConfig {
        service_name: service_name.to_string(),
        bind_address: network_interface
            .ip_address
            .parse()
            .or_else(|_| constants::network::DEFAULT_BIND_ADDRESS.parse())
            .unwrap_or_else(|_| {
                warn!("Failed to parse service bind address, using fallback");
                "127.0.0.1".parse().expect("Hardcoded IP should be valid")
            }),
        service_port,
        health_endpoint: format!(
            "http://{}:{}/health",
            network_interface.ip_address, service_port
        ),
        metrics_endpoint: format!(
            "http://{}:{}/metrics",
            network_interface.ip_address, service_port
        ),
    })
}

/// Service-specific network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNetworkConfig {
    pub service_name: String,
    pub bind_address: IpAddr,
    pub service_port: u16,
    pub health_endpoint: String,
    pub metrics_endpoint: String,
}

/// Network configuration for testing
pub fn testing_network_config() -> NetworkConfig {
    NetworkConfig {
        bind_port: 0, // Use random port for testing
        bind_address: "127.0.0.1".parse().expect("Hardcoded localhost IP should be valid"),
        health_check_interval: Duration::from_secs(1),
        connection_timeout: Duration::from_secs(1),
        max_connections: 10,
        enable_tls: false,
        ssl_config: None,
        domain_config: None,
        proxy_routes: Vec::new(),
        buffer_size: 1024,
        federation_bind_address: "127.0.0.1".parse().expect("Hardcoded localhost IP should be valid"),
        federation_port: 0,
        cors: CorsConfig::default(),
        discovered_endpoints: HashMap::new(),
        available_interfaces: vec!["lo".to_string()],
    }
}
