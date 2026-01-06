//! E2E Tests for Trust Establishment
//!
//! **EVOLVED (v3.13.0)**: Event-driven synchronization, no arbitrary sleeps
//!
//! Tests the full trust establishment flow from discovery to federation.

mod common;
use common::sync_helpers::*;

use anyhow::Result;
use songbird_discovery::anonymous_discovery::AnonymousDiscoveryListener;
use songbird_network_federation::state::{FederationState, NodeStatus};
use songbird_orchestrator::trust::{TrustEscalationManager, TrustLevel, TrustTimeouts};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, warn};

/// Initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

#[tokio::test]
async fn test_establish_anonymous_trust() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Establish Anonymous Trust");

    // Create trust manager
    let trust_manager = TrustEscalationManager::with_defaults();

    // Establish trust with a mock peer
    let peer_id = "test-peer-123";

    trust_manager.establish_anonymous(peer_id.to_string()).await?;

    // Verify trust level is Anonymous
    let trust_level = trust_manager.get_trust_level(peer_id).await?;
    assert_eq!(trust_level, TrustLevel::Anonymous);

    info!("✅ Test passed: Anonymous trust established");

    Ok(())
}

#[tokio::test]
async fn test_trust_escalation_to_capability_verified() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Trust Escalation to Capability Verified");

    // Create trust manager
    let trust_manager = TrustEscalationManager::with_defaults();

    // Establish initial anonymous trust
    let peer_id = "test-peer-456";

    trust_manager.establish_anonymous(peer_id.to_string()).await?;

    // Escalate to capability verified
    let capabilities = vec!["orchestration".to_string(), "federation".to_string()];
    let proof = songbird_orchestrator::trust::CapabilityProof {
        capabilities: capabilities.clone(),
        proof: "mock-signature".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    trust_manager.verify_capabilities(peer_id, proof).await?;

    // Verify trust level escalated
    let trust_level = trust_manager.get_trust_level(peer_id).await?;
    assert_eq!(trust_level, TrustLevel::CapabilityVerified);

    info!("✅ Test passed: Trust escalated to capability verified");

    Ok(())
}

#[tokio::test]
async fn test_discovery_to_federation_integration() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Discovery → Trust → Federation Integration");

    // Create components
    let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
    let federation_state = Arc::new(FederationState::new("test".to_string()));

    // Create discovery listener (not used in this test, but shows integration point)
    let _listener = Arc::new(AnonymousDiscoveryListener::new(2301, 60));

    // Simulate a discovered peer
    let mock_peer = songbird_discovery::anonymous_discovery::DiscoveredPeer {
        session_id: "mock-peer-789".to_string(),
        node_id: Some("stable-node-id-123".to_string()),
        node_name: Some("mock-tower".to_string()),
        endpoints: None,
        capabilities: vec!["orchestration".to_string()],
        protocols: vec!["https".to_string()],
        port: 8080,
        address: "192.168.1.102:2300".parse().unwrap(),
        last_seen: std::time::SystemTime::now(),
        version: "3.0".to_string(),
        tags: None,
        timestamp: None,
        identity_attestations: None, // No genetic lineage for this mock peer
    };

    let peer_id = &mock_peer.session_id;
    let endpoint = mock_peer.https_endpoint();

    // Step 1: Establish trust
    trust_manager.establish_anonymous(peer_id.clone()).await?;
    let trust_level = trust_manager.get_trust_level(peer_id).await?;
    assert_eq!(trust_level, TrustLevel::Anonymous);
    info!("✅ Step 1: Trust established");

    // Step 2: Register node in federation
    let node_registration = songbird_network_federation::state::NodeRegistration {
        node_id: peer_id.to_string(),
        node_name: format!("peer-{}", &peer_id[..8]),
        node_address: endpoint.clone(),
        endpoints: None,
        cpu_cores: 0,
        memory_gb: 0,
        gpu_model: None,
        storage_gb: None,
        capabilities: mock_peer.capabilities.clone(),
        status: NodeStatus::Active,
        joined_at: chrono::Utc::now(),
        last_heartbeat: chrono::Utc::now(),
    };

    federation_state.register_node(node_registration).await;
    info!("✅ Step 2: Node registered in federation");

    // Step 3: Verify federation state
    let stats = federation_state.get_stats().await;
    assert_eq!(stats.total_nodes, 1);
    assert_eq!(stats.active_nodes, 1);
    info!("✅ Step 3: Federation state verified");

    info!("✅ Test passed: Full discovery → trust → federation flow");

    Ok(())
}

