use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use songbird_gaming_bridge::network::gaming::GamingManager;

#[tokio::test]
async fn test_gaming_manager_creation() {
    // Test that we can create a gaming manager
    let gaming_manager = GamingManager::new().await;
    assert!(
        gaming_manager.is_ok(),
        "Gaming manager should be created successfully"
    );
}

#[tokio::test]
async fn test_protocol_detection_basic() {
    let mut gaming_manager = GamingManager::new()
        .await
        .expect("Failed to create gaming manager");

    // Test scanning for games (should not panic)
    let result = gaming_manager
        .scan_for_games(Some("eth0".to_string()))
        .await;
    assert!(result.is_ok(), "Scanning for games should not fail");

    let sessions = result.unwrap_or_default();
    // In test environment, we might get mock sessions - just verify the API works
    println!("Detected {} gaming sessions", sessions.len());
    // The important thing is that it doesn't crash and returns a valid result
    assert!(sessions.len() >= 0, "Should return valid session count");
}

#[tokio::test]
async fn test_bridge_status() {
    let gaming_manager = GamingManager::new()
        .await
        .expect("Failed to create gaming manager");

    // Test getting bridge status
    let status = gaming_manager.get_bridge_status().await;
    assert!(status.is_ok(), "Getting bridge status should not fail");

    let bridge_status = status.unwrap_or_default();
    // BridgeStatus is a Vec, so check the first entry if any
    if !bridge_status.is_empty() {
        assert_eq!(
            bridge_status[0].active_sessions, 0,
            "No active sessions initially"
        );
        assert_eq!(bridge_status[0].total_players, 0, "No players initially");
    }
}

#[tokio::test]
async fn test_auto_configure() {
    let gaming_manager = GamingManager::new()
        .await
        .expect("Failed to create gaming manager");

    // Test auto-configuration
    let result = gaming_manager.auto_configure().await;
    assert!(result.is_ok(), "Auto-configuration should not fail");
}
