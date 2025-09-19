//! Main discovery manager that coordinates all discovery methods

use crate::config::FederationConfig;
use songbird_errors::SongbirdError;
use tracing::{debug, info};

use super::{
    dht::discover_via_dht, docker::discover_from_docker_swarm,
    kubernetes::discover_from_kubernetes, mdns::discover_via_mdns,
    network_utils::verify_federation_endpoint, service_registry::discover_via_service_registry,
    udp_broadcast::discover_via_udp_broadcast,
};

/// Discovery manager for MCP federation endpoints
#[derive(Debug)]
pub struct DiscoveryManager {
    config: FederationConfig,
}

impl DiscoveryManager {
    /// Create new discovery manager
    pub fn new(config: FederationConfig) -> Self {
        Self { config }
    }

    /// Auto-detect federation endpoints using all available methods
    pub async fn auto_detect(&self) -> Result<Vec<String>, SongbirdError> {
        info!("Starting MCP federation auto-detection");

        let mut discovered_endpoints = Vec::with_capacity(64); // Pre-allocate for expected endpoints

        // 1. mDNS/Bonjour service discovery
        if let Ok(endpoints) = discover_via_mdns().await {
            discovered_endpoints.extend(endpoints);
            info!(
                "mDNS discovery found {} endpoints",
                discovered_endpoints.len()
            );
        }

        // 2. UDP broadcast discovery
        if let Ok(endpoints) = discover_via_udp_broadcast().await {
            let current_len = discovered_endpoints.len();
            discovered_endpoints.extend(endpoints);
            info!(
                "UDP broadcast discovery found {} endpoints",
                discovered_endpoints.len() - current_len
            );
        }

        // 3. Consul/etcd service registry lookup
        if let Ok(endpoints) = discover_via_service_registry().await {
            let current_len = discovered_endpoints.len();
            discovered_endpoints.extend(endpoints);
            info!(
                "Service registry discovery found {} endpoints",
                discovered_endpoints.len() - current_len
            );
        }

        // 4. DHT-based discovery
        if let Ok(endpoints) = discover_via_dht(&self.config).await {
            let current_len = discovered_endpoints.len();
            discovered_endpoints.extend(endpoints);
            info!(
                "DHT discovery found {} endpoints",
                discovered_endpoints.len() - current_len
            );
        }

        // 5. Kubernetes discovery (if running in K8s)
        if super::kubernetes::is_running_in_kubernetes() {
            if let Ok(endpoints) = discover_from_kubernetes().await {
                let current_len = discovered_endpoints.len();
                discovered_endpoints.extend(endpoints);
                info!(
                    "Kubernetes discovery found {} endpoints",
                    discovered_endpoints.len() - current_len
                );
            }
        }

        // 6. Docker Swarm discovery (if running in Docker)
        if super::docker::is_running_in_docker() {
            if let Ok(endpoints) = discover_from_docker_swarm().await {
                let current_len = discovered_endpoints.len();
                discovered_endpoints.extend(endpoints);
                info!(
                    "Docker discovery found {} endpoints",
                    discovered_endpoints.len() - current_len
                );
            }
        }

        // 7. Network scanning (fallback if no other methods found endpoints)
        if discovered_endpoints.is_empty() {
            if let Ok(endpoints) = self.discover_via_network_scan().await {
                discovered_endpoints.extend(endpoints);
                info!(
                    "Network scan discovery found {} endpoints",
                    discovered_endpoints.len()
                );
            }
        }

        // Remove duplicates and validate endpoints
        discovered_endpoints.sort();
        discovered_endpoints.dedup();

        // Final validation of all discovered endpoints
        let mut validated_endpoints = Vec::new();
        for endpoint in discovered_endpoints {
            if verify_federation_endpoint(&endpoint).await? {
                validated_endpoints.push(endpoint);
            }
        }

        info!(
            "Auto-detection completed: {} unique validated endpoints found",
            validated_endpoints.len()
        );

        Ok(validated_endpoints)
    }

    /// Perform network scanning as fallback discovery method
    pub async fn discover_via_network_scan(&self) -> Result<Vec<String>, SongbirdError> {
        debug!("Starting network scan discovery (fallback method)");

        // Use comprehensive network scan from DHT module
        super::dht::comprehensive_network_scan().await
    }

    /// Get configuration
    pub fn config(&self) -> &FederationConfig {
        &self.config
    }

    /// Validate a list of endpoints
    pub async fn validate_endpoints(
        &self,
        endpoints: Vec<String>,
    ) -> Result<Vec<String>, SongbirdError> {
        let mut validated_endpoints = Vec::new();
        for endpoint in endpoints {
            if verify_federation_endpoint(&endpoint).await? {
                validated_endpoints.push(endpoint);
            }
        }
        Ok(validated_endpoints)
    }

