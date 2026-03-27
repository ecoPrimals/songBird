// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn test_circuit_breaker_closed_to_open() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(3)
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    // Should start in Closed state
    assert!(matches!(breaker.state().await, CircuitState::Closed { .. }));

    // Fail 3 times
    for _ in 0..3 {
        let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("test")) }).await;
    }

    // Should now be Open
    assert!(matches!(breaker.state().await, CircuitState::Open { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_open_rejects_immediately() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    // Fail once to open circuit
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("test")) }).await;

    // Next call should fail immediately without executing
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = Arc::clone(&call_count);

    let result = breaker
        .call(|| {
            let cc = Arc::clone(&call_count_clone);
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Ok::<(), std::io::Error>(())
            }
        })
        .await;

    // Should fail with CircuitBreakerError::Open
    assert!(matches!(result, Err(CircuitBreakerError::Open)));
    // Should not have executed the operation
    assert_eq!(call_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_recovery() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(100))
        .success_threshold(2)
        .build()
        .unwrap();

    // Fail to open circuit
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("test")) }).await;

    // std::time::Instant is not driven by tokio's paused clock; wait for real elapsed time
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Succeed twice to close circuit
    for _ in 0..2 {
        let _ = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    }

    // Should be back in Closed state
    assert!(matches!(breaker.state().await, CircuitState::Closed { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_manual_reset() {
    let breaker = CircuitBreaker::builder().failure_threshold(1).build().unwrap();

    // Fail to open circuit
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("test")) }).await;

    assert!(matches!(breaker.state().await, CircuitState::Open { .. }));

    // Manual reset
    breaker.reset().await;

    // Should be Closed again
    assert!(matches!(breaker.state().await, CircuitState::Closed { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_stats() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(5)
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let stats = breaker.stats().await;
    assert_eq!(stats.failure_threshold, 5);
    assert_eq!(stats.timeout, Duration::from_secs(30));
    assert!(matches!(stats.state, CircuitState::Closed { .. }));
}

#[test]
fn config_validate_rejects_zero_failure_threshold() {
    let c = CircuitBreakerConfig {
        failure_threshold: 0,
        ..CircuitBreakerConfig::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn config_validate_rejects_zero_success_threshold() {
    let c = CircuitBreakerConfig {
        success_threshold: 0,
        ..CircuitBreakerConfig::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn config_validate_rejects_zero_timeout() {
    let c = CircuitBreakerConfig {
        timeout: Duration::ZERO,
        ..CircuitBreakerConfig::default()
    };
    assert!(c.validate().is_err());
}

#[tokio::test]
async fn success_in_closed_clears_failure_streak_before_open() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(3)
        .timeout(Duration::from_secs(60))
        .build()
        .expect("build");

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("a")) }).await;
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("b")) }).await;
    let _ = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("c")) }).await;
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("d")) }).await;
    assert!(matches!(breaker.state().await, CircuitState::Closed { .. }));
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("e")) }).await;
    assert!(matches!(breaker.state().await, CircuitState::Open { .. }));
}

#[tokio::test]
async fn failure_in_half_open_reopens_immediately() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(50))
        .success_threshold(3)
        .build()
        .expect("build");

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("open")) }).await;
    tokio::time::sleep(Duration::from_millis(80)).await;

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("half-open fail")) }).await;
    assert!(matches!(breaker.state().await, CircuitState::Open { .. }));
}

#[tokio::test]
async fn operation_timeout_returns_timeout_error() {
    tokio::time::pause();

    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        timeout: Duration::from_secs(60),
        success_threshold: 1,
        operation_timeout: Some(Duration::from_millis(20)),
    };
    let breaker = CircuitBreaker::new(config).expect("new");

    let handle = tokio::spawn(async move {
        breaker
            .call(|| async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok::<(), std::io::Error>(())
            })
            .await
    });

    tokio::time::advance(Duration::from_millis(25)).await;
    let result = handle.await.expect("join");

    match result {
        Err(CircuitBreakerError::Timeout(t)) => assert_eq!(t, Duration::from_millis(20)),
        other => panic!("expected Timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn stats_reflects_failures_in_closed_state() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(5)
        .timeout(Duration::from_secs(30))
        .build()
        .expect("build");
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("x")) }).await;
    let stats = breaker.stats().await;
    assert_eq!(stats.current_failures, 1);
    assert!(matches!(
        stats.state,
        CircuitState::Closed {
            failures: 1
        }
    ));
}

#[tokio::test]
async fn half_open_single_success_stays_half_open_until_success_threshold() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(50))
        .success_threshold(2)
        .build()
        .expect("build");

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(80)).await;

    let _ = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(matches!(breaker.state().await, CircuitState::HalfOpen));

    let stats = breaker.stats().await;
    assert_eq!(stats.current_successes, 1);
}

#[tokio::test]
async fn new_rejects_invalid_config() {
    let bad = CircuitBreakerConfig {
        failure_threshold: 0,
        ..CircuitBreakerConfig::default()
    };
    assert!(CircuitBreaker::new(bad).is_err());
}

