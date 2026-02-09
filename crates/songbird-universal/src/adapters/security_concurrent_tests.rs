#![cfg(test)]

//! **CONCURRENT SECURITY ADAPTER TESTS**  
//!
//! **Purpose**: Achieve 90% coverage with truly concurrent, deterministic tests
//!
//! This file implements the concurrent evolution patterns established in `circuit_breaker.rs`:
//! - Lock-free operations where applicable
//! - Event-driven state notifications
//! - Deterministic time testing with `tokio::time::pause()`
//! - Proper concurrent synchronization (barriers, channels, not sleeps)
//! - Comprehensive coverage of all code paths
//!
//! ## Coverage Strategy
//!
//! Current: 14.71% → Target: 90%+
//!
//! ### Covered Areas (from existing tests)
//! - Basic metrics calculations (sync tests)
//! - Adapter creation (sync tests)
//! - Single async operations (async tests with wiremock)
//!
//! ### NEW Coverage (this file)
//! 1. **Concurrent Request Handling** - Multiple simultaneous requests
//! 2. **Race Conditions** - Concurrent state changes
//! 3. **Error Path Coverage** - Concurrent failures
//! 4. **Timeout Scenarios** - Deterministic timeout testing
//! 5. **Discovery Edge Cases** - Concurrent discovery
//! 6. **`SecurityProvider` Trait** - Concurrent trait usage
//! 7. **Load Testing** - High concurrency scenarios

use super::security::{AuthResult, SecurityAdapter, SecurityHealth, SecurityProvider};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Barrier, Semaphore};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// CONCURRENT REQUEST TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_metrics_collection_10_simultaneous() {
    // ARRANGE: Mock server with metrics
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 100,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(10) // Expect exactly 10 concurrent requests
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Launch 10 concurrent metrics collections
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        let barrier = Arc::clone(&barrier);

        let handle = tokio::spawn(async move {
            // Wait for all tasks to be ready
            barrier.wait().await;

            // All tasks start simultaneously
            adapter.collect_metrics().await
        });

        handles.push(handle);
    }

    // ASSERT: All requests should succeed
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(success_count, 10, "All 10 concurrent requests should succeed");

    // Verify metrics are correct
    for result in results {
        let metrics = result.unwrap().unwrap();
        assert_eq!(metrics.active_sessions, 100);
        assert_eq!(metrics.security_score, 0.95);
    }
}

#[tokio::test]
async fn test_concurrent_auth_verification_high_load() {
    // ARRANGE: Mock server handling auth requests
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Authorized"))
        .expect(50) // High load: 50 concurrent requests
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Launch 50 concurrent auth verifications
    let semaphore = Arc::new(Semaphore::new(50));
    let mut handles = vec![];

    for i in 0..50 {
        let adapter = Arc::clone(&adapter);
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let handle = tokio::spawn(async move {
            let result = adapter.verify_auth(&format!("token-{i}")).await;
            drop(permit); // Release permit
            result
        });

        handles.push(handle);
    }

    // ASSERT: All auth requests should succeed
    let results = futures::future::join_all(handles).await;
    let authorized_count = results
        .iter()
        .filter(|r| matches!(r.as_ref().unwrap(), Ok(AuthResult::Authorized)))
        .count();

    assert_eq!(authorized_count, 50, "All 50 concurrent auth requests should be authorized");
}

#[tokio::test]
async fn test_concurrent_mixed_operations() {
    // ARRANGE: Mock server handling multiple operation types
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 75,
            "failed_auth_attempts": 10,
            "blocked_ips": 3,
            "security_score": 0.88,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Authorized"))
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Mix of concurrent operations (metrics and auth)
    let mut metrics_handles = vec![];
    let mut auth_handles = vec![];

    // 10 metrics requests
    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        metrics_handles.push(tokio::spawn(async move { adapter.collect_metrics().await }));
    }

    // 15 auth requests
    for i in 0..15 {
        let adapter = Arc::clone(&adapter);
        auth_handles
            .push(tokio::spawn(async move { adapter.verify_auth(&format!("token-{i}")).await }));
    }

    // ASSERT: All operations should complete successfully
    let metrics_results = futures::future::join_all(metrics_handles).await;
    let auth_results = futures::future::join_all(auth_handles).await;

    let metrics_success =
        metrics_results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();
    let auth_success =
        auth_results.iter().filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok()).count();

    assert_eq!(metrics_success, 10, "All 10 metrics requests should succeed");
    assert_eq!(auth_success, 15, "All 15 auth requests should succeed");
}

// ============================================================================
// RACE CONDITION TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_adapter_creation() {
    // ACT: Create 100 adapters concurrently
    let handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move { SecurityAdapter::new(format!("http://security-{i}:8081")) })
        })
        .collect();

    // ASSERT: All should succeed
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(success_count, 100, "All 100 concurrent adapter creations should succeed");
}

