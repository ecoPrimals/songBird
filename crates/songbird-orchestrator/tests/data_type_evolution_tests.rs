// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Data Type Evolution Tests
//!
//! Tests the evolution from u64 to String for `first_seen_at` field
//! to ensure `BearDog` API compatibility

use songbird_orchestrator::security_capability_client::{
    ConnectionInfo, DiscoveryContext, TrustEvaluationRequest,
};
use std::collections::HashMap;

/// Test: `DiscoveryContext` with String timestamp
#[test]
fn test_discovery_context_string_timestamp() {
    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: "1767368141".to_string(),
        metadata: HashMap::new(),
    };

    assert_eq!(context.first_seen_at, "1767368141");
    assert!(!context.first_seen_at.is_empty());
}

/// Test: Serialization of `DiscoveryContext` with String timestamp
#[test]
fn test_discovery_context_serialization() {
    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: "1767368141".to_string(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&context).expect("Failed to serialize");

    // Should serialize as string, not integer
    assert!(json.contains(r#""first_seen_at":"1767368141""#));
    assert!(!json.contains(r#""first_seen_at":1767368141"#)); // Not as integer
}

/// Test: Deserialization of `DiscoveryContext` from JSON with String
#[test]
fn test_discovery_context_deserialization() {
    let json = r#"{
        "discovery_method": "udp_multicast",
        "first_seen_at": "1767368141",
        "metadata": {}
    }"#;

    let context: DiscoveryContext = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(context.first_seen_at, "1767368141");
    assert_eq!(context.discovery_method, "udp_multicast");
}

/// Test: `TrustEvaluationRequest` with String timestamp in `HashMap` context
#[test]
fn test_trust_evaluation_request_with_string_timestamp() {
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), "udp_multicast".to_string()),
                ("first_seen_at".to_string(), "1767368141".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    assert!(request.context.is_some());
    let context = request.context.unwrap();
    assert_eq!(context.get("first_seen_at"), Some(&"1767368141".to_string()));
    assert_eq!(context.get("discovery_method"), Some(&"udp_multicast".to_string()));
}

/// Test: `TrustEvaluationRequest` serialization (`BearDog` API format)
#[test]
fn test_trust_evaluation_request_beardog_format() {
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), "udp_multicast".to_string()),
                ("first_seen_at".to_string(), "1767368141".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize");

    // Should contain string timestamp, not integer
    assert!(json.contains(r#""first_seen_at":"1767368141""#));

    // Verify it can be deserialized back
    let _deserialized: TrustEvaluationRequest =
        serde_json::from_str(&json).expect("Failed to deserialize");
}

/// Test: u64 to String conversion
#[test]
fn test_u64_to_string_conversion() {
    let timestamp_u64: u64 = 1767368141;
    let timestamp_string = timestamp_u64.to_string();

    assert_eq!(timestamp_string, "1767368141");

    // Can be converted back if needed
    let parsed: u64 = timestamp_string.parse().expect("Failed to parse");
    assert_eq!(parsed, timestamp_u64);
}

/// Test: Different timestamp formats
#[test]
fn test_various_timestamp_formats() {
    let timestamps = vec![
        0u64,
        1704196800u64, // Jan 1, 2024
        1767368141u64, // Test value
        u64::MAX,      // Max value
    ];

    for ts in timestamps {
        let context = DiscoveryContext {
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: ts.to_string(),
            metadata: HashMap::new(),
        };

        // Should serialize without error
        let json = serde_json::to_string(&context).expect("Failed to serialize");
        assert!(json.contains(&format!(r#""first_seen_at":"{ts}""#)));
    }
}

/// Test: Empty context (backward compatibility)
#[test]
fn test_trust_request_without_context() {
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: None,
    };

    assert!(request.context.is_none());

    // Should serialize without error
    let json = serde_json::to_string(&request).expect("Failed to serialize");
    // Context may be omitted or null depending on serde settings
    assert!(json.contains(r#""peer_id":"tower2""#));
}

/// Test: Metadata in `DiscoveryContext`
#[test]
fn test_discovery_context_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("network".to_string(), "lan".to_string());
    metadata.insert("interface".to_string(), "eth0".to_string());

    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: "1767368141".to_string(),
        metadata,
    };

    assert_eq!(context.metadata.len(), 2);
    assert_eq!(context.metadata.get("network").unwrap(), "lan");
}

/// Test: Round-trip serialization/deserialization
#[test]
fn test_round_trip_serialization() {
    let original = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: "1767368141".to_string(),
        metadata: HashMap::new(),
    };

    // Serialize
    let json = serde_json::to_string(&original).expect("Failed to serialize");

    // Deserialize
    let deserialized: DiscoveryContext =
        serde_json::from_str(&json).expect("Failed to deserialize");

    // Compare
    assert_eq!(original.discovery_method, deserialized.discovery_method);
    assert_eq!(original.first_seen_at, deserialized.first_seen_at);
    assert_eq!(original.metadata.len(), deserialized.metadata.len());
}

/// Test: `BearDog` API compatibility (exact format)
#[test]
fn test_beardog_api_exact_format() {
    let request = TrustEvaluationRequest {
        peer_id: "tower2".to_string(),
        peer_family: Some("iidn".to_string()), // ✅ v3.14.1
        peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
        connection_info: Some(ConnectionInfo {
            endpoint: "https://192.168.1.135:8080".to_string(),
            protocol: "tarpc".to_string(),
        }),
        context: Some(
            vec![
                ("discovery_method".to_string(), "udp_multicast".to_string()),
                ("first_seen_at".to_string(), "1767368141".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    };

    let json = serde_json::to_string_pretty(&request).expect("Failed to serialize");

    // Verify format matches BearDog expectations
    assert!(json.contains(r#""first_seen_at": "1767368141""#));
    assert!(json.contains(r#""discovery_method": "udp_multicast""#));

    // This format should be accepted by BearDog API (no 422 errors)
}

/// Test: Current timestamp conversion
#[test]
fn test_current_timestamp_conversion() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: now.to_string(),
        metadata: HashMap::new(),
    };

    // Should be a valid string representation
    assert!(!context.first_seen_at.is_empty());

    // Should be parseable back to u64
    let parsed: u64 = context.first_seen_at.parse().expect("Failed to parse");
    assert_eq!(parsed, now);
}

/// Test: Very large timestamp values
#[test]
fn test_large_timestamp_values() {
    let large_timestamp = u64::MAX - 1000;

    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: large_timestamp.to_string(),
        metadata: HashMap::new(),
    };

    // Should handle large values without overflow
    let json = serde_json::to_string(&context).expect("Failed to serialize");
    assert!(json.contains(&large_timestamp.to_string()));
}

/// Test: Zero timestamp (edge case)
#[test]
fn test_zero_timestamp() {
    let context = DiscoveryContext {
        discovery_method: "udp_multicast".to_string(),
        first_seen_at: "0".to_string(),
        metadata: HashMap::new(),
    };

    assert_eq!(context.first_seen_at, "0");

    let json = serde_json::to_string(&context).expect("Failed to serialize");
    assert!(json.contains(r#""first_seen_at":"0""#));
}

/// Test: Multiple discovery contexts with different timestamps
#[test]
fn test_multiple_contexts() {
    let contexts: Vec<DiscoveryContext> = (0..10)
        .map(|i| DiscoveryContext {
            discovery_method: "udp_multicast".to_string(),
            first_seen_at: (1767368141 + i).to_string(),
            metadata: HashMap::new(),
        })
        .collect();

    assert_eq!(contexts.len(), 10);

    // All should serialize correctly
    for (i, context) in contexts.iter().enumerate() {
        let json = serde_json::to_string(context).expect("Failed to serialize");
        assert!(json.contains(&format!(r#""first_seen_at":"{}""#, 1767368141 + i)));
    }
}
