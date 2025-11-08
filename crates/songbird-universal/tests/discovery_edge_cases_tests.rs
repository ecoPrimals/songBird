//! Discovery Mechanism Edge Case Tests
//!
//! Comprehensive tests for service discovery edge cases and boundary conditions

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

use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo,
};
use songbird_universal::{create_universal_adapter_with_config, UnifiedAdapterConfig};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a test service with specified parameters
fn create_test_service(name: &str, endpoint: &str, capabilities: Vec<&str>) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        endpoint: endpoint.to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: capabilities
            .iter()
            .map(|c| DiscoveredCapability {
                name: (*c).to_string(),
                version: "1.0".to_string(),
                description: format!("{} capability", c),
                provider: name.to_string(),
                endpoint: format!("{}/api/v1/{}", endpoint, c),
                qos_metrics: QosMetrics::default(),
                health_status: HealthStatus::Healthy,
            })
            .collect(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

// ============================================================================
// SERVICE INFO EDGE CASES
// ============================================================================

#[test]
fn test_service_info_with_empty_name() {
    // ARRANGE & ACT: Create service with empty name
    let service = ServiceInfo {
        name: String::new(),
        endpoint: format!("http://localhost:{}", test_orchestrator_port()),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // ASSERT: Should allow empty name (validation happens at higher level)
    assert_eq!(service.name, "");
}

#[test]
fn test_service_info_with_empty_endpoint() {
    // ARRANGE & ACT: Create service with empty endpoint
    let service = ServiceInfo {
        name: "test-service".to_string(),
        endpoint: String::new(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // ASSERT: Should allow empty endpoint
    assert_eq!(service.endpoint, "");
}

#[test]
fn test_service_info_with_no_capabilities() {
    // ARRANGE & ACT: Create service with no capabilities
    let service = create_test_service("test-service", "http://localhost:8080", vec![]);

    // ASSERT: Should allow empty capabilities list
    assert_eq!(service.capabilities.len(), 0);
}

#[test]
fn test_service_info_with_many_capabilities() {
    // ARRANGE & ACT: Create service with many capabilities
    let capabilities: Vec<DiscoveredCapability> = (0..100)
        .map(|i| DiscoveredCapability {
            name: format!("capability-{}", i),
            version: "1.0".to_string(),
            description: format!("Capability {}", i),
            provider: "test-service".to_string(),
            endpoint: format!("http://localhost:8080/api/v1/capability-{}", i),
            qos_metrics: QosMetrics::default(),
            health_status: HealthStatus::Healthy,
        })
        .collect();

    let service = ServiceInfo {
        name: "test-service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities,
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // ASSERT: Should handle many capabilities
    assert_eq!(service.capabilities.len(), 100);
}

#[test]
fn test_service_info_with_duplicate_capability_names() {
    // ARRANGE & ACT: Create service with duplicate capability names
    let service = create_test_service(
        "test-service",
        "http://localhost:8080",
        vec!["compute", "compute", "compute"],
    );

    // ASSERT: Should allow duplicates (deduplication happens at higher level)
    assert_eq!(service.capabilities.len(), 3);
}

#[test]
fn test_service_info_with_very_long_name() {
    // ARRANGE & ACT: Create service with very long name
    let long_name = "a".repeat(10_000);
    let service = ServiceInfo {
        name: long_name,
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // ASSERT: Should handle long names
    assert_eq!(service.name.len(), 10_000);
}

#[test]
fn test_service_info_with_special_characters_in_name() {
    // ARRANGE: Create services with special characters
    let special_names = vec![
        "service@example.com",
        "service/with/slashes",
        "service:with:colons",
        "service-with-dashes",
        "service_with_underscores",
        "service.with.dots",
        "service#with#hashes",
    ];

    // ACT & ASSERT: All should be allowed
    for name in special_names {
        let service = ServiceInfo {
            name: name.to_string(),
            endpoint: "http://localhost:8080".to_string(),
            primal_type: PrimalType::new("test"),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        assert_eq!(service.name, name);
    }
}

#[test]
fn test_service_info_with_unicode_in_name() {
    // ARRANGE & ACT: Create service with Unicode characters
    let service = ServiceInfo {
        name: "服务-サービス-сервис-خدمة".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    // ASSERT: Should handle Unicode
    assert!(service.name.contains("服务"));
    assert!(service.name.contains("サービス"));
}

// ============================================================================
// PRIMAL TYPE EDGE CASES
// ============================================================================

#[test]
fn test_primal_type_with_empty_category() {
    // ARRANGE & ACT: Create PrimalType with empty category
    let primal_type = PrimalType::new("");

    // ASSERT: Should allow empty category
    assert_eq!(primal_type.category, "");
}

#[test]
fn test_primal_type_with_very_long_category() {
    // ARRANGE & ACT: Create PrimalType with very long category
    let long_category = "a".repeat(10_000);
    let primal_type = PrimalType::new(&long_category);

    // ASSERT: Should handle long categories
    assert_eq!(primal_type.category.len(), 10_000);
}

#[test]
fn test_primal_type_with_special_characters() {
    // ARRANGE: Test various special characters
    let special_categories = vec!["compute@v2", "ai/ml/nlp", "storage:s3", "security#auth"];

    // ACT & ASSERT: All should be allowed
    for category in special_categories {
        let primal_type = PrimalType::new(category);
        assert_eq!(primal_type.category, category);
    }
}

// ============================================================================
// DISCOVERED CAPABILITY EDGE CASES
// ============================================================================

#[test]
fn test_discovered_capability_with_empty_fields() {
    // ARRANGE & ACT: Create capability with empty fields
    let capability = DiscoveredCapability {
        name: String::new(),
        version: String::new(),
        description: String::new(),
        provider: String::new(),
        endpoint: String::new(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    // ASSERT: Should allow empty fields
    assert_eq!(capability.name, "");
    assert_eq!(capability.version, "");
}

#[test]
fn test_discovered_capability_with_very_long_description() {
    // ARRANGE & ACT: Create capability with very long description
    let long_desc = "A".repeat(100_000);
    let capability = DiscoveredCapability {
        name: "test".to_string(),
        version: "1.0".to_string(),
        description: long_desc.clone(),
        provider: "test".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    // ASSERT: Should handle long descriptions
    assert_eq!(capability.description.len(), 100_000);
}

// ============================================================================
// QOS METRICS EDGE CASES
// ============================================================================

#[test]
fn test_qos_metrics_with_all_none() {
    // ARRANGE & ACT: Create QoS metrics with all None values
    let qos = QosMetrics {
        latency_ms: None,
        throughput_ops_sec: None,
        availability: None,
        reliability: None,
    };

    // ASSERT: Should allow all None
    assert!(qos.latency_ms.is_none());
    assert!(qos.throughput_ops_sec.is_none());
    assert!(qos.availability.is_none());
    assert!(qos.reliability.is_none());
}

#[test]
fn test_qos_metrics_with_zero_values() {
    // ARRANGE & ACT: Create QoS metrics with zero values
    let qos = QosMetrics {
        latency_ms: Some(0.0),
        throughput_ops_sec: Some(0.0),
        availability: Some(0.0),
        reliability: Some(0.0),
    };

    // ASSERT: Should allow zero values
    assert_eq!(qos.latency_ms, Some(0.0));
    assert_eq!(qos.throughput_ops_sec, Some(0.0));
}

#[test]
fn test_qos_metrics_with_negative_values() {
    // ARRANGE & ACT: Create QoS metrics with negative values (invalid but allowed)
    let qos = QosMetrics {
        latency_ms: Some(-100.0),
        throughput_ops_sec: Some(-1000.0),
        availability: Some(-0.5),
        reliability: Some(-0.99),
    };

    // ASSERT: Should allow negative values (validation happens elsewhere)
    assert_eq!(qos.latency_ms, Some(-100.0));
}

#[test]
fn test_qos_metrics_with_very_large_values() {
    // ARRANGE & ACT: Create QoS metrics with very large values
    let qos = QosMetrics {
        latency_ms: Some(f64::MAX),
        throughput_ops_sec: Some(f64::MAX),
        availability: Some(1000.0), // Invalid but allowed
        reliability: Some(1000.0),  // Invalid but allowed
    };

    // ASSERT: Should allow very large values
    assert_eq!(qos.latency_ms, Some(f64::MAX));
}

#[test]
fn test_qos_metrics_with_infinity() -> SongbirdResult<()> {
    // ARRANGE & ACT: Create QoS metrics with infinity
    let qos = QosMetrics {
        latency_ms: Some(f64::INFINITY),
        throughput_ops_sec: Some(f64::INFINITY),
        availability: Some(f64::INFINITY),
        reliability: Some(f64::INFINITY),
    };

    // ASSERT: Should allow infinity
    assert!(qos
        .latency_ms
        .ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?
        .is_infinite());
    Ok(())
}

#[test]
fn test_qos_metrics_with_nan() -> SongbirdResult<()> {
    // ARRANGE & ACT: Create QoS metrics with NaN
    let qos = QosMetrics {
        latency_ms: Some(f64::NAN),
        throughput_ops_sec: Some(f64::NAN),
        availability: Some(f64::NAN),
        reliability: Some(f64::NAN),
    };

    // ASSERT: Should allow NaN
    assert!(qos
        .latency_ms
        .ok_or_else(|| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?
        .is_nan());
    Ok(())
}

// ============================================================================
// HEALTH STATUS EDGE CASES
// ============================================================================

#[test]
fn test_health_status_all_variants() {
    // ARRANGE: Test all health status variants
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    // ACT & ASSERT: All should be valid
    for status in statuses {
        let service = ServiceInfo {
            name: "test".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            primal_type: PrimalType::new("test"),
            capabilities: vec![],
            health: status.clone(),
            metadata: HashMap::new(),
        };
        assert_eq!(service.health, status);
    }
}

// ============================================================================
// METADATA EDGE CASES
// ============================================================================

#[test]
fn test_metadata_with_empty_map() {
    // ARRANGE & ACT: Create service with empty metadata
    let service = create_test_service("test", "http://localhost:8080", vec![]);

    // ASSERT: Should allow empty metadata
    assert_eq!(service.metadata.len(), 0);
}

#[test]
fn test_metadata_with_many_entries() {
    // ARRANGE: Create metadata with many entries
    let mut metadata = HashMap::new();
    for i in 0..1000 {
        metadata.insert(format!("key-{}", i), format!("value-{}", i));
    }

    // ACT: Create service with large metadata
    let service = ServiceInfo {
        name: "test".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata,
    };

    // ASSERT: Should handle many metadata entries
    assert_eq!(service.metadata.len(), 1000);
}

#[test]
fn test_metadata_with_empty_keys_and_values() {
    // ARRANGE: Create metadata with empty keys and values
    let mut metadata = HashMap::new();
    metadata.insert("".to_string(), "".to_string());
    metadata.insert("key".to_string(), "".to_string());
    metadata.insert("".to_string(), "value".to_string());

    // ACT: Create service
    let service = ServiceInfo {
        name: "test".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    // ASSERT: Should allow empty keys and values
    assert!(service.metadata.contains_key(""));
}

#[test]
fn test_metadata_with_very_long_keys_and_values() {
    // ARRANGE: Create metadata with very long keys and values
    let long_key = "k".repeat(10_000);
    let long_value = "v".repeat(10_000);
    let mut metadata = HashMap::new();
    metadata.insert(long_key.clone(), long_value.clone());

    // ACT: Create service
    let service = ServiceInfo {
        name: "test".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata,
    };

    // ASSERT: Should handle long keys and values
    assert!(service.metadata.contains_key(&long_key));
    assert_eq!(service.metadata.get(&long_key), Some(&long_value));
}

#[test]
fn test_metadata_with_special_characters() {
    // ARRANGE: Create metadata with special characters
    let mut metadata = HashMap::new();
    metadata.insert("key@#$%".to_string(), "value!@#$%^&*()".to_string());
    metadata.insert("key/with/slashes".to_string(), "value\\with\\backslashes".to_string());
    metadata.insert("key\nwith\nnewlines".to_string(), "value\twith\ttabs".to_string());

    // ACT: Create service
    let service = ServiceInfo {
        name: "test".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        primal_type: PrimalType::new("test"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    // ASSERT: Should handle special characters
    assert!(service.metadata.contains_key("key@#$%"));
}

// ============================================================================
// DISCOVERY CONFIG EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_adapter_with_single_discovery_endpoint() {
    // ARRANGE: Create adapter with single endpoint
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:65534/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = create_universal_adapter_with_config(config);

    // ASSERT: Should create successfully
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);
}

#[tokio::test]
async fn test_adapter_with_duplicate_discovery_endpoints() {
    // ARRANGE: Create adapter with duplicate endpoints
    let endpoint = "http://localhost:65534/discovery".to_string();
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![endpoint.clone(), endpoint.clone(), endpoint.clone()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = create_universal_adapter_with_config(config);

    // ASSERT: Should handle duplicates (may discover same services multiple times)
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[test]
fn test_adapter_config_with_very_short_timeout() {
    // ARRANGE & ACT: Create config with very short timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_nanos(1),
        ..Default::default()
    };

    // ASSERT: Should allow very short timeout
    assert_eq!(config.discovery_timeout.as_nanos(), 1);
}

#[test]
fn test_adapter_config_with_auto_discovery_disabled() {
    // ARRANGE & ACT: Create config with auto-discovery disabled
    let config = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };

    // ASSERT: Should allow disabling auto-discovery
    assert!(!config.auto_discovery);
}

// ============================================================================
// ENDPOINT FORMAT EDGE CASES
// ============================================================================

#[test]
fn test_service_with_various_endpoint_formats() {
    // ARRANGE: Test various endpoint formats
    let endpoints = vec![
        "http://localhost:8080",
        "https://example.com:443",
        "http://127.0.0.1:3000",
        "http://[::1]:8080", // IPv6
        "http://service.local:9000",
        "http://192.168.1.1:8888",
    ];

    // ACT & ASSERT: All should be accepted
    for endpoint in endpoints {
        let service = ServiceInfo {
            name: "test".to_string(),
            endpoint: endpoint.to_string(),
            primal_type: PrimalType::new("test"),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        assert_eq!(service.endpoint, endpoint);
    }
}

#[test]
fn test_service_with_invalid_endpoint_formats() {
    // ARRANGE: Test invalid (but allowed) endpoint formats
    let invalid_endpoints = vec![
        "not-a-url",
        "ftp://invalid-protocol.com",
        "://missing-protocol",
        "http://:8080", // Missing host
        "",             // Empty
    ];

    // ACT & ASSERT: All should be accepted (validation happens elsewhere)
    for endpoint in invalid_endpoints {
        let service = ServiceInfo {
            name: "test".to_string(),
            endpoint: endpoint.to_string(),
            primal_type: PrimalType::new("test"),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        assert_eq!(service.endpoint, endpoint);
    }
}
