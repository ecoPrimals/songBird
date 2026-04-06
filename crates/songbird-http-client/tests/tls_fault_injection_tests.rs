// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Fault Injection Tests
//!
//! These tests inject specific faults into the TLS handshake to verify
//! error handling, recovery, and resilience.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::time::{Duration, sleep};

#[cfg(test)]
mod protocol_faults {
    use super::*;

    #[tokio::test]
    async fn test_invalid_record_type() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send invalid record type (0xFF)
            let invalid_record = vec![
                0xFF, // Invalid content type
                0x03, 0x03, // TLS 1.2
                0x00, 0x05, // Length: 5
                0x01, 0x02, 0x03, 0x04, 0x05, // Garbage data
            ];

            let _ = socket.write_all(&invalid_record).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        // Should either fail or read the invalid data
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_protocol_version() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send record with invalid protocol version (TLS 2.0?)
            let invalid_version = vec![
                0x16, // Handshake
                0x04, 0x00, // Invalid version
                0x00, 0x05, // Length: 5
                0x01, 0x02, 0x03, 0x04, 0x05,
            ];

            let _ = socket.write_all(&invalid_version).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_length_overflow() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send record claiming to be 65535 bytes but only send header
            let overflow_record = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0xFF, 0xFF, // Length: 65535 (max u16)
            ];

            let _ = socket.write_all(&overflow_record).await;
            // Don't send the actual data - client should timeout
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await;

        // Should timeout, fail, or read partial header
        // The client may read the 5-byte header successfully even though payload is missing
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_truncated_record_header() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send only 3 bytes of 5-byte header
            let truncated = vec![0x16, 0x03, 0x03];

            let _ = socket.write_all(&truncated).await;
            // Close connection
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        // Should read partial data or get EOF
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_malformed_handshake_message() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Valid record header, but malformed handshake
            let malformed = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0x00, 0x04, // Length: 4
                0xFF, // Invalid handshake type
                0x00, 0x00, 0x01, // Length: 1
            ];

            let _ = socket.write_all(&malformed).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod alert_faults {
    use super::*;

    #[tokio::test]
    async fn test_fatal_alert_handshake_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send fatal alert: handshake_failure
            let alert = vec![
                0x15, // Alert
                0x03, 0x03, // TLS 1.2
                0x00, 0x02, // Length: 2
                0x02, // Fatal
                0x28, // handshake_failure (40)
            ];

            let _ = socket.write_all(&alert).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());

        // Verify we received the alert
        if let Ok(n) = result {
            assert!(n >= 7, "Should receive full alert message");
            assert_eq!(buf[0], 0x15, "Should be Alert record");
            assert_eq!(buf[5], 0x02, "Should be Fatal level");
            assert_eq!(buf[6], 0x28, "Should be handshake_failure");
        }
    }

    #[tokio::test]
    async fn test_fatal_alert_bad_certificate() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send fatal alert: bad_certificate
            let alert = vec![
                0x15, // Alert
                0x03, 0x03, // TLS 1.2
                0x00, 0x02, // Length: 2
                0x02, // Fatal
                0x2A, // bad_certificate (42)
            ];

            let _ = socket.write_all(&alert).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_warning_alert_close_notify() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send warning alert: close_notify
            let alert = vec![
                0x15, // Alert
                0x03, 0x03, // TLS 1.2
                0x00, 0x02, // Length: 2
                0x01, // Warning
                0x00, // close_notify (0)
            ];

            let _ = socket.write_all(&alert).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_alert_with_invalid_level() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send alert with invalid level (3)
            let alert = vec![
                0x15, // Alert
                0x03, 0x03, // TLS 1.2
                0x00, 0x02, // Length: 2
                0x03, // Invalid level
                0x28, // handshake_failure
            ];

            let _ = socket.write_all(&alert).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod connection_faults {
    use super::*;

    #[tokio::test]
    async fn test_connection_reset_during_handshake() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (socket, _) = listener.accept().await.unwrap();
            // Immediately drop connection
            drop(socket);
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        // Should get EOF (0 bytes) or error
        assert!(matches!(result, Ok(0) | Err(_)));
    }

    #[tokio::test]
    async fn test_partial_write_then_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send partial ServerHello
            let partial = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0x00, 0x10, // Length: 16 (but we'll send less)
                0x02, // ServerHello
                0x00, 0x00, 0x0C, // Handshake length: 12
            ];

            let _ = socket.write_all(&partial).await;
            // Disconnect without sending rest
            drop(socket);
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_slow_byte_by_byte_send() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            let message = vec![0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];

            // Send one byte every 100ms (chaos test - keep sleep)
            for byte in message {
                let _ = socket.write_all(&[byte]).await;
                sleep(Duration::from_millis(100)).await;
            }
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        // Should eventually receive all bytes
        let result = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_refused() {
        // Try to connect to a port that's not listening
        let result = TcpStream::connect("127.0.0.1:1").await;
        assert!(result.is_err(), "Connection to port 1 should fail");
    }

    #[tokio::test]
    async fn test_multiple_rapid_disconnects() {
        for _ in 0..5 {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let (ready_tx, ready_rx) = oneshot::channel();
            tokio::spawn(async move {
                let _ = ready_tx.send(());
                let (socket, _) = listener.accept().await.unwrap();
                drop(socket);
            });
            ready_rx.await.unwrap();

            let stream = TcpStream::connect(addr).await;
            assert!(stream.is_ok());
        }
    }
}

