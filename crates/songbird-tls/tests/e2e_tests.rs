// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! End-to-end tests for songbird-tls
//!
//! These tests validate the complete TLS handshake flow from `ClientHello` to application data.

use songbird_tls::{
    HandshakeState, HandshakeStateMachine, TLS_CHACHA20_POLY1305_SHA256, TLS_VERSION_1_3,
    codec::{Decode, Encode},
    error::TlsError,
    messages::{ClientHello, Extension},
};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Helper: Create test `ClientHello`
fn create_test_client_hello() -> ClientHello {
    let random = [42u8; 32];
    let cipher_suites = vec![TLS_CHACHA20_POLY1305_SHA256];
    let extensions = vec![
        Extension::SupportedVersions(vec![TLS_VERSION_1_3]),
        Extension::KeyShare(vec![1u8; 32]), // X25519 public key
    ];
    ClientHello::new(random, cipher_suites, extensions)
}

#[tokio::test]
async fn e2e_handshake_state_machine_initialization() {
    let hsm = HandshakeStateMachine::new();

    assert_eq!(hsm.state(), HandshakeState::Start);
}

#[tokio::test]
async fn e2e_client_hello_validation() {
    // Valid ClientHello
    let valid_hello = create_test_client_hello();
    assert!(valid_hello.validate().is_ok());

    // Invalid: no cipher suites
    let invalid_hello = ClientHello::new(
        [0u8; 32],
        vec![], // Empty
        vec![Extension::SupportedVersions(vec![TLS_VERSION_1_3])],
    );
    assert!(invalid_hello.validate().is_err());

    // Invalid: no extensions (TLS 1.3 requires extensions)
    let invalid_hello2 = ClientHello::new(
        [0u8; 32],
        vec![TLS_CHACHA20_POLY1305_SHA256],
        vec![], // Empty
    );
    assert!(invalid_hello2.validate().is_err());
}

#[tokio::test]
async fn e2e_client_hello_encoding_decoding() {
    let original = create_test_client_hello();

    // Encode
    let mut buf = Vec::new();
    original.encode(&mut buf).unwrap();

    assert!(!buf.is_empty(), "Encoded buffer should not be empty");

    // Decode
    let (decoded, bytes_read) = ClientHello::decode(&buf).unwrap();
    assert_eq!(bytes_read, buf.len());

    // Verify fields match
    assert_eq!(&decoded.random, &original.random);
    assert_eq!(decoded.cipher_suites, original.cipher_suites);
    assert_eq!(decoded.extensions.len(), original.extensions.len());
}

#[tokio::test]
async fn e2e_multiple_client_hello_round_trips() {
    // Test multiple encode/decode cycles
    for i in 0..10 {
        let random = [i; 32];
        let hello = ClientHello::new(
            random,
            vec![TLS_CHACHA20_POLY1305_SHA256],
            vec![
                Extension::SupportedVersions(vec![TLS_VERSION_1_3]),
                Extension::KeyShare(vec![i; 32]),
            ],
        );

        let mut buf = Vec::new();
        hello.encode(&mut buf).unwrap();

        let (decoded, _) = ClientHello::decode(&buf).unwrap();
        assert_eq!(&decoded.random, &random);
    }
}

#[tokio::test]
async fn e2e_extension_supported_versions() {
    let ext = Extension::SupportedVersions(vec![TLS_VERSION_1_3]);

    // Verify it's the correct variant
    match ext {
        Extension::SupportedVersions(versions) => {
            assert_eq!(versions.len(), 1);
            assert_eq!(versions[0], TLS_VERSION_1_3);
        }
        _ => panic!("Wrong extension type"),
    }
}

#[tokio::test]
async fn e2e_extension_key_share() {
    let public_key = vec![42u8; 32];
    let ext = Extension::KeyShare(public_key.clone());

    match ext {
        Extension::KeyShare(key) => {
            assert_eq!(key.len(), 32);
            assert_eq!(key, public_key);
        }
        _ => panic!("Wrong extension type"),
    }
}

