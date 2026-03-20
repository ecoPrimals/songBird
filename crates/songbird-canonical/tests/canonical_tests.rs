// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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
    reason = "test assertions and harness ergonomics"
)]

//! Canonical Types Tests
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![expect(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![expect(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![expect(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Testing canonical type definitions and conversions.

use songbird_canonical::types::{Endpoint, RequestId, ServiceId};
use songbird_test_utils::test_bind_address;
use songbird_test_utils::test_dashboard_port;
use songbird_test_utils::test_orchestrator_port;
use songbird_types::SongbirdError;

#[test]
fn test_canonical_type_creation() {
    // Test: Canonical types should be creatable with various constructors
    let service_id = ServiceId::new("test-service");
    assert_eq!(service_id.as_str(), "test-service");

    let endpoint = Endpoint::new("https", "example.com", 443);
    assert_eq!(endpoint.protocol, "https");
    assert_eq!(endpoint.host, "example.com");
    assert_eq!(endpoint.port, 443);

    let request_id = RequestId::new();
    assert!(!request_id.uuid().to_string().is_empty());
}

#[test]
fn test_canonical_validation() {
    // Test: Canonical types should validate their data
    let service_id = ServiceId::new("valid-service-123");
    assert_eq!(service_id.as_str(), "valid-service-123");

    let bind_addr = test_bind_address("test");
    let endpoint = Endpoint::new("http", bind_addr.as_str(), test_orchestrator_port());
    let url = endpoint.to_url();
    assert_eq!(url, format!("http://{}:{}", bind_addr, test_orchestrator_port()));

    // Endpoint with path
    let endpoint_with_path = Endpoint::new("https", "api.example.com", 443).with_path("/v1/users");
    assert_eq!(endpoint_with_path.to_url(), "https://api.example.com:443/v1/users");
}

#[test]
fn test_canonical_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Canonical types should serialize to JSON
    let service_id = ServiceId::new("my-service");
    let json = serde_json::to_string(&service_id)
        .map_err(|_| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("my-service"));

    let endpoint = Endpoint::new("tcp", "192.168.1.1", 9000);
    let json = serde_json::to_string(&endpoint)
        .map_err(|_| SongbirdError::configuration("Should serialize".to_string()))?;
    assert!(json.contains("tcp"));
    assert!(json.contains("192.168.1.1"));
    assert!(json.contains("9000"));
    Ok(())
}

#[test]
fn test_canonical_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    // Test: Canonical types should deserialize from JSON
    let json = r#""test-service""#;
    let service_id: ServiceId = serde_json::from_str(json)
        .map_err(|_| SongbirdError::configuration("Should deserialize".to_string()))?;
    assert_eq!(service_id.as_str(), "test-service");

    let json = r#"{"protocol":"https","host":"example.com","port":443,"path":null}"#;
    let endpoint: Endpoint = serde_json::from_str(json)
        .map_err(|_| SongbirdError::configuration("Should deserialize".to_string()))?;
    assert_eq!(endpoint.protocol, "https");
    assert_eq!(endpoint.host, "example.com");
    assert_eq!(endpoint.port, 443);
    Ok(())
}

#[test]
fn test_canonical_conversion() {
    // Test: Canonical types should convert from various types
    let service_id: ServiceId = "string-service".into();
    assert_eq!(service_id.as_str(), "string-service");

    let service_id: ServiceId = String::from("owned-service").into();
    assert_eq!(service_id.as_str(), "owned-service");

    // Endpoint URL conversion
    let bind_addr = test_bind_address("test");
    let endpoint = Endpoint::new("http", bind_addr.as_str(), test_dashboard_port());
    assert_eq!(endpoint.to_url(), format!("http://{}:{}", bind_addr, test_dashboard_port()));
}

#[test]
fn test_canonical_equality() {
    // Test: Canonical types should compare correctly
    let service_id1 = ServiceId::new("service-a");
    let service_id2 = ServiceId::new("service-a");
    let service_id3 = ServiceId::new("service-b");

    assert_eq!(service_id1, service_id2);
    assert_ne!(service_id1, service_id3);

    let endpoint1 = Endpoint::new("https", "api.com", 443);
    let endpoint2 = Endpoint::new("https", "api.com", 443);
    let endpoint3 = Endpoint::new("http", "api.com", 80);

    assert_eq!(endpoint1, endpoint2);
    assert_ne!(endpoint1, endpoint3);
}

#[test]
fn test_canonical_clone() {
    // Test: Canonical types should clone correctly
    let service_id = ServiceId::new("cloneable-service");
    let cloned = service_id.clone();
    assert_eq!(service_id, cloned);

    let endpoint = Endpoint::new("grpc", "service.local", 50051);
    let cloned_endpoint = endpoint.clone();
    assert_eq!(endpoint, cloned_endpoint);
    assert_eq!(cloned_endpoint.port, 50051);
}

#[test]
fn test_canonical_defaults() {
    // Test: Canonical types should have sensible defaults
    let request_id = RequestId::new();
    assert!(request_id.uuid().to_string().len() == 36); // UUID v4 length

    // Endpoint can be created with minimal parameters
    let endpoint = Endpoint::new("http", test_bind_address("test").as_str(), 8080);
    assert!(endpoint.path.is_none());

    // ServiceId can be created from simple strings
    let service_id = ServiceId::new("default");
    assert!(!service_id.as_str().is_empty());
}

#[test]
fn test_canonical_thread_safety() {
    // Test: Canonical types should be Send + Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ServiceId>();
    assert_sync::<ServiceId>();
    assert_send::<Endpoint>();
    assert_sync::<Endpoint>();
    assert_send::<RequestId>();
    assert_sync::<RequestId>();
}

#[test]
fn test_canonical_documentation() {
    // Test: Canonical types should be well-documented (compile-time check)
    // This test verifies that types have proper Debug implementation
    let service_id = ServiceId::new("documented");
    let debug_str = format!("{service_id:?}");
    assert!(debug_str.contains("ServiceId"));

    let endpoint = Endpoint::new("wss", "ws.example.com", 8443);
    let debug_str = format!("{endpoint:?}");
    assert!(debug_str.contains("Endpoint"));
    assert!(debug_str.contains("wss"));
}
