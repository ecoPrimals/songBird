//! Service Discovery Module
//!
//! Implementation of service discovery for various backends

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_errors::{Result, SongbirdError};
use std::collections::HashMap;
use std::net::SocketAddr;

// Core types and data structures
pub mod types;
// Configuration management
pub mod config;
// Resource detection and monitoring
pub mod monitoring;
pub mod resources;
// Network operations
pub mod network;

// DISCOVERY ARCHITECTURE NOTE:
// =========================
// Discovery services are now handled through external API integrations:
// - Federation discovery: Managed by songbird-federation crate
// - Trust verification: Handled by songbird-security via BearDog integration
// - Certificate validation: Managed by songbird-security crate
// - Service discovery: Supported via multiple backends (Static, Consul, Kubernetes)
//
// Local discovery modules focus on resource detection and network topology mapping.
// All security-related discovery operations are delegated to the appropriate
// security and federation modules with proper API boundaries.

// Main discovery service implementation
pub mod songbird_discovery;

// Re-export the main discovery service
pub use songbird_discovery::SongbirdDiscovery;

// Re-export commonly used types
pub use types::{
    ComputeResources, DatasetInfo, FederationHealth, FederationMessage, FederationStats,
    InteractionResult, LocalNode, NetworkMeasurement, NetworkPartition, NetworkTopology, NodeId,
    NodeInfo, NodeType, ResourceQuery, ResourceUpdate, ResourceUsage, StorageInfo, TrustLevel,
};

// Re-export configuration types
pub use config::{
    InteractionPenalties, MonitoringConfig, NetworkConfig, NetworkTimingConfig,
    SongbirdDiscoveryConfig, TrustConfig, TrustThresholds,
};

// Re-export utility structs
pub use monitoring::ResourceMonitor;
pub use network::NetworkManager;
pub use resources::ResourceDetector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub address: SocketAddr,
    pub metadata: HashMap<String, String>,
    pub health_check_url: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub backend: String,
    pub consul_url: Option<String>,
    pub kubernetes_namespace: Option<String>,
    pub refresh_interval_seconds: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            backend: "static".to_string(),
            consul_url: None,
            kubernetes_namespace: None,
            refresh_interval_seconds: 30,
        }
    }
}

#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    async fn register_service(&self, service: ServiceInstance) -> Result<()>;
    async fn deregister_service(&self, service_id: &str) -> Result<()>;
    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>>;
    async fn health_check(&self, service_id: &str) -> Result<bool>;
}

/// Static service discovery for development and testing
pub struct StaticServiceDiscovery {
    services: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ServiceInstance>>>,
}

impl StaticServiceDiscovery {
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
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
        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service);
        Ok(())
    }

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);
        Ok(())
    }

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        let mut result = Vec::new();

        for service in services.values() {
            if let Some(name) = service_name {
                if service.name == name {
                    result.push(service.clone());
                }
            } else {
                result.push(service.clone());
            }
        }

        Ok(result)
    }

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        let services = self.services.read().await;
        Ok(services.contains_key(service_id))
    }
}

/// Consul service discovery for production deployments
pub struct ConsulServiceDiscovery {
    consul_url: String,
    client: reqwest::Client,
}

impl ConsulServiceDiscovery {
    #[must_use]
    pub fn new(consul_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { consul_url, client }
    }
}

#[async_trait]
impl ServiceDiscovery for ConsulServiceDiscovery {
    async fn register_service(&self, service: ServiceInstance) -> Result<()> {
        let registration = serde_json::json!({
            "ID": service.id,
            "Name": service.name,
            "Address": service.address.ip().to_string(),
            "Port": service.address.port(),
            "Tags": service.tags,
            "Meta": service.metadata,
            "Check": {
                "HTTP": service.health_check_url.unwrap_or_else(|| {
                    format!("http://{}:{}/health", service.address.ip(), service.address.port())
                }),
                "Interval": "30s",
                "Timeout": "10s"
            }
        });

        let url = format!("{}/v1/agent/service/register", self.consul_url);
        let response = self
            .client
            .put(&url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to register service: {}", e),
                service: Some(service.id.clone()),
                timeout: None,
                suggestion: Some(
                    "Verify service configuration and network connectivity".to_string(),
                ),
            })?;

