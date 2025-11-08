//! Comprehensive Capabilities Types Tests
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
//! Tests for capability type definitions and metrics.

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::capabilities::*;
use std::collections::HashMap;

// ============================================================================
// CAPABILITY TESTS
// ============================================================================

#[test]
fn test_capability_creation() {
    let cap = Capability {
        capability_type: "compute".to_string(),
        name: "container_runtime".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(cap.capability_type, "compute");
    assert_eq!(cap.name, "container_runtime");
    assert_eq!(cap.version, "1.0.0");
    assert!(cap.available);
}

#[test]
fn test_capability_with_parameters() {
    let mut params = HashMap::new();
    params.insert("max_cpu".to_string(), serde_json::json!(8));
    params.insert("max_memory".to_string(), serde_json::json!("16GB"));

    let cap = Capability {
        capability_type: "compute".to_string(),
        name: "vm_runtime".to_string(),
        version: "2.0.0".to_string(),
        parameters: params,
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(cap.parameters.len(), 2);
    assert_eq!(cap.parameters.get("max_cpu"), Some(&serde_json::json!(8)));
}

#[test]
fn test_capability_unavailable() {
    let cap = Capability {
        capability_type: "storage".to_string(),
        name: "block_storage".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: false,
    };

    assert!(!cap.available);
}

#[test]
fn test_capability_clone() {
    let cap1 = Capability {
        capability_type: "security".to_string(),
        name: "encryption".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1.name, cap2.name);
    assert_eq!(cap1.capability_type, cap2.capability_type);
}

#[test]
fn test_capability_equality() {
    let cap1 = Capability {
        capability_type: "network".to_string(),
        name: "load_balancer".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let cap2 = Capability {
        capability_type: "network".to_string(),
        name: "load_balancer".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_serialization() -> SongbirdResult<()> {
    let cap = Capability {
        capability_type: "ai".to_string(),
        name: "model_inference".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let json = serde_json::to_string(&cap)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: Capability = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.name, cap.name);
    Ok(())
}

#[test]
fn test_capability_debug() -> SongbirdResult<()> {
    let cap = Capability {
        capability_type: "test".to_string(),
        name: "debug_test".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let debug_str = format!("{:?}", cap);
    assert!(debug_str.contains("Capability"));
    Ok(())
}

// ============================================================================
// QOS METRICS TESTS
// ============================================================================

#[test]
fn test_qos_metrics_default() {
    let qos = QoSMetrics::default();
    assert_eq!(qos.latency_ms, 100.0);
    assert_eq!(qos.throughput_ops_sec, 1000.0);
    assert_eq!(qos.availability, 0.99);
    assert_eq!(qos.reliability, 0.99);
}

#[test]
fn test_qos_metrics_custom() {
    let qos = QoSMetrics {
        latency_ms: 50.0,
        throughput_ops_sec: 2000.0,
        availability: 0.999,
        reliability: 0.9999,
        resource_usage: ResourceMetrics::default(),
    };

    assert_eq!(qos.latency_ms, 50.0);
    assert_eq!(qos.throughput_ops_sec, 2000.0);
    assert_eq!(qos.availability, 0.999);
    assert_eq!(qos.reliability, 0.9999);
}

#[test]
fn test_qos_metrics_high_performance() {
    let qos = QoSMetrics {
        latency_ms: 10.0,
        throughput_ops_sec: 10000.0,
        availability: 0.9999,
        reliability: 0.9999,
        resource_usage: ResourceMetrics::default(),
    };

    assert!(qos.latency_ms < 20.0);
    assert!(qos.throughput_ops_sec > 5000.0);
    assert!(qos.availability > 0.999);
}

#[test]
fn test_qos_metrics_clone() -> SongbirdResult<()> {
    let qos1 = QoSMetrics::default();
    let qos2 = qos1.clone();
    assert_eq!(qos1.latency_ms, qos2.latency_ms);
    Ok(())
}

#[test]
fn test_qos_metrics_equality() -> SongbirdResult<()> {
    let qos1 = QoSMetrics::default();
    let qos2 = QoSMetrics::default();
    assert_eq!(qos1, qos2);
    Ok(())
}

#[test]
fn test_qos_metrics_serialization() -> SongbirdResult<()> {
    let qos = QoSMetrics::default();
    let json = serde_json::to_string(&qos)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: QoSMetrics = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.latency_ms, qos.latency_ms);
    Ok(())
}

#[test]
fn test_qos_metrics_debug() -> SongbirdResult<()> {
    let qos = QoSMetrics::default();
    let debug_str = format!("{:?}", qos);
    assert!(debug_str.contains("QoSMetrics"));
    Ok(())
}

// ============================================================================
// RESOURCE METRICS TESTS
// ============================================================================

#[test]
fn test_resource_metrics_default() {
    let metrics = ResourceMetrics::default();
    assert_eq!(metrics.cpu_percent, 10.0);
    assert_eq!(metrics.memory_mb, 512);
    assert_eq!(metrics.network_mbps, 10.0);
    assert_eq!(metrics.storage_mb, 1024);
}

#[test]
fn test_resource_metrics_custom() {
    let metrics = ResourceMetrics {
        cpu_percent: 50.0,
        memory_mb: 2048,
        network_mbps: 100.0,
        storage_mb: 10000,
    };

    assert_eq!(metrics.cpu_percent, 50.0);
    assert_eq!(metrics.memory_mb, 2048);
    assert_eq!(metrics.network_mbps, 100.0);
    assert_eq!(metrics.storage_mb, 10000);
}

#[test]
fn test_resource_metrics_high_usage() {
    let metrics = ResourceMetrics {
        cpu_percent: 90.0,
        memory_mb: 8192,
        network_mbps: 1000.0,
        storage_mb: 100_000,
    };

    assert!(metrics.cpu_percent > 80.0);
    assert!(metrics.memory_mb > 4096);
}

#[test]
fn test_resource_metrics_low_usage() {
    let metrics = ResourceMetrics {
        cpu_percent: 5.0,
        memory_mb: 128,
        network_mbps: 1.0,
        storage_mb: 256,
    };

    assert!(metrics.cpu_percent < 10.0);
    assert!(metrics.memory_mb < 512);
}

#[test]
fn test_resource_metrics_clone() -> SongbirdResult<()> {
    let metrics1 = ResourceMetrics::default();
    let metrics2 = metrics1.clone();
    assert_eq!(metrics1.cpu_percent, metrics2.cpu_percent);
    Ok(())
}

#[test]
fn test_resource_metrics_equality() -> SongbirdResult<()> {
    let metrics1 = ResourceMetrics::default();
    let metrics2 = ResourceMetrics::default();
    assert_eq!(metrics1, metrics2);
    Ok(())
}

#[test]
fn test_resource_metrics_serialization() -> SongbirdResult<()> {
    let metrics = ResourceMetrics::default();
    let json = serde_json::to_string(&metrics)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ResourceMetrics = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.cpu_percent, metrics.cpu_percent);
    Ok(())
}

#[test]
fn test_resource_metrics_debug() -> SongbirdResult<()> {
    let metrics = ResourceMetrics::default();
    let debug_str = format!("{:?}", metrics);
    assert!(debug_str.contains("ResourceMetrics"));
    Ok(())
}

// ============================================================================
// DISCOVERY CONFIG TESTS
// ============================================================================

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert_eq!(config.refresh_interval.as_secs(), 300); // 5 minutes
    assert_eq!(config.discovery_timeout.as_secs(), 10);
    assert_eq!(config.max_concurrent_discoveries, 10);
    assert!(config.auto_discovery);
    assert!(!config.enable_network_discovery);
}

#[test]
fn test_discovery_config_custom() {
    use std::time::Duration;

    let config = DiscoveryConfig {
        refresh_interval: Duration::from_secs(60),
        discovery_timeout: Duration::from_secs(5),
        max_concurrent_discoveries: 20,
        auto_discovery: false,
        enable_network_discovery: true,
    };

    assert_eq!(config.refresh_interval.as_secs(), 60);
    assert_eq!(config.discovery_timeout.as_secs(), 5);
    assert_eq!(config.max_concurrent_discoveries, 20);
    assert!(!config.auto_discovery);
    assert!(config.enable_network_discovery);
}

#[test]
fn test_discovery_config_fast_refresh() -> SongbirdResult<()> {
    use std::time::Duration;

    let config = DiscoveryConfig {
        refresh_interval: Duration::from_secs(10),
        discovery_timeout: Duration::from_secs(2),
        max_concurrent_discoveries: 5,
        auto_discovery: true,
        enable_network_discovery: false,
    };

    assert!(config.refresh_interval.as_secs() < 30);
    Ok(())
}

#[test]
fn test_discovery_config_clone() -> SongbirdResult<()> {
    let config1 = DiscoveryConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.max_concurrent_discoveries, config2.max_concurrent_discoveries);
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
// PRIMAL TYPE TESTS (from capabilities module)
// ============================================================================

#[test]
fn test_primal_type_all_variants() {
    let security = PrimalType::Security;
    let compute = PrimalType::Compute;
    let storage = PrimalType::Storage;
    let ai = PrimalType::AI;
    let generic = PrimalType::Generic;

    assert_eq!(security, PrimalType::Security);
    assert_eq!(compute, PrimalType::Compute);
    assert_eq!(storage, PrimalType::Storage);
    assert_eq!(ai, PrimalType::AI);
    assert_eq!(generic, PrimalType::Generic);
}

#[test]
fn test_primal_type_equality() -> SongbirdResult<()> {
    assert_eq!(PrimalType::Compute, PrimalType::Compute);
    assert_ne!(PrimalType::Compute, PrimalType::Storage);
    Ok(())
}

#[test]
fn test_primal_type_clone() -> SongbirdResult<()> {
    let type1 = PrimalType::AI;
    let type2 = type1.clone();
    assert_eq!(type1, type2);
    Ok(())
}

#[test]
fn test_primal_type_debug() -> SongbirdResult<()> {
    let primal_type = PrimalType::Security;
    let debug_str = format!("{:?}", primal_type);
    assert!(debug_str.contains("Security"));
    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_capability_with_full_metrics() {
    let resource_metrics = ResourceMetrics {
        cpu_percent: 25.0,
        memory_mb: 1024,
        network_mbps: 50.0,
        storage_mb: 5000,
    };

    let qos_metrics = QoSMetrics {
        latency_ms: 75.0,
        throughput_ops_sec: 1500.0,
        availability: 0.995,
        reliability: 0.998,
        resource_usage: resource_metrics,
    };

    let cap = Capability {
        capability_type: "compute".to_string(),
        name: "kubernetes_cluster".to_string(),
        version: "1.25.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics,
        available: true,
    };

    assert_eq!(cap.qos_metrics.resource_usage.cpu_percent, 25.0);
}

#[test]
fn test_discovery_config_with_timeout() {
    use std::time::Duration;

    let config = DiscoveryConfig {
        refresh_interval: Duration::from_secs(120),
        discovery_timeout: Duration::from_secs(15),
        max_concurrent_discoveries: 15,
        auto_discovery: true,
        enable_network_discovery: true,
    };

    assert!(config.discovery_timeout.as_secs() >= 10);
}

#[test]
fn test_capability_types_comprehensive() {
    let types =
        vec!["compute", "storage", "network", "security", "ai", "database", "cache", "messaging"];

    for cap_type in types {
        let cap = Capability {
            capability_type: cap_type.to_string(),
            name: format!("{}_service", cap_type),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            qos_metrics: QoSMetrics::default(),
            available: true,
        };

        assert_eq!(cap.capability_type, cap_type);
    }
}

#[test]
fn test_qos_degradation_scenario() {
    let normal_qos = QoSMetrics {
        latency_ms: 50.0,
        throughput_ops_sec: 2000.0,
        availability: 0.999,
        reliability: 0.999,
        resource_usage: ResourceMetrics::default(),
    };

    let degraded_qos = QoSMetrics {
        latency_ms: 150.0,
        throughput_ops_sec: 500.0,
        availability: 0.95,
        reliability: 0.96,
        resource_usage: ResourceMetrics::default(),
    };

    assert!(degraded_qos.latency_ms > normal_qos.latency_ms);
    assert!(degraded_qos.throughput_ops_sec < normal_qos.throughput_ops_sec);
}

#[test]
fn test_resource_scaling_scenario() {
    let low = ResourceMetrics {
        cpu_percent: 10.0,
        memory_mb: 512,
        network_mbps: 10.0,
        storage_mb: 1024,
    };

    let medium = ResourceMetrics {
        cpu_percent: 50.0,
        memory_mb: 2048,
        network_mbps: 100.0,
        storage_mb: 10240,
    };

    let high = ResourceMetrics {
        cpu_percent: 90.0,
        memory_mb: 8192,
        network_mbps: 1000.0,
        storage_mb: 102_400,
    };

    assert!(low.cpu_percent < medium.cpu_percent);
    assert!(medium.cpu_percent < high.cpu_percent);
}
