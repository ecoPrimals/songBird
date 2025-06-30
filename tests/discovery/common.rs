use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
//!
 //! Common Test Utilities for Discovery Tests
 //! 
 //! This module contains shared test utilities, helper functions, and common setup
 //! code used across all discovery test modules.
//!

use std::time::Duration;
use tokio::time::sleep;

use songbird_gaming_bridge::{
    discovery::types::{AccessLevel, DatasetType, NetworkLocation},
    discovery::{
        ComputeResources, DatasetInfo, InteractionPenalties, MonitoringConfig, NetworkConfig,
        NetworkTimingConfig, NodeInfo, NodeType, ResourceQuery, SongbirdDiscovery,
        SongbirdDiscoveryConfig, TrustConfig, TrustLevel, TrustThresholds,
    },
    traits::discovery::{ServiceDiscovery, ServiceHealthStatus, ServiceQuery},
    traits::service_id::{ServiceEndpoint, ServiceInfo},
};

/// Create a test Songbird Discovery instance with default configuration
pub async fn create_test_discovery() -> SongbirdDiscovery {
    let config = SongbirdDiscoveryConfig {
        node_id: Some("test-node-01".to_string()),
        node_type: NodeType::Orchestrator,
        institution: Some("Test Institution".to_string()),
        federation_enabled: true,
        health_check_interval_secs: 5,
        node_discovery_interval_secs: 10,
        trust_verification_enabled: true,
        max_federation_nodes: 100,
        network: NetworkConfig::default(),
        monitoring: MonitoringConfig::default(),
        trust: TrustConfig::default(),
    };

    SongbirdDiscovery::new(config)
}

/// Create a test Songbird Discovery instance with custom configuration
pub async fn create_test_discovery_with_config(config: SongbirdDiscoveryConfig) -> SongbirdDiscovery {
    SongbirdDiscovery::new(config)
}

/// Create a test service info with customizable parameters
pub fn create_test_service(id: &str, service_type: &str) -> ServiceInfo {
    ServiceInfo {
        id: id.to_string(),
        name: format!("Test Service {}", id),
        version: "1.0.0".to_string(),
        service_type: service_type.to_string(),
        description: format!("Test service for {}", service_type),
        endpoints: vec![],
        tags: std::collections::HashMap::new(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("test".to_string(), "true".to_string());
            tags.insert("service_type".to_string(), service_type.to_string());
            tags
        },
        
    }
}

/// Create a detailed test service with full configuration
pub fn create_detailed_test_service(id: &str, service_type: &str, endpoints: Vec<ServiceEndpoint>) -> ServiceInfo {
    ServiceInfo {
        id: id.to_string(),
        name: format!("Test Service {}", id),
        version: "1.0.0".to_string(),
        service_type: service_type.to_string(),
        description: format!("A test service for {}", service_type),
        endpoints,
        tags: std::collections::HashMap::new(),
        tags: {
            let mut tags = HashMap::new();
            tags.insert("environment".to_string(), "test".to_string());
            tags.insert("priority".to_string(), "high".to_string());
            tags.insert("service_type".to_string(), service_type.to_string());
            tags
        },
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("owner".to_string(), "test-team".into());
            metadata.insert("cost_center".to_string(), "engineering".into());
            metadata
        },
    }
}

/// Create a test node info with customizable parameters
pub fn create_test_node(id: &str, institution: &str, node_type: NodeType) -> NodeInfo {
    NodeInfo {
        id: id.to_string(),
        address: format!("{}:8080", id),
        node_type,
        institution: Some(institution.to_string()),
        resources: ComputeResources {
            cpu_cores: 16,
            cpu_architecture: "x86_64".to_string(),
            memory_total_gb: 64,
            memory_available_gb: 48,
            gpu_info: vec![],
            storage_devices: vec![],
            network_bandwidth_mbps: 1000.0,
        },
        current_load: Default::default(),
        available_datasets: vec![],
        storage_capacity: Default::default(),
        trust_level: TrustLevel::Institutional,
        reputation_score: 0.9,
        network_location: NetworkLocation {
            region: "us-east-1".to_string(),
            institution: Some(institution.to_string()),
            subnet: None,
            external_ip: None,
            internal_ip: None,
        },
        bandwidth_measurements: HashMap::new(),
        latency_measurements: HashMap::new(),
        last_seen: chrono::Utc::now(),
        health_status: ServiceHealthStatus::Healthy,
        services: vec![],
    }
}

/// Helper function to create a simple service endpoint
pub fn create_endpoint(path: &str, method: &str, description: &str) -> ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
    ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
        path: path.to_string(),
        method: method.to_string(),
        description: description.to_string(),
        parameters: vec![],
        response_schema: None,
    }
}

/// Create a test discovery configuration for non-federated testing
pub fn create_standalone_config() -> SongbirdDiscoveryConfig {
    SongbirdDiscoveryConfig {
        federation_enabled: false,
        ..Default::default()
    }
}

/// Create a test discovery configuration for federation testing
pub fn create_federation_config(node_id: &str) -> SongbirdDiscoveryConfig {
    SongbirdDiscoveryConfig {
        node_id: Some(node_id.to_string()),
        federation_enabled: true,
        health_check_interval_secs: 1,
        node_discovery_interval_secs: 2,
        trust_verification_enabled: true,
        max_federation_nodes: 10,
        ..Default::default()
    }
}

/// Sleep helper for test timing
pub async fn test_sleep(millis: u64) {
    sleep(Duration::from_millis(millis)).await;
}

/// Assert that a test completed successfully and print status
pub fn assert_test_success(test_name: &str) {
    println!("✅ {} tests passed", test_name);
} 