    /// Set configuration without async
    pub fn set_config(&mut self, config: FederationConfig) {
        self.config = config;
    }

    /// Discover endpoints using specific methods
    pub async fn discover_with_methods(
        &self,
        methods: &[DiscoveryMethod],
    ) -> Result<Vec<String>, SongbirdError> {
        let mut endpoints = Vec::new();

        for method in methods {
            let method_endpoints = match method {
                DiscoveryMethod::MDNS => discover_via_mdns().await.unwrap_or_default(),
                DiscoveryMethod::UDPBroadcast => {
                    discover_via_udp_broadcast().await.unwrap_or_default()
                }
                DiscoveryMethod::ServiceRegistry => {
                    discover_via_service_registry().await.unwrap_or_default()
                }
                DiscoveryMethod::DHT => discover_via_dht(&self.config).await.unwrap_or_default(),
                DiscoveryMethod::Kubernetes => discover_from_kubernetes().await.unwrap_or_default(),
                DiscoveryMethod::Docker => discover_from_docker_swarm().await.unwrap_or_default(),
                DiscoveryMethod::NetworkScan => {
                    self.discover_via_network_scan().await.unwrap_or_default()
                }
            };

            let current_len = endpoints.len();
            endpoints.extend(method_endpoints);
            debug!(
                "{:?} discovery found {} endpoints",
                method,
                endpoints.len() - current_len
            );
        }

        // Remove duplicates and validate
        endpoints.sort();
        endpoints.dedup();

        let mut validated_endpoints = Vec::new();
        for endpoint in endpoints {
            if verify_federation_endpoint(&endpoint).await? {
                validated_endpoints.push(endpoint);
            }
        }

        Ok(validated_endpoints)
    }

    /// Get discovery status for all methods
    pub async fn get_discovery_status(&self) -> DiscoveryStatus {
        DiscoveryStatus {
            mdns_available: discover_via_mdns().await.is_ok(),
            udp_broadcast_available: discover_via_udp_broadcast().await.is_ok(),
            service_registry_available: discover_via_service_registry().await.is_ok(),
            dht_available: discover_via_dht(&self.config).await.is_ok(),
            kubernetes_available: super::kubernetes::is_running_in_kubernetes()
                && discover_from_kubernetes().await.is_ok(),
            docker_available: super::docker::is_running_in_docker()
                && discover_from_docker_swarm().await.is_ok(),
            network_scan_available: true, // Network scan is always available as fallback
        }
    }

    /// Perform targeted discovery for specific service types
    pub async fn discover_service_types(
        &self,
        service_types: &[&str],
    ) -> Result<Vec<String>, SongbirdError> {
        super::dht::targeted_service_scan(service_types).await
    }
}

/// Available discovery methods
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryMethod {
    MDNS,
    UDPBroadcast,
    ServiceRegistry,
    DHT,
    Kubernetes,
    Docker,
    NetworkScan,
}

/// Discovery status for all methods
#[derive(Debug, Default)]
pub struct DiscoveryStatus {
    pub mdns_available: bool,
    pub udp_broadcast_available: bool,
    pub service_registry_available: bool,
    pub dht_available: bool,
    pub kubernetes_available: bool,
    pub docker_available: bool,
    pub network_scan_available: bool,
}

impl DiscoveryStatus {
    /// Get list of available methods
    pub fn available_methods(&self) -> Vec<DiscoveryMethod> {
        let mut methods = Vec::new();

        if self.mdns_available {
            methods.push(DiscoveryMethod::MDNS);
        }
        if self.udp_broadcast_available {
            methods.push(DiscoveryMethod::UDPBroadcast);
        }
        if self.service_registry_available {
            methods.push(DiscoveryMethod::ServiceRegistry);
        }
        if self.dht_available {
            methods.push(DiscoveryMethod::DHT);
        }
        if self.kubernetes_available {
            methods.push(DiscoveryMethod::Kubernetes);
        }
        if self.docker_available {
            methods.push(DiscoveryMethod::Docker);
        }
        if self.network_scan_available {
            methods.push(DiscoveryMethod::NetworkScan);
        }

        methods
    }

    /// Get number of available methods
    pub fn available_count(&self) -> usize {
        self.available_methods().len()
    }

    /// Check if any discovery methods are available
    pub fn has_available_methods(&self) -> bool {
        self.available_count() > 0
    }
}

impl Default for DiscoveryManager {
    fn default() -> Self {
        Self::new(FederationConfig::default())
    }
}
