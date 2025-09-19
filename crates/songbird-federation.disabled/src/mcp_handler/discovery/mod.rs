//! MCP Federation Discovery
//!
//! Handles service discovery mechanisms for MCP federation:
//! - mDNS/Bonjour discovery
//! - UDP broadcast discovery
//! - Service registry lookup (Consul/etcd)
//! - DHT-based discovery
//! - Kubernetes service discovery
//! - Docker Swarm service discovery
//! - Network scanning
//!
//! ## Refactored Architecture
//!
//! The MCP handler discovery system is organized into focused modules:
//! - `manager` - Main DiscoveryManager coordination and auto-detection
//! - `mdns` - mDNS/Bonjour service discovery implementation
//! - `udp_broadcast` - UDP broadcast discovery and listeners
//! - `service_registry` - Consul/etcd service registry discovery
//! - `dht` - DHT-based discovery and network scanning
//! - `kubernetes` - Kubernetes service discovery
//! - `docker` - Docker Swarm service discovery
//! - `network_utils` - Network utility functions and scanning

pub mod dht;
pub mod docker;
pub mod kubernetes;
pub mod manager;
pub mod mdns;
pub mod network_utils;
pub mod service_registry;
pub mod udp_broadcast;

// Re-export main types for backward compatibility
pub use dht::{comprehensive_network_scan, discover_via_dht, targeted_service_scan};
pub use docker::{discover_from_docker_swarm, get_container_id, is_running_in_docker};
pub use kubernetes::{discover_from_kubernetes, get_current_namespace, is_running_in_kubernetes};
pub use manager::{DiscoveryManager, DiscoveryMethod, DiscoveryStatus};
pub use mdns::{advertise_via_mdns, discover_via_mdns};
pub use network_utils::{
    get_federation_ports, get_local_subnets, is_port_open, scan_subnet_for_federation,
    send_udp_broadcast, verify_federation_endpoint,
};
pub use service_registry::{
    discover_from_consul, discover_from_etcd, discover_via_service_registry,
};
pub use udp_broadcast::{
    create_discovery_response, discover_via_udp_broadcast, parse_discovery_response,
    start_discovery_listener, DiscoveryResponse, ServiceInfo,
};

// Legacy compatibility - re-export the main discovery function
/// Auto-detect federation endpoints using all available methods
pub async fn auto_detect(
    config: crate::config::FederationConfig,
) -> Result<Vec<String>, songbird_errors::SongbirdError> {
    let manager = DiscoveryManager::new(config);
    manager.auto_detect().await
}
