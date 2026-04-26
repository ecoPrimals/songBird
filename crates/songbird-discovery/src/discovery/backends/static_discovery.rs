// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Static service discovery for development and testing
//!
//! ## Native Async Traits
//! This module uses native async trait methods (Rust 1.75+) for zero-cost abstractions.

#![allow(async_fn_in_trait, reason = "async fn in trait for StaticServiceDiscovery trait objects")]
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
            .filter(|service| query.name.as_ref().is_none_or(|name| service.name == *name))
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
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::StaticServiceDiscovery;
    use crate::traits::discovery::{ServiceDiscovery, ServiceHealthStatus, ServiceQuery};
    use crate::traits::service::{ServiceInfo, ServiceStatus};
    use chrono::Utc;
    use std::collections::HashMap;

    fn sample_service(service_id: &str, name: &str) -> ServiceInfo {
        ServiceInfo {
            service_id: service_id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: "api".to_string(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("{service_id}-i1"),
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }

    #[tokio::test]
    async fn new_and_default_are_empty() {
        let a = StaticServiceDiscovery::new();
        let b = StaticServiceDiscovery::default();
        assert_eq!(a.service_count().await, 0);
        assert_eq!(b.service_count().await, 0);
    }

    #[tokio::test]
    async fn with_services_populates_map() {
        let s1 = sample_service("a", "Alpha");
        let s2 = sample_service("b", "Beta");
        let d = StaticServiceDiscovery::with_services(vec![s1, s2]).await;
        assert_eq!(d.service_count().await, 2);
        assert!(d.has_service("a").await);
        let mut names: Vec<String> =
            d.get_all_services().await.into_iter().map(|s| s.name).collect();
        names.sort();
        assert_eq!(names, vec!["Alpha".to_string(), "Beta".to_string()]);
    }

    #[tokio::test]
    async fn discover_filters_by_name_when_set() {
        let d = StaticServiceDiscovery::with_services(vec![
            sample_service("x", "One"),
            sample_service("y", "Two"),
        ])
        .await;

        let mut q = ServiceQuery::new();
        q.name = Some("Two".to_string());
        let found = ServiceDiscovery::discover(&d, q).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].service_id, "y");

        let all = ServiceDiscovery::discover(&d, ServiceQuery::new()).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn register_unregister_and_clear() {
        let d = StaticServiceDiscovery::new();
        ServiceDiscovery::register(&d, sample_service("z", "Zed")).await.unwrap();
        assert!(ServiceDiscovery::exists(&d, "z").await.unwrap());
        ServiceDiscovery::unregister(&d, "z").await.unwrap();
        assert!(!ServiceDiscovery::exists(&d, "z").await.unwrap());
        ServiceDiscovery::register(&d, sample_service("z", "Zed")).await.unwrap();
        d.clear().await;
        assert_eq!(d.service_count().await, 0);
    }

    #[tokio::test]
    async fn update_health_and_metadata_are_ok() {
        let d = StaticServiceDiscovery::with_services(vec![sample_service("m", "Meta")]).await;
        assert!(
            ServiceDiscovery::update_health(&d, "m", ServiceHealthStatus::Healthy).await.is_ok()
        );
        let mut meta = HashMap::new();
        meta.insert("k".to_string(), "v".to_string());
        assert!(ServiceDiscovery::update_metadata(&d, "m", meta).await.is_ok());
    }

    #[tokio::test]
    async fn list_all_matches_discover_empty_query() {
        let d = StaticServiceDiscovery::with_services(vec![sample_service("p", "Ping")]).await;
        let a = ServiceDiscovery::list_all(&d).await.unwrap();
        let b = ServiceDiscovery::discover(&d, ServiceQuery::new()).await.unwrap();
        assert_eq!(a.len(), b.len());
    }
}
