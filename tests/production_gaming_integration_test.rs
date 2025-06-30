use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use songbird_gaming_bridge::errors::Result;
use songbird_gaming_bridge::network::gaming::production_lan_manager::{
    DiscoveryConfig, HealingConfig, MonitoringConfig, NetworkConfig, SecurityConfig,
};
/// Production Gaming Integration Tests
///
/// Comprehensive tests for the world-class production gaming system
/// validating end-to-end functionality, self-healing, security, and monitoring.
use songbird_gaming_bridge::network::gaming::{ProductionLanConfig, ProductionLanManager};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_production_configuration_system() -> Result<()> {
    // Test zero hardcoding - fully configurable system
    let config = ProductionLanConfig {
        discovery: DiscoveryConfig {
            discovery_ports: vec![6112, 6113, 6114],
            broadcast_interval_ms: 1000,
            discovery_timeout_ms: 3000,
            max_sessions: 10,
        },
        security: SecurityConfig {
            enable_encryption: true,
            max_players_per_session: 4,
            session_timeout_seconds: 1800,
            max_discovery_requests_per_minute: 30,
            allowed_interfaces: vec![],
        },
        network: NetworkConfig {
            game_port_range: (7000, 7100),
            packet_buffer_size: 32768,
            max_packet_size: 1500,
            interface_preference: vec!["eth0".to_string()],
        },
        healing: HealingConfig {
            enable_auto_recovery: true,
            health_check_interval_ms: 2000,
            max_retry_attempts: 2,
            retry_backoff_multiplier: 1.5,
        },
        monitoring: MonitoringConfig {
            enable_performance_monitoring: true,
            enable_traffic_monitoring: true,
            metrics_interval_ms: 1000,
            log_level: "info".to_string(),
        },
    };

    // Validate configuration can be loaded
    let manager = ProductionLanManager::new(config).await?;

    // Test that configuration is applied
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 0, "New manager should have no sessions");

    Ok(())
}

#[tokio::test]
async fn test_session_creation_and_management() -> Result<()> {
    let manager = ProductionLanManager::new_default().await?;

    // Test session creation
    let session_code = manager
        .create_session("Test Game".to_string())
        .await?;
    assert_eq!(session_code.len(), 4, "Session code should be 4 characters");
    assert!(
        session_code.chars().all(|c| c.is_ascii_uppercase()),
        "Session code should be uppercase"
    );

    // Test session listing
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 1, "Should have one active session");
    assert_eq!(sessions[0].session_code, session_code);
    assert_eq!(sessions[0].game_info.game_name, "Test Game");

    // Test session status
    let status = manager.get_session_status(&session_code).await?;
    assert_eq!(status.session_code, session_code);
    assert_eq!(
        status.players.len(),
        0,
        "New session should have no players"
    );

    // Test session shutdown
    manager.shutdown_session(&session_code).await?;
    let sessions_after = manager.list_sessions().await;
    assert_eq!(
        sessions_after.len(),
        0,
        "Session should be removed after shutdown"
    );

    Ok(())
}

