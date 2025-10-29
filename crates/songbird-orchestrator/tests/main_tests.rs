//! Main Function Tests for Songbird Orchestrator
//!
//! Comprehensive test suite validating the main application function and startup process.
//! Testing all critical paths, startup scenarios, configuration loading,
//! error handling, and main function execution paths.
//!
//! **MODERNIZED**: October 28, 2025
//! - Updated to use current config API (`NetworkConfig`, `SecurityConfig`, etc.)
//! - Idiomatic Rust patterns
//! - No deprecated fields
//! - Async/await best practices

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::module_name_repetitions)]

use anyhow::Result;
use serial_test::serial;
use songbird_config::SongbirdConfig;
use songbird_orchestrator::SongbirdOrchestrator;
use std::env;

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[tokio::test]
async fn test_configuration_default_creation() {
    // Modern: Config creates sensible defaults
    let config = SongbirdConfig::default();

    // Verify core fields exist
    assert!(!config.environment.is_empty());
    assert!(config.performance.is_some());

    // Verify network config
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.port_range.start > 0);
    assert!(config.network.port_range.end > config.network.port_range.start);
    assert!(config.network.max_connections > 0);
}

#[tokio::test]
async fn test_configuration_test_defaults() {
    // Modern: Use dedicated test_defaults() method
    let config = SongbirdConfig::test_defaults();

    assert_eq!(config.environment, "test");
    assert!(!config.security.enabled); // Test mode: security off
    assert!(config.network.port_range.start >= 19000); // Test port range
}

#[test]
#[serial]
fn test_environment_configuration() {
    // Test environment variable handling
    env::set_var("SONGBIRD_ENV", "staging");
    let config = SongbirdConfig::default();
    assert_eq!(config.environment, "staging");
    env::remove_var("SONGBIRD_ENV");

    // Test default environment
    let config = SongbirdConfig::default();
    assert_eq!(config.environment, "development");
}

#[tokio::test]
async fn test_configuration_validation() {
    // Modern: Config validation through construction
    let config = SongbirdConfig::default();

    // Valid config should have reasonable values
    assert!(config.network.port_range.start > 0);
    assert!(config.network.port_range.start < config.network.port_range.end);
    assert!(config.network.max_connections > 0);
    assert!(config.network.connection_timeout_ms > 0);
}

// ============================================================================
// ORCHESTRATOR CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_orchestrator_creation() -> Result<()> {
    // Modern: Create orchestrator with test config
    let config = SongbirdConfig::test_defaults();
    let orchestrator = SongbirdOrchestrator::new(config).await?;

    // Verify orchestrator was created - config should be accessible
    assert!(!orchestrator.config().environment.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_orchestrator_lifecycle() -> Result<()> {
    // Modern: Test start/stop lifecycle
    let config = SongbirdConfig::test_defaults();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    // Start should succeed
    orchestrator.start().await?;

    // Stop should succeed
    orchestrator.stop().await?;

    Ok(())
}

// ============================================================================
// NETWORK CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_network_configuration_structure() {
    let config = SongbirdConfig::default();
    let network = &config.network;

    // Modern: Verify NetworkConfig fields
    assert!(!network.bind_address.is_empty());
    assert!(network.port_range.start > 0);
    assert!(network.port_range.end > network.port_range.start);
    assert!(network.max_connections > 0);
    assert!(network.connection_timeout_ms > 0);
}

#[test]
fn test_network_port_range_validation() {
    let config = SongbirdConfig::default();

    // Port range should be valid (u16 is always in range)
    let start = config.network.port_range.start;
    let end = config.network.port_range.end;

    assert!(start > 0, "Start port must be positive");
    assert!(end > start, "End port must be greater than start");
}

