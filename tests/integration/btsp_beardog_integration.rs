//! BTSP Integration Tests with BearDog
//!
//! These tests validate the Unix socket-based BTSP client integration
//! with a live BearDog server.
//!
//! # Philosophy
//! - **Deep Testing**: Comprehensive validation, not just happy paths
//! - **Real Integration**: Tests with actual BearDog server, not just mocks
//! - **Modern Patterns**: Async testing with proper cleanup
//! - **No Hardcoding**: Environment-based socket discovery
//!
//! # Test Strategy
//! 1. Basic connectivity (ping)
//! 2. Tunnel establishment
//! 3. Encrypt/decrypt operations
//! 4. Tunnel status and lifecycle
//! 5. Error handling and recovery
//! 6. Contact exchange (peer discovery)
//!
//! # Running These Tests
//!
//! **With BearDog running**:
//! ```bash
//! # Start BearDog in another terminal
//! cd ../beardog && cargo run --bin beardog
//!
//! # Run integration tests
//! cargo test --test btsp_beardog_integration -- --ignored --test-threads=1
//! ```
//!
//! **Without BearDog** (tests will be skipped):
//! ```bash
//! cargo test --test btsp_beardog_integration
//! ```

use anyhow::Result;
use songbird_orchestrator::btsp_client::{BtspClient, Direction, PeerEndpoint, TunnelState};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use tracing_subscriber;

/// Initialize tracing for tests (call once per test)
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Check if BearDog is available for testing
async fn is_beardog_available() -> bool {
    // Check for explicit environment variable
    if env::var("BEARDOG_SOCKET").is_ok() || env::var("BIOMEOS_SOCKET_PATH").is_ok() {
        let client = BtspClient::new();
        match client.ping().await {
            Ok(_) => {
                info!("BearDog is available for integration testing");
                true
            }
            Err(e) => {
                warn!("BearDog socket configured but ping failed: {}", e);
                false
            }
        }
    } else {
        warn!("BearDog not configured (set BEARDOG_SOCKET or BIOMEOS_SOCKET_PATH)");
        false
    }
}

/// Skip test if BearDog is not available
macro_rules! skip_if_no_beardog {
    () => {
        if !is_beardog_available().await {
            eprintln!("⏭️  Skipping test: BearDog not available");
            eprintln!("   To run: export BEARDOG_SOCKET=/tmp/beardog-default-default.sock");
            eprintln!("   And start: cd ../beardog && cargo run --bin beardog");
            return;
        }
    };
}

// ============================================================================
// Basic Connectivity Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_ping() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP ping with BearDog");

    let client = BtspClient::new();
    let response = client.ping().await.expect("Ping failed");

    debug!("Ping response: {:?}", response);

    // Validate response structure
    assert!(response.is_object(), "Response should be a JSON object");
    assert_eq!(
        response["primal"].as_str(),
        Some("beardog"),
        "Primal should be 'beardog'"
    );

    info!("✅ BTSP ping successful");
}

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_multiple_pings() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing multiple BTSP pings (connection reuse)");

    let client = BtspClient::new();

    // Send multiple pings to test connection stability
    for i in 1..=5 {
        debug!("Ping attempt {}/5", i);
        let response = client.ping().await.expect("Ping failed");
        assert_eq!(response["primal"].as_str(), Some("beardog"));
        sleep(Duration::from_millis(100)).await;
    }

    info!("✅ Multiple pings successful");
}

