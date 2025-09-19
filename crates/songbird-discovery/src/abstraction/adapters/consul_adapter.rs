//! # Consul Provider Adapter
//!
//! Provides Consul service discovery using the universal provider pattern

use async_trait::async_trait;
use futures::stream::{self, Stream};
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use crate::abstraction::{
    capabilities::DiscoveryCapability,
    providers::{
        DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderFactory, ProviderMetadata,
        ServiceMetrics,
    },
};

use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_errors::{SongbirdError};

/// Factory for creating Consul providers from configuration
pub struct ConsulProviderFactory;

#[async_trait]
impl ProviderFactory for ConsulProviderFactory {
    fn provider_type(&self) -> &str {
        "consul"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<Box<dyn DiscoveryProvider>> {
        // Extract Consul URL from flexible configuration
        let consul_url = config
            .parameters
            .get("url")
            .or_else(|| config.parameters.get("consul_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SongbirdError::configuration_error(
                    "Consul URL required in parameters.url or parameters.consul_url",
                )
            })?;

        // Create native consul adapter (no longer using deprecated backend)
        let adapter = ConsulProviderAdapter::new_native(config.id, consul_url.to_string());
        Ok(Box::new(adapter))
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<()> {
        // Validate required parameters
        if !config.parameters.contains_key("url") && !config.parameters.contains_key("consul_url") {
            return Err(SongbirdError::internal_error(configuration_error(
                "Consul provider requires 'url' or 'consul_url' parameter",
            ));
        }

        // Validate URL format if provided
        if let Some(url) = config.parameters.get("url").and_then(|v| v.as_str()) {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(SongbirdError::internal_error(configuration_error(
                    "Consul URL must start with http:// or https://",
                ));
            }
        }

        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        let mut parameters = HashMap::new();
        parameters.insert(
            "url".to_string(),
            serde_json::Value::String("http://localhost:8500".to_string()),
        );
        parameters.insert(
            "datacenter".to_string(),
            serde_json::Value::String("dc1".to_string()),
        );
        parameters.insert(
            "token".to_string(),
            serde_json::Value::String("${CONSUL_TOKEN}".to_string()),
        );

        let mut environment = HashMap::new();
        environment.insert(
            "CONSUL_HTTP_ADDR".to_string(),
            "http://localhost:8500".to_string(),
        );

        ProviderConfig {
            id,
            name,
            parameters,
            environment,
            timeout_ms: Some(10000),
            retry_config: None,
        }
    }
}

/// Native Consul provider adapter (no longer wrapping deprecated backend)
pub struct ConsulProviderAdapter {
    metadata: ProviderMetadata,
    consul_url: String,
    client: reqwest::Client,
}

impl ConsulProviderAdapter {
    /// Create new native consul adapter
    pub fn new_native(id: String, consul_url: String) -> Self {
        let metadata = ProviderMetadata {
            id: id.clone(),
            name: format!("Consul Provider ({id})"),
            version: "1.0.0".to_string(),
            capabilities: vec![
                DiscoveryCapability::ServiceRegistration,
                DiscoveryCapability::ServiceUnregistration,
                DiscoveryCapability::ServiceDiscovery,
                DiscoveryCapability::HealthChecking,
                DiscoveryCapability::ServiceListing,
                DiscoveryCapability::ServiceExistence,
                DiscoveryCapability::ServiceMetrics,
                DiscoveryCapability::LoadBalancingHints,
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), "consul".to_string());
                meta.insert("protocol".to_string(), "http".to_string());
                meta.insert("vendor".to_string(), "hashicorp".to_string());
                meta
            },
            healthy: true,
            load_score: 0.5, // Medium load score
        };

        Self {
            metadata,
            consul_url,
            client: reqwest::Client::new(),
        }
    }

    /// Convert ServiceInfo to ServiceInstance for legacy backend
    fn to_service_instance(
        &self,
        service: &ServiceInfo,
    ) -> crate::discovery::core::ServiceInstance {
        use std::net::{IpAddr, SocketAddr};

        let ip: IpAddr = service.host.parse().unwrap_or_else(|_| {
            "127.0.0.1"
                .parse()
                .expect("127.0.0.1 is a valid IP address")
        });
        let address = SocketAddr::new(ip, service.port);

        crate::discovery::core::ServiceInstance {
            id: service.service_id.clone(),
            name: service.name.clone(),
            address,
            metadata: service
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
            health_check_url: service.health_check_endpoint.clone(),
            tags: service.tags.clone(),
        }
    }

    /// Parse Consul API response into ServiceInfo list
    fn parse_consul_response(&self, response: &serde_json::Value) -> Result<Vec<ServiceInfo>> {
        let mut services = Vec::new();

        if let Some(service_list) = response.as_array() {
            // Health service response format
            for service_entry in service_list {
                if let Some(service_info) = self.parse_consul_service(service_entry) {
                    services.push(service_info);
                }
            }
        } else if let Some(service_map) = response.as_object() {
            // Agent services response format
            for (_, service_data) in service_map {
                if let Some(service_info) = self.parse_consul_service(service_data) {
                    services.push(service_info);
                }
            }
        }

        Ok(services)
    }

