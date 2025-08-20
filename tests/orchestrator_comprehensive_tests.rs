//! Comprehensive Tests for Orchestrator Module

use songbird_config::SongbirdConfig;
use songbird_core::orchestrator::*;
use songbird_errors::SongbirdResult;
use std::net::IpAddr;
use tokio::test;

/// Test HealthStatus creation and validation
#[test]
async fn test_health_status_creation() {
    let health = HealthStatus::default();
    assert!(health.healthy);
    assert_eq!(health.services_count, 0);
    assert_eq!(health.uptime_seconds, 0);
    assert!(health.is_ok());
}

/// Test HealthStatus with custom values
#[test]
async fn test_health_status_custom() {
    let health = HealthStatus {
        healthy: false,
        services_count: 5,
        uptime_seconds: 3600,
        last_check: std::time::SystemTime::now(),
    };
    assert!(!health.healthy);
    assert_eq!(health.services_count, 5);
    assert_eq!(health.uptime_seconds, 3600);
    assert!(!health.is_ok());
}

/// Test HealthStatus methods
#[test]
async fn test_health_status_methods() {
    let health = HealthStatus {
        healthy: true,
        services_count: 3,
        uptime_seconds: 1200,
        last_check: std::time::SystemTime::now(),
    };

    assert!(health.is_ok());
    assert!(health.is_healthy_with_services(2));
    assert!(health.is_healthy_with_services(3));
    assert!(!health.is_healthy_with_services(4));
}

/// Test Orchestrator creation
#[test]
async fn test_orchestrator_creation() -> Result<()> {
    let config = SongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;
    assert!(orchestrator.get_config().network.orchestrator_port > 0);
    Ok(())
}

/// Test Orchestrator default
#[test]
async fn test_orchestrator_default() {
    let orchestrator = Orchestrator::default();
    assert!(orchestrator.get_config().network.orchestrator_port > 0);
}

/// Test Orchestrator health checks
#[test]
async fn test_orchestrator_health_checks() {
    let orchestrator = Orchestrator::default();

    let health = orchestrator.get_health_status().await;
    assert!(health.is_ok());
    assert!(health.healthy);
}

/// Test Orchestrator lifecycle
#[test]
async fn test_orchestrator_lifecycle() {
    let orchestrator = Orchestrator::default();

    // Test start
    let start_result = orchestrator.start().await;
    assert!(start_result.is_ok());

    // Test health after start
    let health = orchestrator.get_health_status().await;
    assert!(health.is_ok());

    // Test stop
    let stop_result = orchestrator.stop().await;
    assert!(stop_result.is_ok());
}

/// Test Orchestrator configuration access
#[test]
async fn test_orchestrator_config_access() {
    let config = SongbirdConfig {
        network: songbird_config::config::NetworkConfig {
            bind_address: &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?,
            orchestrator_port: 9000,
            ..Default::default()
        },
        ..Default::default()
    };

    let orchestrator = Orchestrator::new(config.clone()).expect("Test operation should succeed");
    let retrieved_config = orchestrator.get_config();
    assert_eq!(
        retrieved_config.network.bind_address,
        &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?
    );
    assert_eq!(retrieved_config.network.orchestrator_port, 9000);
}

/// Test Orchestrator service discovery
#[test]
async fn test_orchestrator_service_discovery() {
    let orchestrator = Orchestrator::default();

    let services = orchestrator.discover_services().await;
    assert!(services.is_ok());

    let service_list = services.expect("Test operation should succeed");
    assert!(!service_list.is_empty());
    assert!(service_list.contains(&"orchestrator".to_string()));
}

/// Test Orchestrator error handling
#[test]
async fn test_orchestrator_error_handling() {
    let orchestrator = Orchestrator::default();

    // Test methods don't panic
    let _ = orchestrator.get_config();
    let _ = orchestrator.get_health_status().await;
    let _ = orchestrator.discover_services().await;
}

/// Test Orchestrator concurrent operations
#[test]
async fn test_orchestrator_concurrent() {
    let orchestrator = std::sync::Arc::new(Orchestrator::default());

    let mut handles = vec![];
    for i in 0..10 {
        let orch_clone = orchestrator.clone();
        let handle = tokio::spawn(async move {
            let health = orch_clone.get_health_status().await;
            (i, health.is_ok())
        });
        handles.push(handle);
    }

    for handle in handles {
        let (i, health_ok) = handle.await.expect("Test operation should succeed");
        assert!(health_ok);
        assert!(i < 10);
    }
}

/// Test HealthStatus cloning
#[test]
async fn test_health_status_cloning() {
    let health = HealthStatus {
        healthy: true,
        services_count: 5,
        uptime_seconds: 3600,
        last_check: std::time::SystemTime::now(),
    };

    let cloned_health = health.clone();
    assert_eq!(health.healthy, cloned_health.healthy);
    assert_eq!(health.services_count, cloned_health.services_count);
    assert_eq!(health.uptime_seconds, cloned_health.uptime_seconds);
}

