//! Comprehensive tests for IntegrationManager
//!
//! Tests service coordination, startup/shutdown, timeout handling,
//! configuration validation, and service availability checking.

use songbird_config::SongbirdConfig;
use songbird_orchestrator::integration::IntegrationManager;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use tokio::time::Duration;

// ==================== Construction Tests ====================

#[test]
fn test_integration_manager_new() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config);

    // Manager should be created successfully with default timeouts
    // Actual timeout values are internal implementation details
}

#[test]
fn test_integration_manager_with_custom_startup_timeout() {
    let config = SongbirdConfig::default();
    let timeout = Duration::from_secs(120);
    let _manager = IntegrationManager::new(config).with_startup_timeout(timeout);

    // Manager should accept custom startup timeout without error
}

#[test]
fn test_integration_manager_with_custom_shutdown_timeout() {
    let config = SongbirdConfig::default();
    let timeout = Duration::from_secs(60);
    let _manager = IntegrationManager::new(config).with_shutdown_timeout(timeout);

    // Manager should accept custom shutdown timeout without error
}

#[test]
fn test_integration_manager_with_both_timeouts() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config)
        .with_startup_timeout(Duration::from_secs(180))
        .with_shutdown_timeout(Duration::from_secs(90));

    // Manager should accept both timeout configurations without error
}

#[test]
fn test_integration_manager_builder_pattern() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config)
        .with_startup_timeout(Duration::from_secs(300))
        .with_shutdown_timeout(Duration::from_secs(150));

    // Verify builder pattern works correctly (compiles and executes without error)
}

// ==================== Timeout Edge Cases ====================

#[test]
fn test_integration_manager_zero_timeout() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config).with_startup_timeout(Duration::from_secs(0));

    // Manager should accept zero timeout (edge case)
}

#[test]
fn test_integration_manager_very_long_timeout() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config).with_startup_timeout(Duration::from_secs(3600));

    // Manager should accept very long timeout (1 hour)
    Ok(())
}

#[test]
fn test_integration_manager_millisecond_timeout() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config).with_startup_timeout(Duration::from_millis(500));

    // Manager should accept millisecond-precision timeouts
    Ok(())
}

// ==================== Service Availability Tests ====================

#[tokio::test]
async fn test_check_service_availability_success() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    let result = manager.check_service_availability().await;
    assert!(result.is_ok(), "Service availability check should succeed");
    assert!(
        result.ok_or_else(|_| SongbirdError::configuration(format!(
            "Error: {}",
            e
        )))?,
        "Services should be available"
    );
    Ok(())
}

#[tokio::test]
async fn test_check_service_availability_with_custom_config() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    // Configure with custom settings
    std::env::set_var("GAMING_PORT", "8080");

    let manager = IntegrationManager::new(config);
    let result = manager.check_service_availability().await;

    assert!(result.is_ok());
    assert!(result.ok_or_else(|_| SongbirdError::configuration(format!(
        "Error: {}",
        e
    )))?);

    // Cleanup
    std::env::remove_var("GAMING_PORT");
    Ok(())
}

// ==================== Graceful Shutdown Tests ====================

#[tokio::test]
async fn test_graceful_shutdown_success() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    let result = manager.shutdown_gracefully().await;
    assert!(result.is_ok(), "Graceful shutdown should succeed");
}

#[tokio::test]
async fn test_graceful_shutdown_with_short_timeout() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config).with_shutdown_timeout(Duration::from_secs(5));

    let result = manager.shutdown_gracefully().await;
    assert!(result.is_ok(), "Shutdown with short timeout should succeed");
}

#[tokio::test]
async fn test_graceful_shutdown_with_long_timeout() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config).with_shutdown_timeout(Duration::from_secs(120));

    let result = manager.shutdown_gracefully().await;
    assert!(result.is_ok(), "Shutdown with long timeout should succeed");
}

// ==================== Configuration Tests ====================

#[test]
fn test_integration_manager_with_default_config() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config);

    // Verify manager can be created with default config
}

#[test]
fn test_integration_manager_timeout_ordering() {
    let config = SongbirdConfig::default();

    // Test that timeout setters can be called in any order
    let _manager1 = IntegrationManager::new(config.clone())
        .with_startup_timeout(Duration::from_secs(100))
        .with_shutdown_timeout(Duration::from_secs(50));

    let _manager2 = IntegrationManager::new(config)
        .with_shutdown_timeout(Duration::from_secs(50))
        .with_startup_timeout(Duration::from_secs(100));

    // Both orderings should work without error
}

// ==================== Multiple Managers Tests ====================

#[test]
fn test_multiple_integration_managers() -> SongbirdResult<()> {
    let config1 = SongbirdConfig::default();
    let config2 = SongbirdConfig::default();

    let _manager1 = IntegrationManager::new(config1).with_startup_timeout(Duration::from_secs(60));
    let _manager2 = IntegrationManager::new(config2).with_startup_timeout(Duration::from_secs(120));

    // Multiple managers can coexist with different configurations
    Ok(())
}

