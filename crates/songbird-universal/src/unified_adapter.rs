//! # 🔧 Unified Universal Adapter
//!
//! **SINGLE SOURCE OF TRUTH FOR UNIVERSAL ADAPTATION** ✅
//!
//! This module consolidates all fragmented `UniversalCapabilityAdapter` implementations
//! into a single, unified adapter that can handle any capability type.

use crate::capabilities::Capability;
use crate::types::{HealthStatus, ServiceInfo, UniversalRequest, UniversalResponse};
use serde::{Deserialize, Serialize};
use songbird_types::SafeEnv;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
// CapabilityProvider and PerformanceMetrics have been moved to canonical traits

// ============================================================================
// UNIFIED UNIVERSAL ADAPTER
// ============================================================================

/// **UNIFIED**: Universal capability adapter that consolidates all adapter patterns
///
/// This replaces:
/// - `songbird-universal::adapters::UniversalCapabilityAdapter`
/// - `songbird-universal::capabilities::UniversalCapabilityAdapter`
/// - `songbird-universal-primals::universal_adapter::*`
/// - Various other adapter fragments
#[derive(Debug, Clone)]
pub struct UnifiedUniversalAdapter {
    /// Registry of discovered capabilities
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    /// Active service connections (reserved for connection pooling implementation)
    #[allow(dead_code)]
    service_connections: Arc<RwLock<HashMap<String, ServiceConnection>>>,
    /// Adapter configuration
    config: UnifiedAdapterConfig,
    // Note: HTTP client created on-demand in methods to support async initialization
}

/// **UNIFIED**: Capability registry for discovered services
///
/// **FUTURE OPTIMIZATION**: Consider `Arc<str>` for service IDs and capability names
/// when profiling shows clone overhead. Current design prioritizes simplicity.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// Map of service ID to their capabilities
    pub service_capabilities: HashMap<String, Vec<Capability>>,
    /// Map of capability type to services that provide it
    pub capability_providers: HashMap<String, Vec<String>>,
    /// Service metadata and health information
    pub service_info: HashMap<String, ServiceInfo>,
    /// Last update timestamp for each service
    pub last_updated: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// **UNIFIED**: Service connection information
#[derive(Debug, Clone)]
pub struct ServiceConnection {
    /// Service endpoint
    pub endpoint: String,
    /// Connection health status
    pub health: HealthStatus,
    /// Performance metrics
    pub metrics: std::collections::HashMap<String, f64>, // Simplified metrics for now
    /// Last successful communication
    pub last_contact: chrono::DateTime<chrono::Utc>,
}

/// **UNIFIED**: Adapter configuration
#[derive(Debug, Clone)]
pub struct UnifiedAdapterConfig {
    /// Discovery timeout
    pub discovery_timeout: std::time::Duration,
    /// Health check interval
    pub health_check_interval: std::time::Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable automatic service discovery
    pub auto_discovery: bool,
    /// Service discovery endpoints
    pub discovery_endpoints: Vec<String>,
}

impl Default for UnifiedAdapterConfig {
    fn default() -> Self {
        Self {
            discovery_timeout: std::time::Duration::from_secs(30),
            health_check_interval: std::time::Duration::from_secs(60),
            max_concurrent_requests: 100,
            auto_discovery: true,
            discovery_endpoints: {
                let host = SafeEnv::get_or_default(
                    "ADAPTER_DISCOVERY_HOST",
                    songbird_config::canonical::constants::get_bind_address(),
                );
                let capabilities_port = SafeEnv::get_port(
                    "ADAPTER_CAPABILITIES_PORT",
                    songbird_config::canonical::constants::network::default_orchestrator_port(),
                )
                .to_string();
                let services_port = SafeEnv::get_port(
                    "ADAPTER_SERVICES_PORT",
                    songbird_config::defaults::ports::discovery_port(),
                )
                .to_string();
                vec![
                    format!("http://{}:{}/capabilities", host, capabilities_port),
                    format!("http://{}:{}/services", host, services_port),
                ]
            },
        }
    }
}

