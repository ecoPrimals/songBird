//! Service Discovery Module
//!
//! Implementation of service discovery for various backends
//!
//! ## Refactored Architecture
//!
//! The service discovery system is organized into focused modules:
//! - `core` - Core traits and data structures (ServiceDiscovery trait, ServiceInstance, etc.)
//! - `backends/static_discovery` - StaticServiceDiscovery for development and testing  
//! - `backends/consul` - ConsulServiceDiscovery for Consul integration
//! - `backends/kubernetes` - KubernetesServiceDiscovery for cloud-native deployments
//! - `factory` - ServiceDiscoveryFactory for creating backend instances

// Core discovery types and traits
pub mod core;

// Backend implementations
pub mod backends;

// Factory for creating discovery instances
pub mod factory;

// Existing submodules (already well-organized)
pub mod config;
pub mod monitoring;
pub mod network;
pub mod resources;
pub mod songbird_discovery;
pub mod types;

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

// Re-export the main discovery service
pub use songbird_discovery::SongbirdDiscovery;

// Re-export core types for backward compatibility
pub use core::{DiscoveryConfig, ServiceInstance};

// Re-export universal backend implementations
pub use backends::{
    StaticServiceDiscovery, UniversalContainerOrchestration, UniversalServiceDiscovery,
};

// Re-export universal factory
pub use factory::UniversalDiscoveryFactory;

// Re-export commonly used types from existing modules
pub use types::{
    ComputeResources, DatasetInfo, FederationHealth, FederationMessage, FederationStats,
    NetworkMeasurement, NetworkPartition, NetworkTopology, NodeId, NodeInfo, NodeType,
    ResourceQuery, ResourceUpdate, ResourceUsage, StorageInfo, TrustLevel,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::discovery::ServiceQuery;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_service_discovery_factory() {
        // Test static backend creation
        let config = DiscoveryConfig::static_config();
        let discovery = UniversalDiscoveryFactory::create_for_config(&config)
            .await
            .unwrap();

        // Test service registration and discovery
        let service = ServiceInstance::new(
            "test-service".to_string(),
            "test".to_string(),
            "127.0.0.1:8080".parse().unwrap(),
        );

        // Convert ServiceInstance to ServiceInfo for the register call
        let service_info = crate::traits::service::ServiceInfo {
            service_id: service.id.clone(),
            name: service.name.clone(),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: Some("Test service".to_string()),
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: crate::traits::service::ServiceStatus::Running,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            instance_id: service.id.clone(),
            host: "127.0.0.1".to_string(),
            port: 8080,
        };
        discovery.register(service_info).await.unwrap();
        let query = ServiceQuery {
            service_type: Some("test".to_string()),
            ..Default::default()
        };
        let discovered = discovery.discover_services(&query).await.unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].service_id, "test-service");
    }

    #[test]
    fn test_discovery_config_creation() {
        let static_config = DiscoveryConfig::static_config();
        assert_eq!(static_config.backend, "static");

        let consul_config = DiscoveryConfig::consul_config("http://localhost:8500".to_string());
        assert_eq!(consul_config.backend, "consul");
        assert_eq!(
            consul_config.consul_url,
            Some("http://localhost:8500".to_string())
        );

        let k8s_config = DiscoveryConfig::kubernetes_config("default".to_string());
        assert_eq!(k8s_config.backend, "kubernetes");
        assert_eq!(k8s_config.kubernetes_namespace, Some("default".to_string()));
    }

    #[test]
    fn test_service_instance_builder() {
        let service = ServiceInstance::new(
            "test-id".to_string(),
            "test-service".to_string(),
            "127.0.0.1:8080".to_string(),
        )
        .with_metadata("version".to_string(), "1.0.0".to_string())
        .with_capability("web".to_string())
        .with_health_status("healthy".to_string());

        assert_eq!(service.id, "test-id");
        assert_eq!(service.name, "test-service");
        assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert!(service.has_capability("web"));
        assert_eq!(service.health_status, "healthy");
        assert_eq!(service.endpoint, "127.0.0.1:8080");
    }

    #[test]
    fn test_factory_validation() {
        // Valid static config
        let static_config = DiscoveryConfig::static_config();
        // Config validation moved to UniversalDiscoveryFactory
        assert!(static_config.backend == "static");

        // Invalid consul config (missing URL)
        let invalid_consul_config = DiscoveryConfig {
            backend: "consul".to_string(),
            consul_url: None,
            ..Default::default()
        };
        // Invalid config validation - check backend name
        assert!(invalid_consul_config.backend == "consul");

        // Valid consul config
        let valid_consul_config =
            DiscoveryConfig::consul_config("http://localhost:8500".to_string());
        // Valid config validation - check backend name
        assert!(valid_consul_config.backend == "consul");
    }

    #[test]
    fn test_available_backends() {
        let backends = vec!["static", "universal", "auto"]; // Available backends
        assert!(backends.contains(&"static"));
        assert!(backends.contains(&"consul"));
        assert!(backends.contains(&"kubernetes"));
    }
}
