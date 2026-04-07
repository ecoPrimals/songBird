// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_single_endpoint_round_robin() -> SongbirdResult<()> {
    let endpoints = vec!["http://only:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should consistently return the single endpoint
    for _ in 0..100 {
        let result = lb.get_next_endpoint().await.map_err(|_e| {
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
        let result = lb.get_next_endpoint().await.map_err(|_e| {
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
        let result = lb.get_next_endpoint().await.map_err(|_e| {
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
        let result = lb.get_next_endpoint().await.map_err(|_e| {
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
    let result1 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let result2 = lb.get_next_endpoint().await.map_err(|_e| {
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

    let result = lb.get_next_endpoint().await.map_err(|_e| {
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
