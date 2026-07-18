// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_available_count_with_mixed_health() {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Mark half as unavailable
    for ep in endpoints.iter().take(5) {
        lb.mark_endpoint_unavailable(ep).await;
    }

    let available = lb.available_count().await;
    assert_eq!(available, 5);
}

#[tokio::test]
async fn test_available_count_all_available() {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    let available = lb.available_count().await;
    assert_eq!(available, 10);
}

#[tokio::test]
async fn test_available_count_all_unavailable() {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    for endpoint in &endpoints {
        lb.mark_endpoint_unavailable(endpoint).await;
    }

    let available = lb.available_count().await;
    assert_eq!(available, 0);
}

#[tokio::test]
async fn test_get_endpoints_returns_all() {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{i}:8080")).collect();

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
