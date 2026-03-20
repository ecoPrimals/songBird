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
    clippy::cast_possible_wrap
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Async Integration Tests for Unified Adapter
//!
//! **Goal**: Test unified adapter under realistic async scenarios
//! **Coverage Target**: Concurrent access, initialization, state management
//!
//! This suite tests:
//! - Adapter initialization
//! - Capability provider lookup
//! - Concurrent access patterns
//! - Registry statistics
//! - Error handling

use songbird_universal::unified_adapter::UnifiedUniversalAdapter;
use std::sync::Arc;

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_create_new_adapter() {
    let adapter = UnifiedUniversalAdapter::new();

    // Should be able to query for capabilities (even if empty)
    let result = adapter.find_capability_providers("compute").await;
    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition").len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_adapter_initial_state() {
    let adapter = UnifiedUniversalAdapter::new();

    // Initial registry should be empty
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_adapter_instances() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = UnifiedUniversalAdapter::new();

    // Each should be independent
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

// ============================================================================
// CAPABILITY LOOKUP TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_find_nonexistent_capability() {
    let adapter = UnifiedUniversalAdapter::new();

    // Should return empty, not error
    let result = adapter.find_capability_providers("nonexistent").await;
    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition").len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_find_multiple_capabilities() {
    let adapter = UnifiedUniversalAdapter::new();

    // Multiple lookups should all succeed
    let compute = adapter.find_capability_providers("compute").await;
    let storage = adapter.find_capability_providers("storage").await;
    let ai = adapter.find_capability_providers("ai").await;

    assert!(compute.is_ok());
    assert!(storage.is_ok());
    assert!(ai.is_ok());
}

// ============================================================================
// CONCURRENT ACCESS TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_capability_lookups() {
    let adapter = Arc::new(UnifiedUniversalAdapter::new());

    // Concurrent lookups on empty registry
    let mut handles = vec![];
    for _ in 0..20 {
        let adapter_clone = Arc::clone(&adapter);
        handles.push(tokio::spawn(async move {
            adapter_clone.find_capability_providers("compute").await
        }));
    }

    let results = futures::future::join_all(handles).await;

    // All should succeed (returning empty lists)
    for result in results {
        assert!(result.is_ok());
        let providers = result.expect("test precondition");
        assert!(providers.is_ok());
        assert_eq!(providers.expect("test precondition").len(), 0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_stats_access() {
    let adapter = Arc::new(UnifiedUniversalAdapter::new());

    // Concurrent stats access
    let mut handles = vec![];
    for _ in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        handles.push(tokio::spawn(async move { adapter_clone.get_registry_stats().await }));
    }

    let results = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        let stats = result.expect("test precondition");
        assert_eq!(stats.total_services, 0);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_mixed_operations() {
    let adapter = Arc::new(UnifiedUniversalAdapter::new());

    // Mix of stats and lookups
    let mut handles = vec![];

    for i in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        if i % 2 == 0 {
            handles.push(tokio::spawn(async move {
                let _ = adapter_clone.get_registry_stats().await;
            }));
        } else {
            handles.push(tokio::spawn(async move {
                let _ = adapter_clone.find_capability_providers("compute").await;
            }));
        }
    }

    // Should all complete without deadlock
    futures::future::join_all(handles).await;
}

// ============================================================================
// STATISTICS TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_registry_stats_structure() {
    let adapter = UnifiedUniversalAdapter::new();

    let stats = adapter.get_registry_stats().await;

    // Check all fields exist and have expected initial values
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_stats_consistency_across_calls() {
    let adapter = UnifiedUniversalAdapter::new();

    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;

    // Should be consistent
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
    assert_eq!(stats1.healthy_services, stats2.healthy_services);
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lookup_with_empty_string() {
    let adapter = UnifiedUniversalAdapter::new();

    let result = adapter.find_capability_providers("").await;
    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition").len(), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lookup_with_special_characters() {
    let adapter = UnifiedUniversalAdapter::new();

    let result = adapter.find_capability_providers("comp-ute/storage@v1").await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_lookup_with_long_string() {
    let adapter = UnifiedUniversalAdapter::new();

    let long_capability = "a".repeat(1000);
    let result = adapter.find_capability_providers(&long_capability).await;
    assert!(result.is_ok());
}

// ============================================================================
// THREAD SAFETY TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_clone_and_concurrent_use() {
    let adapter = UnifiedUniversalAdapter::new();
    let cloned = adapter.clone();

    // Use both concurrently
    let (stats1, stats2) = tokio::join!(adapter.get_registry_stats(), cloned.get_registry_stats());

    assert_eq!(stats1.total_services, stats2.total_services);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_shared_state_across_clones() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    // Both should see the same empty registry
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
}
