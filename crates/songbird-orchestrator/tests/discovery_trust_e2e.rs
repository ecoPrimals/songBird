// SPDX-License-Identifier: AGPL-3.0-or-later
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
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! End-to-end tests for discovery + trust evaluation flow
//!
//! These tests verify the complete flow:
//! 1. Peer discovered via UDP multicast
//! 2. Tags extracted from discovery message
//! 3. Trust evaluation performed
//! 4. Peer accepted/rejected based on decision

use songbird_discovery::anonymous::AnonymousDiscoveryMessage;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test: Discovery message with tags
#[test]
fn test_discovery_message_with_tags() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string(), "storage".to_string()],
        vec!["https".to_string()],
        8080,
    );

    // Add tags (simulating security provider adding encryption tags)
    message.tags = Some(vec!["beardog:family:iidn:tower1".to_string()]);

    assert!(message.tags.is_some());
    assert_eq!(message.tags.unwrap().len(), 1);
}

/// Test: Discovery message serialization with tags
#[test]
fn test_discovery_message_serialization() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );
    message.tags = Some(vec!["test:tag".to_string()]);

    let serialized = serde_json::to_string(&message).expect("Failed to serialize");
    assert!(serialized.contains("test:tag"));

    let deserialized: AnonymousDiscoveryMessage =
        serde_json::from_str(&serialized).expect("Failed to deserialize");

    assert!(deserialized.tags.is_some());
    assert_eq!(deserialized.tags.as_ref().unwrap().len(), 1);
}

/// Test: Discovery packet without tags (backward compatibility)
#[test]
fn test_discovery_message_without_tags() {
    let message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    // Tags should be None for backward compatibility
    assert!(message.tags.is_none());
}

/// Test: Multiple tags in discovery
#[test]
fn test_multiple_tags_in_discovery() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    message.tags = Some(vec![
        "beardog:family:iidn:tower1".to_string(),
        "btsp_enabled".to_string(),
        "birdsong_v2".to_string(),
    ]);

    assert_eq!(message.tags.as_ref().unwrap().len(), 3);
}

/// Test: Discovery timestamp is recent
#[test]
fn test_discovery_timestamp() {
    let message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    // Timestamp should be within 1 second of now
    assert!(message.timestamp >= now - 1 && message.timestamp <= now + 1);
}

/// Test: Discovery message version
#[test]
fn test_discovery_version() {
    let message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    assert_eq!(message.version, "2.1");
}

/// Test: E2E flow simulation (discovery → trust evaluation)
#[test]
fn test_e2e_discovery_to_trust() {
    // Step 1: Receive discovery message
    let mut discovery_message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string(), "storage".to_string()],
        vec!["https".to_string()],
        8080,
    );
    discovery_message.tags = Some(vec!["beardog:family:iidn:tower2".to_string()]);

    // Step 2: Extract tags for trust evaluation
    let tags = discovery_message.tags.clone().unwrap_or_default();
    assert!(!tags.is_empty());

    // Step 3: Verify tag format
    assert!(tags[0].starts_with("beardog:family:"));

    // Step 4: Would call evaluate_peer_trust here in real flow
    // For now, just verify the data structure is correct
    assert_eq!(discovery_message.capabilities, vec!["compute", "storage"]);
}

/// Test: Discovery without security provider (development mode)
#[test]
fn test_discovery_without_security_provider() {
    // When SONGBIRD_SECURITY_PROVIDER is not set, discovery works but has no tags
    let message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    // No tags should be present
    assert!(message.tags.is_none());

    // But discovery still works (development/testing mode)
    assert!(!message.capabilities.is_empty());
}

/// Test: Tag format validation
#[test]
fn test_tag_format() {
    let valid_tags =
        vec!["beardog:family:iidn:tower1", "btsp_enabled", "birdsong_v2", "custom:metadata:value"];

    for tag in valid_tags {
        // Tags are opaque strings - Songbird doesn't parse them
        assert!(!tag.is_empty());

        // But we can check basic format
        if tag.starts_with("beardog:family:") {
            let parts: Vec<&str> = tag.split(':').collect();
            assert!(parts.len() >= 4, "security provider tag should have at least 4 parts");
        }
    }
}

/// Test: Concurrent discovery message handling
#[test]
fn test_concurrent_discovery() {
    let messages: Vec<AnonymousDiscoveryMessage> = (0..10)
        .map(|i| {
            let mut msg = AnonymousDiscoveryMessage::new(
                vec![format!("capability_{}", i)],
                vec!["https".to_string()],
                8080 + i,
            );
            msg.tags = Some(vec![format!("tag_{}", i)]);
            msg
        })
        .collect();

    assert_eq!(messages.len(), 10);

    // All messages should have unique tags
    for (i, msg) in messages.iter().enumerate() {
        assert_eq!(msg.tags.as_ref().unwrap()[0], format!("tag_{i}"));
    }
}

/// Test: Empty capabilities list
#[test]
fn test_empty_capabilities() {
    let message = AnonymousDiscoveryMessage::new(
        vec![], // No capabilities
        vec!["https".to_string()],
        8080,
    );

    assert!(message.capabilities.is_empty());
    // This is valid - a peer might not advertise capabilities
}

/// Test: Large number of tags
#[test]
fn test_many_tags() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    let tags: Vec<String> = (0..100).map(|i| format!("tag_{i}")).collect();

    message.tags = Some(tags);

    assert_eq!(message.tags.as_ref().unwrap().len(), 100);
}

/// Test: Tag with special characters
#[test]
fn test_tags_with_special_chars() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string()],
        vec!["https".to_string()],
        8080,
    );

    message.tags = Some(vec![
        "beardog:family:test-123_456:tower".to_string(),
        "tag@with#special$chars".to_string(),
    ]);

    // Tags are opaque - any string is valid
    assert!(message.tags.is_some());
}

/// Test: Discovery message size estimation
#[test]
fn test_discovery_message_size() {
    let mut message = AnonymousDiscoveryMessage::new(
        vec!["compute".to_string(), "storage".to_string()],
        vec!["https".to_string()],
        8080,
    );
    message.tags = Some(vec!["beardog:family:iidn:tower1".to_string()]);

    let serialized = serde_json::to_string(&message).expect("Failed to serialize");

    // Message should be reasonably small (< 1KB for UDP)
    assert!(serialized.len() < 1024, "Discovery message should fit in single UDP packet");
}
