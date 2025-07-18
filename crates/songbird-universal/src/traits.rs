//! Universal traits for ecosystem integration

use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    DiscoveryError, LoadBalancingError, PrimalType, ProtocolCharacteristics, ProtocolError,
    RegistryError, ServiceCapability, ServiceError, ServiceHealth, UniversalEvent,
    UniversalRequest, UniversalResponse,
};

/// Universal trait that ALL services must implement for Songbird integration
#[async_trait]
pub trait UniversalServiceProvider: Send + Sync {
    /// Service identification
    fn service_id(&self) -> &str;
    fn primal_type(&self) -> PrimalType;
    fn instance_id(&self) -> &str;

    /// Capabilities (extensible)
    fn capabilities(&self) -> Vec<ServiceCapability>;

    /// Health check (universal)
    async fn health_check(&self) -> ServiceHealth;

    /// Handle requests (completely agnostic)
    async fn handle_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, ServiceError>;

    /// Lifecycle management
    async fn initialize(&mut self, config: serde_json::Value) -> Result<(), ServiceError>;
    async fn shutdown(&mut self) -> Result<(), ServiceError>;

    /// Optional: Advanced capabilities
    async fn metrics(&self) -> Result<serde_json::Value, ServiceError> {
        Ok(serde_json::json!({}))
    }

    async fn configuration(&self) -> Result<serde_json::Value, ServiceError> {
        Ok(serde_json::json!({}))
    }
}

/// Universal trait for ecosystem integration
#[async_trait]
pub trait EcosystemIntegration: Send + Sync {
    /// Register service with Songbird
    async fn register_with_songbird(&self) -> Result<String, ServiceError>;

    /// Handle incoming requests from other services
    async fn handle_ecosystem_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, ServiceError>;

    /// Report health status to Songbird
    async fn report_health(&self, health: ServiceHealth) -> Result<(), ServiceError>;

    /// Update service capabilities
    async fn update_capabilities(
        &self,
        capabilities: Vec<ServiceCapability>,
    ) -> Result<(), ServiceError>;

    /// Deregister from ecosystem
    async fn deregister(&self) -> Result<(), ServiceError>;
}

/// Universal protocol handler trait
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Protocol name (e.g., "http", "websocket", "tarpc", "grpc")
    fn protocol_name(&self) -> &str;

    /// Handle universal request in protocol-specific way
    async fn handle_request(
        &self,
        request: UniversalRequest,
        endpoint: &str,
    ) -> Result<UniversalResponse, ProtocolError>;

    /// Check if protocol can handle specific service type
    fn can_handle_service(&self, service_type: &str) -> bool;

    /// Get optimal protocol characteristics
    fn characteristics(&self) -> ProtocolCharacteristics;
}

/// Universal discovery backend trait
#[async_trait]
pub trait DiscoveryBackend: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Discover services
    async fn discover_services(
        &self,
        filters: &crate::DiscoveryFilters,
    ) -> Result<Vec<crate::ServiceInfo>, DiscoveryError>;

    /// Watch for service changes
    async fn watch_services(
        &self,
        callback: Box<dyn Fn(crate::ServiceEvent) + Send + Sync>,
    ) -> Result<(), DiscoveryError>;

    /// Notify service registered
    async fn notify_service_registered(&self, service_id: &str) -> Result<(), DiscoveryError>;

    /// Notify service deregistered
    async fn notify_service_deregistered(&self, service_id: &str) -> Result<(), DiscoveryError>;
}

/// Universal load balancing strategy trait
#[async_trait]
pub trait LoadBalancingStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;

    /// Select service instance from available instances
    async fn select_instance(
        &self,
        instances: &[crate::RegisteredService],
        request: &UniversalRequest,
    ) -> Result<crate::RegisteredService, LoadBalancingError>;

    /// Update instance weights (for weighted strategies)
    async fn update_weights(
        &self,
        _weights: HashMap<String, f64>,
    ) -> Result<(), LoadBalancingError> {
        // Default implementation - no-op for strategies that don't use weights
        Ok(())
    }

    /// Get strategy configuration
    fn configuration(&self) -> serde_json::Value;

    /// Handle strategy-specific metrics
    async fn collect_metrics(&self) -> Result<serde_json::Value, LoadBalancingError> {
        Ok(serde_json::json!({}))
    }
}

