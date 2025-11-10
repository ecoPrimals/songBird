//! Core types for universal discovery system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use songbird_types::SongbirdResult;

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Discovery configuration for universal discovery
///
/// **LOCAL DEFINITION**: Network-focused with comprehensive field alignment.
/// **MODERN RUST**: Detailed network discovery configuration.
/// Field mappings to canonical:
/// - `enable_auto_discovery` → auto_discovery (top-level)
/// - `discovery_interval` (Duration) → service_discovery.discovery_interval_secs (u64, convert)
/// - `enable_network_scanning` → network_discovery.scan_local_network
/// - `network_scan_ranges` → Part of network_discovery scope
/// - `discovery_ports` → common_ports
/// - `enable_dns_discovery` → network_discovery.enabled (DNS as protocol)
/// - `dns_discovery_domains` → network_discovery.discovery_protocols
/// - `enable_multicast_discovery` → network_discovery.enabled (multicast as protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable automatic discovery (→ auto_discovery)
    pub enable_auto_discovery: bool,
    /// Discovery interval (→ service_discovery.discovery_interval_secs)
    pub discovery_interval: Duration,
    /// Enable network scanning (→ network_discovery.scan_local_network)
    pub enable_network_scanning: bool,
    /// Network scan ranges (part of network_discovery scope)
    pub network_scan_ranges: Vec<String>,
    /// Discovery ports to scan (→ common_ports)
    pub discovery_ports: Vec<u16>,
    /// Enable DNS discovery (→ network_discovery.discovery_protocols: "dns")
    pub enable_dns_discovery: bool,
    /// DNS discovery domains (→ network_discovery.discovery_protocols)
    pub dns_discovery_domains: Vec<String>,
    /// Enable multicast discovery (→ network_discovery.discovery_protocols: "multicast")
    pub enable_multicast_discovery: bool,
    /// Multicast addresses
    pub multicast_addresses: Vec<String>,
    /// Enable Kubernetes discovery
    pub enable_kubernetes_discovery: bool,
    /// Kubernetes namespace
    pub kubernetes_namespace: Option<String>,
    /// Enable Consul discovery
    pub enable_consul_discovery: bool,
    /// Consul endpoints
    pub consul_endpoints: Vec<String>,
    /// Service health check interval
    pub health_check_interval: Duration,
    /// Service timeout
    pub service_timeout: Duration,
    /// Maximum concurrent discoveries
    pub max_concurrent_discoveries: usize,
    /// Discovery retry attempts
    pub discovery_retry_attempts: u32,
    /// Discovery retry delay
    pub discovery_retry_delay: Duration,
}

impl Default for DiscoveryConfig  {fn default() -> Self  {Self {
            enable_auto_discovery: true,
            discovery_interval: Duration::from_secs(300), // 5 minutes
            enable_network_scanning: true,
            network_scan_ranges: vec!["192.168.0.0/16".to_string(), "10.0.0.0/8".to_string()],
            discovery_ports: vec![8080, 8081, 8082, 8083, 8084, 6112])
            enable_dns_discovery: true,
            dns_discovery_domains: vec!["local".to_string(), "songbird".to_string()],
            enable_multicast_discovery: true,
            multicast_addresses: vec!["224.0.0.251:5353".to_string()],
            enable_kubernetes_discovery: false,
            kubernetes_namespace: None,
            enable_consul_discovery: false,
            consul_endpoints: Vec::new(),
            health_check_interval: Duration::from_secs(60)
            service_timeout: Duration::from_secs(30)
            max_concurrent_discoveries: 10,
            discovery_retry_attempts: 3,
            discovery_retry_delay: Duration::from_secs(5),
        }
    }
}

/// Discovered service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService  {/// Unique service identifier
    pub service_id: String,
    /// Service name
    pub service_name: String,
    /// Service endpoint
    pub endpoint: String,
    /// Service capabilities
    pub capabilities: Vec<ServiceCapability>,
    /// Service metadata
    pub metadata: ServiceMetadata,
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    /// Discovery timestamp
    pub discovered_at: SystemTime,
    /// Last health check
    pub last_health_check: Option<SystemTime>,
    /// Health status
    pub health_status: UniversalHealthStatus,
    /// Service version
    pub version: Option<String>,
    /// Service tags
    pub tags: Vec<String>,
}

/// Service discovery methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryMethod  {NetworkScan)
    DnsDiscovery,
    MulticastDiscovery,
    KubernetesApi,
    ConsulApi,
    Manual,
    Configuration,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability  {/// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Capability parameters
    pub parameters: HashMap<String, serde_json::Value>)
    /// Quality of service metrics
    pub qos_metrics: Option<QosMetrics>,
}

/// Quality of Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosMetrics  {/// Average latency in milliseconds
    pub latency_ms: Option<f64>,
    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,
    /// Availability percentage
    pub availability: Option<f64>,
    /// Reliability score
    pub reliability: Option<f64>,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata  {/// Service description
    pub description: Option<String>,
    /// Service owner
    pub owner: Option<String>,
    /// Service environment
    pub environment: Option<String>,
    /// Additional metadata
    pub additional: HashMap<String, serde_json::Value>)
}

/// Universal health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UniversalHealthStatus  {Healthy)
    Degraded,
    Unhealthy,
    Unknown,
}

/// Discovery events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryEvent  {ServiceDiscovered  {service_id: String,
        service_name: String,
        endpoint: String,
        method: DiscoveryMethod,
        timestamp: SystemTime,
    })
    ServiceLost  {service_id: String,
        service_name: String,
        last_seen: SystemTime,
        timestamp: SystemTime,
    })
    ServiceHealthChanged  {service_id: String,
        old_status: UniversalHealthStatus,
        new_status: UniversalHealthStatus,
        timestamp: SystemTime,
    })
    DiscoveryError  {method: DiscoveryMethod,
        error: String,
        timestamp: SystemTime,
    })
}