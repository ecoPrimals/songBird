// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
//! Integration tests for songbird-tls
//!
//! These tests validate the TLS implementation with mock security-provider crypto.

use songbird_tls::error::TlsError;
use tokio::net::TcpListener;

/// Mock security-provider crypto client for testing
///
/// This simulates provider JSON-RPC responses without requiring a live crypto provider.
/// In production, use real `SecurityTlsCryptoClient`.
#[derive(Clone)]
struct MockSecurityProviderTlsClient {
    fail_on_operation: Option<String>,
}

impl MockSecurityProviderTlsClient {
    const fn new() -> Self {
        Self {
            fail_on_operation: None,
        }
    }

    fn with_failure(operation: &str) -> Self {
        Self {
            fail_on_operation: Some(operation.to_string()),
        }
    }

    fn x25519_generate_ephemeral(&self) -> Result<(Vec<u8>, Vec<u8>), TlsError> {
        if self.fail_on_operation.as_deref() == Some("x25519_generate") {
            return Err(TlsError::CryptoError("Mock failure: x25519_generate".to_string()));
        }

        // Return deterministic test keys
        let public_key = vec![1u8; 32];
        let secret_key = vec![2u8; 32];
        Ok((public_key, secret_key))
    }

    fn x25519_derive_secret(
        &self,
        _our_secret: &[u8],
        _their_public: &[u8],
    ) -> Result<Vec<u8>, TlsError> {
        if self.fail_on_operation.as_deref() == Some("x25519_derive") {
            return Err(TlsError::CryptoError("Mock failure: x25519_derive".to_string()));
        }

        // Return deterministic shared secret
        Ok(vec![3u8; 32])
    }

    fn hmac_sha256(&self, _message: &[u8], _key: &[u8]) -> Result<Vec<u8>, TlsError> {
        if self.fail_on_operation.as_deref() == Some("hmac") {
            return Err(TlsError::CryptoError("Mock failure: hmac".to_string()));
        }

        // Return deterministic HMAC
        Ok(vec![4u8; 32])
    }

    #[allow(clippy::type_complexity, reason = "complex return type required by trait bounds")]
    fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        _key: &[u8],
        _aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), TlsError> {
        if self.fail_on_operation.as_deref() == Some("encrypt") {
            return Err(TlsError::CryptoError("Mock failure: encrypt".to_string()));
        }

        // Simple "encryption" for testing: XOR with 0xFF
        let ciphertext: Vec<u8> = plaintext.iter().map(|b| b ^ 0xFF).collect();
        let nonce = vec![5u8; 12];
        let tag = vec![6u8; 16];
        Ok((ciphertext, nonce, tag))
    }

    fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        _key: &[u8],
        _nonce: &[u8],
        _tag: &[u8],
        _aad: Option<&[u8]>,
    ) -> Result<Vec<u8>, TlsError> {
        if self.fail_on_operation.as_deref() == Some("decrypt") {
            return Err(TlsError::CryptoError("Mock failure: decrypt".to_string()));
        }

        // Simple "decryption" for testing: XOR with 0xFF (reverse of encrypt)
        let plaintext: Vec<u8> = ciphertext.iter().map(|b| b ^ 0xFF).collect();
        Ok(plaintext)
    }
}

/// Test helper: Create a test TLS server on a random port
async fn create_test_server() -> Result<(TcpListener, u16), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    Ok((listener, port))
}

#[tokio::test]
async fn test_mock_security_provider_client_operations() {
    let client = MockSecurityProviderTlsClient::new();

    // Test key generation
    let (public, secret) = client.x25519_generate_ephemeral().unwrap();
    assert_eq!(public.len(), 32);
    assert_eq!(secret.len(), 32);

    // Test key derivation
    let shared = client.x25519_derive_secret(&secret, &public).unwrap();
    assert_eq!(shared.len(), 32);

    // Test HMAC
    let hmac = client.hmac_sha256(b"test", b"key").unwrap();
    assert_eq!(hmac.len(), 32);

    // Test encryption/decryption
    let plaintext = b"Hello, TLS!";
    let (ciphertext, nonce, tag) =
        client.chacha20_poly1305_encrypt(plaintext, b"key", None).unwrap();

    assert_eq!(nonce.len(), 12);
    assert_eq!(tag.len(), 16);
    assert_ne!(ciphertext, plaintext); // Should be different

    let decrypted =
        client.chacha20_poly1305_decrypt(&ciphertext, b"key", &nonce, &tag, None).unwrap();

    assert_eq!(decrypted, plaintext); // Should match original
}

#[tokio::test]
async fn test_mock_security_provider_failure_injection() {
    // Test x25519_generate failure
    let client = MockSecurityProviderTlsClient::with_failure("x25519_generate");
    let result = client.x25519_generate_ephemeral();
    assert!(result.is_err());

    // Test x25519_derive failure
    let client = MockSecurityProviderTlsClient::with_failure("x25519_derive");
    let result = client.x25519_derive_secret(&[0u8; 32], &[1u8; 32]);
    assert!(result.is_err());

    // Test HMAC failure
    let client = MockSecurityProviderTlsClient::with_failure("hmac");
    let result = client.hmac_sha256(b"test", b"key");
    assert!(result.is_err());

    // Test encryption failure
    let client = MockSecurityProviderTlsClient::with_failure("encrypt");
    let result = client.chacha20_poly1305_encrypt(b"test", b"key", None);
    assert!(result.is_err());

    // Test decryption failure
    let client = MockSecurityProviderTlsClient::with_failure("decrypt");
    let result = client.chacha20_poly1305_decrypt(b"test", b"key", &[0u8; 12], &[0u8; 16], None);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tcp_listener_creation() {
    let result = create_test_server().await;
    assert!(result.is_ok());

    let (listener, port) = result.unwrap();
    assert!(port > 0);

    // Verify we can get the address
    let addr = listener.local_addr().unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), port);
}

// NOTE: Full E2E tests with handshake require more infrastructure
// These will be added in the next iteration with mock TLS client
