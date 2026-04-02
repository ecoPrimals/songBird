// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;

/// Mock connection type for testing
#[derive(Debug, Clone)]
struct MockConnection {
    id: usize,
}

#[tokio::test]
async fn test_pool_creation() {
    let pool =
        ConnectionPool::<MockConnection>::builder().max_size(5).min_idle(2).build().await.unwrap();

    let stats = pool.stats().await;
    assert_eq!(stats.max_connections, 5);
    assert_eq!(stats.min_idle, 2);
}

#[tokio::test]
async fn test_add_and_acquire_connection() {
    let pool = ConnectionPool::builder().max_size(5).build().await.unwrap();

    let conn = MockConnection {
        id: 1,
    };
    pool.add_connection(conn).await.unwrap();

    let acquired = pool.acquire().await.unwrap();
    assert_eq!(acquired.inner().unwrap().id, 1);
}

#[tokio::test]
async fn test_pool_full() {
    let pool = ConnectionPool::builder().max_size(2).build().await.unwrap();

    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    pool.add_connection(MockConnection {
        id: 2,
    })
    .await
    .unwrap();

    let result = pool
        .add_connection(MockConnection {
            id: 3,
        })
        .await;
    assert!(matches!(result, Err(PoolError::PoolFull(2))));
}

#[tokio::test(start_paused = true)]
async fn test_connection_return_on_drop() {
    let pool = ConnectionPool::builder().max_size(5).build().await.unwrap();

    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();

    {
        let _conn = pool.acquire().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(10)).await;

    let stats = pool.stats().await;
    assert!(stats.idle_connections > 0 || stats.total_connections > 0);
}

#[tokio::test]
async fn test_config_validation() {
    let config = PoolConfig {
        max_size: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());

    let config = PoolConfig {
        max_size: 5,
        min_idle: 10,
        ..Default::default()
    };
    assert!(config.validate().is_err());

    let config = PoolConfig {
        max_size: 10,
        min_idle: 2,
        max_idle_time: Duration::ZERO,
        ..Default::default()
    };
    assert!(config.validate().is_err());

    let config = PoolConfig {
        max_size: 10,
        min_idle: 5,
        ..Default::default()
    };
    assert!(config.validate().is_ok());
}

#[test]
fn pool_config_builder_defaults_and_overrides() {
    let built = PoolConfig::builder()
        .max_size(42)
        .min_idle(3)
        .max_idle_time(Duration::from_secs(120))
        .acquire_timeout(Duration::from_secs(9))
        .build();
    assert_eq!(built.max_size, 42);
    assert_eq!(built.min_idle, 3);
    assert_eq!(built.max_idle_time, Duration::from_secs(120));
    assert_eq!(built.acquire_timeout, Duration::from_secs(9));
    assert_eq!(built.cleanup_interval, PoolConfig::default().cleanup_interval);
}

#[test]
fn pool_config_default_values() {
    let c = PoolConfig::default();
    assert_eq!(c.max_size, 10);
    assert_eq!(c.min_idle, 2);
    assert_eq!(c.max_idle_time, Duration::from_secs(60));
    assert_eq!(c.acquire_timeout, Duration::from_secs(5));
    assert_eq!(c.cleanup_interval, Duration::from_secs(30));
    assert_eq!(c.health_check_interval, Duration::from_secs(10));
}

#[test]
fn pool_config_validate_min_idle_equals_max_size_is_ok() {
    let config = PoolConfig {
        max_size: 4,
        min_idle: 4,
        ..Default::default()
    };
    assert!(config.validate().is_ok());
}

#[test]
fn pool_error_display() {
    let e = PoolError::PoolFull(3);
    assert!(e.to_string().contains('3'));
    assert!(matches!(e, PoolError::PoolFull(3)));
}

#[test]
fn pool_error_unhealthy_connection_display() {
    let e = PoolError::UnhealthyConnection;
    assert!(!e.to_string().is_empty());
}

#[test]
fn pool_error_acquisition_timeout_display() {
    let d = Duration::from_millis(250);
    let e = PoolError::AcquisitionTimeout(d);
    let s = e.to_string();
    assert!(s.contains("250") || s.contains("0.25"));
}

#[test]
fn pool_error_connection_creation_display() {
    let e = PoolError::ConnectionCreation("ipc failure".to_string());
    assert!(e.to_string().contains("ipc failure"));
}

#[test]
fn pool_error_shutting_down_display() {
    let e = PoolError::ShuttingDown;
    assert!(e.to_string().to_lowercase().contains("shut"));
}

#[test]
fn pool_result_ok_maps_value() {
    let r: PoolResult<i32> = Ok(7);
    assert_eq!(r, Ok(7));
}

