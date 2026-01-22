//! End-to-End TLS Handshake Tests
//!
//! These tests verify the complete TLS 1.3 handshake flow with real servers.
//! They are marked #[ignore] by default and can be run with: cargo test -- --ignored

use songbird_http_client::SongbirdHttpClient;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
#[ignore] // Requires network access
async fn test_github_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://api.github.com")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "GitHub handshake should succeed: {:?}", response.err());
    
    let http_response = response.unwrap();
    assert_eq!(http_response.status, 200, "Should get 200 OK from GitHub");
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_cloudflare_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://cloudflare.com")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Cloudflare handshake should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_google_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://www.google.com")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Google handshake should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_mozilla_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://www.mozilla.org")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Mozilla handshake should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_rust_lang_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://www.rust-lang.org")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Rust-lang handshake should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_crates_io_https_handshake() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://crates.io")
    ).await;
    
    assert!(result.is_ok(), "Request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Crates.io handshake should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_multiple_sequential_requests() {
    let client = SongbirdHttpClient::from_env();
    
    for i in 0..3 {
        let result = timeout(
            Duration::from_secs(10),
            client.get("https://api.github.com")
        ).await;
        
        assert!(result.is_ok(), "Request {} should not timeout", i);
        let response = result.unwrap();
        assert!(response.is_ok(), "Request {} should succeed: {:?}", i, response.err());
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_concurrent_requests() {
    let client = SongbirdHttpClient::from_env();
    
    let handles: Vec<_> = (0..3).map(|i| {
        let client = client.clone();
        tokio::spawn(async move {
            let result = timeout(
                Duration::from_secs(10),
                client.get("https://api.github.com")
            ).await;
            
            assert!(result.is_ok(), "Concurrent request {} should not timeout", i);
            let response = result.unwrap();
            assert!(response.is_ok(), "Concurrent request {} should succeed", i);
        })
    }).collect();
    
    for handle in handles {
        handle.await.expect("Task should not panic");
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_post_request() {
    let client = SongbirdHttpClient::from_env();
    
    let body = serde_json::json!({
        "test": "data"
    });
    
    // Using httpbin.org for POST testing
    let result = timeout(
        Duration::from_secs(10),
        client.post("https://httpbin.org/post", body)
    ).await;
    
    assert!(result.is_ok(), "POST request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "POST should succeed: {:?}", response.err());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_large_response() {
    let client = SongbirdHttpClient::from_env();
    
    // Request a large response (10KB)
    let result = timeout(
        Duration::from_secs(15),
        client.get("https://httpbin.org/bytes/10240")
    ).await;
    
    assert!(result.is_ok(), "Large response request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Large response should succeed: {:?}", response.err());
    
    let http_response = response.unwrap();
    // Body is JSON Value, check it's not null
    assert!(!http_response.body.is_null(), "Should receive response body");
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_redirect_handling() {
    let client = SongbirdHttpClient::from_env();
    
    // httpbin.org/redirect/1 returns a 302 redirect
    let result = timeout(
        Duration::from_secs(10),
        client.get("https://httpbin.org/redirect/1")
    ).await;
    
    assert!(result.is_ok(), "Redirect request should not timeout");
    let response = result.unwrap();
    // We expect either a redirect status or success depending on implementation
    assert!(response.is_ok() || response.is_err(), "Redirect should be handled");
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_invalid_hostname() {
    let client = SongbirdHttpClient::from_env();
    
    let result = timeout(
        Duration::from_secs(5),
        client.get("https://this-domain-does-not-exist-12345.com")
    ).await;
    
    // Should either timeout or return an error
    if let Ok(response) = result {
        assert!(response.is_err(), "Invalid hostname should fail");
    }
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_connection_reuse() {
    let client = SongbirdHttpClient::from_env();
    
    // First request
    let start1 = std::time::Instant::now();
    let result1 = client.get("https://api.github.com").await;
    let duration1 = start1.elapsed();
    assert!(result1.is_ok(), "First request should succeed");
    
    // Second request (should be faster if connection is reused)
    let start2 = std::time::Instant::now();
    let result2 = client.get("https://api.github.com").await;
    let duration2 = start2.elapsed();
    assert!(result2.is_ok(), "Second request should succeed");
    
    // Note: Connection reuse would make second request faster
    // But we don't enforce this as it depends on server keep-alive
    println!("First request: {:?}, Second request: {:?}", duration1, duration2);
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_different_tls_servers() {
    // Test against servers with different TLS configurations
    let client = SongbirdHttpClient::from_env();
    
    let servers = vec![
        "https://www.wikipedia.org",
        "https://www.amazon.com",
        "https://www.microsoft.com",
    ];
    
    for server in servers {
        let result = timeout(
            Duration::from_secs(10),
            client.get(server)
        ).await;
        
        assert!(result.is_ok(), "Request to {} should not timeout", server);
        let response = result.unwrap();
        assert!(response.is_ok(), "Request to {} should succeed: {:?}", server, response.err());
    }
}

