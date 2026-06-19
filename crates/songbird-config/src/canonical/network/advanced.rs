// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Advanced Network Features
//!
//! Advanced networking features including service discovery, SSL/TLS,
//! proxy configuration, TURN relays, `UPnP`, and network measurements.

#![allow(missing_docs, reason = "network matrix types; pair with `NetworkConfig` module docs")]

use serde::{Deserialize, Serialize};
use songbird_types::primal_names::SELF_NAME;

// ============================================================================
// SERVICE ENDPOINTS AND DISCOVERY
// ============================================================================

/// Service endpoint configuration for service discovery
///
/// **Merged from**: `unified/network.rs`\
/// **Purpose**: Detailed service endpoint information for discovery and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
    pub scheme: String, // http, https, tcp, etc.
    pub path: Option<String>,
    pub timeout_secs: Option<u64>,
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    #[must_use]
    pub fn new(host: &str, port: u16, scheme: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            scheme: scheme.to_string(),
            path: None,
            timeout_secs: None,
        }
    }

    /// Get the full URL for this endpoint
    #[must_use]
    pub fn full_url(&self) -> String {
        let base = format!("{}://{}:{}", self.scheme, self.host, self.port);
        match &self.path {
            Some(path) => format!("{base}{path}"),
            None => base,
        }
    }

    /// Create endpoint from environment variables
    #[must_use]
    pub fn from_env(env_prefix: &str) -> Option<Self> {
        let host_env = format!("{env_prefix}_HOST");
        let port_env = format!("{env_prefix}_PORT");
        let scheme_env = format!("{env_prefix}_SCHEME");

        let host = songbird_process_env::var(&host_env).ok()?;
        let port = songbird_process_env::var(&port_env).ok()?.parse().ok()?;
        let scheme =
            songbird_process_env::var(&scheme_env).unwrap_or_else(|_| String::from("http"));

        Some(Self::new(&host, port, &scheme))
    }
}

impl Default for ServiceEndpoint {
    fn default() -> Self {
        use songbird_types::constants::LOCALHOST;
        Self::new(LOCALHOST, songbird_types::defaults::ports::DEFAULT_HTTP_PORT, "http")
    }
}

/// Self-awareness configuration - each primal knows itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAwareConfig {
    pub id: String,
    pub endpoint: ServiceEndpoint,
    pub capabilities: Vec<String>,
}

impl Default for SelfAwareConfig {
    fn default() -> Self {
        Self {
            id: songbird_process_env::var("SONGBIRD_PRIMAL_ID")
                .unwrap_or_else(|_| SELF_NAME.to_string()),
            endpoint: ServiceEndpoint::from_env("SONGBIRD_SELF").unwrap_or_default(),
            capabilities: songbird_process_env::var("SONGBIRD_CAPABILITIES")
                .unwrap_or_else(|_| String::from("orchestration,service_discovery,load_balancing"))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }
}

/// Universal discovery configuration for capability-based routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalDiscoveryConfig {
    pub enabled: bool,
    pub discovery_methods: Vec<String>,
    pub service_discovery: ServiceDiscoveryEndpoints,
}

impl Default for UniversalDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_UNIVERSAL_DISCOVERY_ENABLED").is_ok(),
            discovery_methods: songbird_process_env::var("SONGBIRD_DISCOVERY_METHODS")
                .unwrap_or_else(|_| String::from("network_scan,service_registry,mdns"))
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            service_discovery: ServiceDiscoveryEndpoints::default(),
        }
    }
}

/// Service discovery endpoints for infrastructure backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryEndpoints {
    pub consul: Vec<ServiceEndpoint>,
    pub etcd: Vec<ServiceEndpoint>,
    pub kubernetes: Vec<ServiceEndpoint>,
    pub docker: Vec<ServiceEndpoint>,
}

impl Default for ServiceDiscoveryEndpoints {
    fn default() -> Self {
        use songbird_types::constants::LOCALHOST;
        Self {
            consul: vec![
                ServiceEndpoint::new(LOCALHOST, 8500, "http"),
                ServiceEndpoint::new(LOCALHOST, 8500, "http"),
            ],
            etcd: vec![
                ServiceEndpoint::new(LOCALHOST, 2379, "http"),
                ServiceEndpoint::new(LOCALHOST, 2380, "http"),
            ],
            kubernetes: vec![ServiceEndpoint::new(
                LOCALHOST,
                songbird_types::defaults::ports::DEFAULT_HTTP_PORT,
                "https",
            )],
            docker: vec![
                ServiceEndpoint::new(LOCALHOST, 2375, "http"),
                ServiceEndpoint::new(LOCALHOST, 2376, "https"),
            ],
        }
    }
}

// ============================================================================
// PROXY AND SSL CONFIGURATION
// ============================================================================

/// Reverse proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyConfig {
    pub enabled: bool,
    pub upstream_timeout_secs: u64,
    pub max_upstream_connections: usize,
}

impl Default for ReverseProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            upstream_timeout_secs: 30,
            max_upstream_connections: 100,
        }
    }
}

/// Advanced SSL/TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_SSL_ENABLED").is_ok(),
            cert_path: songbird_process_env::var("SONGBIRD_SSL_CERT").ok(),
            key_path: songbird_process_env::var("SONGBIRD_SSL_KEY").ok(),
            ca_path: songbird_process_env::var("SONGBIRD_SSL_CA").ok(),
        }
    }
}