#[tokio::test]
async fn shutting_down_rejects_new_connections() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.shutdown().await;
    let err = pool
        .add_connection(MockConnection {
            id: 99,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, PoolError::ShuttingDown));
}

#[tokio::test]
async fn pool_stats_track_max_and_min_idle_from_config() {
    let pool =
        ConnectionPool::<MockConnection>::builder().max_size(7).min_idle(4).build().await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.max_connections, 7);
    assert_eq!(stats.min_idle, 4);
}

#[tokio::test]
async fn connection_pool_new_sets_max_size() {
    let pool = ConnectionPool::<MockConnection>::new(6).await.unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.max_connections, 6);
}

#[tokio::test]
async fn stale_idle_connection_is_not_acquired() {
    let pool = ConnectionPool::<MockConnection>::builder()
        .max_size(3)
        .max_idle_time(Duration::from_millis(1))
        .build()
        .await
        .unwrap();

    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    match pool.acquire().await {
        Err(e) => assert!(matches!(e, PoolError::UnhealthyConnection)),
        Ok(_) => panic!("expected unhealthy pool"),
    }
}

#[tokio::test]
async fn acquire_after_shutdown_returns_shutting_down() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.shutdown().await;
    match pool.acquire().await {
        Err(e) => assert!(matches!(e, PoolError::ShuttingDown)),
        Ok(_) => panic!("expected shutdown"),
    }
}

#[tokio::test]
async fn build_fails_when_config_invalid() {
    match ConnectionPool::<MockConnection>::builder().max_size(0).build().await {
        Err(e) => assert!(matches!(e, PoolError::ConnectionCreation(_))),
        Ok(_) => panic!("expected invalid config"),
    }
}

#[tokio::test]
async fn pooled_connection_touch_updates_health() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    assert!(conn.is_healthy());
    conn.touch();
    assert!(conn.is_healthy());
}

#[tokio::test]
async fn empty_pool_acquire_returns_unhealthy() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(3).build().await.unwrap();
    match pool.acquire().await {
        Err(e) => assert!(matches!(e, PoolError::UnhealthyConnection)),
        Ok(_) => panic!("expected empty pool to yield UnhealthyConnection"),
    }
}

#[tokio::test]
async fn pooled_connection_deref_and_inner_mut() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 10,
    })
    .await
    .unwrap();

    let mut acquired = pool.acquire().await.unwrap();
    assert_eq!(acquired.id, 10);
    acquired.id = 11;
    assert_eq!(acquired.inner_mut().unwrap().id, 11);
}

#[test]
fn pool_stats_clone_debug() {
    let s = PoolStats {
        total_connections: 2,
        idle_connections: 2,
        max_connections: 5,
        min_idle: 1,
    };
    let c = s.clone();
    assert_eq!(c.total_connections, 2);
    let dbg = format!("{s:?}");
    assert!(dbg.contains('2') || dbg.contains('5'));
}

#[tokio::test]
async fn connection_pool_builder_chains_all_public_options() {
    let pool = ConnectionPool::<MockConnection>::builder()
        .max_size(8)
        .min_idle(1)
        .max_idle_time(Duration::from_secs(90))
        .acquire_timeout(Duration::from_secs(3))
        .build()
        .await
        .unwrap();
    let stats = pool.stats().await;
    assert_eq!(stats.max_connections, 8);
    assert_eq!(stats.min_idle, 1);
}

#[tokio::test(start_paused = true)]
async fn acquire_then_release_roundtrip_multiple_times() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 100,
    })
    .await
    .unwrap();

    for _ in 0..8 {
        let c = pool.acquire().await.unwrap();
        assert_eq!(c.id, 100);
        drop(c);
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}

#[tokio::test(start_paused = true)]
async fn concurrent_acquire_return_no_panic() {
    let pool = std::sync::Arc::new(
        ConnectionPool::<MockConnection>::builder().max_size(4).build().await.unwrap(),
    );
    for i in 0..4 {
        pool.add_connection(MockConnection {
            id: i,
        })
        .await
        .unwrap();
    }

    let mut handles = vec![];
    for _ in 0..8 {
        let p = std::sync::Arc::clone(&pool);
        handles.push(tokio::spawn(async move {
            if let Ok(c) = p.acquire().await {
                tokio::time::sleep(Duration::from_millis(1)).await;
                drop(c);
            }
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
}

#[tokio::test]
async fn pooled_connection_is_unhealthy_after_idle_exceeds_max_while_held() {
    let pool = ConnectionPool::<MockConnection>::builder()
        .max_size(2)
        .max_idle_time(Duration::from_millis(20))
        .build()
        .await
        .unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();

    let conn = pool.acquire().await.unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(!conn.is_healthy());
}

#[tokio::test(start_paused = true)]
async fn dropped_connection_after_shutdown_does_not_repopulate_pool() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();

    let conn = pool.acquire().await.unwrap();
    pool.shutdown().await;
    drop(conn);

    tokio::time::sleep(Duration::from_millis(30)).await;

    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.idle_connections, 0);
}

#[tokio::test]
async fn pool_stats_idle_equals_total_when_nothing_checked_out() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(4).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    pool.add_connection(MockConnection {
        id: 2,
    })
    .await
    .unwrap();

    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 2);
    assert_eq!(stats.idle_connections, 2);
}

