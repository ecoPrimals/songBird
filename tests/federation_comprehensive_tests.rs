//! Comprehensive Federation Testing Suite
//!
//! Tests covering the actual federation functionality including:
//! - FederationManager lifecycle
//! - Federation modes (Standalone, Cluster, Federation)
//! - Heartbeat functionality
//! - Status management
//! - Error handling and edge cases

use std::collections::HashMap;
use std::time::Duration;
use chrono::Utc;

use songbird_orchestrator::federation::{
    FederationConfig, FederationManager, FederationMode, FederationStatus,
};

#[cfg(test)]
mod federation_comprehensive_tests {
    use super::*;

    // ============================================================================
    // TEST UTILITIES
    // ============================================================================

    fn create_test_federation_config(mode: FederationMode) -> FederationConfig {
        FederationConfig {
            mode,
            cluster_name: "test-cluster".to_string(),
            heartbeat_interval: Duration::from_secs(10),
            peer_discovery_enabled: true,
        }
    }

    // ============================================================================
    // FEDERATION CONFIG TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_config_default() {
        let config = FederationConfig::default();

        assert!(matches!(config.mode, FederationMode::Standalone));
        assert_eq!(config.cluster_name, "default-cluster");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert!(config.peer_discovery_enabled);
    }

    #[tokio::test]
    async fn test_federation_config_custom_standalone() {
        let config = FederationConfig {
            mode: FederationMode::Standalone,
            cluster_name: "standalone-cluster".to_string(),
            heartbeat_interval: Duration::from_secs(60),
            peer_discovery_enabled: false,
        };

        assert!(matches!(config.mode, FederationMode::Standalone));
        assert_eq!(config.cluster_name, "standalone-cluster");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
        assert!(!config.peer_discovery_enabled);
    }

    #[tokio::test]
    async fn test_federation_config_custom_cluster() {
        let config = FederationConfig {
            mode: FederationMode::Cluster,
            cluster_name: "production-cluster".to_string(),
            heartbeat_interval: Duration::from_secs(15),
            peer_discovery_enabled: true,
        };

        assert!(matches!(config.mode, FederationMode::Cluster));
        assert_eq!(config.cluster_name, "production-cluster");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(15));
        assert!(config.peer_discovery_enabled);
    }

    #[tokio::test]
    async fn test_federation_config_custom_federation() {
        let config = FederationConfig {
            mode: FederationMode::Federation,
            cluster_name: "multi-org-federation".to_string(),
            heartbeat_interval: Duration::from_secs(45),
            peer_discovery_enabled: true,
        };

        assert!(matches!(config.mode, FederationMode::Federation));
        assert_eq!(config.cluster_name, "multi-org-federation");
        assert_eq!(config.heartbeat_interval, Duration::from_secs(45));
        assert!(config.peer_discovery_enabled);
    }

    // ============================================================================
    // FEDERATION MANAGER CREATION TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_manager_creation_standalone() {
        let config = create_test_federation_config(FederationMode::Standalone);
        let manager = FederationManager::new(config);

        assert!(matches!(manager.get_mode(), FederationMode::Standalone));
        
        let status = manager.get_status();
        assert!(matches!(status.mode, FederationMode::Standalone));
        assert!(status.cluster_id.is_none());
        assert!(status.last_heartbeat.is_none());
        assert_eq!(status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_creation_cluster() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let manager = FederationManager::new(config);

        assert!(matches!(manager.get_mode(), FederationMode::Cluster));
        
        let status = manager.get_status();
        assert!(matches!(status.mode, FederationMode::Cluster));
        assert!(status.cluster_id.is_none());
        assert!(status.last_heartbeat.is_none());
        assert_eq!(status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_creation_federation() {
        let config = create_test_federation_config(FederationMode::Federation);
        let manager = FederationManager::new(config);

        assert!(matches!(manager.get_mode(), FederationMode::Federation));
        
        let status = manager.get_status();
        assert!(matches!(status.mode, FederationMode::Federation));
        assert!(status.cluster_id.is_none());
        assert!(status.last_heartbeat.is_none());
        assert_eq!(status.connected_peers.len(), 0);
    }

    // ============================================================================
    // FEDERATION MANAGER LIFECYCLE TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_manager_start_standalone() {
        let config = create_test_federation_config(FederationMode::Standalone);
        let mut manager = FederationManager::new(config);

        let result = manager.start().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert!(status.last_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_federation_manager_start_cluster() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let mut manager = FederationManager::new(config);

        let result = manager.start().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert!(status.last_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_federation_manager_start_federation() {
        let config = create_test_federation_config(FederationMode::Federation);
        let mut manager = FederationManager::new(config);

        let result = manager.start().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert!(status.last_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_federation_manager_stop_standalone() {
        let config = create_test_federation_config(FederationMode::Standalone);
        let mut manager = FederationManager::new(config);

        // Start first to populate some state
        manager.start().await.unwrap();

        let result = manager.stop().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert_eq!(status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_stop_cluster() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let mut manager = FederationManager::new(config);

        // Start first to populate some state
        manager.start().await.unwrap();

        let result = manager.stop().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert_eq!(status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_stop_federation() {
        let config = create_test_federation_config(FederationMode::Federation);
        let mut manager = FederationManager::new(config);

        // Start first to populate some state
        manager.start().await.unwrap();

        let result = manager.stop().await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert_eq!(status.connected_peers.len(), 0);
    }

    // ============================================================================
    // HEARTBEAT TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_manager_send_heartbeat_standalone() {
        let config = create_test_federation_config(FederationMode::Standalone);
        let manager = FederationManager::new(config);

        let result = manager.send_heartbeat().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_federation_manager_send_heartbeat_cluster() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let manager = FederationManager::new(config);

        let result = manager.send_heartbeat().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_federation_manager_send_heartbeat_federation() {
        let config = create_test_federation_config(FederationMode::Federation);
        let manager = FederationManager::new(config);

        let result = manager.send_heartbeat().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_federation_manager_multiple_heartbeats() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let manager = FederationManager::new(config);

        // Send multiple heartbeats
        for _ in 0..5 {
            let result = manager.send_heartbeat().await;
            assert!(result.is_ok());
        }
    }

    // ============================================================================
    // FEDERATION LIFECYCLE TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_manager_lifecycle_standalone() {
        let config = create_test_federation_config(FederationMode::Standalone);
        let mut manager = FederationManager::new(config);

        // Initial state
        let initial_status = manager.get_status();
        assert!(initial_status.last_heartbeat.is_none());
        assert_eq!(initial_status.connected_peers.len(), 0);

        // Start
        manager.start().await.unwrap();
        let started_status = manager.get_status();
        assert!(started_status.last_heartbeat.is_some());

        // Send heartbeat
        manager.send_heartbeat().await.unwrap();

        // Stop
        manager.stop().await.unwrap();
        let stopped_status = manager.get_status();
        assert_eq!(stopped_status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_lifecycle_cluster() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let mut manager = FederationManager::new(config);

        // Initial state
        let initial_status = manager.get_status();
        assert!(matches!(initial_status.mode, FederationMode::Cluster));
        assert!(initial_status.last_heartbeat.is_none());

        // Start
        manager.start().await.unwrap();
        let started_status = manager.get_status();
        assert!(started_status.last_heartbeat.is_some());

        // Send heartbeat
        manager.send_heartbeat().await.unwrap();

        // Stop
        manager.stop().await.unwrap();
        let stopped_status = manager.get_status();
        assert_eq!(stopped_status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_lifecycle_federation() {
        let config = create_test_federation_config(FederationMode::Federation);
        let mut manager = FederationManager::new(config);

        // Initial state
        let initial_status = manager.get_status();
        assert!(matches!(initial_status.mode, FederationMode::Federation));
        assert!(initial_status.last_heartbeat.is_none());

        // Start
        manager.start().await.unwrap();
        let started_status = manager.get_status();
        assert!(started_status.last_heartbeat.is_some());

        // Send heartbeat
        manager.send_heartbeat().await.unwrap();

        // Stop
        manager.stop().await.unwrap();
        let stopped_status = manager.get_status();
        assert_eq!(stopped_status.connected_peers.len(), 0);
    }

    // ============================================================================
    // MULTIPLE INSTANCES TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_manager_multiple_instances() {
        // Test that multiple federation managers can coexist
        let config1 = FederationConfig {
            mode: FederationMode::Standalone,
            cluster_name: "instance1".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            peer_discovery_enabled: true,
        };

        let config2 = FederationConfig {
            mode: FederationMode::Cluster,
            cluster_name: "instance2".to_string(),
            heartbeat_interval: Duration::from_secs(60),
            peer_discovery_enabled: false,
        };

        let mut manager1 = FederationManager::new(config1);
        let mut manager2 = FederationManager::new(config2);

        // Start both managers
        manager1.start().await.unwrap();
        manager2.start().await.unwrap();

        // Verify they have different configurations
        assert!(matches!(manager1.get_mode(), FederationMode::Standalone));
        assert!(matches!(manager2.get_mode(), FederationMode::Cluster));

        // Send heartbeats from both
        manager1.send_heartbeat().await.unwrap();
        manager2.send_heartbeat().await.unwrap();

        // Stop both managers
        manager1.stop().await.unwrap();
        manager2.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_federation_manager_multiple_start_stop_cycles() {
        let config = create_test_federation_config(FederationMode::Federation);
        let mut manager = FederationManager::new(config);

        // Multiple start/stop cycles
        for _ in 0..3 {
            manager.start().await.unwrap();
            let status = manager.get_status();
            assert!(status.last_heartbeat.is_some());

            manager.stop().await.unwrap();
            let status = manager.get_status();
            assert_eq!(status.connected_peers.len(), 0);
        }
    }

    // ============================================================================
    // FEDERATION MODE SERIALIZATION TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_mode_serialization() {
        use serde_json;

        let standalone = FederationMode::Standalone;
        let cluster = FederationMode::Cluster;
        let federation = FederationMode::Federation;

        // Test serialization of all modes
        let standalone_json = serde_json::to_string(&standalone).unwrap();
        let cluster_json = serde_json::to_string(&cluster).unwrap();
        let federation_json = serde_json::to_string(&federation).unwrap();

        // Test deserialization
        let standalone_deser: FederationMode = serde_json::from_str(&standalone_json).unwrap();
        let cluster_deser: FederationMode = serde_json::from_str(&cluster_json).unwrap();
        let federation_deser: FederationMode = serde_json::from_str(&federation_json).unwrap();

        assert!(matches!(standalone_deser, FederationMode::Standalone));
        assert!(matches!(cluster_deser, FederationMode::Cluster));
        assert!(matches!(federation_deser, FederationMode::Federation));
    }

    #[tokio::test]
    async fn test_federation_status_serialization() {
        let status = FederationStatus {
            mode: FederationMode::Cluster,
            cluster_id: Some("test-cluster-123".to_string()),
            last_heartbeat: Some(Utc::now()),
            connected_peers: HashMap::from([
                ("peer1".to_string(), "192.168.1.10".to_string()),
                ("peer2".to_string(), "192.168.1.11".to_string()),
            ]),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: FederationStatus = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.mode, FederationMode::Cluster));
        assert_eq!(deserialized.cluster_id, Some("test-cluster-123".to_string()));
        assert!(deserialized.last_heartbeat.is_some());
        assert_eq!(deserialized.connected_peers.len(), 2);
    }

    #[tokio::test]
    async fn test_federation_config_serialization() {
        let config = FederationConfig {
            mode: FederationMode::Federation,
            cluster_name: "serialization-test".to_string(),
            heartbeat_interval: Duration::from_secs(120),
            peer_discovery_enabled: false,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: FederationConfig = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized.mode, FederationMode::Federation));
        assert_eq!(deserialized.cluster_name, "serialization-test");
        assert_eq!(deserialized.heartbeat_interval, Duration::from_secs(120));
        assert!(!deserialized.peer_discovery_enabled);
    }

    // ============================================================================
    // EDGE CASES AND ERROR HANDLING TESTS
    // ============================================================================

    #[tokio::test]
    async fn test_federation_config_edge_cases() {
        // Very short heartbeat interval
        let config = FederationConfig {
            mode: FederationMode::Cluster,
            cluster_name: "edge-case-cluster".to_string(),
            heartbeat_interval: Duration::from_millis(1),
            peer_discovery_enabled: false,
        };

        let manager = FederationManager::new(config);
        assert!(matches!(manager.get_mode(), FederationMode::Cluster));
    }

    #[tokio::test]
    async fn test_federation_config_long_cluster_name() {
        // Very long cluster name
        let long_name = "a".repeat(1000);
        let config = FederationConfig {
            mode: FederationMode::Federation,
            cluster_name: long_name.clone(),
            heartbeat_interval: Duration::from_secs(30),
            peer_discovery_enabled: true,
        };

        let manager = FederationManager::new(config);
        assert!(matches!(manager.get_mode(), FederationMode::Federation));
    }

    #[tokio::test]
    async fn test_federation_config_zero_heartbeat_interval() {
        // Zero heartbeat interval
        let config = FederationConfig {
            mode: FederationMode::Standalone,
            cluster_name: "zero-heartbeat".to_string(),
            heartbeat_interval: Duration::from_secs(0),
            peer_discovery_enabled: true,
        };

        let manager = FederationManager::new(config);
        let result = manager.send_heartbeat().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_federation_status_empty_peers() {
        let config = create_test_federation_config(FederationMode::Cluster);
        let mut manager = FederationManager::new(config);

        manager.start().await.unwrap();
        let status = manager.get_status();
        assert_eq!(status.connected_peers.len(), 0);

        manager.stop().await.unwrap();
        let status = manager.get_status();
        assert_eq!(status.connected_peers.len(), 0);
    }

    #[tokio::test]
    async fn test_federation_manager_concurrent_operations() {
        let config = create_test_federation_config(FederationMode::Federation);
        let manager = FederationManager::new(config);

        // Concurrent heartbeats (should all succeed since heartbeat is read-only)
        let heartbeat_tasks = (0..5).map(|_| {
            let manager = &manager;
            async move {
                manager.send_heartbeat().await
            }
        });

        let results = futures::future::join_all(heartbeat_tasks).await;
        for result in results {
            assert!(result.is_ok());
        }
    }
} 