/// HTTP/HTTPS proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProxyConfig {
    pub enabled: bool,
    pub bind_address: String,
    pub bind_port: u16,
    pub target_address: String,
    pub target_port: u16,
    pub connection_timeout_ms: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        use songbird_types::constants::{LOCALHOST, PRODUCTION_BIND_ADDRESS};
        use songbird_types::defaults::ports::DEFAULT_ORCHESTRATOR_PORT;
        Self {
            enabled: songbird_process_env::var("SONGBIRD_PROXY_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            bind_address: songbird_process_env::var("SONGBIRD_PROXY_BIND_ADDRESS")
                .unwrap_or_else(|_| PRODUCTION_BIND_ADDRESS.to_string()),
            bind_port: songbird_process_env::var("SONGBIRD_PROXY_BIND_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT),
            target_address: songbird_process_env::var("SONGBIRD_PROXY_TARGET_ADDRESS")
                .unwrap_or_else(|_| LOCALHOST.to_string()),
            target_port: songbird_process_env::var("SONGBIRD_PROXY_TARGET_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_ORCHESTRATOR_PORT),
            connection_timeout_ms: 5000,
        }
    }
}

/// Domain name configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DomainConfig {
    pub domain_name: String,
    pub tls_enabled: bool,
    pub certificate_path: Option<String>,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            domain_name: songbird_process_env::var("SONGBIRD_DOMAIN")
                .unwrap_or_else(|_| songbird_types::constants::LOCALHOST.to_string()),
            tls_enabled: songbird_process_env::var("SONGBIRD_TLS_ENABLED").is_ok(),
            certificate_path: songbird_process_env::var("SONGBIRD_TLS_CERT").ok(),
        }
    }
}

// ============================================================================
// NAT TRAVERSAL AND LOCAL DISCOVERY
// ============================================================================

/// TURN relay configuration for NAT traversal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TURNRelay {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub enabled: bool,
}

impl TURNRelay {
    /// Create a new TURN relay configuration
    #[must_use]
    pub const fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            enabled: true,
        }
    }

    /// Check if the TURN relay configuration is expired (disabled)
    #[must_use]
    pub const fn is_expired(&self) -> bool {
        !self.enabled
    }
}

/// `UPnP` device configuration for local network discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UPnPDevice {
    pub device_id: String,
    pub friendly_name: String,
    pub device_type: String,
    pub enabled: bool,
}

impl UPnPDevice {
    /// Create a new `UPnP` device configuration
    #[must_use]
    pub const fn new(device_id: String, friendly_name: String, device_type: String) -> Self {
        Self {
            device_id,
            friendly_name,
            device_type,
            enabled: true,
        }
    }
}

// ============================================================================
// NETWORK TOPOLOGY AND MEASUREMENTS
// ============================================================================

/// Network topology discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNetworkTopology {
    pub discovery_enabled: bool,
    pub topology_mapping: bool,
    pub peer_discovery_timeout: u64,
}

impl Default for DiscoveryNetworkTopology {
    fn default() -> Self {
        Self {
            discovery_enabled: true,
            topology_mapping: false,
            peer_discovery_timeout: 30,
        }
    }
}

/// Network connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub connection_id: String,
    pub remote_addr: String,
    pub connection_type: String,
    pub established_at: u64,
}

/// Network measurement and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeasurement {
    pub latency_ms: u64,
    pub bandwidth_mbps: f64,
    pub packet_loss_rate: f64,
    pub jitter_ms: u64,
}

impl Default for NetworkMeasurement {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            bandwidth_mbps: 0.0,
            packet_loss_rate: 0.0,
            jitter_ms: 0,
        }
    }
}

// ============================================================================
// TCP/UDP CONFIGURATION
// ============================================================================

/// TCP-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub keepalive: bool,
    pub keepalive_config: TcpKeepAliveConfig,
    pub nodelay: bool,
    pub buffer_config: SocketBufferConfig,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            keepalive: true,
            keepalive_config: TcpKeepAliveConfig::default(),
            nodelay: true,
            buffer_config: SocketBufferConfig::default(),
        }
    }
}

/// TCP keep-alive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpKeepAliveConfig {
    pub time_secs: u64,
    pub interval_secs: u64,
    pub probes: u32,
}

impl Default for TcpKeepAliveConfig {
    fn default() -> Self {
        Self {
            time_secs: 60,
            interval_secs: 10,
            probes: 5,
        }
    }
}

/// Socket buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketBufferConfig {
    pub recv_buffer_size: usize,
    pub send_buffer_size: usize,
}

impl Default for SocketBufferConfig {
    fn default() -> Self {
        Self {
            recv_buffer_size: 65536, // 64KB
            send_buffer_size: 65536, // 64KB
        }
    }
}

/// UDP-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    pub broadcast: bool,
    pub multicast: bool,
    pub multicast_ttl: u8,
    pub buffer_config: SocketBufferConfig,
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            broadcast: false,
            multicast: false,
            multicast_ttl: 1,
            buffer_config: SocketBufferConfig::default(),
        }
    }
}

/// Network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceConfig {
    pub bind_address: String,
    pub interface_name: Option<String>,
    pub ipv6_enabled: bool,
}

impl Default for NetworkInterfaceConfig {
    fn default() -> Self {
        Self {
            bind_address: songbird_types::constants::PRODUCTION_BIND_ADDRESS.to_string(),
            interface_name: None,
            ipv6_enabled: true,
        }
    }
}

#[cfg(test)]
#[path = "advanced_tests.rs"]
mod tests;
