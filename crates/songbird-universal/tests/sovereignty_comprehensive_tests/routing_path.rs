// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// ROUTING PATH TESTS
// ============================================================================

use crate::helpers::create_test_service_with_sovereignty;
use crate::imports::*;

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
