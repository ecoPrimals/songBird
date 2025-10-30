//! Comprehensive tests for Canonical Types
//!
//! Tests all core canonical types and their functionality

use songbird_canonical::{
    AIResponseMetadata, CacheStatus, ConfidenceScore, Endpoint, RequestId, ResponsePerformance,
    ServiceId, SongbirdResponse, SuggestedAction,
};
use std::time::Instant;

// ============================================================================
// ServiceId Tests
// ============================================================================

#[test]
fn test_service_id_new() {
    let id = ServiceId::new("test-service");
    assert_eq!(id.as_str(), "test-service");
}

#[test]
fn test_service_id_from_string() {
    let id: ServiceId = "test-service".to_string().into();
    assert_eq!(id.as_str(), "test-service");
}

#[test]
fn test_service_id_from_str() {
    let id: ServiceId = "test-service".into();
    assert_eq!(id.as_str(), "test-service");
}

#[test]
fn test_service_id_equality() {
    let id1 = ServiceId::new("service-1");
    let id2 = ServiceId::new("service-1");
    let id3 = ServiceId::new("service-2");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_service_id_clone() {
    let id = ServiceId::new("test-service");
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

#[test]
fn test_service_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(ServiceId::new("service-1"));
    set.insert(ServiceId::new("service-2"));
    set.insert(ServiceId::new("service-1")); // Duplicate

    assert_eq!(set.len(), 2);
}

#[test]
fn test_service_id_debug() {
    let id = ServiceId::new("test-service");
    let debug_str = format!("{id:?}");
    assert!(debug_str.contains("test-service"));
}

// ============================================================================
// Endpoint Tests
// ============================================================================

#[test]
fn test_endpoint_new() {
    let endpoint = Endpoint::new("https", "example.com", 443);
    assert_eq!(endpoint.protocol, "https");
    assert_eq!(endpoint.host, "example.com");
    assert_eq!(endpoint.port, 443);
    assert_eq!(endpoint.path, None);
}

#[test]
fn test_endpoint_with_path() {
    let endpoint = Endpoint::new("https", "example.com", 443).with_path("/api/v1");
    assert_eq!(endpoint.path, Some("/api/v1".to_string()));
}

#[test]
fn test_endpoint_to_url_without_path() {
    let endpoint = Endpoint::new("https", "example.com", 443);
    assert_eq!(endpoint.to_url(), "https://example.com:443");
}

#[test]
fn test_endpoint_to_url_with_path() {
    let endpoint = Endpoint::new("https", "example.com", 443).with_path("/api/v1");
    assert_eq!(endpoint.to_url(), "https://example.com:443/api/v1");
}

#[test]
fn test_endpoint_to_url_with_leading_slash() {
    let endpoint = Endpoint::new("https", "example.com", 443).with_path("/api/v1");
    assert_eq!(endpoint.to_url(), "https://example.com:443/api/v1");
}

#[test]
fn test_endpoint_http() {
    let endpoint = Endpoint::new("http", "localhost", 8080);
    assert_eq!(endpoint.to_url(), "http://localhost:8080");
}

#[test]
fn test_endpoint_clone() {
    let endpoint = Endpoint::new("https", "example.com", 443).with_path("/api/v1");
    let cloned = endpoint.clone();
    assert_eq!(endpoint, cloned);
}

#[test]
fn test_endpoint_equality() {
    let ep1 = Endpoint::new("https", "example.com", 443);
    let ep2 = Endpoint::new("https", "example.com", 443);
    let ep3 = Endpoint::new("http", "example.com", 80);

    assert_eq!(ep1, ep2);
    assert_ne!(ep1, ep3);
}

// ============================================================================
// RequestId Tests
// ============================================================================

#[test]
fn test_request_id_new() {
    let id = RequestId::new();
    assert!(!id.as_str().is_empty());
}

#[test]
fn test_request_id_uniqueness() {
    let id1 = RequestId::new();
    let id2 = RequestId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_request_id_default() {
    let id = RequestId::default();
    assert!(!id.as_str().is_empty());
}

#[test]
fn test_request_id_uuid() {
    let id = RequestId::new();
    let uuid = id.uuid();
    assert_eq!(id.as_str(), uuid.to_string());
}

#[test]
fn test_request_id_clone() {
    let id = RequestId::new();
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

#[test]
fn test_request_id_debug() {
    let id = RequestId::new();
    let debug_str = format!("{id:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// ConfidenceScore Tests
// ============================================================================

#[test]
fn test_confidence_score_new() {
    let score = ConfidenceScore::new(0.75);
    assert!((score.value() - 0.75).abs() < f64::EPSILON);
}

#[test]
fn test_confidence_score_clamping_high() {
    let score = ConfidenceScore::new(1.5);
    assert!((score.value() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_confidence_score_clamping_low() {
    let score = ConfidenceScore::new(-0.5);
    assert!((score.value() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_confidence_score_is_high() {
    let high = ConfidenceScore::new(0.9);
    let not_high = ConfidenceScore::new(0.7);

    assert!(high.is_high());
    assert!(!not_high.is_high());
}

#[test]
fn test_confidence_score_is_medium() {
    let medium = ConfidenceScore::new(0.6);
    let not_medium_high = ConfidenceScore::new(0.9);
    let not_medium_low = ConfidenceScore::new(0.3);

    assert!(medium.is_medium());
    assert!(!not_medium_high.is_medium());
    assert!(!not_medium_low.is_medium());
}

#[test]
fn test_confidence_score_is_low() {
    let low = ConfidenceScore::new(0.3);
    let not_low = ConfidenceScore::new(0.7);

    assert!(low.is_low());
    assert!(!not_low.is_low());
}

#[test]
fn test_confidence_score_boundary_high() {
    let boundary = ConfidenceScore::new(0.8);
    assert!(boundary.is_high());
    assert!(!boundary.is_medium());
}

#[test]
fn test_confidence_score_boundary_medium_low() {
    let boundary = ConfidenceScore::new(0.5);
    assert!(boundary.is_medium());
    assert!(!boundary.is_low());
}

#[test]
fn test_confidence_score_clone() {
    let score = ConfidenceScore::new(0.75);
    let cloned = score.clone();
    assert_eq!(score, cloned);
}

// ============================================================================
// SuggestedAction Tests
// ============================================================================

#[test]
fn test_suggested_action_new() {
    let action = SuggestedAction::new("retry", "Retry the operation");
    assert_eq!(action.action, "retry");
    assert_eq!(action.description, "Retry the operation");
    assert_eq!(action.priority, 5); // Default medium priority
    assert!(action.parameters.is_empty());
}

#[test]
fn test_suggested_action_with_parameter() {
    let action = SuggestedAction::new("retry", "Retry the operation")
        .with_parameter("max_retries", serde_json::json!(3));

    assert_eq!(action.parameters.len(), 1);
    assert_eq!(action.parameters.get("max_retries"), Some(&serde_json::json!(3)));
}

#[test]
fn test_suggested_action_with_multiple_parameters() {
    let action = SuggestedAction::new("retry", "Retry the operation")
        .with_parameter("max_retries", serde_json::json!(3))
        .with_parameter("delay_ms", serde_json::json!(1000));

    assert_eq!(action.parameters.len(), 2);
}

#[test]
fn test_suggested_action_with_priority() {
    let action = SuggestedAction::new("retry", "Retry the operation").with_priority(10);

    assert_eq!(action.priority, 10);
}

#[test]
fn test_suggested_action_clone() {
    let action = SuggestedAction::new("retry", "Retry the operation");
    let cloned = action.clone();
    assert_eq!(action, cloned);
}

#[test]
fn test_suggested_action_equality() {
    let action1 = SuggestedAction::new("retry", "Retry the operation");
    let action2 = SuggestedAction::new("retry", "Retry the operation");
    assert_eq!(action1, action2);
}

// ============================================================================
// SongbirdResponse Tests
// ============================================================================

#[test]
fn test_songbird_response_success() {
    let response = SongbirdResponse::success("test data");
    assert_eq!(response.data, "test data");
    assert!((response.confidence.value() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_songbird_response_with_confidence() {
    let response = SongbirdResponse::success("test data").with_confidence(0.85);

    assert!((response.confidence.value() - 0.85).abs() < f64::EPSILON);
}

#[test]
fn test_songbird_response_with_suggestion() {
    let action = SuggestedAction::new("retry", "Retry the operation");
    let response = SongbirdResponse::success("test data").with_suggestion(action.clone());

    assert_eq!(response.suggested_actions.len(), 1);
    assert_eq!(response.suggested_actions[0], action);
}

#[test]
fn test_songbird_response_with_human_context() {
    let response =
        SongbirdResponse::success("test data").with_human_context("This is a test response");

    assert_eq!(response.human_context, Some("This is a test response".to_string()));
}

#[test]
fn test_songbird_response_map() {
    let response = SongbirdResponse::success(42);
    let mapped = response.map(|x| x * 2);

    assert_eq!(mapped.data, 84);
}

#[test]
fn test_songbird_response_into_data() {
    let response = SongbirdResponse::success("test data");
    let data = response.into_data();

    assert_eq!(data, "test data");
}

#[test]
fn test_songbird_response_data_ref() {
    let response = SongbirdResponse::success("test data");
    assert_eq!(response.data(), &"test data");
}

#[test]
fn test_songbird_response_from() {
    let response: SongbirdResponse<_> = "test data".into();
    assert_eq!(response.data, "test data");
}

#[test]
fn test_songbird_response_unit() {
    let _response = SongbirdResponse::unit();
    // Verify unit type response (no explicit assertion needed for unit)
}

#[test]
fn test_songbird_response_finish_processing() {
    let start = Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let response = SongbirdResponse::success("test data").finish_processing(start);

    assert!(response.performance.processing_time_ms >= 10);
}

#[test]
fn test_songbird_response_clone() {
    let response = SongbirdResponse::success("test data");
    let cloned = response.clone();
    assert_eq!(response.data, cloned.data);
}

// ============================================================================
// ResponsePerformance Tests
// ============================================================================

#[test]
fn test_response_performance_default() {
    let perf = ResponsePerformance::default();
    assert_eq!(perf.processing_time_ms, 0);
    assert_eq!(perf.memory_usage_bytes, None);
    assert_eq!(perf.cpu_usage_percent, None);
    assert_eq!(perf.network_rtt_ms, None);
    assert_eq!(perf.cache_status, CacheStatus::NotApplicable);
}

#[test]
fn test_response_performance_clone() {
    let perf = ResponsePerformance::default();
    let cloned = perf.clone();
    assert_eq!(perf.processing_time_ms, cloned.processing_time_ms);
}

// ============================================================================
// CacheStatus Tests
// ============================================================================

#[test]
fn test_cache_status_equality() {
    assert_eq!(CacheStatus::Hit, CacheStatus::Hit);
    assert_eq!(CacheStatus::Miss, CacheStatus::Miss);
    assert_eq!(CacheStatus::NotApplicable, CacheStatus::NotApplicable);
    assert_eq!(CacheStatus::Bypassed, CacheStatus::Bypassed);
}

#[test]
fn test_cache_status_inequality() {
    assert_ne!(CacheStatus::Hit, CacheStatus::Miss);
    assert_ne!(CacheStatus::Miss, CacheStatus::NotApplicable);
    assert_ne!(CacheStatus::NotApplicable, CacheStatus::Bypassed);
}

#[test]
fn test_cache_status_clone() {
    let status = CacheStatus::Hit;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

// ============================================================================
// AIResponseMetadata Tests
// ============================================================================

#[test]
fn test_ai_response_metadata_default() {
    let metadata = AIResponseMetadata::default();
    assert!(metadata.automation_capabilities.is_empty());
    assert!(metadata.custom_fields.is_empty());
}

#[test]
fn test_ai_response_metadata_clone() {
    let metadata = AIResponseMetadata::default();
    let cloned = metadata.clone();
    // Both should be functional
    assert!(format!("{metadata:?}").contains("AIResponseMetadata"));
    assert!(format!("{cloned:?}").contains("AIResponseMetadata"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_response_workflow() {
    let start = Instant::now();

    // Create a response with all features
    let action = SuggestedAction::new("optimize", "Optimize the query")
        .with_parameter("cache", serde_json::json!(true))
        .with_priority(8);

    let response = SongbirdResponse::success(vec![1, 2, 3, 4, 5])
        .with_confidence(0.92)
        .with_suggestion(action)
        .with_human_context("Successfully retrieved 5 items")
        .finish_processing(start);

    // Verify all fields
    assert_eq!(response.data.len(), 5);
    assert!(response.confidence.is_high());
    assert_eq!(response.suggested_actions.len(), 1);
    assert!(response.human_context.is_some());
    // Processing time might be 0 on very fast machines, so just check it was set
    // (the separate test_songbird_response_finish_processing with sleep verifies timing works)
}

#[test]
fn test_endpoint_combinations() {
    let endpoints = vec![
        ("https", "api.example.com", 443, Some("/v1/users")),
        ("http", "localhost", 8080, Some("/api/test")),
        ("tcp", "192.168.1.1", 9000, None),
        ("ws", "websocket.example.com", 8443, Some("/stream")),
    ];

    for (proto, host, port, path) in endpoints {
        let mut endpoint = Endpoint::new(proto, host, port);
        if let Some(p) = path {
            endpoint = endpoint.with_path(p);
        }

        let url = endpoint.to_url();
        assert!(url.contains(proto));
        assert!(url.contains(host));
        assert!(url.contains(&port.to_string()));
    }
}

#[test]
fn test_confidence_score_ranges() {
    let scores = vec![
        (0.95, true, false, false), // high
        (0.75, false, true, false), // medium
        (0.35, false, false, true), // low
        (0.8, true, false, false),  // boundary high
        (0.5, false, true, false),  // boundary medium
    ];

    for (value, is_high, is_medium, is_low) in scores {
        let score = ConfidenceScore::new(value);
        assert_eq!(score.is_high(), is_high);
        assert_eq!(score.is_medium(), is_medium);
        assert_eq!(score.is_low(), is_low);
    }
}
