//! # Discovery Core
//!
//! Core traits and types for service discovery with sovereignty compliance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import universal capability adapter instead of hardcoded backends

/// Configuration for service discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryConfig {
    /// Enable network scanning
    /// Enable Network Scan field
    pub enable_network_scan: bool,
    /// Enable environment-based discovery
    /// Enable Environment Discovery field
    pub enable_environment_discovery: bool,
    /// Enable container discovery
    /// Enable Container Discovery field
    pub enable_container_discovery: bool,
    /// Discovery timeout in seconds
    /// Timeout Seconds field
    pub timeout_seconds: u64,
    /// Health check interval in seconds
    /// Health Check Interval field
    pub health_check_interval: u64,
    /// Discovery backend type
    /// Backend field
    pub backend: String,
    /// Consul URL for consul backend
    /// Consul Url field
    pub consul_url: Option<String>,
    /// Kubernetes namespace for kubernetes backend
    /// Kubernetes Namespace field
    pub kubernetes_namespace: Option<String>,
}

impl Default for CanonicalDiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_network_scan: true,
            enable_environment_discovery: true,
            enable_container_discovery: false,
            timeout_seconds: 30,
            health_check_interval: 60,
            backend: "static".to_string(),
            consul_url: None,
            kubernetes_namespace: None,
        }
    }
}

impl CanonicalDiscoveryConfig {
    /// Create static backend configuration
    #[must_use]
    pub fn static_config() -> Self {
        Self {
            backend: "static".to_string(),
            ..Default::default()
        }
    }

    /// Create consul backend configuration
    #[must_use]
    pub fn consul_config(consul_url: String) -> Self {
        Self {
            backend: "service_discovery".to_string(),
            consul_url: Some(consul_url),
            ..Default::default()
        }
    }

    /// Create kubernetes backend configuration
    #[must_use]
    pub fn kubernetes_config(namespace: String) -> Self {
        Self {
            backend: "container_orchestration".to_string(),
            kubernetes_namespace: Some(namespace),
            ..Default::default()
        }
    }
}

/// Service instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// Unique service identifier
    pub id: String,
    /// Human-readable service name
    /// Name identifier
    pub name: String,
    /// Service endpoint URL
    pub endpoint: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Current health status
    pub health_status: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ServiceInstance {
    /// Create a new service instance
    #[must_use]
    pub fn new(id: String, name: String, endpoint: String) -> Self {
        Self {
            id,
            name,
            endpoint,
            capabilities: Vec::new(),
            health_status: "unknown".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add a capability to the service
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }
    /// Set health status
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_health_status(mut self, status: String) -> Self {
        self.health_status = status;
        self
    }

    /// Add metadata
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    /// Check if service has a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if service is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.health_status == "healthy"
    }
}

//  trait is now imported directly from traits module where needed

/*
// REPLACED: Local trait replaced with canonical trait from traits module
// This code has been moved to the canonical traits module
*/
pub struct ComputeResources {
    /// Cpu Cores field
    pub cpu_cores: u32,
    /// Memory Gb field
    pub memory_gb: u32,
    /// Gpu Count field
    pub gpu_count: u32,
}

/// Dataset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Name identifier
    pub name: String,
    /// Size Bytes field
    pub size_bytes: u64,
    /// Format field
    pub format: String,
}

/// Federation health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationHealth {
    /// Current status of the operation or entity
    pub status: String,
    /// Last Check field
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Federation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    /// Id field
    pub id: String,
    /// Content field
    pub content: String,
    /// Timestamp when this was created or last updated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Federation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStats {
    /// Total Nodes field
    pub total_nodes: u32,
    /// Active Nodes field
    pub active_nodes: u32,
    /// Total Messages field
    pub total_messages: u64,
}

/// Interaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct InteractionResult {
    /// Success field
    pub success: bool,
    /// Message field
    pub message: String,
}

/// Local node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNode {
    /// Id field
    pub id: String,
    /// Name identifier
    pub name: String,
    /// Endpoint field
    pub endpoint: String,
}

/// Network measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMeasurement {
    /// Latency Ms field
    pub latency_ms: u32,
    /// Bandwidth Mbps field
    pub bandwidth_mbps: u32,
}

/// Network partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPartition {
    /// Id field
    pub id: String,
    /// Nodes field
    pub nodes: Vec<String>,
}
/// Network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Nodes field
    pub nodes: Vec<LocalNode>,
    pub connections: Vec<(String, String)>,
}
/// Node identifier;
pub type NodeId = String;

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Id field
    pub id: NodeId,
    /// Name identifier
    pub name: String,
    /// Node Type field
    pub node_type: NodeType,
}

/// Node type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Security, Security,
    Compute,
    Storage,
    Gateway,
    Unknown,
}

/// Resource query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    /// Resource Type field
    pub resource_type: String,
    pub filters: HashMap<String, String>,
}

