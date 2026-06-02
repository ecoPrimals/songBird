// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_empty_endpoints_error() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    let error_msg = result.expect_err("testing error case");
    assert_eq!(error_msg.to_string(), "No endpoints configured");
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
    assert_eq!(result.expect_err("testing error case").to_string(), "No available endpoints");
}