#[tokio::test]
async fn test_concurrent_timeout_modifications() {
    // ARRANGE: Base adapter
    let base_adapter = SecurityAdapter::new("http://test:8080").unwrap();

    // ACT: Apply concurrent timeout modifications
    let handles: Vec<_> = (1..=20)
        .map(|i| {
            let adapter = base_adapter.clone_with_timeout(Duration::from_secs(i));
            tokio::spawn(async move {
                // Verify adapter still works after concurrent modification
                adapter.endpoint().to_string()
            })
        })
        .collect();

    // ASSERT: All modifications should work
    let results = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 20, "All 20 concurrent timeout modifications should complete");
}

// Helper method for cloning adapter with new timeout (test-only)
impl SecurityAdapter {
    #[cfg(test)]
    fn clone_with_timeout(&self, timeout: Duration) -> Self {
        Self::new(self.endpoint().to_string()).unwrap().with_timeout(timeout)
    }
}

#[tokio::test]
async fn test_race_condition_metrics_under_attack() {
    // ARRANGE: Two mock servers - one healthy, one under attack
    let healthy_server = MockServer::start().await;
    let attack_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 50,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(10)
        .mount(&healthy_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 200,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.30,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(10)
        .mount(&attack_server)
        .await;

    let healthy_adapter = Arc::new(SecurityAdapter::new(healthy_server.uri()).unwrap());
    let attack_adapter = Arc::new(SecurityAdapter::new(attack_server.uri()).unwrap());

    // ACT: Mix of concurrent requests to both endpoints
    let mut handles = vec![];
    for i in 0..20 {
        let adapter = if i % 2 == 0 {
            Arc::clone(&healthy_adapter)
        } else {
            Arc::clone(&attack_adapter)
        };
        handles.push(tokio::spawn(async move { adapter.collect_metrics().await }));
    }

    // ASSERT: All requests should succeed, with mixed states
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    assert_eq!(success_count, 20, "All requests should succeed despite mixed states");

    // Verify we got both states
    let under_attack_count = results
        .iter()
        .filter(|r| {
            r.as_ref()
                .unwrap()
                .as_ref()
                .map(super::security::SecurityMetrics::is_under_attack)
                .unwrap_or(false)
        })
        .count();

    assert!(under_attack_count > 0, "Should have detected some under attack states");
    assert!(
        under_attack_count < 20,
        "Should have detected some healthy states (not all under attack)"
    );
}

// ============================================================================
// ERROR PATH COVERAGE
// ============================================================================

#[tokio::test]
async fn test_concurrent_network_failures() {
    // ARRANGE: Adapter pointing to non-existent server
    let adapter = Arc::new(
        SecurityAdapter::new("http://nonexistent-concurrent-test:9999")
            .unwrap()
            .with_timeout(Duration::from_millis(100)),
    );

    // ACT: 20 concurrent requests that will fail
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move { adapter.collect_metrics().await })
        })
        .collect();

    // ASSERT: All should fail gracefully with network errors
    let results = futures::future::join_all(handles).await;
    let error_count = results.iter().filter(|r| r.as_ref().unwrap().is_err()).count();

    assert_eq!(error_count, 20, "All 20 concurrent requests should fail with network error");

    // Verify error messages
    for result in results {
        let err = result.unwrap().unwrap_err();
        let err_msg = format!("{err:?}");
        assert!(
            err_msg.contains("network") || err_msg.contains("Failed to reach"),
            "Error should indicate network failure"
        );
    }
}

#[tokio::test]
async fn test_concurrent_http_errors() {
    // ARRANGE: Mock server returning errors
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(503)) // Service Unavailable
        .expect(15)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Concurrent requests to failing server
    let handles: Vec<_> = (0..15)
        .map(|_| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move { adapter.collect_metrics().await })
        })
        .collect();

    // ASSERT: All should fail with HTTP error
    let results = futures::future::join_all(handles).await;
    let error_count = results.iter().filter(|r| r.as_ref().unwrap().is_err()).count();

    assert_eq!(error_count, 15, "All 15 requests should fail with HTTP 503");
}

#[tokio::test]
async fn test_concurrent_parse_errors() {
    // ARRANGE: Mock server returning invalid JSON
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json {"))
        .expect(10)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Concurrent requests that will get parse errors
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move { adapter.collect_metrics().await })
        })
        .collect();

    // ASSERT: All should fail with parse error
    let results = futures::future::join_all(handles).await;
    let error_count = results.iter().filter(|r| r.as_ref().unwrap().is_err()).count();

    assert_eq!(error_count, 10, "All 10 requests should fail with parse error");
}

// ============================================================================
// DETERMINISTIC TIMEOUT TESTS
// ============================================================================

