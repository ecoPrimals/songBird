// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_concurrent_endpoint_selection() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{i}:8080")).collect();

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
        let result = handle.await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_health_marking() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..20).map(|i| format!("http://endpoint{i}:8080")).collect();

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
        handle.await.map_err(|_e| {
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
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{i}:8080")).collect();

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
        assert!(handle.await.map_err(|_e| SongbirdError::configuration("Error"))?);
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_strategy_switching() -> SongbirdResult<()> {
    // Tests rapid concurrent access to load balancer
    let endpoints: Vec<String> = (0..5).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));

    let mut handles = vec![];
    for _ in 0..100 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await }));
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_availability_checks() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..10).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    let mut handles = vec![];

    // Concurrently check availability
    for _ in 0..50 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.available_count().await }));
    }

    for handle in handles {
        let count = handle.await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(count <= 10);
    }
    Ok(())
}

#[tokio::test]
async fn test_high_concurrency_selection() -> SongbirdResult<()> {
    let endpoints: Vec<String> = (0..20).map(|i| format!("http://endpoint{i}:8080")).collect();

    let lb = Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::Random));

    let mut handles = vec![];

    // Very high concurrency - 1000 concurrent requests
    for _ in 0..1000 {
        let lb_clone = Arc::clone(&lb);
        handles.push(tokio::spawn(async move { lb_clone.get_next_endpoint().await.is_ok() }));
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result);
    }
    Ok(())
}
