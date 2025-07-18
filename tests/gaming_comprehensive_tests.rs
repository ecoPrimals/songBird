use songbird_errors::Result;
use songbird_network::gaming::production_lan::{ProductionLanConfig, ProductionLanManager};
use songbird_network::gaming::types::IpxAddress;
use songbird_network::gaming::{
    BridgeStatus, DetectedGameSession, GameProtocolClass, GameSessionStatus, GamingManager,
    NatType, PlayerEndpoint, VirtualNetwork,
};
use std::time::Duration;

#[tokio::test]
async fn test_gaming_network_manager_creation() -> Result<()> {
    let manager = GamingManager::new().await?;
    assert!(manager.lan_sessions.read().await.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_gaming_protocol_detection() -> Result<()> {
    let mut manager = GamingManager::new().await?;

    // Test protocol detection by scanning for games
    let detected_games = manager.scan_for_games(Some("lo".to_string())).await?;
    assert_eq!(detected_games.len(), 0); // No games running in test environment

    Ok(())
}

#[tokio::test]
async fn test_gaming_session_creation() -> Result<()> {
    let mut manager = GamingManager::new().await?;

    // Create a mock detected game session
    let session = DetectedGameSession {
        session_id: "test_session".to_string(),
        protocol_class: GameProtocolClass::IpxBased,
        local_ports: vec![6112],
        remote_endpoints: vec!["192.168.1.1:6112".parse().unwrap()],
        process_id: None,
        game_name: Some("Test Game".to_string()),
        detected_at: std::time::SystemTime::now(),
        confidence: 0.9,
    };

    // Test bridge creation
    let bridge_id = manager.create_bridge(&session).await?;
    assert!(!bridge_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_protocol_classes() -> Result<()> {
    let protocols = vec![
        GameProtocolClass::IpxBased,
        GameProtocolClass::DirectPlay,
        GameProtocolClass::UdpBroadcast,
        GameProtocolClass::TcpHostClient,
        GameProtocolClass::BattleNet,
        GameProtocolClass::GameSpy,
        GameProtocolClass::QuakeProtocol,
        GameProtocolClass::DoomProtocol,
        GameProtocolClass::UnknownLearning,
    ];

    for protocol in protocols {
        assert!(!format!("{protocol:?}").is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_player_endpoint_creation() -> Result<()> {
    let endpoint = PlayerEndpoint {
        player_id: "player1".to_string(),
        display_name: "Test Player".to_string(),
        real_address: "192.168.1.100:6112".parse().unwrap(),
        virtual_address: Some("10.0.0.1".parse().unwrap()),
        nat_type: NatType::None,
    };

    assert_eq!(endpoint.player_id, "player1");
    assert_eq!(endpoint.display_name, "Test Player");
    // Don't compare NatType directly as it doesn't implement PartialEq
    assert!(matches!(endpoint.nat_type, NatType::None));

    Ok(())
}

#[tokio::test]
async fn test_detected_game_session_creation() -> Result<()> {
    let session = DetectedGameSession {
        session_id: "test_session_001".to_string(),
        protocol_class: GameProtocolClass::IpxBased,
        local_ports: vec![6112, 6113],
        remote_endpoints: vec!["192.168.1.1:6112".parse().unwrap()],
        process_id: Some(1234),
        game_name: Some("StarCraft".to_string()),
        detected_at: std::time::SystemTime::now(),
        confidence: 0.95,
    };

    assert_eq!(session.session_id, "test_session_001");
    assert_eq!(session.protocol_class, GameProtocolClass::IpxBased);
    assert_eq!(session.local_ports.len(), 2);
    assert_eq!(session.confidence, 0.95);

    Ok(())
}

#[tokio::test]
async fn test_virtual_network_creation() -> Result<()> {
    let mut players = std::collections::HashMap::new();
    players.insert(
        "player1".to_string(),
        IpxAddress {
            network: 0x12345678,
            node: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            socket: 6112,
        },
    );
    players.insert(
        "player2".to_string(),
        IpxAddress {
            network: 0x12345678,
            node: [0x00, 0x11, 0x22, 0x33, 0x44, 0x66],
            socket: 6112,
        },
    );

    let network = VirtualNetwork::IPX {
        network_id: 123456,
        players,
        broadcast_enabled: true,
    };

    match network {
        VirtualNetwork::IPX {
            network_id,
            players,
            broadcast_enabled,
        } => {
            assert_eq!(network_id, 123456);
            assert!(broadcast_enabled);
            assert!(!players.is_empty());
        }
        _ => panic!("Expected IPX network type"),
    }

    Ok(())
}

#[tokio::test]
async fn test_gaming_manager_auto_configure() -> Result<()> {
    let manager = GamingManager::new().await?;
    let config = manager.auto_configure().await?;

    // Basic validation of the configuration structure - check it has discovery settings
    assert!(!config.discovery.discovery_ports.is_empty()); // Should have discovery ports

    Ok(())
}

#[tokio::test]
async fn test_production_lan_config_creation() -> Result<()> {
    let config = ProductionLanConfig::default();

    // Check that the config has the expected structure
    assert!(!config.discovery.discovery_ports.is_empty()); // Should have discovery ports
                                                           // Note: encryption is enabled by default in the environment config

    Ok(())
}

#[tokio::test]
async fn test_production_lan_manager_creation() -> Result<()> {
    let config = ProductionLanConfig::default();
    let manager = ProductionLanManager::new(config).await?;

    // Check that we can get the manager's statistics
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 0); // No active sessions initially

    Ok(())
}

#[tokio::test]
async fn test_gaming_protocol_equality() -> Result<()> {
    let protocol1 = GameProtocolClass::IpxBased;
    let protocol2 = GameProtocolClass::IpxBased;
    let protocol3 = GameProtocolClass::DirectPlay;

    assert_eq!(protocol1, protocol2);
    assert_ne!(protocol1, protocol3);

    Ok(())
}

#[tokio::test]
async fn test_nat_type_variants() -> Result<()> {
    let nat_types = vec![
        NatType::None,
        NatType::Open,
        NatType::FullCone,
        NatType::RestrictedCone,
        NatType::PortRestrictedCone,
        NatType::Symmetric,
        NatType::Unknown,
    ];

    for nat_type in nat_types {
        // Check that all NAT types can be formatted
        assert!(!format!("{nat_type:?}").is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_bridge_status_creation() -> Result<()> {
    let status = BridgeStatus {
        active_sessions: 5,
        protocols_active: vec![GameProtocolClass::IpxBased, GameProtocolClass::DirectPlay],
        total_players: 20,
        uptime: Duration::from_secs(3600),
    };

    assert_eq!(status.active_sessions, 5);
    assert_eq!(status.protocols_active.len(), 2);
    assert_eq!(status.total_players, 20);
    assert_eq!(status.uptime.as_secs(), 3600);

    Ok(())
}

#[tokio::test]
async fn test_game_session_status_variants() -> Result<()> {
    let statuses = vec![
        GameSessionStatus::Creating,
        GameSessionStatus::Active,
        GameSessionStatus::Waiting,
        GameSessionStatus::Error("Test error".to_string()),
        GameSessionStatus::Closed,
    ];

    for status in statuses {
        assert!(!format!("{status:?}").is_empty());
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_gaming_manager() -> Result<()> {
    let manager = std::sync::Arc::new(GamingManager::new().await?);
    let mut handles = vec![];

    // Test concurrent access to the manager
    for _ in 0..5 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let sessions = manager_clone.get_active_sessions().await;
            sessions.is_empty() // Should be empty in test environment
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        assert!(handle.await.unwrap());
    }

    Ok(())
}

#[tokio::test]
async fn test_gaming_manager_session_management() -> Result<()> {
    let manager = GamingManager::new().await?;

    // Get initial session count
    let initial_sessions = manager.get_active_sessions().await;
    assert_eq!(initial_sessions.len(), 0);

    // Test bridge status retrieval
    let bridge_status = manager.get_bridge_status().await?;
    assert_eq!(bridge_status.len(), 0); // No active bridges in test environment

    Ok(())
}

#[tokio::test]
async fn test_gaming_serialization() -> Result<()> {
    let session = DetectedGameSession {
        session_id: "serialization_test".to_string(),
        protocol_class: GameProtocolClass::BattleNet,
        local_ports: vec![6112],
        remote_endpoints: vec!["192.168.1.1:6112".parse().unwrap()],
        process_id: Some(5678),
        game_name: Some("Diablo II".to_string()),
        detected_at: std::time::SystemTime::now(),
        confidence: 0.85,
    };

    // Test serialization
    let serialized = serde_json::to_string(&session)?;
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: DetectedGameSession = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized.session_id, session.session_id);
    assert_eq!(deserialized.protocol_class, session.protocol_class);
    assert_eq!(deserialized.confidence, session.confidence);

    Ok(())
}

#[tokio::test]
async fn test_virtual_network_variants() -> Result<()> {
    let ipx_network = VirtualNetwork::IPX {
        network_id: 1000,
        players: std::collections::HashMap::new(),
        broadcast_enabled: true,
    };

    let udp_network = VirtualNetwork::UDP {
        subnet: "192.168.1.0/24".to_string(),
        players: std::collections::HashMap::new(),
        broadcast_address: "192.168.1.255".parse().unwrap(),
    };

    let tcp_network = VirtualNetwork::TCP {
        host_address: "192.168.1.1:6112".parse().unwrap(),
        players: std::collections::HashMap::new(),
    };

    // Test that all network types can be created
    assert!(matches!(ipx_network, VirtualNetwork::IPX { .. }));
    assert!(matches!(udp_network, VirtualNetwork::UDP { .. }));
    assert!(matches!(tcp_network, VirtualNetwork::TCP { .. }));

    Ok(())
}

#[tokio::test]
async fn test_gaming_performance() -> Result<()> {
    let start_time = std::time::Instant::now();

    // Test performance of creating multiple gaming managers
    for _ in 0..10 {
        let _manager = GamingManager::new().await?;
    }

    let elapsed = start_time.elapsed();

    // Manager creation should be reasonably fast
    assert!(elapsed < Duration::from_secs(5));

    Ok(())
}
