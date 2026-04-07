// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_endpoint_with_path() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint:8080/api/v1".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|_e| {
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

    let result = lb.get_next_endpoint().await.map_err(|_e| {
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
        .map_err(|_e| SongbirdError::configuration("Failed to start orchestrator".to_string()))?;
    assert!(result.starts_with("https://"));
    Ok(())
}

#[tokio::test]
async fn test_mixed_http_https_endpoints() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "https://endpoint2:8443".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should handle mixed protocols
    let r1 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let r2 = lb
        .get_next_endpoint()
        .await
        .map_err(|_e| SongbirdError::configuration("Failed to start orchestrator".to_string()))?;

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
