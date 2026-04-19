// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Consul Provider Adapter
//!
//! Provides Consul service discovery using the universal provider pattern
//!
//! # Native Async Traits (Rust 1.75+)
//! Uses native async fn in traits for high-performance Consul integration

use futures_util::stream::{self, Stream};
use songbird_http_client::IpcHttpClient;
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use super::{DiscoveryProviderImpl, ProviderFactory};
use crate::abstraction::{
    capabilities::DiscoveryCapability,
    providers::{
        DiscoveryProvider, LoadBalancingHints, ProviderConfig, ProviderMetadata, ServiceMetrics,
    },
};

use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_types::SongbirdError;

type Result<T> = songbird_types::SongbirdResult<T>;

/// Factory for creating Consul providers from configuration
#[derive(Debug, Clone, Copy)]
pub struct ConsulProviderFactory;

impl ProviderFactory for ConsulProviderFactory {
    fn provider_type(&self) -> &'static str {
        "consul"
    }

    async fn create_provider(&self, config: ProviderConfig) -> Result<DiscoveryProviderImpl> {
        // Extract Consul URL from flexible configuration
        let consul_url = config
            .parameters
            .get("url")
            .or_else(|| config.parameters.get("consul_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SongbirdError::configuration(
                    "Consul URL required in parameters.url or parameters.consul_url",
                )
            })?;

        // Create native consul adapter (no longer using deprecated backend)
        let adapter = ConsulProviderAdapter::new_native(config.id, consul_url.to_string()).await?;
        Ok(DiscoveryProviderImpl::Consul(adapter))
    }

    fn validate_config(&self, config: &ProviderConfig) -> Result<()> {
        // Validate required parameters
        if !config.parameters.contains_key("url") && !config.parameters.contains_key("consul_url") {
            return Err(SongbirdError::configuration(
                "Consul provider requires 'url' or 'consul_url' parameter",
            ));
        }

        // Validate URL format if provided
        if let Some(url) = config.parameters.get("url").and_then(|v| v.as_str()) {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(SongbirdError::configuration(
                    "Consul URL must start with http:// or https://",
                ));
            }
        }

        Ok(())
    }

    fn default_config(&self, id: String, name: String) -> ProviderConfig {
        use songbird_config::canonical::constants;
        use songbird_config::defaults::default_host;

        // Get configurable defaults from environment
        let consul_host =
            songbird_process_env::var("CONSUL_HOST").unwrap_or_else(|_| default_host());
        let consul_port = songbird_process_env::var("CONSUL_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8500); // Standard Consul port
        let consul_datacenter =
            songbird_process_env::var("CONSUL_DATACENTER").unwrap_or_else(|_| "dc1".to_string());
        let consul_protocol =
            songbird_process_env::var("CONSUL_PROTOCOL").unwrap_or_else(|_| "http".to_string());

        let consul_url = format!("{consul_protocol}://{consul_host}:{consul_port}");

        let mut parameters = HashMap::new();
        parameters.insert("url".to_string(), serde_json::Value::String(consul_url.clone()));
        parameters.insert("datacenter".to_string(), serde_json::Value::String(consul_datacenter));
        parameters
            .insert("token".to_string(), serde_json::Value::String("${CONSUL_TOKEN}".to_string()));

        let mut environment = HashMap::new();
        environment.insert("CONSUL_HTTP_ADDR".to_string(), consul_url);

        ProviderConfig {
            id,
            name,
            parameters,
            environment,
            timeout_ms: Some(constants::get_connection_timeout_ms()),
            retry_config: None,
        }
    }
}

/// Native Consul provider adapter (no longer wrapping deprecated backend)
pub struct ConsulProviderAdapter {
    metadata: ProviderMetadata,
    consul_url: String,
    client: IpcHttpClient,
}

impl ConsulProviderAdapter {
    /// Create new native consul adapter
    pub async fn new_native(id: String, consul_url: String) -> Result<Self> {
        // Determine protocol from URL
        let protocol = if consul_url.starts_with("https://") {
            "https"
        } else {
            "http"
        }
        .to_string();

        // Get version from environment or use crate version
        let version = songbird_process_env::var("SONGBIRD_VERSION")
            .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

        let metadata = ProviderMetadata {
            id: id.clone(),
            name: format!("Consul Provider ({id})"),
            version,
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
                meta.insert("protocol".to_string(), protocol);
                meta.insert("vendor".to_string(), "hashicorp".to_string());
                meta
            },
            healthy: true,
            load_score: 0.5, // Medium load score - configurable via ConsulConfig
        };

