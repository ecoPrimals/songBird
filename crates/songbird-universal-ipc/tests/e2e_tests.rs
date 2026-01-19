//! End-to-End Integration Tests for Universal IPC
//!
//! These tests verify complete workflows across multiple components:
//! - Full registration → discovery → connection flow
//! - Multi-primal scenarios
//! - Cross-platform compatibility
//! - Real-world usage patterns

use songbird_universal_ipc::{capability, ipc};
use songbird_universal_ipc::capability::discovery;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

/// E2E Test: Complete primal registration and discovery flow
#[tokio::test]
async fn test_e2e_full_primal_lifecycle() {
    // Initialize IPC
    ipc::init().expect("Failed to initialize IPC");

    // Register a primal with capabilities
    let endpoint = ipc::register("test-primal-e2e", vec!["crypto".to_string(), "storage".to_string()])
        .await
        .expect("Failed to register primal");

    // Start listener
    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        if let Ok(mut stream) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            if let Ok(n) = stream.read(&mut buf).await {
                // Echo back
                let _ = stream.write_all(&buf[..n]).await;
            }
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Client: Discover by capability
    let providers = discovery::discover_all("crypto")
        .await
        .expect("Failed to discover providers");

    assert!(!providers.is_empty(), "Should find at least one crypto provider");
    assert_eq!(providers[0].primal_id, "test-primal-e2e");

    // Client: Connect and communicate
    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Failed to connect");

    // Send message
    stream.write_all(b"hello").await.expect("Failed to write");

    // Receive echo
    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(1), stream.read(&mut buf))
        .await
        .expect("Timeout waiting for response")
        .expect("Failed to read");

    assert_eq!(&buf[..n], b"hello", "Should receive echo");

    // Cleanup
    server_handle.abort();
}

/// E2E Test: Multiple primals with different capabilities
#[tokio::test]
async fn test_e2e_multi_primal_discovery() {
    ipc::init().expect("Failed to initialize IPC");

    // Register multiple primals
    let _crypto_endpoint = ipc::register("crypto-primal", vec!["crypto".to_string()])
        .await
        .expect("Failed to register crypto primal");

    let _storage_endpoint = ipc::register("storage-primal", vec!["storage".to_string()])
        .await
        .expect("Failed to register storage primal");

    let _multi_endpoint = ipc::register(
        "multi-primal",
        vec!["crypto".to_string(), "storage".to_string()],
    )
    .await
    .expect("Failed to register multi primal");

    // Discover crypto providers
    let crypto_providers = capability::discover_all("crypto")
        .await
        .expect("Failed to discover crypto providers");

    assert_eq!(
        crypto_providers.len(),
        2,
        "Should find 2 crypto providers"
    );

    // Discover storage providers
    let storage_providers = capability::discover_all("storage")
        .await
        .expect("Failed to discover storage providers");

    assert_eq!(
        storage_providers.len(),
        2,
        "Should find 2 storage providers"
    );

    // Verify multi-primal appears in both
    let multi_in_crypto = crypto_providers
        .iter()
        .any(|p| p.primal_id == "multi-primal");
    let multi_in_storage = storage_providers
        .iter()
        .any(|p| p.primal_id == "multi-primal");

    assert!(multi_in_crypto, "Multi-primal should be in crypto");
    assert!(multi_in_storage, "Multi-primal should be in storage");
}

