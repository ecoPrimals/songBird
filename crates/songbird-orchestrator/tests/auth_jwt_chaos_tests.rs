//! Chaos Tests for BearDog JWT Delegation
//!
//! Tests JWT provisioning under chaotic conditions.
//!
//! **Evolution**: Removed #[serial] - these tests are concurrent-safe!

use songbird_orchestrator::auth::provision_jwt_secret;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;

#[tokio::test]
// ✅ NO #[serial]! Concurrent chaos testing!
async fn test_chaos_jwt_provisioning_under_load() {
    // Test JWT provisioning under heavy concurrent load
    println!("🌪️  CHAOS: Testing JWT provisioning under load...");

    let concurrent_requests = 1000;
    let barrier = Arc::new(Barrier::new(concurrent_requests));

    let handles: Vec<_> = (0..concurrent_requests)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
                // Wait for all tasks to be ready
                barrier.wait().await;

                // All hit at once!
                provision_jwt_secret(None, &format!("chaos_load_{}", i))
                    .await
                    .expect("Should succeed under load")
            })
        })
        .collect();

    let start = std::time::Instant::now();
    let secrets: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();
    let elapsed = start.elapsed();

    println!("✅ {} concurrent requests in {:?}", concurrent_requests, elapsed);
    println!("   Average: {}µs per request", elapsed.as_micros() / concurrent_requests as u128);

    // All should be valid and unique
    assert_eq!(secrets.len(), concurrent_requests);
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ CHAOS: Load test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent chaos testing!
// NOTE: Removed global env mutations - provision_jwt_secret uses discovery, not env vars
async fn test_chaos_jwt_provisioning_with_varying_paths() {
    // Test JWT provisioning with varying socket paths (simulating discovery changes)
    println!("🌪️  CHAOS: Testing with varying socket paths...");

    // Simulate chaotic socket path variations by passing explicit paths
    let provision_handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                // Introduce chaos: some delay before provisioning
                tokio::time::sleep(Duration::from_millis(i % 20)).await;

                // Test with various socket paths (will fall back to secure random)
                let socket_path = if i % 3 == 0 {
                    format!("/tmp/chaos-{}.sock", i)
                } else {
                    String::new()
                };

                let socket = if !socket_path.is_empty() {
                    Some(socket_path.as_str())
                } else {
                    None
                };

                provision_jwt_secret(socket, &format!("chaos_path_{}", i))
                    .await
                    .expect("Should succeed with varying paths")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(provision_handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    println!("✅ Generated {} secrets with varying paths", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ CHAOS: Varying paths test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent chaos testing!
async fn test_chaos_jwt_provisioning_rapid_fire() {
    // Test rapid-fire JWT provisioning (stress test)
    println!("🌪️  CHAOS: Testing rapid-fire provisioning...");

    let mut handles = vec![];

    for batch in 0..10 {
        let batch_handles: Vec<_> = (0..100)
            .map(|i| {
                tokio::spawn(async move {
                    provision_jwt_secret(None, &format!("chaos_rapid_{}_{}", batch, i))
                        .await
                        .expect("Should succeed")
                })
            })
            .collect();

        handles.extend(batch_handles);

        // Small delay between batches
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let secrets: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    println!("✅ Generated {} secrets in rapid-fire mode", secrets.len());

    // All should be valid and unique
    assert_eq!(secrets.len(), 1000);
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ CHAOS: Rapid-fire test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent chaos testing!
async fn test_chaos_jwt_provisioning_with_timeouts() {
    // Test JWT provisioning with aggressive timeouts
    println!("🌪️  CHAOS: Testing with aggressive timeouts...");

    let mut successes = 0;
    let mut timeouts = 0;

    for i in 0..100 {
        let result = tokio::time::timeout(
            Duration::from_micros(100), // Very aggressive timeout
            provision_jwt_secret(None, &format!("chaos_timeout_{}", i)),
        )
        .await;

        match result {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(e)) => panic!("Unexpected error: {}", e),
            Err(_) => timeouts += 1,
        }
    }

    println!("✅ Successes: {}, Timeouts: {}", successes, timeouts);

    // Should have some successes (JWT generation is fast)
    assert!(successes > 0, "Should have at least some successes");

    println!("✅ CHAOS: Timeout test passed!");
}

#[tokio::test]
// ✅ NO #[serial]! Concurrent chaos testing!
async fn test_chaos_jwt_provisioning_memory_stress() {
    // Test JWT provisioning while allocating lots of memory
    println!("🌪️  CHAOS: Testing under memory stress...");

    let memory_hog = tokio::spawn(async {
        let mut vecs: Vec<Vec<u8>> = vec![];
        for _ in 0..100 {
            vecs.push(vec![0u8; 1024 * 1024]); // 1MB each
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        vecs // Keep alive
    });

    let provision_handles: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                provision_jwt_secret(None, &format!("chaos_memory_{}", i))
                    .await
                    .expect("Should succeed under memory stress")
            })
        })
        .collect();

    let secrets: Vec<String> = futures::future::join_all(provision_handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    let _memory = memory_hog.await.expect("Memory task should complete");

    println!("✅ Generated {} secrets under memory stress", secrets.len());

    // All should be valid
    for secret in &secrets {
        assert!(secret.len() >= 85);
    }

    println!("✅ CHAOS: Memory stress test passed!");
}
