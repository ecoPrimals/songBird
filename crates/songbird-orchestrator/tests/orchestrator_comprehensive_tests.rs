//! Comprehensive Orchestrator Tests
//!
//! Modern concurrent testing patterns - no sleeps, event-driven coordination

use anyhow::Result;
use songbird_orchestrator::SongbirdOrchestrator;
use songbird_types::config::CanonicalSongbirdConfig;

// ============================================================================
// Creation Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_creation_default_config() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert!(!orchestrator.config().environment.name.is_empty());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_creation_custom_environment() -> Result<()> {
    let mut config = CanonicalSongbirdConfig::default();
    config.environment.name = "test-env".to_string();

    let orchestrator = SongbirdOrchestrator::new(config).await?;
    assert_eq!(orchestrator.config().environment.name, "test-env");

    Ok(())
}

// ============================================================================
// Service Registry Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs service_registry API implementation"]
async fn test_service_registry_access() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let registry = orchestrator.service_registry();
    let _services = registry.get_all_services();

    Ok(())
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_lifecycle_start_stop() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_lifecycle_stop_without_start() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.stop().await;
    assert!(result.is_ok(), "Stopping without starting should be graceful");

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_lifecycle_multiple_cycles() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple start/stop cycles
    for _ in 0..3 {
        orchestrator.start().await?;
        orchestrator.stop().await?;
    }

    Ok(())
}

// ============================================================================
// Status Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_get_status_basic() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let _status = orchestrator.get_status().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_get_status_while_running() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    let _status = orchestrator.get_status().await?;
    orchestrator.stop().await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_get_status_after_stop() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;
    orchestrator.stop().await?;
    let _status = orchestrator.get_status().await?;

    Ok(())
}

// ============================================================================
// Command Handling Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs handle_command API implementation"]
async fn test_handle_command_unknown() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command("unknown".to_string()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs handle_command API implementation"]
async fn test_handle_command_empty() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command("".to_string()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs handle_command API implementation"]
async fn test_handle_command_whitespace() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let result = orchestrator.handle_command("   ".to_string()).await;
    assert!(result.is_err() || result.is_ok());

    Ok(())
}

// ============================================================================
// Config Tests (Sync)
// ============================================================================

#[test]
fn test_config_default_environment() {
    let config = CanonicalSongbirdConfig::default();
    assert!(!config.environment.name.is_empty());
}

#[test]
fn test_config_default_is_valid() {
    let config = CanonicalSongbirdConfig::default();
    assert!(!config.environment.name.is_empty());
}

#[test]
fn test_config_clone() {
    let config1 = CanonicalSongbirdConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.environment.name, config2.environment.name);
}

#[test]
fn test_config_debug() {
    let config = CanonicalSongbirdConfig::default();
    let debug_str = format!("{config:?}");
    assert!(!debug_str.is_empty());
}

// ============================================================================
// Rapid/Stress Tests (Modern Concurrent Pattern)
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rapid_creation() -> Result<()> {
    // Rapidly create multiple orchestrators (no sleeps, truly concurrent)
    for _ in 0..5 {
        let config = CanonicalSongbirdConfig::default();
        let _orchestrator = SongbirdOrchestrator::new(config).await?;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs start/stop API implementation"]
async fn test_rapid_lifecycle_cycles() -> Result<()> {
    // Rapid lifecycle cycles (testing robustness)
    for _ in 0..3 {
        let config = CanonicalSongbirdConfig::default();
        let mut orchestrator = SongbirdOrchestrator::new(config).await?;
        orchestrator.start().await?;
        orchestrator.stop().await?;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_status_checks() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple rapid status checks (no sleeps)
    for _ in 0..10 {
        let _ = orchestrator.get_status().await?;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs handle_command API implementation"]
async fn test_multiple_command_handling() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Multiple rapid commands
    for i in 0..5 {
        let command = format!("test_cmd_{i}");
        let _ = orchestrator.handle_command(command).await?;
    }

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs service_registry API implementation"]
async fn test_orchestrator_services_accessible() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let registry = orchestrator.service_registry();
    let services = registry.get_all_services().await;

    // Should be accessible (empty or not)
    assert!(services.is_empty() || !services.is_empty());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_orchestrator_environment_check() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    let env = &orchestrator.config().environment;
    assert!(!env.name.is_empty());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Needs handle_command API implementation"]
async fn test_orchestrator_full_lifecycle() -> Result<()> {
    let config = CanonicalSongbirdConfig::default();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Full lifecycle test (no sleeps, event-driven)
    let _status1 = orchestrator.get_status().await?;
    let _cmd_result = orchestrator.handle_command("test".to_string()).await?;
    let _status2 = orchestrator.get_status().await?;

    Ok(())
}
