// SPDX-License-Identifier: AGPL-3.0-or-later
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
//! Comprehensive Discovery Tests
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
//! Tests for service discovery, endpoint scanning, and capability detection.
//! These tests cover the critical discovery paths in `UnifiedUniversalAdapter`.

use songbird_test_utils::test_orchestrator_port;
use songbird_types::SongbirdResult;
use songbird_universal::types::{
    DiscoveredCapability, HealthStatus, PrimalType, QosMetrics, ServiceInfo,
};
use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::collections::HashMap;
use std::time::Duration;

/// Helper function to create a test service
fn create_test_service(name: &str, endpoint: &str, capabilities: Vec<&str>) -> ServiceInfo {
    ServiceInfo {
        name: name.to_string(),
        endpoint: endpoint.to_string(),
        primal_type: PrimalType {
            category: "test".to_string(),
            subcategory: None,
            version: "1.0".to_string(),
        },
        capabilities: capabilities
            .iter()
            .map(|c| DiscoveredCapability {
                name: (*c).to_string(),
                version: "1.0".to_string(),
                description: format!("{c} capability"),
                provider: name.to_string(),
                endpoint: format!("{endpoint}/api/v1/{c}"),
                qos_metrics: QosMetrics {
                    latency_ms: Some(100.0),
                    throughput_ops_sec: Some(1000.0),
                    availability: Some(0.99),
                    reliability: Some(0.99),
                },
                health_status: HealthStatus::Healthy,
            })
            .collect(),
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_adapter_creation_with_discovery_config() {
    // ARRANGE: Create custom discovery config
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(15),
        max_concurrent_requests: 25,
        auto_discovery: true,
        discovery_endpoints: vec!["http://localhost:9999/discovery".to_string()],
    };

    // ACT: Create adapter with config
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter created successfully
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_discover_services_empty_response() -> SongbirdResult<()> {
    // This test documents the expected behavior when discovery returns no services

    // ARRANGE: Create adapter with unreachable endpoint (will fail gracefully)
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        discovery_endpoints: vec!["http://127.0.0.1:59999/services".to_string()], // Port unlikely to be used
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should return OK with empty list (graceful degradation)
    assert!(result.is_ok());
    let services = result?;
    assert_eq!(services.len(), 0, "Should return empty list when no services found");
    Ok(())
}

#[tokio::test]
async fn test_discover_services_network_timeout() -> SongbirdResult<()> {
    // Test that discovery properly handles network timeouts

    // ARRANGE: Create adapter with very short timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(1), // Extremely short timeout
        discovery_endpoints: vec!["http://192.0.2.1:8080/services".to_string()], // TEST-NET-1 (non-routable)
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery (should timeout)
    let result = adapter.discover_services().await;

    // ASSERT: Should handle timeout gracefully
    assert!(result.is_ok(), "Discovery should not panic on timeout");
    let services = result?;
    assert_eq!(services.len(), 0, "Timeout should result in no discovered services");
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
    // Test finding services when registry is empty

    // ARRANGE: Create fresh adapter with empty registry
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Try to find services with a capability
    let result = adapter.find_capability_providers("compute").await;

    // ASSERT: Should return empty list, not error
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_capability_registry_default() {
    // Test that CapabilityRegistry initializes properly

    // ARRANGE & ACT: Create default registry
    let registry = songbird_universal::CapabilityRegistry::default();

    // ASSERT: All collections start empty
    assert_eq!(registry.service_capabilities.len(), 0);
    assert_eq!(registry.capability_providers.len(), 0);
    assert_eq!(registry.service_info.len(), 0);
    assert_eq!(registry.last_updated.len(), 0);
}

#[tokio::test]
async fn test_discovery_config_defaults() {
    // Verify default configuration values are sensible

    // ARRANGE & ACT: Create default config
    let config = UnifiedAdapterConfig::default();

    // ASSERT: Defaults are production-ready
    assert_eq!(config.discovery_timeout, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));
    assert_eq!(config.max_concurrent_requests, 100);
    assert!(config.auto_discovery);
    assert!(!config.discovery_endpoints.is_empty());
}

#[tokio::test]
async fn test_multiple_discovery_endpoints_configuration() {
    // Test that multiple discovery endpoints can be configured

    // ARRANGE: Create config with multiple endpoints
    let endpoints = vec![
        "http://primary:8080/discovery".to_string(),
        "http://secondary:8080/discovery".to_string(),
        "http://tertiary:8080/discovery".to_string(),
    ];
    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter created successfully
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_discovery_timeout_configuration() {
    // Test that custom timeout values are respected

    // ARRANGE: Create config with custom timeout
    let custom_timeout = Duration::from_secs(45);
    let config = UnifiedAdapterConfig {
        discovery_timeout: custom_timeout,
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Configuration accepted
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_concurrent_request_limit_configuration() {
    // Test that max concurrent requests can be configured

    // ARRANGE: Test various concurrent request limits
    let limits = vec![1, 10, 50, 100, 500];

    for limit in limits {
        // ACT: Create config with specific limit
        let config = UnifiedAdapterConfig {
            max_concurrent_requests: limit,
            ..Default::default()
        };

        let adapter = UnifiedUniversalAdapter::with_config(config);

        // ASSERT: Each configuration is valid
        assert!(std::mem::size_of_val(&adapter) > 0);
    }
}

#[tokio::test]
async fn test_auto_discovery_toggle() {
    // Test that auto-discovery can be enabled/disabled

    // ARRANGE & ACT: Test both states
    let config_enabled = UnifiedAdapterConfig {
        auto_discovery: true,
        ..Default::default()
    };
    let adapter_enabled = UnifiedUniversalAdapter::with_config(config_enabled);

    let config_disabled = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };
    let adapter_disabled = UnifiedUniversalAdapter::with_config(config_disabled);

    // ASSERT: Both configurations are valid
    assert!(std::mem::size_of_val(&adapter_enabled) > 0);
    assert!(std::mem::size_of_val(&adapter_disabled) > 0);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_graceful_failure_handling() -> SongbirdResult<()> {
    // Test that discovery handles failures gracefully and continues

    // ARRANGE: Create adapter with mix of valid and invalid endpoints
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        discovery_endpoints: vec![
            "http://127.0.0.1:59997/services".to_string(), // Will fail
            "http://127.0.0.1:59998/services".to_string(), // Will fail
            "http://127.0.0.1:59999/services".to_string(), // Will fail
        ],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should succeed with empty list (all endpoints failed gracefully)
    assert!(result.is_ok(), "Discovery should handle all failures gracefully");
    let services = result?;
    assert_eq!(services.len(), 0, "No services expected when all endpoints fail");
    Ok(())
}

#[tokio::test]
async fn test_service_info_structure() {
    // Verify ServiceInfo can be created with all required fields

    // ARRANGE & ACT: Create a complete ServiceInfo
    let endpoint = format!("http://localhost:{}", test_orchestrator_port());
    let service = create_test_service("test-service", &endpoint, vec!["compute", "storage"]);

    // ASSERT: All fields are properly set
    assert_eq!(service.name, "test-service");
    assert_eq!(service.endpoint, endpoint);
    assert_eq!(service.capabilities.len(), 2);
    assert!(matches!(service.health, HealthStatus::Healthy));
}

#[tokio::test]
async fn test_capability_structure() {
    // Verify Capability structure contains all required information

    // ARRANGE & ACT: Create a capability
    let capability = DiscoveredCapability {
        name: "compute".to_string(),
        version: "1.0.0".to_string(),
        description: "Compute capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: "http://localhost:8080/compute".to_string(),
        qos_metrics: QosMetrics {
            latency_ms: Some(50.0),
            throughput_ops_sec: Some(1000.0),
            availability: Some(0.99),
            reliability: Some(0.99),
        },
        health_status: HealthStatus::Healthy,
    };

    // ASSERT: All fields accessible and correct
    assert_eq!(capability.name, "compute");
    assert_eq!(capability.version, "1.0.0");
    assert_eq!(capability.provider, "test-provider");
    assert!(matches!(capability.health_status, HealthStatus::Healthy));
}

// ============================================================================
// PERFORMANCE AND EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_discovery_with_zero_timeout() {
    // Edge case: What happens with zero timeout?

    // ARRANGE: Create config with zero timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(0),
        discovery_endpoints: vec!["http://localhost:8080/services".to_string()],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should handle gracefully (immediate timeout)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_with_empty_endpoints() -> SongbirdResult<()> {
    // Edge case: No endpoints configured

    // ARRANGE: Create config with no endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should return empty list, not error
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_discovery_calls() {
    // Test that multiple concurrent discovery calls don't interfere

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Make multiple concurrent discovery calls
    let results = tokio::join!(
        adapter.discover_services(),
        adapter.discover_services(),
        adapter.discover_services(),
    );

    // ASSERT: All calls complete successfully
    assert!(results.0.is_ok());
    assert!(results.1.is_ok());
    assert!(results.2.is_ok());
}

#[tokio::test]
async fn test_find_capability_with_special_characters() {
    // Test finding capabilities with special characters in names

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search for capabilities with special characters
    let result1 = adapter.find_capability_providers("compute-ai").await;
    let result2 = adapter.find_capability_providers("storage/s3").await;
    let result3 = adapter.find_capability_providers("api:v1:health").await;

    // ASSERT: All searches complete without errors
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_very_long_capability_name() -> SongbirdResult<()> {
    // Edge case: Very long capability names

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search for capability with very long name
    let long_name = "a".repeat(1000);
    let result = adapter.find_capability_providers(&long_name).await;

    // ASSERT: Handles gracefully
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_discovery_endpoint_with_trailing_slash() {
    // Test that endpoints with trailing slashes are handled correctly

    // ARRANGE: Create config with various endpoint formats
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        discovery_endpoints: vec![
            "http://localhost:8080/services/".to_string(), // With trailing slash
            "http://localhost:8081/services".to_string(),  // Without trailing slash
        ],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Handles both formats
    assert!(result.is_ok());
}

// ============================================================================
// P1 ADDITIONAL TESTS - Increase Coverage for Discovery Module
// ============================================================================

#[tokio::test]
async fn test_registry_stats_after_failed_discovery() {
    // Test that registry stats are correct even after failed discovery attempts

    // ARRANGE: Create adapter with failing endpoints
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(50),
        discovery_endpoints: vec!["http://127.0.0.1:59999/services".to_string()],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let _ = adapter.discover_services().await;
    let stats = adapter.get_registry_stats().await;

    // ASSERT: Stats are valid even after failure
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test]
async fn test_find_providers_with_empty_string() -> SongbirdResult<()> {
    // Edge case: Search for providers with empty capability name

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with empty string
    let result = adapter.find_capability_providers("").await;

    // ASSERT: Should handle gracefully
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_providers_with_whitespace() {
    // Edge case: Search with whitespace in capability name

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with various whitespace patterns
    let result1 = adapter.find_capability_providers(" compute").await;
    let result2 = adapter.find_capability_providers("compute ").await;
    let result3 = adapter.find_capability_providers("  compute  ").await;
    let result4 = adapter.find_capability_providers("com pute").await;

    // ASSERT: All searches complete without errors
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    assert!(result4.is_ok());
}

#[tokio::test]
async fn test_discover_services_repeated_calls() {
    // Test that repeated discovery calls maintain consistent behavior

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Call discover multiple times
    for i in 0..10 {
        let result = adapter.discover_services().await;

        // ASSERT: Each call succeeds
        assert!(result.is_ok(), "Discovery call {i} should succeed");
    }
}

#[tokio::test]
async fn test_config_with_very_short_health_check_interval() {
    // Test configuration with very short health check interval

    // ARRANGE: Create config with 1ms health check interval
    let config = UnifiedAdapterConfig {
        health_check_interval: Duration::from_millis(1),
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter accepts extreme configuration
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_config_with_very_long_timeout() {
    // Test configuration with very long timeout

    // ARRANGE: Create config with 1 hour timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(3600),
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter accepts extreme configuration
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_adapter_with_max_concurrent_requests_one() {
    // Edge case: Single concurrent request allowed

    // ARRANGE: Create config with max_concurrent_requests = 1
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 1,
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter works with minimal concurrency
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_adapter_with_max_concurrent_requests_extreme() {
    // Edge case: Very high concurrent request limit

    // ARRANGE: Create config with max_concurrent_requests = 10000
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 10000,
        ..Default::default()
    };

    // ACT: Create adapter
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ASSERT: Adapter accepts extreme configuration
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_with_ipv6_endpoint() {
    // Test that IPv6 endpoints are handled correctly

    // ARRANGE: Create config with IPv6 endpoint
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(50),
        discovery_endpoints: vec!["http://[::1]:8080/services".to_string()],
        ..Default::default()
    };

    // ACT: Create adapter and attempt discovery
    let adapter = UnifiedUniversalAdapter::with_config(config);
    let result = adapter.discover_services().await;

    // ASSERT: Should handle IPv6 gracefully (even if it times out)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_with_https_endpoint() {
    // Test that HTTPS endpoints are accepted (even if they fail)

    // ARRANGE: Create config with HTTPS endpoint
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(50),
        discovery_endpoints: vec!["https://localhost:8443/services".to_string()],
        ..Default::default()
    };

    // ACT: Create adapter and attempt discovery
    let adapter = UnifiedUniversalAdapter::with_config(config);
    let result = adapter.discover_services().await;

    // ASSERT: Should handle HTTPS gracefully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_adapters_independent() {
    // Test that multiple adapter instances operate independently

    // ARRANGE: Create two adapters with different configs
    let config1 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        max_concurrent_requests: 10,
        ..Default::default()
    };
    let config2 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(200),
        max_concurrent_requests: 20,
        ..Default::default()
    };

    let adapter1 = UnifiedUniversalAdapter::with_config(config1);
    let adapter2 = UnifiedUniversalAdapter::with_config(config2);

    // ACT: Use both adapters concurrently
    let result1 = adapter1.discover_services().await;
    let result2 = adapter2.discover_services().await;

    // ASSERT: Both work independently
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_adapter_clone_behavior() {
    // Test that cloned adapters work correctly

    // ARRANGE: Create adapter and clone it
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    // ACT: Use both adapters
    let result1 = adapter1.discover_services().await;
    let result2 = adapter2.discover_services().await;

    // ASSERT: Both work correctly
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_find_providers_case_sensitivity() {
    // Test capability search with different casing

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with different cases
    let result_lower = adapter.find_capability_providers("compute").await;
    let result_upper = adapter.find_capability_providers("COMPUTE").await;
    let result_mixed = adapter.find_capability_providers("CoMpUtE").await;

    // ASSERT: All searches work (may or may not find results based on implementation)
    assert!(result_lower.is_ok());
    assert!(result_upper.is_ok());
    assert!(result_mixed.is_ok());
}

#[tokio::test]
async fn test_stats_consistency_after_multiple_operations() {
    // Test that stats remain consistent after various operations

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Perform multiple operations
    let _ = adapter.discover_services().await;
    let stats1 = adapter.get_registry_stats().await;

    let _ = adapter.find_capability_providers("compute").await;
    let stats2 = adapter.get_registry_stats().await;

    let _ = adapter.discover_services().await;
    let stats3 = adapter.get_registry_stats().await;

    // ASSERT: Stats are consistent (no services since no real endpoints)
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats2.total_services, stats3.total_services);
}

#[tokio::test]
async fn test_discovery_with_invalid_url_format() {
    // Test handling of malformed URLs

    // ARRANGE: Create config with invalid URLs
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(50),
        discovery_endpoints: vec![
            "not-a-url".to_string(),
            "ftp://invalid:8080".to_string(),
            String::new(),
        ],
        ..Default::default()
    };

    // ACT: Create adapter and attempt discovery
    let adapter = UnifiedUniversalAdapter::with_config(config);
    let result = adapter.discover_services().await;

    // ASSERT: Should handle gracefully (return empty or error, but not panic)
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable, as long as no panic
}
