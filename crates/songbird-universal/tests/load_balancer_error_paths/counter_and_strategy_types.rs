// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_round_robin_counter_wrap() -> SongbirdResult<()> {
    let endpoints = vec!["http://endpoint1:8080".to_string(), "http://endpoint2:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Call many times to potentially trigger counter wrap
    for _ in 0..10000 {
        let result = lb.get_next_endpoint().await;
        assert!(result.is_ok());
    }
    Ok(())
}

#[test]
fn test_load_balancing_strategy_clone() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::RoundRobin;
    let cloned = strategy;
    assert_eq!(strategy, cloned);
    Ok(())
}

#[test]
fn test_load_balancing_strategy_debug() -> SongbirdResult<()> {
    let strategy = LoadBalancingStrategy::LeastLoaded;
    let debug_str = format!("{:?}", strategy);
    assert!(!debug_str.is_empty());
    Ok(())
}
