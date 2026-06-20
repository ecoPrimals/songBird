// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Static Provider Adapter
//!
//! Provides static service discovery using the universal provider pattern
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for optimal performance

use std::any::Any;
use std::collections::HashMap;

use super::{DiscoveryProviderImpl, ProviderFactory};
use crate::abstraction::{
    capabilities::DiscoveryCapability,
    providers::{
        DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderMetadata, ServiceMetrics,
    },
};
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use futures_util::Stream;
use futures_util::stream;
use songbird_types::SongbirdResult;

type Result<T> = SongbirdResult<T>;
use std::pin::Pin;

/// Factory for creating Static providers from configuration
#[derive(Debug, Clone, Copy)]
pub struct StaticProviderFactory;

impl ProviderFactory for StaticProviderFactory {
    fn provider_type(&self) -> &'static str {
        "static"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<DiscoveryProviderImpl> {
        // Extract predefined services from configuration
        let services = config
            .parameters
            .get("services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|service_value| {
                        // Parse each service from JSON
                        serde_json::from_value::<ServiceInfo>(service_value.clone()).ok()
                    })
                    .collect::<Vec<ServiceInfo>>()
            })
            .unwrap_or_default();

        // Create native static adapter (no longer using deprecated backend)
        let adapter = StaticProviderAdapter::new_native(config.id, services);
        Ok(DiscoveryProviderImpl::Static(adapter))
    }

    fn validate_config(&self, _config: &ProviderConfig) -> Result<()> {
        // Static provider doesn't require any specific configuration
        // Services can be provided optionally
        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();

        // Example predefined services with configurable defaults
        let example_host = songbird_process_env::var("EXAMPLE_SERVICE_HOST")
            .unwrap_or_else(|_| songbird_config::defaults::default_host());
        let example_port = songbird_process_env::var("EXAMPLE_SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(songbird_types::defaults::ports::DEFAULT_HTTP_PORT);

        let example_services = serde_json::json!([
            {
                "service_id": "example-api",
                "name": "Example API",
                "version": "1.0.0",
                "service_type": "api",
                "description": "Example API service",
                "endpoints": [],
                "health_check_endpoint": format!("http://{}:{}/health", example_host, example_port),
                "metadata": {},
                "tags": ["api", "example"],
                "dependencies": [],
                "status": "Running",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "instance_id": "example-api-1",
                "host": example_host,
                "port": example_port
            }
        ]);

        parameters.insert("services".to_string(), example_services);

        ProviderConfig {
            id,
            name,
            parameters,
            environment: HashMap::new(),
            timeout_ms: Some(1000), // Fast for static
            retry_config: None,
        }
    }
}

/// Native static provider adapter (no longer wrapping deprecated backend)
pub struct StaticProviderAdapter {
    metadata: ProviderMetadata,
    services: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, ServiceInfo>>>,
}

impl StaticProviderAdapter {
    /// Create new native static adapter
    #[must_use]
    pub fn new_native(id: String, initial_services: Vec<ServiceInfo>) -> Self {
        let metadata = ProviderMetadata {
            id: id.clone(),
            name: format!("Static Provider ({id})"),
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration,
                DiscoveryCapability::ServiceUnregistration,
                DiscoveryCapability::ServiceDiscovery,
                DiscoveryCapability::ServiceListing,
                DiscoveryCapability::ServiceExistence,
                DiscoveryCapability::MetadataUpdating,
                // Note: Static doesn't support watching or health checking
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "static".to_string());
                meta.insert("protocol".to_string(), "memory".to_string());
                meta.insert(
                    "vendor".to_string(),
                    String::from(songbird_types::primal_names::SELF_NAME),
                );
                meta.insert("persistence".to_string(), "memory".to_string());
                meta
            },
            healthy: true,
            load_score: 0.1, // Very low load for in-memory
        };

        // Initialize services map with provided services
        let services_map: std::collections::HashMap<String, ServiceInfo> = initial_services
            .into_iter()
            .map(|service| (service.service_id.clone(), service))
            .collect();

        Self {
            metadata,
            services: std::sync::Arc::new(tokio::sync::RwLock::new(services_map)),
        }
    }
}

