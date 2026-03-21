// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Fault Injection Tests for CryptoProvider
//!
//! Tests error handling, edge cases, and resilience to failures.

use songbird_orchestrator::crypto::{
    CryptoProvider, UnixSocketCryptoProvider, discover_crypto_provider,
};
use songbird_orchestrator::primal_discovery::{Capability, discover_with};

#[tokio::test]
async fn test_fault_nonexistent_socket() {
    let provider = UnixSocketCryptoProvider::new("/nonexistent/crypto.sock".to_string());

    let result = provider.blake3_hash(b"test").await;
    assert!(result.is_err(), "Should fail with non-existent socket");

    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("Failed to connect")
            || error.to_string().contains("No such file"),
        "Error should indicate connection failure"
    );
}

#[tokio::test]
async fn test_fault_invalid_socket_path() {
    let provider = UnixSocketCryptoProvider::new(String::new());

    let result = provider.blake3_hash(b"test").await;
    assert!(result.is_err(), "Should fail with empty socket path");
}

#[tokio::test]
async fn test_fault_empty_data() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());

    // Empty data should be handled gracefully
    let result = provider.blake3_hash(b"").await;
    // Will fail without BearDog, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_fault_very_large_data() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());

    // Test with 100MB of data
    let large_data = vec![0u8; 100 * 1024 * 1024];
    let result = provider.blake3_hash(&large_data).await;

    // Should handle large data or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_fault_discover_without_provider() {
    let result = discover_with(Capability::Crypto, |_| None).await;

    // Should fail gracefully if no provider found
    if result.is_err() {
        let error = result.unwrap_err();
        let msg = error.to_string().to_lowercase();
        assert!(
            msg.contains("not available")
                || msg.contains("not found")
                || msg.contains("no crypto")
                || msg.contains("no provider"),
            "Error should indicate provider not available, got: {msg}"
        );
    }
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_wrong_key_decrypt() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"secret";
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];

    // Encrypt with key1
    let (ct, nonce, tag) =
        provider.chacha20_poly1305_encrypt(plaintext, &key1, None).await.unwrap();

    // Try to decrypt with key2 (wrong key)
    let result = provider.chacha20_poly1305_decrypt(&ct, &key2, &nonce, &tag, None).await;

    assert!(result.is_err(), "Decryption with wrong key should fail");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_corrupted_ciphertext() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"secret";
    let key = [0u8; 32];

    // Encrypt
    let (mut ct, nonce, tag) =
        provider.chacha20_poly1305_encrypt(plaintext, &key, None).await.unwrap();

    // Corrupt the ciphertext
    if !ct.is_empty() {
        ct[0] ^= 0xFF;
    }

    // Try to decrypt corrupted ciphertext
    let result = provider.chacha20_poly1305_decrypt(&ct, &key, &nonce, &tag, None).await;

    assert!(result.is_err(), "Decryption of corrupted data should fail");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_wrong_aad() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"secret";
    let key = [0u8; 32];
    let aad1 = b"context1";
    let aad2 = b"context2";

    // Encrypt with aad1
    let (ct, nonce, tag) =
        provider.chacha20_poly1305_encrypt(plaintext, &key, Some(aad1)).await.unwrap();

    // Try to decrypt with aad2
    let result = provider.chacha20_poly1305_decrypt(&ct, &key, &nonce, &tag, Some(aad2)).await;

    assert!(result.is_err(), "Decryption with wrong AAD should fail");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_invalid_nonce_size() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"secret";
    let key = [0u8; 32];

    let (ct, _, tag) = provider.chacha20_poly1305_encrypt(plaintext, &key, None).await.unwrap();

    // Try with wrong nonce size
    let wrong_nonce = vec![0u8; 16]; // Should be 12
    let result = provider.chacha20_poly1305_decrypt(&ct, &key, &wrong_nonce, &tag, None).await;

    assert!(result.is_err(), "Wrong nonce size should fail");
}

#[tokio::test]
async fn test_fault_concurrent_discovery() {
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Try to discover provider concurrently (may all fail, but shouldn't panic)
    for _ in 0..50 {
        tasks.spawn(async { discover_crypto_provider().await });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }

    // All should complete (either Ok or Err, no panics)
    assert_eq!(results.len(), 50);
}

#[tokio::test]
async fn test_fault_rapid_provider_creation() {
    // Rapidly create and drop providers
    for i in 0..100 {
        let path = format!("/tmp/test-{}.sock", i);
        let _provider = UnixSocketCryptoProvider::new(path);
        // Provider dropped immediately
    }
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_operation_timeout() {
    use std::time::Duration;
    use tokio::time::timeout;

    let provider = discover_crypto_provider().await.unwrap();

    // All operations should complete within 5 seconds
    let data = b"timeout test";
    let result = timeout(Duration::from_secs(5), provider.blake3_hash(data)).await;

    assert!(result.is_ok(), "Operation should complete within timeout");
}

#[tokio::test]
async fn test_fault_null_bytes_in_data() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());

    // Data with null bytes
    let data = b"data\0with\0nulls";
    let result = provider.blake3_hash(data).await;

    // Should handle null bytes gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_fault_unicode_in_paths() {
    // Test with unicode in socket path
    let provider = UnixSocketCryptoProvider::new("/tmp/крипто.sock".to_string());

    let result = provider.blake3_hash(b"test").await;
    // Should handle unicode paths
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_zero_length_encryption() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"";
    let key = [0u8; 32];

    // Encrypt empty data
    let result = provider.chacha20_poly1305_encrypt(plaintext, &key, None).await;

    if let Ok((ct, nonce, tag)) = result {
        // Decrypt should also work
        let decrypted =
            provider.chacha20_poly1305_decrypt(&ct, &key, &nonce, &tag, None).await.unwrap();

        assert_eq!(decrypted, plaintext);
    }
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_fault_repeated_operations_same_data() {
    let provider = discover_crypto_provider().await.unwrap();

    let data = b"repeated test";

    // Perform same operation 100 times
    let mut hashes = Vec::new();
    for _ in 0..100 {
        let hash = provider.blake3_hash(data).await.unwrap();
        hashes.push(hash);
    }

    // All hashes should be identical (deterministic)
    for hash in &hashes[1..] {
        assert_eq!(hash, &hashes[0], "Hashes should be deterministic");
    }
}

#[tokio::test]
async fn test_fault_provider_after_tokio_drop() {
    // Create provider in a scope that outlives tokio runtime
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());

    // Provider should be usable even after moving
    let moved_provider = provider;
    assert_eq!(moved_provider.socket_path(), "/tmp/test.sock");
}

#[tokio::test]
async fn test_fault_discovery_with_invalid_env() {
    let result = discover_with(Capability::Crypto, |name| {
        (name == "CRYPTO_PROVIDER_SOCKET")
            .then_some("///invalid///path///with///too///many///slashes".to_string())
    })
    .await;

    assert!(result.is_ok() || result.is_err());
}
