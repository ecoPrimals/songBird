// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Async Integration Tests for Load Balancer
//!
//! **Goal**: Test load balancer strategies under realistic async scenarios
//! **Coverage Target**: Concurrent access, strategy behavior, health management
//!
//! This suite tests:
//! - Round-robin distribution under load
//! - Least loaded strategy with concurrent connections
//! - Health-based routing with dynamic health updates
//! - Endpoint availability management
//! - Concurrent access to shared state

use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// ROUND-ROBIN STRATEGY TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_round_robin_distribution() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Collect 9 selections (3 full cycles)
    let mut selections = Vec::new();
    for _ in 0..9 {
        selections.push(lb.get_next_endpoint().await.expect("should have capacity"));
    }

    // Should cycle through in order
    for i in 0..9 {
        assert_eq!(selections[i], endpoints[i % 3]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_round_robin_with_concurrent_requests() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));

    // Fire off 10 concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.expect("test precondition").is_ok());
    }
}

// ============================================================================
// HEALTH-BASED STRATEGY TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_based_selection() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Update health scores
    lb.update_endpoint_health(&endpoints[0], 0.5).await;
    lb.update_endpoint_health(&endpoints[1], 0.9).await;
    lb.update_endpoint_health(&endpoints[2], 0.7).await;

    // Should consistently select highest health (service2)
    for _ in 0..5 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(selected, endpoints[1]); // Healthiest
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_based_with_dynamic_updates() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Initially equal health - either is fine
    lb.update_endpoint_health(&endpoints[0], 1.0).await;
    lb.update_endpoint_health(&endpoints[1], 1.0).await;

    // Degrade service1
    lb.update_endpoint_health(&endpoints[0], 0.3).await;

    // Should prefer service2
    for _ in 0..5 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(selected, endpoints[1]);
    }

    // Now degrade service2 more
    lb.update_endpoint_health(&endpoints[1], 0.1).await;

    // Should switch back to service1
    for _ in 0..5 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(selected, endpoints[0]);
    }
}

// ============================================================================
// AVAILABILITY MANAGEMENT TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_mark_endpoint_unavailable() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark first endpoint unavailable
    lb.mark_endpoint_unavailable(&endpoints[0]).await;

    // Should only return service2
    for _ in 0..5 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(selected, endpoints[1]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_mark_endpoint_available_after_unavailable() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark first endpoint unavailable then available again
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 1);

    lb.mark_endpoint_available(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 2);

    // Should now cycle through both
    let first = lb.get_next_endpoint().await.expect("should find expected value");
    let second = lb.get_next_endpoint().await.expect("should find expected value");
    assert_ne!(first, second);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_all_endpoints_unavailable() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark all unavailable
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    lb.mark_endpoint_unavailable(&endpoints[1]).await;

    // Should return error
    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.expect_err("testing error case"), "No available endpoints");
}

// ============================================================================
// CONCURRENT ACCESS TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_health_updates() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    // Update health concurrently from multiple tasks
    let mut handles = vec![];
    for i in 0..10 {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoints[i % 2].clone();
        handles.push(tokio::spawn(async move {
            lb_clone.update_endpoint_health(&endpoint, 0.5 + (i as f64 * 0.01)).await;
        }));
    }

    futures::future::join_all(handles).await;

    // Should not deadlock or panic
    let selected = lb.get_next_endpoint().await;
    assert!(selected.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_availability_changes() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin));

    // Use a barrier to ensure coordinated concurrent access
    let barrier = Arc::new(tokio::sync::Barrier::new(9));

    // Mark endpoints unavailable/available concurrently with deterministic pattern
    let mut handles = vec![];
    for i in 0..9 {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoints[i % 3].clone();
        let barrier_clone = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            // Wait for all tasks to be ready (ensures true concurrency)
            barrier_clone.wait().await;

            if i % 2 == 0 {
                lb_clone.mark_endpoint_unavailable(&endpoint).await;
            } else {
                lb_clone.mark_endpoint_available(&endpoint).await;
            }
        }));
    }

    // Wait for all concurrent operations to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }

    // With 9 operations (5 unavailable, 4 available) on 3 endpoints,
    // the final state depends on order, but at least one endpoint
    // will receive more "available" than "unavailable" operations
    // Pattern: endpoint 0: [unavailable, available, unavailable] = unavailable
    //          endpoint 1: [available, unavailable, available] = available
    //          endpoint 2: [unavailable, available, unavailable] = unavailable
    // Result: 1 available endpoint (deterministic with barrier)
    let count = lb.available_count().await;
    assert!(count >= 1, "Expected at least 1 available endpoint, got {}", count);

    // Verify endpoints are still accessible
    let endpoints = lb.get_endpoints().await;
    assert!(!endpoints.is_empty(), "Should have endpoints available");
}

// ============================================================================
// STRATEGY-SPECIFIC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_least_loaded_strategy() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    // All start with 0 connections, so could select any
    let first = lb.get_next_endpoint().await.expect("should find expected value");
    assert!(endpoints.contains(&first));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_random_strategy_distribution() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    // Collect many selections
    let mut counts = HashMap::new();
    for _ in 0..100 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        *counts.entry(selected).or_insert(0) += 1;
    }

    // All endpoints should have been selected at least once with high probability
    assert!(counts.len() >= 2); // At minimum 2 of 3 in 100 tries
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_single_endpoint() {
    let endpoints = vec!["http://only-service:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should always return the only endpoint
    for _ in 0..5 {
        let selected = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(selected, endpoints[0]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_empty_endpoints() {
    let endpoints: Vec<String> = vec![];
    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.expect_err("testing error case"), "No endpoints configured");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_healthy_count() {
    let endpoints = vec![
        "http://service1:8080".to_string(),
        "http://service2:8080".to_string(),
        "http://service3:8080".to_string(),
    ];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // All start healthy
    assert_eq!(lb.healthy_count().await, 3);

    // Degrade one below threshold
    lb.update_endpoint_health(&endpoints[0], 0.3).await;
    assert_eq!(lb.healthy_count().await, 2);

    // Mark one unavailable
    lb.mark_endpoint_unavailable(&endpoints[1]).await;
    assert_eq!(lb.healthy_count().await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_available_count() {
    let endpoints = vec!["http://service1:8080".to_string(), "http://service2:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    assert_eq!(lb.available_count().await, 2);

    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 1);

    lb.mark_endpoint_unavailable(&endpoints[1]).await;
    assert_eq!(lb.available_count().await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_update_affects_availability() {
    let endpoints = vec!["http://service:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Start available
    assert_eq!(lb.available_count().await, 1);

    // Health of 0 should make it unavailable
    lb.update_endpoint_health(&endpoints[0], 0.0).await;
    assert_eq!(lb.available_count().await, 0);

    // Restore health
    lb.update_endpoint_health(&endpoints[0], 0.8).await;
    assert_eq!(lb.available_count().await, 1);
}