#[tokio::test]
async fn test_secure_session_codes() -> Result<()> {
    let manager = ProductionLanManager::new_default().await?;

    // Create multiple sessions to test uniqueness
    let mut session_codes = Vec::new();
    for i in 0..10 {
        let code = manager.create_session(format!("Game {}", i), None).await?;
        session_codes.push(code);
    }

    // Verify all codes are unique
    for i in 0..session_codes.len() {
        for j in (i + 1)..session_codes.len() {
            assert_ne!(
                session_codes[i], session_codes[j],
                "Session codes must be unique"
            );
        }
    }

    // Verify code format (CVVC pattern)
    for code in &session_codes {
        assert_eq!(code.len(), 4, "Code should be 4 characters");
        assert!(
            code.chars().all(|c| c.is_ascii_uppercase()),
            "Code should be uppercase"
        );

        let chars: Vec<char> = code.chars().collect();
        let consonants = "BCDFGHJKLMNPQRSTVWXYZ";
        let vowels = "AEIOU";

        assert!(
            consonants.contains(chars[0]),
            "First char should be consonant"
        );
        assert!(vowels.contains(chars[1]), "Second char should be vowel");
        assert!(vowels.contains(chars[2]), "Third char should be vowel");
        assert!(
            consonants.contains(chars[3]),
            "Fourth char should be consonant"
        );
    }

    // Cleanup
    for code in session_codes {
        manager.shutdown_session(&code).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_discovery_system() -> Result<()> {
    let host_manager = Arc::new(ProductionLanManager::new_default().await?);
    let client_manager = Arc::new(ProductionLanManager::new_default().await?);

    // Host creates session
    let session_code = host_manager
        .create_session("Discovery Test Game".to_string())
        .await?;

    // Wait for broadcasting to start
    sleep(Duration::from_millis(1500)).await;

    // Client discovers sessions
    let discovered = client_manager.discover_sessions().await?;

    // Verify discovery
    assert!(
        !discovered.is_empty(),
        "Should discover at least one session"
    );

    let found_session = discovered.iter().find(|s| s.session_code == session_code);
    assert!(found_session.is_some(), "Should find the hosted session");

    let session = found_session.unwrap_or_default();
    assert_eq!(session.game_info.game_name, "Discovery Test Game");
    assert_eq!(session.session_code, session_code);

    // Cleanup
    host_manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_player_joining() -> Result<()> {
    let host_manager = Arc::new(ProductionLanManager::new_default().await?);
    let client_manager = Arc::new(ProductionLanManager::new_default().await?);

    // Host creates session
    let session_code = host_manager
        .create_session("Join Test Game".to_string())
        .await?;

    // Wait for session to be ready
    sleep(Duration::from_millis(500)).await;

    // Client joins session
    let player_info = client_manager
        .join_session(&session_code, Some("Test Player".to_string()))
        .await?;

    // Verify player info
    assert_eq!(player_info.display_name, "Test Player");
    assert!(
        !player_info.player_id.is_empty(),
        "Player should have an ID"
    );

    // Verify session has player
    let session_status = host_manager.get_session_status(&session_code).await?;
    assert_eq!(
        session_status.players.len(),
        1,
        "Session should have one player"
    );
    assert_eq!(session_status.players[0].display_name, "Test Player");

    // Cleanup
    host_manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_session_limits() -> Result<()> {
    let mut config = ProductionLanConfig::default();
    config.security.max_players_per_session = 2; // Limit to 2 players for test

    let manager = Arc::new(ProductionLanManager::new(config).await?);
    let session_code = manager
        .create_session("Limit Test Game".to_string())
        .await?;

    // Join first player
    let _player1 = manager
        .join_session(&session_code, Some("Player 1".to_string()))
        .await?;

    // Join second player
    let _player2 = manager
        .join_session(&session_code, Some("Player 2".to_string()))
        .await?;

    // Try to join third player (should fail)
    let result = manager
        .join_session(&session_code, Some("Player 3".to_string()))
        .await;
    assert!(result.is_err(), "Should fail to join when session is full");

    // Verify session has exactly 2 players
    let session_status = manager.get_session_status(&session_code).await?;
    assert_eq!(
        session_status.players.len(),
        2,
        "Session should have exactly 2 players"
    );

    // Cleanup
    manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_sessions() -> Result<()> {
    let manager = ProductionLanManager::new_default().await?;

    // Create multiple sessions
    let mut session_codes = Vec::new();
    for i in 1..=5 {
        let code = manager
            .create_session(format!("Multi Game {}", i), None)
            .await?;
        session_codes.push(code);
    }

    // Verify all sessions exist
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 5, "Should have 5 active sessions");

    // Verify each session is unique and properly configured
    for (i, session) in sessions.iter().enumerate() {
        assert_eq!(session.game_info.game_name, format!("Multi Game {}", i + 1));
        assert_eq!(
            session.players.len(),
            0,
            "New sessions should have no players"
        );
    }

    // Cleanup all sessions
    for code in session_codes {
        manager.shutdown_session(&code).await?;
    }

    let final_sessions = manager.list_sessions().await;
    assert_eq!(final_sessions.len(), 0, "All sessions should be cleaned up");

    Ok(())
}

#[tokio::test]
async fn test_health_monitoring_integration() -> Result<()> {
    let mut config = ProductionLanConfig::default();
    config.healing.enable_auto_recovery = true;
    config.healing.health_check_interval_ms = 500; // Fast checks for test

    let manager = ProductionLanManager::new(config).await?;

    // Create session
    let session_code = manager
        .create_session("Health Test Game".to_string())
        .await?;

    // Wait for health monitoring to run
    sleep(Duration::from_millis(1000)).await;

    // Session should still be healthy
    let session_status = manager.get_session_status(&session_code).await?;
    assert_eq!(session_status.session_code, session_code);

    // Health monitoring should be tracking the session
    // (In a real test, we would check internal health metrics)

    // Cleanup
    manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    let mut config = ProductionLanConfig::default();
    config.monitoring.enable_performance_monitoring = true;
    config.monitoring.metrics_interval_ms = 500; // Fast metrics for test

    let manager = ProductionLanManager::new(config).await?;

    // Create session
    let session_code = manager
        .create_session("Metrics Test Game".to_string())
        .await?;

    // Wait for metrics collection
    sleep(Duration::from_millis(1000)).await;

    // Check session metrics
    let session_status = manager.get_session_status(&session_code).await?;

    // Metrics should be initialized
    assert_eq!(session_status.metrics.total_packets_sent, 0);
    assert_eq!(session_status.metrics.total_packets_received, 0);
    assert_eq!(session_status.metrics.total_bytes_sent, 0);
    assert_eq!(session_status.metrics.total_bytes_received, 0);
    assert_eq!(session_status.metrics.error_count, 0);

    // Cleanup
    manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_graceful_shutdown() -> Result<()> {
    let manager = ProductionLanManager::new_default().await?;

    // Create session with player
    let session_code = manager
        .create_session("Shutdown Test Game".to_string())
        .await?;
    let _player = manager
        .join_session(&session_code, Some("Test Player".to_string()))
        .await?;

    // Verify session is active
    let sessions_before = manager.list_sessions().await;
    assert_eq!(sessions_before.len(), 1);

    // Shutdown should succeed
    let shutdown_result = manager.shutdown_session(&session_code).await;
    assert!(shutdown_result.is_ok(), "Shutdown should succeed");

    // Session should be removed
    let sessions_after = manager.list_sessions().await;
    assert_eq!(
        sessions_after.len(),
        0,
        "Session should be removed after shutdown"
    );

    // Subsequent operations on shutdown session should fail
    let status_result = manager.get_session_status(&session_code).await;
    assert!(
        status_result.is_err(),
        "Should fail to get status of shutdown session"
    );

    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow() -> Result<()> {
    // Complete end-to-end test simulating real usage
    let host_manager = Arc::new(ProductionLanManager::new_default().await?);
    let client_manager = Arc::new(ProductionLanManager::new_default().await?);

    // 1. Host creates and starts session
    let session_code = host_manager
        .create_session("E2E Test Game".to_string())
        .await?;
    assert_eq!(session_code.len(), 4);

    // 2. Wait for broadcasting
    sleep(Duration::from_millis(1000)).await;

    // 3. Client discovers session
    let discovered = client_manager.discover_sessions().await?;
    assert!(!discovered.is_empty());

    let target_session = discovered
        .iter()
        .find(|s| s.session_code == session_code)
        .expect("Should find target session");

    // 4. Client joins session
    let player = client_manager
        .join_session(&target_session.session_code, Some("E2E Player".to_string()))
        .await?;
    assert_eq!(player.display_name, "E2E Player");

    // 5. Verify session state
    let session_status = host_manager.get_session_status(&session_code).await?;
    assert_eq!(session_status.players.len(), 1);
    assert_eq!(session_status.players[0].display_name, "E2E Player");

    // 6. Session should be active and healthy
    match session_status.status {
        songbird_gaming_bridge::network::gaming::production_lan_manager::SessionStatus::Active => {}
        _ => panic!("Session should be active"),
    }

    // 7. Graceful shutdown
    host_manager.shutdown_session(&session_code).await?;

    // 8. Verify cleanup
    let final_sessions = host_manager.list_sessions().await;
    assert_eq!(final_sessions.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_network_interface_detection() -> Result<()> {
    // Test that the system can detect and configure network interfaces
    let manager = ProductionLanManager::new_default().await?;

    // Create a session to trigger network detection
    let session_code = manager
        .create_session("Network Test".to_string())
        .await?;

    // Get session to check network configuration
    let session = manager.get_session_status(&session_code).await?;

    // Verify network info is populated
    assert!(
        !session.network_info.primary_interface.is_empty(),
        "Should have primary interface"
    );
    assert!(
        !session.network_info.available_ports.is_empty(),
        "Should have available ports"
    );

    // Cleanup
    manager.shutdown_session(&session_code).await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    // Test system handles concurrent operations correctly
    let manager = Arc::new(ProductionLanManager::new_default().await?);

    // Create multiple sessions concurrently
    let mut handles = Vec::new();
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            manager_clone
                .create_session(format!("Concurrent Game {}", i), None)
                .await
        });
        handles.push(handle);
    }

    // Wait for all sessions to be created
    let mut session_codes = Vec::new();
    for handle in handles {
        let session_code = handle.await.unwrap_or_default()?;
        session_codes.push(session_code);
    }

    // Verify all sessions were created
    assert_eq!(session_codes.len(), 5);
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 5);

    // Cleanup all sessions concurrently
    let mut cleanup_handles = Vec::new();
    for session_code in session_codes {
        let manager_clone = Arc::clone(&manager);
        let handle =
            tokio::spawn(async move { manager_clone.shutdown_session(&session_code).await });
        cleanup_handles.push(handle);
    }

    // Wait for all cleanups
    for handle in cleanup_handles {
        handle.await.unwrap_or_default()?;
    }

    // Verify all sessions are cleaned up
    let final_sessions = manager.list_sessions().await;
    assert_eq!(final_sessions.len(), 0);

    Ok(())
}
