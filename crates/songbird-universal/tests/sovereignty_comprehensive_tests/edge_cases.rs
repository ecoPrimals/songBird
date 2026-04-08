// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ============================================================================
// EDGE CASE AND ERROR HANDLING TESTS
// ============================================================================

use crate::helpers::create_test_service_with_sovereignty;
use crate::imports::*;

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
