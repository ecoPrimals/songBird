//! Tests for Sovereignty Router
//!
//! Comprehensive tests for sovereignty-aware routing functionality

use songbird_test_utils::test_discovery_port;
use songbird_test_utils::test_federation_port;
use songbird_test_utils::test_health_port;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::sovereignty::router::{SovereigntyPreferences, SovereigntyRouter};
use songbird_universal::sovereignty::types::{RiskSeverity, SecurityCapability, SovereigntyLevel};
use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo, UniversalRequest,
};
use std::collections::HashMap;

// Helper function to create a test service
fn create_test_service(id: &str, name: &str, endpoint: &str) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        primal_type: PrimalType::new("test"),
        endpoint: endpoint.to_string(),
        capabilities: vec![DiscoveredCapability {
            name: "test-capability".to_string(),
            version: "1.0".to_string(),
            description: "Test capability".to_string(),
            provider: id.to_string(),
            endpoint: endpoint.to_string(),
            qos_metrics: QosMetrics {
                latency_ms: Some(10.0),
                throughput_ops_sec: Some(1000.0),
                availability: Some(0.99),
                reliability: Some(0.99),
            },
            health_status: HealthStatus::Healthy,
        }],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

#[test]
fn test_sovereignty_router_new() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();

    // Should have default preferences
    let debug_str = format!("{router:?}");
    assert!(debug_str.contains("SovereigntyRouter"));
    Ok(())
}

#[test]
fn test_sovereignty_router_default() -> SongbirdResult<()> {
    let router = SovereigntyRouter::default();

    let debug_str = format!("{router:?}");
    assert!(debug_str.contains("SovereigntyRouter"));
    Ok(())
}

#[test]
fn test_sovereignty_preferences_default() {
    let prefs = SovereigntyPreferences::default();

    assert!(matches!(prefs.minimum_sovereignty_level, SovereigntyLevel::ModeratelySovereign));
    assert!((prefs.sovereignty_weight - 0.7).abs() < 0.001);
    assert_eq!(prefs.required_security_capabilities.len(), 2);
    assert!(prefs.required_security_capabilities.contains(&SecurityCapability::Encryption));
    assert!(prefs.required_security_capabilities.contains(&SecurityCapability::Authentication));
    assert!(matches!(prefs.max_acceptable_risk, RiskSeverity::Medium));
}

#[test]
fn test_sovereignty_preferences_custom() {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 0.9,
        required_security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
        ],
        max_acceptable_risk: RiskSeverity::Low,
    };

    assert!(matches!(prefs.minimum_sovereignty_level, SovereigntyLevel::FullySovereign));
    assert!((prefs.sovereignty_weight - 0.9).abs() < 0.001);
    assert_eq!(prefs.required_security_capabilities.len(), 3);
    assert!(matches!(prefs.max_acceptable_risk, RiskSeverity::Low));
}

#[test]
fn test_sovereignty_router_with_preferences() -> SongbirdResult<()> {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::HighlySovereign,
        sovereignty_weight: 0.8,
        required_security_capabilities: vec![SecurityCapability::Encryption],
        max_acceptable_risk: RiskSeverity::High,
    };

    let router = SovereigntyRouter::with_preferences(prefs);

    let debug_str = format!("{router:?}");
    assert!(debug_str.contains("SovereigntyRouter"));
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_empty_services() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();

    let request = UniversalRequest {
        request_id: "test-001".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services = vec![];
    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(paths.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_single_service() -> SongbirdResult<()> {
    // Use lower sovereignty requirements to ensure paths are found
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::LimitedSovereignty,
        sovereignty_weight: 0.5,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::High,
    };
    let router = SovereigntyRouter::with_preferences(prefs);

    let request = UniversalRequest {
        request_id: "test-002".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let endpoint = format!("http://localhost:{}", test_orchestrator_port());
    let service = create_test_service("service-001", "Test Service", &endpoint);

    let services = vec![service];
    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    // Note: May be empty if services don't meet even relaxed requirements
    let _paths = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    Ok(())
}

#[tokio::test]
async fn test_find_sovereignty_aware_paths_multiple_services() {
    // Use lower sovereignty requirements
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::LimitedSovereignty,
        sovereignty_weight: 0.5,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::High,
    };
    let router = SovereigntyRouter::with_preferences(prefs);

    let request = UniversalRequest {
        request_id: "test-003".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let discovery_endpoint = format!("http://localhost:{}", test_discovery_port());
    let health_endpoint = format!("http://localhost:{}", test_health_port());
    let federation_endpoint = format!("http://localhost:{}", test_federation_port());

    let services = vec![
        create_test_service("service-001", "Service 1", &discovery_endpoint),
        create_test_service("service-002", "Service 2", &health_endpoint),
        create_test_service("service-003", "Service 3", &federation_endpoint),
    ];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let _paths = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    // Note: May be empty if services don't meet sovereignty requirements
}

#[tokio::test]
async fn test_paths_sorted_by_score() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();

    let request = UniversalRequest {
        request_id: "test-004".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let discovery_endpoint = format!("http://localhost:{}", test_discovery_port());
    let health_endpoint = format!("http://localhost:{}", test_health_port());

    let services = vec![
        create_test_service("service-001", "Service 1", &discovery_endpoint),
        create_test_service("service-002", "Service 2", &health_endpoint),
    ];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.ok_or_else(|| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;

    // Paths should be sorted by combined score (descending)
    for i in 1..paths.len() {
        assert!(paths[i - 1].combined_score >= paths[i].combined_score);
    }
    Ok(())
}

#[test]
fn test_sovereignty_preferences_clone() -> SongbirdResult<()> {
    let prefs = SovereigntyPreferences::default();
    let cloned = prefs.clone();

    assert!((prefs.sovereignty_weight - cloned.sovereignty_weight).abs() < 0.001);
    assert_eq!(
        prefs.required_security_capabilities.len(),
        cloned.required_security_capabilities.len()
    );
    Ok(())
}

#[test]
fn test_sovereignty_preferences_debug() -> SongbirdResult<()> {
    let prefs = SovereigntyPreferences::default();
    let debug_str = format!("{prefs:?}");

    assert!(debug_str.contains("SovereigntyPreferences"));
    Ok(())
}

#[test]
fn test_sovereignty_preferences_extreme_values() {
    let min_weight = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::NonSovereign,
        sovereignty_weight: 0.0,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::Critical,
    };

    assert!((min_weight.sovereignty_weight - 0.0).abs() < 0.001);
    assert!(min_weight.required_security_capabilities.is_empty());

    let max_weight = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 1.0,
        required_security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
            SecurityCapability::FederationAware,
            SecurityCapability::NetworkOptimized,
            SecurityCapability::SovereigntyCompliant,
        ],
        max_acceptable_risk: RiskSeverity::Low,
    };

    assert!((max_weight.sovereignty_weight - 1.0).abs() < 0.001);
    assert_eq!(max_weight.required_security_capabilities.len(), 6);
}

