//! Enhanced Load Balancer Tests
//!
//! Comprehensive tests for load balancing strategies, edge cases, and concurrent access.

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use std::sync::Arc;
use tokio::sync::Barrier;

// ==================== Strategy Tests ====================

#[tokio::test]
async fn test_round_robin_cycles_correctly() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Test full cycle
    for _ in 0..3 {
        for expected in &endpoints {
            let result = lb.get_next_endpoint().await.map_err(|e| {
                SongbirdError::configuration(format!(
                    "Error: {}",
                    e
                ))
            })?;
            assert_eq!(&result, expected);
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_round_robin_with_single_endpoint() -> SongbirdResult<()> {
    let endpoints = vec!["http://single:8080".to_string()];
    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should always return the same endpoint
    for _ in 0..10 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[0]);
    }
    Ok(())
}

#[tokio::test]
async fn test_least_loaded_selects_correctly() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::LeastLoaded);

    // Note: Cannot directly set load in tests due to private fields
    // This test verifies the strategy works with default loads (all 0)
    // Real load tracking would be done through request tracking in production

    // Should select first available endpoint when all have same load
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(endpoints.contains(&result));
    Ok(())
}

#[tokio::test]
async fn test_health_based_selects_healthiest() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http::endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Set different health scores using public API
    lb.update_endpoint_health(&endpoints[0], 0.5).await;
    lb.update_endpoint_health(&endpoints[1], 0.9).await;
    lb.update_endpoint_health(&endpoints[2], 0.3).await;

    // Should select endpoint2 (healthiest)
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[1]);
    Ok(())
}

#[tokio::test]
async fn test_random_selects_from_available() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    // Should select one of the endpoints
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert!(endpoints.contains(&result));
    Ok(())
}

// ==================== Availability Tests ====================

#[tokio::test]
async fn test_empty_endpoints_returns_error() {
    let lb = LoadBalancer::new(vec![], LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No endpoints configured");
}

#[tokio::test]
async fn test_all_unavailable_returns_error() {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark all unavailable
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    lb.mark_endpoint_unavailable(&endpoints[1]).await;

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "No available endpoints");
}

#[tokio::test]
async fn test_mark_endpoint_unavailable() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Initially both available
    assert_eq!(lb.available_count().await, 2);

    // Mark one unavailable
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 1);

    // Should only return the available endpoint
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[1]);
    Ok(())
}

#[tokio::test]
async fn test_mark_endpoint_available() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark unavailable then available
    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 0);

    lb.mark_endpoint_available(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 1);

    // Should be able to get endpoint again
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[0]);
    Ok(())
}

#[tokio::test]
async fn test_partial_availability_with_round_robin() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Mark middle one unavailable
    lb.mark_endpoint_unavailable(&endpoints[1]).await;

    // Should only cycle through available ones
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
    assert_eq!(r2, endpoints[2]);
    assert_eq!(r3, endpoints[0]); // Wrap around
    Ok(())
}

// ==================== Health Score Tests ====================

#[tokio::test]
async fn test_update_endpoint_health() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Update health score
    lb.update_endpoint_health(&endpoints[0], 0.75).await;

    let eps = lb.get_endpoints().await;
    assert!((eps[0].health_score - 0.75).abs() < 0.001);
    Ok(())
}

#[tokio::test]
async fn test_health_score_affects_selection() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Set one as healthier
    lb.update_endpoint_health(&endpoints[0], 0.3).await;
    lb.update_endpoint_health(&endpoints[1], 0.9).await;

    // Should prefer healthier endpoint
    for _ in 0..5 {
        let result = lb.get_next_endpoint().await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(result, endpoints[1]);
    }
    Ok(())
}

#[tokio::test]
async fn test_equal_health_scores_selectable() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Set equal low health scores (but not zero, as that might make them unavailable)
    lb.update_endpoint_health(&endpoints[0], 0.1).await;
    lb.update_endpoint_health(&endpoints[1], 0.1).await;

    // Should still select one (they're equally healthy)
    let result = lb.get_next_endpoint().await;
    assert!(result.is_ok());
    assert!(endpoints.contains(&result.ok_or_else(|| SongbirdError::configuration(format!(
        "Error: {}",
        e
    )))?));
    Ok(())
}

// ==================== Concurrent Access Tests ====================

