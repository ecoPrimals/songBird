// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive Tests for Discovery Types
//!
//! **Target**: Boost coverage from 0% to 90%+
//! **Pattern**: Modern concurrent testing, no sleeps

use super::types::*;
use crate::capabilities::{Capability, QoSMetrics};
use crate::types::PrimalType;
use std::collections::HashMap;
use tokio::time::Duration;

// Helper to create test capabilities
fn create_test_capability(cap_type: &str) -> Capability {
    Capability {
        capability_type: cap_type.to_string(),
        name: format!("{}_service", cap_type),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    }
}

// ============================================================================
// DiscoveryConfig Tests
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert!(config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
    assert!(config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout, Duration::from_secs(30));
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: true,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(60),
    };

    assert!(!config.mechanisms.enable_environment_scan);
    assert!(config.mechanisms.enable_network_scanning);
    assert!(!config.mechanisms.enable_container_discovery);
    assert_eq!(config.timeout, Duration::from_secs(60));
}

#[test]
fn test_discovery_config_clone() {
    let config1 = DiscoveryConfig::default();
    let config2 = config1.clone();

    assert_eq!(
        config1.mechanisms.enable_environment_scan,
        config2.mechanisms.enable_environment_scan
    );
    assert_eq!(config1.timeout, config2.timeout);
}

// ============================================================================
// DiscoveredPrimal Tests
// ============================================================================

#[test]
fn test_discovered_primal_creation() {
    let primal = DiscoveredPrimal::new(
        "test-primal".to_string(),
        PrimalType::new("security"),
        "http://localhost:8080".to_string(),
        vec![create_test_capability("security")],
        DiscoveryMethod::Environment,
    );

    assert_eq!(primal.name, "test-primal");
    assert_eq!(primal.endpoint, "http://localhost:8080");
    assert_eq!(primal.discovery_method, DiscoveryMethod::Environment);
    assert_eq!(primal.health, PrimalHealth::Unknown);
    assert!(primal.metadata.is_empty());
}

#[test]
fn test_discovered_primal_with_metadata() {
    let primal = DiscoveredPrimal::new(
        "test-primal".to_string(),
        PrimalType::new("compute"),
        "http://localhost:9000".to_string(),
        vec![],
        DiscoveryMethod::NetworkScan,
    )
    .with_metadata("region".to_string(), "us-west-2".to_string())
    .with_metadata("version".to_string(), "1.0.0".to_string());

    assert_eq!(primal.metadata.len(), 2);
    assert_eq!(primal.metadata.get("region"), Some(&"us-west-2".to_string()));
    assert_eq!(primal.metadata.get("version"), Some(&"1.0.0".to_string()));
}

#[test]
fn test_discovered_primal_is_healthy() {
    let mut primal = DiscoveredPrimal::new(
        "test".to_string(),
        PrimalType::new("storage"),
        "http://localhost:7000".to_string(),
        vec![],
        DiscoveryMethod::Manual,
    );

    // Initially unknown, not healthy
    assert!(!primal.is_healthy());

    // Set to healthy
    primal.health = PrimalHealth::Healthy;
    assert!(primal.is_healthy());

    // Set to degraded, not healthy
    primal.health = PrimalHealth::Degraded;
    assert!(!primal.is_healthy());

    // Set to unhealthy
    primal.health = PrimalHealth::Unhealthy;
    assert!(!primal.is_healthy());
}

#[test]
fn test_discovered_primal_has_capability() {
    let security_cap = create_test_capability("security");
    let storage_cap = create_test_capability("storage");

    let primal = DiscoveredPrimal::new(
        "test".to_string(),
        PrimalType::new("security"),
        "http://localhost:8080".to_string(),
        vec![security_cap.clone()],
        DiscoveryMethod::ServiceRegistry,
    );

    assert!(primal.has_capability(&security_cap));
    assert!(!primal.has_capability(&storage_cap));
}

#[test]
fn test_discovered_primal_multiple_capabilities() {
    let security = create_test_capability("security");
    let storage = create_test_capability("storage");
    let compute = create_test_capability("compute");

    let primal = DiscoveredPrimal::new(
        "multi-cap".to_string(),
        PrimalType::new("multipurpose"),
        "http://localhost:5000".to_string(),
        vec![security.clone(), storage.clone()],
        DiscoveryMethod::MDNS,
    );

    assert!(primal.has_capability(&security));
    assert!(primal.has_capability(&storage));
    assert!(!primal.has_capability(&compute));
}

#[test]
fn test_discovered_primal_serialization() {
    let primal = DiscoveredPrimal::new(
        "serialize-test".to_string(),
        PrimalType::new("ai"),
        "http://localhost:6000".to_string(),
        vec![],
        DiscoveryMethod::ContainerOrchestration,
    );

    // Should be serializable
    let json = serde_json::to_string(&primal).expect("Failed to serialize");
    assert!(!json.is_empty());

    // Should be deserializable
    let deserialized: DiscoveredPrimal =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.name, primal.name);
    assert_eq!(deserialized.endpoint, primal.endpoint);
}