        let client = IpcHttpClient::new()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            metadata,
            consul_url,
            client,
        })
    }

    /// Convert `ServiceInfo` to `ServiceInstance`.
    ///
    /// Returns an error if the host address cannot be parsed — no silent
    /// fallback to localhost (capability-based: require valid address from
    /// the discovery source).
    fn to_service_instance(
        &self,
        service: &ServiceInfo,
    ) -> Result<crate::discovery::core::ServiceInstance> {
        use std::net::IpAddr;

        let _: IpAddr = service.host.parse().map_err(|_| {
            SongbirdError::configuration(format!(
                "Consul service '{}' has unparseable host '{}' — capability discovery requires a valid address",
                service.service_id, service.host,
            ))
        })?;

        let protocol = if self.consul_url.starts_with("https://") {
            "https"
        } else {
            "http"
        };
        let endpoint = format!("{protocol}://{}:{}", service.host, service.port);

        Ok(crate::discovery::core::ServiceInstance {
            id: service.service_id.clone(),
            name: service.name.clone(),
            endpoint,
            capabilities: Vec::new(),
            health_status: "unknown".to_string(),
            metadata: service.metadata.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
        })
    }

    /// Parse Consul API response into `ServiceInfo` list
    #[must_use]
    fn parse_consul_response(&self, response: &serde_json::Value) -> Vec<ServiceInfo> {
        let mut services = Vec::new();

        if let Some(service_list) = response.as_array() {
            for service_entry in service_list {
                if let Some(service_info) = self.parse_consul_service(service_entry) {
                    services.push(service_info);
                }
            }
        } else if let Some(service_map) = response.as_object() {
            for (_, service_data) in service_map {
                if let Some(service_info) = self.parse_consul_service(service_data) {
                    services.push(service_info);
                }
            }
        }

        services
    }

    /// Parse individual Consul service into `ServiceInfo`.
    ///
    /// Returns `None` (skips the entry) when required fields (`ID`, `Address`,
    /// `Port`) are absent — no silent fallback to localhost or default ports.
    fn parse_consul_service(&self, service: &serde_json::Value) -> Option<ServiceInfo> {
        let id = service["ID"].as_str()?.to_string();
        let name = service["Service"].as_str().unwrap_or(&id).to_string();
        let address = if let Some(a) = service["Address"].as_str().filter(|a| !a.is_empty()) {
            a
        } else {
            tracing::warn!(
                "Consul service '{name}' has no Address — skipping (capability-based: require valid address)"
            );
            return None;
        };
        let port = if let Some(p) = service["Port"].as_u64() {
            p as u16
        } else {
            tracing::warn!(
                "Consul service '{name}' has no Port — skipping (capability-based: require valid port)"
            );
            return None;
        };

        // Determine protocol from consul URL or service metadata
        let protocol = if self.consul_url.starts_with("https://") {
            "https"
        } else {
            "http"
        };

        // Get version from service metadata or use unknown
        let version = service["Version"]
            .as_str()
            .map_or_else(|| "unknown".to_string(), std::string::ToString::to_string);

        Some(ServiceInfo {
            service_id: id.clone(),
            name,
            version,
            service_type: "consul".to_string(),
            description: Some("Service discovered from Consul".to_string()),
            endpoints: vec![],
            health_check_endpoint: Some(format!("{protocol}://{address}:{port}/health")),
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

    async fn register(&self, service: ServiceInfo) -> Result<()> {
        let _instance = self.to_service_instance(&service)?;
        tracing::info!("📝 Registering service {} via Consul adapter", service.service_id);

        Err(SongbirdError::discovery(
            "Consul registration requires native API integration (trait interface update pending)",
        ))
    }

    async fn unregister(&self, service_id: &str) -> Result<()> {
        Err(SongbirdError::discovery(format!(
            "Consul unregister not implemented in native adapter (service_id={service_id})"
        )))
    }

    async fn update_health(&self, service_id: &str, _health: ServiceHealthStatus) -> Result<()> {
        Err(SongbirdError::discovery(format!(
            "Consul health update not implemented in native adapter (service_id={service_id})"
        )))
    }

    async fn update_metadata(
        &self,
        service_id: &str,
        _metadata: HashMap<String, String>,
    ) -> Result<()> {
        Err(SongbirdError::discovery(format!(
            "Consul metadata update not implemented in native adapter (service_id={service_id})"
        )))
    }

    async fn discover(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        tracing::info!("🔍 Discovering services via native Consul adapter");

        let url = match query.name.as_deref() {
            Some(name) => format!("{}/v1/health/service/{name}", self.consul_url),
            None => format!("{}/v1/agent/services", self.consul_url),
        };

        let response = self
            .client
            .get(&url)
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to query Consul: {e}")))?;

        if !response.is_success() {
            return Err(SongbirdError::network(format!(
                "Consul query failed with status: {}",
                response.status()
            )));
        }

        let consul_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SongbirdError::network(format!("Failed to parse Consul response: {e}")))?;

        // Parse consul response into ServiceInfo
        let services = self.parse_consul_response(&consul_response);
        Ok(services)
    }

    async fn watch(
        &self,
        _query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        // Consul supports watching, but the legacy backend doesn't expose it properly
        tracing::warn!("🔍 Consul watching not yet implemented in adapter");
        Ok(Box::pin(stream::empty()))
    }

    async fn list_all(&self) -> Result<Vec<ServiceInfo>> {
        tracing::info!("📋 Listing all services via Consul adapter");

        // For now, return an error indicating the legacy backend needs updating
        Err(SongbirdError::discovery(
            "Legacy Consul backend needs trait interface updates to work with adapter",
        ))
    }

    async fn exists(&self, service_id: &str) -> Result<bool> {
        tracing::debug!("❓ Checking if service {} exists via Consul adapter", service_id);

        // For now, return an error indicating the legacy backend needs updating
        Err(SongbirdError::discovery(
            "Legacy Consul backend needs trait interface updates to work with adapter",
        ))
    }

    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        tracing::debug!("📊 Getting metrics for service {} via Consul adapter", service_id);

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
        tracing::debug!("⚖️ Getting load balancing hints for {} via Consul adapter", service_name);

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

    #[tokio::test]
    async fn test_consul_provider_metadata() {
        use songbird_config::canonical::constants;

        let test_url = format!("http://{}:8500", constants::network::DEFAULT_HOST);
        let adapter = ConsulProviderAdapter::new_native("test-consul".to_string(), test_url)
            .await
            .expect("create consul adapter");

        assert_eq!(adapter.metadata().id, "test-consul");
        assert!(
            adapter.metadata().capabilities.contains(&DiscoveryCapability::ServiceRegistration)
        );
        assert!(adapter.metadata().capabilities.contains(&DiscoveryCapability::HealthChecking));
    }
}
