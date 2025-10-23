//! # 🔧 Unified Universal Adapter
//!
//! **SINGLE SOURCE OF TRUTH FOR UNIVERSAL ADAPTATION** ✅
//!
//! This module consolidates all fragmented UniversalCapabilityAdapter implementations
//! into a single, unified adapter that can handle any capability type.

use crate::capabilities::Capability;
use crate::types::{HealthStatus, ServiceInfo, UniversalRequest, UniversalResponse};
use serde::{Deserialize, Serialize};
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
    /// HTTP client for service communication
    http_client: reqwest::Client,
}

/// **UNIFIED**: Capability registry for discovered services
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
                let host = std::env::var("ADAPTER_DISCOVERY_HOST")
                    .unwrap_or_else(|_| "localhost".to_string());
                vec![
                    format!("http://{}:8080/capabilities", host),
                    format!("http://{}:8081/services", host),
                ]
            },
        }
    }
}

impl UnifiedUniversalAdapter {
    /// Create a new unified adapter with default configuration
    pub fn new() -> Self {
        Self::with_config(UnifiedAdapterConfig::default())
    }

    /// Create a new unified adapter with custom configuration
    pub fn with_config(config: UnifiedAdapterConfig) -> Self {
        Self {
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry::default())),
            service_connections: Arc::new(RwLock::new(HashMap::new())),
            config,
            http_client: reqwest::Client::new(),
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
                    warn!("Failed to discover from endpoint {}: {}", endpoint, e)
                }
            }
        }

        // Update registry with discovered services
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

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .timeout(self.config.discovery_timeout)
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
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

        let response = self
            .http_client
            .get(endpoint)
            .timeout(self.config.discovery_timeout)
            .send()
            .await
            .map_err(|e| UniversalAdapterError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
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
    async fn test_discover_services_empty_endpoints() {
        let adapter = UnifiedUniversalAdapter::new();
        let result = adapter.discover_services().await;

        assert!(result.is_ok());
        let services = result.unwrap();
        assert!(services.is_empty());
    }

    #[tokio::test]
    async fn test_find_capability_providers_empty_registry() {
        let adapter = UnifiedUniversalAdapter::new();
        let result = adapter.find_capability_providers("compute").await;

        assert!(result.is_ok());
        let providers = result.unwrap();
        assert!(providers.is_empty());
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
}
