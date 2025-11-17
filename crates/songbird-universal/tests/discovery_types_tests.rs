//! Comprehensive Discovery Types Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests for discovery module data structures and types.

use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::discovery::*;
use std::collections::HashMap;

// ============================================================================
// DISCOVERY CONFIG TESTS
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert!(config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
    assert!(config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout.as_secs(), 30);
}

#[test]
fn test_discovery_config_custom() -> SongbirdResult<()> {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    let config = DiscoveryConfig {
        mechanisms,
        timeout: tokio::time::Duration::from_secs(60),
    };

    assert!(config.mechanisms.enable_environment_scan);
    assert!(!config.mechanisms.enable_network_scanning);
    assert!(!config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout.as_secs(), 60);
    Ok(())
}

#[test]
fn test_discovery_config_clone() -> SongbirdResult<()> {
    let config1 = DiscoveryConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.timeout, config2.timeout);
    Ok(())
}

#[test]
fn test_discovery_config_debug() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("DiscoveryConfig"));
    Ok(())
}

// ============================================================================
// DISCOVERY MECHANISMS TESTS
// ============================================================================

#[test]
fn test_discovery_mechanisms_all_enabled() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: true,
    };

    assert!(mechanisms.enable_environment_scan);
    assert!(mechanisms.enable_network_scanning);
    assert!(mechanisms.enable_container_discovery);
}

#[test]
fn test_discovery_mechanisms_all_disabled() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: false,
        enable_network_scanning: false,
        enable_container_discovery: false,
    };

    assert!(!mechanisms.enable_environment_scan);
    assert!(!mechanisms.enable_network_scanning);
    assert!(!mechanisms.enable_container_discovery);
}

#[test]
fn test_discovery_mechanisms_selective() {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: false,
        enable_container_discovery: true,
    };

    assert!(mechanisms.enable_environment_scan);
    assert!(!mechanisms.enable_network_scanning);
    assert!(mechanisms.enable_container_discovery);
}

#[test]
fn test_discovery_mechanisms_clone() -> SongbirdResult<()> {
    let mech1 = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: true,
    };
    let mech2 = mech1.clone();
    assert_eq!(mech1.enable_environment_scan, mech2.enable_environment_scan);
    Ok(())
}

#[test]
fn test_discovery_mechanisms_debug() -> SongbirdResult<()> {
    let mechanisms = DiscoveryMechanisms {
        enable_environment_scan: true,
        enable_network_scanning: true,
        enable_container_discovery: true,
    };
    let debug_str = format!("{:?}", mechanisms);
    assert!(debug_str.contains("DiscoveryMechanisms"));
    Ok(())
}

// ============================================================================
// DISCOVERED PRIMAL TESTS
// ============================================================================

