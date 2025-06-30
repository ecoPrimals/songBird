use std::collections::HashMap;
// Gaming Network Bridge - Unit Tests
//
// Comprehensive test suite for the universal gaming network bridge

pub mod protocol_detection;
pub mod protocol_translation;
pub mod bridge_management;
pub mod auto_configuration;
pub mod types_validation;

use songbird_gaming_bridge::network::gaming::*;
use songbird_gaming_bridge::errors::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::SystemTime;

/// Create a mock StarCraft game session for testing
pub fn create_mock_starcraft_session() -> DetectedGameSession {
    DetectedGameSession {
        session_id: "test_starcraft_123".to_string(),
        protocol_class: GameProtocolClass::IpxBased,
        local_ports: vec![6112, 6113, 6114],
        remote_endpoints: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 6112),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101)), 6112),
        ],
        process_id: Some(1234),
        game_name: Some("StarCraft: Brood War".to_string()),
        detected_at: SystemTime::now(),
        confidence: 0.95,
    }
}

/// Create a mock Age of Empires game session for testing
pub fn create_mock_aoe_session() -> DetectedGameSession {
    DetectedGameSession {
        session_id: "test_aoe_456".to_string(),
        protocol_class: GameProtocolClass::DirectPlay,
        local_ports: vec![2300, 2301],
        remote_endpoints: vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 102)), 2300),
        ],
        process_id: Some(5678),
        game_name: Some("Age of Empires II".to_string()),
        detected_at: SystemTime::now(),
        confidence: 0.85,
    }
}

/// Create mock raw packet data for testing
pub fn create_mock_ipx_packet() -> Vec<u8> {
    // Mock IPX packet with header
    vec![
        0xFF, 0xFF, // Checksum
        0x00, 0x20, // Length (32 bytes)
        0x00,       // Transport control
        0x04,       // Packet type (PEP)
        0x00, 0x00, 0x43, 0x21, // Dest network
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Dest node
        0x86, 0x9C, // Dest socket (StarCraft)
        0x00, 0x00, 0x43, 0x21, // Src network  
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // Src node
        0x86, 0x9C, // Src socket
        // Payload (game data)
        0x01, 0x02, 0x03, 0x04,
    ]
}

/// Create mock DirectPlay packet data
pub fn create_mock_directplay_packet() -> Vec<u8> {
    vec![
        0x00, 0x01, // DirectPlay header
        0x00, 0x10, // Message length
        0x02,       // Message type
        0x00, 0x00, 0x00, // Session ID
        // Payload
        0x41, 0x67, 0x65, 0x20, 0x6F, 0x66, 0x20, 0x45, // "Age of E"
        0x6D, 0x70, 0x69, 0x72, 0x65, 0x73, 0x00,       // "mpires\0"
    ]
}

/// Create a mock player endpoint
pub fn create_mock_player(id: &str, addr: &str) -> PlayerEndpoint {
    PlayerEndpoint {
        player_id: id.to_string(),
        display_name: format!("Player {}", id),
        real_address: addr.parse().unwrap_or_default(),
        virtual_address: None,
        nat_type: NatType::Unknown,
    }
}

/// Helper function to create test raw packets
pub fn create_test_raw_packet(data: Vec<u8>, src: &str, dst: &str) -> RawPacket {
    RawPacket {
        data,
        src_addr: src.parse().unwrap_or_default(),
        dst_addr: dst.parse().unwrap_or_default(),
        protocol: TransportProtocol::UDP,
        timestamp: SystemTime::now(),
    }
} 