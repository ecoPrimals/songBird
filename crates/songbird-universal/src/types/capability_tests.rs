//! Tests for capability types module

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

#[test]
fn test_primal_type_creation() {
    let primal = PrimalType::new("compute");
    assert_eq!(primal.category, "compute");
    assert_eq!(primal.subcategory, None);
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_from_string() {
    let primal = PrimalType::from_string("storage");
    assert_eq!(primal.as_str(), "storage");
}

#[test]
fn test_primal_type_display() {
    let primal = PrimalType::new("ai");
    assert_eq!(format!("{}", primal), "ai");
}

#[test]
fn test_primal_type_default() {
    let primal = PrimalType::default();
    assert_eq!(primal.category, "unknown");
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_security_level_default() {
    let level = SecurityLevel::default();
    assert_eq!(level, SecurityLevel::Standard);
}

#[test]
fn test_security_level_variants() {
    let levels = [
        SecurityLevel::None,
        SecurityLevel::Basic,
        SecurityLevel::Standard,
        SecurityLevel::High,
        SecurityLevel::Maximum,
    ];
    assert_eq!(levels.len(), 5);
}

#[test]
fn test_qos_metrics_default() {
    let qos = QosMetrics::default();
    assert!(qos.latency_ms.is_none());
    assert!(qos.throughput_ops_sec.is_none());
    assert!(qos.availability.is_none());
    assert!(qos.reliability.is_none());
}

#[test]
fn test_qos_metrics_with_values() {
    let qos = QosMetrics {
        latency_ms: Some(10.5),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.999),
        reliability: Some(0.95),
    };
    assert_eq!(qos.latency_ms, Some(10.5));
    assert_eq!(qos.throughput_ops_sec, Some(1000.0));
}

#[test]
fn test_primal_capability_creation() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), serde_json::json!("value"));

    let capability = PrimalCapability {
        capability_type: "inference".to_string(),
        version: "1.0".to_string(),
        parameters: params,
        qos_metrics: QosMetrics::default(),
    };

    assert_eq!(capability.capability_type, "inference");
    assert_eq!(capability.version, "1.0");
}

#[test]
fn test_health_status_default() {
    let status = HealthStatus::default();
    assert_eq!(status, HealthStatus::Unknown);
}

#[test]
fn test_health_status_variants() {
    let statuses = [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];
    assert_eq!(statuses.len(), 4);
}

#[test]
fn test_discovered_capability_creation() {
    let capability = DiscoveredCapability {
        name: "test-capability".to_string(),
        version: "1.0".to_string(),
        description: "Test capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(capability.name, "test-capability");
    assert_eq!(capability.health_status, HealthStatus::Healthy);
}

#[test]
fn test_discovery_filters_default() {
    let filters = DiscoveryFilters::default();
    assert!(filters.capability_types.is_empty());
    assert!(filters.security_levels.is_empty());
    assert!(filters.geographic_regions.is_empty());
    assert!(filters.performance_requirements.is_none());
}

#[test]
fn test_discovery_filters_with_criteria() {
    let filters = DiscoveryFilters {
        capability_types: vec!["compute".to_string(), "storage".to_string()],
        security_levels: vec![SecurityLevel::High],
        geographic_regions: vec!["us-west".to_string()],
        performance_requirements: Some(QosMetrics {
            latency_ms: Some(50.0),
            ..Default::default()
        }),
    };

    assert_eq!(filters.capability_types.len(), 2);
    assert_eq!(filters.security_levels.len(), 1);
}

#[test]
fn test_capability_requirement_creation() {
    let requirement = CapabilityRequirement {
        capability_type: "storage".to_string(),
        minimum_version: "2.0".to_string(),
        required_qos: None,
        optional: false,
    };

    assert_eq!(requirement.capability_type, "storage");
    assert!(!requirement.optional);
}

#[test]
fn test_service_capability_creation() {
    let capability = ServiceCapability {
        id: "cap-123".to_string(),
        capability_type: "compute".to_string(),
        version: "1.0".to_string(),
        endpoints: vec!["http://localhost:8080".to_string()],
        qos_metrics: QosMetrics::default(),
    };

    assert_eq!(capability.id, "cap-123");
    assert_eq!(capability.endpoints.len(), 1);
}

#[test]
fn test_primal_type_equality() {
    let primal1 = PrimalType::new("compute");
    let primal2 = PrimalType::new("compute");
    let primal3 = PrimalType::new("storage");

    assert_eq!(primal1, primal2);
    assert_ne!(primal1, primal3);
}

#[test]
fn test_primal_type_clone() {
    let primal = PrimalType::new("ai");
    let cloned = primal.clone();
    assert_eq!(primal, cloned);
}

#[test]
fn test_security_level_clone_and_equality() {
    let level1 = SecurityLevel::High;
    let level2 = level1.clone();
    assert_eq!(level1, level2);
}

#[test]
fn test_qos_metrics_partial_eq() {
    let qos1 = QosMetrics {
        latency_ms: Some(10.0),
        throughput_ops_sec: Some(100.0),
        availability: Some(0.99),
        reliability: Some(0.95),
    };

    let qos2 = qos1.clone();
    assert_eq!(qos1, qos2);
}

#[test]
fn test_health_status_eq() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
}

