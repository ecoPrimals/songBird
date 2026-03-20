// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Chaos Tests for CryptoProvider
//!
//! Extreme load testing, stress testing, and resilience testing.

use songbird_orchestrator::crypto::discover_crypto_provider;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_1000_concurrent_operations() {
    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let mut tasks = JoinSet::new();

    // Spawn 1000 concurrent hash operations
    for i in 0..1000 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            let data = format!("chaos test {}", i);
            p.blake3_hash(data.as_bytes()).await
        });
    }

    // Count successes
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = tasks.join_next().await {
        match result.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    println!("Chaos test: {} succeeded, {} failed", success_count, error_count);

    // Allow up to 1% failure rate under extreme load
    assert!(success_count >= 990, "At least 99% should succeed (got {})", success_count);
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_mixed_operations_concurrent() {
    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let mut tasks = JoinSet::new();

    // Spawn 500 operations of different types
    for i in 0..500 {
        let p = Arc::clone(&provider);

        tasks.spawn(async move {
            let data = format!("test {}", i).into_bytes();

            match i % 5 {
                0 => p.blake3_hash(&data).await.map(|_| ()),
                1 => p.hmac_sha256(&data, &data).await.map(|_| ()),
                2 => p.x25519_generate_ephemeral("chaos").await.map(|_| ()),
                3 => {
                    let key = [0u8; 32];
                    p.chacha20_poly1305_encrypt(&data, &key, None).await.map(|_| ())
                }
                _ => p.blake3_hash(&data).await.map(|_| ()),
            }
        });
    }

    // Collect results
    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 490, "At least 98% should succeed under mixed load");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_rapid_fire_sequential() {
    let provider = discover_crypto_provider().await.unwrap();

    // Perform 1000 operations as fast as possible
    let mut success_count = 0;
    for i in 0..1000 {
        let data = format!("rapid {}", i);
        if provider.blake3_hash(data.as_bytes()).await.is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 990, "At least 99% should succeed in rapid fire");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_memory_pressure() {
    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let mut tasks = JoinSet::new();

    // Test with large data under concurrent load
    for i in 0..100 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            // 1MB per operation = 100MB total concurrent
            let large_data = vec![i as u8; 1024 * 1024];
            p.blake3_hash(&large_data).await
        });
    }

    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 95, "At least 95% should succeed under memory pressure");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_encryption_round_trips() {
    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let mut tasks = JoinSet::new();

    // Perform 200 encryption/decryption round trips concurrently
    for i in 0..200 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            let plaintext = format!("round trip {}", i).into_bytes();
            let key = [i as u8; 32];

            // Encrypt
            let (ct, nonce, tag) = p.chacha20_poly1305_encrypt(&plaintext, &key, None).await?;

            // Decrypt
            let decrypted = p.chacha20_poly1305_decrypt(&ct, &key, &nonce, &tag, None).await?;

            // Verify
            if decrypted == plaintext {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Round trip verification failed"))
            }
        });
    }

    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 200, "All round trips should succeed");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_provider_drop_and_recreate() {
    // Test creating and dropping providers rapidly
    for i in 0..100 {
        let provider = discover_crypto_provider().await.unwrap();
        let data = format!("drop test {}", i);
        let _hash = provider.blake3_hash(data.as_bytes()).await.unwrap();
        drop(provider);
    }
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_varying_data_sizes() {
    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let mut tasks = JoinSet::new();

    // Test with varying data sizes from 1 byte to 10MB
    for i in 0..100 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            // Exponentially increasing sizes
            let size = 1 << (i % 20); // 1, 2, 4, 8, ... up to 1MB
            let data = vec![i as u8; size];
            p.blake3_hash(&data).await
        });
    }

    let mut success_count = 0;
    while let Some(result) = tasks.join_next().await {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    assert!(success_count >= 95, "At least 95% should succeed with varying sizes");
}

#[tokio::test]
#[ignore] // Requires BearDog running
async fn test_chaos_sustained_load() {
    use std::time::{Duration, Instant};

    let provider = Arc::new(discover_crypto_provider().await.unwrap());
    let start = Instant::now();
    let duration = Duration::from_secs(10); // 10 seconds of sustained load

    let mut operation_count = 0;
    let mut error_count = 0;

    while start.elapsed() < duration {
        let data = format!("sustained {}", operation_count);
        match provider.blake3_hash(data.as_bytes()).await {
            Ok(_) => operation_count += 1,
            Err(_) => error_count += 1,
        }
    }

    println!(
        "Sustained load: {} ops in {:?} ({} ops/sec), {} errors",
        operation_count,
        start.elapsed(),
        operation_count / 10,
        error_count
    );

    // Should handle at least 100 ops/sec
    assert!(operation_count >= 1000, "Should handle sustained load (got {} ops)", operation_count);

    // Error rate should be minimal
    assert!(error_count < operation_count / 100, "Error rate should be <1%");
}
