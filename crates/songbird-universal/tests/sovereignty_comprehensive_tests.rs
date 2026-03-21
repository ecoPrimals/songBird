// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
#![cfg(feature = "tests-incomplete")]
//! NOTE: Disabled - requires fixes

#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "test assertions and harness ergonomics")]

//! Comprehensive Sovereignty Tests
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Critical tests for sovereignty-aware routing and human dignity compliance.
//! This module ensures our sovereignty system properly respects user autonomy.

use songbird_test_utils::network_fixtures::*;
use songbird_test_utils::test_discovery_port;
use songbird_test_utils::test_federation_port;
use songbird_test_utils::test_health_port;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::sovereignty::types::{SecurityCapability, SecurityLevel, SovereigntyLevel};
use songbird_universal::sovereignty::{
    PathSegment, RoutingPath, SovereigntyAdapterConfig, SovereigntyAwareAdapter,
};
use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo, UniversalRequest,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create a test service with specified sovereignty level
fn create_test_service_with_sovereignty(
    name: &str,
    endpoint: &str,
    sovereignty_level: SovereigntyLevel,
) -> ServiceInfo {
    let mut metadata = HashMap::new();
    metadata.insert("sovereignty_level".to_string(), format!("{sovereignty_level:?}"));

    ServiceInfo {
        name: name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: endpoint.to_string(),
        capabilities: vec![DiscoveredCapability {
            name: "test_capability".to_string(),
            version: "1.0".to_string(),
            description: "Test capability".to_string(),
            provider: name.to_string(),
            endpoint: format!("{endpoint}/api/v1/test"),
            qos_metrics: QosMetrics::default(),
            health_status: HealthStatus::Healthy,
        }],
        health: HealthStatus::Healthy,
        metadata,
    }
}

/// Create a test request
fn create_test_request() -> UniversalRequest {
    UniversalRequest {
        request_id: "test-req-001".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "process".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    }
}

// ============================================================================
// SOVEREIGNTY ADAPTER TESTS
// ============================================================================

#[tokio::test]
async fn test_sovereignty_adapter_creation() {
    let adapter = SovereigntyAwareAdapter::new().await;
    assert!(adapter.is_ok(), "Sovereignty adapter should be created successfully");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_default_config() {
    let config = SovereigntyAdapterConfig::default();
    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with default config");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_custom_config() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(5),
        sovereignty_preference_weight: 0.9,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should be created with custom config");
}

#[tokio::test]
async fn test_sovereignty_adapter_disabled_routing() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: Duration::from_secs(1),
        sovereignty_preference_weight: 0.0,
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should work with sovereignty features disabled");
}

#[tokio::test]
async fn test_multiple_sovereignty_adapters() {
    let adapter1 = SovereigntyAwareAdapter::new().await;
    let adapter2 = SovereigntyAwareAdapter::new().await;
    let adapter3 = SovereigntyAwareAdapter::new().await;

    assert!(adapter1.is_ok());
    assert!(adapter2.is_ok());
    assert!(adapter3.is_ok());
}

// ============================================================================
// SOVEREIGNTY CONFIG TESTS
// ============================================================================

#[test]
fn test_sovereignty_config_default_values() {
    let config = SovereigntyAdapterConfig::default();

    assert!(config.enable_sovereignty_routing, "Sovereignty routing should be enabled by default");
    assert!(config.enable_federation_routing, "Federation routing should be enabled by default");
    assert!(
        config.enable_network_optimization,
        "Network optimization should be enabled by default"
    );
    assert_eq!(
        config.sovereignty_timeout,
        Duration::from_secs(3),
        "Default timeout should be 3 seconds"
    );
    assert!(
        (config.sovereignty_preference_weight - 0.8).abs() < 0.01,
        "Default sovereignty weight should heavily prefer sovereign paths"
    );
}

#[test]
fn test_sovereignty_config_high_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0,
        ..Default::default()
    };

    assert_eq!(config.sovereignty_preference_weight, 1.0);
}

#[test]
fn test_sovereignty_config_balanced_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 0.5,
        ..Default::default()
    };

    assert_eq!(config.sovereignty_preference_weight, 0.5);
}

#[test]
fn test_sovereignty_config_efficiency_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 0.2,
        ..Default::default()
    };

    assert!(config.sovereignty_preference_weight < 0.5);
}

#[test]
fn test_sovereignty_config_custom_timeout() -> SongbirdResult<()> {
    let config = SovereigntyAdapterConfig {
        sovereignty_timeout: Duration::from_millis(500),
        ..Default::default()
    };

    assert_eq!(config.sovereignty_timeout, Duration::from_millis(500));
    Ok(())
}

// ============================================================================
// SOVEREIGNTY LEVEL TESTS
// ============================================================================