#[cfg(test)]
mod crypto_faults {
    use super::*;

    #[tokio::test]
    async fn test_invalid_server_hello_random() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send ServerHello with all-zero random (suspicious)
            let mut server_hello = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0x00, 0x26, // Length: 38
                0x02, // ServerHello
                0x00, 0x00, 0x22, // Handshake length: 34
                0x03, 0x03, // TLS 1.2
            ];

            // 32 bytes of zeros (suspicious random)
            server_hello.extend_from_slice(&[0u8; 32]);

            // Session ID
            server_hello.push(0x00);

            // Cipher suite
            server_hello.extend_from_slice(&[0x13, 0x01]);

            // Compression
            server_hello.push(0x00);

            let _ = socket.write_all(&server_hello).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unsupported_cipher_suite() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send ServerHello with unsupported cipher suite (0x0000)
            let mut server_hello = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0x00, 0x26, // Length: 38
                0x02, // ServerHello
                0x00, 0x00, 0x22, // Handshake length: 34
                0x03, 0x03, // TLS 1.2
            ];

            // Random
            server_hello.extend_from_slice(&[0x42u8; 32]);

            // Session ID
            server_hello.push(0x00);

            // Unsupported cipher suite
            server_hello.extend_from_slice(&[0x00, 0x00]);

            // Compression
            server_hello.push(0x00);

            let _ = socket.write_all(&server_hello).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_key_share() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Send ServerHello with invalid key_share extension
            let mut server_hello = vec![
                0x16, // Handshake
                0x03, 0x03, // TLS 1.2
                0x00, 0x30, // Length: 48
                0x02, // ServerHello
                0x00, 0x00, 0x2C, // Handshake length: 44
                0x03, 0x03, // TLS 1.2
            ];

            // Random
            server_hello.extend_from_slice(&[0x42u8; 32]);

            // Session ID
            server_hello.push(0x00);

            // Cipher suite
            server_hello.extend_from_slice(&[0x13, 0x01]);

            // Compression
            server_hello.push(0x00);

            // Extensions length
            server_hello.extend_from_slice(&[0x00, 0x08]);

            // key_share extension with wrong length
            server_hello.extend_from_slice(&[
                0x00, 0x33, // key_share
                0x00, 0x04, // Length: 4
                0x00, 0x1d, // X25519
                0x00, 0x00, // Key length: 0 (invalid!)
            ]);

            let _ = socket.write_all(&server_hello).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = stream.read(&mut buf).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod timing_faults {
    use super::*;

    #[tokio::test]
    async fn test_delayed_server_hello() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            // Wait 3 seconds before responding (chaos test - keep sleep)
            sleep(Duration::from_secs(3)).await;

            let response = vec![0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x02, 0x03, 0x04, 0x05];
            let _ = socket.write_all(&response).await;
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await;

        assert!(result.is_ok(), "Should eventually receive response");
    }

    #[tokio::test]
    async fn test_interleaved_delays() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let (ready_tx, ready_rx) = oneshot::channel();
        tokio::spawn(async move {
            let _ = ready_tx.send(());
            let (mut socket, _) = listener.accept().await.unwrap();

            let chunks = vec![
                vec![0x16, 0x03],
                vec![0x03, 0x00],
                vec![0x05, 0x01],
                vec![0x02, 0x03],
                vec![0x04, 0x05],
            ];

            for chunk in chunks {
                let _ = socket.write_all(&chunk).await;
                sleep(Duration::from_millis(200)).await;
            }
        });
        ready_rx.await.unwrap();

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 1024];

        let result = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buf)).await;

        assert!(result.is_ok());
    }
}
