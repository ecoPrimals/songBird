//! E2E tests for RFC 8446 compliant TLS handshake with message decryption
//!
//! These tests verify that:
//! 1. Handshake traffic keys are derived after ServerHello
//! 2. Post-handshake messages are decrypted before adding to transcript
//! 3. Transcript hash is computed over PLAINTEXT messages (RFC 8446 Section 4.4.1)
//! 4. Application traffic keys are derived with correct transcript hash
//! 5. HTTPS connections work end-to-end

use songbird_http_client::client::SongbirdHttpClient;
use std::time::Duration;
use tokio::time::timeout;

/// Test full HTTPS request with handshake decryption
///
/// This test verifies the complete flow:
/// 1. ClientHello sent (plaintext added to transcript)
/// 2. ServerHello received (plaintext added to transcript)
/// 3. Handshake traffic keys derived
/// 4. EncryptedExtensions received and DECRYPTED (plaintext added to transcript)
/// 5. Certificate received and DECRYPTED (plaintext added to transcript)
/// 6. CertificateVerify received and DECRYPTED (plaintext added to transcript)
/// 7. Server Finished received and DECRYPTED (plaintext added to transcript)
/// 8. Transcript hash computed (over ALL plaintext messages)
/// 9. Application traffic keys derived (with transcript hash)
/// 10. HTTP request/response exchanged (with application keys)
#[tokio::test]
#[ignore] // Requires BearDog and real HTTPS server
async fn test_full_https_with_handshake_decryption() {
    // Initialize client
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    // Make HTTPS request to real server
    let result = timeout(
        Duration::from_secs(30),
        client.get("https://httpbin.org/get")
    ).await;

    // Should succeed with RFC 8446 compliant handshake
    assert!(result.is_ok(), "HTTPS request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "HTTPS request should succeed: {:?}", response.err());
    
    let body = response.unwrap();
    assert!(!body.is_empty(), "Response body should not be empty");
    println!("✅ Full HTTPS request succeeded with RFC 8446 compliant handshake");
    println!("Response preview: {}", String::from_utf8_lossy(&body[..std::cmp::min(200, body.len())]));
}

/// Test HTTPS with GitHub API (real-world server)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_github_api_with_decryption() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    let result = timeout(
        Duration::from_secs(30),
        client.get("https://api.github.com/zen")
    ).await;

    assert!(result.is_ok(), "GitHub API request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "GitHub API request should succeed: {:?}", response.err());
    
    let body = response.unwrap();
    let text = String::from_utf8_lossy(&body);
    println!("✅ GitHub API response: {}", text);
    assert!(!body.is_empty(), "GitHub response should not be empty");
}

/// Test HTTPS with Google (requires ALPN, comprehensive handshake)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_google_with_decryption() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    let result = timeout(
        Duration::from_secs(30),
        client.get("https://www.google.com/")
    ).await;

    assert!(result.is_ok(), "Google request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "Google request should succeed: {:?}", response.err());
    
    let body = response.unwrap();
    println!("✅ Google HTTPS response received: {} bytes", body.len());
    assert!(body.len() > 1000, "Google response should be substantial");
}

/// Test multiple sequential HTTPS requests (sequence number handling)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_multiple_https_requests_sequential() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    // Make 3 sequential requests
    for i in 1..=3 {
        println!("Making request {}/3...", i);
        
        let result = timeout(
            Duration::from_secs(30),
            client.get("https://httpbin.org/get")
        ).await;

        assert!(result.is_ok(), "Request {} should not timeout", i);
        let response = result.unwrap();
        assert!(response.is_ok(), "Request {} should succeed: {:?}", i, response.err());
        
        let body = response.unwrap();
        assert!(!body.is_empty(), "Response {} should not be empty", i);
        println!("✅ Request {}/3 succeeded ({} bytes)", i, body.len());
    }
}

/// Test HTTPS with CloudFlare (strict TLS requirements)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_cloudflare_with_decryption() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    let result = timeout(
        Duration::from_secs(30),
        client.get("https://cloudflare.com/")
    ).await;

    assert!(result.is_ok(), "CloudFlare request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "CloudFlare request should succeed: {:?}", response.err());
    
    let body = response.unwrap();
    println!("✅ CloudFlare HTTPS response received: {} bytes", body.len());
    assert!(!body.is_empty(), "CloudFlare response should not be empty");
}

/// Test HTTPS POST request with body
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_post_with_decryption() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    let result = timeout(
        Duration::from_secs(30),
        client.post("https://httpbin.org/post", b"test data")
    ).await;

    assert!(result.is_ok(), "POST request should not timeout");
    let response = result.unwrap();
    assert!(response.is_ok(), "POST request should succeed: {:?}", response.err());
    
    let body = response.unwrap();
    let text = String::from_utf8_lossy(&body);
    println!("✅ POST response: {}", &text[..std::cmp::min(200, text.len())]);
    assert!(text.contains("test data"), "Response should echo posted data");
}

/// Test HTTPS connection reuse (if implemented)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_connection_reuse() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    // Make two requests to same host
    let result1 = client.get("https://httpbin.org/get").await;
    assert!(result1.is_ok(), "First request should succeed");
    
    let result2 = client.get("https://httpbin.org/headers").await;
    assert!(result2.is_ok(), "Second request should succeed");
    
    println!("✅ Both requests to same host succeeded");
}

/// Test HTTPS with different cipher suites (if negotiated)
#[tokio::test]
#[ignore] // Requires BearDog and internet connection
async fn test_https_cipher_suite_negotiation() {
    let client = SongbirdHttpClient::new("/tmp/beardog.sock")
        .expect("Failed to create HTTP client");

    // Try different servers that may negotiate different cipher suites
    let servers = [
        "https://httpbin.org/get",
        "https://api.github.com/",
        "https://www.google.com/",
    ];

    for server in servers.iter() {
        let result = timeout(
            Duration::from_secs(30),
            client.get(server)
        ).await;

        assert!(result.is_ok(), "Request to {} should not timeout", server);
        let response = result.unwrap();
        assert!(response.is_ok(), "Request to {} should succeed: {:?}", server, response.err());
        println!("✅ Successfully connected to {}", server);
    }
}