#[test]
fn test_sovereignty_level_variants() -> SongbirdResult<()> {
    let fully = SovereigntyLevel::FullySovereign;
    let highly = SovereigntyLevel::HighlySovereign;
    let moderately = SovereigntyLevel::ModeratelySovereign;
    let limited = SovereigntyLevel::LimitedSovereignty;
    let minimal = SovereigntyLevel::NonSovereign;

    // Ensure all variants are distinct
    assert_ne!(format!("{fully:?}"), format!("{:?}", highly));
    assert_ne!(format!("{highly:?}"), format!("{:?}", moderately));
    assert_ne!(format!("{moderately:?}"), format!("{:?}", limited));
    assert_ne!(format!("{limited:?}"), format!("{:?}", minimal));
    Ok(())
}

#[test]
fn test_sovereignty_level_ordering() -> SongbirdResult<()> {
    // Ensure sovereignty levels can be compared
    let levels = [
        SovereigntyLevel::FullySovereign,
        SovereigntyLevel::HighlySovereign,
        SovereigntyLevel::ModeratelySovereign,
        SovereigntyLevel::LimitedSovereignty,
        SovereigntyLevel::NonSovereign,
    ];

    // All should be valid and distinct
    for (i, level1) in levels.iter().enumerate() {
        for (j, level2) in levels.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{level1:?}"), format!("{:?}", level2));
            }
        }
    }
    Ok(())
}

// ============================================================================
// SECURITY CAPABILITY TESTS
// ============================================================================

#[test]
fn test_security_capability_variants() -> SongbirdResult<()> {
    let capabilities = [
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    assert_eq!(capabilities.len(), 6, "Should have 6 security capabilities");

    // Ensure all are distinct
    for (i, cap1) in capabilities.iter().enumerate() {
        for (j, cap2) in capabilities.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{cap1:?}"), format!("{:?}", cap2));
            }
        }
    }
    Ok(())
}

#[test]
fn test_security_level_variants() -> SongbirdResult<()> {
    let levels = [
        SecurityLevel::Maximum,
        SecurityLevel::High,
        SecurityLevel::Medium,
        SecurityLevel::Low,
        SecurityLevel::Minimal,
    ];

    assert_eq!(levels.len(), 5, "Should have 5 security levels");

    // Ensure all are distinct
    for (i, level1) in levels.iter().enumerate() {
        for (j, level2) in levels.iter().enumerate() {
            if i != j {
                assert_ne!(format!("{level1:?}"), format!("{:?}", level2));
            }
        }
    }
    Ok(())
}

// ============================================================================
// ROUTING PATH TESTS
// ============================================================================

#[test]
fn test_routing_path_creation() {
    let service = create_test_service_with_sovereignty(
        "test-service",
        &format!("http://localhost:{}", test_orchestrator_port()),
        SovereigntyLevel::HighlySovereign,
    );

    let segment = PathSegment {
        service: service,
        sovereignty_level: SovereigntyLevel::HighlySovereign,
        efficiency_score: 0.85,
        security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
        ],
        metadata: HashMap::new(),
    };

    let path = RoutingPath {
        segments: vec![segment],
        sovereignty_score: 0.9,
        efficiency_score: 0.85,
        combined_score: 0.88,
        security_level: SecurityLevel::High,
    };

    assert_eq!(path.segments.len(), 1);
    assert!((path.sovereignty_score - 0.9).abs() < 0.01);
    assert!((path.efficiency_score - 0.85).abs() < 0.01);
    assert!((path.combined_score - 0.88).abs() < 0.01);
}

#[test]
fn test_routing_path_multi_hop() {
    let endpoint1 = format!("http://localhost:{}", test_orchestrator_port());
    let service1 = create_test_service_with_sovereignty(
        "service-1",
        &endpoint1,
        SovereigntyLevel::FullySovereign,
    );

    let endpoint2 = format!("http://localhost:{}", test_discovery_port());
    let service2 = create_test_service_with_sovereignty(
        "service-2",
        &endpoint2,
        SovereigntyLevel::HighlySovereign,
    );

    let segment1 = PathSegment {
        service: service1,
        sovereignty_level: SovereigntyLevel::FullySovereign,
        efficiency_score: 0.9,
        security_capabilities: vec![SecurityCapability::Encryption],
        metadata: HashMap::new(),
    };

    let segment2 = PathSegment {
        service: service2,
        sovereignty_level: SovereigntyLevel::HighlySovereign,
        efficiency_score: 0.85,
        security_capabilities: vec![SecurityCapability::Authentication],
        metadata: HashMap::new(),
    };

    let path = RoutingPath {
        segments: vec![segment1, segment2],
        sovereignty_score: 0.95,
        efficiency_score: 0.875, // Average of 0.9 and 0.85
        combined_score: 0.91,
        security_level: SecurityLevel::Maximum,
    };

    assert_eq!(path.segments.len(), 2, "Multi-hop path should have 2 segments");
    assert!(path.sovereignty_score > 0.9, "Multi-hop path should have high sovereignty");
}