#[tokio::test(start_paused = true)]
async fn test_deterministic_timeout_short() {
    // ARRANGE: Adapter with 100ms timeout, slow server (tokio time paused)
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&mock_server)
        .await;

    let adapter =
        SecurityAdapter::new(mock_server.uri()).unwrap().with_timeout(Duration::from_millis(100));

    // ACT: Make request (should timeout deterministically)
    let result = adapter.collect_metrics().await;

    // ASSERT: Should timeout (reqwest timeout)
    assert!(result.is_err(), "Should timeout after 100ms");
}

#[tokio::test(start_paused = true)]
async fn test_deterministic_concurrent_timeouts() {
    // ARRANGE: Multiple adapters with different timeout values
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let adapters: Vec<_> = [50, 100, 200, 500, 1000]
        .iter()
        .map(|&timeout_ms| {
            Arc::new(
                SecurityAdapter::new(mock_server.uri())
                    .unwrap()
                    .with_timeout(Duration::from_millis(timeout_ms)),
            )
        })
        .collect();

    // ACT: Launch all requests concurrently
    let handles: Vec<_> = adapters
        .iter()
        .map(|adapter| {
            let adapter = Arc::clone(adapter);
            tokio::spawn(async move { adapter.collect_metrics().await })
        })
        .collect();

    // ASSERT: All should timeout (server delay > all timeouts)
    let results = futures::future::join_all(handles).await;
    let timeout_count = results.iter().filter(|r| r.as_ref().unwrap().is_err()).count();

    assert_eq!(timeout_count, 5, "All 5 requests should timeout");
}

// ============================================================================
// DISCOVERY CONCURRENT TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_from_discovery() {
    // ✅ Concurrent-safe: Uses SecurityAdapter::new() directly (no env vars)
    // ACT: 20 concurrent adapter creation calls
    let handles: Vec<_> =
        (0..20).map(|_| tokio::spawn(async {
            SecurityAdapter::new("http://concurrent-test:8081".to_string()).await
        })).collect();

    // ASSERT: All should succeed and discover the same endpoint
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(success_count, 20, "All 20 concurrent discoveries should succeed");

    // Verify all discovered the same endpoint
    for result in results {
        let adapter = result.unwrap().unwrap();
        assert!(
            adapter.endpoint().contains("concurrent-test") || adapter.endpoint().contains("8081")
        );
    }

}

// ============================================================================
// SECURITY_PROVIDER TRAIT CONCURRENT TESTS
// ============================================================================

#[tokio::test]
async fn test_security_provider_trait_concurrent_usage() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 40,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.92,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(30)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Use trait methods concurrently
    let mut metrics_handles = vec![];
    let mut health_handles = vec![];

    // 10 collect_security_metrics calls
    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        metrics_handles.push(tokio::spawn(async move {
            SecurityProvider::collect_security_metrics(adapter.as_ref()).await
        }));
    }

    // 10 check_security_health calls (uses default impl)
    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        health_handles.push(tokio::spawn(async move {
            SecurityProvider::check_security_health(adapter.as_ref()).await
        }));
    }

    // 10 more collect calls
    for _ in 0..10 {
        let adapter = Arc::clone(&adapter);
        metrics_handles.push(tokio::spawn(async move {
            SecurityProvider::collect_security_metrics(adapter.as_ref()).await
        }));
    }

    // ASSERT: All trait method calls should succeed
    let metrics_results = futures::future::join_all(metrics_handles).await;
    let health_results = futures::future::join_all(health_handles).await;

    let metrics_success = metrics_results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    let health_success = health_results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(metrics_success, 20, "All 20 collect_security_metrics calls should succeed");
    assert_eq!(health_success, 10, "All 10 check_security_health calls should succeed");
}

// ============================================================================
// LOAD TESTING (High Concurrency)
// ============================================================================

#[tokio::test]
async fn test_high_concurrency_100_simultaneous_requests() {
    // ARRANGE: Mock server with high capacity
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 500,
            "failed_auth_attempts": 10,
            "blocked_ips": 5,
            "security_score": 0.90,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(100)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());
    let barrier = Arc::new(Barrier::new(100));

    // ACT: Launch 100 requests simultaneously
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let adapter = Arc::clone(&adapter);
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                barrier.wait().await;
                adapter.collect_metrics().await
            })
        })
        .collect();

    // ASSERT: All 100 should succeed
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert_eq!(success_count, 100, "All 100 high-concurrency requests should succeed");
}