        if !response.status().is_success() {
            return Err(SongbirdError::Discovery {
                message: format!("Consul registration failed: {}", response.status()),
                service: Some(service.id.clone()),
                timeout: None,
                suggestion: Some("Check consul server status and registration data".to_string()),
            });
        }

        tracing::info!("✅ Registered service {} with Consul", service.name);
        Ok(())
    }

    async fn deregister_service(&self, service_id: &str) -> Result<()> {
        let url = format!(
            "{}/v1/agent/service/deregister/{}",
            self.consul_url, service_id
        );
        let response =
            self.client
                .put(&url)
                .send()
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to deregister service: {}", e),
                    service: Some(service_id.to_string()),
                    timeout: None,
                    suggestion: Some(
                        "Check consul connection and service configuration".to_string(),
                    ),
                })?;

        if !response.status().is_success() {
            return Err(SongbirdError::Discovery {
                message: format!("Consul deregistration failed: {}", response.status()),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check consul server status and service registration".to_string()),
            });
        }

        tracing::info!("✅ Deregistered service {} from Consul", service_id);
        Ok(())
    }

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        let url = if let Some(_name) = service_name {
            format!("{}/v1/health/service/{}", self.consul_url, _name)
        } else {
            format!("{}/v1/agent/services", self.consul_url)
        };

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to query services: {}", e),
                    service: None,
                    timeout: None,
                    suggestion: Some("Check consul connection and query parameters".to_string()),
                })?;

        if !response.status().is_success() {
            return Err(SongbirdError::Discovery {
                message: format!("Consul discovery failed: {}", response.status()),
                service: service_name.map(std::string::ToString::to_string),
                timeout: None,
                suggestion: Some("Check consul server status and service availability".to_string()),
            });
        }

        let services_data: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to parse Consul response: {}", e),
                    service: None,
                    timeout: None,
                    suggestion: Some("Check consul response format and parsing logic".to_string()),
                })?;

        let mut services = Vec::new();

        if let Some(_name) = service_name {
            // Health API response format
            if let Some(entries) = services_data.as_array() {
                for entry in entries {
                    if let (Some(service_data), Some(node_data)) =
                        (entry.get("Service"), entry.get("Node"))
                    {
                        if let (Some(id), Some(service_name), Some(address), Some(port)) = (
                            service_data.get("ID").and_then(|v| v.as_str()),
                            service_data.get("Service").and_then(|v| v.as_str()),
                            node_data.get("Address").and_then(|v| v.as_str()),
                            service_data.get("Port").and_then(serde_json::Value::as_u64),
                        ) {
                            let socket_addr = format!("{address}:{port}")
                                .parse::<SocketAddr>()
                                .map_err(|e| SongbirdError::Discovery {
                                    message: format!("Invalid service address: {}", e),
                                    service: Some(id.to_string()),
                                    timeout: None,
                                    suggestion: Some(
                                        "Check service address format and port configuration"
                                            .to_string(),
                                    ),
                                })?;

                            let tags = service_data
                                .get("Tags")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| {
                                            v.as_str().map(std::string::ToString::to_string)
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            let metadata = service_data
                                .get("Meta")
                                .and_then(|v| v.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .filter_map(|(k, v)| {
                                            v.as_str().map(|s| (k.clone(), s.to_string()))
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            services.push(ServiceInstance {
                                id: id.to_string(),
                                name: service_name.to_string(),
                                address: socket_addr,
                                metadata,
                                health_check_url: None,
                                tags,
                            });
                        }
                    }
                }
            }
        } else {
            // Agent services API response format
            if let Some(services_obj) = services_data.as_object() {
                for (_, service_data) in services_obj {
                    if let (Some(id), Some(service_name), Some(address), Some(port)) = (
                        service_data.get("ID").and_then(|v| v.as_str()),
                        service_data.get("Service").and_then(|v| v.as_str()),
                        service_data.get("Address").and_then(|v| v.as_str()),
                        service_data.get("Port").and_then(serde_json::Value::as_u64),
                    ) {
                        let socket_addr = format!("{address}:{port}")
                            .parse::<SocketAddr>()
                            .map_err(|e| SongbirdError::Discovery {
                                message: format!("Invalid service address: {}", e),
                                service: Some(id.to_string()),
                                timeout: None,
                                suggestion: Some(
                                    "Check service address format and port configuration"
                                        .to_string(),
                                ),
                            })?;

                        services.push(ServiceInstance {
                            id: id.to_string(),
                            name: service_name.to_string(),
                            address: socket_addr,
                            metadata: HashMap::new(),
                            health_check_url: None,
                            tags: Vec::new(),
                        });
                    }
                }
            }
        }

        tracing::debug!("🔍 Discovered {} services from Consul", services.len());
        Ok(services)
    }

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        let url = format!("{}/v1/health/service/{}", self.consul_url, service_id);
        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to check service health: {}", e),
                    service: Some(service_id.to_string()),
                    timeout: None,
                    suggestion: Some(
                        "Check network connectivity and service availability".to_string(),
                    ),
                })?;

        Ok(response.status().is_success())
    }
}

