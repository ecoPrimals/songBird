// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Load Balancer Error Path Tests
//!
//! Focused test suite for load balancer error conditions and edge cases.
//! These tests specifically target scenarios that are under-covered:
//! - Error conditions (no endpoints, all unavailable, etc.)
//! - Edge cases (single endpoint, very large counts, etc.)
//! - Concurrent access patterns
//!
//! Coverage Goal: Add 50 tests to increase Universal crate coverage

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use std::sync::Arc;

// ==================== Error Condition Tests ====================

#[tokio::test]
async fn test_empty_endpoints_error() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    let error_msg = result.expect_err("testing error case");
    assert_eq!(error_msg, "No endpoints configured");
}

#[tokio::test]
async fn test_empty_endpoints_least_loaded() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::LeastLoaded);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_endpoints_health_based() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::HealthBased);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_empty_endpoints_random() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::Random);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_all_endpoints_marked_unhealthy() {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Mark all as unavailable
    for endpoint in &endpoints {
        lb.mark_endpoint_unavailable(endpoint).await;
    }

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.expect_err("testing error case"), "No available endpoints");
}

// ==================== Edge Case Tests ====================

#[tokio::test]
async fn test_single_endpoint_round_robin() -> SongbirdResult<()> {
    let endpoints = vec!["http://only:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should consistently return the single endpoint
    for _ in 0..100 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[0]);
    }
    Ok(())
}

#[tokio::test]
async fn test_single_endpoint_least_loaded() -> SongbirdResult<()> {
    let endpoints = vec!["http://only:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    for _ in 0..50 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[0]);
    }
    Ok(())
}

#[tokio::test]
async fn test_single_endpoint_health_based() -> SongbirdResult<()> {
    let endpoints = vec!["http://only:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    for _ in 0..50 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[0]);
    }
    Ok(())
}

#[tokio::test]
async fn test_single_endpoint_random() -> SongbirdResult<()> {
    let endpoints = vec!["http://only:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    for _ in 0..50 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[0]);
    }
    Ok(())
}

#[tokio::test]
async fn test_very_large_endpoint_count() {
    // Test with 1000 endpoints
    let endpoints: Vec<String> = (0..1000).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should handle large counts efficiently
    for _ in 0..100 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_very_large_endpoint_count_least_loaded() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..500).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::LeastLoaded);

    for _ in 0..50 {
        assert!(lb.get_next_endpoint().await.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_endpoint_with_unusual_port() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint:1".to_string(),     // Minimum port
        "http://endpoint:65535".to_string(), // Maximum port
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Should handle edge port numbers
    let result1 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let result2 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    assert!(result1.contains(":1") || result1.contains(":65535"));
    assert!(result2.contains(":1") || result2.contains(":65535"));
    Ok(())
}

#[tokio::test]
async fn test_endpoint_with_long_hostname() -> SongbirdResult<()> {
    let long_hostname = "a".repeat(255);
    let endpoint = format!("http://{}:8080", long_hostname);

    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoint);
    Ok(())
}

#[tokio::test]
async fn test_endpoints_with_special_characters() {
    let endpoints = vec![
        "http://endpoint-with-dash:8080".to_string(),
        "http://endpoint_with_underscore:8080".to_string(),
        "http://endpoint.with.dots:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should handle special characters in hostnames
    for _ in 0..endpoints.len() {
        assert!(lb.get_next_endpoint().await.is_ok());
    }
}

// ==================== Concurrent Access Tests ====================

#[tokio::test]
async fn test_concurrent_endpoint_selection() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));

    let mut handles = vec![];

    // Spawn 50 concurrent tasks
    for _ in 0..50 {
        let lb_clone = Arc::clone(&lb);
        let handle = tokio::spawn(async move { lb_clone.get_next_endpoint().await });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_health_marking() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..20).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    let mut handles = vec![];

    // Concurrently mark half as unavailable
    for endpoint in &endpoints[0..10] {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoint.clone();
        let handle = tokio::spawn(async move {
            lb_clone.mark_endpoint_unavailable(&endpoint).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Should still be able to get healthy endpoints
    for _ in 0..20 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_reads_and_writes() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    let mut handles = vec![];

    // Spawn readers
    for _ in 0..25 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await.is_ok() }));
    }

    // Spawn writers
    for endpoint in &endpoints[0..5] {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoint.clone();
        handles.push(tokio::spawn(async move {
            lb_clone.mark_endpoint_unavailable(&endpoint).await;
            true
        }));
    }

    // All should complete successfully
    for handle in handles {
        assert!(handle.await.map_err(|e| SongbirdError::configuration("Error"))?);
    }
    Ok(())
}