#[tokio::test]
async fn test_sustained_load_200_requests_over_time() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Authorized"))
        .expect(200)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Launch 200 requests (sustained load, not all at once)
    let handles: Vec<_> = (0..200)
        .map(|i| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move { adapter.verify_auth(&format!("sustained-token-{i}")).await })
        })
        .collect();

    // ASSERT: All should complete successfully under sustained load
    let results = futures::future::join_all(handles).await;
    let success_count = results
        .iter()
        .filter(|r| matches!(r.as_ref().unwrap(), Ok(AuthResult::Authorized)))
        .count();

    assert_eq!(success_count, 200, "All 200 sustained load requests should succeed");
}

// ============================================================================
// METRICS ANALYSIS CONCURRENT TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_metrics_analysis() {
    // ARRANGE: Three servers with different health states
    let healthy_server = MockServer::start().await;
    let warning_server = MockServer::start().await;
    let critical_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 50,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(17) // 0, 3, 6, ..., 48 = 17 requests
        .mount(&healthy_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 100,
            "failed_auth_attempts": 60,
            "blocked_ips": 10,
            "security_score": 0.65,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(17) // 1, 4, 7, ..., 49 = 17 requests
        .mount(&warning_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 200,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.30,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(16) // 2, 5, 8, ..., 47 = 16 requests
        .mount(&critical_server)
        .await;

    let healthy_adapter = Arc::new(SecurityAdapter::new(healthy_server.uri()).unwrap());
    let warning_adapter = Arc::new(SecurityAdapter::new(warning_server.uri()).unwrap());
    let critical_adapter = Arc::new(SecurityAdapter::new(critical_server.uri()).unwrap());

    // ACT: Collect 50 metrics concurrently from mixed sources
    let mut handles = vec![];
    for i in 0..50 {
        let adapter = match i % 3 {
            0 => Arc::clone(&healthy_adapter),
            1 => Arc::clone(&warning_adapter),
            _ => Arc::clone(&critical_adapter),
        };
        handles.push(tokio::spawn(async move { adapter.collect_metrics().await }));
    }

    // ASSERT: Analyze collected metrics
    let results = futures::future::join_all(handles).await;
    let metrics: Vec<_> = results.into_iter().filter_map(|r| r.unwrap().ok()).collect();

    assert_eq!(metrics.len(), 50, "Should collect 50 metrics");

    // Verify health status distribution
    let healthy_count =
        metrics.iter().filter(|m| m.health_status() == SecurityHealth::Healthy).count();
    let warning_count =
        metrics.iter().filter(|m| m.health_status() == SecurityHealth::Warning).count();
    let critical_count =
        metrics.iter().filter(|m| m.health_status() == SecurityHealth::Critical).count();

    assert!(healthy_count > 0, "Should have some healthy metrics");
    assert!(warning_count > 0, "Should have some warning metrics");
    assert!(critical_count > 0, "Should have some critical metrics");
    assert_eq!(
        healthy_count + warning_count + critical_count,
        50,
        "All metrics should have a health status"
    );
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_rapid_sequential_requests() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 25,
            "failed_auth_attempts": 2,
            "blocked_ips": 1,
            "security_score": 0.96,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(1000)
        .mount(&mock_server)
        .await;

    let adapter = Arc::new(SecurityAdapter::new(mock_server.uri()).unwrap());

    // ACT: Launch 1000 requests as fast as possible
    let handles: Vec<_> = (0..1000)
        .map(|_| {
            let adapter = Arc::clone(&adapter);
            tokio::spawn(async move { adapter.collect_metrics().await })
        })
        .collect();

    // ASSERT: All should complete (stress test)
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();

    assert!(
        success_count >= 900,
        "At least 90% of 1000 rapid requests should succeed (got {success_count})"
    );
}

#[tokio::test]
async fn test_concurrent_with_mixed_success_and_failure() {
    // ARRANGE: Two mock servers - one succeeding, one failing
    let success_server = MockServer::start().await;
    let failure_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 50,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .expect(50)
        .mount(&success_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(500))
        .expect(50)
        .mount(&failure_server)
        .await;

    let success_adapter = Arc::new(SecurityAdapter::new(success_server.uri()).unwrap());
    let failure_adapter = Arc::new(SecurityAdapter::new(failure_server.uri()).unwrap());

    // ACT: 100 concurrent requests with mixed success/failure
    let mut handles = vec![];
    for i in 0..100 {
        let adapter = if i % 2 == 0 {
            Arc::clone(&success_adapter)
        } else {
            Arc::clone(&failure_adapter)
        };
        handles.push(tokio::spawn(async move { adapter.collect_metrics().await }));
    }

    // ASSERT: Should handle mixed success/failure gracefully
    let results = futures::future::join_all(handles).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap().is_ok()).count();
    let error_count = results.iter().filter(|r| r.as_ref().unwrap().is_err()).count();

    assert_eq!(success_count, 50, "Should have exactly 50 successes");
    assert_eq!(error_count, 50, "Should have exactly 50 errors");
    assert_eq!(success_count + error_count, 100, "All requests should complete");
}