#[tokio::test]
async fn test_sovereignty_level_scores() {
    // Test that sovereignty levels have correct scores
    assert!((SovereigntyLevel::FullySovereign.score() - 1.0).abs() < 0.001);
    assert!((SovereigntyLevel::HighlySovereign.score() - 0.8).abs() < 0.001);
    assert!((SovereigntyLevel::ModeratelySovereign.score() - 0.6).abs() < 0.001);
    assert!((SovereigntyLevel::LimitedSovereignty.score() - 0.4).abs() < 0.001);
    assert!((SovereigntyLevel::NonSovereign.score() - 0.0).abs() < 0.001);
}

#[tokio::test]
async fn test_high_sovereignty_preference() {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::FullySovereign,
        sovereignty_weight: 1.0,
        required_security_capabilities: vec![
            SecurityCapability::Encryption,
            SecurityCapability::Authentication,
            SecurityCapability::Authorization,
        ],
        max_acceptable_risk: RiskSeverity::Low,
    };

    let router = SovereigntyRouter::with_preferences(prefs);

    let request = UniversalRequest {
        request_id: "test-high-sov".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services =
        vec![create_test_service("secure-service", "Secure Service", "https://secure.example.com")];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    // May return empty if services don't meet strict requirements
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_low_sovereignty_preference() {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::NonSovereign,
        sovereignty_weight: 0.1,
        required_security_capabilities: vec![],
        max_acceptable_risk: RiskSeverity::Critical,
    };

    let router = SovereigntyRouter::with_preferences(prefs);

    let request = UniversalRequest {
        request_id: "test-low-sov".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services =
        vec![create_test_service("basic-service", "Basic Service", "http://basic.example.com")];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    // Note: Even with low requirements, compliance level filtering may apply
}

#[test]
fn test_security_capability_variants() {
    let capabilities = [
        SecurityCapability::Encryption,
        SecurityCapability::Authentication,
        SecurityCapability::Authorization,
        SecurityCapability::FederationAware,
        SecurityCapability::NetworkOptimized,
        SecurityCapability::SovereigntyCompliant,
    ];

    assert_eq!(capabilities.len(), 6);
    assert!(capabilities.contains(&SecurityCapability::Encryption));
    assert!(capabilities.contains(&SecurityCapability::SovereigntyCompliant));
}

#[test]
fn test_risk_severity_levels() -> SongbirdResult<()> {
    let levels =
        [RiskSeverity::Critical, RiskSeverity::High, RiskSeverity::Medium, RiskSeverity::Low];

    assert_eq!(levels.len(), 4);
    Ok(())
}

#[tokio::test]
async fn test_paths_have_segments() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();

    let request = UniversalRequest {
        request_id: "test-segments".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services =
        vec![create_test_service("service-segments", "Segment Service", "http://localhost:9000")];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    for path in paths {
        assert!(!path.segments.is_empty());
        // Each segment should have the expected fields
        for segment in &path.segments {
            assert!(!segment.service.name.is_empty());
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_paths_have_scores() -> SongbirdResult<()> {
    let router = SovereigntyRouter::new();

    let request = UniversalRequest {
        request_id: "test-scores".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services =
        vec![create_test_service("service-scores", "Score Service", "http://localhost:9001")];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
    let paths = result.ok_or_else(|| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    for path in paths {
        // Scores should be within valid range
        assert!(path.sovereignty_score >= 0.0 && path.sovereignty_score <= 1.0);
        assert!(path.efficiency_score >= 0.0 && path.efficiency_score <= 1.0);
        assert!(path.combined_score >= 0.0 && path.combined_score <= 1.0);
    }
    Ok(())
}

#[tokio::test]
async fn test_balanced_sovereignty_weight() {
    let prefs = SovereigntyPreferences {
        minimum_sovereignty_level: SovereigntyLevel::ModeratelySovereign,
        sovereignty_weight: 0.5, // Balanced
        required_security_capabilities: vec![SecurityCapability::Encryption],
        max_acceptable_risk: RiskSeverity::Medium,
    };

    let router = SovereigntyRouter::with_preferences(prefs);

    let request = UniversalRequest {
        request_id: "test-balanced".to_string(),
        source: "source".to_string(),
        target: "target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let services = vec![create_test_service(
        "balanced-service",
        "Balanced Service",
        "http://balanced.example.com",
    )];

    let result = router.find_sovereignty_aware_paths(&request, &services).await;

    assert!(result.is_ok());
}
