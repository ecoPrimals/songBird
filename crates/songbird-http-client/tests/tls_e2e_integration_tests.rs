// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::uninlined_format_args,
    clippy::unused_async,
    reason = "E2E TLS integration harness builds wire-format messages and mock servers"
)]

//! TLS End-to-End Integration Tests
//!
//! These tests verify complete TLS handshake flows with mock servers
//! simulating real-world scenarios.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, sleep};

/// Helper: Mock TLS server that follows protocol
async fn mock_tls_server(port: u16) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();

        let (mut socket, _) = listener.accept().await.unwrap();

        // Read ClientHello
        let mut header = [0u8; 5];
        socket.read_exact(&mut header).await.unwrap();

        assert_eq!(header[0], 0x16, "Expected Handshake record");

        let length = u16::from_be_bytes([header[3], header[4]]) as usize;
        let mut client_hello = vec![0u8; length];
        socket.read_exact(&mut client_hello).await.unwrap();

        // Send ServerHello
        let mut server_hello = vec![];
        server_hello.push(0x16); // Handshake
        server_hello.extend_from_slice(&[0x03, 0x03]); // TLS 1.2

        // Handshake message
        let mut hs_msg = vec![];
        hs_msg.push(0x02); // ServerHello
        hs_msg.extend_from_slice(&[0x00, 0x00, 0x50]); // Length placeholder
        hs_msg.extend_from_slice(&[0x03, 0x03]); // Version
        hs_msg.extend_from_slice(&[0x42u8; 32]); // Server random
        hs_msg.push(0x00); // Session ID length
        hs_msg.extend_from_slice(&[0x13, 0x01]); // Cipher suite
        hs_msg.push(0x00); // Compression

        // Extensions with key_share
        let mut extensions = vec![];
        extensions.extend_from_slice(&[0x00, 0x33]); // key_share
        extensions.extend_from_slice(&[0x00, 0x24]); // Length: 36
        extensions.extend_from_slice(&[0x00, 0x1d]); // x25519
        extensions.extend_from_slice(&[0x00, 0x20]); // Key length: 32
        extensions.extend_from_slice(&[0x88u8; 32]); // Server public key

        hs_msg.extend_from_slice(&(extensions.len() as u16).to_be_bytes());
        hs_msg.extend_from_slice(&extensions);

        // Fix length
        let hs_len = hs_msg.len() - 4;
        hs_msg[1] = ((hs_len >> 16) & 0xFF) as u8;
        hs_msg[2] = ((hs_len >> 8) & 0xFF) as u8;
        hs_msg[3] = (hs_len & 0xFF) as u8;

        server_hello.extend_from_slice(&(hs_msg.len() as u16).to_be_bytes());
        server_hello.extend_from_slice(&hs_msg);

        socket.write_all(&server_hello).await.unwrap();
        socket.flush().await.unwrap();

        // Send mock encrypted messages (ChangeCipherSpec + encrypted handshake)
        let ccs = vec![0x14, 0x03, 0x03, 0x00, 0x01, 0x01]; // ChangeCipherSpec
        socket.write_all(&ccs).await.unwrap();

        // Encrypted handshake messages (mock)
        for _ in 0..3 {
            let encrypted = vec![0x17, 0x03, 0x03, 0x00, 0x20]; // ApplicationData header
            socket.write_all(&encrypted).await.unwrap();
            socket.write_all(&[0x99u8; 32]).await.unwrap(); // Encrypted payload
            sleep(Duration::from_millis(50)).await;
        }

        socket.flush().await.unwrap();

        // Keep connection open
        sleep(Duration::from_secs(2)).await;
    })
}

#[tokio::test]
#[ignore = "integration test requires setup"]
async fn test_complete_tls_handshake_flow() {
    let port = 18443;
    let _server = mock_tls_server(port).await;

    sleep(Duration::from_millis(100)).await;

    // Connect as client
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();

    // Send ClientHello
    let client_hello = build_minimal_client_hello();
    stream.write_all(&client_hello).await.unwrap();
    stream.flush().await.unwrap();

    // Read ServerHello
    let mut header = [0u8; 5];
    let result = tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut header)).await;

    assert!(result.is_ok(), "Should receive ServerHello");

    let length = u16::from_be_bytes([header[3], header[4]]) as usize;
    let mut server_hello = vec![0u8; length];
    stream.read_exact(&mut server_hello).await.unwrap();

    assert_eq!(server_hello[0], 0x02, "Should be ServerHello");

    // Read post-handshake messages
    let mut messages_read = 0;
    for _ in 0..5 {
        let mut msg_header = [0u8; 5];
        let result =
            tokio::time::timeout(Duration::from_millis(500), stream.read_exact(&mut msg_header))
                .await;

        if result.is_err() {
            break;
        }

        let msg_len = u16::from_be_bytes([msg_header[3], msg_header[4]]) as usize;
        let mut msg_data = vec![0u8; msg_len];
        stream.read_exact(&mut msg_data).await.unwrap();

        messages_read += 1;
    }

    assert!(messages_read >= 3, "Should read post-handshake messages");
}