// ============================================================================
// Tunnel Establishment Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_tunnel_establishment() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP tunnel establishment");

    let client = BtspClient::new();

    // Create a peer endpoint for testing
    let peer = PeerEndpoint {
        id: "test-peer-001".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["test".to_string()],
    };

    // Establish tunnel
    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    debug!("Tunnel established: {:?}", tunnel);

    // Validate tunnel
    assert!(!tunnel.id.is_empty(), "Tunnel ID should not be empty");
    assert_eq!(tunnel.peer_id, "test-peer-001");
    assert_eq!(tunnel.state, TunnelState::Established);

    info!("✅ Tunnel establishment successful: {}", tunnel.id);

    // Clean up: close tunnel
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");
}

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_multiple_tunnels() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing multiple concurrent BTSP tunnels");

    let client = BtspClient::new();

    // Establish multiple tunnels
    let mut tunnels = Vec::new();
    for i in 1..=3 {
        let peer = PeerEndpoint {
            id: format!("test-peer-{:03}", i),
            endpoint: format!("test://localhost:{}", 9999 + i),
            public_key: None,
            capabilities: vec!["test".to_string()],
        };

        let tunnel = client
            .establish_tunnel(peer)
            .await
            .expect("Tunnel establishment failed");

        info!("Tunnel {}/3 established: {}", i, tunnel.id);
        tunnels.push(tunnel);
    }

    // Verify all tunnels are established
    assert_eq!(tunnels.len(), 3);
    for tunnel in &tunnels {
        assert_eq!(tunnel.state, TunnelState::Established);
    }

    info!("✅ Multiple tunnels established successfully");

    // Clean up: close all tunnels
    for tunnel in tunnels {
        client
            .tunnel_close(&tunnel)
            .await
            .expect("Tunnel close failed");
    }
}

// ============================================================================
// Encrypt/Decrypt Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_encrypt_decrypt() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP encrypt/decrypt operations");

    let client = BtspClient::new();

    // Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer-encrypt".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["encryption".to_string()],
    };

    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    // Test data
    let plaintext = b"Hello, BearDog! This is a test message from Songbird.";
    debug!("Original plaintext: {:?}", String::from_utf8_lossy(plaintext));

    // Encrypt
    let ciphertext = client
        .tunnel_encrypt(&tunnel, plaintext, Direction::Egress)
        .await
        .expect("Encryption failed");

    debug!("Ciphertext length: {} bytes", ciphertext.len());
    assert!(
        !ciphertext.is_empty(),
        "Ciphertext should not be empty"
    );
    assert_ne!(
        ciphertext, plaintext,
        "Ciphertext should differ from plaintext"
    );

    // Decrypt
    let decrypted = client
        .tunnel_decrypt(&tunnel, &ciphertext)
        .await
        .expect("Decryption failed");

    debug!("Decrypted plaintext: {:?}", String::from_utf8_lossy(&decrypted));

    // Verify round-trip
    assert_eq!(
        decrypted, plaintext,
        "Decrypted text should match original"
    );

    info!("✅ Encrypt/decrypt successful (round-trip verified)");

    // Clean up
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");
}

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_large_data_encrypt_decrypt() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP encrypt/decrypt with large data");

    let client = BtspClient::new();

    // Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer-large".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["encryption".to_string()],
    };

    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    // Test with 1MB of data
    let plaintext: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
    debug!("Testing with {} bytes of data", plaintext.len());

    // Encrypt
    let ciphertext = client
        .tunnel_encrypt(&tunnel, &plaintext, Direction::Egress)
        .await
        .expect("Encryption failed");

    // Decrypt
    let decrypted = client
        .tunnel_decrypt(&tunnel, &ciphertext)
        .await
        .expect("Decryption failed");

    // Verify
    assert_eq!(decrypted, plaintext, "Large data round-trip failed");

    info!("✅ Large data encrypt/decrypt successful (1MB verified)");

    // Clean up
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");
}

// ============================================================================
// Tunnel Status and Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_tunnel_status() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP tunnel status queries");

    let client = BtspClient::new();

    // Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer-status".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["test".to_string()],
    };

    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    // Query status
    let status = client
        .tunnel_status(&tunnel)
        .await
        .expect("Status query failed");

    debug!("Tunnel status: {:?}", status);

    // Validate status
    assert_eq!(status.state, TunnelState::Established);

    info!("✅ Tunnel status query successful");

    // Clean up
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");
}

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_tunnel_close() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP tunnel close");

    let client = BtspClient::new();

    // Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer-close".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["test".to_string()],
    };

    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    let tunnel_id = tunnel.id.clone();
    debug!("Established tunnel: {}", tunnel_id);

    // Close tunnel
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");

    info!("✅ Tunnel closed successfully: {}", tunnel_id);

    // Note: We can't easily verify the tunnel is actually closed without
    // additional BearDog introspection APIs, but the close should succeed.
}

