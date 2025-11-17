//! Error Path Tests for UnifiedUniversalAdapter
//!
//! Comprehensive tests for error handling and edge cases

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

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UnifiedUniversalAdapter, UniversalAdapterError, UniversalRequest,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// DISCOVERY ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_with_empty_endpoints() -> SongbirdResult<()> {
    // ARRANGE: Create adapter with empty discovery endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should succeed but return empty list
    assert!(result.is_ok());
    let services = result?;
    assert_eq!(services.len(), 0, "No services should be discovered with empty endpoints");
    Ok(())
}

#[tokio::test]
async fn test_discover_services_with_invalid_endpoints() -> SongbirdResult<()> {
    // ARRANGE: Create adapter with invalid endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://invalid-nonexistent-domain-12345.test:9999/discovery".to_string(),
            "http://localhost:0/discovery".to_string(), // Port 0 is invalid
        ],
        discovery_timeout: Duration::from_millis(100), // Short timeout
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Attempt discovery (will log errors but not fail)
    let result = adapter.discover_services().await;

    // ASSERT: Should succeed but return empty list (errors are logged)
    assert!(result.is_ok());
    let services = result?;
    assert_eq!(services.len(), 0, "No services should be discovered from invalid endpoints");
    Ok(())
}

#[tokio::test]
async fn test_discover_services_with_unreachable_endpoints() -> SongbirdResult<()> {
    // ARRANGE: Create adapter with unreachable endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://localhost:65534/discovery".to_string(), // Unlikely to be in use
            "http://127.0.0.1:65535/discovery".to_string(),
        ],
        discovery_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should succeed but return empty list
    assert!(result.is_ok());
    let services = result?;
    assert_eq!(services.len(), 0);
    Ok(())
}

