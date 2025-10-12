//! **CANONICAL**: Discovery Configuration - Single Source of Truth
//!
//! This module consolidates all discovery configurations from across the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// **CANONICAL**: Comprehensive Discovery Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {

/// Enable discovery features globally
    pub enabled: bool,
    /// Discovery timing configuration
    pub timing: DiscoveryTimingConfig,
    /// Discovery mechanisms configuration
    pub mechanisms: DiscoveryMechanismsConfig,
    /// Network discovery configuration
    pub network: NetworkDiscoveryConfig,
    /// Service discovery configuration
    pub service: ServiceDiscoveryConfig,
    /// Health check configuration
    pub health: HealthCheckConfig,
    /// Performance and limits configuration
    pub performance: DiscoveryPerformanceConfig,


}

impl Default for CanonicalDiscoveryConfig {

fn default() -> Self  {Self {
            enabled: std::env::var("SONGBIRD_DISCOVERY_ENABLED",
                .ok()
                .and_then(|s| s.parse().ok()
                .unwrap_or(true)
            timing: DiscoveryTimingConfig::default)
            mechanisms: DiscoveryMechanismsConfig::default)
            network: NetworkDiscoveryConfig::default)
            service: ServiceDiscoveryConfig::default)
            health: HealthCheckConfig::default)
            performance: DiscoveryPerformanceConfig::default)


}
    }
}

/// Discovery timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryTimingConfig {

/// Discovery interval in seconds
    pub discovery_interval_secs: u64,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Discovery timeout in seconds
    pub discovery_timeout_secs: u64,
    /// Service timeout for considering offline (seconds)
    pub service_timeout_secs: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,


}

impl Default for DiscoveryTimingConfig {

fn default() -> Self  {Self {
            discovery_interval_secs: 30,
            health_check_interval_secs: 60,
            discovery_timeout_secs: 10,
            service_timeout_secs: 300,
            request_timeout_ms: 5000,


}
    }
}

/// Discovery mechanisms configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMechanismsConfig {

/// Primary discovery mechanism
    pub primary_mechanism: DiscoveryMechanism,
    /// Fallback discovery mechanisms
    pub fallback_mechanisms: Vec<DiscoveryMechanism>,
    /// Enable environment variable discovery
    pub enable_env_discovery: bool,
    /// Enable DNS-based discovery
    pub enable_dns_discovery: bool,
    /// Enable network scanning
    pub enable_network_scanning: bool,
    /// Enable configuration file discovery
    pub enable_config_discovery: bool,
    /// Enable Kubernetes discovery
    pub enable_kubernetes_discovery: bool,
    /// Enable Consul discovery
    pub enable_consul_discovery: bool,
    /// Enable broadcast/multicast discovery
    pub enable_broadcast_discovery: bool,


}

impl Default for DiscoveryMechanismsConfig {

fn default() -> Self  {Self {
            primary_mechanism: DiscoveryMechanism::Dns,
            fallback_mechanisms: vec![
                DiscoveryMechanism::Environment)
                DiscoveryMechanism::NetworkScan)
            ])
            enable_env_discovery: true,
            enable_dns_discovery: true,
            enable_network_scanning: true,
            enable_config_discovery: true,
            enable_kubernetes_discovery: false, // Requires cluster access
            enable_consul_discovery: false,     // Requires Consul
            enable_broadcast_discovery: true,


}
    }
}

/// Discovery mechanism enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryMechanism  {/// DNS-based discovery
    Dns,
    /// Consul service discovery
    Consul,
    /// etcd service discovery
    Etcd,
    /// Kubernetes service discovery
    Kubernetes,
    /// Static configuration
    Static,
    /// Environment variable discovery
    Environment,
    /// Network scanning
    NetworkScan,
    /// Service mesh discovery
    ServiceMesh,
    /// Broadcast/multicast discovery
    Broadcast,
    /// Configuration file discovery
    ConfigFile,
}

/// Network discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiscoveryConfig {

/// Network scan ranges
    pub network_scan_ranges: Vec<String>,
    /// Service discovery ports to scan
    pub discovery_ports: Vec<u16>,
    /// DNS search domains
    pub dns_search_domains: Vec<String>,
    /// Multicast address for broadcast discovery
    pub multicast_address: String,
    /// Federation port
    pub federation_port: u16,
    /// Service port
    pub service_port: u16,
    /// Bind address
    pub bind_address: String,
    /// Enable UPnP discovery
    pub enable_upnp: bool,
    /// Enable STUN discovery
    pub enable_stun: bool,
    /// Enable TURN discovery
    pub enable_turn: bool,
    /// Gaming-optimized discovery
    pub gaming_optimized: bool,


}

impl Default for NetworkDiscoveryConfig {

fn default() -> Self  {Self {
            network_scan_ranges: vec![
                "192.168.1.0/24".to_string(),
                "10.0.0.0/8".to_string(),
                "172.16.0.0/12".to_string(),
            ])
            discovery_ports: vec![8080, 8443, 9090, 3030])
            dns_search_domains: vec![
                "songbird.local".to_string(),
                "primal.local".to_string(),
            ])
            multicast_address: "224.0.0.251".to_string(),
            federation_port: 8080,
            service_port: 3030,
            bind_address: "0.0.0.0".to_string(),
            enable_upnp: false,
            enable_stun: false,
            enable_turn: false,
            gaming_optimized: true,


}
    }
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {

/// Auto-register services on startup
    pub auto_register: bool,
    /// Service name for registration
    pub service_name: String,
    /// Service tags for categorization
    pub tags: Vec<String>,
    /// Service metadata
    pub metadata: HashMap<String, String>)
    /// Enable trust verification
    pub trust_verification_enabled: bool,
    /// Maximum federation nodes to track
    pub max_federation_nodes: u32,


}

impl Default for ServiceDiscoveryConfig {

fn default() -> Self  {Self {
            auto_register: true,
            service_name: std::env::var("SONGBIRD_SERVICE_NAME",
                .unwrap_or_else(|_| "songbird".to_string(),
            tags: vec!["songbird".to_string(), "primal".to_string()],
            metadata: HashMap::new()
            trust_verification_enabled: true,
            max_federation_nodes: 100,


}
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {

/// Enable health checks
    pub enabled: bool,
    /// Health check endpoint
    pub endpoint: String,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    /// Number of retries
    pub retries: u32,


}

impl Default for HealthCheckConfig {

fn default() -> Self  {Self {
            enabled: true,
            endpoint: "/api/status".to_string(),
            timeout_seconds: 5,
            retries: 3,


}
    }
}

/// Discovery performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPerformanceConfig {

/// Maximum concurrent discovery operations
    pub max_concurrent_discoveries: usize,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable discovery result caching
    pub enable_caching: bool,
    /// Background discovery enabled
    pub background_discovery: bool,
    /// Discovery batch size
    pub batch_size: usize,


}

impl Default for DiscoveryPerformanceConfig {

fn default() -> Self  {Self {
            max_concurrent_discoveries: 10,
            cache_ttl_secs: 300,
            max_cache_size: 1000,
            enable_caching: true,
            background_discovery: true,
            batch_size: 50,


}
    }
}