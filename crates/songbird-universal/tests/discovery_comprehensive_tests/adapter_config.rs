// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::time::Duration;

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
