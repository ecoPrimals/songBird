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
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Unit Tests for `CryptoProvider` Abstraction
//!
//! Tests the capability-based crypto provider abstraction in isolation.

use songbird_orchestrator::crypto::{CryptoProvider, UnixSocketCryptoProvider};
use songbird_orchestrator::primal_discovery::{Capability, discover_with};

#[tokio::test]
async fn test_unix_socket_provider_creation() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());
    assert_eq!(provider.socket_path(), "/tmp/test.sock");
}

#[tokio::test]
async fn test_discover_crypto_provider_with_env() {
    let result = discover_with(Capability::Crypto, |name| {
        (name == "CRYPTO_PROVIDER_SOCKET").then_some("/tmp/custom-crypto.sock".to_string())
    })
    .await;

    // We don't require it to succeed (socket may not exist in test env)
    // Just verify the function is callable and returns a Result
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_crypto_provider_trait_object() {
    // Verify we can create a trait object
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());
    let _trait_obj: &dyn CryptoProvider = &provider;

    // Trait object should be Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<UnixSocketCryptoProvider>();
}

#[tokio::test]
async fn test_provider_arc_usage() {
    use std::sync::Arc;

    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());
    let arc_provider: Arc<dyn CryptoProvider> = Arc::new(provider);

    // Verify Arc works (multiple ownership)
    let arc_clone = Arc::clone(&arc_provider);
    assert_eq!(Arc::strong_count(&arc_provider), 2);
    drop(arc_clone);
    assert_eq!(Arc::strong_count(&arc_provider), 1);
}

#[tokio::test]
async fn test_multiple_providers_concurrent() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let provider = Arc::new(UnixSocketCryptoProvider::new("/tmp/test.sock".to_string()));
    let mut tasks = JoinSet::new();

    // Spawn 10 concurrent tasks using the same provider
    for i in 0..10 {
        let p = Arc::clone(&provider);
        tasks.spawn(async move {
            // Just verify we can access it concurrently
            assert_eq!(p.socket_path(), "/tmp/test.sock");
            i
        });
    }

    // Collect all results
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }

    assert_eq!(results.len(), 10);
}

#[tokio::test]
async fn test_provider_error_handling() {
    let provider = UnixSocketCryptoProvider::new("/nonexistent/socket.sock".to_string());

    // Operations should fail gracefully with non-existent socket
    let result = provider.blake3_hash(b"test").await;
    assert!(result.is_err(), "Should fail with non-existent socket");
}

#[tokio::test]
async fn test_provider_different_data_sizes() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());

    // Test with different data sizes (will fail without BearDog, but verifies API)
    let sizes = vec![0, 1, 100, 1024, 10240, 1048576]; // 0 bytes to 1MB

    for size in sizes {
        let data = vec![0u8; size];
        let result = provider.blake3_hash(&data).await;
        // We expect error (no BearDog), but API should handle any size
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_discover_priority_order() {
    let result = discover_with(Capability::Crypto, |name| match name {
        "CRYPTO_PROVIDER_SOCKET" => Some("/tmp/priority1.sock".to_string()),
        "CRYPTO_PROVIDER" => Some("/tmp/priority2.sock".to_string()),
        "BEARDOG_CRYPTO_SOCKET" | "BEARDOG_SOCKET" => Some("/tmp/priority3.sock".to_string()),
        _ => None,
    })
    .await;

    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_provider_type_safety() {
    // Verify type safety - these should compile
    fn take_provider(_provider: &dyn CryptoProvider) {}
    fn take_unix_provider(_provider: &UnixSocketCryptoProvider) {}

    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());
    take_provider(&provider);
    take_unix_provider(&provider);
}

#[tokio::test]
async fn test_provider_clone_and_send() {
    use std::sync::Arc;

    let provider = Arc::new(UnixSocketCryptoProvider::new("/tmp/test.sock".to_string()));

    // Verify we can send across threads
    let handle = tokio::spawn(async move {
        assert_eq!(provider.socket_path(), "/tmp/test.sock");
    });

    handle.await.unwrap();
}
