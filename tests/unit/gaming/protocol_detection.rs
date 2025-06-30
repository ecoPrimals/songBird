use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
// Protocol Detection Tests
//
// Tests for the universal game protocol detector

use super::*;
use tokio;

#[tokio::test]
async fn test_detector_initialization() {
    let detector = UniversalGameProtocolDetector::new();
    let result = detector.initialize().await;
    
    assert!(result.is_ok(), "Detector should initialize successfully");
}

#[tokio::test] 
async fn test_scan_network_finds_games() {
    let detector = UniversalGameProtocolDetector::new();
    detector.initialize().await.unwrap_or_default();
    
    let sessions = detector.scan_network(Some("eth0".to_string())).await.unwrap_or_default();
    
    assert!(!sessions.is_empty(), "Should detect at least one game session");
    assert!(sessions.len() >= 2, "Should detect multiple game types");
}

#[tokio::test]
async fn test_starcraft_detection() {
    let detector = UniversalGameProtocolDetector::new();
    detector.initialize().await.unwrap_or_default();
    
    let sessions = detector.detect_game_traffic("test_interface").await.unwrap_or_default();
    
    let starcraft_session = sessions.iter()
        .find(|s| s.protocol_class == GameProtocolClass::IpxBased);
    
    assert!(starcraft_session.is_some(), "Should detect StarCraft session");
    
    let session = starcraft_session.unwrap_or_default();
    assert!(session.local_ports.contains(&6112), "Should detect StarCraft port 6112");
    assert!(session.confidence > 0.8, "Should have high confidence for known game");
    assert_eq!(session.game_name, Some("StarCraft".to_string()));
}

#[tokio::test]
async fn test_age_of_empires_detection() {
    let detector = UniversalGameProtocolDetector::new();
    detector.initialize().await.unwrap_or_default();
    
    let sessions = detector.detect_game_traffic("test_interface").await.unwrap_or_default();
    
    let aoe_session = sessions.iter()
        .find(|s| s.protocol_class == GameProtocolClass::DirectPlay);
    
    assert!(aoe_session.is_some(), "Should detect Age of Empires session");
    
    let session = aoe_session.unwrap_or_default();
    assert!(session.local_ports.contains(&2300), "Should detect DirectPlay port 2300");
    assert_eq!(session.game_name, Some("Age of Empires II".to_string()));
}

#[tokio::test]
async fn test_protocol_learning() {
    let detector = UniversalGameProtocolDetector::new();
    detector.initialize().await.unwrap_or_default();
    
    // Create mock packets for learning
    let packets = vec![
        create_test_raw_packet(
            create_mock_ipx_packet(),
            "192.168.1.100:6112",
            "192.168.1.101:6112"
        ),
    ];
    
    let hints = vec!["starcraft".to_string(), "ipx".to_string()];
    
    let result = detector.learn_protocol("Test Game", &packets, &hints).await;
    
    assert!(result.is_ok(), "Should learn protocol successfully");
    
    let signature = result.unwrap_or_default();
    assert_eq!(signature.protocol_class, GameProtocolClass::IpxBased);
    assert!(signature.ports.contains(&6112), "Should extract port from packets");
}

#[tokio::test]
async fn test_protocol_class_determination() {
    let detector = UniversalGameProtocolDetector::new();
    
    // Test IPX hints
    let ipx_hints = vec!["starcraft".to_string(), "ipx".to_string()];
    let packets = vec![];
    let class = detector.determine_protocol_class(&ipx_hints, &packets).await;
    assert_eq!(class, GameProtocolClass::IpxBased);
    
    // Test DirectPlay hints
    let dp_hints = vec!["directplay".to_string(), "windows".to_string()];
    let class = detector.determine_protocol_class(&dp_hints, &packets).await;
    assert_eq!(class, GameProtocolClass::DirectPlay);
    
    // Test UDP hints
    let udp_hints = vec!["udp".to_string(), "broadcast".to_string()];
    let class = detector.determine_protocol_class(&udp_hints, &packets).await;
    assert_eq!(class, GameProtocolClass::UdpBroadcast);
}

#[tokio::test]
async fn test_confidence_scoring() {
    let detector = UniversalGameProtocolDetector::new();
    detector.initialize().await.unwrap_or_default();
    
    let sessions = detector.scan_network(None).await.unwrap_or_default();
    
    for session in sessions {
        assert!(session.confidence >= 0.0 && session.confidence <= 1.0, 
               "Confidence should be between 0.0 and 1.0");
        
        if session.game_name.is_some() {
            assert!(session.confidence > 0.7, 
                   "Known games should have high confidence");
        }
    }
}

#[test]
fn test_session_id_generation() {
    // Test that session IDs are unique
    let mut ids = std::collections::HashSet::new();
    
    for _ in 0..100 {
        let session = create_mock_starcraft_session();
        assert!(ids.insert(session.session_id), "Session IDs should be unique");
    }
}

#[test]
fn test_protocol_signature_validation() {
    let signature = ProtocolSignature {
        protocol_class: GameProtocolClass::IpxBased,
        ports: vec![6112, 6113],
        packet_patterns: vec![
            PacketPattern {
                offset: 0,
                pattern: vec![0xFF, 0xFF],
                mask: None,
                description: Some("IPX header").to_string(),
            }
        ],
        timing_characteristics: TimingCharacteristics {
            packet_interval_ms: Some(50),
            burst_patterns: true,
            real_time_sensitive: true,
            turn_based: false,
        },
        discovery_method: DiscoveryMethod::IpxBroadcast,
    };
    
    // Validate signature structure
    assert!(!signature.ports.is_empty(), "Should have at least one port");
    assert!(!signature.packet_patterns.is_empty(), "Should have at least one pattern");
    assert!(signature.timing_characteristics.real_time_sensitive, "StarCraft should be real-time sensitive");
} 