#[test]
fn config_validate_accepts_default_config() {
    let c = CircuitBreakerConfig::default();
    assert!(c.validate().is_ok());
}

#[test]
fn circuit_breaker_config_builder_matches_defaults_when_unset() {
    let built = CircuitBreakerConfig::builder().build();
    let def = CircuitBreakerConfig::default();
    assert_eq!(built.failure_threshold, def.failure_threshold);
    assert_eq!(built.timeout, def.timeout);
    assert_eq!(built.success_threshold, def.success_threshold);
    assert_eq!(built.operation_timeout, def.operation_timeout);
}

#[test]
fn circuit_breaker_config_builder_partial_override() {
    let built = CircuitBreakerConfig::builder().failure_threshold(7).build();
    assert_eq!(built.failure_threshold, 7);
    assert_eq!(built.timeout, CircuitBreakerConfig::default().timeout);
}

#[tokio::test]
async fn call_returns_ok_value() {
    let breaker = CircuitBreaker::builder().build().unwrap();
    let v = breaker.call(|| async { Ok::<i32, std::io::Error>(42) }).await.unwrap();
    assert_eq!(v, 42);
}

#[tokio::test]
async fn call_maps_operation_error_to_operation_failed() {
    let breaker = CircuitBreaker::builder().build().unwrap();
    let err =
        breaker.call(|| async { Err::<(), _>(std::io::Error::other("boom")) }).await.unwrap_err();
    match err {
        CircuitBreakerError::OperationFailed(s) => assert!(s.contains("boom")),
        e => panic!("unexpected {e:?}"),
    }
}

#[test]
fn circuit_breaker_error_display_and_debug() {
    let open = CircuitBreakerError::Open;
    assert!(open.to_string().contains("open") || open.to_string().contains("Open"));
    let op = CircuitBreakerError::OperationFailed("x".into());
    assert!(format!("{op}").contains('x'));
    let to = CircuitBreakerError::Timeout(Duration::from_secs(1));
    assert!(format!("{to}").contains("1s") || format!("{to}").contains("sec"));
}

#[test]
fn circuit_state_partial_eq() {
    assert_eq!(
        CircuitState::Closed {
            failures: 2
        },
        CircuitState::Closed {
            failures: 2
        }
    );
    assert_ne!(
        CircuitState::Closed {
            failures: 1
        },
        CircuitState::Closed {
            failures: 2
        }
    );
    assert_ne!(
        CircuitState::HalfOpen,
        CircuitState::Closed {
            failures: 0
        }
    );
}

#[tokio::test]
async fn stats_when_open_reports_zero_current_failures() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("fail")) }).await;
    let stats = breaker.stats().await;
    assert!(matches!(stats.state, CircuitState::Open { .. }));
    assert_eq!(stats.current_failures, 0);
    assert_eq!(stats.failure_threshold, 1);
}

#[tokio::test]
async fn success_threshold_one_closes_from_half_open_in_single_call() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(40))
        .success_threshold(1)
        .build()
        .unwrap();

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(60)).await;

    let _ = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(matches!(breaker.state().await, CircuitState::Closed { .. }));
}

#[tokio::test]
async fn reset_clears_half_open_success_counter() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(40))
        .success_threshold(3)
        .build()
        .unwrap();

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("fail")) }).await;
    tokio::time::sleep(Duration::from_millis(60)).await;

    let _ = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(matches!(breaker.state().await, CircuitState::HalfOpen));

    breaker.reset().await;
    let stats = breaker.stats().await;
    assert_eq!(stats.current_successes, 0);
    assert!(matches!(stats.state, CircuitState::Closed { .. }));
}

#[tokio::test]
async fn closed_tracks_failure_count_until_threshold_minus_one() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(4)
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();

    for _ in 0..3 {
        let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("e")) }).await;
    }
    assert!(matches!(
        breaker.state().await,
        CircuitState::Closed {
            failures: 3
        }
    ));
}

#[tokio::test]
async fn open_rejects_until_timeout_then_allows_half_open_probe() {
    let breaker = CircuitBreaker::builder()
        .failure_threshold(1)
        .timeout(Duration::from_millis(30))
        .success_threshold(2)
        .build()
        .unwrap();

    let _ = breaker.call(|| async { Err::<(), _>(std::io::Error::other("fail")) }).await;
    let r = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(matches!(r, Err(CircuitBreakerError::Open)));

    tokio::time::sleep(Duration::from_millis(50)).await;

    let r2 = breaker.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(r2.is_ok());
    assert!(matches!(breaker.state().await, CircuitState::HalfOpen));
}

#[tokio::test]
async fn circuit_breaker_builder_build_rejects_invalid_config() {
    let result = CircuitBreaker::builder().failure_threshold(0).build();
    match result {
        Ok(_) => panic!("expected invalid config to be rejected"),
        Err(s) => assert!(s.contains("failure_threshold") || s.contains("threshold")),
    }
}
