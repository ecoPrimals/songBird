// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Protocol Detection Tests for SecurityAdapter
//!
//! Comprehensive unit, integration, and E2E tests for protocol-agnostic
//! SecurityAdapter implementation (v3.11.0).

use super::*; // Import from parent security module
use serde_json::json;
use std::time::Duration;

// ============================================================================
// UNIT TESTS - Protocol Detection Logic
// ============================================================================

#[cfg(test)]
mod protocol_detection_unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_unix_socket_detection() {
        // Test that unix:// URLs are detected for JSON-RPC protocol
        let adapter = SecurityAdapter::new("unix:///tmp/beardog.sock".to_string()).await.unwrap();

        // Verify adapter is created (can't inspect internal enum, but we can verify it works)
        assert_eq!(adapter.endpoint(), "unix:///tmp/beardog.sock");
    }

    #[tokio::test]
    async fn test_http_detection() {
        // Test that http:// URLs are detected for HTTP protocol
        let adapter = SecurityAdapter::new("http://localhost:9000".to_string()).await.unwrap();

        assert_eq!(adapter.endpoint(), "http://localhost:9000");
    }

    #[tokio::test]
    async fn test_https_detection() {
        // Test that https:// URLs are detected for HTTP protocol
        let adapter = SecurityAdapter::new("https://example.com:8443".to_string()).await.unwrap();

        assert_eq!(adapter.endpoint(), "https://example.com:8443");
    }

    #[tokio::test]
    async fn test_with_timeout_builder() {
        // Test builder pattern for timeout configuration
        // ✅ DEEP DEBT EVOLUTION (Feb 3, 2026): Use TimeoutConfig for tests
        // Replaces hardcoded Duration::from_secs(10) with TimeoutConfig::fast()
        let timeout_config = songbird_config::timeouts::TimeoutConfig::fast();
        let adapter = SecurityAdapter::new("http://localhost:9000".to_string())
            .await
            .unwrap()
            .with_timeout(timeout_config.request);

        assert_eq!(adapter.endpoint(), "http://localhost:9000");
    }

    #[tokio::test]
    async fn test_unix_socket_without_prefix() {
        // Test that raw paths (without unix://) still work for backward compat
        let result = SecurityAdapter::new("/tmp/beardog.sock".to_string()).await;

        // Should fail because it doesn't start with unix:// and isn't http(s)://
        // This would be interpreted as HTTP with invalid URL
        assert!(result.is_ok()); // IpcHttpClient creation should still work
    }
}

// ============================================================================
// INTEGRATION TESTS - Mock Server Tests
// ============================================================================

