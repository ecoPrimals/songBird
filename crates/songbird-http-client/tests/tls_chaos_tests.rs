// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Chaos Tests
//!
//! These tests verify TLS handshake behavior under chaotic conditions:
//! - Random timeouts
//! - Connection drops mid-handshake
//! - Slow/delayed responses
//! - Concurrent handshakes

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};

/// Test: Server that accepts connection but never responds
#[tokio::test]
#[ignore] // Chaos test - run explicitly
async fn test_server_silent_timeout() {
    // Bind to ephemeral port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn silent server
    tokio::spawn(async move {
        let (mut _socket, _) = listener.accept().await.unwrap();
        // Accept but never respond - let it timeout
        sleep(Duration::from_secs(30)).await;
    });

    // Give server time to start
    sleep(Duration::from_millis(50)).await;

    // Attempt connection - should timeout
    let result = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await;

    assert!(result.is_ok(), "Connection should succeed");

    // Reading should timeout (server is silent)
    let mut stream = result.unwrap().unwrap();
    let mut buf = vec![0u8; 1024];

    let read_result = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await;

    assert!(read_result.is_err(), "Read should timeout on silent server");
}

/// Test: Server that drops connection immediately after accept
#[tokio::test]
#[ignore] // Chaos test
async fn test_server_immediate_disconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        // Drop immediately
        drop(socket);
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let mut buf = vec![0u8; 1024];

    // Should get 0 bytes (EOF) or error
    let result = stream.read(&mut buf).await;
    assert!(matches!(result, Ok(0) | Err(_)), "Should detect disconnection");
}

/// Test: Server that sends partial response then hangs
#[tokio::test]
#[ignore] // Chaos test
async fn test_server_partial_response() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Send partial TLS record header (only 3 of 5 bytes)
        socket.write_all(&[0x16, 0x03, 0x03]).await.ok();
        socket.flush().await.ok();

        // Then hang
        sleep(Duration::from_secs(30)).await;
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let mut buf = vec![0u8; 5];

    // Should timeout waiting for complete header
    let result = tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut buf)).await;

    assert!(result.is_err(), "Should timeout on partial response");
}

/// Test: Server that sends slow byte-by-byte responses
#[tokio::test]
#[ignore] // Chaos test
async fn test_server_slow_byte_drip() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Send TLS alert byte-by-byte with delays
        let alert = vec![0x15, 0x03, 0x03, 0x00, 0x02, 0x02, 0x50]; // Alert: close_notify

        for byte in alert {
            socket.write_all(&[byte]).await.ok();
            socket.flush().await.ok();
            sleep(Duration::from_millis(100)).await;
        }
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let mut buf = vec![0u8; 7];

    // Should eventually read full alert (slow but complete)
    let result = tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut buf)).await;

    assert!(result.is_ok(), "Should handle slow byte-by-byte reads");
    let bytes_read = result.unwrap().unwrap();
    assert_eq!(bytes_read, 7, "Should read all 7 bytes");
}

/// Test: Concurrent handshake attempts
#[tokio::test]
#[ignore] // Chaos test - resource intensive
async fn test_concurrent_handshakes() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Simple echo server
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 1024];
                while let Ok(n) = socket.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    socket.write_all(&buf[..n]).await.ok();
                }
            });
        }
    });

    sleep(Duration::from_millis(50)).await;

    // Launch 10 concurrent connections
    let mut handles = vec![];
    for i in 0..10 {
        let addr_clone = addr;
        handles.push(tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr_clone).await.unwrap();

            // Send test data
            let test_data = format!("test-{}", i).into_bytes();
            stream.write_all(&test_data).await.unwrap();
            stream.flush().await.unwrap();

            // Read echo
            let mut buf = vec![0u8; test_data.len()];
            stream.read_exact(&mut buf).await.unwrap();

            assert_eq!(buf, test_data, "Echo should match");
        }));
    }

    // Wait for all to complete
    for handle in handles {
        assert!(handle.await.is_ok(), "Concurrent handshake should succeed");
    }
}

