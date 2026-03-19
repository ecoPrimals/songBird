//! Static service discovery for development and testing
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(async_fn_in_trait)]
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::traits::discovery::{ServiceDiscovery, ServiceEvent, ServiceHealthStatus, ServiceQuery};
use crate::traits::service::ServiceInfo;
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Static service discovery for development and testing
pub struct StaticServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
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
    pub async fn with_services(services: Vec<ServiceInfo>) -> Self {
        let discovery = Self::new();
        let mut service_map = discovery.services.write().await;
        for service in services {
            service_map.insert(service.service_id.clone(), service);
        }
        drop(service_map);
        discovery
    }

    /// Get all registered services
    pub async fn get_all_services(&self) -> Vec<ServiceInfo> {
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

// Native async trait implementation (no boxing overhead)
impl ServiceDiscovery for StaticServiceDiscovery {
    async fn register(&self, service: ServiceInfo) -> Result<()> {
        tracing::info!("Registering service: {} ({})", service.name, service.service_id);

        self.services.write().await.insert(service.service_id.clone(), service);

        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!("Deregistering service: {}", service_id);

        self.services.write().await.remove(service_id);

        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let filtered_services: Vec<ServiceInfo> = self
            .services
            .read()
            .await
            .values()
            .filter(|service| query.name.as_ref().map_or(true, |name| service.name == *name))
            // All static services are considered healthy
            .cloned()
            .collect();

        tracing::debug!(
            "Discovered {} services{}",
            filtered_services.len(),
            query.name.as_ref().map_or_else(String::new, |name| format!(" for {name}"))
        );

        Ok(filtered_services)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = ServiceEvent> + Send>>> {
        use futures_util::stream;
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(&self, service_id: &str, health: ServiceHealthStatus) -> Result<()> {
        tracing::info!("Updating health for service {} to {:?}", service_id, health);
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        self.discover(ServiceQuery::new()).await
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn is_registered(&self, service_id: &str) -> Result<bool> {
        self.exists(service_id).await
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if let Some(service) = self.services.write().await.get_mut(service_id) {
            service
                .metadata
                .extend(metadata.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))));
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[path = "static_discovery_tests.rs"]
mod tests;