#[test]
fn test_discovered_primal_clone() {
    let primal1 = DiscoveredPrimal::new(
        "clone-test".to_string(),
        PrimalType::new("compute"),
        "http://localhost:4000".to_string(),
        vec![create_test_capability("compute")],
        DiscoveryMethod::Environment,
    );

    let primal2 = primal1.clone();

    assert_eq!(primal1.name, primal2.name);
    assert_eq!(primal1.endpoint, primal2.endpoint);
    assert_eq!(primal1.discovery_method, primal2.discovery_method);
}

// ============================================================================
// DiscoveryMethod Tests
// ============================================================================

#[test]
fn test_discovery_method_variants() {
    assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
    assert_eq!(DiscoveryMethod::NetworkScan, DiscoveryMethod::NetworkScan);
    assert_eq!(DiscoveryMethod::ContainerOrchestration, DiscoveryMethod::ContainerOrchestration);
    assert_eq!(DiscoveryMethod::ServiceRegistry, DiscoveryMethod::ServiceRegistry);
    assert_eq!(DiscoveryMethod::MDNS, DiscoveryMethod::MDNS);
    assert_eq!(DiscoveryMethod::Manual, DiscoveryMethod::Manual);
}

#[test]
fn test_discovery_method_inequality() {
    assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::NetworkScan);
    assert_ne!(DiscoveryMethod::Manual, DiscoveryMethod::MDNS);
}

#[test]
fn test_discovery_method_serialization() {
    let method = DiscoveryMethod::ServiceRegistry;

    let json = serde_json::to_string(&method).expect("Failed to serialize");
    let deserialized: DiscoveryMethod = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(method, deserialized);
}

#[test]
fn test_discovery_method_clone() {
    let method1 = DiscoveryMethod::MDNS;
    let method2 = method1.clone();

    assert_eq!(method1, method2);
}

// ============================================================================
// PrimalHealth Tests
// ============================================================================

#[test]
fn test_primal_health_variants() {
    assert_eq!(PrimalHealth::Healthy, PrimalHealth::Healthy);
    assert_eq!(PrimalHealth::Degraded, PrimalHealth::Degraded);
    assert_eq!(PrimalHealth::Unhealthy, PrimalHealth::Unhealthy);
    assert_eq!(PrimalHealth::Unknown, PrimalHealth::Unknown);
}

#[test]
fn test_primal_health_inequality() {
    assert_ne!(PrimalHealth::Healthy, PrimalHealth::Degraded);
    assert_ne!(PrimalHealth::Unhealthy, PrimalHealth::Unknown);
}

#[test]
fn test_primal_health_serialization() {
    let health = PrimalHealth::Healthy;

    let json = serde_json::to_string(&health).expect("Failed to serialize");
    let deserialized: PrimalHealth = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(health, deserialized);
}

#[test]
fn test_primal_health_clone() {
    let health1 = PrimalHealth::Degraded;
    let health2 = health1.clone();

    assert_eq!(health1, health2);
}

// ============================================================================
// DiscoveryError Tests
// ============================================================================

#[test]
fn test_discovery_error_no_primals_found() {
    let error = DiscoveryError::NoPrimalsFound;
    let error_str = error.to_string();

    assert_eq!(error_str, "No primals discovered");
}

#[test]
fn test_discovery_error_timeout() {
    let timeout = Duration::from_secs(30);
    let error = DiscoveryError::Timeout(timeout);
    let error_str = error.to_string();

    assert!(error_str.contains("Discovery timeout"));
    assert!(error_str.contains("30s"));
}

#[test]
fn test_discovery_error_network() {
    let error = DiscoveryError::NetworkError("Connection refused".to_string());
    let error_str = error.to_string();

    assert_eq!(error_str, "Network error: Connection refused");
}

#[test]
fn test_discovery_error_config() {
    let error = DiscoveryError::ConfigError("Invalid port".to_string());
    let error_str = error.to_string();

    assert_eq!(error_str, "Configuration error: Invalid port");
}

#[test]
fn test_discovery_error_health_check_failed() {
    let error = DiscoveryError::HealthCheckFailed {
        primal: "beardog".to_string(),
        reason: "Timeout".to_string(),
    };
    let error_str = error.to_string();

    assert!(error_str.contains("Health check failed for beardog"));
    assert!(error_str.contains("Timeout"));
}

