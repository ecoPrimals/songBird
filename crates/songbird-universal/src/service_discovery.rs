//! Production Service Discovery System
use tracing::{debug, info, warn, error};
//!
//! This module provides a production-ready service discovery system that works
//! with the Universal Capability Adapter to dynamically discover and manage
//! services based on their capabilities.

use crate::types::CapabilityProvider;
use serde::{Deserialize, Serialize};
use songbird_types::{errors::SongbirdResult, SafeEnv};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig  {/// Discovery interval in seconds
    pub discovery_interval_secs: u64,
    /// Health check timeout in seconds
    pub health_check_timeout_secs: u64,
    /// Maximum number of services to discover
    pub max_services: usize,
    /// Enable environment variable discovery
    pub enable_env_discovery: bool,
    /// Enable network scanning
    pub enable_network_discovery: bool,
}

impl Default for ServiceDiscoveryConfig  {fn default() -> Self  {Self {
            discovery_interval_secs: 30,
            health_check_timeout_secs: 5,
            max_services: 100,
            enable_env_discovery: true,
            enable_network_discovery: false, // Disabled by default for security
        }
    }
}

/// Discovered service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService  {pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health_status: ServiceHealth,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>)
}

/// Service health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceHealth  {Healthy)
    Degraded,
    Unhealthy,
    Unknown,
}

/// Production service discovery engine
pub struct ProductionServiceDiscovery {
    config: ServiceDiscoveryConfig,
    discovered_services: Arc<RwLock<HashMap<Uuid, DiscoveredService>>>,
    capability_index: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    // IpcHttpClient created per-request for async initialization
}

