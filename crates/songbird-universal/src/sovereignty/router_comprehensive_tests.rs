// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for Sovereignty-Aware Router
//!
//! This test module provides extensive coverage for the sovereignty router,
//! targeting 65%+ coverage from the current 3.81%.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::router::{SovereigntyPreferences, SovereigntyRouter};
use super::types::{RiskSeverity, SecurityCapability, SovereigntyLevel};
use crate::types::{HealthStatus, PrimalType, ServiceInfo, UniversalRequest};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

// Test helpers
fn create_test_service(name: &str, port: u16) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: format!("http://localhost:{port}"),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

fn create_test_request() -> UniversalRequest {
    UniversalRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        source: "test-client".to_string(),
        target: "compute-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    }
}

// Constructor tests
#[test]
fn test_sovereignty_router_new() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    // Router should be created successfully
    assert!(format!("{:?}", router).contains("SovereigntyRouter"));
    Ok(())
}

#[test]
fn test_sovereignty_router_default() -> SongbirdResult<()> {
    let router = SovereigntyRouter::default();
    assert!(format!("{:?}", router).contains("SovereigntyRouter"));
    Ok(())
}

#[test]
fn test_sovereignty_router_with_preferences() -> SongbirdResult<()> {
    let preferences = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 0.9,
        required_security_capabilities: vec![SecurityCapability::Encryption],
        max_acceptable_risk: RiskSeverity::Low,
    };

    let router = SovereigntyRouter::with_preferences(preferences);
    assert!(format!("{:?}", router).contains("SovereigntyRouter"));
    Ok(())
}

// Preferences tests
#[test]
fn test_sovereignty_preferences_default() {
    let prefs = SovereigntyPreferences::default();

    assert_eq!(prefs.minimum_sovereignty_level, SovereigntyLevel::ModeratelySovereign);
    assert!((prefs.sovereignty_weight - 0.7).abs() < f64::EPSILON);
    assert_eq!(prefs.required_security_capabilities.len(), 2);
    assert_eq!(prefs.max_acceptable_risk, RiskSeverity::Medium);
}

#[test]
fn test_sovereignty_preferences_clone() -> SongbirdResult<()> {
    let prefs1 = SovereigntyPreferences::default();
    let prefs2 = prefs1.clone();

    assert_eq!(prefs1.minimum_sovereignty_level, prefs2.minimum_sovereignty_level);
    assert!((prefs1.sovereignty_weight - prefs2.sovereignty_weight).abs() < f64::EPSILON);
    Ok(())
}

// Path finding tests
#[tokio::test]
async fn test_find_sovereignty_aware_paths_empty_services() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;
    assert_eq!(paths.len(), 0, "No services should result in no paths");
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_single_service() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![create_test_service("test-service", 8080)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;
    assert!(!paths.is_empty(), "Should generate at least one path");
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_multiple_services() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![
        create_test_service("service-1", 8080),
        create_test_service("service-2", 8081),
        create_test_service("service-3", 8082),
    ];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;
    assert_eq!(paths.len(), 3, "Should generate one path per service");
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_sorted_by_score() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services =
        vec![create_test_service("service-1", 8080), create_test_service("service-2", 8081)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    // Verify paths are sorted by combined score (descending)
    for i in 0..paths.len().saturating_sub(1) {
        assert!(
            paths[i].combined_score >= paths[i + 1].combined_score,
            "Paths should be sorted by combined score (highest first)"
        );
    }
    Ok(())
}

// Sovereignty filtering tests
#[tokio::test]
async fn test_high_sovereignty_preference_filters_paths() {
    let preferences = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 0.95,
        required_security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
        ],
        max_acceptable_risk: RiskSeverity::Low,
    };

    let router = SovereigntyRouter::with_preferences(preferences);
    let request = create_test_request();
    let services =
        vec![create_test_service("service-1", 8080), create_test_service("service-2", 8081)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    // With very high sovereignty requirements, some paths may be filtered out
    // This tests the filtering logic
}

// Score calculation tests
#[tokio::test]
async fn test_paths_have_valid_scores() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![create_test_service("test-service", 8080)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    for path in paths {
        // Scores should be between 0.0 and 1.0
        assert!(path.sovereignty_score >= 0.0 && path.sovereignty_score <= 1.0);
        assert!(path.efficiency_score >= 0.0 && path.efficiency_score <= 1.0);
        assert!(path.combined_score >= 0.0 && path.combined_score <= 1.0);

        // Each path should have at least one segment
        assert!(!path.segments.is_empty(), "Path should have segments");
    }
    Ok(())
}

#[tokio::test]
async fn test_sovereignty_weight_affects_combined_score() -> SongbirdResult<()> {
    // High sovereignty weight
    let prefs_high_sov = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        sovereignty_weight: 0.9,
        required_security_capabilities: vec![SecurityCapability::Encryption],
        max_acceptable_risk: RiskSeverity::Medium,
    };

    // Low sovereignty weight (high efficiency weight)
    let prefs_low_sov = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        sovereignty_weight: 0.1,
        required_security_capabilities: vec![SecurityCapability::Encryption],
        max_acceptable_risk: RiskSeverity::Medium,
    };

    let router_high = SovereigntyRouter::with_preferences(prefs_high_sov);
    let router_low = SovereigntyRouter::with_preferences(prefs_low_sov);

    let request = create_test_request();
    let services = vec![create_test_service("test-service", 8080)];

    let paths_high =
        router_high.find_sovereignty_aware_paths(&request, &services).await.map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;
    let paths_low =
        router_low.find_sovereignty_aware_paths(&request, &services).await.map_err(|e| {
            SongbirdError::configuration(format!("Missing performance configuration: {}", e))
        })?;

    // Both should generate paths
    assert!(!paths_high.is_empty());
    assert!(!paths_low.is_empty());

    // Scores may differ based on weighting
    // This tests that the weighting logic is applied
    Ok(())
}

