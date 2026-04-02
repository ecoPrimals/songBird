// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! End-to-End Tests for `CryptoProvider`
//!
//! Tests the complete flow from discovery to crypto operations.
//! These tests require a running `BearDog` instance.

use songbird_orchestrator::crypto::discover_crypto_provider;

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_discover_and_hash() {
    let provider = discover_crypto_provider()
        .await
        .expect("Failed to discover crypto provider - is BearDog running?");

    let data = b"Hello, Songbird!";
    let hash = provider.blake3_hash(data).await.unwrap();

    assert_eq!(hash.len(), 32, "BLAKE3 hash should be 32 bytes");
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_complete_crypto_flow() {
    let provider = discover_crypto_provider().await.expect("Failed to discover crypto provider");

    // Test all operations in sequence
    let data = b"test data";

    // 1. Hash
    let hash = provider.blake3_hash(data).await.unwrap();
    assert_eq!(hash.len(), 32);

    // 2. HMAC
    let key = b"secret key";
    let mac = provider.hmac_sha256(key, data).await.unwrap();
    assert_eq!(mac.len(), 32);

    // 3. X25519 key exchange
    let (pk, sk) = provider.x25519_generate_ephemeral("e2e_test").await.unwrap();
    assert_eq!(pk.len(), 32);
    assert_eq!(sk.len(), 32);

    let shared = provider.x25519_derive_secret(&sk, &pk).await.unwrap();
    assert_eq!(shared.len(), 32);

    // 4. ChaCha20-Poly1305 encryption
    let plaintext = b"secret message";
    let key = [0u8; 32];

    let (ciphertext, nonce, tag) =
        provider.chacha20_poly1305_encrypt(plaintext, &key, None).await.unwrap();

    assert_eq!(ciphertext.len(), plaintext.len());
    assert_eq!(nonce.len(), 12);
    assert_eq!(tag.len(), 16);

    // 5. ChaCha20-Poly1305 decryption
    let decrypted =
        provider.chacha20_poly1305_decrypt(&ciphertext, &key, &nonce, &tag, None).await.unwrap();

    assert_eq!(decrypted, plaintext);
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_ed25519_sign_verify() {
    let provider = discover_crypto_provider().await.unwrap();

    let message = b"Sign this message";
    let signature = provider.sign_ed25519(message, "e2e_test_key", "e2e_test").await.unwrap();

    assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");

    // Note: Verification requires the public key
    // In a real scenario, we'd retrieve it from BearDog
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_performance_single_operation() {
    use std::time::Instant;

    let provider = discover_crypto_provider().await.unwrap();
    let data = b"performance test data";

    let start = Instant::now();
    let _hash = provider.blake3_hash(data).await.unwrap();
    let duration = start.elapsed();

    // Should complete in under 10ms (generous for JSON-RPC over Unix socket)
    assert!(
        duration.as_millis() < 10,
        "Blake3 hash took {}ms (expected <10ms)",
        duration.as_millis()
    );
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_multiple_operations_sequential() {
    let provider = discover_crypto_provider().await.unwrap();

    // Perform 10 operations sequentially
    for i in 0..10 {
        let data = format!("test data {i}");
        let hash = provider.blake3_hash(data.as_bytes()).await.unwrap();
        assert_eq!(hash.len(), 32);
    }
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_large_data_hash() {
    let provider = discover_crypto_provider().await.unwrap();

    // Test with 1MB of data
    let large_data = vec![0u8; 1024 * 1024];
    let hash = provider.blake3_hash(&large_data).await.unwrap();

    assert_eq!(hash.len(), 32);
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_encryption_with_aad() {
    let provider = discover_crypto_provider().await.unwrap();

    let plaintext = b"secret with AAD";
    let key = [1u8; 32];
    let aad = b"additional authenticated data";

    let (ciphertext, nonce, tag) =
        provider.chacha20_poly1305_encrypt(plaintext, &key, Some(aad)).await.unwrap();

    // Decrypt with correct AAD
    let decrypted = provider
        .chacha20_poly1305_decrypt(&ciphertext, &key, &nonce, &tag, Some(aad))
        .await
        .unwrap();

    assert_eq!(decrypted, plaintext);

    // Decrypt with wrong AAD should fail
    let wrong_aad = b"wrong data";
    let result =
        provider.chacha20_poly1305_decrypt(&ciphertext, &key, &nonce, &tag, Some(wrong_aad)).await;

    assert!(result.is_err(), "Decryption with wrong AAD should fail");
}

#[tokio::test]
#[ignore = "requires running BearDog crypto provider"] // Requires BearDog running
async fn test_e2e_concurrent_operations() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let provider = Arc::new(discover_crypto_provider().await.expect("Failed to discover provider"));

    let mut tasks = JoinSet::new();

    // Spawn 50 concurrent hash operations
    for i in 0..50 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            let data = format!("concurrent test {i}");
            p.blake3_hash(data.as_bytes()).await
        });
    }

    // All should succeed
    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        let hash_result = result.unwrap();
        if hash_result.is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 50, "All concurrent operations should succeed");
}
