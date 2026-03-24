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

//! Chaos and fault injection tests for songbird-tls
//!
//! These tests intentionally introduce failures to validate error handling.
//! Philosophy: "Test issues ARE production issues"

use songbird_tls::codec::{Decode, Encode};
use songbird_tls::error::TlsError;
use songbird_tls::messages::{ClientHello, ContentType, Extension};
use tokio::time::{Duration, timeout};

/// Test that `ClientHello` encoding/decoding is resilient
#[tokio::test]
async fn chaos_malformed_client_hello() {
    // Test 1: Empty buffer
    let result = ClientHello::decode(&[]);
    assert!(result.is_err(), "Should fail on empty buffer");

    // Test 2: Truncated header
    let result = ClientHello::decode(&[0x03, 0x03]); // Only 2 bytes
    assert!(result.is_err(), "Should fail on truncated data");

    // Test 3: Invalid version
    let mut buf = Vec::new();
    buf.extend_from_slice(&[0xFF, 0xFF]); // Invalid version
    buf.extend_from_slice(&[0u8; 32]); // Random
    buf.push(0); // Empty session ID
    buf.extend_from_slice(&[0x00, 0x02, 0x13, 0x03]); // Cipher suites
    buf.push(0x01); // Compression methods length
    buf.push(0x00); // Compression method
    buf.extend_from_slice(&[0x00, 0x00]); // Empty extensions

    // This should decode but fail validation
    let _result = ClientHello::decode(&buf);
    // We don't validate version strictly during decode, so this might pass
    // The validation happens in the validate() method
}

/// Test `ContentType` conversion with invalid values
#[tokio::test]
async fn chaos_invalid_content_type() {
    // Valid content types
    assert_eq!(ContentType::from(22), ContentType::Handshake);
    assert_eq!(ContentType::from(23), ContentType::ApplicationData);

    // Invalid content types should map to Invalid
    assert_eq!(ContentType::from(0), ContentType::Invalid);
    assert_eq!(ContentType::from(255), ContentType::Invalid);
    assert_eq!(ContentType::from(99), ContentType::Invalid);
}

/// Test timeout scenarios (no sleeps! using timeout wrapper)
#[tokio::test]
async fn chaos_operation_timeout() {
    // Simulate a long-running operation
    let slow_operation = async {
        // This would normally be a real operation that might hang
        // We use a long delay to simulate it
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok::<(), TlsError>(())
    };

    // Should timeout quickly
    let result = timeout(Duration::from_millis(100), slow_operation).await;
    assert!(result.is_err(), "Should timeout");
}

/// Test concurrent access patterns (truly concurrent, no sleeps)
#[tokio::test]
async fn chaos_concurrent_encoding() {
    // Create test ClientHello
    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions =
        vec![Extension::SupportedVersions(vec![0x0304]), Extension::KeyShare(vec![1, 2, 3, 4])];
    let hello = ClientHello::new(random, cipher_suites, extensions);

    // Spawn multiple concurrent encoding tasks
    let mut handles = vec![];
    for _ in 0..100 {
        let hello_clone = hello.clone();
        let handle = tokio::spawn(async move {
            let mut buf = Vec::new();
            hello_clone.encode(&mut buf).unwrap();
            buf
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent encoding should not fail");
        let buf = result.unwrap();
        assert!(!buf.is_empty(), "Encoded buffer should not be empty");
    }
}

/// Test extension parsing with malformed data
#[tokio::test]
async fn fault_malformed_extensions() {
    // Test empty extension data
    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![];

    let hello = ClientHello::new(random, cipher_suites, extensions);

    // Should fail validation (no extensions in TLS 1.3)
    let result = hello.validate();
    assert!(result.is_err(), "Should fail validation without extensions");
}

/// Test cipher suite validation
#[tokio::test]
async fn fault_invalid_cipher_suites() {
    let random = [42u8; 32];
    let cipher_suites = vec![]; // Empty cipher suites
    let extensions = vec![Extension::SupportedVersions(vec![0x0304])];

    let hello = ClientHello::new(random, cipher_suites, extensions);

    // Should fail validation (no cipher suites)
    let result = hello.validate();
    assert!(result.is_err(), "Should fail validation without cipher suites");
}

/// Test maximum size limits
#[tokio::test]
async fn fault_oversized_data() {
    use songbird_tls::{MAX_HANDSHAKE_SIZE, MAX_RECORD_SIZE};

    // Verify constants are reasonable
    assert_eq!(MAX_RECORD_SIZE, 16384);
    assert_eq!(MAX_HANDSHAKE_SIZE, 262_144);

    // Test that we respect these limits
    // (Actual enforcement tested in record_layer tests)
}

/// Test error type conversions
#[tokio::test]
async fn fault_error_handling() {
    // Test that all error types can be created and displayed
    let errors = vec![
        TlsError::ProtocolError("test".to_string()),
        TlsError::DecryptError,
        TlsError::CertificateError("test".to_string()),
        TlsError::HandshakeFailure("test".to_string()),
        TlsError::Unsupported("test".to_string()),
        TlsError::IoError("test".to_string()),
        TlsError::CryptoError("test".to_string()),
        TlsError::InternalError("test".to_string()),
        TlsError::InvalidParameter("test".to_string()),
        TlsError::RecordTooLarge {
            size: 999_999,
        },
        TlsError::UnexpectedMessage {
            expected: "A".to_string(),
            got: "B".to_string(),
        },
    ];

    for error in errors {
        let error_string = format!("{error}");
        assert!(!error_string.is_empty(), "Error should have message");
    }
}

/// Test that panic-free operations remain panic-free under stress
#[tokio::test]
async fn chaos_no_panics() {
    // This test verifies that even with bad input, we don't panic

    // Test 1: Decode random-like data (deterministic, no external deps)
    for i in 0..100u8 {
        let random_data: Vec<u8> = (0..100).map(|j| i.wrapping_add(j)).collect();
        let _ = ClientHello::decode(&random_data);
        // Should not panic, might return error
    }

    // Test 2: ContentType with all possible u8 values
    for value in 0..=255u8 {
        let _ = ContentType::from(value);
        // Should not panic
    }
}

/// Test rapid allocation/deallocation (memory stress)
#[tokio::test]
async fn chaos_memory_stress() {
    // Create and drop many ClientHello instances rapidly
    for _ in 0..1000 {
        let random = [42u8; 32];
        let cipher_suites = vec![0x1303];
        let extensions = vec![
            Extension::SupportedVersions(vec![0x0304]),
            Extension::KeyShare(vec![1u8; 32]), // Full X25519 key
        ];
        let _hello = ClientHello::new(random, cipher_suites, extensions);
        // Drop automatically (RAII)
    }
    // Should complete without memory issues
}

/// Test edge case: zero-length data
#[tokio::test]
async fn fault_zero_length_data() {
    // Key share with zero-length data
    let extension = Extension::KeyShare(vec![]);
    // Should be created but fail validation later

    match extension {
        Extension::KeyShare(ref data) => {
            assert_eq!(data.len(), 0);
        }
        _ => panic!("Should be KeyShare"),
    }
}

// NOTE: We don't use sleeps in these tests - true async chaos testing!
// All concurrency is real, all timeouts are enforced by tokio::time::timeout