/// Universal service store trait
#[async_trait]
pub trait UniversalServiceStore: Send + Sync {
    async fn store_service(&self, service: crate::RegisteredService) -> Result<(), RegistryError>;
    async fn get_service(
        &self,
        service_id: &str,
    ) -> Result<Option<crate::RegisteredService>, RegistryError>;
    async fn get_all_services(&self) -> Result<Vec<crate::RegisteredService>, RegistryError>;
    async fn get_services_by_primal(
        &self,
        primal_type: PrimalType,
    ) -> Result<Vec<crate::RegisteredService>, RegistryError>;
    async fn update_service_health(
        &self,
        service_id: &str,
        health: crate::HealthStatus,
    ) -> Result<(), RegistryError>;
    async fn update_service_capabilities(
        &self,
        service_id: &str,
        capabilities: Vec<ServiceCapability>,
    ) -> Result<(), RegistryError>;
    async fn remove_service(&self, service_id: &str) -> Result<(), RegistryError>;
    async fn service_exists(&self, service_id: &str) -> Result<bool, RegistryError>;
    async fn count_services(&self) -> Result<u64, RegistryError>;
    async fn count_services_by_primal(&self) -> Result<HashMap<PrimalType, u64>, RegistryError>;
    async fn count_services_by_health(
        &self,
    ) -> Result<HashMap<crate::HealthStatus, u64>, RegistryError>;
    async fn cleanup_expired_services(
        &self,
        expiry_duration: std::time::Duration,
    ) -> Result<Vec<String>, RegistryError>;
}

/// Universal health checker trait
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// Check health of a service
    async fn check_health(&self, service_id: &str) -> Result<ServiceHealth, ServiceError>;

    /// Get check interval
    fn check_interval(&self) -> std::time::Duration;

    /// Get timeout for health checks
    fn timeout(&self) -> std::time::Duration;
}

/// Universal metrics backend trait
#[async_trait]
pub trait MetricsBackend: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Record metric
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: &HashMap<String, String>,
    ) -> Result<(), crate::MetricsError>;

    /// Increment counter
    async fn increment_counter(
        &self,
        name: &str,
        labels: &HashMap<String, String>,
    ) -> Result<(), crate::MetricsError>;

    /// Record histogram
    async fn record_histogram(
        &self,
        name: &str,
        value: f64,
        labels: &HashMap<String, String>,
    ) -> Result<(), crate::MetricsError>;
}

/// Universal security provider trait
#[async_trait]
pub trait SecurityProvider: Send + Sync {
    /// Authenticate request
    async fn authenticate(
        &self,
        request: &UniversalRequest,
    ) -> Result<crate::SecurityContext, crate::SecurityError>;

    /// Authorize request
    async fn authorize(
        &self,
        context: &crate::SecurityContext,
        operation: &str,
    ) -> Result<bool, crate::SecurityError>;

    /// Encrypt data
    async fn encrypt(
        &self,
        data: &[u8],
        context: &crate::SecurityContext,
    ) -> Result<Vec<u8>, crate::SecurityError>;

    /// Decrypt data
    async fn decrypt(
        &self,
        data: &[u8],
        context: &crate::SecurityContext,
    ) -> Result<Vec<u8>, crate::SecurityError>;
}

/// Universal event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle universal event
    async fn handle_event(&self, event: UniversalEvent) -> Result<(), crate::EventError>;

    /// Get event types this handler can process
    fn supported_event_types(&self) -> Vec<String>;
}

/// Universal configuration provider trait
#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    /// Get configuration for a service
    async fn get_configuration(
        &self,
        service_id: &str,
    ) -> Result<serde_json::Value, crate::ConfigError>;

    /// Update configuration for a service
    async fn update_configuration(
        &self,
        service_id: &str,
        config: serde_json::Value,
    ) -> Result<(), crate::ConfigError>;

    /// Watch for configuration changes
    async fn watch_configuration(
        &self,
        service_id: &str,
        callback: Box<dyn Fn(serde_json::Value) + Send + Sync>,
    ) -> Result<(), crate::ConfigError>;
}