#[tokio::test]
#[ignore = "integration test"]
async fn test_client_hello_format_validation() {
    let client_hello = build_minimal_client_hello();

    // Verify structure
    assert_eq!(client_hello[0], 0x16, "Should be Handshake record");
    assert_eq!(client_hello[1], 0x03, "Should be TLS 1.2 version (compat)");
    assert_eq!(client_hello[2], 0x03, "Should be TLS 1.2 version");
    assert_eq!(client_hello[5], 0x01, "Should be ClientHello message");

    // Verify minimum size (> 40 bytes)
    assert!(client_hello.len() > 40, "ClientHello should be substantial");
}

#[tokio::test]
#[ignore = "integration test"]
async fn test_multiple_handshakes_sequential() {
    let port = 18444;

    for i in 0..3 {
        let _server = mock_tls_server(port + i).await;
        sleep(Duration::from_millis(100)).await;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port + i)).await.unwrap();

        stream.write_all(&build_minimal_client_hello()).await.unwrap();
        stream.flush().await.unwrap();

        let mut header = [0u8; 5];
        let result =
            tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut header)).await;

        assert!(result.is_ok(), "Handshake {} should succeed", i);
    }
}

/// Helper: Build minimal valid `ClientHello`
fn build_minimal_client_hello() -> Vec<u8> {
    let mut hello = vec![];

    // TLS Record Header
    hello.push(0x16); // Handshake
    hello.extend_from_slice(&[0x03, 0x03]); // TLS 1.2 (legacy)

    // Handshake Message
    let mut hs = vec![];
    hs.push(0x01); // ClientHello
    hs.extend_from_slice(&[0x00, 0x00, 0x00]); // Length placeholder
    hs.extend_from_slice(&[0x03, 0x03]); // Version
    hs.extend_from_slice(&[0x11u8; 32]); // Random
    hs.push(0x00); // Session ID length
    hs.extend_from_slice(&[0x00, 0x02, 0x13, 0x01]); // Cipher suites
    hs.push(0x01); // Compression methods length
    hs.push(0x00); // No compression

    // Extensions
    let mut ext = vec![];
    // SNI
    ext.extend_from_slice(&[0x00, 0x00]); // SNI type
    let sni = b"example.com";
    ext.extend_from_slice(&((sni.len() + 5) as u16).to_be_bytes());
    ext.extend_from_slice(&((sni.len() + 3) as u16).to_be_bytes());
    ext.push(0x00); // host_name
    ext.extend_from_slice(&(sni.len() as u16).to_be_bytes());
    ext.extend_from_slice(sni);

    hs.extend_from_slice(&(ext.len() as u16).to_be_bytes());
    hs.extend_from_slice(&ext);

    // Fix handshake length
    let hs_len = hs.len() - 4;
    hs[1] = ((hs_len >> 16) & 0xFF) as u8;
    hs[2] = ((hs_len >> 8) & 0xFF) as u8;
    hs[3] = (hs_len & 0xFF) as u8;

    // Add to record
    hello.extend_from_slice(&(hs.len() as u16).to_be_bytes());
    hello.extend_from_slice(&hs);

    hello
}

#[cfg(test)]
mod e2e_scenarios {
    use super::*;

    #[tokio::test]
    #[ignore = "Integration: artificial TCP/TLS delay scenario; run with --ignored"]
    async fn test_handshake_with_delays() {
        // Server that responds slowly
        let port = 18445;
        tokio::spawn(async move {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
            let (mut socket, _) = listener.accept().await.unwrap();

            // Read ClientHello
            let mut buf = vec![0u8; 1024];
            let _ = socket.read(&mut buf).await;

            // Wait before responding
            sleep(Duration::from_millis(500)).await;

            // Send minimal ServerHello
            socket.write_all(&[0x16, 0x03, 0x03, 0x00, 0x02, 0x02, 0x00]).await.ok();
        });

        sleep(Duration::from_millis(100)).await;

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
        stream.write_all(&build_minimal_client_hello()).await.unwrap();

        let mut buf = vec![0u8; 1024];
        let result = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut buf)).await;

        assert!(result.is_ok(), "Should handle delayed response");
    }
}
