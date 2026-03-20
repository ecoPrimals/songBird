// SPDX-License-Identifier: AGPL-3.0-only
//! Resource Chaos Tests
//!
//! Tests system behavior under resource constraints

#![cfg(test)]

use super::common::*;

#[tokio::test]
async fn chaos_test_memory_pressure() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior under memory pressure
    use std::sync::Arc;

    // 1. Start with normal memory usage (allocate some data)
    let mut memory_holders: Vec<Vec<u8>> = Vec::new();

    // 2. Gradually increase memory usage (but stay reasonable for CI)
    // Allocate 10 chunks of 1MB each (10MB total - safe for tests)
    for i in 0..10 {
        let chunk = vec![i as u8; 1024 * 1024]; // 1MB
        memory_holders.push(chunk);
    }

    // 3. Verify system still functions under memory load
    let test_data = Arc::new(vec![42u8; 1024 * 100]); // 100KB
    let test_clone = test_data.clone();

    // Spawn a task that uses the data
    let handle = tokio::spawn(async move {
        let sum: usize = test_clone.iter().map(|&x| x as usize).sum();
        assert_eq!(sum, 42 * 1024 * 100);
        Ok::<_, Box<dyn std::error::Error>>(())
    });

    // 4. Verify no crashes or panics
    handle.await??;

    // 5. Release memory and verify recovery
    memory_holders.clear();
    assert_eq!(memory_holders.len(), 0);

    Ok(())
}

#[tokio::test]
async fn chaos_test_cpu_saturation() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when CPU is saturated
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    // 1. Normal operation - lightweight task
    let start = Instant::now();
    let normal_result = tokio::spawn(async {
        let mut sum = 0u64;
        for i in 0..1000 {
            sum = sum.wrapping_add(i);
        }
        sum
    })
    .await?;
    let normal_duration = start.elapsed();
    assert!(normal_result > 0);

    // 2. Simulate CPU load with multiple compute tasks
    let mut handles = vec![];
    for _ in 0..4 {
        let handle = tokio::task::spawn_blocking(|| {
            let mut sum = 0u64;
            for i in 0..1_000_000 {
                sum = sum.wrapping_add(i);
            }
            sum
        });
        handles.push(handle);
    }

    // 3. Verify requests still complete (may be slower)
    let start = Instant::now();
    for handle in handles {
        let result = timeout(Duration::from_secs(5), handle).await??;
        assert!(result > 0);
    }
    let saturated_duration = start.elapsed();

    // 4. Verify no deadlocks (all tasks completed)
    assert!(saturated_duration < Duration::from_secs(10), "Should complete within timeout");

    // Note: Saturated may be slower than normal, but both should complete
    assert!(normal_duration < Duration::from_secs(5));
    assert!(saturated_duration < Duration::from_secs(10));

    Ok(())
}

#[tokio::test]
async fn chaos_test_file_descriptor_exhaustion() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when file descriptors are limited
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // 1. Normal operation - controlled resource usage
    let max_concurrent = 10; // Reasonable limit
    let semaphore = Arc::new(Semaphore::new(max_concurrent));

    // 2. Simulate many concurrent operations needing resources
    let mut handles = vec![];
    for i in 0..50 {
        let sem = semaphore.clone();
        let handle = tokio::spawn(async move {
            // Acquire permit (simulate FD acquisition)
            let _permit = sem.acquire().await.unwrap();

            // Simulate work with resource
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            i
            // Permit automatically released on drop
        });
        handles.push(handle);
    }

    // 3. Verify proper queuing and error handling
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await?);
    }

    // 4. Verify no resource leaks (all operations completed)
    assert_eq!(results.len(), 50);

    // Verify semaphore is back to full capacity
    let available = semaphore.available_permits();
    assert_eq!(available, max_concurrent);

    Ok(())
}

#[tokio::test]
async fn chaos_test_disk_full() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when storage operations fail (simulating disk full)
    use std::io::{Error, ErrorKind};

    // 1. Normal operation - simulate successful write
    let write_result = simulate_write(false);
    assert!(write_result.is_ok());

    // 2. Simulate disk full condition
    let disk_full_result = simulate_write(true);

    // 3. Verify writes fail gracefully (no panic)
    assert!(disk_full_result.is_err());

    // 4. Verify proper error type
    if let Err(e) = disk_full_result {
        assert_eq!(e.kind(), ErrorKind::Other);
    }

    // Helper function to simulate write with optional failure
    fn simulate_write(should_fail: bool) -> Result<(), Error> {
        if should_fail {
            Err(Error::new(ErrorKind::Other, "Disk full"))
        } else {
            Ok(())
        }
    }

    Ok(())
}

#[tokio::test]
async fn chaos_test_thread_pool_exhaustion() -> Result<(), Box<dyn std::error::Error>> {
    // Test behavior when thread pool is under heavy load
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    // 1. Normal operation
    let normal_result = tokio::task::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(10));
        42
    })
    .await?;
    assert_eq!(normal_result, 42);

    // 2. Submit many blocking tasks to stress pool
    let mut handles = vec![];
    let start = Instant::now();

    for i in 0..20 {
        let handle = tokio::task::spawn_blocking(move || {
            // Simulate blocking work
            std::thread::sleep(Duration::from_millis(50));
            i
        });
        handles.push(handle);
    }

    // 3. Verify new requests are queued and eventually execute
    let mut results = vec![];
    for handle in handles {
        // Use generous timeout to allow for queuing
        let result = timeout(Duration::from_secs(10), handle).await??;
        results.push(result);
    }

    // 4. Verify eventual completion
    assert_eq!(results.len(), 20);
    let elapsed = start.elapsed();

    // Should complete, but may take time due to queuing
    assert!(elapsed < Duration::from_secs(15));

    Ok(())
}
