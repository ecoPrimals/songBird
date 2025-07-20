//! Static service discovery for development and testing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::super::core::{ServiceDiscovery, ServiceInstance};
use songbird_errors::Result;

/// Static service discovery for development and testing
pub struct StaticServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceInstance>>>,
}

impl StaticServiceDiscovery {
    /// Create new static service discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Pre-populate with services
    pub async fn with_services(services: Vec<ServiceInstance>) -> Self {
        let discovery = Self::new();
        let mut service_map = discovery.services.write().await;
        for service in services {
            service_map.insert(service.id.clone(), service);
        }
        drop(service_map);
        discovery
    }

    /// Get all registered services
    pub async fn get_all_services(&self) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        services.values().cloned().collect()
    }

    /// Get service count
    pub async fn service_count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }

    /// Check if service exists
    pub async fn has_service(&self, service_id: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(service_id)
    }

    /// Clear all services
    pub async fn clear(&self) {
        let mut services = self.services.write().await;
        services.clear();
    }
}

impl Default for StaticServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceDiscovery for StaticServiceDiscovery {
    async fn register_service(&self, service: ServiceInstance) -> Result<()> {
        tracing::info!("Registering service: {} ({})", service.name, service.id);

        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service);

        Ok(())
    }

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        tracing::info!("Deregistering service: {}", service_id);

        let mut services = self.services.write().await;
        services.remove(service_id);

        Ok(())
    }

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        let services = self.services.read().await;

        let filtered_services: Vec<ServiceInstance> = services
            .values()
            .filter(|service| match service_name {
                Some(name) => service.name == name,
                None => true,
            })
            .filter(|service| service.is_healthy()) // Only return healthy services
            .cloned()
            .collect();

        tracing::debug!(
            "Discovered {} services{}",
            filtered_services.len(),
            match service_name {
                Some(name) => format!(" for '{name}'"),
                None => String::new(),
            }
        );

        Ok(filtered_services)
    }

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        let services = self.services.read().await;

        match services.get(service_id) {
            Some(service) => {
                let is_healthy = service.is_healthy();
                tracing::debug!("Health check for {}: {}", service_id, is_healthy);
                Ok(is_healthy)
            }
            None => {
                tracing::warn!("Service {} not found for health check", service_id);
                Ok(false)
            }
        }
    }
}
