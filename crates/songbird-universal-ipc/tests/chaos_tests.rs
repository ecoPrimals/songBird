//! Chaos Engineering Tests for Universal IPC
//!
//! These tests verify system behavior under adverse conditions:
//! - Rapid registration/unregistration
//! - Connection storms
//! - Resource exhaustion
//! - Race conditions
//! - Concurrent access patterns

use songbird_universal_ipc::{capability, ipc};
use songbird_universal_ipc::capability::discovery;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

/// Chaos Test: Rapid registration and unregistration
#[tokio::test]
async fn test_chaos_rapid_register_unregister() {
    ipc::init().expect("Failed to initialize IPC");

    let iterations = 50;
    for i in 0..iterations {
        let primal_id = format!("chaos-primal-{}", i);

        // Register
        let _endpoint = ipc::register(&primal_id, vec!["chaos".to_string()])
            .await
            .expect("Failed to register");

        // Immediately unregister
        // ipc::unregister (not implemented yet)(&primal_id)
            .await
            .expect("Failed to unregister");
    }

    // Verify all cleaned up
    let providers = capability::discover_all("chaos")
        .await
        .expect("Failed to discover");
    assert_eq!(providers.len(), 0, "All primals should be unregistered");
}

/// Chaos Test: Connection storm (many concurrent connections)
#[tokio::test]
async fn test_chaos_connection_storm() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("storm-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

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

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create 100 concurrent clients
    let mut handles = vec![];
    for i in 0..100 {
        let path = endpoint.path.clone();
        let handle = tokio::spawn(async move {
            if let Ok(mut stream) = timeout(Duration::from_secs(2), ipc::connect(&path)).await {
                if let Ok(mut stream) = stream {
                    let msg = format!("c{}", i);
                    let _ = stream.write_all(msg.as_bytes()).await;
                    let mut buf = vec![0u8; 64];
                    let _ = stream.read(&mut buf).await;
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        let _ = handle.await;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    let final_count = connection_count.load(Ordering::SeqCst);
    assert!(
        final_count >= 80,
        "Should handle most connections (got {})",
        final_count
    );

    server_handle.abort();
}

/// Chaos Test: Concurrent registration of same primal ID
#[tokio::test]
async fn test_chaos_concurrent_registration() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = "concurrent-primal";
    let success_count = Arc::new(AtomicUsize::new(0));

    // Try to register same ID concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let count = Arc::clone(&success_count);
        let handle = tokio::spawn(async move {
            if ipc::register(primal_id, vec!["test".to_string()])
                .await
                .is_ok()
            {
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
    let providers = capability::discover_all("test")
        .await
        .expect("Failed to discover");

    let concurrent_primals = providers
        .iter()
        .filter(|p| p.id == primal_id)
        .count();
    assert_eq!(concurrent_primals, 1, "Should have exactly one registration");
}

/// Chaos Test: Discovery during rapid changes
#[tokio::test]
async fn test_chaos_discovery_during_changes() {
    ipc::init().expect("Failed to initialize IPC");

    // Spawn task that constantly registers/unregisters
    let churn_handle = tokio::spawn(async {
        for i in 0..50 {
            let primal_id = format!("churn-{}", i % 5);
            let _ = ipc::register(&primal_id, vec!["churn".to_string()]).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = // ipc::unregister (not implemented yet)(&primal_id).await;
        }
    });

    // Spawn tasks that constantly discover
    let mut discovery_handles = vec![];
    for _ in 0..10 {
        let handle = tokio::spawn(async {
            for _ in 0..20 {
                let _ = capability::discover_all("churn").await;
                tokio::time::sleep(Duration::from_millis(5)).await;
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
    let _endpoint = ipc::register("final-primal", vec!["test".to_string()])
        .await
        .expect("System should still work");
}

/// Chaos Test: Listener drop during connections
#[tokio::test]
async fn test_chaos_listener_drop() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("drop-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Drop listener immediately
    drop(listener);

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Try to connect (should fail gracefully)
    let result = timeout(Duration::from_secs(1), ipc::connect(&endpoint.path)).await;

    // Either timeout or connection error is acceptable
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Connection should fail when listener is dropped"
    );
}

/// Chaos Test: Massive capability registration
#[tokio::test]
async fn test_chaos_massive_capabilities() {
    ipc::init().expect("Failed to initialize IPC");

    // Register primal with 100 capabilities
    let capabilities: Vec<String> = (0..100).map(|i| format!("cap-{}", i)).collect();

    let _endpoint = ipc::register("massive-cap-primal", capabilities.clone())
        .await
        .expect("Failed to register with many capabilities");

    // Verify all capabilities are discoverable
    for cap in &capabilities {
        let providers = capability::discover_all(cap)
            .await
            .expect("Failed to discover");
        assert_eq!(providers.len(), 1, "Should find primal for {}", cap);
    }
}

/// Chaos Test: Concurrent discovery of different capabilities
#[tokio::test]
async fn test_chaos_concurrent_discovery() {
    ipc::init().expect("Failed to initialize IPC");

    // Register primals with various capabilities
    for i in 0..10 {
        let primal_id = format!("disco-primal-{}", i);
        let cap = format!("cap-{}", i % 3); // 3 different capabilities
        let _ = ipc::register(&primal_id, vec![cap]).await;
    }

    // Concurrent discovery of all capabilities
    let mut handles = vec![];
    for i in 0..3 {
        let cap = format!("cap-{}", i);
        let handle = tokio::spawn(async move {
            for _ in 0..20 {
                let providers = capability::discover_all(&cap)
                    .await
                    .expect("Discovery failed");
                assert!(!providers.is_empty(), "Should find providers for {}", cap);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Discovery task failed");
    }
}

/// Chaos Test: Rapid connect/disconnect cycles
#[tokio::test]
async fn test_chaos_rapid_connect_disconnect() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("cycle-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Server: Accept connections rapidly
    let server_handle = tokio::spawn(async move {
        for _ in 0..50 {
            if let Ok(stream) = listener.accept().await {
                // Immediately drop connection
                drop(stream);
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Client: Rapid connect/disconnect
    for _ in 0..50 {
        if let Ok(stream) = timeout(Duration::from_millis(100), ipc::connect(&endpoint.path)).await
        {
            if let Ok(stream) = stream {
                // Immediately drop
                drop(stream);
            }
        }
    }

    server_handle.abort();

    // System should still work
    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Should still be able to connect");
    let _ = stream.write_all(b"test").await;
}

/// Chaos Test: Memory pressure (many simultaneous registrations)
#[tokio::test]
async fn test_chaos_memory_pressure() {
    ipc::init().expect("Failed to initialize IPC");

    // Register 100 primals simultaneously
    let mut handles = vec![];
    for i in 0..100 {
        let primal_id = format!("mem-primal-{}", i);
        let handle = tokio::spawn(async move {
            ipc::register(&primal_id, vec!["memory".to_string()])
                .await
                .expect("Failed to register");
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.expect("Registration task failed");
    }

    // Verify all registered
    let providers = capability::discover_all("memory")
        .await
        .expect("Failed to discover");
    assert!(
        providers.len() >= 90,
        "Should register most primals (got {})",
        providers.len()
    );

    // Cleanup all
    for i in 0..100 {
        let primal_id = format!("mem-primal-{}", i);
        let _ = // ipc::unregister (not implemented yet)(&primal_id).await;
    }
}