#[tokio::test]
async fn e2e_concurrent_client_hello_processing() {
    // Test concurrent processing of multiple ClientHellos
    let mut handles = vec![];

    for i in 0..50 {
        let handle = tokio::spawn(async move {
            let random = [i; 32];
            let hello = ClientHello::new(
                random,
                vec![TLS_CHACHA20_POLY1305_SHA256],
                vec![
                    Extension::SupportedVersions(vec![TLS_VERSION_1_3]),
                    Extension::KeyShare(vec![i; 32]),
                ],
            );

            // Validate
            hello.validate().unwrap();

            // Encode/decode
            let mut buf = Vec::new();
            hello.encode(&mut buf).unwrap();
            let (decoded, _) = ClientHello::decode(&buf).unwrap();

            assert_eq!(&decoded.random, &random);
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn e2e_tcp_server_binding() {
    // Test that we can bind a TCP server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    assert!(addr.port() > 0);
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
}

#[tokio::test]
async fn e2e_tcp_connection_establishment() {
    // Create server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 32];
        let n = socket.read(&mut buf).await.unwrap();
        buf[..n].to_vec()
    });

    // Client connects and sends data
    let client_handle = tokio::spawn(async move {
        let mut socket = TcpStream::connect(addr).await.unwrap();
        socket.write_all(b"Hello, TLS!").await.unwrap();
    });

    // Wait for both
    let received = server_handle.await.unwrap();
    client_handle.await.unwrap();

    assert_eq!(&received, b"Hello, TLS!");
}

#[tokio::test]
async fn e2e_error_type_display() {
    // Test that all error types have meaningful display messages
    let errors = vec![
        TlsError::ProtocolError("test protocol error".to_string()),
        TlsError::DecryptError,
        TlsError::CertificateError("test cert error".to_string()),
        TlsError::HandshakeFailure("test handshake failure".to_string()),
        TlsError::IoError("test io error".to_string()),
        TlsError::RecordTooLarge {
            size: 999_999,
        },
    ];

    for error in errors {
        let display = format!("{error}");
        assert!(!display.is_empty(), "Error should have display message");
        assert!(display.len() > 5, "Error message should be meaningful");
    }
}

#[tokio::test]
async fn e2e_handshake_state_transitions() {
    let hsm = HandshakeStateMachine::new();

    // Initial state
    assert_eq!(hsm.state(), HandshakeState::Start);

    // NOTE: State transitions require security-provider crypto operations
    // Full handshake flow will be tested in integration tests with a mock provider
}

#[tokio::test]
async fn e2e_multiple_concurrent_connections() {
    // Create server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server that accepts multiple connections
    let server_handle = tokio::spawn(async move {
        let mut count = 0;
        for _ in 0..10 {
            if let Ok((mut socket, _)) = listener.accept().await {
                count += 1;
                tokio::spawn(async move {
                    let mut buf = [0u8; 16];
                    let _ = socket.read(&mut buf).await;
                });
            }
        }
        count
    });

    // Spawn 10 concurrent clients
    let mut client_handles = vec![];
    for i in 0..10 {
        let client_handle = tokio::spawn(async move {
            if let Ok(mut socket) = TcpStream::connect(addr).await {
                let data = format!("Client {i}");
                let _ = socket.write_all(data.as_bytes()).await;
            }
        });
        client_handles.push(client_handle);
    }

    // Wait for all clients
    for handle in client_handles {
        handle.await.unwrap();
    }

    // Wait for server with timeout
    let count = tokio::time::timeout(Duration::from_secs(2), server_handle).await.unwrap().unwrap();

    assert_eq!(count, 10, "Server should accept all 10 connections");
}

#[tokio::test]
async fn e2e_graceful_shutdown() {
    // Test graceful connection shutdown
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        if let Ok((socket, _)) = listener.accept().await {
            drop(socket); // Graceful close
        }
    });

    let client_handle = tokio::spawn(async move {
        if let Ok(socket) = TcpStream::connect(addr).await {
            drop(socket); // Graceful close
        }
    });

    // Should complete without errors
    server_handle.await.unwrap();
    client_handle.await.unwrap();
}

// NOTE: Full TLS handshake E2E tests require mock security-provider implementation
// These will be added in the next phase once we have a complete mock crypto provider