#[test]
fn test_discovery_error_clone() {
    let error1 = DiscoveryError::NoPrimalsFound;
    let error2 = error1.clone();

    assert_eq!(error1.to_string(), error2.to_string());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_complete_primal_lifecycle() {
    // Create primal
    let mut primal = DiscoveredPrimal::new(
        "lifecycle-test".to_string(),
        PrimalType::new("security"),
        "http://10.0.0.10:8080".to_string(),
        vec![create_test_capability("security"), create_test_capability("storage")],
        DiscoveryMethod::NetworkScan,
    );

    // Add metadata
    primal = primal
        .with_metadata("datacenter".to_string(), "dc1".to_string())
        .with_metadata("rack".to_string(), "rack5".to_string());

    // Initially unknown health
    assert!(!primal.is_healthy());

    // Health check passes
    primal.health = PrimalHealth::Healthy;
    assert!(primal.is_healthy());

    // Verify capabilities
    assert!(primal.has_capability(&create_test_capability("security")));
    assert!(primal.has_capability(&create_test_capability("storage")));
    assert!(!primal.has_capability(&create_test_capability("compute")));

    // Verify metadata
    assert_eq!(primal.metadata.len(), 2);
    assert_eq!(primal.metadata.get("datacenter"), Some(&"dc1".to_string()));
}

#[test]
fn test_multiple_primals_different_methods() {
    let primals = [
        DiscoveredPrimal::new(
            "env-primal".to_string(),
            PrimalType::new("storage"),
            "http://localhost:7000".to_string(),
            vec![],
            DiscoveryMethod::Environment,
        ),
        DiscoveredPrimal::new(
            "scan-primal".to_string(),
            PrimalType::new("compute"),
            "http://192.168.1.100:9000".to_string(),
            vec![],
            DiscoveryMethod::NetworkScan,
        ),
        DiscoveredPrimal::new(
            "k8s-primal".to_string(),
            PrimalType::new("ai"),
            "http://service.namespace.svc:6000".to_string(),
            vec![],
            DiscoveryMethod::ContainerOrchestration,
        ),
    ];

    assert_eq!(primals.len(), 3);
    assert_eq!(primals[0].discovery_method, DiscoveryMethod::Environment);
    assert_eq!(primals[1].discovery_method, DiscoveryMethod::NetworkScan);
    assert_eq!(primals[2].discovery_method, DiscoveryMethod::ContainerOrchestration);
}

#[test]
fn test_config_with_selective_mechanisms() {
    let config = DiscoveryConfig {
        mechanisms: DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: false,
        },
        timeout: Duration::from_secs(10),
    };

    // Only environment scanning enabled
    assert!(config.mechanisms.enable_environment_scan);
    assert!(!config.mechanisms.enable_network_scanning);
    assert!(!config.mechanisms.enable_container_discovery);
}

#[test]
fn test_primal_with_empty_capabilities() {
    let primal = DiscoveredPrimal::new(
        "no-caps".to_string(),
        PrimalType::new("unknown"),
        "http://localhost:1234".to_string(),
        vec![],
        DiscoveryMethod::Manual,
    );

    assert!(primal.capabilities.is_empty());
    assert!(!primal.has_capability(&create_test_capability("security")));
}

#[test]
fn test_primal_with_many_capabilities() {
    let caps = vec![
        create_test_capability("security"),
        create_test_capability("storage"),
        create_test_capability("compute"),
        create_test_capability("ai"),
    ];

    let primal = DiscoveredPrimal::new(
        "all-caps".to_string(),
        PrimalType::new("universal"),
        "http://localhost:5555".to_string(),
        caps.clone(),
        DiscoveryMethod::ServiceRegistry,
    );

    assert_eq!(primal.capabilities.len(), 4);
    for cap in caps {
        assert!(primal.has_capability(&cap));
    }
}

#[test]
fn test_primal_metadata_updates() {
    let primal = DiscoveredPrimal::new(
        "metadata-test".to_string(),
        PrimalType::new("test"),
        "http://test:1000".to_string(),
        vec![],
        DiscoveryMethod::Manual,
    )
    .with_metadata("key1".to_string(), "value1".to_string())
    .with_metadata("key2".to_string(), "value2".to_string())
    .with_metadata("key3".to_string(), "value3".to_string());

    assert_eq!(primal.metadata.len(), 3);
    assert!(primal.metadata.contains_key("key1"));
    assert!(primal.metadata.contains_key("key2"));
    assert!(primal.metadata.contains_key("key3"));
}

#[test]
fn test_health_state_transitions() {
    let mut primal = DiscoveredPrimal::new(
        "health-test".to_string(),
        PrimalType::new("test"),
        "http://test:2000".to_string(),
        vec![],
        DiscoveryMethod::MDNS,
    );

    // Start unknown
    assert_eq!(primal.health, PrimalHealth::Unknown);
    assert!(!primal.is_healthy());

    // Transition to healthy
    primal.health = PrimalHealth::Healthy;
    assert!(primal.is_healthy());

    // Transition to degraded
    primal.health = PrimalHealth::Degraded;
    assert!(!primal.is_healthy());

    // Transition to unhealthy
    primal.health = PrimalHealth::Unhealthy;
    assert!(!primal.is_healthy());

    // Back to healthy
    primal.health = PrimalHealth::Healthy;
    assert!(primal.is_healthy());
}