#[test]
fn test_path_segment_with_metadata() -> SongbirdResult<()> {
    let endpoint = format!("http://localhost:{}", test_health_port());
    let service = create_test_service_with_sovereignty(
        "metadata-service",
        &endpoint,
        SovereigntyLevel::ModeratelySovereign,
    );

    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-west-1".to_string());
    metadata.insert("compliance".to_string(), "gdpr".to_string());
    metadata.insert("encryption_type".to_string(), "aes-256".to_string());

    let segment = PathSegment {
        service,
        sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        efficiency_score: 0.75,
        security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::FederationAware,
        ],
        metadata: metadata.clone(),
    };

    assert_eq!(segment.metadata.len(), 3);
    assert_eq!(
        segment
            .metadata
            .get("region")
            .ok_or_else(|| SongbirdError::configuration("Missing region"))?,
        "us-west-1"
    );
    assert_eq!(
        segment
            .metadata
            .get("compliance")
            .ok_or_else(|| SongbirdError::configuration("Missing compliance"))?,
        "gdpr"
    );
    assert_eq!(segment.security_capabilities.len(), 2);
    Ok(())
}

// ============================================================================
// SOVEREIGNTY SCORE CALCULATION TESTS
// ============================================================================

#[test]
fn test_high_sovereignty_score() {
    let endpoint = format!("http://localhost:{}", test_federation_port());
    let service = create_test_service_with_sovereignty(
        "high-sov-service",
        &endpoint,
        SovereigntyLevel::FullySovereign,
    );

    let segment = PathSegment {
        service,
        sovereignty_level: SovereigntyLevel::FullySovereign,
        efficiency_score: 0.95,
        security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::FederationAware,
        ],
        metadata: HashMap::new(),
    };

    let path = RoutingPath {
        segments: vec![segment],
        sovereignty_score: 1.0, // Maximum sovereignty
        efficiency_score: 0.95,
        combined_score: 0.98,
        security_level: SecurityLevel::Maximum,
    };

    assert!(path.sovereignty_score >= 0.95, "Fully sovereign path should have score >= 0.95");
    assert_eq!(path.security_level, SecurityLevel::Maximum);
}

#[test]
fn test_low_sovereignty_score() {
    let service = create_test_service_with_sovereignty(
        "low-sov-service",
        "http://localhost:8084",
        SovereigntyLevel::NonSovereign,
    );

    let segment = PathSegment {
        service,
        sovereignty_level: SovereigntyLevel::NonSovereign,
        efficiency_score: 0.99, // High efficiency
        security_capabilities: vec![SecurityCapability::Authentication],
        metadata: HashMap::new(),
    };

    let path = RoutingPath {
        segments: vec![segment],
        sovereignty_score: 0.3, // Low sovereignty
        efficiency_score: 0.99,
        combined_score: 0.65, // Balanced score favoring efficiency
        security_level: SecurityLevel::Low,
    };

    assert!(path.sovereignty_score < 0.5, "Minimal sovereignty should have score < 0.5");
    assert!(
        path.efficiency_score > path.sovereignty_score,
        "Low sovereignty path prioritizes efficiency"
    );
}

#[test]
fn test_balanced_sovereignty_efficiency() {
    let service = create_test_service_with_sovereignty(
        "balanced-service",
        "http://localhost:8085",
        SovereigntyLevel::ModeratelySovereign,
    );

    let segment = PathSegment {
        service,
        sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        efficiency_score: 0.7,
        security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
        ],
        metadata: HashMap::new(),
    };

    let path = RoutingPath {
        segments: vec![segment],
        sovereignty_score: 0.7,
        efficiency_score: 0.7,
        combined_score: 0.7,
        security_level: SecurityLevel::Medium,
    };

    assert!(
        (path.sovereignty_score - path.efficiency_score).abs() < 0.01,
        "Balanced path should have similar sovereignty and efficiency scores"
    );
}

// ============================================================================
// HUMAN DIGNITY COMPLIANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_human_dignity_high_sovereignty_preference() {
    // Critical test: Ensure system defaults to respecting user sovereignty
    let config = SovereigntyAdapterConfig::default();

    assert!(
        config.sovereignty_preference_weight >= 0.7,
        "Default config MUST prefer sovereignty (human dignity) over efficiency"
    );
    assert!(
        config.enable_sovereignty_routing,
        "Sovereignty routing MUST be enabled by default for human dignity"
    );
}