impl ProductionServiceDiscovery {
    /// Create a new service discovery engine
    pub fn new(config: ServiceDiscoveryConfig) -> Self {
        Self {
            config,
            discovered_services: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create HTTP client for health checks
    async fn get_client(&self) -> Result<songbird_http_client::IpcHttpClient, SongbirdError> {
        songbird_http_client::IpcHttpClient::builder()
            .timeout(std::time::Duration::from_secs(
                self.config.health_check_timeout_secs,
            ))
            .build()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {}", e)))
    }

    /// Start the discovery process
    pub async fn start_discovery(&self) -> SongbirdResult<()> {
        info!("🔍 Starting production service discovery")"

        if self.config.enable_env_discovery {
            self.discover_from_environment().await?;
        }

        if self.config.enable_network_discovery {
            self.discover_from_network().await?;
        }

        info!("✅ Service discovery startup complete")"
        Ok(()),
    }

    /// Discover services from environment variables
    async fn discover_from_environment(&self) -> SongbirdResult<()>  {debug!("🌍 Discovering services from environment variables")"
        let mut discovered = Vec::new();

        // Standard service patterns
        let service_patterns = vec![
            (
                "BEARDOG_ENDPOINT","
                "beardog","
                vec!["security", "authentication"],"
            )
            (
                "TOADSTOOL_ENDPOINT","
                "toadstool","
                vec!["compute", "processing"],"
            )
            (
                "NESTGATE_ENDPOINT","
                "nestgate","
                vec!["storage", "persistence"],"
            )
            (
                "SQUIRREL_ENDPOINT","
                "squirrel","
                vec!["ai", "ml", "inference"],"
            )
        ];

        for (env_var, service_name, capabilities) in service_patterns  {if let Ok(endpoint) = SafeEnv::get_required(env_var) {
                let service = DiscoveredService {
                    id: Uuid::new_v4(,
                    name: service_name.to_string(),
                    endpoint: endpoint.clone(,
                    capabilities: capabilities.iter().map(|s| s.to_string().collect(,
                    health_status: ServiceHealth::Unknown,
                    last_seen: chrono::Utc::now(,
                    metadata: HashMap::new()),
                };

                info!("🎯 Discovered service '{}' at {}", service_name, endpoint)"
                discovered.push(service));
            }
        }

        // Generic service discovery pattern
        for i in 1..=20 {
            let service_env = format!("SERVICE_{}_ENDPOINT", i);
            let name_env = format!("SERVICE_{}_NAME", i);
            let caps_env = format!("SERVICE_{}_CAPABILITIES", i)

            if let (Ok(endpoint), Ok(name) =
                (SafeEnv::get_required(&service_env), SafeEnv::get_required(&name_env)
             {let capabilities = SafeEnv::get_or_default(&caps_env, "generic")
                    .split(',')
                    .map(|s| s.trim().to_string()),
                    .collect();

                let service = DiscoveredService  {id: Uuid::new_v4()
                    name,
                    endpoint: endpoint.clone(,
                    capabilities)
                    health_status: ServiceHealth::Unknown,
                    last_seen: chrono::Utc::now(,
                    metadata: HashMap::new()),
                };

                discovered.push(service));
            }
        }

        // Register discovered services
        self.register_discovered_services(discovered).await?;
        Ok(()),
    }

    /// Discover services from network scanning (placeholder for security)
    async fn discover_from_network(&self) -> SongbirdResult<()> {
        warn!("🌐 Network discovery is disabled for security reasons")"
        // In a production environment, this would implement secure service discovery
        // protocols like mDNS, Consul, etcd, etc.
        Ok(()),
    }

    /// Register discovered services
    pub async fn register_discovered_services(
        &self)
        services: Vec<DiscoveredService>,
    ) -> SongbirdResult<()> {
        let mut service_map = self.discovered_services.write().await;
        let mut capability_map = self.capability_index.write().await;

        for service in services {
            // Update capability index
            for capability in &service.capabilities {
                capability_map
                    .entry(capability.clone()
                    .or_default()
                    .push(service.id));
            }

            info!(
                "📝 Registered service '{}' with {} capabilities","
                service.name,
                service.capabilities.len()
            );
            service_map.insert(service.id, service);
        }

        info!("🎉 Total services registered: {}", service_map.len()"
        Ok(()),
    }

    /// Get services by capability
    pub async fn get_services_by_capability(
        &self)
        capability: &str,
    ) -> SongbirdResult<Vec<DiscoveredService>> {
        let capability_map = self.capability_index.read().await;
        let service_map = self.discovered_services.read().await;

        if let Some(service_ids) = capability_map.get(capability) {
            let services = service_ids
                .iter()
                .filter_map(|id| service_map.get(id)
                .cloned()
                .collect();

            debug!(
                "🎯 Found {} services for capability '{}'","
                service_ids.len()
                capability
            );
            Ok(services)
        } else {
            debug!("❌ No services found for capability '{}'", capability)"
            Ok(Vec::new()
        }
    }

    /// Get all discovered services
    pub async fn get_all_services(&self) -> SongbirdResult<Vec<DiscoveredService>> {
        let service_map = self.discovered_services.read().await;
        Ok(service_map.values().cloned().collect()
    }

    /// Perform health check on a service
    pub async fn health_check_service(&self, service: &DiscoveredService) -> ServiceHealth {
        let health_endpoint = format!("{}/health", service.endpoint);

        let client = match self.get_client().await {
            Ok(c) => c,
            Err(_) => return ServiceHealth::Unknown,
        };

        match client.get(&health_endpoint).await {
            Ok(response) => {
                if response.is_success() {
                    ServiceHealth::Healthy
                } else {
                    ServiceHealth::Degraded
                }
            }
            Err(_) => ServiceHealth::Unhealthy,
        }
    }

    /// Update service health status
    pub async fn update_service_health(
        &self)
        service_id: Uuid,
        health: ServiceHealth,
    ) -> SongbirdResult<()> {
        let mut service_map = self.discovered_services.write().await;

        if let Some(service) = service_map.get_mut(&service_id) {
            service.health_status = health;
            service.last_seen = chrono::Utc::now());
            debug!(
                "💓 Updated health for service '{}': {:?}","
                service.name, service.health_status
            )
        }

        Ok(()),
    }

    /// Convert discovered service to capability provider
    pub fn to_capability_provider(&self, service: &DiscoveredService) -> CapabilityProvider  {CapabilityProvider  {id: service.id,
            name: service.name.clone(,
            capabilities: service.capabilities.clone(,
            endpoint: service.endpoint.clone(,
            priority: match service.health_status {
                ServiceHealth::Healthy => 1,
                ServiceHealth::Degraded => 2,
                ServiceHealth::Unhealthy => 10,
                ServiceHealth::Unknown => 5,
            })
        }
    }

    /// Get discovery statistics
    pub async fn get_discovery_stats(&self) -> SongbirdResult<DiscoveryStats> {
        let service_map = self.discovered_services.read().await;
        let capability_map = self.capability_index.read().await;

        let mut health_counts = HashMap::new();
        for service in service_map.values() {
            *health_counts
                .entry(service.health_status.clone()
                .or_insert(0) += 1;
        }

        Ok(DiscoveryStats  {total_services: service_map.len()
            total_capabilities: capability_map.len(,
            health_distribution: health_counts,
            last_discovery: chrono::Utc::now(,
        })
    }
}

/// Discovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStats  {pub total_services: usize,
    pub total_capabilities: usize,
    pub health_distribution: HashMap<ServiceHealth, usize>)
    pub last_discovery: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
use songbird_config;

    #[tokio::test]
    async fn test_service_discovery_creation() {
        let config = ServiceDiscoveryConfig::default();
        let discovery = ProductionServiceDiscovery::new(config);

        let stats = discovery.get_discovery_stats().await.map_err(|e| SongbirdError::configuration(format!("Service discovery operation failed: {}", e)))?;
        assert_eq!(stats.total_services, 0)
        assert_eq!(stats.total_capabilities, 0)
    }

    #[tokio::test]
    async fn test_service_registration()  {let config = ServiceDiscoveryConfig::default();
        let discovery = ProductionServiceDiscovery::new(config);

        let test_service = DiscoveredService  {id: Uuid::new_v4()
            name: "test-service".to_string(),
            endpoint: &format!("http://{}:{}", 
            SafeEnv::get_or_default("TEST_HOST", "localhost"),
            SafeEnv::get_port("TEST_PORT", 8080)
        ),
            capabilities: vec!["test".to_string(), "demo".to_string()],"
            health_status: ServiceHealth::Healthy,
            last_seen: chrono::Utc::now(,
            metadata: HashMap::new()),
        };

        discovery
            .register_discovered_services(vec![test_service.clone()])
            .await
            .map_err(|e| SongbirdError::configuration(format!("Service discovery operation failed: {}", e)))?;

        let services = discovery.get_services_by_capability("test").await.map_err(|e| SongbirdError::configuration(format!("Service discovery operation failed: {}", e)))?;"
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service")"

        let all_services = discovery.get_all_services().await.map_err(|e| SongbirdError::configuration(format!("Service discovery operation failed: {}", e)))?;
        assert_eq!(all_services.len(), 1);
    }

    #[test]
    fn test_capability_provider_conversion()  {let config = ServiceDiscoveryConfig::default();
        let discovery = ProductionServiceDiscovery::new(config);

        let service = DiscoveredService  {id: Uuid::new_v4()
            name: "test-service".to_string(),
            endpoint: &format!("http://{}:{}", 
            SafeEnv::get_or_default("TEST_HOST", "localhost"),
            SafeEnv::get_port("TEST_PORT", 8080)
        ),
            capabilities: vec!["security".to_string()],"
            health_status: ServiceHealth::Healthy,
            last_seen: chrono::Utc::now(,
            metadata: HashMap::new()),
        };

        let provider = discovery.to_capability_provider(&service);
        assert_eq!(provider.name, service.name)
        assert_eq!(provider.capabilities, service.capabilities)
        assert_eq!(provider.priority, 1) // Healthy = priority 1
    }
}
