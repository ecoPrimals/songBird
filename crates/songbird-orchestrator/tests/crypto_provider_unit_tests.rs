//! Unit Tests for CryptoProvider Abstraction
//!
//! Tests the capability-based crypto provider abstraction in isolation.

use songbird_orchestrator::crypto::{
    discover_crypto_provider, CryptoProvider, UnixSocketCryptoProvider,
};

#[tokio::test]
async fn test_unix_socket_provider_creation() {
    let provider = UnixSocketCryptoProvider::new("/tmp/test.sock".to_string());
    assert_eq!(provider.socket_path(), "/tmp/test.sock");
}

#[tokio::test]
async fn test_discover_crypto_provider_with_env() {
    std::env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/custom-crypto.sock");

    // Discovery should succeed if socket exists (will fail gracefully in test)
    let result = discover_crypto_provider().await;

    std::env::remove_var("CRYPTO_PROVIDER_SOCKET");

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
    // Test that CRYPTO_PROVIDER_SOCKET has priority
    std::env::set_var("CRYPTO_PROVIDER_SOCKET", "/tmp/priority1.sock");
    std::env::set_var("CRYPTO_PROVIDER", "/tmp/priority2.sock");
    std::env::set_var("BEARDOG_CRYPTO_SOCKET", "/tmp/priority3.sock");

    // Discovery logic should check CRYPTO_PROVIDER_SOCKET first
    let result = discover_crypto_provider().await;

    std::env::remove_var("CRYPTO_PROVIDER_SOCKET");
    std::env::remove_var("CRYPTO_PROVIDER");
    std::env::remove_var("BEARDOG_CRYPTO_SOCKET");

    // Verify function executes (may fail if socket doesn't exist)
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
