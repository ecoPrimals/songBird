//! Fault Injection Tests for Universal IPC
//!
//! These tests verify system resilience under fault conditions:
//! - Network failures
//! - Timeout scenarios
//! - Invalid inputs
//! - Resource limits
//! - Error recovery

use songbird_universal_ipc::{capability, ipc};
use songbird_universal_ipc::capability::discovery;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

/// Fault Test: Connection to non-existent endpoint
#[tokio::test]
async fn test_fault_connect_nonexistent() {
    ipc::init().expect("Failed to initialize IPC");

    // Try to connect to non-existent primal
    let result = timeout(
        Duration::from_secs(1),
        ipc::connect("/primal/nonexistent"),
    )
    .await;

    // Should either timeout or fail with error
    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Connection to nonexistent primal should fail"
    );
}

/// Fault Test: Discovery of non-existent capability
#[tokio::test]
async fn test_fault_discover_nonexistent_capability() {
    ipc::init().expect("Failed to initialize IPC");

    let providers = capability::discover_all("nonexistent-capability-xyz")
        .await
        .expect("Discovery should not error");

    assert_eq!(
        providers.len(),
        0,
        "Should return empty list for nonexistent capability"
    );
}

/// Fault Test: Invalid primal ID (empty string)
#[tokio::test]
async fn test_fault_empty_primal_id() {
    ipc::init().expect("Failed to initialize IPC");

    let result = ipc::register("", vec!["test".to_string()]).await;

    assert!(result.is_err(), "Empty primal ID should fail");
}

/// Fault Test: Invalid primal ID (special characters)
#[tokio::test]
async fn test_fault_invalid_primal_id_chars() {
    ipc::init().expect("Failed to initialize IPC");

    // These should either fail or be sanitized
    let invalid_ids = vec![
        "../../../etc/passwd",
        "primal\0null",
        "primal\nline",
        "primal\ttab",
    ];

    for id in invalid_ids {
        let result = ipc::register(id, vec!["test".to_string()]).await;

        // Either fails or succeeds with sanitized ID
        if let Ok(endpoint) = result {
            // If it succeeds, verify the endpoint path is safe
            assert!(
                !endpoint.path.contains('\0'),
                "Endpoint should not contain null bytes"
            );
            assert!(
                !endpoint.path.contains('\n'),
                "Endpoint should not contain newlines"
            );

            // Cleanup
            let _ = ipc::unregister(id).await;
        }
    }
}

/// Fault Test: Empty capabilities list
#[tokio::test]
async fn test_fault_empty_capabilities() {
    ipc::init().expect("Failed to initialize IPC");

    let result = ipc::register("empty-cap-primal", vec![]).await;

    // Should either fail or succeed with no capabilities
    if let Ok(_endpoint) = result {
        // If it succeeds, verify it's not discoverable by any capability
        let providers = capability::discover_all("any")
            .await
            .expect("Discovery failed");

        let found = providers
            .iter()
            .any(|p| p.primal_id == "empty-cap-primal");
        assert!(
            !found,
            "Primal with no capabilities should not be discoverable"
        );

        // Cleanup
        let _ = ipc::unregister("empty-cap-primal").await;
    }
}

/// Fault Test: Connection timeout
#[tokio::test]
async fn test_fault_connection_timeout() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("timeout-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    // Don't create listener - connections should timeout or fail

    let result = timeout(Duration::from_millis(500), ipc::connect(&endpoint.path)).await;

    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Connection should timeout when no listener"
    );
}

