// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// SOVEREIGNTY SCORE CALCULATION TESTS
// ============================================================================

use crate::helpers::create_test_service_with_sovereignty;
use crate::imports::*;

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