#[tokio::test]
async fn test_concurrent_round_robin_access() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin));
    let barrier = Arc::new(Barrier::new(10));

    let mut handles = vec![];

    // Spawn 10 concurrent tasks
    for _ in 0..10 {
        let lb_clone = Arc::clone(&lb);
        let barrier_clone = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            lb_clone.get_next_endpoint().await
        }));
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
async fn test_concurrent_health_updates() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased));

    let mut handles = vec![];

    // Concurrent health updates
    for i in 0..10 {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoints[i % 2].clone();
        let health = (i as f64) * 0.1;

        handles.push(tokio::spawn(async move {
            lb_clone.update_endpoint_health(&endpoint, health).await;
        }));
    }

    // All should complete
    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Endpoints should have valid health scores
    let eps = lb.get_endpoints().await;
    for ep in eps {
        assert!(ep.health_score >= 0.0 && ep.health_score <= 1.0);
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_availability_changes() {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = Arc::new(LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin));

    let mut handles = vec![];

    // Concurrent availability changes
    for i in 0..20 {
        let lb_clone = Arc::clone(&lb);
        let endpoint = endpoints[i % 3].clone();

        handles.push(tokio::spawn(async move {
            if i % 2 == 0 {
                lb_clone.mark_endpoint_unavailable(&endpoint).await;
            } else {
                lb_clone.mark_endpoint_available(&endpoint).await;
            }
        }));
    }

    // All should complete without panic
    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // At least one endpoint should be available
    assert!(lb.available_count().await > 0);
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_non_existent_endpoint_mark_unavailable() {
    let endpoints = vec!["http://endpoint1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Mark non-existent endpoint - should not panic
    lb.mark_endpoint_unavailable("http://nonexistent:8080").await;

    // Original endpoint should still be available
    assert_eq!(lb.available_count().await, 1);
}

#[tokio::test]
async fn test_non_existent_endpoint_health_update() {
    let endpoints = vec!["http://endpoint1:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    // Update non-existent endpoint - should not panic
    lb.update_endpoint_health("http://nonexistent:8080", 0.5).await;

    // Original endpoint should be unaffected
    let eps = lb.get_endpoints().await;
    assert_eq!(eps.len(), 1);
    assert!((eps[0].health_score - 1.0).abs() < 0.001); // Default health
}

#[tokio::test]
async fn test_get_endpoints_returns_clones() {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let eps1 = lb.get_endpoints().await;
    let eps2 = lb.get_endpoints().await;

    assert_eq!(eps1.len(), eps2.len());
    assert_eq!(eps1[0].url, eps2[0].url);
}

#[tokio::test]
async fn test_available_count_accuracy() {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    assert_eq!(lb.available_count().await, 3);

    lb.mark_endpoint_unavailable(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 2);

    lb.mark_endpoint_unavailable(&endpoints[1]).await;
    assert_eq!(lb.available_count().await, 1);

    lb.mark_endpoint_available(&endpoints[0]).await;
    assert_eq!(lb.available_count().await, 2);
}

// ==================== Integration Tests ====================

#[tokio::test]
async fn test_strategy_switch_scenario() -> SongbirdResult<()> {
    // Test that different strategies work with same endpoints
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    // Test each strategy
    for strategy in [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastLoaded,
        LoadBalancingStrategy::HealthBased,
        LoadBalancingStrategy::Random,
    ] {
        let lb = LoadBalancer::new(endpoints.clone(), strategy);
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok(), "Strategy {:?} should work", strategy);
    }
    Ok(())
}

#[tokio::test]
async fn test_realistic_health_scenario() -> SongbirdResult<()> {
    let endpoints = vec![
        "http://endpoint1:8080".to_string(),
        "http://endpoint2:8080".to_string(),
        "http://endpoint3:8080".to_string(),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::HealthBased);

    // Simulate varying health scores
    lb.update_endpoint_health(&endpoints[0], 0.5).await;
    lb.update_endpoint_health(&endpoints[1], 0.8).await;
    lb.update_endpoint_health(&endpoints[2], 0.9).await;

    // Should prefer endpoint3 (healthiest)
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[2]);

    // Update health - endpoint3 degrades
    lb.update_endpoint_health(&endpoints[2], 0.2).await;

    // Should now prefer endpoint2
    let result = lb.get_next_endpoint().await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(result, endpoints[1]);
    Ok(())
}