impl UnifiedUniversalAdapter {
    /// Create a new unified adapter with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(UnifiedAdapterConfig::default())
    }

    /// Create a new unified adapter with custom configuration
    #[must_use]
    pub fn with_config(config: UnifiedAdapterConfig) -> Self {
        Self {
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())),
            service_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            // HTTP client created on-demand in methods
        }
    }

    /// Discover services and their capabilities
    ///
    /// # Errors
    ///
    /// This function logs errors but does not fail - it returns all successfully discovered services
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        info!("🔍 Starting universal service discovery");

        let mut discovered_services = Vec::new();

        for endpoint in &self.config.discovery_endpoints {
            match self.discover_from_endpoint(endpoint).await {
                Ok(mut services) => {
                    discovered_services.append(&mut services);
                }
                Err(e) => {
                    warn!("Failed to discover from endpoint {}: {}", endpoint, e);
                }
            }
        }

        // Update registry with discovered services
        {
            let mut registry = self.capability_registry.write().await;
            for service in &discovered_services {
                registry.service_info.insert(service.name.clone(), service.clone());
                registry.last_updated.insert(service.name.clone(), chrono::Utc::now());

                // Index capabilities
                for capability in &service.capabilities {
                    registry
                        .capability_providers
                        .entry(capability.name.clone())
                        .or_insert_with(Vec::new)
                        .push(service.name.clone());
                }
            }
        } // Drop registry lock here

        info!("✅ Discovered {} services", discovered_services.len());
        Ok(discovered_services)
    }

    /// Find services that provide specific capabilities
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn find_capability_providers(
        &self,
        capability_type: &str,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        let registry = self.capability_registry.read().await;

        let providers =
            registry.capability_providers.get(capability_type).cloned().unwrap_or_default();

        let mut services = Vec::new();
        for provider in providers {
            if let Some(service) = registry.service_info.get(&provider) {
                services.push(service.clone());
            }
        }

        debug!("Found {} providers for capability '{}'", services.len(), capability_type);
        Ok(services)
    }

    /// Route a request to the best available service
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The required capability is not found
    /// - No services are available for the capability
    /// - All service requests fail
    pub async fn route_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, UniversalAdapterError> {
        // Extract required capability from request
        let capability_type = request
            .parameters
            .get("capability_type")
            .and_then(|v| v.as_str())
            .ok_or(UniversalAdapterError::MissingCapability)?;

        // Find providers
        let providers = self.find_capability_providers(capability_type).await?;
        if providers.is_empty() {
            return Err(UniversalAdapterError::NoProvidersAvailable(capability_type.to_string()));
        }

        // Select best provider (simple round-robin for now)
        let provider = &providers[0];

        // Route request to selected provider
        self.send_request_to_service(provider, request).await
    }

    /// Send request to a specific service
    async fn send_request_to_service(
        &self,
        service: &ServiceInfo,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, UniversalAdapterError> {
        let url = format!("{}/api/v1/{}", service.endpoint, request.action);

        // Create HTTP client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            UniversalAdapterError::NetworkError(format!("Failed to create HTTP client: {}", e))
        })?;

        // IpcHttpClient::post() returns RequestBuilder directly
        let request_builder = client.post(&url).await;

        let response = request_builder
            .json(&request)
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.is_success() {
            let universal_response: UniversalResponse = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string()))?;
            Ok(universal_response)
        } else {
            Err(UniversalAdapterError::ServiceError(format!("HTTP {}", response.status())))
        }
    }

    /// Discover services from a specific endpoint
    async fn discover_from_endpoint(
        &self,
        endpoint: &str,
    ) -> Result<Vec<ServiceInfo>, UniversalAdapterError> {
        debug!("Discovering services from endpoint: {}", endpoint);

        // Create HTTP client on-demand
        let client = songbird_http_client::IpcHttpClient::new().await.map_err(|e| {
            UniversalAdapterError::NetworkError(format!("Failed to create HTTP client: {}", e))
        })?;

        let response = client
            .get(endpoint)
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.is_success() {
            let services: Vec<ServiceInfo> = response
                .json()
                .await
                .map_err(|e| UniversalAdapterError::ParseError(e.to_string()))?;
            Ok(services)
        } else {
            Err(UniversalAdapterError::DiscoveryError(format!("HTTP {}", response.status())))
        }
    }

    /// Get current registry statistics
    pub async fn get_registry_stats(&self) -> RegistryStats {
        let registry = self.capability_registry.read().await;

        RegistryStats {
            total_services: registry.service_info.len(),
            total_capabilities: registry.capability_providers.len(),
            healthy_services: registry
                .service_info
                .values()
                .filter(|s| s.health == HealthStatus::Healthy)
                .count(),
        }
    }
}

