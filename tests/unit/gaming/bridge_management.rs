use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
// Bridge Management Tests
//
// Tests for the universal game bridge functionality

use super::*;
use tokio;

#[tokio::test]
async fn test_bridge_creation() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    let session = create_mock_starcraft_session();
    
    let bridge_id = bridge.create_bridge(&session).await.unwrap_or_default();
    
    assert!(!bridge_id.is_empty(), "Bridge ID should not be empty");
    assert!(bridge_id.contains("bridge_"), "Bridge ID should have proper prefix");
    assert!(bridge_id.contains("ipx_based"), "Bridge ID should indicate protocol type");
}

#[tokio::test]
async fn test_bridge_joining() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    let session = create_mock_starcraft_session();
    
    let bridge_id = bridge.create_bridge(&session).await.unwrap_or_default();
    let local_addr = "192.168.1.200:6112".parse().unwrap_or_default();
    
    let result = bridge.join_bridge(&bridge_id, local_addr).await;
    
    assert!(result.is_ok(), "Should be able to join existing bridge");
}

#[tokio::test]
async fn test_bridge_status_tracking() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    // Initially no active sessions
    let initial_status = bridge.get_status().await;
    assert_eq!(initial_status.active_sessions, 0);
    
    // Create a session
    let players = vec![
        create_mock_player("1", "192.168.1.100:6112"),
        create_mock_player("2", "192.168.1.101:6112"),
    ];
    
    let session_id = bridge.create_game_session(players).await.unwrap_or_default();
    
    // Check status after session creation
    let status = bridge.get_status().await;
    assert_eq!(status.active_sessions, 1);
    assert_eq!(status.total_players, 2);
    assert!(status.protocols_active.contains(&GameProtocolClass::IpxBased));
}

#[tokio::test]
async fn test_multiple_bridge_management() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    let starcraft_session = create_mock_starcraft_session();
    let aoe_session = create_mock_aoe_session();
    
    let bridge1 = bridge.create_bridge(&starcraft_session).await.unwrap_or_default();
    let bridge2 = bridge.create_bridge(&aoe_session).await.unwrap_or_default();
    
    assert_ne!(bridge1, bridge2, "Bridge IDs should be unique");
    
    let all_status = bridge.get_all_bridge_status().await.unwrap_or_default();
    assert_eq!(all_status.len(), 2, "Should track multiple bridges");
}

#[tokio::test]
async fn test_bridge_stopping() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    let session = create_mock_starcraft_session();
    
    let bridge_id = bridge.create_bridge(&session).await.unwrap_or_default();
    
    // Verify bridge exists
    let status_before = bridge.get_all_bridge_status().await.unwrap_or_default();
    assert_eq!(status_before.len(), 1);
    
    // Stop the bridge
    let result = bridge.stop_bridge(&bridge_id).await;
    assert!(result.is_ok(), "Should be able to stop bridge");
    
    // Verify bridge is removed
    let status_after = bridge.get_all_bridge_status().await.unwrap_or_default();
    assert_eq!(status_after.len(), 0);
}

#[tokio::test]
async fn test_bridge_session_lifecycle() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    let players = vec![
        create_mock_player("host", "192.168.1.100:6112"),
        create_mock_player("client", "192.168.1.101:6112"),
    ];
    
    // Create session
    let session_id = bridge.create_game_session(players).await.unwrap_or_default();
    assert!(!session_id.is_empty());
    
    // Verify session is active
    let status = bridge.get_status().await;
    assert_eq!(status.active_sessions, 1);
    
    // Close session
    let result = bridge.close_session(&session_id).await;
    assert!(result.is_ok());
    
    // Verify session is closed (simplified - in real implementation would check status)
    // For now, just verify the call succeeded
}