    /// Parse individual Consul service into ServiceInfo
    fn parse_consul_service(&self, service: &serde_json::Value) -> Option<ServiceInfo> {
        let id = service["ID"].as_str()?.to_string();
        let name = service["Service"].as_str().unwrap_or(&id).to_string();
        let address = service["Address"].as_str().unwrap_or("localhost");
        let port = service["Port"]
            .as_u64()
            .unwrap_or(songbird_config::constants::DEFAULT_PORT as u64) as u16;

        Some(ServiceInfo {
            service_id: id.clone(),
            name: name.clone(),
            version: "1.0.0".to_string(),
            service_type: "consul".to_string(),
            description: Some("Service discovered from Consul".to_string()),
            endpoints: vec![],
            health_check_endpoint: Some(format!("http://{address}:{port}/health")),
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: crate::traits::service::ServiceStatus::Running,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            instance_id: id,
            host: address.to_string(),
            port,
        })
    }
}

#[async_trait]
impl DiscoveryProvider for ConsulProviderAdapter {
    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
        tracing::info!("🏛️ Initializing Consul discovery provider adapter");
        // The ConsulServiceDiscovery doesn't need explicit initialization
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🏛️ Shutting down Consul discovery provider adapter");
        // The ConsulServiceDiscovery doesn't need explicit shutdown
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // For now, assume healthy. In a real implementation, you'd ping Consul
        Ok(true)
    }

    async fn register_service(&self, service: ServiceInfo) -> Result<()> {
        let _instance = self.to_service_instance(&service);
        // Note: The legacy backend has different method signatures
        // This is a temporary implementation until we fix the trait interface
        tracing::info!(
            "📝 Registering service {} via Consul adapter",
            service.service_id
        );

        // For now, return an error indicating the legacy backend needs updating
        Err(SongbirdError::internal_error(operation_error(
            "Legacy Consul backend needs trait interface updates to work with adapter",
        ))
    }

    async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        tracing::info!("🔍 Discovering services via native Consul adapter");

        let url = match query.name.as_deref() {
            Some(name) => format!("{}/v1/health/service/{}", self.consul_url, name),
            None => format!("{}/v1/agent/services", self.consul_url),
        };

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                SongbirdError::network(format!("Failed to query Consul: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(SongbirdError::internal_error(network_error(format!(
                "Consul query failed with status: {}",
                response.status()
            )));
        }

        let consul_response: serde_json::Value = response.json().await.map_err(|e| {
            SongbirdError::network(format!("Failed to parse Consul response: {e}"))
        })?;

        // Parse consul response into ServiceInfo
        let services = self.parse_consul_response(&consul_response)?;
        Ok(services)
    }

    async fn watch_services(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        // Consul supports watching, but the legacy backend doesn't expose it properly
        tracing::warn!("🔍 Consul watching not yet implemented in adapter");
        Ok(Box::pin(stream::empty()))
    }

    async fn list_all_services(&self) -> Result<Vec<ServiceInfo>> {
        tracing::info!("📋 Listing all services via Consul adapter");

        // For now, return an error indicating the legacy backend needs updating
        Err(SongbirdError::internal_error(operation_error(
            "Legacy Consul backend needs trait interface updates to work with adapter",
        ))
    }

    async fn service_exists(&self, service_id: &str) -> Result<bool> {
        tracing::debug!(
            "❓ Checking if service {} exists via Consul adapter",
            service_id
        );

        // For now, return an error indicating the legacy backend needs updating
        Err(SongbirdError::internal_error(operation_error(
            "Legacy Consul backend needs trait interface updates to work with adapter",
        ))
    }

    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        tracing::debug!(
            "📊 Getting metrics for service {} via Consul adapter",
            service_id
        );

        // Consul can provide some metrics through health checks
        Ok(ServiceMetrics {
            service_id: service_id.to_string(),
            request_count: 0,
            error_count: 0,
            average_response_time_ms: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            custom_metrics: HashMap::new(),
        })
    }

    async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints> {
        tracing::debug!(
            "⚖️ Getting load balancing hints for {} via Consul adapter",
            service_name
        );

        // Consul can provide load balancing through health scores
        Ok(LoadBalancingHints {
            service_name: service_name.to_string(),
            preferred_instances: vec![],
            weights: HashMap::new(),
            health_scores: HashMap::new(),
            locality_preferences: vec![],
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consul_factory_validation() {
        let factory = ConsulProviderFactory;

        // Test valid config
        let mut valid_config = factory.default_config("test".to_string(), "Test".to_string());
        assert!(factory.validate_config(&valid_config).is_ok());

        // Test invalid config (no URL)
        valid_config.parameters.clear();
        assert!(factory.validate_config(&valid_config).is_err());
    }

    #[test]
    fn test_consul_provider_metadata() {
        let adapter = ConsulProviderAdapter::new_native(
            "test-consul".to_string(),
            "http://localhost:8500".to_string(),
        );

        assert_eq!(adapter.metadata().id, "test-consul");
        assert!(adapter
            .metadata()
            .capabilities
            .contains(&DiscoveryCapability::ServiceRegistration));
        assert!(adapter
            .metadata()
            .capabilities
            .contains(&DiscoveryCapability::HealthChecking));
    }
}