#[test]
fn test_discovered_primal_creation() {
    use songbird_universal::capabilities::Capability;
    use songbird_universal::types::PrimalType;

    let capability = Capability {
        capability_type: "compute".to_string(),
        name: "container_runtime".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: songbird_universal::capabilities::QoSMetrics::default(),
        available: true,
    };

    let primal = DiscoveredPrimal {
        name: "test-primal".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: format!("http://localhost:{}", test_orchestrator_port()),
        capabilities: vec![capability],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Environment,
        discovered_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    assert_eq!(primal.name, "test-primal");
    assert_eq!(primal.endpoint, format!("http://localhost:{}", test_orchestrator_port()));
    assert_eq!(primal.capabilities.len(), 1);
    assert_eq!(primal.health, PrimalHealth::Healthy);
    assert_eq!(primal.discovery_method, DiscoveryMethod::Environment);
}

#[test]
fn test_discovered_primal_with_metadata() {
    use songbird_universal::types::PrimalType;

    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-west".to_string());
    metadata.insert("tier".to_string(), "production".to_string());

    let primal = DiscoveredPrimal {
        name: "prod-primal".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "https://ai.example.com".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Kubernetes,
        discovered_at: chrono::Utc::now(),
        metadata,
    };

    assert_eq!(primal.metadata.len(), 2);
    assert_eq!(primal.metadata.get("region"), Some(&"us-west".to_string()));
}

#[test]
fn test_discovered_primal_clone() {
    use songbird_universal::types::PrimalType;

    let primal1 = DiscoveredPrimal {
        name: "test".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: format!("http://storage:{}", test_orchestrator_port()),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Docker,
        discovered_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let primal2 = primal1.clone();
    assert_eq!(primal1.name, primal2.name);
    assert_eq!(primal1.endpoint, primal2.endpoint);
}

#[test]
fn test_discovered_primal_debug() -> SongbirdResult<()> {
    use songbird_universal::types::PrimalType;

    let primal = DiscoveredPrimal {
        name: "debug-test".to_string(),
        primal_type: PrimalType::new("security"),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Environment,
        discovered_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let debug_str = format!("{:?}", primal);
    assert!(debug_str.contains("DiscoveredPrimal"));
    Ok(())
}

// ============================================================================
// DISCOVERY METHOD TESTS
// ============================================================================

#[test]
fn test_discovery_method_all_variants() {
    let env = DiscoveryMethod::Environment;
    let network = DiscoveryMethod::NetworkScan;
    let mdns = DiscoveryMethod::Mdns;
    let config = DiscoveryMethod::Configuration;
    let k8s = DiscoveryMethod::Kubernetes;
    let docker = DiscoveryMethod::Docker;

    assert_eq!(env, DiscoveryMethod::Environment);
    assert_eq!(network, DiscoveryMethod::NetworkScan);
    assert_eq!(mdns, DiscoveryMethod::Mdns);
    assert_eq!(config, DiscoveryMethod::Configuration);
    assert_eq!(k8s, DiscoveryMethod::Kubernetes);
    assert_eq!(docker, DiscoveryMethod::Docker);
}

#[test]
fn test_discovery_method_equality() -> SongbirdResult<()> {
    assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
    assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::Docker);
    Ok(())
}

#[test]
fn test_discovery_method_clone() -> SongbirdResult<()> {
    let method1 = DiscoveryMethod::Kubernetes;
    let method2 = method1.clone();
    assert_eq!(method1, method2);
    Ok(())
}

#[test]
fn test_discovery_method_debug() -> SongbirdResult<()> {
    let method = DiscoveryMethod::Kubernetes;
    let debug_str = format!("{:?}", method);
    assert!(debug_str.contains("Kubernetes"));
    Ok(())
}

// ============================================================================
// PRIMAL HEALTH TESTS
// ============================================================================

#[test]
fn test_primal_health_all_variants() -> SongbirdResult<()> {
    let healthy = PrimalHealth::Healthy;
    let degraded = PrimalHealth::Degraded;
    let unhealthy = PrimalHealth::Unhealthy;
    let unknown = PrimalHealth::Unknown;

    assert_eq!(healthy, PrimalHealth::Healthy);
    assert_eq!(degraded, PrimalHealth::Degraded);
    assert_eq!(unhealthy, PrimalHealth::Unhealthy);
    assert_eq!(unknown, PrimalHealth::Unknown);
    Ok(())
}

#[test]
fn test_primal_health_equality() -> SongbirdResult<()> {
    assert_eq!(PrimalHealth::Healthy, PrimalHealth::Healthy);
    assert_ne!(PrimalHealth::Healthy, PrimalHealth::Degraded);
    Ok(())
}

#[test]
fn test_primal_health_clone() -> SongbirdResult<()> {
    let health1 = PrimalHealth::Healthy;
    let health2 = health1.clone();
    assert_eq!(health1, health2);
    Ok(())
}

#[test]
fn test_primal_health_debug() -> SongbirdResult<()> {
    let health = PrimalHealth::Healthy;
    let debug_str = format!("{:?}", health);
    assert!(debug_str.contains("Healthy"));
    Ok(())
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_discovered_primal_serialization() -> SongbirdResult<()> {
    use songbird_universal::types::PrimalType;

    let primal = DiscoveredPrimal {
        name: "serialize-test".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: format!("http://localhost:{}", test_orchestrator_port()),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Environment,
        discovered_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&primal)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: DiscoveredPrimal = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.name, primal.name);
    assert_eq!(deserialized.endpoint, primal.endpoint);
    Ok(())
}

#[test]
fn test_discovery_method_serialization() -> SongbirdResult<()> {
    let method = DiscoveryMethod::Kubernetes;
    let json = serde_json::to_string(&method)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: DiscoveryMethod = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized, method);
    Ok(())
}

#[test]
fn test_primal_health_serialization() -> SongbirdResult<()> {
    let health = PrimalHealth::Healthy;
    let json = serde_json::to_string(&health)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: PrimalHealth = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized, health);
    Ok(())
}

