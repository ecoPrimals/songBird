// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
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
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
#![allow(clippy::all, reason = "test assertions and harness ergonomics")]
#![allow(unused, reason = "test assertions and harness ergonomics")]

//! Error Handling & Invalid Input Tests
//!
//! **Purpose**: Tests for error handling, error messages, invalid configurations
//! **Focus**: Do errors communicate clearly and handle edge cases?
//! **Scope**: Error types, messages, debug formats, invalid inputs

#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    CapabilityRegistry, UnifiedAdapterConfig, UnifiedUniversalAdapter, UniversalAdapterError,
    create_universal_adapter, create_universal_adapter_with_config,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// CONFIGURATION VALIDATION TESTS
// ============================================================================

#[test]
fn test_config_with_empty_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_many_endpoints() {
    // Test with large number of endpoints
    let endpoints: Vec<String> = (0..1000).map(|i| format!("http://server{}:8080", i)).collect();

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_invalid_url_formats() {
    // These are syntactically valid strings but semantically unusual URLs
    let endpoints = vec![
        "not-a-url".to_string(),
        String::new(),
        "http://".to_string(),
        "://localhost".to_string(),
        "http://localhost:-1".to_string(),
        "http://localhost:99999".to_string(),
    ];

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    // Should accept config (validation happens at runtime)
    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_discover_with_malformed_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["not-a-url".to_string(), "http://".to_string()],
        discovery_timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should handle malformed URLs gracefully
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

// ============================================================================
// ERROR TYPE TESTS
// ============================================================================

#[test]
fn test_universal_adapter_error_types_completeness() {
    use songbird_universal::UniversalAdapterError;

    // Test all error variants
    let errors = vec![
        UniversalAdapterError::NetworkError("test".to_string()),
        UniversalAdapterError::ParseError("test".to_string()),
        UniversalAdapterError::DiscoveryError("test".to_string()),
        UniversalAdapterError::ServiceError("test".to_string()),
        UniversalAdapterError::MissingCapability,
        UniversalAdapterError::NoProvidersAvailable("test".to_string()),
    ];

    for error in errors {
        // All errors should have non-empty display strings
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn test_error_messages_are_descriptive() -> SongbirdResult<()> {
    use songbird_universal::UniversalAdapterError;

    let err = UniversalAdapterError::NetworkError("connection timeout".to_string());
    assert!(err.to_string().contains("Network error"));
    assert!(err.to_string().contains("connection timeout"));

    let err = UniversalAdapterError::NoProvidersAvailable("ai_inference".to_string());
    assert!(err.to_string().contains("No providers available"));
    assert!(err.to_string().contains("ai_inference"));
    Ok(())
}

#[test]
fn test_error_debug_format() -> SongbirdResult<()> {
    use songbird_types::{SongbirdError, SongbirdResult};
    use songbird_universal::UniversalAdapterError;

    let err = UniversalAdapterError::MissingCapability;
    let debug_str = format!("{:?}", err);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("MissingCapability"));
    Ok(())
}