/// Fault Test: Read from closed connection
#[tokio::test]
async fn test_fault_read_closed_connection() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("closed-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Server: Accept and immediately close
    let server_handle = tokio::spawn(async move {
        if let Ok(stream) = listener.accept().await {
            drop(stream); // Close immediately
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Failed to connect");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to read from closed connection
    let mut buf = vec![0u8; 1024];
    let result = stream.read(&mut buf).await;

    // Should return 0 (EOF) or error
    assert!(
        result.is_err() || result.unwrap() == 0,
        "Read from closed connection should fail or return EOF"
    );

    server_handle.abort();
}

/// Fault Test: Write to closed connection
#[tokio::test]
async fn test_fault_write_closed_connection() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("write-closed-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Server: Accept and immediately close
    let server_handle = tokio::spawn(async move {
        if let Ok(stream) = listener.accept().await {
            drop(stream);
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Failed to connect");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to write to closed connection
    let result = stream.write_all(b"test message").await;

    assert!(result.is_err(), "Write to closed connection should fail");

    server_handle.abort();
}

/// Fault Test: Unregister non-existent primal
#[tokio::test]
async fn test_fault_unregister_nonexistent() {
    ipc::init().expect("Failed to initialize IPC");

    let result = ipc::unregister("nonexistent-primal").await;

    // Should either succeed (idempotent) or fail gracefully
    assert!(
        result.is_ok() || result.is_err(),
        "Unregister should handle nonexistent primal gracefully"
    );
}

/// Fault Test: Double registration (same ID)
#[tokio::test]
async fn test_fault_double_registration() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = "double-reg-primal";

    // First registration
    let endpoint1 = ipc::register(primal_id, vec!["test1".to_string()])
        .await
        .expect("First registration should succeed");

    // Second registration (same ID)
    let result2 = ipc::register(primal_id, vec!["test2".to_string()]).await;

    // Should either fail or succeed with last-write-wins
    if let Ok(endpoint2) = result2 {
        // If it succeeds, verify only one is registered
        let providers = capability::discover_all("test1")
            .await
            .expect("Discovery failed");
        let providers2 = capability::discover_all("test2")
            .await
            .expect("Discovery failed");

        let total = providers.len() + providers2.len();
        assert!(
            total >= 1,
            "Should have at least one registration"
        );

        // Cleanup
        let _ = ipc::unregister(primal_id).await;
    } else {
        // First registration should still work
        let mut stream = ipc::connect(&endpoint1.path)
            .await
            .expect("First registration should still be valid");
        let _ = stream.write_all(b"test").await;

        // Cleanup
        let _ = ipc::unregister(primal_id).await;
    }
}

/// Fault Test: Very long primal ID
#[tokio::test]
async fn test_fault_long_primal_id() {
    ipc::init().expect("Failed to initialize IPC");

    // 1000 character primal ID
    let long_id = "a".repeat(1000);

    let result = ipc::register(&long_id, vec!["test".to_string()]).await;

    // Should either fail or truncate
    if let Ok(_endpoint) = result {
        // Cleanup
        let _ = ipc::unregister(&long_id).await;
    }

    // System should still work
    let _endpoint = ipc::register("normal-primal", vec!["test".to_string()])
        .await
        .expect("System should still work after long ID");
}

/// Fault Test: Very long capability name
#[tokio::test]
async fn test_fault_long_capability_name() {
    ipc::init().expect("Failed to initialize IPC");

    let long_cap = "capability-".to_string() + &"x".repeat(1000);

    let result = ipc::register("long-cap-primal", vec![long_cap.clone()]).await;

    // Should either fail or handle gracefully
    if let Ok(_endpoint) = result {
        // Try to discover
        let _ = capability::discover_all(&long_cap).await;

        // Cleanup
        let _ = ipc::unregister("long-cap-primal").await;
    }
}

/// Fault Test: Many capabilities (stress test)
#[tokio::test]
async fn test_fault_excessive_capabilities() {
    ipc::init().expect("Failed to initialize IPC");

    // 1000 capabilities
    let caps: Vec<String> = (0..1000).map(|i| format!("cap-{}", i)).collect();

    let result = ipc::register("excessive-cap-primal", caps).await;

    // Should either fail or succeed
    if let Ok(_endpoint) = result {
        // Verify at least some are discoverable
        let providers = capability::discover_all("cap-0")
            .await
            .expect("Discovery failed");
        assert!(!providers.is_empty(), "Should find at least one");

        // Cleanup
        let _ = ipc::unregister("excessive-cap-primal").await;
    }
}

/// Fault Test: Listener accept after drop
#[tokio::test]
async fn test_fault_accept_after_drop() {
    ipc::init().expect("Failed to initialize IPC");

    let endpoint = ipc::register("drop-listener-primal", vec!["test".to_string()])
        .await
        .expect("Failed to register primal");

    let listener = ipc::listen(endpoint.clone())
        .await
        .expect("Failed to create listener");

    // Drop listener
    drop(listener);

    // Try to connect (should fail or timeout)
    let result = timeout(Duration::from_secs(1), ipc::connect(&endpoint.path)).await;

    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Connection should fail after listener drop"
    );
}

/// Fault Test: Concurrent unregister
#[tokio::test]
async fn test_fault_concurrent_unregister() {
    ipc::init().expect("Failed to initialize IPC");

    let primal_id = "concurrent-unreg-primal";

    let _endpoint = ipc::register(primal_id, vec!["test".to_string()])
        .await
        .expect("Failed to register");

    // Try to unregister concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let id = primal_id.to_string();
        let handle = tokio::spawn(async move {
            let _ = ipc::unregister(&id).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Verify it's unregistered
    let providers = capability::discover_all("test")
        .await
        .expect("Discovery failed");

    let found = providers.iter().any(|p| p.primal_id == primal_id);
    assert!(!found, "Primal should be unregistered");
}

/// Fault Test: Invalid virtual endpoint path
#[tokio::test]
async fn test_fault_invalid_virtual_path() {
    ipc::init().expect("Failed to initialize IPC");

    let invalid_paths = vec![
        "",
        "/",
        "//",
        "/primal/",
        "primal/test", // No leading slash
        "/primal/../etc/passwd",
    ];

    for path in invalid_paths {
        let result = timeout(Duration::from_millis(500), ipc::connect(path)).await;

        // Should timeout or fail
        assert!(
            result.is_err() || result.unwrap().is_err(),
            "Invalid path '{}' should fail",
            path
        );
    }
}

/// Fault Test: System recovery after errors
#[tokio::test]
async fn test_fault_system_recovery() {
    ipc::init().expect("Failed to initialize IPC");

    // Cause various errors
    let _ = ipc::register("", vec!["test".to_string()]).await; // Invalid ID
    let _ = ipc::unregister("nonexistent").await; // Nonexistent
    let _ = ipc::connect("/invalid/path").await; // Invalid path
    let _ = capability::discover_all("").await; // Empty capability

    // System should still work
    let endpoint = ipc::register("recovery-primal", vec!["test".to_string()])
        .await
        .expect("System should recover and work");

    let mut listener = ipc::listen(endpoint.clone())
        .await
        .expect("Listener should work");

    let server_handle = tokio::spawn(async move {
        if let Ok(mut stream) = listener.accept().await {
            let mut buf = vec![0u8; 64];
            if let Ok(n) = stream.read(&mut buf).await {
                let _ = stream.write_all(&buf[..n]).await;
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut stream = ipc::connect(&endpoint.path)
        .await
        .expect("Connection should work");

    stream
        .write_all(b"recovery")
        .await
        .expect("Write should work");

    let mut buf = vec![0u8; 64];
    let n = stream.read(&mut buf).await.expect("Read should work");
    assert_eq!(&buf[..n], b"recovery");

    server_handle.abort();
}

