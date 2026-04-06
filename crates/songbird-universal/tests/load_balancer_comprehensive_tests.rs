// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Load Balancer Tests
//!
//! Tests all load balancing strategies, health tracking, and failover scenarios

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::load_balancer::{LoadBalancer, LoadBalancingStrategy};

#[tokio::test]
async fn test_round_robin_basic() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![
        format!("http://service1:{}", port),
        format!("http://service2:{}", port),
        format!("http://service3:{}", port),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // First request should go to service1
    let ep1 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep1.as_str(), format!("http://service1:{}", port).as_str());

    // Second request should go to service2
    let ep2 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep2.as_str(), format!("http://service2:{}", port).as_str());

    // Third request should go to service3
    let ep3 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep3.as_str(), format!("http://service3:{}", port).as_str());

    // Fourth request should wrap back to service1
    let ep4 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep4.as_str(), format!("http://service1:{}", port).as_str());
    Ok(())
}

#[tokio::test]
async fn test_round_robin_with_unavailable_endpoint() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![
        format!("http://service1:{}", port),
        format!("http://service2:{}", port),
        format!("http://service3:{}", port),
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Mark service2 as unavailable
    lb.mark_endpoint_unavailable(&format!("http://service2:{}", port)).await;

    // Should only cycle between service1 and service3
    let ep1 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let ep2 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let ep3 = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;

    // Collect endpoints (should only be service1 and service3)
    let mut seen = vec![ep1.clone(), ep2.clone(), ep3];
    seen.sort();
    seen.dedup();

    assert_eq!(seen.len(), 2);
    assert!(!seen.contains(&format!("http://service2:{}", port)));
    Ok(())
}

#[tokio::test]
async fn test_health_based_selection() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![
        format!("http://service1:{}", port),
        format!("http://service2:{}", port),
        format!("http://service3:{}", port),
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    // Set different health scores
    lb.update_endpoint_health(&format!("http://service1:{}", port), 0.5).await;
    lb.update_endpoint_health(&format!("http://service2:{}", port), 0.9).await;
    lb.update_endpoint_health(&format!("http://service3:{}", port), 0.3).await;

    // Should select service2 (highest health score)
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep.as_str(), format!("http://service2:{}", port));
    Ok(())
}

#[tokio::test]
async fn test_least_loaded_selection() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port), format!("http://service2:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::LeastLoaded);

    // Get endpoint (should return one with least connections)
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let service1 = format!("http://service1:{}", port);
    let service2 = format!("http://service2:{}", port);
    assert!(ep == service1 || ep == service2);
    Ok(())
}

#[tokio::test]
async fn test_random_selection() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![
        format!("http://service1:{}", port),
        format!("http://service2:{}", port),
        format!("http://service3:{}", port),
    ];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::Random);

    // Get multiple endpoints
    let mut selected = Vec::new();
    for _ in 0..20 {
        selected.push(lb.get_next_endpoint().await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?);
    }

    // Should have selected at least 2 different endpoints (statistically very likely)
    selected.sort();
    selected.dedup();
    assert!(selected.len() >= 2);
    Ok(())
}

#[tokio::test]
async fn test_no_endpoints_configured() {
    let endpoints: Vec<String> = vec![];
    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.expect_err("testing error case"), "No endpoints configured");
}

#[tokio::test]
async fn test_all_endpoints_unavailable() {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port), format!("http://service2:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Mark all endpoints unavailable
    lb.mark_endpoint_unavailable(&format!("http://service1:{}", port)).await;
    lb.mark_endpoint_unavailable(&format!("http://service2:{}", port)).await;

    let result = lb.get_next_endpoint().await;
    assert!(result.is_err());
    assert_eq!(result.expect_err("testing error case"), "No available endpoints");
}

#[tokio::test]
async fn test_endpoint_recovery() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Mark unavailable
    lb.mark_endpoint_unavailable(&format!("http://service1:{}", port)).await;
    assert!(lb.get_next_endpoint().await.is_err());

    // Mark available again
    lb.mark_endpoint_available(&format!("http://service1:{}", port)).await;
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep.as_str(), format!("http://service1:{}", port));
    Ok(())
}