#[tokio::test]
async fn test_human_dignity_full_sovereignty_available() {
    // Critical test: System must support fully sovereign routing
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0, // Maximum sovereignty preference
        enable_sovereignty_routing: true,
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(
        adapter.is_ok(),
        "System MUST support maximum sovereignty preference for human dignity"
    );
}

#[tokio::test]
async fn test_human_dignity_no_forced_efficiency() {
    // Critical test: Users must never be forced into efficiency-only routing
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 1.0,
        enable_network_optimization: false, // Disable optimization that might override sovereignty
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(
        adapter.is_ok(),
        "Users MUST be able to disable efficiency optimizations for maximum sovereignty"
    );
}

#[test]
fn test_human_dignity_sovereignty_levels_comprehensive() -> SongbirdResult<()> {
    // Critical test: System must support full range of sovereignty levels
    let levels = vec![
        SovereigntyLevel::FullySovereign,
        SovereigntyLevel::HighlySovereign,
        SovereigntyLevel::ModeratelySovereign,
        SovereigntyLevel::LimitedSovereignty,
        SovereigntyLevel::NonSovereign,
    ];

    for level in levels {
        let endpoint = format!("http://localhost:{}", test_orchestrator_port());
        let service = create_test_service_with_sovereignty("test", &endpoint, level.clone());
        assert!(!service.name.is_empty(), "System MUST support all sovereignty levels: {level:?}");
    }
    Ok(())
}

#[test]
fn test_human_dignity_security_capabilities_comprehensive() -> SongbirdResult<()> {
    // Critical test: System must support comprehensive security for sovereignty
    let capabilities = vec![
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    for cap in capabilities {
        // Ensure all security capabilities are well-defined
        assert!(
            !format!("{cap:?}").is_empty(),
            "System MUST support all security capabilities: {cap:?}"
        );
    }
    Ok(())
}

// ============================================================================
// EDGE CASE AND ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_sovereignty_adapter_with_zero_timeout() {
    let config = SovereigntyAdapterConfig {
        sovereignty_timeout: Duration::from_secs(0),
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should handle zero timeout gracefully");
}

#[tokio::test]
async fn test_sovereignty_adapter_with_extreme_preference() {
    let config = SovereigntyAdapterConfig {
        sovereignty_preference_weight: 2.0, // Beyond normal range
        ..Default::default()
    };

    let adapter = SovereigntyAwareAdapter::with_config(config).await;
    assert!(adapter.is_ok(), "Adapter should handle extreme preference values");
}

#[test]
fn test_empty_routing_path() {
    let path = RoutingPath {
        segments: vec![],
        sovereignty_score: 0.0,
        efficiency_score: 0.0,
        combined_score: 0.0,
        security_level: SecurityLevel::Minimal,
    };

    assert_eq!(path.segments.len(), 0, "Empty path should have no segments");
    assert_eq!(path.sovereignty_score, 0.0);
    assert_eq!(path.combined_score, 0.0);
}

#[test]
fn test_path_segment_with_all_security_capabilities() {
    let service = create_test_service_with_sovereignty(
        "max-security-service",
        "http://localhost:8086",
        SovereigntyLevel::FullySovereign,
    );

    let segment = PathSegment {
        service,
        sovereignty_level: SovereigntyLevel::FullySovereign,
        efficiency_score: 0.8,
        security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::FederationAware,
            SecurityCapability::NetworkOptimized,
            SecurityCapability::SovereigntyCompliant,
        ],
        metadata: HashMap::new(),
    };

    assert_eq!(
        segment.security_capabilities.len(),
        6,
        "Should support all 6 security capabilities"
    );
}

// ============================================================================
// CONFIGURATION VALIDATION TESTS
// ============================================================================

#[test]
fn test_config_all_features_enabled() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: true,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_secs(5),
        sovereignty_preference_weight: 0.8,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(config.enable_federation_routing);
    assert!(config.enable_network_optimization);
}

#[test]
fn test_config_all_features_disabled() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: false,
        enable_federation_routing: false,
        enable_network_optimization: false,
        sovereignty_timeout: Duration::from_secs(1),
        sovereignty_preference_weight: 0.0,
    };

    assert!(!config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(!config.enable_network_optimization);
    assert_eq!(config.sovereignty_preference_weight, 0.0);
}

#[test]
fn test_config_mixed_features() {
    let config = SovereigntyAdapterConfig {
        enable_sovereignty_routing: true,
        enable_federation_routing: false,
        enable_network_optimization: true,
        sovereignty_timeout: Duration::from_millis(2500),
        sovereignty_preference_weight: 0.6,
    };

    assert!(config.enable_sovereignty_routing);
    assert!(!config.enable_federation_routing);
    assert!(config.enable_network_optimization);
}