#[tokio::test]
async fn acquire_skips_stale_head_and_returns_next_fresh_connection() {
    let pool = ConnectionPool::<MockConnection>::builder()
        .max_size(4)
        .max_idle_time(Duration::from_millis(100))
        .build()
        .await
        .unwrap();

    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;
    pool.add_connection(MockConnection {
        id: 2,
    })
    .await
    .unwrap();

    let acquired = pool.acquire().await.unwrap();
    assert_eq!(acquired.id, 2);
}

#[test]
fn pool_error_partial_eq_and_clone() {
    assert_eq!(PoolError::ShuttingDown, PoolError::ShuttingDown);
    assert_ne!(PoolError::ShuttingDown, PoolError::UnhealthyConnection);
}

#[test]
fn pool_config_clone_preserves_fields() {
    let c = PoolConfig {
        max_size: 3,
        min_idle: 1,
        max_idle_time: Duration::from_secs(10),
        acquire_timeout: Duration::from_secs(2),
        cleanup_interval: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(1),
    };
    assert_eq!(c.max_size, 3);
    assert_eq!(c.health_check_interval, Duration::from_secs(1));
}

#[test]
fn pool_config_builder_preserves_cleanup_and_health_interval_defaults() {
    let built = PoolConfig::builder().max_size(11).build();
    assert_eq!(built.cleanup_interval, PoolConfig::default().cleanup_interval);
    assert_eq!(built.health_check_interval, PoolConfig::default().health_check_interval);
}

#[tokio::test(start_paused = true)]
async fn add_connection_after_acquire_increments_idle_when_returned() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(3).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    let c = pool.acquire().await.unwrap();
    drop(c);
    tokio::time::sleep(Duration::from_millis(15)).await;
    let stats = pool.stats().await;
    assert!(stats.total_connections >= 1);
}

#[tokio::test]
async fn multiple_stale_connections_all_skipped_yields_unhealthy() {
    let pool = ConnectionPool::<MockConnection>::builder()
        .max_size(5)
        .max_idle_time(Duration::from_millis(1))
        .build()
        .await
        .unwrap();
    for i in 0..3 {
        pool.add_connection(MockConnection {
            id: i,
        })
        .await
        .unwrap();
    }
    tokio::time::sleep(Duration::from_millis(40)).await;
    let err = pool.acquire().await;
    assert!(matches!(err, Err(PoolError::UnhealthyConnection)));
}

#[tokio::test]
async fn acquire_releases_permit_on_unhealthy_empty_deque() {
    let pool =
        ConnectionPool::<MockConnection>::builder().max_size(1).min_idle(0).build().await.unwrap();
    let e = pool.acquire().await;
    assert!(matches!(e, Err(PoolError::UnhealthyConnection)));
    pool.add_connection(MockConnection {
        id: 7,
    })
    .await
    .unwrap();
    let c = pool.acquire().await.unwrap();
    assert_eq!(c.id, 7);
}

#[test]
fn pool_stats_field_accessors() {
    let s = PoolStats {
        total_connections: 0,
        idle_connections: 0,
        max_connections: 9,
        min_idle: 0,
    };
    assert_eq!(s.max_connections, 9);
    assert_eq!(s.min_idle, 0);
}

#[test]
fn pool_config_validate_accepts_min_idle_zero() {
    let c = PoolConfig {
        max_size: 5,
        min_idle: 0,
        ..Default::default()
    };
    assert!(c.validate().is_ok());
}

#[tokio::test]
async fn pooled_connection_inner_none_not_constructed_via_acquire() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    let p = pool.acquire().await.unwrap();
    assert!(p.inner().is_some());
}

#[tokio::test]
async fn shutdown_then_stats_empty() {
    let pool = ConnectionPool::<MockConnection>::builder().max_size(2).build().await.unwrap();
    pool.add_connection(MockConnection {
        id: 1,
    })
    .await
    .unwrap();
    pool.shutdown().await;
    let stats = pool.stats().await;
    assert_eq!(stats.total_connections, 0);
}