impl DiscoveryProvider for StaticProviderAdapter {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        tracing::info!("🗂️ Initializing Static discovery provider adapter");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🗂️ Shutting down Static discovery provider adapter");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Static provider is always healthy (in-memory)
        Ok(true)
    }

    async fn register(&self, service: ServiceInfo) -> Result<()> {
        tracing::info!("📝 Registering service {} via Static adapter", service.service_id);
        let mut services = self.services.write().await;
        services.insert(service.service_id.clone(), service);
        Ok(())
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        tracing::info!("🗑️ Unregistering service {} via Static adapter", service_id);
        let mut services = self.services.write().await;
        services.remove(service_id);
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        tracing::debug!("🔍 Discovering services via Static adapter");
        let services = self.services.read().await;
        let mut results = Vec::new();

        for service in services.values() {
            // Apply query filters
            if let Some(name) = &query.name {
                if service.name != *name {
                    continue;
                }
            }
            if let Some(service_type) = &query.service_type {
                if service.service_type != *service_type {
                    continue;
                }
            }
            results.push(service.clone());
        }

        Ok(results)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        tracing::debug!("👀 Watching services via Static adapter (not implemented)");
        // Static adapter doesn't support watching - return empty stream
        Ok(Box::pin(stream::empty()))
    }

    async fn update_health(&self, service_id: &str, _health: ServiceHealthStatus) -> Result<()> {
        tracing::debug!("💊 Updating health for service {} via Static adapter", service_id);
        // For static adapter, we don't need to update health status
        // The service is either registered or not
        Ok(())
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        tracing::debug!("🏷️ Updating metadata for service {} via Static adapter", service_id);
        // Update metadata for the service
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            for (key, value) in metadata {
                service.metadata.insert(key, serde_json::Value::String(value));
            }
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        tracing::debug!("📋 Listing all services via Static adapter");
        let services = self.services.read().await;
        Ok(services.values().cloned().collect())
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        tracing::debug!("❓ Checking if service {} exists via Static adapter", service_id);
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }

    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        tracing::debug!("📊 Getting metrics for service {} via Static adapter", service_id);

        // Static provider doesn't have real metrics, return defaults
        Ok(ServiceMetrics {
            service_id: service_id.to_string(),
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 1.0, // Very fast for static
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 1024, // Minimal memory
            custom_metrics: HashMap::new(),
        })
    }

    async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints> {
        tracing::debug!("⚖️ Getting load balancing hints for {} via Static adapter", service_name);

        // For static services, all instances have equal weight
        let services_map = self.services.read().await;
        let services: Vec<ServiceInfo> =
            services_map.values().filter(|service| service.name == service_name).cloned().collect();

        let mut weights = HashMap::new();
        let mut health_scores = HashMap::new();
        let mut preferred_instances = Vec::new();

        for service in services {
            weights.insert(service.service_id.clone(), 1.0); // Equal weight
            health_scores.insert(service.service_id.clone(), 1.0); // Assume healthy
            preferred_instances.push(service.service_id);
        }

        Ok(LoadBalancingHints {
            service_name: service_name.to_string(),
            preferred_instances,
            weights,
            health_scores,
            locality_preferences: vec!["local".to_string()], // Static is always local
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abstraction::providers::DiscoveryProvider;
    use crate::traits::ServiceInfo;
    use crate::traits::service::ServiceStatus;
    use songbird_config;
    use songbird_types::unified_constants::*;

    #[tokio::test]
    async fn test_static_factory_creation() {
        let factory = StaticProviderFactory;
        let config = factory.default_config("test".to_string(), "Test".to_string());

        assert!(factory.validate_config(&config).is_ok());

        let provider = factory.create_provider(config).await.expect("create static provider");
        assert_eq!(provider.metadata().id, "test");
    }

    #[tokio::test]
    async fn test_static_provider_operations() {
        let initial_services = vec![];
        let adapter =
            StaticProviderAdapter::new_native("test-static".to_string(), initial_services);

        assert_eq!(adapter.metadata().id, "test-static");
        assert!(
            adapter.metadata().capabilities.contains(&DiscoveryCapability::ServiceRegistration)
        );
        assert!(adapter.health_check().await.expect("health check"));
    }

    fn sample_service(id: &str, name: &str, service_type: &str, port: u16) -> ServiceInfo {
        use chrono::Utc;
        use std::collections::HashMap;

        ServiceInfo {
            service_id: id.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            service_type: service_type.to_string(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: Some(format!("http://localhost:{port}/health")),
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            instance_id: format!("{id}-instance"),
            host: "localhost".to_string(),
            port,
        }
    }

    #[tokio::test]
    async fn test_register_unregister_and_exists_lifecycle() {
        let adapter = StaticProviderAdapter::new_native("lifecycle".to_string(), vec![]);
        let service = sample_service("svc-1", "API", "api", 8080);

        adapter.register(service.clone()).await.unwrap();
        assert!(adapter.exists("svc-1").await.unwrap());

        adapter.unregister("svc-1").await.unwrap();
        assert!(!adapter.exists("svc-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_discover_filter_by_name_and_type() {
        use crate::traits::ServiceQuery;

        let adapter = StaticProviderAdapter::new_native(
            "filters".to_string(),
            vec![
                sample_service("api-1", "User API", "api", 8080),
                sample_service("db-1", "Database", "storage", 5432),
            ],
        );

        let mut by_name = ServiceQuery::new();
        by_name.name = Some("User API".to_string());
        let name_matches = adapter.discover(by_name).await.unwrap();
        assert_eq!(name_matches.len(), 1);
        assert_eq!(name_matches[0].service_id, "api-1");

        let mut by_type = ServiceQuery::new();
        by_type.service_type = Some("storage".to_string());
        let type_matches = adapter.discover(by_type).await.unwrap();
        assert_eq!(type_matches.len(), 1);
        assert_eq!(type_matches[0].service_id, "db-1");
    }

    #[tokio::test]
    async fn test_discover_no_match_returns_empty() {
        use crate::traits::ServiceQuery;

        let adapter = StaticProviderAdapter::new_native(
            "empty".to_string(),
            vec![sample_service("svc-1", "API", "api", 8080)],
        );

        let mut query = ServiceQuery::new();
        query.name = Some("Nonexistent".to_string());
        assert!(adapter.discover(query).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_all_and_metadata_update() {
        use std::collections::HashMap;

        let adapter = StaticProviderAdapter::new_native(
            "list".to_string(),
            vec![sample_service("svc-1", "API", "api", 8080)],
        );

        let all = adapter.list_all().await.unwrap();
        assert_eq!(all.len(), 1);

        let mut meta = HashMap::new();
        meta.insert("env".to_string(), "test".to_string());
        adapter.update_metadata("svc-1", meta).await.unwrap();

        let updated = adapter.list_all().await.unwrap();
        assert_eq!(
            updated[0].metadata.get("env"),
            Some(&serde_json::Value::String("test".to_string()))
        );
    }

    #[tokio::test]
    async fn test_health_status_and_metrics_reporting() {
        use crate::traits::discovery::ServiceHealthStatus;

        let adapter = StaticProviderAdapter::new_native(
            "health".to_string(),
            vec![sample_service("svc-1", "API", "api", 8080)],
        );

        assert!(adapter.health_check().await.unwrap());
        adapter.update_health("svc-1", ServiceHealthStatus::Healthy).await.unwrap();

        let metrics = adapter.get_service_metrics("svc-1").await.unwrap();
        assert_eq!(metrics.service_id, "svc-1");
        assert!(metrics.average_response_time_ms > 0.0);
    }

    #[tokio::test]
    async fn test_load_balancing_hints_for_service_name() {
        let adapter = StaticProviderAdapter::new_native(
            "lb".to_string(),
            vec![
                sample_service("api-1", "User API", "api", 8080),
                sample_service("api-2", "User API", "api", 8081),
            ],
        );

        let hints = adapter.get_load_balancing_hints("User API").await.unwrap();
        assert_eq!(hints.preferred_instances.len(), 2);
        assert_eq!(hints.weights.get("api-1"), Some(&1.0));
    }

    #[tokio::test]
    async fn test_initialize_and_shutdown_lifecycle() {
        let mut adapter = StaticProviderAdapter::new_native("lifecycle-ops".to_string(), vec![]);
        let config = ProviderConfig {
            id: "lifecycle-ops".to_string(),
            name: "Lifecycle".to_string(),
            parameters: HashMap::new(),
            environment: HashMap::new(),
            timeout_ms: Some(1000),
            retry_config: None,
        };

        adapter.initialize(config).await.unwrap();
        adapter.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_factory_creates_provider_with_configured_services() {
        let factory = StaticProviderFactory;
        let mut config = factory.default_config("configured".to_string(), "Configured".to_string());
        config.parameters.insert(
            "services".to_string(),
            serde_json::json!([{
                "service_id": "custom-svc",
                "name": "Custom",
                "version": "2.0.0",
                "service_type": "worker",
                "endpoints": [],
                "metadata": {},
                "tags": [],
                "dependencies": [],
                "status": "Running",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z",
                "instance_id": "custom-svc-1",
                "host": "localhost",
                "port": 9000
            }]),
        );

        let provider = factory.create_provider(config).await.unwrap();
        match provider {
            DiscoveryProviderImpl::Static(adapter) => {
                assert!(adapter.exists("custom-svc").await.unwrap());
            }
            _ => panic!("Expected static provider"),
        }
    }
}