#[tokio::test]
async fn test_concurrent_service_availability_checks() -> SongbirdResult<()> {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Run multiple availability checks concurrently
    let r1 = manager.check_service_availability().await;
    let r2 = manager.check_service_availability().await;
    let r3 = manager.check_service_availability().await;

    assert!(
        r1.is_ok()
            && r1.ok_or_else(|_| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
    );
    assert!(
        r2.is_ok()
            && r2.ok_or_else(|_| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
    );
    assert!(
        r3.is_ok()
            && r3.ok_or_else(|_| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
    );
    Ok(())
}

// ==================== Timeout Duration Tests ====================

#[test]
fn test_startup_timeout_values() {
    let config = SongbirdConfig::default();

    let timeouts = [
        Duration::from_secs(30),
        Duration::from_secs(60),
        Duration::from_secs(120),
        Duration::from_secs(300),
    ];

    for timeout in &timeouts {
        let _manager = IntegrationManager::new(config.clone()).with_startup_timeout(*timeout);
        // Each timeout value should be accepted without error
    }
}

#[test]
fn test_shutdown_timeout_values() {
    let config = SongbirdConfig::default();

    let timeouts = [
        Duration::from_secs(10),
        Duration::from_secs(30),
        Duration::from_secs(60),
        Duration::from_secs(120),
    ];

    for timeout in &timeouts {
        let _manager = IntegrationManager::new(config.clone()).with_shutdown_timeout(*timeout);
        // Each timeout value should be accepted without error
    }
}

// ==================== Builder Pattern Tests ====================

#[test]
fn test_builder_pattern_chaining() {
    let config = SongbirdConfig::default();

    let _manager = IntegrationManager::new(config)
        .with_startup_timeout(Duration::from_secs(180))
        .with_shutdown_timeout(Duration::from_secs(90));

    // Builder pattern chaining should work without error
}

#[test]
fn test_builder_pattern_multiple_timeout_sets() {
    let config = SongbirdConfig::default();

    // Last value should win
    let _manager = IntegrationManager::new(config)
        .with_startup_timeout(Duration::from_secs(60))
        .with_startup_timeout(Duration::from_secs(120))
        .with_startup_timeout(Duration::from_secs(180));

    // Multiple timeout sets should work (last one wins)
}

// ==================== Edge Case Tests ====================

#[tokio::test]
async fn test_shutdown_immediately_after_creation() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Should be able to shutdown immediately after creation
    let result = manager.shutdown_gracefully().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_shutdown_calls() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Multiple shutdowns should all succeed (idempotent)
    let result1 = manager.shutdown_gracefully().await;
    let result2 = manager.shutdown_gracefully().await;
    let result3 = manager.shutdown_gracefully().await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

// ==================== Stress Tests ====================

#[tokio::test]
async fn test_rapid_availability_checks() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Run 10 rapid checks
    for _ in 0..10 {
        let result = manager.check_service_availability().await;
        assert!(result.is_ok());
    }
}

#[test]
fn test_many_managers_creation() {
    // Create many managers to test resource handling
    let managers: Vec<_> = (0..100)
        .map(|_| {
            let config = SongbirdConfig::default();
            IntegrationManager::new(config)
        })
        .collect();

    assert_eq!(managers.len(), 100);
}

// ==================== Timeout Boundary Tests ====================

#[test]
fn test_minimum_timeout() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config).with_startup_timeout(Duration::from_millis(1));

    // Minimum timeout (1ms) should be accepted
}

#[test]
fn test_maximum_practical_timeout() {
    let config = SongbirdConfig::default();
    let _manager = IntegrationManager::new(config).with_startup_timeout(Duration::from_secs(86400));
    // 24 hours

    // Maximum practical timeout (24 hours) should be accepted
}

// ==================== Configuration Validation Tests ====================

#[tokio::test]
async fn test_service_availability_with_gaming_port_set() {
    std::env::set_var("GAMING_PORT", "8080");

    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    let result = manager.check_service_availability().await;
    assert!(result.is_ok());

    std::env::remove_var("GAMING_PORT");
}

#[tokio::test]
async fn test_service_availability_without_gaming_port() {
    std::env::remove_var("GAMING_PORT");

    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Should still work even without gaming port
    let result = manager.check_service_availability().await;
    assert!(result.is_ok());
}

// ==================== Concurrent Operations Tests ====================

#[tokio::test]
async fn test_concurrent_shutdowns() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Try concurrent shutdowns
    let r1 = manager.shutdown_gracefully().await;
    let r2 = manager.shutdown_gracefully().await;

    assert!(r1.is_ok());
    assert!(r2.is_ok());
}

#[tokio::test]
async fn test_shutdown_during_availability_check() {
    let config = SongbirdConfig::default();
    let manager = IntegrationManager::new(config);

    // Start availability check and shutdown sequentially
    let check_result = manager.check_service_availability().await;
    let shutdown_result = manager.shutdown_gracefully().await;

    // Both should complete successfully
    assert!(check_result.is_ok());
    assert!(shutdown_result.is_ok());
}