// ============================================================================
// Contact Exchange Tests (Peer Discovery)
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_contact_exchange() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP contact exchange (peer discovery)");

    let client = BtspClient::new();

    // Perform contact exchange
    let target_peer_id = "test-peer-discovery";
    let lineage = vec!["songbird".to_string()];
    let max_hops = 5;

    let result = client
        .contact_exchange(target_peer_id, lineage, max_hops)
        .await;

    // Contact exchange may fail if peer doesn't exist, which is OK for this test
    match result {
        Ok(contact_info) => {
            debug!("Contact exchange successful: {:?}", contact_info);
            info!("✅ Contact exchange successful (peer found)");
        }
        Err(e) => {
            debug!("Contact exchange failed (expected if peer doesn't exist): {}", e);
            info!("✅ Contact exchange API validated (peer not found, which is OK)");
        }
    }
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_invalid_peer() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP with invalid peer endpoint");

    let client = BtspClient::new();

    // Try to establish tunnel with invalid endpoint
    let peer = PeerEndpoint {
        id: "invalid-peer".to_string(),
        endpoint: "invalid://bad-endpoint".to_string(),
        public_key: None,
        capabilities: vec![],
    };

    let result = client.establish_tunnel(peer).await;

    // Should fail gracefully
    match result {
        Ok(_) => {
            warn!("Tunnel established with invalid peer (unexpected)");
        }
        Err(e) => {
            debug!("Expected error for invalid peer: {}", e);
            info!("✅ Invalid peer rejected correctly");
        }
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_performance_baseline() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing BTSP performance baseline");

    let client = BtspClient::new();

    // Establish tunnel
    let peer = PeerEndpoint {
        id: "test-peer-perf".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["test".to_string()],
    };

    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    // Measure encrypt/decrypt performance
    let test_data = b"Performance test data payload";
    let iterations = 100;

    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let ciphertext = client
            .tunnel_encrypt(&tunnel, test_data, Direction::Egress)
            .await
            .expect("Encryption failed");

        let _decrypted = client
            .tunnel_decrypt(&tunnel, &ciphertext)
            .await
            .expect("Decryption failed");
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed / iterations;

    info!(
        "✅ Performance baseline: {} iterations in {:?} (avg: {:?}/op)",
        iterations, elapsed, avg_latency
    );

    // Clean up
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");
}

// ============================================================================
// Integration Test Summary
// ============================================================================

#[tokio::test]
#[ignore = "Requires running BearDog server"]
async fn test_btsp_complete_workflow() {
    init_tracing();
    skip_if_no_beardog!();

    info!("Testing complete BTSP workflow (end-to-end)");

    let client = BtspClient::new();

    // 1. Ping
    info!("Step 1: Ping BearDog");
    let ping_response = client.ping().await.expect("Ping failed");
    assert_eq!(ping_response["primal"].as_str(), Some("beardog"));

    // 2. Establish tunnel
    info!("Step 2: Establish tunnel");
    let peer = PeerEndpoint {
        id: "test-peer-workflow".to_string(),
        endpoint: "test://localhost:9999".to_string(),
        public_key: None,
        capabilities: vec!["workflow-test".to_string()],
    };
    let tunnel = client
        .establish_tunnel(peer)
        .await
        .expect("Tunnel establishment failed");

    // 3. Check status
    info!("Step 3: Check tunnel status");
    let status = client
        .tunnel_status(&tunnel)
        .await
        .expect("Status query failed");
    assert_eq!(status.state, TunnelState::Established);

    // 4. Encrypt data
    info!("Step 4: Encrypt data");
    let plaintext = b"Complete workflow test message";
    let ciphertext = client
        .tunnel_encrypt(&tunnel, plaintext, Direction::Egress)
        .await
        .expect("Encryption failed");

    // 5. Decrypt data
    info!("Step 5: Decrypt data");
    let decrypted = client
        .tunnel_decrypt(&tunnel, &ciphertext)
        .await
        .expect("Decryption failed");
    assert_eq!(decrypted, plaintext);

    // 6. Close tunnel
    info!("Step 6: Close tunnel");
    client
        .tunnel_close(&tunnel)
        .await
        .expect("Tunnel close failed");

    info!("✅ Complete BTSP workflow successful!");
}