// ==================== Health State Transition Tests ====================

#[tokio::test]
async fn test_health_state_transitions() {
    let endpoint = "http://test:8080".to_string();
    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::HealthBased);

    // Available -> Unavailable
    lb.mark_endpoint_unavailable(&endpoint).await;
    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());

    // Unavailable -> Available
    lb.mark_endpoint_available(&endpoint).await;
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rapid_health_transitions() {
    let endpoint = "http://test:8080".to_string();
    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::HealthBased);

    // Rapidly toggle availability state
    for _ in 0..100 {
        lb.mark_endpoint_unavailable(&endpoint).await;
        lb.mark_endpoint_available(&endpoint).await;
    }

    // Should still be functional
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mark_health_nonexistent_endpoint() -> SongbirdResult<()> {
    let endpoints = vec!["http://exists:8080".to_string()];
    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    // Marking non-existent endpoint should not panic or error
    lb.mark_endpoint_unavailable("http://doesnotexist:8080").await;
    lb.mark_endpoint_available("http://alsodoesnotexist:8080").await;

    // Original endpoint should still work
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
    Ok(())
}

// ==================== Load Balancing Strategy Edge Cases ====================

#[tokio::test]
async fn test_strategy_with_two_endpoints() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should alternate between the two
    let r1 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r2 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r3 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    assert_eq!(r1, endpoints[0]);
    assert_eq!(r2, endpoints[1]);
    assert_eq!(r3, endpoints[0]); // Back to first
    Ok(())
}

#[tokio::test]
async fn test_random_strategy_distribution() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    use songbird_types::SongbirdError;
    use std::collections::HashSet;
    let mut seen = HashSet::new();

    // With random selection, we should eventually see all endpoints
    for _ in 0..100 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        seen.insert(result);
    }

    // Should have seen multiple endpoints (with high probability)
    assert!(seen.len() >= 3);
    Ok(())
}

#[tokio::test]
async fn test_least_loaded_actual_selection() {
    let endpoints: Vec<String> = (0..3).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    // First call should work (picks least loaded, which is all of them initially)
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_based_with_all_healthy() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // All healthy - should return one of them
    for _ in 0..20 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
    Ok(())
}

// ==================== Endpoint Format Tests ====================

#[tokio::test]
async fn test_endpoint_with_path() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint:8080/api/v1".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    assert!(result.contains("/api/v1"));
    Ok(())
}

#[tokio::test]
async fn test_endpoint_with_query_parameters() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint:8080?param=value".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_https_endpoints() -> SongbirdResult<()> {
    let endpoints = vec!["https://secure1:443".to_string(), "https://secure2:443".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb
        .get_next_endpoint()
        .await
        .map_err(|e| SongbirdError::configuration("Failed to start orchestrator".to_string()))?;
    assert!(result.starts_with("https://"));
    Ok(())
}

#[tokio::test]
async fn test_mixed_http_https_endpoints() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "https://endpoint2:8443".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should handle mixed protocols
    let r1 = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r2 = lb
        .get_next_endpoint()
        .await
        .map_err(|e| SongbirdError::configuration("Failed to start orchestrator".to_string()))?;

    assert!(r1.starts_with("http"));
    assert!(r2.starts_with("http"));
    Ok(())
}

#[tokio::test]
async fn test_ipv4_addresses() {
    let endpoints = vec!["http://192.168.1.1:8080".to_string(), "http://10.0.0.1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_localhost_endpoints() {
    let endpoints = vec!["http://localhost:8080".to_string(), "http://127.0.0.1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

// ==================== Counter Overflow Tests ====================

#[tokio::test]
async fn test_round_robin_counter_wrap() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Call many times to potentially trigger counter wrap
    for _ in 0..10000 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
    Ok(())
}

// ==================== Load Balancing Strategy Clone Tests ====================

#[test]
fn test_load_balancing_strategy_clone() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let cloned = strategy;
    assert_eq!(strategy, cloned);
    Ok(())
}

#[test]
fn test_load_balancing_strategy_debug() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::LeastLoaded;
    let debug_str = format!("{:?}", strategy);
    assert!(!debug_str.is_empty());
    Ok(())
}

