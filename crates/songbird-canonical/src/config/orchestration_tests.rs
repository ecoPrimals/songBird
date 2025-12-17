//! Tests for Orchestration Configuration
//!
//! Comprehensive test coverage for orchestration configuration structures.

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// OrchestrationConfig Tests
// ============================================================================

#[test]
fn test_orchestration_config_default() -> SongbirdResult<()> {
    let config = OrchestrationConfig::default();

    assert!(config.discovery.enabled);
    assert!(config.health.enabled);
    // Verify config can be created with defaults
    Ok(())
}

#[test]
fn test_orchestration_config_serialization() -> SongbirdResult<()> {
    let config = OrchestrationConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: OrchestrationConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(config.discovery.enabled, deserialized.discovery.enabled);
    Ok(())
}

#[test]
fn test_orchestration_config_clone() {
    let config = OrchestrationConfig::default();
    let cloned = config.clone();

    assert_eq!(config.discovery.enabled, cloned.discovery.enabled);
}

// ============================================================================
// ServiceDiscoveryConfig Tests
// ============================================================================

#[test]
fn test_service_discovery_config_default() {
    let config = ServiceDiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.interval_seconds, 30);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.max_services, 100);
}

#[test]
fn test_service_discovery_config_custom() {
    let mut config = ServiceDiscoveryConfig::default();
    config.interval_seconds = 60;
    config.timeout_ms = 10000;
    config.max_services = 200;

    assert_eq!(config.interval_seconds, 60);
    assert_eq!(config.timeout_ms, 10000);
    assert_eq!(config.max_services, 200);
}

#[test]
fn test_service_discovery_config_disabled() {
    let mut config = ServiceDiscoveryConfig::default();
    config.enabled = false;

    assert!(!config.enabled);
}

// ============================================================================
// LoadBalancingConfig Tests
// ============================================================================

#[test]
fn test_load_balancing_config_default() {
    let config = LoadBalancingConfig::default();

    assert_eq!(config.health_check_interval_seconds, 10);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.request_timeout_ms, 30000);
}

#[test]
fn test_load_balancing_config_strategies() {
    let round_robin = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        ..Default::default()
    };
    let least_conn = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        ..Default::default()
    };
    let health_based = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::HealthBased,
        ..Default::default()
    };

    // Verify different strategies can be configured
    let _ = round_robin;
    let _ = least_conn;
    let _ = health_based;
}

#[test]
fn test_load_balancing_config_custom_retries() {
    let mut config = LoadBalancingConfig::default();
    config.max_retries = 5;

    assert_eq!(config.max_retries, 5);
}

// ============================================================================
// LoadBalancingStrategy Tests
// ============================================================================

#[test]
fn test_load_balancing_strategy_variants() -> SongbirdResult<()> {
    let round_robin = LoadBalancingStrategy::RoundRobin;
    let least_conn = LoadBalancingStrategy::LeastConnections;
    let health = LoadBalancingStrategy::HealthBased;

    // Verify all variants exist
    let _ = round_robin;
    let _ = least_conn;
    let _ = health;
    Ok(())
}

#[test]
fn test_load_balancing_strategy_serialization() -> SongbirdResult<()> {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::HealthBased,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
        let deserialized: LoadBalancingStrategy = serde_json::from_str(&json)
            .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;
        let _ = deserialized;
    }
    Ok(())
}

// ============================================================================
// HealthConfig Tests
// ============================================================================

#[test]
fn test_health_config_default() {
    let config = HealthConfig::default();

    assert!(config.enabled);
    assert_eq!(config.check_interval_seconds, 30);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.success_threshold, 2);
}

#[test]
fn test_health_config_custom_thresholds() {
    let mut config = HealthConfig::default();
    config.failure_threshold = 5;
    config.success_threshold = 3;

    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
}

#[test]
fn test_health_config_custom_intervals() {
    let mut config = HealthConfig::default();
    config.check_interval_seconds = 60;
    config.timeout_ms = 10000;

    assert_eq!(config.check_interval_seconds, 60);
    assert_eq!(config.timeout_ms, 10000);
}

#[test]
fn test_health_config_disabled() {
    let mut config = HealthConfig::default();
    config.enabled = false;

    assert!(!config.enabled);
}

// ============================================================================
// ScalingConfig Tests
// ============================================================================

#[test]
fn test_scaling_config_default() {
    let config = ScalingConfig::default();

    assert!(!config.enabled); // Disabled by default
    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
    assert_eq!(config.target_cpu_percent, 70.0);
    assert_eq!(config.target_memory_percent, 80.0);
    assert_eq!(config.check_interval_seconds, 60);
}

#[test]
fn test_scaling_config_custom_instances() {
    let mut config = ScalingConfig::default();
    config.min_instances = 2;
    config.max_instances = 20;

    assert_eq!(config.min_instances, 2);
    assert_eq!(config.max_instances, 20);
}

#[test]
fn test_scaling_config_custom_thresholds() {
    let mut config = ScalingConfig::default();
    config.target_cpu_percent = 90.0;
    config.target_memory_percent = 85.0;

    assert_eq!(config.target_cpu_percent, 90.0);
    assert_eq!(config.target_memory_percent, 85.0);
}

#[test]
fn test_scaling_config_enabled() {
    let mut config = ScalingConfig::default();
    config.enabled = true;

    assert!(config.enabled);
}

#[test]
fn test_scaling_config_check_interval() {
    let mut config = ScalingConfig::default();
    config.check_interval_seconds = 120;

    assert_eq!(config.check_interval_seconds, 120);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_orchestration_config_production_preset() {
    let config = OrchestrationConfig {
        discovery: ServiceDiscoveryConfig {
            enabled: true,
            interval_seconds: 60,
            timeout_ms: 10000,
            max_services: 500,
        },
        load_balancing: LoadBalancingConfig {
            strategy: LoadBalancingStrategy::HealthBased,
            health_check_interval_seconds: 15,
            max_retries: 5,
            request_timeout_ms: 60000,
        },
        health: HealthConfig {
            enabled: true,
            check_interval_seconds: 15,
            timeout_ms: 3000,
            failure_threshold: 5,
            success_threshold: 3,
        },
        scaling: ScalingConfig {
            enabled: true,
            min_instances: 3,
            max_instances: 50,
            target_cpu_percent: 85.0,
            target_memory_percent: 75.0,
            check_interval_seconds: 30,
        },
    };

    assert!(config.discovery.enabled);
    assert_eq!(config.discovery.max_services, 500);
    assert_eq!(config.scaling.min_instances, 3);
    assert_eq!(config.health.failure_threshold, 5);
}

#[test]
fn test_orchestration_config_minimal() {
    let config = OrchestrationConfig {
        discovery: ServiceDiscoveryConfig {
            enabled: false,
            interval_seconds: 300,
            timeout_ms: 30000,
            max_services: 10,
        },
        load_balancing: LoadBalancingConfig {
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_interval_seconds: 60,
            max_retries: 1,
            request_timeout_ms: 10000,
        },
        health: HealthConfig {
            enabled: false,
            check_interval_seconds: 300,
            timeout_ms: 30000,
            failure_threshold: 10,
            success_threshold: 1,
        },
        scaling: ScalingConfig {
            enabled: false,
            min_instances: 1,
            max_instances: 1,
            target_cpu_percent: 100.0,
            target_memory_percent: 100.0,
            check_interval_seconds: 600,
        },
    };

    assert!(!config.discovery.enabled);
    assert!(!config.health.enabled);
    assert!(!config.scaling.enabled);
    assert_eq!(config.scaling.max_instances, 1);
}
