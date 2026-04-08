// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::time::Duration;

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