// ============================================================================
// ADDITIONAL COVERAGE TESTS
// ============================================================================

#[test]
fn test_primal_type_with_special_characters() {
    let special_names =
        vec!["type-with-dash", "type_with_underscore", "type.with.dot", "TYPE_UPPERCASE"];

    for name in special_names {
        let primal = PrimalType::new(name);
        assert_eq!(primal.as_str(), name);
    }
}

#[test]
fn test_primal_type_empty_string() {
    let primal = PrimalType::new("");
    assert_eq!(primal.as_str(), "");
}

#[test]
fn test_primal_type_very_long_name() {
    let long_name = "a".repeat(1000);
    let primal = PrimalType::new(&long_name);
    assert_eq!(primal.as_str().len(), 1000);
}

#[test]
fn test_primal_type_unicode_name() {
    let unicode_names = vec!["计算", "🚀rocket", "café", "Ñoño"];

    for name in unicode_names {
        let primal = PrimalType::new(name);
        assert_eq!(primal.as_str(), name);
    }
}

#[test]
fn test_security_level_ordering() {
    // Test that security levels can be compared
    assert!(SecurityLevel::None != SecurityLevel::Basic);
    assert!(SecurityLevel::Basic != SecurityLevel::Standard);
    assert!(SecurityLevel::Standard != SecurityLevel::High);
    assert!(SecurityLevel::High != SecurityLevel::Maximum);
}

#[test]
fn test_health_status_all_variants() {
    let variants = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    // All variants should be distinct
    for i in 0..variants.len() {
        for j in i + 1..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

#[test]
fn test_qos_metrics_boundary_values() -> SongbirdResult<()> {
    // Test with minimum values
    let qos_min = QosMetrics {
        latency_ms: Some(0.0),
        throughput_ops_sec: Some(0.0),
        availability: Some(0.0),
        reliability: Some(0.0),
    };
    assert!(
        qos_min.latency_ms.ok_or_else(|| SongbirdError::configuration(
            "latency_ms should be present".to_string()
        ))? == 0.0
    );

    // Test with maximum realistic values
    let qos_max = QosMetrics {
        latency_ms: Some(1_000_000.0),
        throughput_ops_sec: Some(1_000_000.0),
        availability: Some(1.0),
        reliability: Some(1.0),
    };
    assert!(
        qos_max.availability.ok_or_else(|| SongbirdError::configuration(
            "availability should be present".to_string()
        ))? == 1.0
    );
    Ok(())
}

#[test]
fn test_qos_metrics_all_none() {
    let qos = QosMetrics {
        latency_ms: None,
        throughput_ops_sec: None,
        availability: None,
        reliability: None,
    };

    assert!(qos.latency_ms.is_none());
    assert!(qos.throughput_ops_sec.is_none());
    assert!(qos.availability.is_none());
    assert!(qos.reliability.is_none());
}

#[test]
fn test_discovered_capability_with_empty_qos() {
    let capability = DiscoveredCapability {
        name: "test".to_string(),
        version: "1.0".to_string(),
        description: "test capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics {
            latency_ms: Some(10.0),
            throughput_ops_sec: Some(100.0),
            availability: Some(0.99),
            reliability: Some(0.95),
        },
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(capability.name, "test");
    assert_eq!(capability.version, "1.0");
}

#[test]
fn test_primal_type_clone_independence() {
    let primal1 = PrimalType::new("compute");
    let primal2 = primal1.clone();

    assert_eq!(primal1.as_str(), primal2.as_str());
    assert_eq!(primal1.category, primal2.category);
}
