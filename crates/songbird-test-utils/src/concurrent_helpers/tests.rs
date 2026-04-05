// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for [`super`] concurrent helpers.

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::sleep;

use super::{
    AsyncBarrier, CompletionWaiter, ConcurrencyLimiter, ReadinessSignal, RetryPolicy,
    unique_unix_socket,
};

#[tokio::test(start_paused = true)]
async fn test_readiness_signal() {
    let ready = Arc::new(ReadinessSignal::new());
    let ready_clone = ready.clone();

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        ready_clone.signal();
    });

    ready.wait().await.unwrap();
    assert!(ready.is_ready().await);
}

#[tokio::test(start_paused = true)]
async fn test_completion_waiter() {
    let waiter = Arc::new(CompletionWaiter::new(3));

    for _ in 0..3 {
        let w = waiter.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            w.complete();
        });
    }

    waiter.wait_all().await.unwrap();
    assert_eq!(waiter.remaining().await, 0);
}

#[tokio::test(start_paused = true)]
async fn test_async_barrier() {
    let barrier = Arc::new(AsyncBarrier::new(3));
    let completed = Arc::new(RwLock::new(0));

    for _ in 0..3 {
        let b = barrier.clone();
        let c = completed.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            b.wait().await.unwrap();
            let mut count = c.write().await;
            *count += 1;
        });
    }

    sleep(Duration::from_millis(200)).await;
    assert_eq!(*completed.read().await, 3);
}

#[tokio::test(start_paused = true)]
async fn test_retry_policy() {
    let policy = RetryPolicy::default();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let result = policy
        .retry_with_backoff(move || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                if count < 3 {
                    Err(anyhow::anyhow!("Not yet"))
                } else {
                    Ok("Success")
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[test]
fn test_unique_unix_socket() {
    let socket1 = unique_unix_socket();
    let socket2 = unique_unix_socket();

    assert_ne!(socket1, socket2);
    assert!(socket1.to_string_lossy().contains("songbird-test"));
}

#[tokio::test(start_paused = true)]
async fn test_concurrency_limiter() {
    let limiter = Arc::new(ConcurrencyLimiter::new(2));
    let concurrent = Arc::new(RwLock::new(0));
    let max_concurrent = Arc::new(RwLock::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let l = limiter.clone();
        let c = concurrent.clone();
        let m = max_concurrent.clone();

        let handle = tokio::spawn(async move {
            let _guard = l.acquire().await.unwrap();

            {
                let mut count = c.write().await;
                *count += 1;
                let mut max = m.write().await;
                *max = (*max).max(*count);
            }

            sleep(Duration::from_millis(10)).await;

            {
                let mut count = c.write().await;
                *count -= 1;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(*max_concurrent.read().await, 2);
}

#[tokio::test(start_paused = true)]
async fn readiness_wait_second_call_uses_fast_path() {
    let ready = Arc::new(ReadinessSignal::new());
    ready.signal();
    for _ in 0..16 {
        tokio::task::yield_now().await;
    }
    ready.wait().await.expect("first wait");
    ready
        .wait_with_timeout(Duration::from_secs(1))
        .await
        .expect("second wait should fast-return when already signaled");
    assert!(ready.is_ready().await);
}

#[tokio::test(start_paused = true)]
async fn readiness_reset_clears_signaled_state() {
    let ready = Arc::new(ReadinessSignal::new());
    ready.signal();
    for _ in 0..16 {
        tokio::task::yield_now().await;
    }
    ready.wait().await.expect("wait after signal");
    ready.reset().await;
    assert!(!ready.is_ready().await, "reset should clear readiness");
}

#[test]
fn readiness_signal_default_matches_new() {
    let a = ReadinessSignal::default();
    let b = ReadinessSignal::new();
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
}

#[tokio::test(start_paused = true)]
async fn completion_waiter_zero_tasks_finishes_immediately() {
    let waiter = Arc::new(CompletionWaiter::new(0));
    waiter.wait_all().await.expect("no tasks means immediate success");
    assert_eq!(waiter.remaining().await, 0);
}

#[tokio::test(start_paused = true)]
async fn completion_waiter_extra_complete_is_noop() {
    let waiter = Arc::new(CompletionWaiter::new(1));
    waiter.complete();
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
    waiter.complete();
    waiter.wait_all().await.expect("wait after duplicate complete");
    assert_eq!(waiter.remaining().await, 0);
}

#[tokio::test(start_paused = true)]
async fn completion_waiter_remaining_starts_at_count() {
    let waiter = Arc::new(CompletionWaiter::new(4));
    assert_eq!(waiter.remaining().await, 4);
}

#[tokio::test(start_paused = true)]
async fn async_barrier_single_participant_releases() {
    let barrier = Arc::new(AsyncBarrier::new(1));
    barrier.wait().await.expect("one participant should unblock");
}

#[tokio::test(start_paused = true)]
async fn retry_policy_succeeds_on_first_attempt_without_extra_delay() {
    let policy = RetryPolicy::new(5, Duration::ZERO, Duration::from_secs(1), 2.0);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_c = calls.clone();
    let out = policy
        .retry_with_backoff(move || {
            let calls_c = calls_c.clone();
            async move {
                calls_c.fetch_add(1, Ordering::SeqCst);
                Ok::<&str, String>("ok")
            }
        })
        .await
        .expect("retry should succeed");
    assert_eq!(out, "ok");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn retry_policy_returns_last_error_after_max_attempts() {
    let policy = RetryPolicy::new(2, Duration::ZERO, Duration::from_millis(1), 2.0);
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_c = attempts.clone();
    let err = policy
        .retry_with_backoff(move || {
            let attempts_c = attempts_c.clone();
            async move {
                attempts_c.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("always fail")
            }
        })
        .await
        .expect_err("should exhaust retries");
    assert_eq!(err.to_string(), "always fail");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn retry_policy_default_has_expected_shape() {
    let p = RetryPolicy::default();
    let p2 = RetryPolicy::new(5, Duration::from_millis(100), Duration::from_secs(5), 2.0);
    assert_eq!(format!("{p:?}"), format!("{p2:?}"));
}

#[test]
fn concurrency_limiter_try_acquire_and_available_permits() {
    let limiter = ConcurrencyLimiter::new(2);
    assert_eq!(limiter.available_permits(), 2);
    let _a = limiter.try_acquire().expect("first permit");
    assert_eq!(limiter.available_permits(), 1);
    let _b = limiter.try_acquire().expect("second permit");
    assert_eq!(limiter.available_permits(), 0);
    assert!(limiter.try_acquire().is_none());
}