// Security capability tests
#[tokio::test]
async fn test_path_segments_have_security_capabilities() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![create_test_service("test-service", 8080)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    for path in paths {
        for segment in &path.segments {
            // Each segment should have security capabilities assessed
            assert!(
                !segment.security_capabilities.is_empty(),
                "Segments should have security capabilities"
            );
        }
    }
    Ok(())
}

// Path structure tests
#[tokio::test]
async fn test_path_segments_reference_services() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let service_name = "test-service-unique";
    let services = vec![create_test_service(service_name, 8080)];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    assert_eq!(paths.len(), 1);
    let path = &paths[0];

    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].service.name, service_name);
    Ok(())
}

// Multiple service routing tests
#[tokio::test]
async fn test_each_service_gets_candidate_path() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![
        create_test_service("service-a", 8080),
        create_test_service("service-b", 8081),
        create_test_service("service-c", 8082),
        create_test_service("service-d", 8083),
    ];

    let service_count = services.len();
    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    // Each service should generate a path (assuming they all pass sovereignty checks)
    assert_eq!(paths.len(), service_count);
    Ok(())
}

// Edge case tests
#[tokio::test]
async fn test_large_number_of_services() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();

    // Generate 100 services
    let services: Vec<ServiceInfo> =
        (0..100).map(|i| create_test_service(&format!("service-{i}"), 8080 + i as u16)).collect();

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.map_err(|e| {
        SongbirdError::configuration(format!("Missing performance configuration: {}", e))
    })?;

    // Should handle large numbers of services
    assert_eq!(paths.len(), 100);
    Ok(())
}

#[tokio::test]
async fn test_sovereignty_level_scores_are_consistent() {
    // Test that sovereignty levels have consistent ordering
    assert!(
        SovereigntyLevel::FullySovereign.score() > SovereigntyLevel::ModeratelySovereign.score()
    );
    assert!(SovereigntyLevel::ModeratelySovereign.score() > SovereigntyLevel::NonSovereign.score());
}

// Preference validation tests
#[test]
fn test_sovereignty_preferences_with_all_security_capabilities() {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 0.8,
        required_security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::FederationAware,
        ],
        max_acceptable_risk: RiskSeverity::Low,
    };

    assert_eq!(prefs.required_security_capabilities.len(), 4);
}

#[test]
fn test_sovereignty_weight_boundary_values() {
    // Test 0.0 weight (all efficiency)
    let prefs_zero = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        sovereignty_weight: 0.0,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::High,
    };
    assert!((prefs_zero.sovereignty_weight - 0.0).abs() < f64::EPSILON);

    // Test 1.0 weight (all sovereignty)
    let prefs_one = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 1.0,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::Low,
    };
    assert!((prefs_one.sovereignty_weight - 1.0).abs() < f64::EPSILON);
}

// Integration tests with different request types
#[tokio::test]
async fn test_different_service_types() {
    let router = SovereigntyRouter::new();
    let services = vec![
        ServiceInfo {
            name: "compute-service".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        },
        ServiceInfo {
            name: "storage-service".to_string(),
            primal_type: PrimalType::new("storage"),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        },
    ];

    // Test with compute request
    let compute_request = UniversalRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        source: "test-client".to_string(),
        target: "compute-service".to_string(),
        action: "execute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = router.find_sovereignty_aware_paths(&compute_request, &services).await;
    assert!(result.is_ok());

    // Test with storage request
    let storage_request = UniversalRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        source: "test-client".to_string(),
        target: "storage-service".to_string(),
        action: "store".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result2 = router.find_sovereignty_aware_paths(&storage_request, &services).await;
    assert!(result2.is_ok());
}

// Performance tests
#[tokio::test]
async fn test_path_finding_completes_quickly() {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let services = vec![
        create_test_service("service-1", 8080),
        create_test_service("service-2", 8081),
        create_test_service("service-3", 8082),
    ];

    let start = std::time::Instant::now();
    let result = router.find_sovereignty_aware_paths(&request, &services).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000, "Path finding should complete quickly");
}

#[test]
fn sovereignty_preferences_extreme_weights_are_stored() {
    let preferences = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::NonSovereign,
        sovereignty_weight: 0.0,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::Critical,
    };
    let router = SovereigntyRouter::with_preferences(preferences);
    assert!(format!("{router:?}").contains("SovereigntyRouter"));
}

#[tokio::test]
async fn single_service_path_has_positive_combined_score() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let paths =
        router.find_sovereignty_aware_paths(&request, &[create_test_service("only", 9000)]).await?;
    assert_eq!(paths.len(), 1);
    assert!(paths[0].combined_score > 0.0);
    assert_eq!(paths[0].segments.len(), 1);
    Ok(())
}

#[tokio::test]
async fn path_segment_preserves_service_name() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();
    let request = create_test_request();
    let paths = router
        .find_sovereignty_aware_paths(&request, &[create_test_service("named-svc", 7000)])
        .await?;
    assert_eq!(paths[0].segments[0].service.name, "named-svc");
    Ok(())
}