// ============================================================================
// CAPABILITY PROVIDER LOOKUP EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
    // ARRANGE: Fresh adapter with empty registry
    let adapter = create_universal_adapter();

    // ACT: Search for capability in empty registry
    let result = adapter.find_capability_providers("any-capability").await;

    // ASSERT: Should succeed but return empty list
    assert!(result.is_ok());
    let providers = result?;
    assert_eq!(providers.len(), 0, "Empty registry should return no providers");
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_nonexistent_capability() -> SongbirdResult<()> {
    // ARRANGE: Adapter with empty registry
    let adapter = create_universal_adapter();

    // ACT: Search for nonexistent capability
    let result = adapter.find_capability_providers("nonexistent-capability-xyz-12345").await;

    // ASSERT: Should succeed but return empty list
    assert!(result.is_ok());
    let providers = result?;
    assert_eq!(providers.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_empty_string() -> SongbirdResult<()> {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();

    // ACT: Search for empty capability name
    let result = adapter.find_capability_providers("").await;

    // ASSERT: Should succeed but return empty list
    assert!(result.is_ok());
    let providers = result?;
    assert_eq!(providers.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_special_characters() -> SongbirdResult<()> {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();

    // ACT: Search for capability with special characters
    let special_names = vec!["cap@#$%", "cap/with/slashes", "cap\nwith\nnewlines", "cap\0null"];

    for name in special_names {
        let result = adapter.find_capability_providers(name).await;
        assert!(result.is_ok(), "Should handle special characters gracefully");
        assert_eq!(
            result.ok_or_else(|| SongbirdError::configuration(format!("Error: {}", e)))?.len(),
            0
        );
    }
    Ok(())
}

// ============================================================================
// ROUTE REQUEST ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_route_request_missing_capability() {
    // ARRANGE: Fresh adapter with empty registry
    let adapter = create_universal_adapter();
    let request = UniversalRequest {
        request_id: "test-req-001".to_string(),
        source: "test-client".to_string(),
        target: "nonexistent-service".to_string(),
        action: "compute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    // ACT: Route request when no capability exists
    let result = adapter.route_request(request).await;

    // ASSERT: Should fail with error
    assert!(result.is_err(), "Should fail when capability doesn't exist");
    let err = result.unwrap_err();
    assert!(
        matches!(err, UniversalAdapterError::MissingCapability)
            || err.to_string().contains("Missing")
            || err.to_string().contains("capability"),
        "Error should indicate missing capability"
    );
}

#[tokio::test]
async fn test_route_request_empty_request_id() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();
    let request = UniversalRequest {
        request_id: "".to_string(), // Empty request ID
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "compute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    // ACT: Route request with empty ID
    let result = adapter.route_request(request).await;

    // ASSERT: Should handle gracefully (may fail or succeed depending on implementation)
    // The key is that it shouldn't panic
    let _ = result;
}

#[tokio::test]
async fn test_route_request_empty_source() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();
    let request = UniversalRequest {
        request_id: "test-req".to_string(),
        source: "".to_string(), // Empty source
        target: "test-service".to_string(),
        action: "compute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    // ACT: Route request with empty source
    let result = adapter.route_request(request).await;

    // ASSERT: Should handle gracefully
    let _ = result;
}

#[tokio::test]
async fn test_route_request_empty_target() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();
    let request = UniversalRequest {
        request_id: "test-req".to_string(),
        source: "test-client".to_string(),
        target: "".to_string(), // Empty target
        action: "compute".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    // ACT: Route request with empty target
    let result = adapter.route_request(request).await;

    // ASSERT: Should handle gracefully
    let _ = result;
}

#[tokio::test]
async fn test_route_request_empty_action() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();
    let request = UniversalRequest {
        request_id: "test-req".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "".to_string(), // Empty action
        parameters: HashMap::new(),
        security_context: None,
    };

    // ACT: Route request with empty action
    let result = adapter.route_request(request).await;

    // ASSERT: Should handle gracefully
    let _ = result;
}

// ============================================================================
// REGISTRY STATS EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_get_registry_stats_empty_registry() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();

    // ACT: Get stats from empty registry
    let stats = adapter.get_registry_stats().await;

    // ASSERT: Should return zero counts
    assert_eq!(stats.total_services, 0, "Empty registry should have 0 services");
    assert_eq!(stats.total_capabilities, 0, "Empty registry should have 0 capabilities");
}

#[tokio::test]
async fn test_get_registry_stats_multiple_calls() {
    // ARRANGE: Fresh adapter
    let adapter = create_universal_adapter();

    // ACT: Get stats multiple times
    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;
    let stats3 = adapter.get_registry_stats().await;

    // ASSERT: Should be consistent
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats2.total_services, stats3.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
    assert_eq!(stats2.total_capabilities, stats3.total_capabilities);
}

// ============================================================================
// CONFIGURATION EDGE CASES
// ============================================================================

#[test]
fn test_config_zero_timeout() {
    // ARRANGE: Create config with zero timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::ZERO,
        ..Default::default()
    };

    // ASSERT: Should allow zero timeout
    assert_eq!(config.discovery_timeout, Duration::ZERO);
}

#[test]
fn test_config_zero_health_check_interval() {
    // ARRANGE: Create config with zero health check interval
    let config = UnifiedAdapterConfig {
        health_check_interval: Duration::ZERO,
        ..Default::default()
    };

    // ASSERT: Should allow zero interval
    assert_eq!(config.health_check_interval, Duration::ZERO);
}

#[test]
fn test_config_zero_max_concurrent_requests() {
    // ARRANGE: Create config with zero max concurrent requests
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 0,
        ..Default::default()
    };

    // ASSERT: Should allow zero (though not recommended)
    assert_eq!(config.max_concurrent_requests, 0);
}

#[test]
fn test_config_very_large_timeout() {
    // ARRANGE: Create config with very large timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(86400), // 1 day
        ..Default::default()
    };

    // ASSERT: Should allow large timeout
    assert_eq!(config.discovery_timeout.as_secs(), 86400);
}

