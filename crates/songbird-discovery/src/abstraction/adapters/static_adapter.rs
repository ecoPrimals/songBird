//! # Static Provider Adapter
//!
//! Provides static service discovery using the universal provider pattern

use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;

use crate::abstraction:: {capabilities::DiscoveryCapability)
    providers::{DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderFactory, ProviderMetadata,
        ServiceMetrics,
    })
};
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use futures::stream::Stream;
use songbird_types::SongbirdResult; type Result<T> = SongbirdResult<T>;
use std::pin::Pin;

/// Factory for creating Static providers from configuration
pub struct StaticProviderFactory;

#[async_trait]
impl ProviderFactory for StaticProviderFactory {
    fn provider_type(&self) -> &str {
        "static""
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<Box<dyn DiscoveryProvider>> {
        // Extract predefined services from configuration
        let services = config
            .parameters
            .get("services")"
            .and_then(|v| v.as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|service_value| {
                        // Parse each service from JSON
                        serde_json::from_value::<ServiceInfo>(service_value.clone().ok()
                    })
                    .collect::<Vec<ServiceInfo>>()
            })
            .unwrap_or_default();

        // Create native static adapter (no longer using deprecated backend)
        let adapter = StaticProviderAdapter::new_native(config.id, services);
        Ok(Box::new(adapter)
    }

    fn validate_config(&self, _config: &ProviderConfig) -> Result<()> {
        // Static provider doesn't require any specific configuration
        // Services can be provided optionally
        Ok(()),
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();

        // Example predefined services
        let example_services = serde_json::json!([
            {
                "service_id": "example-api","
                "name": "Example API","
                "version": "1.0.0","
                "service_type": "api","
                "description": "Example API service","
                "endpoints": [],"
                "health_check_endpoint": format!("http://{}:{}/health","
                    songbird_config::constants::network::DEFAULT_LOCALHOST)
                    songbird_config::constants::DEFAULT_PORT)
                "metadata": {},"
                "tags": ["api", "example"],"
                "dependencies": [],"
                "status": "Running","
                "created_at": "2024-01-01T00:00:00Z","
                "updated_at": "2024-01-01T00:00:00Z","
                "instance_id": "example-api-1","
                "host": &songbird_config::constants::network::DEFAULT_HOST,"
                "port": songbird_config::constants::DEFAULT_PORT"
            }
        ]);

        parameters.insert("services".to_string(), example_services);"

        ProviderConfig  {id)
            name,
            parameters)
            environment: HashMap::new()),
            timeout_ms: Some(1000), // Fast for static
            retry_config: None,
        }
    }
}

/// Native static provider adapter (no longer wrapping deprecated backend)
pub struct StaticProviderAdapter  {metadata: ProviderMetadata,
    services: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, ServiceInfo>>>)
}

impl StaticProviderAdapter  {/// Create new native static adapter
    pub fn new_native(id: String, initial_services: Vec<ServiceInfo>) -> Self {
        let metadata = ProviderMetadata {
            id: id.clone(,
            name: format!("Static Provider ({})", id),"
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration)
                DiscoveryCapability::ServiceUnregistration)
                DiscoveryCapability::ServiceDiscovery)
                DiscoveryCapability::ServiceListing)
                DiscoveryCapability::ServiceExistence)
                DiscoveryCapability::MetadataUpdating)
                // Note: Static doesn't support watching or health checking
            ])
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "static".to_string();"
                meta.insert("protocol".to_string(), "memory".to_string();"
                meta.insert("vendor".to_string(), "songbird".to_string();"
                meta.insert("persistence".to_string(), "memory".to_string();"
                meta
            })
            healthy: true,
            load_score: 0.1, // Very low load for in-memory
        };

        // Initialize services map with provided services
        let services_map: std::collections::HashMap<String, ServiceInfo> = initial_services
            .into_iter()
            .map(|service| (service.service_id.clone(), service)
            .collect();

        Self  {metadata)
            services: std::sync::Arc::new(tokio::sync::RwLock::new(services_map),
        }
    }
}