// ============================================================================
// UNIVERSAL PRIMAL DISCOVERY TESTS
// ============================================================================

#[test]
fn test_universal_primal_discovery_creation() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let discovery = UniversalPrimalDiscovery::new(config);

    // Should create successfully
    let debug_str = format!("{:?}", discovery);
    assert!(debug_str.contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[test]
fn test_universal_primal_discovery_with_custom_config() -> SongbirdResult<()> {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: tokio::time::Duration::from_secs(10),
    };

    let discovery = UniversalPrimalDiscovery::new(config);
    let debug_str = format!("{:?}", discovery);
    assert!(debug_str.contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[test]
fn test_universal_primal_discovery_clone() -> SongbirdResult<()> {
    let config = DiscoveryConfig::default();
    let discovery1 = UniversalPrimalDiscovery::new(config);
    let discovery2 = discovery1.clone();

    // Both should be valid
    let debug1 = format!("{:?}", discovery1);
    let debug2 = format!("{:?}", discovery2);
    assert!(debug1.contains("UniversalPrimalDiscovery"));
    assert!(debug2.contains("UniversalPrimalDiscovery"));
    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_discovery_workflow() -> SongbirdResult<()> {
    // Create configuration
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: true,
        },
        timeout: tokio::time::Duration::from_secs(30),
    };

    // Create discovery engine
    let discovery = UniversalPrimalDiscovery::new(config);

    // Verify it was created successfully
    let debug_str = format!("{:?}", discovery);
    assert!(debug_str.contains("UniversalPrimalDiscovery"));
    Ok(())
}

#[test]
fn test_primal_health_lifecycle() {
    let mut health = PrimalHealth::Unknown;
    assert_eq!(health, PrimalHealth::Unknown);

    health = PrimalHealth::Healthy;
    assert_eq!(health, PrimalHealth::Healthy);

    health = PrimalHealth::Degraded;
    assert_eq!(health, PrimalHealth::Degraded);

    health = PrimalHealth::Unhealthy;
    assert_eq!(health, PrimalHealth::Unhealthy);
}

#[test]
fn test_discovery_methods_categorization() {
    let local_methods = vec![DiscoveryMethod::Environment, DiscoveryMethod::Configuration];

    let network_methods = vec![DiscoveryMethod::NetworkScan, DiscoveryMethod::Mdns];

    let orchestration_methods = vec![DiscoveryMethod::Kubernetes, DiscoveryMethod::Docker];

    assert_eq!(local_methods.len(), 2);
    assert_eq!(network_methods.len(), 2);
    assert_eq!(orchestration_methods.len(), 2);
}

#[test]
fn test_complete_primal_discovery_scenario() {
    use songbird_universal::capabilities::Capability;
    use songbird_universal::types::PrimalType;

    // Discover a primal via environment
    let mut metadata = HashMap::new();
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("region".to_string(), "us-east-1".to_string());

    let cap1 = Capability {
        capability_type: "compute".to_string(),
        name: "container_runtime".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: songbird_universal::capabilities::QoSMetrics::default(),
        available: true,
    };

    let cap2 = Capability {
        capability_type: "storage".to_string(),
        name: "object_store".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: songbird_universal::capabilities::QoSMetrics::default(),
        available: true,
    };

    let primal_type = PrimalType::new("compute");

    let primal = DiscoveredPrimal {
        name: "production-compute".to_string(),
        primal_type: primal_type.clone(),
        endpoint: "https://compute.prod.example.com".to_string(),
        capabilities: vec![cap1, cap2],
        health: PrimalHealth::Healthy,
        discovery_method: DiscoveryMethod::Kubernetes,
        discovered_at: chrono::Utc::now(),
        metadata,
    };

    // Verify all fields
    assert_eq!(primal.name, "production-compute");
    assert_eq!(primal.primal_type, primal_type);
    assert!(primal.endpoint.starts_with("https://"));
    assert_eq!(primal.capabilities.len(), 2);
    assert_eq!(primal.health, PrimalHealth::Healthy);
    assert_eq!(primal.discovery_method, DiscoveryMethod::Kubernetes);
    assert_eq!(primal.metadata.len(), 2);
}