impl Default for UnifiedUniversalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// **UNIFIED**: Error types for universal adapter operations
/// Errors that can occur during universal adapter operations
#[derive(Debug, thiserror::Error)]
pub enum UniversalAdapterError {
    /// Network communication error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Failed to parse response or configuration
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Service discovery failed
    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    /// Service-level error
    #[error("Service error: {0}")]
    ServiceError(String),

    /// Required capability is missing
    #[error("Missing required capability")]
    MissingCapability,

    /// No providers available for the requested capability
    #[error("No providers available for capability: {0}")]
    NoProvidersAvailable(String),
}

// Convert UniversalAdapterError to SongbirdError for test compatibility
impl From<UniversalAdapterError> for songbird_types::SongbirdError {
    fn from(err: UniversalAdapterError) -> Self {
        use UniversalAdapterError::{
            DiscoveryError, MissingCapability, NetworkError, NoProvidersAvailable, ParseError,
            ServiceError,
        };
        match err {
            NetworkError(msg) | ParseError(msg) | DiscoveryError(msg) | ServiceError(msg) => {
                Self::from(msg)
            }
            MissingCapability => Self::from("Required capability is missing"),
            NoProvidersAvailable(cap) => {
                Self::from(format!("No providers available for capability: {}", cap))
            }
        }
    }
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of registered services
    pub total_services: usize,
    /// Total number of available capabilities
    pub total_capabilities: usize,
    /// Number of healthy services
    pub healthy_services: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::{Capability, QoSMetrics, ResourceMetrics};
    use crate::types::{DiscoveredCapability, PrimalType, QosMetrics};
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::collections::HashMap;

    // Helper functions for test data creation
    fn create_test_qos_metrics() -> QosMetrics {
        QosMetrics {
            latency_ms: Some(10.0),
            throughput_ops_sec: Some(1000.0),
            availability: Some(0.99),
            reliability: Some(0.99),
        }
    }

    fn create_test_qos_metrics_capability() -> QoSMetrics {
        QoSMetrics {
            latency_ms: 10.0,
            throughput_ops_sec: 1000.0,
            availability: 0.99,
            reliability: 0.99,
            resource_usage: ResourceMetrics {
                cpu_percent: 50.0,
                memory_mb: 512,
                network_mbps: 100.0,
                storage_mb: 1024,
            },
        }
    }

    fn create_test_discovered_capability(
        name: &str,
        endpoint: &str,
        provider: &str,
    ) -> DiscoveredCapability {
        DiscoveredCapability {
            name: name.to_string(),
            version: "1.0".to_string(),
            description: format!("{name} capability"),
            provider: provider.to_string(),
            endpoint: endpoint.to_string(),
            qos_metrics: create_test_qos_metrics(),
            health_status: HealthStatus::Healthy,
        }
    }

    #[test]
    fn test_unified_adapter_creation() {
        let adapter = UnifiedUniversalAdapter::new();
        // Default config has 2 discovery endpoints
        assert_eq!(adapter.config.discovery_endpoints.len(), 2);
        assert!(adapter.config.auto_discovery);
    }

    #[test]
    fn test_capability_registry_default() {
        let registry = CapabilityRegistry::default();
        assert!(registry.service_capabilities.is_empty());
        assert!(registry.capability_providers.is_empty());
        assert!(registry.service_info.is_empty());
        assert!(registry.last_updated.is_empty());
    }

