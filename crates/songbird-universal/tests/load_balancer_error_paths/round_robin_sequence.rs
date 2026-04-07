// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;

#[tokio::test]
async fn test_round_robin_predictable_sequence() -> SongbirdResult<()> {
    let endpoints =
        vec!["http://a:8080".to_string(), "http://b:8080".to_string(), "http://c:8080".to_string()];

    let lb = LoadBalancer::new(endpoints.clone(), LoadBalancingStrategy::RoundRobin);

    // Should cycle through in order
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|_e| SongbirdError::configuration("Error"))?,
        endpoints[0]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|_e| SongbirdError::configuration("Error"))?,
        endpoints[1]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|_e| SongbirdError::configuration("Error"))?,
        endpoints[2]
    );
    assert_eq!(
        lb.get_next_endpoint().await.map_err(|_e| SongbirdError::configuration("Error"))?,
        endpoints[0]
    );
    Ok(())
}