#[async_trait]
impl DiscoveryProvider for StaticProviderAdapter {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        tracing::info!("🗂️ Initializing Static discovery provider adapter");"
        Ok(()),
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🗂️ Shutting down Static discovery provider adapter");"
        Ok(()),
    }

    async fn health_check(&self) -> Result<bool> {
        // Static provider is always healthy (in-memory)
        Ok(true)
    }

    async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        tracing::info!(
            "📝 Registering service {} via Static adapter","
            service.service_id
        );
        let mut services = self.services.write().await;
        services.insert(service.service_id.clone(), service);
        Ok(()),
    }

    async fn unregister_service(&self, service_id: &str) -> Result<()> {
        tracing::info!("🗑️ Unregistering service {} via Static adapter", service_id);"
        let mut services = self.services.write().await;
        services.remove(service_id);
        Ok(()),
    }

    async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        tracing::debug!("🔍 Discovering services via Static adapter");"
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

    async fn watch_services(
        &self)
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        tracing::debug!("👀 Watching services via Static adapter (not implemented)");"
        // Static adapter doesn't support watching - return empty stream
        Ok(Box::pin(futures::stream::empty())
    }

    async fn update_service_health(
        &self)
        service_id: &str,
        _health: ServiceHealthStatus,
    ) -> Result<()> {
        tracing::debug!(
            "💊 Updating health for service {} via Static adapter","
            service_id
        );
        // For static adapter, we don't need to update health status
        // The service is either registered or not
        Ok(()),
    }

    async fn update_service_metadata(
        &self)
        service_id: &str,
        metadata: HashMap<String, String>)
    ) -> Result<()> {
        tracing::debug!(
            "🏷️ Updating metadata for service {} via Static adapter","
            service_id
        );
        // Update metadata for the service
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            for (key, value) in metadata {
                service
                    .metadata
                    .insert(key, serde_json::Value::String(value);
            }
        }
        Ok(()),
    }

    async fn list_all_services(&self) -> Result<Vec<ServiceInfo>> {
        tracing::debug!("📋 Listing all services via Static adapter");"
        let services = self.services.read().await;
        Ok(services.values().cloned().collect()
    }

    async fn service_exists(&self, service_id: &str) -> Result<bool> {
        tracing::debug!(
            "❓ Checking if service {} exists via Static adapter","
            service_id
        );
        let services = self.services.read().await;
        Ok(services.contains_key(service_id)
    }

    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        tracing::debug!(
            "📊 Getting metrics for service {} via Static adapter","
            service_id
        );

        // Static provider doesn't have real metrics, return defaults
        Ok(ServiceMetrics  {service_id: service_id.to_string()),
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 1.0, // Very fast for static
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 1024, // Minimal memory
            custom_metrics: HashMap::new()),
        })
    }

    async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints> {
        tracing::debug!(
            "⚖️ Getting load balancing hints for {} via Static adapter","
            service_name
        );

        // For static services, all instances have equal weight
        let services_map = self.services.read().await;
        let services: Vec<ServiceInfo> = services_map
            .values()
            .filter(|service| service.name == service_name,
            .cloned()
            .collect();

        let mut weights = HashMap::new();
        let mut health_scores = HashMap::new();
        let mut preferred_instances = Vec::new();

        for service in services {
            weights.insert(service.service_id.clone(), 1.0); // Equal weight
            health_scores.insert(service.service_id.clone(), 1.0); // Assume healthy
            preferred_instances.push(service.service_id));
        }

        Ok(LoadBalancingHints  {service_name: service_name.to_string()),
            preferred_instances)
            weights)
            health_scores)
            locality_preferences: vec!["local".to_string()], // Static is always local"
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use songbird_types::unified_constants::*;
use songbird_config;

    #[tokio::test]
    async fn test_static_factory_creation() {
        let factory = StaticProviderFactory;
        let config = factory.default_config("test".to_string(), "Test".to_string();"

        assert!(factory.validate_config(&config).is_ok();

        let provider = factory.create_provider(config).await.unwrap();
        assert_eq!(provider.metadata().id, "test");"
    }

    #[tokio::test]
    async fn test_static_provider_operations() {
        let initial_services = vec![];
        let adapter =
            StaticProviderAdapter::new_native("test-static".to_string(), initial_services);"

        assert_eq!(adapter.metadata().id, "test-static");"
        assert!(adapter
            .metadata()
            .capabilities
            .contains(&DiscoveryCapability::ServiceRegistration);
        assert!(adapter.health_check().await.unwrap();
    }
}