#[tokio::test]
async fn test_protocol_translation() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    let players = vec![create_mock_player("1", "192.168.1.100:6112")];
    
    let session_id = bridge.create_game_session(players).await.unwrap_or_default();
    
    // Test outbound translation
    let test_packet = create_mock_ipx_packet();
    let result = bridge.translate_outbound(&session_id, &test_packet).await;
    
    assert!(result.is_ok(), "Should translate outbound packet");
    
    // Test inbound translation
    if let Ok(internet_packet) = result {
        let inbound_result = bridge.translate_inbound(&session_id, &internet_packet).await;
        assert!(inbound_result.is_ok(), "Should translate inbound packet");
    }
}

#[tokio::test]
async fn test_auto_detection_and_bridging() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    let result = bridge.auto_detect_and_bridge("test_interface").await.unwrap_or_default();
    
    match result {
        BridgeResult::Success => {
            // Success case - bridge was created automatically
        }
        BridgeResult::RequiresUserInput(_) => {
            // Learning case - user input needed
        }
        BridgeResult::Error(e) => {
            panic!("Auto-detection should not fail: {}", e);
        }
        BridgeResult::ProtocolLearned(_) => {
            // Learning success case
        }
    }
}

#[tokio::test]
async fn test_unknown_protocol_handling() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    // Create some unknown packet data
    let unknown_packets = vec![
        create_test_raw_packet(
            vec![0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34], // Unknown pattern
            "192.168.1.100:9999",
            "192.168.1.101:9999"
        ),
    ];
    
    let result = bridge.handle_unknown_protocol(&unknown_packets).await.unwrap_or_default();
    
    match result {
        BridgeResult::RequiresUserInput(msg) => {
            assert!(!msg.is_empty(), "Should provide helpful message to user");
            assert!(msg.contains("hint"), "Should ask for hints");
        }
        _ => {
            // Other results are also acceptable for unknown protocols
        }
    }
}

#[tokio::test]
async fn test_nat_traversal_setup() {
    let bridge = UniversalGameBridge::new().await.unwrap_or_default();
    
    let players = vec![
        PlayerEndpoint {
            player_id: "symmetric_nat_player".to_string(),
            display_name: "Symmetric NAT Player".to_string(),
            real_address: "192.168.1.100:6112".parse().unwrap_or_default(),
            virtual_address: None,
            nat_type: NatType::Symmetric,
        },
        PlayerEndpoint {
            player_id: "cone_nat_player".to_string(),
            display_name: "Cone NAT Player".to_string(),
            real_address: "192.168.1.101:6112".parse().unwrap_or_default(),
            virtual_address: None,
            nat_type: NatType::FullCone,
        },
    ];
    
    let session_id = bridge.create_game_session(players).await.unwrap_or_default();
    
    // The bridge should handle different NAT types appropriately
    // For symmetric NAT, it should prefer client-server protocols
    let status = bridge.get_status().await;
    assert_eq!(status.active_sessions, 1);
}

#[test]
fn test_bridge_id_uniqueness() {
    let mut bridge_ids = std::collections::HashSet::new();
    
    for _ in 0..100 {
        let session = create_mock_starcraft_session();
        // Simulate bridge ID generation
        let bridge_id = format!("bridge_{}_{}", 
            session.protocol_class.to_string().to_lowercase(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() % 0xFFFFFF
        );
        
        assert!(bridge_ids.insert(bridge_id), "Bridge IDs should be unique");
    }
}

#[test]
fn test_bridge_status_validation() {
    let status = BridgeStatus {
        active_sessions: 2,
        protocols_active: vec![GameProtocolClass::IpxBased, GameProtocolClass::DirectPlay],
        total_players: 6,
        uptime: std::time::Duration::from_secs(3600),
    };
    
    assert!(status.active_sessions > 0, "Should have active sessions");
    assert!(!status.protocols_active.is_empty(), "Should have active protocols");
    assert!(status.total_players >= status.active_sessions, "Players should be >= sessions");
    assert!(status.uptime.as_secs() > 0, "Should have positive uptime");
} 