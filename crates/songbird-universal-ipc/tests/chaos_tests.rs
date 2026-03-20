// SPDX-License-Identifier: AGPL-3.0-only
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
    clippy::must_use_candidate
)]

//! Chaos Engineering Tests for Universal IPC
//!
//! **Concurrency Evolution**: Each test uses unique primal/capability names
//! to avoid collision in the shared global registry. Tests clean up after
//! themselves. No sleep-based coordination — uses timeouts and yields.
//!
//! Tests verify system behavior under adverse conditions:
//! - Rapid registration/unregistration
//! - Connection storms
//! - Resource exhaustion
//! - Race conditions
//! - Concurrent access patterns

use songbird_universal_ipc::ipc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

/// Generate test-unique names to avoid cross-test collision in global registry
fn unique_id(prefix: &str) -> String {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("{}-{}", prefix, COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Chaos Test: Rapid registration and unregistration
#[tokio::test]
async fn test_chaos_rapid_register_unregister() {
    ipc::init().expect("Failed to initialize IPC");

    let cap = unique_id("rapid-cap");
    let iterations = 50;
    for i in 0..iterations {
        let primal_id = format!("rapid-primal-{}-{}", cap, i);

        // Register
        let _endpoint =
            ipc::register(&primal_id, vec![cap.clone()]).await.expect("Failed to register");

        // Immediately unregister
        ipc::unregister(&primal_id).await.expect("Failed to unregister");
    }

    // Verify all cleaned up
    let providers = ipc::find_by_capability(&cap).await;
    assert_eq!(providers.len(), 0, "All primals should be unregistered");
}

/// Chaos Test: Connection storm (many concurrent connections)
#[tokio::test]
async fn test_chaos_connection_storm() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = unique_id("storm-primal");
    let cap = unique_id("storm-cap");
    let endpoint = ipc::register(&primal_id, vec![cap]).await.expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone()).await.expect("Failed to create listener");

    let connection_count = Arc::new(AtomicUsize::new(0));
    let count_clone = Arc::clone(&connection_count);

    // Server: Accept many connections
    let server_handle = tokio::spawn(async move {
        for _ in 0..100 {
            if let Ok(mut stream) = listener.accept().await {
                let count = Arc::clone(&count_clone);
                tokio::spawn(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    let mut buf = vec![0u8; 64];
                    if let Ok(n) = stream.read(&mut buf).await {
                        let _ = stream.write_all(&buf[..n]).await;
                    }
                });
            }
        }
    });

    // Yield to let server start accepting (no sleep!)
    tokio::task::yield_now().await;

    // Create 100 concurrent clients
    let mut handles = vec![];
    for i in 0..100 {
        let path = endpoint.path.clone();
        let handle = tokio::spawn(async move {
            if let Ok(Ok(mut stream)) = timeout(Duration::from_secs(2), ipc::connect(&path)).await {
                let msg = format!("c{}", i);
                let _ = stream.write_all(msg.as_bytes()).await;
                let mut buf = vec![0u8; 64];
                let _ = stream.read(&mut buf).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        let _ = handle.await;
    }

    // Brief yield for final connections to register
    tokio::task::yield_now().await;

    let final_count = connection_count.load(Ordering::SeqCst);
    assert!(final_count >= 80, "Should handle most connections (got {})", final_count);

    server_handle.abort();

    // Cleanup
    let _ = ipc::unregister(&primal_id).await;
}

/// Chaos Test: Concurrent registration of same primal ID
#[tokio::test]
async fn test_chaos_concurrent_registration() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = unique_id("concurrent-primal");
    let cap = unique_id("concurrent-cap");
    let success_count = Arc::new(AtomicUsize::new(0));

    // Try to register same ID concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let count = Arc::clone(&success_count);
        let id = primal_id.clone();
        let c = cap.clone();
        let handle = tokio::spawn(async move {
            if ipc::register(&id, vec![c]).await.is_ok() {
                count.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Only one should succeed (or all should succeed with last-write-wins)
    let count = success_count.load(Ordering::SeqCst);
    assert!(count >= 1, "At least one registration should succeed");

    // Verify only one is registered
    let services = ipc::find_by_capability(&cap).await;
    let concurrent_primals = services.iter().filter(|path| path.contains(&primal_id)).count();
    assert_eq!(concurrent_primals, 1, "Should have exactly one registration");

    // Cleanup
    let _ = ipc::unregister(&primal_id).await;
}

/// Chaos Test: Discovery during rapid changes
#[tokio::test]
async fn test_chaos_discovery_during_changes() {
    ipc::init().expect("Failed to initialize IPC");

    let cap = unique_id("churn-cap");

    // Spawn task that constantly registers/unregisters
    let churn_cap = cap.clone();
    let churn_handle = tokio::spawn(async move {
        for i in 0..50 {
            let primal_id = format!("churn-{}-{}", churn_cap, i % 5);
            let _ = ipc::register(&primal_id, vec![churn_cap.clone()]).await;
            tokio::task::yield_now().await;
            let _ = ipc::unregister(&primal_id).await;
        }
    });

    // Spawn tasks that constantly discover
    let mut discovery_handles = vec![];
    for _ in 0..10 {
        let disc_cap = cap.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..20 {
                let _ = ipc::find_by_capability(&disc_cap).await;
                tokio::task::yield_now().await;
            }
        });
        discovery_handles.push(handle);
    }

    // Wait for all tasks
    churn_handle.await.expect("Churn task failed");
    for handle in discovery_handles {
        handle.await.expect("Discovery task failed");
    }

    // System should still be functional
    let final_cap = unique_id("final-cap");
    let final_id = unique_id("final-primal");
    let _endpoint =
        ipc::register(&final_id, vec![final_cap]).await.expect("System should still work");

    // Cleanup
    let _ = ipc::unregister(&final_id).await;
}

/// Chaos Test: Listener drop during connections
#[tokio::test]
async fn test_chaos_listener_drop() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = unique_id("drop-primal");
    let cap = unique_id("drop-cap");
    let endpoint = ipc::register(&primal_id, vec![cap]).await.expect("Failed to register primal");

    let listener = ipc::listen(endpoint.clone()).await.expect("Failed to create listener");

    // Drop listener immediately
    drop(listener);

    // Yield to let the drop propagate
    tokio::task::yield_now().await;

    // Try to connect (should fail gracefully)
    let result = timeout(Duration::from_secs(1), ipc::connect(&endpoint.path)).await;

    // Either timeout or connection error is acceptable
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Connection should fail when listener is dropped"
    );

    // Cleanup
    let _ = ipc::unregister(&primal_id).await;
}

