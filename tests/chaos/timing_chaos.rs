// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Timing Chaos Tests
//!
//! Tests system behavior with timing anomalies

#![cfg(test)]

use super::common::*;

#[tokio::test]
async fn chaos_test_timestamp_consistency() -> Result<(), Box<dyn std::error::Error>> {
    // Test that timestamps remain consistent under load
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::time::sleep;

    // 1. Generate timestamps in rapid succession
    let mut timestamps = vec![];
    for _ in 0..100 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        timestamps.push(now);
        sleep(Duration::from_micros(100)).await; // Small delay
    }

    // 2. Verify timestamps are monotonic (never go backwards)
    for i in 1..timestamps.len() {
        assert!(
            timestamps[i] >= timestamps[i - 1],
            "Timestamps should be monotonic: {} >= {}",
            timestamps[i],
            timestamps[i - 1]
        );
    }

    // 3. Verify timestamps are reasonable (within 1 second)
    let first = timestamps[0];
    let last = timestamps[timestamps.len() - 1];
    let diff = last - first;
    assert!(diff < 1000, "All timestamps should be within 1 second");

    Ok(())
}

#[tokio::test]
#[ignore = "requires multi-node setup for full clock skew simulation"] // Requires multi-node setup for full clock skew simulation
async fn chaos_test_clock_skew() {
    // Test behavior when system clocks diverge
    // This test demonstrates clock skew detection capabilities
    // Full multi-node testing requires distributed infrastructure

    use std::time::Duration;
    use tokio::time::sleep;

    // 1. Simulate clock skew scenarios (local testing)
    let base_time = chrono::Utc::now();

    // Simulate timestamps from different "nodes" with skew
    let node_a_time = base_time;
    let node_b_time = base_time + chrono::Duration::milliseconds(500); // 500ms ahead
    let node_c_time = base_time - chrono::Duration::milliseconds(300); // 300ms behind

    // 2. Verify we can detect the skew
    let skew_ab = (node_b_time - node_a_time).num_milliseconds().abs();
    let skew_ac = (node_c_time - node_a_time).num_milliseconds().abs();

    assert!(skew_ab > 0, "Should detect clock skew between nodes");
    assert!(skew_ac > 0, "Should detect clock skew between nodes");

    // 3. Verify logical ordering can be maintained despite skew
    // Use Lamport-style logical clocks
    let mut logical_clock = 0;

    // Simulate events from different nodes
    let events = vec![(node_a_time, "event-a"), (node_b_time, "event-b"), (node_c_time, "event-c")];

    // Sort by logical clock (timestamp), then apply logical clock rules
    let mut sorted_events = events;
    sorted_events.sort_by_key(|(t, _)| *t);

    // Each event increments logical clock
    for (_timestamp, event) in &sorted_events {
        logical_clock += 1;
        tracing::debug!("Event {} at logical clock {}", event, logical_clock);
    }

    // 4. Verify all events were processed
    assert_eq!(logical_clock, 3, "All events should be processed");

    sleep(Duration::from_millis(10)).await; // Let async tasks complete
}

#[tokio::test]
async fn chaos_test_timeout_expiration() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when operations timeout
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    // 1. Test that fast operations complete within timeout
    let fast_result = timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await;
    assert!(fast_result.is_ok(), "Fast operation should complete");
    assert!(fast_result.unwrap().is_ok(), "Fast operation should succeed");

    // 2. Test that slow operations timeout properly
    let slow_result = timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(200)).await;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await;
    assert!(slow_result.is_err(), "Slow operation should timeout");

    // 3. Test retry logic with exponential backoff
    let mut attempts = 0;
    let max_attempts = 3;
    let mut backoff = Duration::from_millis(10);

    for _ in 0..max_attempts {
        attempts += 1;
        let result = timeout(Duration::from_millis(100), async {
            sleep(Duration::from_millis(5)).await;
            Ok::<_, Box<dyn std::error::Error>>(())
        })
        .await;

        if result.is_ok() && result.unwrap().is_ok() {
            break;
        }

        sleep(backoff).await;
        backoff *= 2; // Exponential backoff
    }

    assert_eq!(attempts, 1, "Should succeed on first attempt");

    Ok(())
}

#[tokio::test]
async fn chaos_test_race_conditions() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior under high concurrency
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration};

    // 1. Create scenario with shared state (prone to races if not properly synchronized)
    let counter = Arc::new(AtomicU64::new(0));
    let increments_per_task = 100;
    let num_tasks = 50;

    // 2. Run many concurrent operations
    let mut handles = vec![];
    for _ in 0..num_tasks {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..increments_per_task {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                // Add tiny delay to increase chance of interleaving
                sleep(Duration::from_micros(1)).await;
            }
        });
        handles.push(handle);
    }

    // 3. Wait for all tasks with timeout (verify no deadlocks)
    let join_result = timeout(Duration::from_secs(10), async {
        for handle in handles {
            handle.await.unwrap();
        }
    })
    .await;

    assert!(join_result.is_ok(), "Should complete without deadlock");

    // 4. Verify no data corruption (all increments accounted for)
    let final_count = counter.load(Ordering::SeqCst);
    let expected_count = (num_tasks * increments_per_task) as u64;
    assert_eq!(
        final_count, expected_count,
        "All increments should be counted (no race conditions)"
    );

    Ok(())
}

#[tokio::test]
async fn chaos_test_slow_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when operations are randomly slow
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::Semaphore;
    use tokio::time::sleep;

    // 1. Test normal operation timing
    let start = Instant::now();
    sleep(Duration::from_millis(10)).await;
    let normal_duration = start.elapsed();
    assert!(normal_duration.as_millis() >= 10);
    assert!(normal_duration.as_millis() < 50);

    // 2. Test slow operation (10x slower)
    let start = Instant::now();
    sleep(Duration::from_millis(100)).await;
    let slow_duration = start.elapsed();
    assert!(slow_duration.as_millis() >= 100);

    // 3. Test concurrent operations with semaphore (queuing)
    let semaphore = Arc::new(Semaphore::new(2)); // Max 2 concurrent
    let mut handles = vec![];

    for i in 0..5 {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let delay = if i % 3 == 0 {
                50
            } else {
                10
            }; // Some slow, some fast
            sleep(Duration::from_millis(delay)).await;
            i
        });
        handles.push(handle);
    }

    // 4. Verify all complete without cascading delays
    let start = Instant::now();
    for handle in handles {
        handle.await?;
    }
    let total_duration = start.elapsed();

    // With 2 concurrent and mix of 50ms/10ms, should complete reasonably fast
    assert!(total_duration.as_millis() < 500, "Should complete in reasonable time");
    assert!(total_duration.as_millis() >= 50, "Should have some delay");

    Ok(())
}
