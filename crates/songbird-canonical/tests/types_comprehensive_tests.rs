//! Comprehensive Type Tests for Canonical Crate
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

//!
//! Tests for all core canonical types.

use serde_json::json;
use songbird_canonical::types::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};

// ========== ServiceId Tests ==========

#[test]
fn test_service_id_basic_creation() {
    let id = ServiceId::new("test-service");
    assert_eq!(id.as_str(), "test-service");
}

#[test]
fn test_service_id_from_string() {
    let id: ServiceId = "my-service".into();
    assert_eq!(id.as_str(), "my-service");

    let id2: ServiceId = String::from("owned-service").into();
    assert_eq!(id2.as_str(), "owned-service");
}

#[test]
fn test_service_id_equality() -> SongbirdResult<()> {
    let id1 = ServiceId::new("service-a");
    let id2 = ServiceId::new("service-a");
    let id3 = ServiceId::new("service-b");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
    Ok(())
}

#[test]
fn test_service_id_clone() -> SongbirdResult<()> {
    let id = ServiceId::new("cloneable");
    let cloned = id.clone();
    assert_eq!(id, cloned);
    Ok(())
}

#[test]
fn test_service_id_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let id = ServiceId::new("serialize-me");
    let json = serde_json::to_string(&id)
        .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("serialize-me"));
    Ok(())
}

#[test]
fn test_service_id_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""deserialized-service""#;
    let id: ServiceId = serde_json::from_str(json)
        .map_err(|e| SongbirdError::configuration("Should deserialize".to_string()))?;
    assert_eq!(id.as_str(), "deserialized-service");
    Ok(())
}

// ========== Endpoint Tests ==========

#[test]
fn test_endpoint_basic_creation() {
    let endpoint = Endpoint::new("http", "localhost", 8080);
    assert_eq!(endpoint.protocol, "http");
    assert_eq!(endpoint.host, "localhost");
    assert_eq!(endpoint.port, 8080);
    assert!(endpoint.path.is_none());
}

#[test]
fn test_endpoint_with_path() {
    let endpoint = Endpoint::new("https", "api.example.com", 443).with_path("/v1/users");
    assert_eq!(endpoint.path, Some("/v1/users".to_string()));
}

#[test]
fn test_endpoint_to_url() {
    let endpoint = Endpoint::new("http", "localhost", 3000);
    assert_eq!(endpoint.to_url(), "http://localhost:3000");
}

#[test]
fn test_endpoint_to_url_with_path() {
    let endpoint = Endpoint::new("https", "api.com", 443).with_path("/api/v2/data");
    assert_eq!(endpoint.to_url(), "https://api.com:443/api/v2/data");
}

#[test]
fn test_endpoint_to_url_with_leading_slash() -> SongbirdResult<()> {
    let endpoint = Endpoint::new("http", "example.com", 80).with_path("///triple/slash");
    let url = endpoint.to_url();
    assert!(!url.contains("////")); // Should normalize slashes
    Ok(())
}

#[test]
fn test_endpoint_equality() -> SongbirdResult<()> {
    let ep1 = Endpoint::new("tcp", "192.168.1.1", 9000);
    let ep2 = Endpoint::new("tcp", "192.168.1.1", 9000);
    let ep3 = Endpoint::new("udp", "192.168.1.1", 9000);

    assert_eq!(ep1, ep2);
    assert_ne!(ep1, ep3);
    Ok(())
}

#[test]
fn test_endpoint_clone() -> SongbirdResult<()> {
    let endpoint = Endpoint::new("grpc", "service.local", 50051);
    let cloned = endpoint.clone();
    assert_eq!(endpoint, cloned);
    Ok(())
}

#[test]
fn test_endpoint_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::new("wss", "ws.example.com", 8443);
    let json = serde_json::to_string(&endpoint)
        .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("wss"));
    assert!(json.contains("ws.example.com"));
    assert!(json.contains("8443"));
    Ok(())
}

#[test]
fn test_endpoint_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"protocol":"https","host":"test.com","port":443,"path":null}"#;
    let endpoint: Endpoint = serde_json::from_str(json)
        .map_err(|e| SongbirdError::configuration("Should deserialize".to_string()))?;
    assert_eq!(endpoint.protocol, "https");
    assert_eq!(endpoint.host, "test.com");
    assert_eq!(endpoint.port, 443);
    Ok(())
}

// ========== RequestId Tests ==========

#[test]
fn test_request_id_creation() {
    let id = RequestId::new();
    assert!(!id.as_str().is_empty());
}

#[test]
fn test_request_id_unique() {
    let id1 = RequestId::new();
    let id2 = RequestId::new();
    assert_ne!(id1, id2); // UUIDs should be unique
}