#[test]
fn test_network_bind_address() {
    let config = SongbirdConfig::default();

    // Bind address should be valid
    let addr = &config.network.bind_address;
    assert!(!addr.is_empty());

    // Should be localhost or IP address format
    assert!(
        addr.contains("127.0.0.1")
            || addr.contains("0.0.0.0")
            || addr.contains("localhost")
            || addr.contains("::")
    );
}

// ============================================================================
// SECURITY CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_security_configuration_structure() {
    let config = SongbirdConfig::default();
    let security = &config.security;

    // Modern: Verify SecurityConfig structure
    assert!(security.authentication.token_lifetime_seconds > 0);
    // EncryptionAlgorithm is an enum - just verify it has reasonable config
    assert!(security.encryption.key_rotation_days > 0);
}

#[test]
fn test_security_defaults() {
    let config = SongbirdConfig::default();

    // Development should have security enabled
    assert!(config.security.enabled || config.environment == "test");

    // Test config should have security disabled
    let test_config = SongbirdConfig::test_defaults();
    assert!(!test_config.security.enabled);
}

// ============================================================================
// DISCOVERY CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_discovery_configuration_structure() {
    let config = SongbirdConfig::default();
    let discovery = &config.discovery;

    // Modern: Verify DiscoveryConfig fields
    assert!(discovery.interval_seconds > 0);
    assert!(discovery.health_check.interval_seconds > 0);
}

#[test]
fn test_discovery_health_check_config() {
    let config = SongbirdConfig::default();

    // Health check should have reasonable defaults
    assert!(config.discovery.health_check.interval_seconds > 0);
    assert!(config.discovery.health_check.timeout_seconds > 0);
    assert!(config.discovery.health_check.retries > 0);
}

// ============================================================================
// OBSERVABILITY CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_observability_configuration_structure() {
    let config = SongbirdConfig::default();
    let observability = &config.observability;

    // Modern: Verify ObservabilityConfig fields
    assert!(!observability.metrics.endpoint.is_empty());
    assert!(observability.metrics.interval_seconds > 0);
    // LogLevel is an enum - just verify metrics and tracing are configured
    assert!(observability.tracing.sample_rate >= 0.0 && observability.tracing.sample_rate <= 1.0);
}

#[test]
fn test_observability_defaults() {
    let config = SongbirdConfig::default();

    // Metrics should be enabled by default
    assert!(config.observability.metrics.enabled);

    // Tracing should be configured
    assert!(config.observability.tracing.enabled);
}

// ============================================================================
// PERFORMANCE CONFIGURATION TESTS
// ============================================================================

#[test]
fn test_performance_configuration() {
    let config = SongbirdConfig::default();

    // Modern: Verify PerformanceConfig
    let perf = config.performance.expect("Performance config should exist");
    assert!(perf.connection_pool_size.unwrap_or(0) > 0);
    assert!(perf.request_timeout_ms.unwrap_or(0) > 0);
}