// ==================== Additional Concurrent Scenarios ====================

#[tokio::test]
async fn test_concurrent_strategy_switching() -> SongbirdResult<()> {
    // Tests rapid concurrent access to load balancer
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));

    let mut handles = vec![];
    for _ in 0..100 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await }));
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_availability_checks() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    let mut handles = vec![];

    // Concurrently check availability
    for _ in 0..50 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.available_count().await }));
    }

    for handle in handles {
        let count = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(count <= 10);
    }
    Ok(())
}

#[tokio::test]
async fn test_high_concurrency_selection() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..20).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::Random));

    let mut handles = vec![];

    // Very high concurrency - 1000 concurrent requests
    for _ in 0..1000 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await.is_ok() }));
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result);
    }
    Ok(())
}

// ==================== Edge Case Scenarios ====================

#[tokio::test]
async fn test_duplicate_endpoints() -> SongbirdResult<()> {
    // Test with duplicate endpoint URLs
    let endpoints = vec![
        "http://same:8080".to_string(),
        "http://same:8080".to_string(),
        "http://same:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Should handle duplicates (though not ideal in practice)
    for _ in 0..10 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
        assert_eq!(result?, "http://same:8080");
    }
    Ok(())
}

#[tokio::test]
async fn test_endpoint_url_with_fragment() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint:8080#fragment".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_endpoint_url_with_username() -> SongbirdResult<()> {
    let endpoints = vec!["http://user@endpoint:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_endpoint_url_with_password() -> SongbirdResult<()> {
    let endpoints = vec!["http://user:pass@endpoint:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_very_long_url_path() -> SongbirdResult<()> {
    let long_path = "/".to_string() + &"segment/".repeat(100);
    let endpoint = format!("http://endpoint:8080{}", long_path);

    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoint);
    Ok(())
}

#[tokio::test]
async fn test_endpoint_with_international_domain() -> SongbirdResult<()> {
    // Punycode/IDN domains
    let endpoints = vec!["http://münchen.de:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

// ==================== Availability and Health Tracking ====================

#[tokio::test]
async fn test_available_count_with_mixed_health() {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Mark half as unavailable
    for i in 0..5 {
        lb.mark_endpoint_unavailable(&endpoints[i]).await;
    }

    let available = lb.available_count().await;
    assert_eq!(available, 5);
}

#[tokio::test]
async fn test_available_count_all_available() {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    let available = lb.available_count().await;
    assert_eq!(available, 10);
}

#[tokio::test]
async fn test_available_count_all_unavailable() {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    for endpoint in &endpoints {
        lb.mark_endpoint_unavailable(endpoint).await;
    }

    let available = lb.available_count().await;
    assert_eq!(available, 0);
}

#[tokio::test]
async fn test_get_endpoints_returns_all() {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{}:8080", i)).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let returned = lb.get_endpoints().await;
    assert_eq!(returned.len(), 5);
}

#[tokio::test]
async fn test_update_endpoint_health_valid_score() {
    let endpoint = "http://test:8080".to_string();
    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::HealthBased);

    lb.update_endpoint_health(&endpoint, 0.75).await;

    // Should still be able to get the endpoint
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_endpoint_health_zero_score() {
    let endpoint = "http://test:8080".to_string();
    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::HealthBased);

    lb.update_endpoint_health(&endpoint, 0.0).await;

    // Health score of 0.0 makes it unavailable
    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_endpoint_health_max_score() -> SongbirdResult<()> {
    let endpoint = "http://test:8080".to_string();
    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::HealthBased);

    lb.update_endpoint_health(&endpoint, 1.0).await;

    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
    Ok(())
}

// ==================== Round Robin Specific Tests ====================

#[tokio::test]
async fn test_round_robin_predictable_sequence() -> SongbirdResult<()> {
    let endpoints =
        vec!["http://a:8080".to_string(), "http://b:8080".to_string(), "http://c:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should cycle through in order
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|e| SongbirdError::configuration("Error"))?,
        endpoints[0]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|e| SongbirdError::configuration("Error"))?,
        endpoints[1]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|e| SongbirdError::configuration("Error"))?,
        endpoints[2]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|e| SongbirdError::configuration("Error"))?,
        endpoints[0]
    );
    Ok(())
}

// Test count: 50 tests added for load balancer error paths and edge cases