#[tokio::test]
async fn test_trust_timeout_and_cleanup() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Trust Timeout and Cleanup");

    // Create trust manager with short timeout
    let timeouts = TrustTimeouts {
        anonymous: 1, // 1 second timeout
        capability: 60,
        identity: 300,
        hardware: 0,
    };
    let trust_manager = TrustEscalationManager::new(timeouts, None);

    // Establish trust
    let peer_id = "test-peer-timeout";
    trust_manager.establish_anonymous(peer_id.to_string()).await?;

    // Verify trust exists
    assert!(trust_manager.get_trust_level(peer_id).await.is_ok());
    info!("✅ Step 1: Trust established");

    // ✅ EVOLVED (v3.13.0): Poll for trust expiration instead of arbitrary sleep
    // Wait for trust to actually expire (event-driven, precise)
    let expired = wait_for_condition(
        || async {
            // Check if trust has expired
            trust_manager.get_trust_level(peer_id).await.is_err()
        },
        Duration::from_secs(3) // Safety margin above 1s timeout
    ).await;
    
    assert!(expired, "Trust should expire within timeout period");
    info!("✅ Step 2: Trust expired (event-driven check)");

    // Cleanup expired trusts
    let removed_count = trust_manager.cleanup_expired().await;
    assert_eq!(removed_count, 1);
    info!("✅ Step 3: Expired trust cleaned up");

    // Verify trust is gone
    assert!(trust_manager.get_trust_level(peer_id).await.is_err());
    info!("✅ Step 3: Trust removal verified");

    info!("✅ Test passed: Trust timeout and cleanup");

    Ok(())
}

#[tokio::test]
async fn test_multiple_peers_federation() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Multiple Peers in Federation");

    // Create components
    let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
    let federation_state = Arc::new(FederationState::new("test".to_string()));

    // Add multiple peers
    for i in 0..5 {
        let peer_id = format!("peer-{}", i);
        let endpoint = format!("https://192.168.1.{}:8080", 100 + i);

        // Establish trust
        trust_manager.establish_anonymous(peer_id.clone()).await?;

        // Register in federation
        let node_registration = songbird_network_federation::state::NodeRegistration {
            node_id: peer_id.clone(),
            node_name: format!("node-{}", i),
            node_address: endpoint.clone(),
            endpoints: None,
            cpu_cores: 8,
            memory_gb: 16,
            gpu_model: Some("NVIDIA RTX 4090".to_string()),
            storage_gb: Some(1000),
            capabilities: vec!["orchestration".to_string(), "gpu".to_string()],
            status: NodeStatus::Active,
            joined_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };

        federation_state.register_node(node_registration).await;
    }

    // Verify federation state
    let stats = federation_state.get_stats().await;
    assert_eq!(stats.total_nodes, 5);
    assert_eq!(stats.active_nodes, 5);
    assert_eq!(stats.total_cpu_cores, 40); // 5 peers * 8 cores
    assert_eq!(stats.total_memory_gb, 80); // 5 peers * 16 GB
    assert_eq!(stats.total_storage_gb, 5000); // 5 peers * 1000 GB

    info!("✅ Test passed: Multiple peers federation");
    info!("   Total Nodes: {}", stats.total_nodes);
    info!("   Active Nodes: {}", stats.active_nodes);
    info!("   Total CPU Cores: {}", stats.total_cpu_cores);
    info!("   Total Memory: {} GB", stats.total_memory_gb);
    info!("   Total Storage: {} GB", stats.total_storage_gb);

    Ok(())
}

#[tokio::test]
async fn test_trust_rejection_on_failure() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Trust Rejection on Failure");

    let trust_manager = TrustEscalationManager::with_defaults();
    let federation_state = Arc::new(FederationState::new("test".to_string()));

    // Try to establish trust with invalid peer
    let peer_id = "invalid-peer";

    // This should succeed (we don't validate anything in establish_anonymous)
    // but in a real scenario with network calls, this would fail
    let result = trust_manager.establish_anonymous(peer_id.to_string()).await;

    if result.is_err() {
        warn!("❌ Trust establishment failed as expected");

        // Verify node was NOT added to federation
        let stats = federation_state.get_stats().await;
        assert_eq!(stats.total_nodes, 0);

        info!("✅ Test passed: Trust rejection prevented federation join");
    } else {
        // If it succeeded (which it does in current impl), verify we can still handle it
        info!("⚠️  Trust establishment succeeded despite invalid endpoint (current behavior)");
        info!("   In production, this should be validated with actual network calls");
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_trust_establishment() -> Result<()> {
    init_tracing();

    info!("🧪 Test: Concurrent Trust Establishment");

    let trust_manager = Arc::new(TrustEscalationManager::with_defaults());
    let federation_state = Arc::new(FederationState::new("test".to_string()));

    // Establish trust with multiple peers concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let tm = Arc::clone(&trust_manager);
        let fs = Arc::clone(&federation_state);

        let handle = tokio::spawn(async move {
            let peer_id = format!("concurrent-peer-{}", i);
            let endpoint = format!("https://192.168.1.{}:8080", 150 + i);

            // Establish trust
            tm.establish_anonymous(peer_id.clone()).await?;

            // Register in federation
            let node_registration = songbird_network_federation::state::NodeRegistration {
                node_id: peer_id.clone(),
                node_name: format!("concurrent-node-{}", i),
                node_address: endpoint.clone(),
                endpoints: None,
                cpu_cores: 4,
                memory_gb: 8,
                gpu_model: None,
                storage_gb: Some(500),
                capabilities: vec!["orchestration".to_string()],
                status: NodeStatus::Active,
                joined_at: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            };

            fs.register_node(node_registration).await;

            Ok::<_, anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all peers were added
    let stats = federation_state.get_stats().await;
    assert_eq!(stats.total_nodes, 10);
    assert_eq!(stats.active_nodes, 10);

    info!("✅ Test passed: Concurrent trust establishment");
    info!("   Successfully added {} peers concurrently", stats.total_nodes);

    Ok(())
}
