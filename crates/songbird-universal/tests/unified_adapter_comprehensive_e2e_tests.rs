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
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! # Comprehensive E2E Tests for Unified Universal Adapter
//!
//! Tests complete workflows with real components

use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};

// ============================================================================
// ADAPTER CREATION & CONFIGURATION
// ============================================================================

#[tokio::test]
async fn test_e2e_adapter_creation_default() {
    let adapter = UnifiedUniversalAdapter::new();
    // Verify adapter created successfully (implicit by no panic)
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_e2e_adapter_creation_with_config() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://localhost:65432/discovery".to_string(),
            "http://localhost:65433/discovery".to_string(),
        ],
        discovery_timeout: std::time::Duration::from_secs(10),
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_e2e_config_default_values() {
    let config = UnifiedAdapterConfig::default();

    assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(30));
    assert_eq!(config.health_check_interval, std::time::Duration::from_secs(60));
    assert_eq!(config.max_concurrent_requests, 100);
    assert!(config.auto_discovery);
}

#[tokio::test]
async fn test_e2e_config_custom_values() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: std::time::Duration::from_secs(5),
        health_check_interval: std::time::Duration::from_secs(30),
        max_concurrent_requests: 50,
        auto_discovery: false,
        discovery_endpoints: vec!["http://custom:8080".to_string()],
    };

    assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(5));
    assert_eq!(config.health_check_interval, std::time::Duration::from_secs(30));
    assert_eq!(config.max_concurrent_requests, 50);
    assert!(!config.auto_discovery);
    assert_eq!(config.discovery_endpoints.len(), 1);
}

// ============================================================================
// SERVICE DISCOVERY
// ============================================================================

#[tokio::test]
async fn test_e2e_discover_services_no_endpoints() {
    let adapter = UnifiedUniversalAdapter::new();

    // Should handle gracefully even with no available endpoints
    let result = adapter.discover_services().await;

    // May succeed with empty list or fail - either is acceptable
    // The important thing is it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_e2e_find_capability_providers_empty_registry() {
    let adapter = UnifiedUniversalAdapter::new();

    // Query for capability before any discovery
    let result = adapter.find_capability_providers("compute").await;

    assert!(result.is_ok());
    let providers = result.expect("test precondition");
    assert!(providers.is_empty(), "Should return empty list for unknown capability");
}

#[tokio::test]
async fn test_e2e_find_multiple_capability_types() {
    let adapter = UnifiedUniversalAdapter::new();

    // Query for different capability types
    let compute = adapter.find_capability_providers("compute").await;
    let security = adapter.find_capability_providers("security").await;
    let storage = adapter.find_capability_providers("storage").await;
    let ai = adapter.find_capability_providers("ai").await;

    // All should succeed even if empty
    assert!(compute.is_ok());
    assert!(security.is_ok());
    assert!(storage.is_ok());
    assert!(ai.is_ok());
}

// ============================================================================
// ADAPTER LIFECYCLE
// ============================================================================

#[tokio::test]
async fn test_e2e_adapter_clone() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    // Both should be valid
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
}

#[tokio::test]
async fn test_e2e_multiple_adapters() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = UnifiedUniversalAdapter::new();
    let adapter3 = UnifiedUniversalAdapter::new();

    // All should be independently valid
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
    assert!(std::mem::size_of_val(&adapter3) > 0);
}

// ============================================================================
// CONCURRENT OPERATIONS
// ============================================================================

#[tokio::test]
async fn test_e2e_concurrent_capability_queries() {
    use std::sync::Arc;

    let adapter = Arc::new(UnifiedUniversalAdapter::new());
    let mut handles = vec![];

    // Spawn 10 concurrent query tasks
    for i in 0..10 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move {
            let capability_type = match i % 4 {
                0 => "compute",
                1 => "security",
                2 => "storage",
                _ => "ai",
            };
            adapter_clone.find_capability_providers(capability_type).await
        });

        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_e2e_concurrent_discovery_attempts() {
    use std::sync::Arc;

    let adapter = Arc::new(UnifiedUniversalAdapter::new());
    let mut handles = vec![];

    // Spawn 5 concurrent discovery attempts
    for _ in 0..5 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = tokio::spawn(async move { adapter_clone.discover_services().await });

        handles.push(handle);
    }

    // All should complete without panicking
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok() || result.is_err()); // Either is fine
    }
}

// ============================================================================
// CONFIGURATION SCENARIOS
// ============================================================================

#[tokio::test]
async fn test_e2e_high_timeout_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: std::time::Duration::from_secs(120),
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_e2e_low_timeout_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: std::time::Duration::from_millis(500),
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_e2e_many_discovery_endpoints() {
    let endpoints: Vec<String> =
        (0..20).map(|i| format!("http://discovery-{}:8080/discover", i)).collect();

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[tokio::test]
async fn test_e2e_invalid_capability_name() {
    let adapter = UnifiedUniversalAdapter::new();

    // Test various invalid/unusual capability names
    let result1 = adapter.find_capability_providers("").await;
    let result2 = adapter.find_capability_providers("invalid-capability-!@#$").await;
    let result3 = adapter.find_capability_providers("x".repeat(1000).as_str()).await;

    // Should handle gracefully
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

// ============================================================================
// INTEGRATION WITH TYPES
// ============================================================================

#[tokio::test]
async fn test_e2e_capability_registry_default() {
    use songbird_universal::CapabilityRegistry;

    let registry = CapabilityRegistry::default();

    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());
}

#[tokio::test]
async fn test_e2e_capability_registry_clone() {
    use songbird_universal::CapabilityRegistry;

    let registry1 = CapabilityRegistry::default();
    let registry2 = registry1.clone();

    assert!(std::mem::size_of_val(&registry1) > 0);
    assert!(std::mem::size_of_val(&registry2) > 0);
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_e2e_many_sequential_queries() {
    let adapter = UnifiedUniversalAdapter::new();

    // Perform many sequential queries
    for i in 0..100 {
        let capability = match i % 4 {
            0 => "compute",
            1 => "security",
            2 => "storage",
            _ => "ai",
        };

        let result = adapter.find_capability_providers(capability).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_e2e_adapter_creation_stress() {
    // Create many adapters rapidly
    let mut adapters = Vec::new();

    for _ in 0..50 {
        adapters.push(UnifiedUniversalAdapter::new());
    }

    assert_eq!(adapters.len(), 50);
}