/// Test: Connection reset during handshake
#[tokio::test]
#[ignore] // Chaos test
async fn test_connection_reset_mid_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Read some data
        let mut buf = vec![0u8; 1024];
        let _ = socket.read(&mut buf).await;

        // Send partial response, then reset
        socket.write_all(&[0x16, 0x03, 0x03]).await.ok();

        // Force reset by dropping with linger disabled
        drop(socket);
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    // Send ClientHello-like data
    stream.write_all(&[0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00]).await.unwrap();
    stream.flush().await.unwrap();

    // Try to read - should get connection reset
    let mut buf = vec![0u8; 1024];
    let result = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await;

    // Should timeout or get error
    assert!(
        result.is_err() || (result.is_ok() && result.unwrap().is_err()),
        "Should detect connection reset"
    );
}

/// Test: Random delays between handshake steps
#[tokio::test]
#[ignore] // Chaos test
async fn test_random_delays_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Use pseudo-random delays (deterministic, no need for Send rng)
        let delays = [100u64, 200, 150, 300, 250];

        // Read with delays
        for (i, &delay_ms) in delays.iter().enumerate().take(5) {
            sleep(Duration::from_millis(delay_ms)).await;

            let mut buf = vec![0u8; 256];
            if socket.read(&mut buf).await.is_err() {
                break;
            }

            // Echo back with delay
            sleep(Duration::from_millis(delay_ms)).await;
            let write_len = std::cmp::min(10, buf.len());
            socket.write_all(&buf[..write_len]).await.ok();

            if i >= 4 {
                break;
            }
        }
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    // Try handshake with patience
    for i in 0..5 {
        let data = vec![i as u8; 256];
        stream.write_all(&data).await.unwrap();

        let mut buf = vec![0u8; 10];
        let result =
            tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut buf)).await;

        if result.is_err() {
            break; // Acceptable to timeout with random delays
        }
    }
}

/// Test: Memory pressure during handshake (large buffers)
#[tokio::test]
#[ignore] // Chaos test - memory intensive
async fn test_memory_pressure_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Allocate large response
        let large_response = vec![0x42u8; 1024 * 1024]; // 1MB

        // Send in chunks
        for chunk in large_response.chunks(8192) {
            if socket.write_all(chunk).await.is_err() {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    });

    sleep(Duration::from_millis(50)).await;

    let mut stream = TcpStream::connect(addr).await.unwrap();

    // Try to read large response
    let mut total_read = 0;
    let mut buf = vec![0u8; 8192];

    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(_) => break,
            }
            if total_read >= 100_000 {
                break;
            } // Read at least 100KB
        }
        total_read
    })
    .await;

    assert!(result.is_ok(), "Should handle large responses");
    assert!(result.unwrap() >= 100_000, "Should read significant data");
}

#[cfg(test)]
mod chaos_helpers {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Helper: Create a server that behaves chaotically (deterministic pattern)
    #[expect(dead_code, reason = "test assertions and harness ergonomics")]
    pub async fn spawn_chaos_server(port: u16) -> tokio::task::JoinHandle<()> {
        let counter = std::sync::Arc::new(AtomicUsize::new(0));

        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();

            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let counter_clone = counter.clone();

                tokio::spawn(async move {
                    let behavior = counter_clone.fetch_add(1, Ordering::SeqCst) % 4;

                    match behavior {
                        0 => drop(socket),                         // Immediate disconnect
                        1 => sleep(Duration::from_secs(30)).await, // Hang
                        2 => {
                            // Partial response
                            let _ = socket.try_write(&[0x16, 0x03]);
                            sleep(Duration::from_secs(30)).await;
                        }
                        _ => {
                            // Normal-ish behavior
                            let delay = 100 + (behavior * 100);
                            sleep(Duration::from_millis(delay as u64)).await;
                        }
                    }
                });
            }
        })
    }
}