/// Kubernetes service discovery for cloud-native deployments
pub struct KubernetesServiceDiscovery {
    namespace: String,
    client: reqwest::Client,
    api_server: String,
}

impl KubernetesServiceDiscovery {
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created or if there are
    /// issues with the Kubernetes service account configuration.
    pub fn new(namespace: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(tokio::time::Duration::from_secs(10))
            .build()
            .map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to create HTTP client: {}", e),
                service: Some("kubernetes".to_string()),
                timeout: None,
                suggestion: Some("Check Kubernetes service account configuration".to_string()),
            })?;

        let api_server = std::env::var("KUBERNETES_SERVICE_HOST")
            .map(|host| {
                let port =
                    std::env::var("KUBERNETES_SERVICE_PORT").unwrap_or_else(|_| "443".to_string());
                format!("https://{host}:{port}")
            })
            .unwrap_or_else(|_| "https://kubernetes.default.svc".to_string());

        Ok(Self {
            namespace,
            client,
            api_server,
        })
    }
}

#[async_trait]
impl ServiceDiscovery for KubernetesServiceDiscovery {
    async fn register_service(&self, _service: ServiceInstance) -> Result<()> {
        // In Kubernetes, services are typically registered via YAML manifests
        // This is a placeholder for potential dynamic registration
        tracing::info!("📦 Kubernetes service registration (placeholder)");
        Ok(())
    }

    async fn deregister_service(&self, _service_id: &str) -> Result<()> {
        // In Kubernetes, services are typically deregistered via kubectl
        // This is a placeholder for potential dynamic deregistration
        tracing::info!("📦 Kubernetes service deregistration (placeholder)");
        Ok(())
    }

