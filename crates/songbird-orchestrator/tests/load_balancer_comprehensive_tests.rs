// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive load balancer tests
//!
//! Tests load balancing algorithms, failover, health-based routing, and edge cases

#![cfg(test)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_round_robin_basic() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should cycle through endpoints in order
    let first = lb.get_next_endpoint().await.expect("should find expected value");
    let second = lb.get_next_endpoint().await.expect("should find expected value");
    let third = lb.get_next_endpoint().await.expect("should find expected value");
    let fourth = lb.get_next_endpoint().await.expect("should find expected value");

    assert_eq!(first, endpoints[0]);
    assert_eq!(second, endpoints[1]);
    assert_eq!(third, endpoints[2]);
    assert_eq!(fourth, endpoints[0]); // Wraps around
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_round_robin_single_endpoint() {
    let endpoints = vec!["http://service-1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should always return the same endpoint
    for _ in 0..5 {
        let endpoint = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(endpoint, endpoints[0]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_empty_endpoints() {
    let endpoints: Vec<String> = vec![];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No endpoints configured"), "unexpected: {err_msg}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_all_endpoints_unavailable() {
    let endpoints = vec!["http://service-1:8080".to_string(), "http://service-2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Mark all endpoints as unavailable
    lb.mark_endpoint_unavailable("http://service-1:8080").await;
    lb.mark_endpoint_unavailable("http://service-2:8080").await;

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No available endpoints"), "unexpected: {err_msg}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_endpoint_failover() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark first endpoint as unavailable
    lb.mark_endpoint_unavailable(&endpoints[0]).await;

    // Should skip unavailable endpoint
    let first = lb.get_next_endpoint().await.expect("should find expected value");
    let second = lb.get_next_endpoint().await.expect("should find expected value");
    let third = lb.get_next_endpoint().await.expect("should find expected value");

    // Should only cycle through available endpoints
    assert_ne!(first, endpoints[0]);
    assert_ne!(second, endpoints[0]);
    assert_ne!(third, endpoints[0]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_endpoint_recovery() {
    let endpoints = vec!["http://service-1:8080".to_string(), "http://service-2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark endpoint unavailable, then restore it
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    lb.mark_endpoint_available(&endpoints[0]).await;

    // Endpoint should be available again
    let result = lb.get_next_endpoint().await.expect("should find expected value");
    // Should successfully return an endpoint (either one is fine)
    assert!(result == endpoints[0] || result == endpoints[1]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_based_strategy() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Update health scores
    lb.update_endpoint_health(&endpoints[0], 0.5).await;
    lb.update_endpoint_health(&endpoints[1], 0.9).await;
    lb.update_endpoint_health(&endpoints[2], 0.3).await;

    // Should consistently select healthiest endpoint
    for _ in 0..5 {
        let endpoint = lb.get_next_endpoint().await.expect("should find expected value");
        assert_eq!(endpoint, endpoints[1]); // service-2 has best health (0.9)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_least_loaded_strategy() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    // LeastLoaded strategy should return valid endpoints
    // (actual load tracking would require internal endpoint state manipulation)
    for _ in 0..5 {
        let endpoint = lb.get_next_endpoint().await.expect("should find expected value");
        assert!(endpoints.contains(&endpoint));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_random_strategy() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    // Get multiple endpoints and verify they're all valid
    let mut selected = vec![];
    for _ in 0..10 {
        let endpoint = lb.get_next_endpoint().await.expect("should find expected value");
        assert!(endpoints.contains(&endpoint));
        selected.push(endpoint);
    }

    // With random selection over 10 calls, we should see some variation
    // (statistically very likely, though not guaranteed)
    let unique_selections: std::collections::HashSet<_> = selected.into_iter().collect();
    assert!(unique_selections.len() > 1, "Random strategy should vary selections");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_endpoints() {
    let endpoints = vec!["http://service-1:8080".to_string(), "http://service-2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Get all endpoints
    let all_endpoints = lb.get_endpoints().await;

    // Should have both endpoints
    assert_eq!(all_endpoints.len(), 2);
    assert_eq!(all_endpoints[0].url, endpoints[0]);
    assert_eq!(all_endpoints[1].url, endpoints[1]);

    // All should start as available
    assert!(all_endpoints[0].available);
    assert!(all_endpoints[1].available);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_strategies() {
    let endpoints = vec!["http://service-1:8080".to_string(), "http://service-2:8080".to_string()];

    // Test that each strategy returns valid endpoints
    for strategy in [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastLoaded,
        LoadBalancingStrategy::HealthBased,
        LoadBalancingStrategy::Random,
    ] {
        let lb = LoadBalancer::new(endpoints.clone(), strategy);
        let endpoint = lb.get_next_endpoint().await.expect("should find expected value");
        assert!(endpoints.contains(&endpoint));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_access() {
    let endpoints = vec![
        "http://service-1:8080".to_string(),
        "http://service-2:8080".to_string(),
        "http://service-3:8080".to_string(),
    ];

    let lb = std::sync::Arc::new(LoadBalancer::new(
        endpoints.clone(),
        LoadBalancingStrategy::RoundRobin,
    ));

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let lb_clone = lb.clone();
        let handle = tokio::spawn(async move { lb_clone.get_next_endpoint().await });
        handles.push(handle);
    }

    // All tasks should complete successfully
    for handle in handles {
        let result = handle.await.expect("test precondition");
        assert!(result.is_ok());
        assert!(endpoints.contains(&result.expect("test precondition")));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_health_score_clamping() {
    let endpoints = vec!["http://service-1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Test that health scores are clamped to [0.0, 1.0]
    lb.update_endpoint_health(&endpoints[0], 1.5).await; // Over 1.0
    let all_endpoints = lb.get_endpoints().await;
    assert!(all_endpoints[0].health_score <= 1.0);

    lb.update_endpoint_health(&endpoints[0], -0.5).await; // Under 0.0
    let all_endpoints = lb.get_endpoints().await;
    assert!(all_endpoints[0].health_score >= 0.0);
}