/// E2E Test: Concurrent connections to same primal
#[tokio::test]
async fn test_e2e_concurrent_connections() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("concurrent-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Spawn server that handles multiple connections
    let server_handle = tokio::spawn(async move {
        for _ in 0..5 {
            if let Ok(mut stream) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    if let Ok(n) = stream.read(&mut buf).await {
                        let _ = stream.write_all(&buf[..n]).await;
                    }
                });
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create 5 concurrent clients
    let mut handles = vec![];
    for i in 0..5 {
        let path = endpoint.path.clone();
        let handle = tokio::spawn(async move {
            let mut stream = ipc::connect(&path).await.expect("Failed to connect");

            let msg = format!("client-{}", i);
            stream
                .write_all(msg.as_bytes())
                .await
                .expect("Failed to write");

            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await.expect("Failed to read");

            assert_eq!(&buf[..n], msg.as_bytes());
        });
        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        handle.await.expect("Client task failed");
    }

    server_handle.abort();
}

/// E2E Test: Primal unregistration and cleanup
#[tokio::test]
async fn test_e2e_unregister_cleanup() {
    ipc::init().expect("Failed to initialize IPC");

    // Register primal
    let endpoint = ipc::register("temp-primal", vec!["temp".to_string()])
        .await
        .expect("Failed to register primal");

    // Verify it's discoverable
    let providers = capability::discover_all("temp")
        .await
        .expect("Failed to discover");
    assert_eq!(providers.len(), 1);

    // Unregister
    ipc::unregister("temp-primal")
        .await
        .expect("Failed to unregister");

    // Verify it's no longer discoverable
    let providers = capability::discover_all("temp")
        .await
        .expect("Failed to discover");
    assert_eq!(providers.len(), 0, "Should not find unregistered primal");

    // Verify connection fails
    let result = ipc::connect(&endpoint.path).await;
    assert!(result.is_err(), "Connection should fail after unregister");
}

/// E2E Test: Large message transfer
#[tokio::test]
async fn test_e2e_large_message_transfer() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("large-msg-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Server: Echo large messages
    let server_handle = tokio::spawn(async move {
        if let Ok(mut stream) = listener.accept().await {
            let mut buf = vec![0u8; 1024 * 1024]; // 1 MB buffer
            if let Ok(n) = stream.read(&mut buf).await {
                let _ = stream.write_all(&buf[..n]).await;
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Client: Send large message (100 KB)
    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Failed to connect");

    let large_msg = vec![0x42u8; 100 * 1024]; // 100 KB
    stream
        .write_all(&large_msg)
        .await
        .expect("Failed to write large message");

    // Receive echo
    let mut buf = vec![0u8; 1024 * 1024];
    let n = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .expect("Timeout")
        .expect("Failed to read");

    assert_eq!(n, large_msg.len(), "Should receive full message");
    assert_eq!(&buf[..n], &large_msg[..], "Message should match");

    server_handle.abort();
}

/// E2E Test: Capability-based discovery with filtering
#[tokio::test]
async fn test_e2e_capability_filtering() {
    ipc::init().expect("Failed to initialize IPC");

    // Register primals with different capability sets
    let _p1 = ipc::register("primal-1", vec!["crypto".to_string()])
        .await
        .expect("Failed to register");

    let _p2 = ipc::register(
        "primal-2",
        vec!["crypto".to_string(), "advanced".to_string()],
    )
    .await
    .expect("Failed to register");

    let _p3 = ipc::register("primal-3", vec!["storage".to_string()])
        .await
        .expect("Failed to register");

    // Discover crypto (should find 2)
    let crypto = capability::discover_all("crypto")
        .await
        .expect("Failed to discover");
    assert_eq!(crypto.len(), 2);

    // Discover advanced (should find 1)
    let advanced = capability::discover_all("advanced")
        .await
        .expect("Failed to discover");
    assert_eq!(advanced.len(), 1);
    assert_eq!(advanced[0].primal_id, "primal-2");

    // Discover storage (should find 1)
    let storage = capability::discover_all("storage")
        .await
        .expect("Failed to discover");
    assert_eq!(storage.len(), 1);
    assert_eq!(storage[0].primal_id, "primal-3");

    // Discover non-existent (should find 0)
    let none = capability::discover_all("nonexistent")
        .await
        .expect("Failed to discover");
    assert_eq!(none.len(), 0);
}

/// E2E Test: Reconnection after disconnect
#[tokio::test]
async fn test_e2e_reconnection() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("reconnect-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Server: Accept multiple connections
    let server_handle = tokio::spawn(async move {
        for _ in 0..3 {
            if let Ok(mut stream) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    if let Ok(n) = stream.read(&mut buf).await {
                        let _ = stream.write_all(&buf[..n]).await;
                    }
                });
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Connect, disconnect, reconnect
    for i in 0..3 {
        let mut stream = ipc::connect(&endpoint.path)
            .await
            .expect("Failed to connect");

        let msg = format!("attempt-{}", i);
        stream.write_all(msg.as_bytes()).await.expect("Write failed");

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).await.expect("Read failed");
        assert_eq!(&buf[..n], msg.as_bytes());

        // Explicit drop to disconnect
        drop(stream);

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    server_handle.abort();
}