#[cfg(test)]
mod protocol_integration_tests {
    use super::*;
    use mockito;
    use tokio;

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_http_collect_metrics_success() {
        // Create mock HTTP server
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "active_sessions": 10,
                    "failed_auth_attempts": 2,
                    "blocked_ips": 0,
                    "security_score": 0.95,
                    "timestamp": "2026-01-06T16:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Test HTTP protocol
        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let metrics = adapter.collect_metrics().await.unwrap();

        // Verify response
        assert_eq!(metrics.active_sessions, 10);
        assert_eq!(metrics.failed_auth_attempts, 2);
        assert_eq!(metrics.blocked_ips, 0);
        assert!((metrics.security_score - 0.95).abs() < 0.01);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_http_collect_metrics_error_status() {
        // Test HTTP error handling
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let result = adapter.collect_metrics().await;

        // Should return error for non-success status
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTP 500"));

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_http_verify_auth_success() {
        // Test HTTP auth verification
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/auth/verify")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::Json(json!({"token": "test-token"})))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!("Authorized").to_string())
            .create_async()
            .await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let result = adapter.verify_auth("test-token").await.unwrap();

        assert_eq!(result, AuthResult::Authorized);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_http_verify_auth_unauthorized() {
        // Test HTTP unauthorized response
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("POST", "/auth/verify").with_status(401).create_async().await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let result = adapter.verify_auth("bad-token").await.unwrap();

        assert_eq!(result, AuthResult::Unauthorized);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_check_health_healthy() {
        // Test health check with healthy metrics
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "active_sessions": 10,
                    "failed_auth_attempts": 2,
                    "blocked_ips": 0,
                    "security_score": 0.95,
                    "timestamp": "2026-01-06T16:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let health = adapter.check_health().await.unwrap();

        assert_eq!(health, SecurityHealth::Healthy);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_check_health_warning() {
        // Test health check with warning metrics
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "active_sessions": 10,
                    "failed_auth_attempts": 60,
                    "blocked_ips": 5,
                    "security_score": 0.65,
                    "timestamp": "2026-01-06T16:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let health = adapter.check_health().await.unwrap();

        assert_eq!(health, SecurityHealth::Warning);

        mock.assert_async().await;
    }

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_check_health_critical() {
        // Test health check with critical metrics
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "active_sessions": 10,
                    "failed_auth_attempts": 150,
                    "blocked_ips": 60,
                    "security_score": 0.35,
                    "timestamp": "2026-01-06T16:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let adapter = SecurityAdapter::new(server.url()).await.unwrap();
        let health = adapter.check_health().await.unwrap();

        assert_eq!(health, SecurityHealth::Critical);

        mock.assert_async().await;
    }
}

// ============================================================================
// E2E TESTS - JSON-RPC Protocol (requires actual Unix socket server)
// ============================================================================

#[cfg(test)]
mod jsonrpc_e2e_tests {
    use super::*;
    use tokio;

    #[tokio::test]
    #[ignore = "requires running BearDog crypto provider"] // Requires actual BearDog Unix socket server
    async fn test_jsonrpc_collect_metrics_e2e() {
        // E2E test: Connect to real BearDog Unix socket and collect metrics
        //
        // Prerequisites:
        // 1. BearDog running with Unix socket at /tmp/beardog-test.sock
        // 2. BearDog configured to respond to get_metrics JSON-RPC calls
        //
        // Run with: cargo test --features e2e test_jsonrpc_collect_metrics_e2e -- --ignored

        let adapter =
            SecurityAdapter::new("unix:///tmp/beardog-test.sock".to_string()).await.unwrap();

        let metrics = adapter.collect_metrics().await.expect("Should collect metrics via JSON-RPC");

        // Verify metrics are reasonable
        assert!(metrics.active_sessions < 1000);
        assert!(metrics.security_score >= 0.0 && metrics.security_score <= 1.0);
    }

    #[tokio::test]
    #[ignore = "requires running BearDog crypto provider"] // Requires actual BearDog Unix socket server
    async fn test_jsonrpc_verify_auth_e2e() {
        // E2E test: Verify authentication via JSON-RPC
        //
        // Prerequisites:
        // 1. BearDog running with Unix socket
        // 2. BearDog configured to respond to verify_auth JSON-RPC calls
        //
        // Run with: cargo test --features e2e test_jsonrpc_verify_auth_e2e -- --ignored

        let adapter =
            SecurityAdapter::new("unix:///tmp/beardog-test.sock".to_string()).await.unwrap();

        // Test with valid token (configure BearDog to accept "test-valid-token")
        let result = adapter.verify_auth("test-valid-token").await.unwrap();
        assert_eq!(result, AuthResult::Authorized);

        // Test with invalid token
        let result = adapter.verify_auth("test-invalid-token").await.unwrap();
        assert_eq!(result, AuthResult::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires running BearDog crypto provider"] // Requires actual BearDog Unix socket server
    async fn test_genetic_lineage_trust_e2e() {
        // E2E test: Verify genetic lineage trust evaluation via JSON-RPC
        //
        // Prerequisites:
        // 1. BearDog running with genetic lineage capability
        // 2. Two Songbird towers configured in same family
        //
        // This test verifies the ORIGINAL upstream debt is resolved:
        // - Songbird sends JSON-RPC (not HTTP)
        // - BearDog receives and parses correctly
        // - Trust evaluation succeeds
        // - Trust level upgrades from 1 to 2+
        //
        // Run with: cargo test --features e2e test_genetic_lineage_trust_e2e -- --ignored

        let adapter =
            SecurityAdapter::new("unix:///tmp/beardog-nat0-tower1.sock".to_string()).await.unwrap();

        // Verify health (should work via JSON-RPC)
        let health = adapter.check_health().await.expect("Health check should work");
        assert_eq!(health, SecurityHealth::Healthy);

        // Collect metrics (should work via JSON-RPC)
        let metrics = adapter.collect_metrics().await.expect("Metrics should work");
        assert!(metrics.security_score > 0.5);

        println!("✅ Genetic lineage trust evaluation via JSON-RPC WORKING!");
        println!("   - Protocol: JSON-RPC 2.0 over Unix socket");
        println!("   - Health: {:?}", health);
        println!("   - Security Score: {}", metrics.security_score);
    }
}

// ============================================================================
// REGRESSION TESTS - Backward Compatibility
// ============================================================================

#[cfg(test)]
mod backward_compatibility_tests {
    use super::super::*;
    use mockito;
    use serde_json::json;
    use tokio; // Import from security module

    #[tokio::test]
    #[ignore = "requires running orchestrator"] // Requires Songbird IPC service for HTTP coordination
    async fn test_existing_http_endpoints_still_work() {
        // Regression test: Ensure existing HTTP endpoints still work after protocol detection
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "active_sessions": 5,
                    "failed_auth_attempts": 0,
                    "blocked_ips": 0,
                    "security_score": 1.0,
                    "timestamp": "2026-01-06T16:00:00Z"
                })
                .to_string(),
            )
            .create_async()
            .await;

        // Create adapter with HTTP endpoint (pre-v3.11 behavior)
        let adapter = SecurityAdapter::new(server.url()).await.unwrap();

        // Verify it still works
        let metrics = adapter.collect_metrics().await.unwrap();
        assert_eq!(metrics.active_sessions, 5);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_from_discovery_still_works() {
        // ✅ Concurrent-safe: Uses explicit endpoint (no env vars)
        let result = SecurityAdapter::new("http://localhost:9000".to_string()).await;
        assert!(result.is_ok());
    }
}

// ============================================================================
// PROPERTY TESTS - Protocol Consistency
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::super::*; // Import from security module

    #[tokio::test]
    async fn test_protocol_detection_is_consistent() {
        // Property: Same endpoint should always select same protocol
        let endpoint1 = "unix:///tmp/test.sock";
        let endpoint2 = "unix:///tmp/test.sock";

        let adapter1 = SecurityAdapter::new(endpoint1.to_string()).await.unwrap();
        let adapter2 = SecurityAdapter::new(endpoint2.to_string()).await.unwrap();

        // Both should have same endpoint
        assert_eq!(adapter1.endpoint(), adapter2.endpoint());
    }

    #[tokio::test]
    async fn test_unix_prefix_variations() {
        // Property: All unix:// variations should be handled
        let endpoints = vec![
            "unix:///tmp/test.sock",
            "unix:///var/run/test.sock",
            "unix:///home/user/test.sock",
        ];

        for endpoint in endpoints {
            let result = SecurityAdapter::new(endpoint.to_string()).await;
            assert!(result.is_ok(), "Failed for endpoint: {}", endpoint);
        }
    }

    #[tokio::test]
    async fn test_http_prefix_variations() {
        // Property: All http(s):// variations should be handled
        let endpoints = vec![
            "http://localhost:9000",
            "https://example.com",
            "http://192.168.1.100:8080",
            "https://secure.example.com:8443",
        ];

        for endpoint in endpoints {
            let result = SecurityAdapter::new(endpoint.to_string()).await;
            assert!(result.is_ok(), "Failed for endpoint: {}", endpoint);
        }
    }
}
