//! Comprehensive tests for type conversion utilities
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
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//!
//! Tests conversions between discovery-specific and universal types

use chrono::Utc;
use songbird_discovery::traits::service::{
    ServiceEndpoint, ServiceInfo as DiscoveryServiceInfo, ServiceStatus,
};
use songbird_universal::{HealthStatus, PrimalType, ServiceInfo as UniversalServiceInfo};
use std::collections::HashMap;

#[test]
fn test_discovery_to_universal_conversion() {
    let now = Utc::now();
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), serde_json::Value::String("value1".to_string()));
    metadata.insert("key2".to_string(), serde_json::Value::Number(42.into()));

    let discovery_service = DiscoveryServiceInfo {
        service_id: "test-service-123".to_string(),
        name: "TestService".to_string(),
        version: "1.0.0".to_string(),
        service_type: "api".to_string(),
        description: Some("Test service description".to_string()),
        endpoints: vec![ServiceEndpoint {
            path: "/api/v1".to_string(),
            method: "GET".to_string(),
            description: Some("API endpoint".to_string()),
            parameters: Vec::new(),
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        }],
        health_check_endpoint: Some("/health".to_string()),
        metadata: metadata.clone(),
        tags: vec!["production".to_string(), "api".to_string()],
        dependencies: vec!["database".to_string()],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "instance-456".to_string(),
        host: "localhost".to_string(),
        port: 8080,
    };

    let universal: UniversalServiceInfo = discovery_service.into();

    assert_eq!(universal.name, "TestService");
    assert_eq!(universal.endpoint, "localhost:8080");
    assert!(!universal.metadata.is_empty());
}

#[test]
fn test_universal_to_discovery_conversion() {
    let mut metadata = HashMap::new();
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("region".to_string(), "us-west".to_string());

    let universal_service = UniversalServiceInfo {
        name: "UniversalService".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "api.example.com:443".to_string(),
        capabilities: Vec::new(), // Capabilities are strings in discovery, not capability types
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    let discovery: DiscoveryServiceInfo = universal_service.into();

    assert_eq!(discovery.name, "UniversalService");
    assert_eq!(discovery.host, "api.example.com");
    assert_eq!(discovery.port, 443);
    assert!(!discovery.service_id.is_empty());
    assert!(!discovery.instance_id.is_empty());
}

#[test]
fn test_endpoint_parsing_with_port() {
    let universal = UniversalServiceInfo {
        name: "PortService".to_string(),
        primal_type: PrimalType::new("api"),
        endpoint: "service.local:9090".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.host, "service.local");
    assert_eq!(discovery.port, 9090);
}

#[test]
fn test_endpoint_parsing_without_port() {
    let universal = UniversalServiceInfo {
        name: "NoPortService".to_string(),
        primal_type: PrimalType::new("api"),
        endpoint: "service.local".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.host, "service.local");
    assert_eq!(discovery.port, songbird_config::defaults::ports::orchestrator_port());
    // Default port (configurable via env)
}

#[test]
fn test_endpoint_parsing_with_url() {
    let universal = UniversalServiceInfo {
        name: "URLService".to_string(),
        primal_type: PrimalType::new("api"),
        endpoint: "https://api.example.com:8443/path".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    // Should extract host and port from URL
    assert!(!discovery.host.is_empty());
    assert!(discovery.port > 0);
}

#[test]
fn test_metadata_conversion_preserves_keys() {
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());
    metadata.insert("key3".to_string(), "value3".to_string());

    let universal = UniversalServiceInfo {
        name: "MetadataService".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.metadata.len(), 3);
    assert!(discovery.metadata.contains_key("key1"));
    assert!(discovery.metadata.contains_key("key2"));
    assert!(discovery.metadata.contains_key("key3"));
}

#[test]
fn test_empty_metadata_conversion() {
    let universal = UniversalServiceInfo {
        name: "EmptyMetadata".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert!(discovery.metadata.is_empty());
}

#[test]
fn test_round_trip_conversion_preserves_name() {
    let original_name = "RoundTripService";

    let universal = UniversalServiceInfo {
        name: original_name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();
    let back_to_universal: UniversalServiceInfo = discovery.into();

    assert_eq!(back_to_universal.name, original_name);
}

#[test]
fn test_conversion_generates_unique_ids() {
    let universal1 = UniversalServiceInfo {
        name: "Service1".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let universal2 = UniversalServiceInfo {
        name: "Service2".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8081".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery1: DiscoveryServiceInfo = universal1.into();
    let discovery2: DiscoveryServiceInfo = universal2.into();

    assert_ne!(discovery1.service_id, discovery2.service_id);
    assert_ne!(discovery1.instance_id, discovery2.instance_id);
}

#[test]
fn test_conversion_sets_default_endpoints() {
    let universal = UniversalServiceInfo {
        name: "DefaultEndpoints".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert!(!discovery.endpoints.is_empty());
    assert!(discovery.health_check_endpoint.is_some());
}

#[test]
fn test_conversion_sets_timestamps() {
    let before = Utc::now();

    let universal = UniversalServiceInfo {
        name: "TimestampService".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    let after = Utc::now();

    assert!(discovery.created_at >= before && discovery.created_at <= after);
    assert!(discovery.updated_at >= before && discovery.updated_at <= after);
}

#[test]
fn test_ipv4_endpoint_parsing() {
    let universal = UniversalServiceInfo {
        name: "IPv4Service".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "192.168.1.100:3000".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.host, "192.168.1.100");
    assert_eq!(discovery.port, 3000);
}

#[test]
fn test_ipv6_endpoint_parsing() {
    let universal = UniversalServiceInfo {
        name: "IPv6Service".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "[::1]:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert!(!discovery.host.is_empty());
    assert!(discovery.port > 0);
}

#[test]
fn test_conversion_handles_special_characters_in_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("key-with-dash".to_string(), "value".to_string());
    metadata.insert("key_with_underscore".to_string(), "value".to_string());
    metadata.insert("key.with.dot".to_string(), "value".to_string());

    let universal = UniversalServiceInfo {
        name: "SpecialChars".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata,
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.metadata.len(), 3);
}

#[test]
fn test_conversion_handles_long_service_names() {
    let long_name = "a".repeat(200);

    let universal = UniversalServiceInfo {
        name: long_name.clone(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.name, long_name);
}

#[test]
fn test_conversion_handles_high_port_numbers() {
    let universal = UniversalServiceInfo {
        name: "HighPort".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:65535".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.port, 65535);
}

#[test]
fn test_conversion_sets_running_status() {
    let universal = UniversalServiceInfo {
        name: "StatusService".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    assert_eq!(discovery.status, ServiceStatus::Running);
}

#[test]
fn test_conversion_creates_valid_uuid_ids() {
    let universal = UniversalServiceInfo {
        name: "UUIDService".to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: "localhost:8080".to_string(),
        capabilities: Vec::new(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let discovery: DiscoveryServiceInfo = universal.into();

    // Verify service_id is a valid UUID
    assert!(uuid::Uuid::parse_str(&discovery.service_id).is_ok());
    assert!(uuid::Uuid::parse_str(&discovery.instance_id).is_ok());
}