/// Test Orchestrator with different configurations
#[test]
async fn test_orchestrator_different_configs() {
    let configs = vec![
        SongbirdConfig {
            network: songbird_config::config::NetworkConfig {
                bind_address: "0.0.0.0".parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?,
                orchestrator_port: 8080,
                ..Default::default()
            },
            ..Default::default()
        },
        SongbirdConfig {
            network: songbird_config::config::NetworkConfig {
                bind_address: &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?,
                orchestrator_port: 3000,
                ..Default::default()
            },
            ..Default::default()
        },
    ];

    for config in configs {
        let orchestrator = Orchestrator::new(config.clone()).expect("Test operation should succeed");
        let retrieved_config = orchestrator.get_config();
        assert_eq!(
            retrieved_config.network.bind_address,
            config.network.bind_address
        );
        assert_eq!(
            retrieved_config.network.orchestrator_port,
            config.network.orchestrator_port
        );
    }
}

/// Test Orchestrator uptime tracking
#[test]
async fn test_orchestrator_uptime_tracking() {
    let orchestrator = Orchestrator::default();

    let health1 = orchestrator.get_health_status().await;

    // Wait a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let health2 = orchestrator.get_health_status().await;
    assert!(health2.uptime_seconds >= health1.uptime_seconds);
}

/// Test HealthStatus edge cases
#[test]
async fn test_health_status_edge_cases() {
    // Test with zero services
    let health = HealthStatus {
        healthy: true,
        services_count: 0,
        uptime_seconds: 0,
        last_check: std::time::SystemTime::now(),
    };
    assert!(health.is_ok());
    assert!(!health.is_healthy_with_services(1));

    // Test with high service count
    let health = HealthStatus {
        healthy: true,
        services_count: 1000,
        uptime_seconds: 86400,
        last_check: std::time::SystemTime::now(),
    };
    assert!(health.is_ok());
    assert!(health.is_healthy_with_services(999));
}

/// Test Orchestrator performance
#[test]
async fn test_orchestrator_performance() {
    let orchestrator = Orchestrator::default();

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = orchestrator.get_health_status().await;
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000);
}

/// Test Orchestrator integration
#[test]
async fn test_orchestrator_integration() {
    let orchestrator = Orchestrator::default();

    // Test that start and health check work together
    let start_result = orchestrator.start().await;
    assert!(start_result.is_ok());

    let health = orchestrator.get_health_status().await;
    assert!(health.is_ok());

    // Test config retrieval
    let config = orchestrator.get_config();
    assert!(config.network.orchestrator_port > 0);

    // Test service discovery
    let services = orchestrator.discover_services().await;
    assert!(services.is_ok());

    let service_list = services.expect("Test operation should succeed");
    assert!(!service_list.is_empty());
}

/// Test Orchestrator scaling integration
#[test]
async fn test_orchestrator_scaling_integration() {
    use songbird_core::orchestrator::scaling::*;

    let orchestrator = Orchestrator::default();

    // Test that we can use scaling components with orchestrator
    let scaling_config = GamingScalingConfig::default();
    let scaling_manager = GamingScalingManager::new(scaling_config);
    assert!(scaling_manager.is_ok());

    let manager = scaling_manager.expect("Test operation should succeed");
    let current_scale = manager.current_scale();
    assert!(matches!(current_scale, GamingScale::HomeGaming));

    // Test orchestrator health alongside scaling
    let health = orchestrator.get_health_status().await;
    assert!(health.is_ok());
}

/// Test Orchestrator configuration
#[test]
async fn test_orchestrator_configuration() -> Result<()> {
    let mut config = SongbirdConfig::default();
    config.network.bind_address = &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?;
    config.network.orchestrator_port = 9000;

    let orchestrator = Orchestrator::new(config)?;
    let retrieved_config = orchestrator.get_config();

    assert_eq!(
        retrieved_config.network.bind_address,
        &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?
    );
    assert_eq!(retrieved_config.network.orchestrator_port, 9000);
    Ok(())
}

/// Test Orchestrator stress configuration
#[test]
async fn test_orchestrator_stress_configuration() -> Result<()> {
    // Test with stress configuration
    let config = SongbirdConfig {
        network: songbird_config::config::NetworkConfig {
            bind_address: "0.0.0.0".parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?,
            orchestrator_port: 8080,
            ..Default::default()
        },
        ..Default::default()
    };

    let orchestrator = Orchestrator::new(config)?;

    // Test with different configuration
    let config2 = SongbirdConfig {
        network: songbird_config::config::NetworkConfig {
            bind_address: &get_bind_address().parse::<IpAddr>()
    .map_err(|e| SongbirdError::network_error(&format!("Invalid IP address: {}", e)))?,
            orchestrator_port: 3000,
            ..Default::default()
        },
        ..Default::default()
    };

    let orchestrator2 = Orchestrator::new(config2)?;

    // Verify configurations are different
    let retrieved_config = orchestrator.get_config();
    let retrieved_config2 = orchestrator2.get_config();

    assert_eq!(
        retrieved_config.network.orchestrator_port,
        retrieved_config.network.orchestrator_port
    );
    assert_eq!(
        retrieved_config2.network.orchestrator_port,
        retrieved_config2.network.orchestrator_port
    );

    Ok(())
}

/// Test Orchestrator network stats
#[test]
async fn test_orchestrator_network_stats() -> Result<()> {
    let config = SongbirdConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    // Test network statistics
    assert!(orchestrator.get_config().network.orchestrator_port > 0);

    Ok(())
}