/// Resource update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdate {
    /// Resource Id field
    pub resource_id: String,
    pub updates: HashMap<String, String>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct ResourceUsage {
    /// Cpu Percent field
    pub cpu_percent: f32,
    /// Memory Percent field
    pub memory_percent: f32,
    /// Disk Percent field
    pub disk_percent: f32,
}

/// Storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct StorageInfo {
    /// Total Bytes field
    pub total_bytes: u64,
    /// Used Bytes field
    pub used_bytes: u64,
    /// Available Bytes field
    pub available_bytes: u64,
}

/// Trust level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    High,
    Medium,
    Low,
    Unknown,
}

// Type aliases for backward compatibility
pub type DiscoveryConfig = CanonicalDiscoveryConfig;

// Re-export the ServiceDiscovery trait from our traits module
pub use crate::traits::discovery::ServiceDiscovery;

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    // ================ CanonicalDiscoveryConfig Tests ================

    #[test]
    fn test_canonical_discovery_config_default() {
        let config = CanonicalDiscoveryConfig::default();

        assert!(config.enable_network_scan);
        assert!(config.enable_environment_discovery);
        assert!(!config.enable_container_discovery);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.health_check_interval, 60);
        assert_eq!(config.backend, "static");
        assert!(config.consul_url.is_none());
        assert!(config.kubernetes_namespace.is_none());
    }

    #[test]
    fn test_canonical_discovery_config_static() {
        let config = CanonicalDiscoveryConfig::static_config();

        assert_eq!(config.backend, "static");
        assert!(config.enable_network_scan);
        assert!(config.enable_environment_discovery);
    }

    #[test]
    fn test_canonical_discovery_config_consul() {
        let consul_url = "http://localhost:8500".to_string();
        let config = CanonicalDiscoveryConfig::consul_config(consul_url.clone());

        assert_eq!(config.backend, "service_discovery");
        assert_eq!(config.consul_url, Some(consul_url));
        assert!(config.kubernetes_namespace.is_none());
    }

    #[test]
    fn test_canonical_discovery_config_kubernetes() {
        let namespace = "production".to_string();
        let config = CanonicalDiscoveryConfig::kubernetes_config(namespace.clone());

        assert_eq!(config.backend, "container_orchestration");
        assert_eq!(config.kubernetes_namespace, Some(namespace));
        assert!(config.consul_url.is_none());
    }

    #[test]
    fn test_canonical_discovery_config_clone() {
        let config = CanonicalDiscoveryConfig::default();
        let cloned = config.clone();

        assert_eq!(config.backend, cloned.backend);
        assert_eq!(config.timeout_seconds, cloned.timeout_seconds);
    }

    #[test]
    fn test_canonical_discovery_config_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let config = CanonicalDiscoveryConfig::consul_config("http://consul:8500".to_string());
        let json = serde_json::to_string(&config)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
        let deserialized: CanonicalDiscoveryConfig =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to deserialize: {e}"),
                debug_info: None,
            })?;

        assert_eq!(config.backend, deserialized.backend);
        assert_eq!(config.consul_url, deserialized.consul_url);
        Ok(())
    }

    // ================ ServiceInstance Tests ================

    #[test]
    fn test_service_instance_new() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test-service".to_string(),
            "http://localhost:8080".to_string(),
        );

        assert_eq!(service.id, "svc-001");
        assert_eq!(service.name, "test-service");
        assert_eq!(service.endpoint, "http://localhost:8080");
        assert!(service.capabilities.is_empty());
        assert_eq!(service.health_status, "unknown");
        assert!(service.metadata.is_empty());
    }

    #[test]
    fn test_service_instance_with_capability() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "compute-service".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_capability("compute".to_string())
        .with_capability("gpu".to_string());

        assert_eq!(service.capabilities.len(), 2);
        assert!(service.has_capability("compute"));
        assert!(service.has_capability("gpu"));
        assert!(!service.has_capability("storage"));
    }

    #[test]
    fn test_service_instance_with_health_status() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_health_status("healthy".to_string());

        assert_eq!(service.health_status, "healthy");
        assert!(service.is_healthy());
    }

    #[test]
    fn test_service_instance_with_metadata() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_metadata("region".to_string(), "us-west".to_string())
        .with_metadata("tier".to_string(), "premium".to_string());

        assert_eq!(service.metadata.len(), 2);
        assert_eq!(service.metadata.get("region"), Some(&"us-west".to_string()));
        assert_eq!(service.metadata.get("tier"), Some(&"premium".to_string()));
    }

    #[test]
    fn test_service_instance_has_capability() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_capability("security".to_string());

        assert!(service.has_capability("security"));
        assert!(!service.has_capability("compute"));
    }

    #[test]
    fn test_service_instance_is_healthy() {
        let healthy = ServiceInstance::new(
            "svc-001".to_string(),
            "test".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_health_status("healthy".to_string());

        let unhealthy = ServiceInstance::new(
            "svc-002".to_string(),
            "test".to_string(),
            "http://localhost:8081".to_string(),
        )
        .with_health_status("unhealthy".to_string());

        assert!(healthy.is_healthy());
        assert!(!unhealthy.is_healthy());
    }

    #[test]
    fn test_service_instance_builder_pattern() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "full-service".to_string(),
            "http://localhost:9000".to_string(),
        )
        .with_capability("compute".to_string())
        .with_capability("storage".to_string())
        .with_health_status("healthy".to_string())
        .with_metadata("version".to_string(), "1.0.0".to_string())
        .with_metadata("region".to_string(), "us-east".to_string());

        assert_eq!(service.capabilities.len(), 2);
        assert_eq!(service.metadata.len(), 2);
        assert!(service.is_healthy());
    }

    #[test]
    fn test_service_instance_clone() {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_capability("compute".to_string());

        let cloned = service.clone();

        assert_eq!(service.id, cloned.id);
        assert_eq!(service.name, cloned.name);
        assert_eq!(service.capabilities.len(), cloned.capabilities.len());
    }

    #[test]
    fn test_service_instance_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let service = ServiceInstance::new(
            "svc-001".to_string(),
            "test-service".to_string(),
            "http://localhost:8080".to_string(),
        )
        .with_capability("security".to_string())
        .with_health_status("healthy".to_string());

        let json = serde_json::to_string(&service)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
        let deserialized: ServiceInstance =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to deserialize: {e}"),
                debug_info: None,
            })?;

        assert_eq!(service.id, deserialized.id);
        assert_eq!(service.name, deserialized.name);
        assert_eq!(service.endpoint, deserialized.endpoint);
        Ok(())
    }

    // ================ Additional Core Types Tests ================

    #[test]
    fn test_trust_level_all_variants() {
        let levels = [TrustLevel::High, TrustLevel::Medium, TrustLevel::Low, TrustLevel::Unknown];

        assert_eq!(levels.len(), 4);
    }

    #[test]
    fn test_trust_level_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let level = TrustLevel::High;
        let json = serde_json::to_string(&level)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
        let deserialized: TrustLevel =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Failed to deserialize: {e}"),
                debug_info: None,
            })?;

        // Can't directly compare enums without PartialEq, so serialize both and compare
        let json2 = serde_json::to_string(&deserialized)
            .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
        assert_eq!(json, json2);
        Ok(())
    }

    #[test]
    fn test_interaction_result_success() {
        let result = InteractionResult {
            success: true,
            message: "Operation completed".to_string(),
        };

        assert!(result.success);
        assert_eq!(result.message, "Operation completed");
    }

    #[test]
    fn test_interaction_result_failure() {
        let result = InteractionResult {
            success: false,
            message: "Operation failed".to_string(),
        };

        assert!(!result.success);
        assert_eq!(result.message, "Operation failed");
    }

    #[test]
    fn test_local_node() {
        let node = LocalNode {
            id: "node-001".to_string(),
            name: "primary-node".to_string(),
            endpoint: "http://localhost:9090".to_string(),
        };

        assert_eq!(node.id, "node-001");
        assert_eq!(node.name, "primary-node");
        assert_eq!(node.endpoint, "http://localhost:9090");
    }

    #[test]
    fn test_storage_info() {
        let storage = StorageInfo {
            total_bytes: 1_000_000_000,
            used_bytes: 400_000_000,
            available_bytes: 600_000_000,
        };

        assert_eq!(storage.total_bytes, 1_000_000_000);
        assert_eq!(storage.used_bytes, 400_000_000);
        assert_eq!(storage.available_bytes, 600_000_000);
    }

    #[test]
    fn test_network_measurement() {
        let measurement = NetworkMeasurement {
            latency_ms: 50,
            bandwidth_mbps: 1000,
        };

        assert_eq!(measurement.latency_ms, 50);
        assert_eq!(measurement.bandwidth_mbps, 1000);
    }

    #[test]
    fn test_dataset_info() {
        let dataset = DatasetInfo {
            name: "training-data".to_string(),
            size_bytes: 5_000_000,
            format: "parquet".to_string(),
        };

        assert_eq!(dataset.name, "training-data");
        assert_eq!(dataset.size_bytes, 5_000_000);
        assert_eq!(dataset.format, "parquet");
    }

    #[test]
    fn test_federation_stats() {
        let stats = FederationStats {
            total_nodes: 100,
            active_nodes: 85,
            total_messages: 10_000,
        };

        assert_eq!(stats.total_nodes, 100);
        assert_eq!(stats.active_nodes, 85);
        assert_eq!(stats.total_messages, 10_000);
    }

    #[test]
    fn test_compute_resources() {
        let resources = ComputeResources {
            cpu_cores: 16,
            memory_gb: 64,
            gpu_count: 2,
        };

        assert_eq!(resources.cpu_cores, 16);
        assert_eq!(resources.memory_gb, 64);
        assert_eq!(resources.gpu_count, 2);
    }
}