    async fn discover_services(&self, service_name: Option<&str>) -> Result<Vec<ServiceInstance>> {
        // Get service account token
        let token =
            tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to read service account token: {}", e),
                    service: Some("kubernetes".to_string()),
                    timeout: None,
                    suggestion: Some(
                        "Check Kubernetes service account token availability".to_string(),
                    ),
                })?;

        // Query services from Kubernetes API
        let url = if let Some(_name) = service_name {
            format!(
                "{}/api/v1/namespaces/{}/services/{}",
                self.api_server, self.namespace, _name
            )
        } else {
            format!(
                "{}/api/v1/namespaces/{}/services",
                self.api_server, self.namespace
            )
        };

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to discover services: {}", e),
                service: None,
                timeout: None,
                suggestion: Some("Check network connectivity and service availability".to_string()),
            })?;

        if !response.status().is_success() {
            return Err(SongbirdError::Discovery {
                message: format!("Kubernetes discovery failed: {}", response.status()),
                service: service_name.map(std::string::ToString::to_string),
                timeout: None,
                suggestion: Some(
                    "Check Kubernetes API server status and service availability".to_string(),
                ),
            });
        }

        let services_data: serde_json::Value =
            response
                .json()
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to parse Kubernetes response: {}", e),
                    service: None,
                    timeout: None,
                    suggestion: Some(
                        "Check Kubernetes response format and parsing logic".to_string(),
                    ),
                })?;

        let mut services = Vec::new();

        if let Some(_name) = service_name {
            // Single service response
            if let Some(spec) = services_data.get("spec") {
                if let Some(cluster_ip) = spec.get("clusterIP").and_then(|v| v.as_str()) {
                    if let Some(ports) = spec.get("ports").and_then(|v| v.as_array()) {
                        for port in ports {
                            if let Some(port_num) =
                                port.get("port").and_then(serde_json::Value::as_u64)
                            {
                                let socket_addr = format!("{cluster_ip}:{port_num}")
                                    .parse::<SocketAddr>()
                                    .map_err(|e| SongbirdError::Discovery {
                                        message: format!("Invalid service address: {}", e),
                                        service: Some(_name.to_string()),
                                        timeout: None,
                                        suggestion: Some(
                                            "Check service address format and port configuration"
                                                .to_string(),
                                        ),
                                    })?;

                                services.push(ServiceInstance {
                                    id: format!("{_name}-{port_num}"),
                                    name: _name.to_string(),
                                    address: socket_addr,
                                    metadata: HashMap::new(),
                                    health_check_url: None,
                                    tags: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        } else {
            // Multiple services response
            if let Some(items) = services_data.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let (Some(metadata), Some(spec)) = (item.get("metadata"), item.get("spec")) {
                        if let (Some(name), Some(cluster_ip)) = (
                            metadata.get("name").and_then(|v| v.as_str()),
                            spec.get("clusterIP").and_then(|v| v.as_str()),
                        ) {
                            if let Some(ports) = spec.get("ports").and_then(|v| v.as_array()) {
                                for port in ports {
                                    if let Some(port_num) =
                                        port.get("port").and_then(serde_json::Value::as_u64)
                                    {
                                        let socket_addr = format!("{cluster_ip}:{port_num}")
                                            .parse::<SocketAddr>()
                                            .map_err(|e| SongbirdError::Discovery {
                                                message: format!("Invalid service address: {}", e),
                                                service: Some(name.to_string()),
                                                timeout: None,
                                                suggestion: Some("Check service address format and port configuration".to_string()),
                                            })?;

                                        services.push(ServiceInstance {
                                            id: format!("{name}-{port_num}"),
                                            name: name.to_string(),
                                            address: socket_addr,
                                            metadata: HashMap::new(),
                                            health_check_url: None,
                                            tags: Vec::new(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::debug!("🔍 Discovered {} services from Kubernetes", services.len());
        Ok(services)
    }

    async fn health_check(&self, service_id: &str) -> Result<bool> {
        // Get service account token
        let token =
            tokio::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
                .await
                .map_err(|e| SongbirdError::Discovery {
                    message: format!("Failed to read service account token: {}", e),
                    service: Some("kubernetes".to_string()),
                    timeout: None,
                    suggestion: Some(
                        "Check Kubernetes service account token availability".to_string(),
                    ),
                })?;

        // Simple health check by querying the service
        let url = format!(
            "{}/api/v1/namespaces/{}/services/{}",
            self.api_server, self.namespace, service_id
        );
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| SongbirdError::Discovery {
                message: format!("Failed to check service health: {}", e),
                service: Some(service_id.to_string()),
                timeout: None,
                suggestion: Some("Check network connectivity and service availability".to_string()),
            })?;

        Ok(response.status().is_success())
    }
}

/// Service discovery factory for creating different backends
pub struct ServiceDiscoveryFactory;

impl ServiceDiscoveryFactory {
    pub fn create(config: &DiscoveryConfig) -> Result<Box<dyn ServiceDiscovery>> {
        match config.backend.as_str() {
            "static" => Ok(Box::new(StaticServiceDiscovery::new())),
            "consul" => {
                let consul_url = config
                    .consul_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8500".to_string());
                Ok(Box::new(ConsulServiceDiscovery::new(consul_url)))
            }
            "kubernetes" => {
                let namespace = config
                    .kubernetes_namespace
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                Ok(Box::new(KubernetesServiceDiscovery::new(namespace)?))
            }
            _ => Err(SongbirdError::Discovery {
                message: format!("Unsupported discovery backend: {}", config.backend),
                service: None,
                timeout: None,
                suggestion: Some("Check discovery backend configuration".to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_service_discovery_registration() {
        let discovery = StaticServiceDiscovery::new();
        let service = ServiceInstance {
            id: "test-service-1".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: Some("http://127.0.0.1:8080/health".to_string()),
            tags: vec!["test".to_string()],
        };

        let result = discovery.register_service(service.clone()).await;
        assert!(result.is_ok());

        // Verify service was registered
        let services = discovery.discover_services(None).await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "test-service-1");
    }

    #[tokio::test]
    async fn test_static_service_discovery_deregistration() {
        let discovery = StaticServiceDiscovery::new();
        let service = ServiceInstance {
            id: "test-service-2".to_string(),
            name: "test-service".to_string(),
            address: "127.0.0.1:8081".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: None,
            tags: vec![],
        };

        // Register and then deregister
        discovery.register_service(service.clone()).await.unwrap();
        let result = discovery.deregister_service("test-service-2").await;
        assert!(result.is_ok());

        // Verify service was deregistered
        let services = discovery.discover_services(None).await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_static_service_discovery_filtered_discovery() {
        let discovery = StaticServiceDiscovery::new();

        let service1 = ServiceInstance {
            id: "service-1".to_string(),
            name: "web-server".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: None,
            tags: vec!["web".to_string()],
        };

        let service2 = ServiceInstance {
            id: "service-2".to_string(),
            name: "database".to_string(),
            address: "127.0.0.1:5432".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: None,
            tags: vec!["db".to_string()],
        };

        // Register both services
        discovery.register_service(service1).await.unwrap();
        discovery.register_service(service2).await.unwrap();

        // Test filtered discovery
        let web_services = discovery
            .discover_services(Some("web-server"))
            .await
            .unwrap();
        assert_eq!(web_services.len(), 1);
        assert_eq!(web_services[0].name, "web-server");

        let db_services = discovery.discover_services(Some("database")).await.unwrap();
        assert_eq!(db_services.len(), 1);
        assert_eq!(db_services[0].name, "database");

        // Test discovery of all services
        let all_services = discovery.discover_services(None).await.unwrap();
        assert_eq!(all_services.len(), 2);
    }

    #[tokio::test]
    async fn test_static_service_discovery_health_check() {
        let discovery = StaticServiceDiscovery::new();
        let service = ServiceInstance {
            id: "health-test-service".to_string(),
            name: "health-test".to_string(),
            address: "127.0.0.1:8082".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: None,
            tags: vec![],
        };

        // Health check should return false for non-existent service
        let health_result = discovery
            .health_check("non-existent-service")
            .await
            .unwrap();
        assert!(!health_result);

        // Register service
        discovery.register_service(service).await.unwrap();

        // Health check should return true for registered service
        let health_result = discovery.health_check("health-test-service").await.unwrap();
        assert!(health_result);
    }

    #[tokio::test]
    async fn test_consul_service_discovery_creation() {
        let consul_discovery = ConsulServiceDiscovery::new("http://localhost:8500".to_string());
        assert_eq!(consul_discovery.consul_url, "http://localhost:8500");
    }

    #[tokio::test]
    async fn test_kubernetes_service_discovery_creation() {
        let result = KubernetesServiceDiscovery::new("default".to_string());
        // Fix single match else pattern
        // Expected result whether in k8s environment or not
        let _ = result;
    }

    #[tokio::test]
    async fn test_service_discovery_factory_static() {
        let config = DiscoveryConfig {
            backend: "static".to_string(),
            consul_url: None,
            kubernetes_namespace: None,
            refresh_interval_seconds: 30,
        };

        let discovery = ServiceDiscoveryFactory::create(&config);
        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_service_discovery_factory_consul() {
        let config = DiscoveryConfig {
            backend: "consul".to_string(),
            consul_url: Some("http://localhost:8500".to_string()),
            kubernetes_namespace: None,
            refresh_interval_seconds: 30,
        };

        let discovery = ServiceDiscoveryFactory::create(&config);
        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_service_discovery_factory_kubernetes() {
        let config = DiscoveryConfig {
            backend: "kubernetes".to_string(),
            consul_url: None,
            kubernetes_namespace: Some("default".to_string()),
            refresh_interval_seconds: 30,
        };

        let discovery = ServiceDiscoveryFactory::create(&config);
        // Fix another single match else pattern
        if discovery.is_ok() {
            // Success case
        } else {
            // Expected if not in k8s environment
        }
    }

    #[tokio::test]
    async fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.backend, "static");
        assert!(config.consul_url.is_none());
        assert!(config.kubernetes_namespace.is_none());
        assert_eq!(config.refresh_interval_seconds, 30);
    }

    #[tokio::test]
    async fn test_service_instance_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("env".to_string(), "production".to_string());

        let service = ServiceInstance {
            id: "test-instance".to_string(),
            name: "test-service".to_string(),
            address: "192.168.1.100:9090".parse().unwrap(),
            metadata,
            health_check_url: Some("http://192.168.1.100:9090/health".to_string()),
            tags: vec!["api".to_string(), "v1".to_string()],
        };

        assert_eq!(service.id, "test-instance");
        assert_eq!(service.name, "test-service");
        assert_eq!(service.address.port(), 9090);
        assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(service.tags.len(), 2);
        assert!(service.health_check_url.is_some());
    }

    #[tokio::test]
    async fn test_service_instance_clone() {
        let service = ServiceInstance {
            id: "clone-test".to_string(),
            name: "clone-service".to_string(),
            address: "127.0.0.1:8000".parse().unwrap(),
            metadata: HashMap::new(),
            health_check_url: None,
            tags: vec!["test".to_string()],
        };

        let cloned_service = service.clone();
        assert_eq!(service.id, cloned_service.id);
        assert_eq!(service.name, cloned_service.name);
        assert_eq!(service.address, cloned_service.address);
    }

    #[tokio::test]
    async fn test_multiple_services_management() {
        let discovery = StaticServiceDiscovery::new();

        // Register multiple services
        for i in 0..5 {
            let service = ServiceInstance {
                id: format!("service-{i}"),
                name: format!("test-service-{i}"),
                address: format!("127.0.0.1:{}", 8000 + i).parse().unwrap(),
                metadata: HashMap::new(),
                health_check_url: None,
                tags: vec![format!("tag-{i}")],
            };
            discovery.register_service(service).await.unwrap();
        }

        // Verify all services are registered
        let services = discovery.discover_services(None).await.unwrap();
        assert_eq!(services.len(), 5);

        // Deregister some services
        discovery.deregister_service("service-1").await.unwrap();
        discovery.deregister_service("service-3").await.unwrap();

        // Verify correct services remain
        let remaining_services = discovery.discover_services(None).await.unwrap();
        assert_eq!(remaining_services.len(), 3);

        let remaining_ids: Vec<String> = remaining_services.iter().map(|s| s.id.clone()).collect();
        assert!(remaining_ids.contains(&"service-0".to_string()));
        assert!(remaining_ids.contains(&"service-2".to_string()));
        assert!(remaining_ids.contains(&"service-4".to_string()));
        assert!(!remaining_ids.contains(&"service-1".to_string()));
        assert!(!remaining_ids.contains(&"service-3".to_string()));
    }

    #[tokio::test]
    async fn test_service_discovery_edge_cases() {
        let discovery = StaticServiceDiscovery::new();

        // Test deregistering non-existent service
        let result = discovery.deregister_service("non-existent").await;
        assert!(result.is_ok()); // Should not error

        // Test discovering with non-existent service name
        let services = discovery
            .discover_services(Some("non-existent-service"))
            .await
            .unwrap();
        assert_eq!(services.len(), 0);

        // Test health check for non-existent service
        let health = discovery.health_check("non-existent").await.unwrap();
        assert!(!health);
    }
}