#[tokio::test]
async fn test_health_score_updates() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    // Update health multiple times
    lb.update_endpoint_health(&format!("http://service1:{}", port), 0.5).await;
    lb.update_endpoint_health(&format!("http://service1:{}", port), 0.8).await;
    lb.update_endpoint_health(&format!("http://service1:{}", port), 0.3).await;

    // Should still be available
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep.as_str(), format!("http://service1:{}", port));
    Ok(())
}

#[tokio::test]
async fn test_available_count() {
    let port = test_port();
    let endpoints = vec![
        format!("http://service1:{}", port),
        format!("http://service2:{}", port),
        format!("http://service3:{}", port),
    ];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    assert_eq!(lb.available_count().await, 3);

    lb.mark_endpoint_unavailable(&format!("http://service2:{}", port)).await;
    assert_eq!(lb.available_count().await, 2);

    lb.mark_endpoint_unavailable(&format!("http://service1:{}", port)).await;
    assert_eq!(lb.available_count().await, 1);
}

#[tokio::test]
async fn test_get_all_endpoints() {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port), format!("http://service2:{}", port)];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    let all_endpoints = lb.get_endpoints().await;
    assert_eq!(all_endpoints.len(), 2);

    let urls: Vec<String> = all_endpoints.iter().map(|e| e.url.clone()).collect();
    assert!(urls.contains(&format!("http://service1:{}", port)));
    assert!(urls.contains(&format!("http://service2:{}", port)));
}

#[tokio::test]
async fn test_concurrent_endpoint_access() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port), format!("http://service2:{}", port)];

    let lb = std::sync::Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let lb_clone = lb.clone();
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
async fn test_single_endpoint_many_requests() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Should always return the same endpoint
    for _ in 0..100 {
        let ep = lb.get_next_endpoint().await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert_eq!(ep.as_str(), format!("http://service1:{}", port));
    }
    Ok(())
}

#[tokio::test]
async fn test_load_balancer_shared_access() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port)];

    let lb = std::sync::Arc::new(LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin));
    let lb2 = lb.clone();

    // Both should work
    assert!(lb.get_next_endpoint().await.is_ok());
    assert!(lb2.get_next_endpoint().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_mark_nonexistent_endpoint() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::RoundRobin);

    // Marking non-existent endpoint shouldn't panic
    lb.mark_endpoint_unavailable(&format!("http://nonexistent:{}", port)).await;
    lb.update_endpoint_health(&format!("http://nonexistent:{}", port), 0.5).await;

    // Original endpoint should still work
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep.as_str(), format!("http://service1:{}", port));
    Ok(())
}

#[tokio::test]
async fn test_health_degradation_scenario() -> SongbirdResult<()> {
    let port = test_port();
    let endpoints = vec![format!("http://service1:{}", port), format!("http://service2:{}", port)];

    let lb = LoadBalancer::new(endpoints, LoadBalancingStrategy::HealthBased);

    // Start with good health
    lb.update_endpoint_health(&format!("http://service1:{}", port), 1.0).await;
    lb.update_endpoint_health(&format!("http://service2:{}", port), 1.0).await;

    // Degrade service1
    lb.update_endpoint_health(&format!("http://service1:{}", port), 0.3).await;

    // Should prefer service2
    let ep = lb.get_next_endpoint().await.map_err(|_e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(ep.as_str(), format!("http://service2:{}", port));
    Ok(())
}

#[tokio::test]
async fn test_load_balancer_strategies_enum() {
    let port = test_port();
    // Test that all strategies are constructible
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastLoaded,
        LoadBalancingStrategy::HealthBased,
        LoadBalancingStrategy::Random,
    ];

    let endpoints = vec![format!("http://service1:{}", port)];

    for strategy in strategies {
        let lb = LoadBalancer::new(endpoints.clone(), strategy);
        assert!(lb.get_next_endpoint().await.is_ok());
    }
}