#[test]
fn test_request_id_uuid_format() -> SongbirdResult<()> {
    let id = RequestId::new();
    let uuid_str = id.as_str();
    assert_eq!(uuid_str.len(), 36); // UUID v4 format length
    assert_eq!(uuid_str.chars().filter(|c| *c == '-').count(), 4); // UUID has 4 dashes
    Ok(())
}

#[test]
fn test_request_id_clone() -> SongbirdResult<()> {
    let id = RequestId::new();
    let cloned = id.clone();
    assert_eq!(id, cloned);
    Ok(())
}

#[test]
fn test_request_id_default() -> SongbirdResult<()> {
    let id = RequestId::default();
    assert!(!id.as_str().is_empty());
    Ok(())
}

#[test]
fn test_request_id_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let id = RequestId::new();
    let json = serde_json::to_string(&id)
        .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(!json.is_empty());
    Ok(())
}

// ========== ConfidenceScore Tests ==========

#[test]
fn test_confidence_score_basic() {
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
    assert!(score.value().abs() < f64::EPSILON); // Should be 0.0
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
    let medium = ConfidenceScore::new(0.65);
    let not_medium_high = ConfidenceScore::new(0.9);
    let not_medium_low = ConfidenceScore::new(0.3);

    assert!(medium.is_medium());
    assert!(!not_medium_high.is_medium());
    assert!(!not_medium_low.is_medium());
}

#[test]
fn test_confidence_score_is_low() -> SongbirdResult<()> {
    let low = ConfidenceScore::new(0.3);
    let not_low = ConfidenceScore::new(0.6);

    assert!(low.is_low());
    assert!(!not_low.is_low());
    Ok(())
}

#[test]
fn test_confidence_score_boundaries() -> SongbirdResult<()> {
    // Test exact boundary conditions
    let exactly_high = ConfidenceScore::new(0.8);
    assert!(exactly_high.is_high());
    assert!(!exactly_high.is_medium());

    let exactly_medium_low = ConfidenceScore::new(0.5);
    assert!(exactly_medium_low.is_medium());
    assert!(!exactly_medium_low.is_low());
    Ok(())
}

#[test]
fn test_confidence_score_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let score = ConfidenceScore::new(0.85);
    let json = serde_json::to_string(&score)
        .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("0.85"));
    Ok(())
}

// ========== SuggestedAction Tests ==========

#[test]
fn test_suggested_action_basic() {
    let action = SuggestedAction::new("restart", "Restart the service");
    assert_eq!(action.action, "restart");
    assert_eq!(action.description, "Restart the service");
    assert_eq!(action.priority, 5); // Default medium priority
}

#[test]
fn test_suggested_action_with_parameter() {
    let action = SuggestedAction::new("scale", "Scale up").with_parameter("replicas", json!(3));

    assert_eq!(action.parameters.len(), 1);
    assert_eq!(action.parameters.get("replicas"), Some(&json!(3)));
}

#[test]
fn test_suggested_action_with_priority() {
    let action = SuggestedAction::new("urgent_action", "Do something urgent").with_priority(10);

    assert_eq!(action.priority, 10);
}

#[test]
fn test_suggested_action_builder_pattern() {
    let action = SuggestedAction::new("complex", "Complex action")
        .with_parameter("param1", json!("value1"))
        .with_parameter("param2", json!(42))
        .with_priority(8);

    assert_eq!(action.parameters.len(), 2);
    assert_eq!(action.priority, 8);
}

#[test]
fn test_suggested_action_equality() -> SongbirdResult<()> {
    let action1 = SuggestedAction::new("action-a", "Description A");
    let action2 = SuggestedAction::new("action-a", "Description A");
    let action3 = SuggestedAction::new("action-b", "Description B");

    assert_eq!(action1, action2);
    assert_ne!(action1, action3);
    Ok(())
}

#[test]
fn test_suggested_action_clone() -> SongbirdResult<()> {
    let action =
        SuggestedAction::new("clone-test", "Test cloning").with_parameter("key", json!("value"));
    let cloned = action.clone();

    assert_eq!(action, cloned);
    Ok(())
}

#[test]
fn test_suggested_action_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let action =
        SuggestedAction::new("test", "Test action").with_parameter("param", json!("value"));

    let json = serde_json::to_string(&action)
        .map_err(|e| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("test"));
    assert!(json.contains("Test action"));
    Ok(())
}

// ========== Thread Safety Tests ==========

#[test]
fn test_all_types_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ServiceId>();
    assert_sync::<ServiceId>();

    assert_send::<Endpoint>();
    assert_sync::<Endpoint>();

    assert_send::<RequestId>();
    assert_sync::<RequestId>();

    assert_send::<ConfidenceScore>();
    assert_sync::<ConfidenceScore>();

    assert_send::<SuggestedAction>();
    assert_sync::<SuggestedAction>();
}