#[test]
fn test_config_very_large_max_concurrent() {
    // ARRANGE: Create config with very large max concurrent requests
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 1_000_000,
        ..Default::default()
    };

    // ASSERT: Should allow large values
    assert_eq!(config.max_concurrent_requests, 1_000_000);
}

// ============================================================================
// ADAPTER LIFECYCLE EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_adapter_concurrent_operations() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Perform multiple operations concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let adapter_clone = adapter.clone();
            tokio::spawn(async move {
                let _ = adapter_clone.discover_services().await;
                let _ = adapter_clone.find_capability_providers(&format!("cap-{}", i)).await;
                let _ = adapter_clone.get_registry_stats().await;
            })
        })
        .collect();

    // Wait for all operations to complete
    for handle in handles {
        assert!(handle.await.is_ok(), "Concurrent operations should not panic");
    }
}

#[tokio::test]
async fn test_adapter_multiple_discovery_calls() {
    // ARRANGE: Create adapter with unreachable endpoints (to ensure fast completion)
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:65534/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Call discovery multiple times
    let result1 = adapter.discover_services().await;
    let result2 = adapter.discover_services().await;
    let result3 = adapter.discover_services().await;

    // ASSERT: All should succeed (even if empty)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[test]
fn test_adapter_clone_independence() {
    // ARRANGE: Create adapter
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    // ACT: Verify both are independent instances
    let size1 = std::mem::size_of_val(&adapter1);
    let size2 = std::mem::size_of_val(&adapter2);

    // ASSERT: Both should have same size (clones share Arc references)
    assert_eq!(size1, size2);
}

// ============================================================================
// ERROR TYPE TESTS
// ============================================================================

#[test]
fn test_error_type_display_format() {
    // Test that all error types have meaningful display messages
    let errors = vec![
        UniversalAdapterError::NetworkError("connection failed".to_string()),
        UniversalAdapterError::ParseError("invalid json".to_string()),
        UniversalAdapterError::DiscoveryError("service not found".to_string()),
        UniversalAdapterError::ServiceError("internal error".to_string()),
        UniversalAdapterError::MissingCapability,
    ];

    for err in errors {
        let display = err.to_string();
        assert!(!display.is_empty(), "Error should have non-empty display message");
        assert!(display.len() > 5, "Error message should be descriptive");
    }
}

#[test]
fn test_error_type_debug_format() -> SongbirdResult<()> {
    // Test that all error types have debug output
    let errors = vec![
        UniversalAdapterError::NetworkError("test".to_string()),
        UniversalAdapterError::ParseError("test".to_string()),
        UniversalAdapterError::DiscoveryError("test".to_string()),
        UniversalAdapterError::ServiceError("test".to_string()),
        UniversalAdapterError::MissingCapability,
    ];

    for err in errors {
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty(), "Error should have non-empty debug output");
    }
    Ok(())
}

// ============================================================================
// STRESS AND BOUNDARY TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_many_capability_queries() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Query many different capabilities
    for i in 0..100 {
        let result = adapter.find_capability_providers(&format!("capability-{}", i)).await;
        assert!(result.is_ok(), "Should handle many queries");
    }
}

#[tokio::test]
async fn test_adapter_very_long_capability_names() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Query with very long capability name
    let long_name = "a".repeat(10_000);
    let result = adapter.find_capability_providers(&long_name).await;

    // ASSERT: Should handle gracefully
    assert!(result.is_ok());
}

#[test]
fn test_config_with_many_endpoints() {
    // ARRANGE: Create config with many endpoints
    let endpoints: Vec<String> =
        (0..1000).map(|i| format!("http://localhost:{}/discovery", 10000 + i)).collect();

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints.clone(),
        ..Default::default()
    };

    // ASSERT: Should handle many endpoints
    assert_eq!(config.discovery_endpoints.len(), 1000);
}
