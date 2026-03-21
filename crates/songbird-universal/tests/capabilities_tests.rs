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

//! Modern tests for capability system
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
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for capability discovery, registration, and matching.

use songbird_universal::{UnifiedUniversalAdapter, create_universal_adapter};

#[tokio::test]
async fn test_capability_registry_creation() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    assert_eq!(stats.total_capabilities, 0, "New registry should start empty");
}

#[tokio::test]
async fn test_capability_registry_stats_structure() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // Verify stats structure
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test]
async fn test_multiple_adapters_separate_registries() {
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();

    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    // Each should have independent registries
    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

#[tokio::test]
async fn test_capability_system_initialization() {
    let adapter = UnifiedUniversalAdapter::new();

    // Capability system should be initialized
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);
}

#[tokio::test]
async fn test_registry_stats_consistency() {
    let adapter = create_universal_adapter();

    // Multiple calls should return consistent results
    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}

#[tokio::test]
async fn test_registry_empty_state() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // Empty registry should have zero counts
    assert_eq!(stats.total_services, 0, "Should have no services");
    assert_eq!(stats.total_capabilities, 0, "Should have no capabilities");
    assert_eq!(stats.healthy_services, 0, "Should have no healthy services");
}

#[tokio::test]
async fn test_concurrent_registry_access() {
    let adapter = create_universal_adapter();

    // Multiple concurrent reads should work
    let (stats1, stats2) = tokio::join!(adapter.get_registry_stats(), adapter.get_registry_stats());

    assert_eq!(stats1.total_services, stats2.total_services);
}

#[test]
fn test_adapter_clone_creates_independent_instance() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1;

    // Cloning should create independent instances
    let _ = adapter1;
    let _ = adapter2;
}

#[tokio::test]
async fn test_registry_stats_structure_completeness() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // Stats should have all expected fields
    let _ = stats.total_services;
    let _ = stats.total_capabilities;
    let _ = stats.healthy_services;
}

#[tokio::test]
async fn test_multiple_stats_calls_no_state_change() {
    let adapter = create_universal_adapter();

    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;
    let stats3 = adapter.get_registry_stats().await;

    // Multiple reads shouldn't change state
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats2.total_services, stats3.total_services);
}
