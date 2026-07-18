// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_duplicate_endpoints() -> anyhow::Result<()> {
    let endpoints = vec![
        "http://same:8080".to_string(),
        "http://same:8080".to_string(),
        "http://same:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

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

    let result = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_endpoint_url_with_username() -> SongbirdResult<()> {
    let endpoints = vec!["http://user@endpoint:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_endpoint_url_with_password() -> SongbirdResult<()> {
    let endpoints = vec!["http://user:pass@endpoint:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_very_long_url_path() -> SongbirdResult<()> {
    let long_path = "/".to_string() + &"segment/".repeat(100);
    let endpoint = format!("http://endpoint:8080{long_path}");

    let lb = LoadBalancer::new(vec![endpoint.clone()], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await.map_err(|_e| {
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

    let result = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}