/// Chaos Test: Massive capability registration
#[tokio::test]
async fn test_chaos_massive_capabilities() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = unique_id("massive-cap-primal");
    // Use test-unique capability prefix to avoid cross-test pollution
    let cap_prefix = unique_id("mcap");
    let capabilities: Vec<String> = (0..100).map(|i| format!("{}-{}", cap_prefix, i)).collect();

    let _endpoint = ipc::register(&primal_id, capabilities.clone())
        .await
        .expect("Failed to register with many capabilities");

    // Verify all capabilities are discoverable
    for cap in &capabilities {
        let services = ipc::find_by_capability(cap).await;
        assert_eq!(services.len(), 1, "Should find exactly one primal for {}", cap);
    }

    // Cleanup
    let _ = ipc::unregister(&primal_id).await;
}

/// Chaos Test: Concurrent discovery of different capabilities
#[tokio::test]
async fn test_chaos_concurrent_discovery() {
    ipc::init().expect("Failed to initialize IPC");

    let cap_prefix = unique_id("disco-cap");

    // Register primals with various capabilities
    let mut registered = vec![];
    for i in 0..10 {
        let primal_id = format!("disco-primal-{}-{}", cap_prefix, i);
        let cap = format!("{}-{}", cap_prefix, i % 3); // 3 different capabilities
        let _ = ipc::register(&primal_id, vec![cap]).await;
        registered.push(primal_id);
    }

    // Concurrent discovery of all capabilities
    let mut handles = vec![];
    for i in 0..3 {
        let cap = format!("{}-{}", cap_prefix, i);
        let handle = tokio::spawn(async move {
            for _ in 0..20 {
                let services = ipc::find_by_capability(&cap).await;
                assert!(!services.is_empty(), "Should find providers for {}", cap);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Discovery task failed");
    }

    // Cleanup
    for id in &registered {
        let _ = ipc::unregister(id).await;
    }
}

/// Chaos Test: Rapid connect/disconnect cycles
#[tokio::test]
async fn test_chaos_rapid_connect_disconnect() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = unique_id("cycle-primal");
    let cap = unique_id("cycle-cap");
    let endpoint = ipc::register(&primal_id, vec![cap]).await.expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone()).await.expect("Failed to create listener");

    // Server: Accept connections rapidly
    let server_handle = tokio::spawn(async move {
        while let Ok(Ok(stream)) = timeout(Duration::from_secs(5), listener.accept()).await {
            // Immediately drop connection
            drop(stream);
        }
    });

    // Yield to let server start accepting
    tokio::task::yield_now().await;

    // Client: Rapid connect/disconnect
    for _ in 0..50 {
        if let Ok(Ok(stream)) =
            timeout(Duration::from_millis(200), ipc::connect(&endpoint.path)).await
        {
            // Immediately drop
            drop(stream);
        }
    }

    server_handle.abort();

    // Cleanup
    let _ = ipc::unregister(&primal_id).await;
}

/// Chaos Test: Memory pressure (many simultaneous registrations)
#[tokio::test]
async fn test_chaos_memory_pressure() {
    ipc::init().expect("Failed to initialize IPC");

    let cap = unique_id("memory-cap");

    // Register 100 primals simultaneously
    let mut handles = vec![];
    let mut primal_ids = vec![];
    for i in 0..100 {
        let primal_id = format!("mem-primal-{}-{}", cap, i);
        primal_ids.push(primal_id.clone());
        let c = cap.clone();
        let handle = tokio::spawn(async move {
            ipc::register(&primal_id, vec![c]).await.expect("Failed to register");
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.expect("Registration task failed");
    }

    // Verify all registered
    let services = ipc::find_by_capability(&cap).await;
    assert!(services.len() >= 90, "Should register most primals (got {})", services.len());

    // Cleanup all
    for id in &primal_ids {
        let _ = ipc::unregister(id).await;
    }
}
