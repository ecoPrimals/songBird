// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

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

#[tokio::test]
async fn test_strategy_with_two_endpoints() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should alternate between the two
    let r1 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r2 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r3 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    assert_eq!(r1, endpoints[0]);
    assert_eq!(r2, endpoints[1]);
    assert_eq!(r3, endpoints[0]); // Back to first
    Ok(())
}

#[tokio::test]
async fn test_random_strategy_distribution() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    use songbird_types::SongbirdError;
    use std::collections::HashSet;
    let mut seen = HashSet::new();

    // With random selection, we should eventually see all endpoints
    for _ in 0..100 {
        let result = lb.get_next_endpoint().await.map_err(|_e| {
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
    let endpoints: Vec<String> = (0..3).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    // First call should work (picks least loaded, which is all of them initially)
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_health_based_with_all_healthy() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // All healthy - should return one of them
    for _ in 0..20 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
    Ok(())
}