    #[test]
    fn test_unified_adapter_config_default() {
        let config = UnifiedAdapterConfig::default();
        // Default config includes 2 discovery endpoints
        assert_eq!(config.discovery_endpoints.len(), 2);
        assert!(config.auto_discovery);
        assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.health_check_interval, std::time::Duration::from_secs(60));
        assert_eq!(config.max_concurrent_requests, 100);
    }

    #[tokio::test]
    async fn test_discover_services_empty_endpoints() -> SongbirdResult<()> {
        let adapter = UnifiedUniversalAdapter::new();
        let services = adapter.discover_services().await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to discover services from empty registry: {}",
                e
            ))
        })?;
        assert!(services.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
        let adapter = UnifiedUniversalAdapter::new();
        let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to find capability providers from empty registry: {}",
                e
            ))
        })?;
        assert!(providers.is_empty());
        Ok(())
    }

    #[test]
    fn test_universal_adapter_error_display() {
        let err = UniversalAdapterError::MissingCapability;
        assert_eq!(err.to_string(), "Missing required capability");

        let err = UniversalAdapterError::NoProvidersAvailable("compute".to_string());
        assert_eq!(err.to_string(), "No providers available for capability: compute");

        let err = UniversalAdapterError::NetworkError("timeout".to_string());
        assert_eq!(err.to_string(), "Network error: timeout");

        let err = UniversalAdapterError::ParseError("invalid json".to_string());
        assert_eq!(err.to_string(), "Parse error: invalid json");

        let err = UniversalAdapterError::DiscoveryError("failed".to_string());
        assert_eq!(err.to_string(), "Discovery error: failed");

        let err = UniversalAdapterError::ServiceError("500".to_string());
        assert_eq!(err.to_string(), "Service error: 500");
    }

    #[test]
    fn test_registry_stats_creation() {
        let stats = RegistryStats {
            total_services: 5,
            total_capabilities: 10,
            healthy_services: 4,
        };

        assert_eq!(stats.total_services, 5);
        assert_eq!(stats.total_capabilities, 10);
        assert_eq!(stats.healthy_services, 4);
    }

    #[tokio::test]
    async fn test_concurrent_registry_access() {
        let adapter = Arc::new(UnifiedUniversalAdapter::new());
        let adapter1 = Arc::clone(&adapter);
        let adapter2 = Arc::clone(&adapter);

        // Spawn concurrent tasks
        let task1 = tokio::spawn(async move {
            let _ = adapter1.find_capability_providers("compute").await;
        });

        let task2 = tokio::spawn(async move {
            let _ = adapter2.find_capability_providers("storage").await;
        });

        // Both should complete without deadlock
        let _ = tokio::join!(task1, task2);
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    }

    #[tokio::test]
    async fn test_route_request_missing_capability_type() {
        let adapter = UnifiedUniversalAdapter::new();
        let request = UniversalRequest {
            request_id: "test-1".to_string(),
            source: "test-source".to_string(),
            target: "test-target".to_string(),
            action: "test".to_string(),
            parameters: HashMap::new(), // No capability_type
            security_context: None,
        };

        let result = adapter.route_request(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UniversalAdapterError::MissingCapability));
    }

    #[tokio::test]
    async fn test_route_request_no_providers() {
        let adapter = UnifiedUniversalAdapter::new();
        let mut parameters = HashMap::new();
        parameters.insert(
            "capability_type".to_string(),
            serde_json::Value::String("nonexistent".to_string()),
        );
        let request = UniversalRequest {
            request_id: "test-2".to_string(),
            source: "test-source".to_string(),
            target: "test-target".to_string(),
            action: "test".to_string(),
            parameters,
            security_context: None,
        };

        let result = adapter.route_request(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UniversalAdapterError::NoProvidersAvailable(cap) => {
                assert_eq!(cap, "nonexistent");
            }
            _ => panic!("Expected NoProvidersAvailable error"),
        }
    }

    #[tokio::test]
    async fn test_get_registry_stats_empty() {
        let adapter = UnifiedUniversalAdapter::new();
        let stats = adapter.get_registry_stats().await;

        assert_eq!(stats.total_services, 0);
        assert_eq!(stats.total_capabilities, 0);
        assert_eq!(stats.healthy_services, 0);
    }

    #[tokio::test]
    async fn test_get_registry_stats_with_services() {
        let adapter = UnifiedUniversalAdapter::new();

        // Manually populate registry for testing
        {
            let mut registry = adapter.capability_registry.write().await;

            let service1 = ServiceInfo {
                name: "service1".to_string(),
                primal_type: PrimalType::new("compute"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![create_test_discovered_capability(
                    "compute",
                    "http://localhost:8080",
                    "service1",
                )],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            let service2 = ServiceInfo {
                name: "service2".to_string(),
                primal_type: PrimalType::new("storage"),
                endpoint: "http://localhost:8081".to_string(),
                capabilities: vec![create_test_discovered_capability(
                    "storage",
                    "http://localhost:8081",
                    "service2",
                )],
                health: HealthStatus::Degraded,
                metadata: HashMap::new(),
            };

            registry.service_info.insert("service1".to_string(), service1);
            registry.service_info.insert("service2".to_string(), service2);
            registry
                .capability_providers
                .insert("compute".to_string(), vec!["service1".to_string()]);
            registry
                .capability_providers
                .insert("storage".to_string(), vec!["service2".to_string()]);
        }

        let stats = adapter.get_registry_stats().await;
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.total_capabilities, 2);
        assert_eq!(stats.healthy_services, 1); // Only service1 is healthy
    }

    #[tokio::test]
    async fn test_find_capability_providers_with_data() -> SongbirdResult<()> {
        let adapter = UnifiedUniversalAdapter::new();

        // Populate registry
        {
            let mut registry = adapter.capability_registry.write().await;

            let service = ServiceInfo {
                name: "compute-service".to_string(),
                primal_type: PrimalType::new("compute"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![create_test_discovered_capability(
                    "compute",
                    "http://localhost:8080",
                    "compute-service",
                )],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            registry.service_info.insert("compute-service".to_string(), service);
            registry
                .capability_providers
                .insert("compute".to_string(), vec!["compute-service".to_string()]);
        }

        let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to find capability providers with test data: {}",
                e
            ))
        })?;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "compute-service");
        Ok(())
    }

    #[tokio::test]
    async fn test_find_capability_providers_multiple() -> SongbirdResult<()> {
        let adapter = UnifiedUniversalAdapter::new();

        // Populate registry with multiple providers
        {
            let mut registry = adapter.capability_registry.write().await;

            let service1 = ServiceInfo {
                name: "compute-1".to_string(),
                primal_type: PrimalType::new("compute"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![create_test_discovered_capability(
                    "compute",
                    "http://localhost:8080",
                    "compute-1",
                )],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            let service2 = ServiceInfo {
                name: "compute-2".to_string(),
                primal_type: PrimalType::new("compute"),
                endpoint: "http://localhost:8081".to_string(),
                capabilities: vec![create_test_discovered_capability(
                    "compute",
                    "http://localhost:8081",
                    "compute-2",
                )],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            registry.service_info.insert("compute-1".to_string(), service1);
            registry.service_info.insert("compute-2".to_string(), service2);
            registry.capability_providers.insert(
                "compute".to_string(),
                vec!["compute-1".to_string(), "compute-2".to_string()],
            );
        }

        let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
            SongbirdError::configuration(format!(
                "Failed to find multiple capability providers: {}",
                e
            ))
        })?;
        assert_eq!(providers.len(), 2);
        Ok(())
    }

    #[test]
    fn test_adapter_with_custom_config() {
        let config = UnifiedAdapterConfig {
            discovery_timeout: std::time::Duration::from_secs(10),
            health_check_interval: std::time::Duration::from_secs(30),
            max_concurrent_requests: 50,
            auto_discovery: false,
            discovery_endpoints: vec!["http://custom:9000".to_string()],
        };

        let adapter = UnifiedUniversalAdapter::with_config(config);
        assert_eq!(adapter.config.discovery_timeout, std::time::Duration::from_secs(10));
        assert_eq!(adapter.config.max_concurrent_requests, 50);
        assert!(!adapter.config.auto_discovery);
        assert_eq!(adapter.config.discovery_endpoints.len(), 1);
    }

    #[test]
    fn test_service_connection_creation() {
        let connection = ServiceConnection {
            endpoint: "http://localhost:8080".to_string(),
            health: HealthStatus::Healthy,
            metrics: std::collections::HashMap::new(),
            last_contact: chrono::Utc::now(),
        };

        assert_eq!(connection.endpoint, "http://localhost:8080");
        assert_eq!(connection.health, HealthStatus::Healthy);
        assert!(connection.metrics.is_empty());
    }

    #[test]
    fn test_registry_stats_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let stats = RegistryStats {
            total_services: 10,
            total_capabilities: 20,
            healthy_services: 8,
        };

        // Test serialization
        let json = serde_json::to_string(&stats).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {}", e),
            debug_info: None,
        })?;
        assert!(json.contains("total_services"));
        assert!(json.contains("\"10\"") || json.contains("10"));

        // Test deserialization
        let deserialized: RegistryStats =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(deserialized.total_services, 10);
        assert_eq!(deserialized.total_capabilities, 20);
        assert_eq!(deserialized.healthy_services, 8);
        Ok(())
    }

    #[tokio::test]
    async fn test_capability_registry_indexing() -> SongbirdResult<()> {
        let adapter = UnifiedUniversalAdapter::new();

        // Manually test registry capability indexing logic
        {
            let mut registry = adapter.capability_registry.write().await;

            let capabilities = vec![
                create_test_discovered_capability(
                    "compute",
                    "http://localhost:8080",
                    "multi-cap-service",
                ),
                create_test_discovered_capability(
                    "storage",
                    "http://localhost:8080",
                    "multi-cap-service",
                ),
            ];

            let service = ServiceInfo {
                name: "multi-cap-service".to_string(),
                primal_type: PrimalType::new("generic"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: capabilities.clone(),
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };

            registry.service_info.insert(service.name.clone(), service.clone());
            // Convert DiscoveredCapability to Capability for registry
            let simple_caps: Vec<Capability> = capabilities
                .iter()
                .map(|dc| Capability {
                    capability_type: dc.name.clone(),
                    name: dc.name.clone(),
                    version: dc.version.clone(),
                    parameters: HashMap::new(),
                    qos_metrics: create_test_qos_metrics_capability(),
                    available: true,
                })
                .collect();
            registry.service_capabilities.insert(service.name.clone(), simple_caps);

            // Index capabilities
            for capability in &service.capabilities {
                registry
                    .capability_providers
                    .entry(capability.name.clone())
                    .or_insert_with(Vec::new)
                    .push(service.name.clone());
            }
        }

        // Verify indexing worked
        let compute_providers =
            adapter.find_capability_providers("compute").await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "Failed to find compute providers in capability registry: {}",
                    e
                ))
            })?;
        let storage_providers =
            adapter.find_capability_providers("storage").await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "Failed to find storage providers in capability registry: {}",
                    e
                ))
            })?;

        assert_eq!(compute_providers.len(), 1);
        assert_eq!(storage_providers.len(), 1);
        assert_eq!(compute_providers[0].name, "multi-cap-service");
        assert_eq!(storage_providers[0].name, "multi-cap-service");
        Ok(())
    }

    #[test]
    fn test_error_types_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UniversalAdapterError>();
    }

    #[tokio::test]
    async fn test_adapter_is_clonable() {
        let adapter1 = UnifiedUniversalAdapter::new();
        let adapter2 = adapter1.clone();

        // Both should work independently
        let stats1 = adapter1.get_registry_stats().await;
        let stats2 = adapter2.get_registry_stats().await;

        assert_eq!(stats1.total_services, stats2.total_services);
    }

    #[tokio::test]
    async fn test_concurrent_write_operations() {
        let adapter = Arc::new(UnifiedUniversalAdapter::new());
        let adapter1 = Arc::clone(&adapter);
        let adapter2 = Arc::clone(&adapter);

        // Concurrent writes to registry
        let task1 = tokio::spawn(async move {
            let mut registry = adapter1.capability_registry.write().await;
            let service = ServiceInfo {
                name: "service1".to_string(),
                primal_type: PrimalType::new("generic"),
                endpoint: "http://localhost:8080".to_string(),
                capabilities: vec![],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };
            registry.service_info.insert("service1".to_string(), service);
        });

        let task2 = tokio::spawn(async move {
            let mut registry = adapter2.capability_registry.write().await;
            let service = ServiceInfo {
                name: "service2".to_string(),
                primal_type: PrimalType::new("generic"),
                endpoint: "http://localhost:8081".to_string(),
                capabilities: vec![],
                health: HealthStatus::Healthy,
                metadata: HashMap::new(),
            };
            registry.service_info.insert("service2".to_string(), service);
        });

        // Both should complete without deadlock or data corruption
        let _ = tokio::join!(task1, task2);

        let stats = adapter.get_registry_stats().await;
        assert_eq!(stats.total_services, 2);
    }
}