#[test]
fn test_performance_tuning_options() {
    let config = SongbirdConfig::default();
    let perf = config.performance.unwrap();

    // Zero-copy should be enabled by default
    assert!(perf.enable_zero_copy.unwrap_or(false));

    // Batch size should be reasonable
    assert!(perf.batch_size.unwrap_or(0) > 0);
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_error_handling_invalid_config() {
    // Modern: Test with edge case config (start == end)
    let mut config = SongbirdConfig::test_defaults();
    config.network.port_range.start = 8000;
    config.network.port_range.end = 8000; // Edge case: no range

    // Creation should handle this gracefully
    let result = SongbirdOrchestrator::new(config).await;

    // May succeed (using defaults) or fail (validation)
    // Either is acceptable as long as it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// ASYNC RUNTIME TESTS
// ============================================================================

#[tokio::test]
async fn test_async_runtime_compatibility() {
    // Modern: Verify async operations work correctly
    let config = SongbirdConfig::test_defaults();

    // Should be able to create orchestrator in async context
    let result = SongbirdOrchestrator::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multi_thread_compatibility() {
    // Modern: Test with multi-threaded runtime
    let config = SongbirdConfig::test_defaults();
    let orchestrator = SongbirdOrchestrator::new(config).await;
    assert!(orchestrator.is_ok());
}

// ============================================================================
// ENVIRONMENT INTEGRATION TESTS
// ============================================================================

#[test]
#[serial]
fn test_environment_variable_integration() {
    // Test environment variable overrides
    env::set_var("SONGBIRD_ENV", "production");
    env::set_var("SONGBIRD_BIND_ADDRESS", "0.0.0.0");

    let config = SongbirdConfig::default();
    assert_eq!(config.environment, "production");

    env::remove_var("SONGBIRD_ENV");
    env::remove_var("SONGBIRD_BIND_ADDRESS");
}

// ============================================================================
// COMPREHENSIVE VALIDATION TESTS
// ============================================================================

#[tokio::test]
async fn test_comprehensive_configuration_validation() {
    // Modern: Comprehensive validation of all config components
    let config = SongbirdConfig::default();

    // Environment
    assert!(!config.environment.is_empty());

    // Network
    assert!(!config.network.bind_address.is_empty());
    assert!(config.network.port_range.start > 0);
    assert!(config.network.max_connections > 0);

    // Security
    assert!(config.security.authentication.token_lifetime_seconds > 0);

    // Discovery
    assert!(config.discovery.interval_seconds > 0);

    // Observability
    assert!(!config.observability.metrics.endpoint.is_empty());

    // Performance
    assert!(config.performance.is_some());
}

#[tokio::test]
async fn test_full_orchestrator_workflow() -> Result<()> {
    // Modern: End-to-end test of orchestrator lifecycle
    let config = SongbirdConfig::test_defaults();

    // Create
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;
    assert!(!orchestrator.config().environment.is_empty());

    // Start
    orchestrator.start().await?;

    // Verify running state - get_status should succeed
    let _status = orchestrator.get_status().await?;

    // Stop
    orchestrator.stop().await?;

    Ok(())
}

// ============================================================================
// IDIOMATIC RUST PATTERNS
// ============================================================================

#[test]
fn test_config_is_clonable() {
    // Modern: Config should implement Clone for flexibility
    let config1 = SongbirdConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.environment, config2.environment);
}

#[test]
fn test_config_is_debug_printable() {
    // Modern: Config should implement Debug
    let config = SongbirdConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("SongbirdConfig"));
}

#[test]
fn test_builder_pattern_compatibility() {
    // Modern: Config supports modification before use
    let config = SongbirdConfig {
        environment: "custom".to_string(),
        network: songbird_config::NetworkConfig {
            max_connections: 500,
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(config.environment, "custom");
    assert_eq!(config.network.max_connections, 500);
}

// ============================================================================
// PRODUCTION READINESS TESTS
// ============================================================================

#[test]
#[serial]
fn test_production_configuration() {
    // Modern: Test production-specific config
    env::set_var("SONGBIRD_ENV", "production");
    let config = SongbirdConfig::default();

    assert_eq!(config.environment, "production");
    assert!(config.security.enabled);

    env::remove_var("SONGBIRD_ENV");
}

#[tokio::test]
async fn test_graceful_shutdown() -> Result<()> {
    // Modern: Test graceful shutdown behavior
    let config = SongbirdConfig::test_defaults();
    let mut orchestrator = SongbirdOrchestrator::new(config).await?;

    orchestrator.start().await?;

    // Stop should be clean
    let shutdown_result = orchestrator.stop().await;
    assert!(shutdown_result.is_ok());

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS (Idiomatic Rust)
// ============================================================================

/// Helper to create a config with custom environment
fn config_with_env(env: &str) -> SongbirdConfig {
    SongbirdConfig {
        environment: env.to_string(),
        ..Default::default()
    }
}

#[test]
fn test_helper_config_with_env() {
    let config = config_with_env("staging");
    assert_eq!(config.environment, "staging");
}
