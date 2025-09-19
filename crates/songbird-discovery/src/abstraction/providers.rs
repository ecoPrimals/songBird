//! # Discovery Providers
//!
//! Agnostic provider interface that any discovery system can implement

use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;

use crate::abstraction::capabilities::DiscoveryCapability;
use crate::traits::discovery::ServiceHealthStatus;
use crate::traits::{ServiceEvent, ServiceInfo, ServiceQuery};
use songbird_errors::SongbirdResult; type Result<T> = SongbirdResult<T>;

/// Provider configuration that's completely agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Configuration parameters (completely flexible)
    pub parameters: HashMap<String, serde_json::Value>,
    /// Environment variables to use
    pub environment: HashMap<String, String>,
    /// Connection timeout
    pub timeout_ms: Option<u64>,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

/// Provider metadata
#[derive(Debug, Clone)]
pub struct ProviderMetadata {
    /// Unique provider ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Provider version
    pub version: String,
    /// Capabilities this provider supports
    pub capabilities: Vec<DiscoveryCapability>,
    /// Provider-specific metadata
    pub metadata: HashMap<String, String>,
    /// Health status
    pub healthy: bool,
    /// Load score (lower is better)
    pub load_score: f32,
}

impl Default for ProviderMetadata {
    fn default() -> Self {
        Self {
            id: "unknown".to_string(),
            name: "Unknown Provider".to_string(),
            version: "0.0.0".to_string(),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            healthy: false,
            load_score: 1.0,
        }
    }
}

/// Agnostic discovery provider trait
#[async_trait]
pub trait DiscoveryProvider: Send + Sync {
    /// Get provider metadata
    fn metadata(&self) -> &ProviderMetadata;

    /// Initialize the provider with configuration
    async fn initialize(&mut self, config: ProviderConfig) -> Result<()>;

    /// Shutdown the provider gracefully
    async fn shutdown(&mut self) -> Result<()>;

    /// Check if provider is healthy
    async fn health_check(&self) -> Result<bool>;

    // === Service Operations ===

    /// Register a service (if provider supports ServiceRegistration)
    async fn register_service(&self, _service: ServiceInfo) -> Result<()> {
        // Placeholder implementation - real providers would implement actual registration
        Ok(())
    }

    /// Unregister a service (if provider supports ServiceUnregistration)
    async fn unregister_service(&self, service_id: &str) -> Result<()> {
        let _ = service_id;
        Err(songbird_errors::SongbirdError::operation_error(
            "Service unregistration not supported by this provider",
        ))
    }

    /// Discover services (if provider supports ServiceDiscovery)
    async fn discover_services(&self, query: ServiceQuery) -> Result<Vec<ServiceInfo>> {
        let _ = query;
        Err(songbird_errors::SongbirdError::operation_error(
            "Service discovery not supported by this provider",
        ))
    }

    /// Watch for service changes (if provider supports ServiceWatching)
    async fn watch_services(
        &self,
        query: ServiceQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = ServiceEvent> + Send>>> {
        let _ = query;
        Err(songbird_errors::SongbirdError::operation_error(
            "Service watching not supported by this provider",
        ))
    }

    /// Update service health (if provider supports HealthChecking)
    async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> Result<()> {
        let _ = (service_id, health);
        Err(songbird_errors::SongbirdError::operation_error(
            "Health checking not supported by this provider",
        ))
    }

    /// Update service metadata (if provider supports MetadataUpdating)
    async fn update_service_metadata(
        &self,
        service_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let _ = (service_id, metadata);
        Err(songbird_errors::SongbirdError::operation_error(
            "Metadata updating not supported by this provider",
        ))
    }

    /// List all services (if provider supports ServiceListing)
    async fn list_all_services(&self) -> Result<Vec<ServiceInfo>> {
        Err(songbird_errors::SongbirdError::operation_error(
            "Service listing not supported by this provider",
        ))
    }

    /// Check if service exists (if provider supports ServiceExistence)
    async fn service_exists(&self, service_id: &str) -> Result<bool> {
        let _ = service_id;
        Err(songbird_errors::SongbirdError::operation_error(
            "Service existence checking not supported by this provider",
        ))
    }

    /// Get service metrics (if provider supports ServiceMetrics)
    async fn get_service_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        let _ = service_id;
        Err(songbird_errors::SongbirdError::operation_error(
            "Service metrics not supported by this provider",
        ))
    }

    /// Resolve service dependencies (if provider supports DependencyResolution)
    async fn resolve_dependencies(&self, service_id: &str) -> Result<Vec<ServiceInfo>> {
        let _ = service_id;
        Err(songbird_errors::SongbirdError::operation_error(
            "Dependency resolution not supported by this provider",
        ))
    }

    /// Get load balancing hints (if provider supports LoadBalancingHints)
    async fn get_load_balancing_hints(&self, service_name: &str) -> Result<LoadBalancingHints> {
        let _ = service_name;
        Err(songbird_errors::SongbirdError::operation_error(
            "Load balancing hints not supported by this provider",
        ))
    }

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;
}

/// Service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub service_id: String,
    pub request_count: u64,
    pub error_count: u64,
    pub average_response_time_ms: f64,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub custom_metrics: HashMap<String, f64>,
}

/// Load balancing hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingHints {
    pub service_name: String,
    pub preferred_instances: Vec<String>,
    pub weights: HashMap<String, f32>,
    pub health_scores: HashMap<String, f32>,
    pub locality_preferences: Vec<String>,
}

/// Provider factory trait for creating providers dynamically
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    /// Get the provider type this factory creates
    fn provider_type(&self) -> &str;

    /// Create a new provider instance
    async fn create_provider(&self, config: ProviderConfig) -> Result<Box<dyn DiscoveryProvider>>;

    /// Validate configuration for this provider type
    fn validate_config(&self, config: &ProviderConfig) -> Result<()>;

    /// Get default configuration template
    fn default_config(&self, id: String, name: String) -> ProviderConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        metadata: ProviderMetadata,
    }

    #[async_trait]
    impl DiscoveryProvider for MockProvider {
        fn metadata(&self) -> &ProviderMetadata {
            &self.metadata
        }

        async fn initialize(&mut self, _config: ProviderConfig) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(self.metadata.healthy)
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[tokio::test]
    async fn test_provider_metadata() {
        let metadata = ProviderMetadata {
            id: "test-provider".to_string(),
            name: "Test Provider".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![DiscoveryCapability::ServiceDiscovery],
            metadata: HashMap::new(),
            healthy: true,
            load_score: 0.5,
        };

        let provider = MockProvider { metadata };
        assert_eq!(provider.metadata().id, "test-provider");
        assert!(provider.health_check().await.unwrap());
    }
}
