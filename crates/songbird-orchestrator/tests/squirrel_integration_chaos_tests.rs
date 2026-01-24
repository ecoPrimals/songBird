//! Chaos Tests: Squirrel Integration - Stress & Edge Cases
//!
//! Tests extreme conditions and stress scenarios:
//! - Concurrent request storms
//! - Rapid connection/disconnection
//! - Resource exhaustion
//! - Race conditions
//!
//! **Evolution**: 10 sleeps → event-driven (ReadyNotifier)!

mod common;
use common::event_helpers::ReadyNotifier;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// Helper: Start mock server with configurable behavior + ready notification
async fn start_chaos_server(
    socket_path: &str,
    delay_ms: u64,
    notifier: ReadyNotifier,
) -> tokio::task::JoinHandle<()> {
    let socket_path = socket_path.to_string();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).unwrap();
        notifier.signal_ready(); // ✅ Signal ready immediately after bind

        while let Ok((mut stream, _)) = listener.accept().await {
            let delay = delay_ms;
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();

                if reader.read_line(&mut line).await.is_ok() {
                    // Simulate processing delay (LEGITIMATE chaos timing)
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }

                    if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
                        let response = json!({
                            "jsonrpc": "2.0",
                            "result": {"status": "ok"},
                            "id": request["id"]
                        });

                        let _ = stream
                            .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                            .await;
                        let _ = stream.write_all(b"\n").await;
                    }
                }
            });
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_request_storm() {
    // Fire 100 concurrent requests

    let socket_path = "/tmp/test-songbird-chaos-storm.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    let mut join_set = JoinSet::new();

    for i in 0..100 {
        let path = socket_path.to_string();
        join_set.spawn(async move {
            let mut stream = UnixStream::connect(&path).await?;

            let request = json!({
                "jsonrpc": "2.0",
                "method": "discover_capabilities",
                "params": {},
                "id": i
            });

            stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
            stream.write_all(b"\n").await?;
            stream.flush().await?;

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            let response: serde_json::Value = serde_json::from_str(&line)?;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(response)
        });
    }

    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        if result.is_ok() && result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    // At least 95% should succeed
    assert!(success_count >= 95, "Only {} out of 100 requests succeeded", success_count);

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_rapid_connect_disconnect() {
    // Rapidly connect and disconnect 50 times

    let socket_path = "/tmp/test-songbird-chaos-rapid.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    for _ in 0..50 {
        let stream = UnixStream::connect(socket_path).await;
        assert!(stream.is_ok());
        drop(stream); // Immediately close
    }

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_connection_churn() {
    // Connect, send request, disconnect - 50 times concurrently

    let socket_path = "/tmp/test-songbird-chaos-churn.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    let mut join_set = JoinSet::new();

    for i in 0..50 {
        let path = socket_path.to_string();
        join_set.spawn(async move {
            for _ in 0..3 {
                let mut stream = UnixStream::connect(&path).await?;

                let request = json!({
                    "jsonrpc": "2.0",
                    "method": "discover_capabilities",
                    "params": {},
                    "id": i
                });

                stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
                stream.write_all(b"\n").await?;
                stream.flush().await?;

                drop(stream); // Close without reading response
            }
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        });
    }

    while let Some(result) = join_set.join_next().await {
        assert!(result.is_ok());
    }

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_mixed_methods() {
    // Send mix of discover_capabilities and http.request concurrently

    let socket_path = "/tmp/test-songbird-chaos-mixed.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    let mut join_set = JoinSet::new();

    for i in 0..50 {
        let path = socket_path.to_string();
        let method = if i % 2 == 0 {
            "discover_capabilities"
        } else {
            "http.request"
        };

        join_set.spawn(async move {
            let mut stream = UnixStream::connect(&path).await?;

            let params = if method == "http.request" {
                json!({
                    "method": "GET",
                    "url": "https://httpbin.org/get",
                    "headers": {}
                })
            } else {
                json!({})
            };

            let request = json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": i
            });

            stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
            stream.write_all(b"\n").await?;
            stream.flush().await?;

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        });
    }

    while let Some(result) = join_set.join_next().await {
        assert!(result.is_ok());
    }

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_slow_server_timeouts() {
    // Server with 5s delay, client should timeout gracefully

    let socket_path = "/tmp/test-songbird-chaos-slow.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 5000, notifier).await; // 5s delay
    ready.notified().await; // ✅ Event-driven! No polling!

    // Try to connect with 1s timeout
    let result = tokio::time::timeout(Duration::from_secs(1), async {
        let mut stream = UnixStream::connect(socket_path).await?;

        let request = json!({
            "jsonrpc": "2.0",
            "method": "discover_capabilities",
            "params": {},
            "id": 1
        });

        stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await;

    // Should timeout
    assert!(result.is_err());

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_large_payload() {
    // Send very large HTTP request body

    let socket_path = "/tmp/test-songbird-chaos-large.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    // Create large payload (1MB of data)
    let large_text = "A".repeat(1024 * 1024);

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "http.request",
        "params": {
            "method": "POST",
            "url": "https://httpbin.org/post",
            "headers": {"content-type": "text/plain"},
            "body": large_text
        },
        "id": 1
    });

    let result = stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await;
    assert!(result.is_ok());

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_connection_limit() {
    // Open 200 connections simultaneously

    let socket_path = "/tmp/test-songbird-chaos-limit.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    let semaphore = Arc::new(Semaphore::new(200));
    let mut join_set = JoinSet::new();

    for _ in 0..200 {
        let path = socket_path.to_string();
        let sem = semaphore.clone();

        join_set.spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let stream = UnixStream::connect(&path).await;
            tokio::time::sleep(Duration::from_millis(100)).await; // LEGITIMATE: Hold connection (chaos test)
            stream
        });
    }

    let mut success_count = 0;
    while let Some(result) = join_set.join_next().await {
        if result.is_ok() && result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    // Most should succeed
    assert!(success_count >= 150, "Only {} out of 200 connections succeeded", success_count);

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_malformed_json() {
    // Send malformed JSON repeatedly

    let socket_path = "/tmp/test-songbird-chaos-malformed.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    for _ in 0..10 {
        let mut stream = UnixStream::connect(socket_path).await.unwrap();

        // Send malformed JSON
        let _ = stream.write_all(b"{invalid json}\n").await;
        let _ = stream.flush().await;

        // Server should not crash
        drop(stream);
    }

    // Server should still accept valid connections
    let stream = UnixStream::connect(socket_path).await;
    assert!(stream.is_ok());

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_rapid_method_switching() {
    // Rapidly switch between methods with new connection each time

    let socket_path = "/tmp/test-songbird-chaos-switch.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    for i in 0..10 {
        let mut stream = UnixStream::connect(socket_path).await.unwrap();

        let method = if i % 2 == 0 {
            "discover_capabilities"
        } else {
            "http.request"
        };

        let params = if method == "http.request" {
            json!({"method": "GET", "url": "https://httpbin.org/get", "headers": {}})
        } else {
            json!({})
        };

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": i
        });

        let result = stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await;
        assert!(result.is_ok(), "Write failed at iteration {}", i);
        let result = stream.write_all(b"\n").await;
        assert!(result.is_ok(), "Newline write failed at iteration {}", i);
        let result = stream.flush().await;
        assert!(result.is_ok(), "Flush failed at iteration {}", i);
    }

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_chaos_zero_byte_writes() {
    // Send zero-byte writes

    let socket_path = "/tmp/test-songbird-chaos-zero.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, ready) = ReadyNotifier::new();
    let server = start_chaos_server(socket_path, 0, notifier).await;
    ready.notified().await; // ✅ Event-driven! No polling!

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    for _ in 0..10 {
        let result = stream.write_all(b"").await;
        assert!(result.is_ok());
    }

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}
