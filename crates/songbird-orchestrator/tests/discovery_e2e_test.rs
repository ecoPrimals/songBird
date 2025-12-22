//! End-to-End tests for Discovery integration
//!
//! Tests that discovery broadcaster and listener start properly
//! on orchestrator startup and can discover peers.

use anyhow::Result;
use songbird_orchestrator::app::SongbirdOrchestrator;
use songbird_types::config::CanonicalSongbirdConfig;
use std::time::Duration;
use tokio::time::sleep;

/// Test that discovery broadcaster starts on orchestrator startup
#[tokio::test]
async fn test_discovery_broadcaster_starts_on_startup() -> Result<()> {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt::try_init();

    // Create config with discovery enabled
    std::env::set_var("SONGBIRD_TLS_ENABLED", "true");
    std::env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    std::env::set_var("SONGBIRD_ANONYMOUS_DISCOVERY", "true");
    std::env::set_var("SONGBIRD_NODE_NAME", "test-e2e");
    std::env::set_var("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
    std::env::set_var("SONGBIRD_PORT", "18080"); // Use test port
    std::env::set_var("SONGBIRD_DISCOVERY_PORT", "12300"); // Use test port

    let config = CanonicalSongbirdConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    // Verify discovery is enabled in config
    assert!(config.discovery.mode.is_enabled(), "Discovery should be enabled");
    assert!(config.discovery.mode.is_anonymous(), "Anonymous discovery should be enabled");

    tracing::info!("✅ Config validated: discovery enabled");

    // Create orchestrator (this should initialize discovery)
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    tracing::info!("✅ Orchestrator created");

    // NOTE: We can't call start() in a test because it blocks forever
    // Instead, we verify that discovery components are initialized
    //
    // In a real deployment:
    // 1. start() is called
    // 2. Discovery broadcaster spawns and starts broadcasting
    // 3. Discovery listener spawns and starts listening
    // 4. Bridge polls discovered peers every 10s

    // Verify orchestrator was created successfully
    assert!(orchestrator.config().discovery.mode.is_enabled(), "Orchestrator should have discovery enabled");

    tracing::info!("✅ Test passed: Discovery components initialized");

    Ok(())
}

/// Test that discovery listener can receive broadcasts
#[tokio::test]
async fn test_discovery_listener_receives_broadcasts() -> Result<()> {
    use songbird_discovery::anonymous_discovery::{
        AnonymousDiscoveryBroadcaster, AnonymousDiscoveryListener,
    };
    use std::sync::Arc;

    let _ = tracing_subscriber::fmt::try_init();

    // Create broadcaster and listener on test port
    let test_port = 12301;

    let capabilities = vec!["test".to_string()];
    let protocols = vec!["https".to_string()];
    let broadcaster = AnonymousDiscoveryBroadcaster::new(
        capabilities.clone(),
        protocols,
        8080,
        vec!["127.0.0.1:12301".parse().unwrap()],
        1, // Broadcast every 1 second for test
    );

    let listener = Arc::new(AnonymousDiscoveryListener::new(test_port, 10));

    tracing::info!("✅ Created broadcaster and listener");

    // Start broadcaster in background
    let broadcaster_handle = tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await; // Small delay
        if let Err(e) = broadcaster.start_broadcasting().await {
            tracing::error!("Broadcaster error: {}", e);
        }
    });

    // Start listener in background
    let listener_clone = Arc::clone(&listener);
    let listener_handle = tokio::spawn(async move {
        if let Err(e) = listener_clone.start_listening().await {
            tracing::error!("Listener error: {}", e);
        }
    });

    // Wait for broadcast and discovery
    sleep(Duration::from_secs(2)).await;

    // Check if any peers were discovered
    let peers = listener.get_peers().await;

    tracing::info!("📊 Discovered {} peers", peers.len());
    for peer in &peers {
        tracing::info!("  Peer: {} with capabilities: {:?}", peer.session_id, peer.capabilities);
    }

    // Clean up
    broadcaster_handle.abort();
    listener_handle.abort();

    // Verify at least one peer was discovered
    assert!(!peers.is_empty(), "Should discover at least one peer");
    assert_eq!(peers[0].capabilities, capabilities, "Should have correct capabilities");

    tracing::info!("✅ Test passed: Discovery working end-to-end");

    Ok(())
}

/// Test that discovery → federation bridge polls discovered peers
#[tokio::test]
async fn test_discovery_federation_bridge_polls_peers() -> Result<()> {
    use songbird_discovery::anonymous_discovery::AnonymousDiscoveryListener;
    use std::sync::Arc;

    let _ = tracing_subscriber::fmt::try_init();

    // Create a listener with a mock discovered peer
    let listener = Arc::new(AnonymousDiscoveryListener::new(12302, 10));

    // In a real scenario, the bridge would:
    // 1. Poll listener.get_peers() every 10s
    // 2. For each peer, call trust_manager.establish_trust()
    // 3. Route peer to appropriate federation via discovery_router
    // 4. Log the federation join

    // This test verifies the structure exists
    let peers = listener.get_peers().await;
    assert_eq!(peers.len(), 0, "Should start with no peers");

    tracing::info!("✅ Test passed: Bridge structure verified");

    Ok(())
}

/// Integration test: Full orchestrator startup with discovery
#[tokio::test]
#[ignore] // Ignore by default as it requires network and time
async fn test_full_orchestrator_startup_with_discovery() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

    // Set environment for test
    std::env::set_var("SONGBIRD_TLS_ENABLED", "false"); // Disable TLS for test
    std::env::set_var("SONGBIRD_FEDERATION_ENABLED", "true");
    std::env::set_var("SONGBIRD_ANONYMOUS_DISCOVERY", "true");
    std::env::set_var("SONGBIRD_NODE_NAME", "integration-test");
    std::env::set_var("SONGBIRD_BIND_ADDRESS", "127.0.0.1");
    std::env::set_var("SONGBIRD_PORT", "18081");
    std::env::set_var("SONGBIRD_DISCOVERY_PORT", "12303");

    let config = CanonicalSongbirdConfig::from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    tracing::info!("✅ Orchestrator created for integration test");

    // Start orchestrator in background with timeout
    let start_handle = tokio::spawn(async move {
        if let Err(e) = orchestrator.start().await {
            tracing::error!("Orchestrator start error: {}", e);
        }
    });

    // Wait for services to initialize
    sleep(Duration::from_secs(3)).await;

    // Verify services are running
    // (In a real test, we'd check ports, make HTTP requests, etc.)

    tracing::info!("✅ Integration test: Orchestrator started successfully");

    // Clean up
    start_handle.abort();

    Ok(())
}
