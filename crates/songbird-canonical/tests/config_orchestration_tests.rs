// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for Orchestration Configuration
//!
//! Comprehensive tests for service orchestration configuration structures

use songbird_canonical::config::orchestration::{
    HealthConfig, LoadBalancingConfig, LoadBalancingStrategy, OrchestrationConfig, ScalingConfig,
    ServiceDiscoveryConfig,
};
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_orchestration_config_default() {
    let config = OrchestrationConfig::default();

    assert!(config.discovery.enabled);
    assert!(config.health.enabled);
    // Scaling is disabled by default
    assert!(!config.scaling.enabled);
}

#[test]
fn test_service_discovery_defaults() {
    let config = ServiceDiscoveryConfig::default();

    assert!(config.enabled);
    assert_eq!(config.interval_seconds, 30);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.max_services, 100);
}

#[test]
fn test_service_discovery_custom() {
    let config = ServiceDiscoveryConfig {
        enabled: true,
        interval_seconds: 60,
        timeout_ms: 10000,
        max_services: 500,
    };

    assert!(config.enabled);
    assert_eq!(config.interval_seconds, 60);
    assert_eq!(config.timeout_ms, 10000);
    assert_eq!(config.max_services, 500);
}

#[test]
fn test_load_balancing_defaults() {
    let config = LoadBalancingConfig::default();

    assert!(matches!(config.strategy, LoadBalancingStrategy::RoundRobin));
    assert_eq!(config.health_check_interval_seconds, 10);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.request_timeout_ms, 30000);
}

#[test]
fn test_load_balancing_strategies() {
    let round_robin = LoadBalancingStrategy::RoundRobin;
    let least_conn = LoadBalancingStrategy::LeastConnections;
    let health_based = LoadBalancingStrategy::HealthBased;

    assert!(matches!(round_robin, LoadBalancingStrategy::RoundRobin));
    assert!(matches!(least_conn, LoadBalancingStrategy::LeastConnections));
    assert!(matches!(health_based, LoadBalancingStrategy::HealthBased));
}

#[test]
fn test_load_balancing_custom() {
    let config = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        health_check_interval_seconds: 5,
        max_retries: 5,
        request_timeout_ms: 60000,
    };

    assert!(matches!(config.strategy, LoadBalancingStrategy::LeastConnections));
    assert_eq!(config.health_check_interval_seconds, 5);
    assert_eq!(config.max_retries, 5);
}

#[test]
fn test_health_config_defaults() {
    let config = HealthConfig::default();

    assert!(config.enabled);
    assert_eq!(config.check_interval_seconds, 30);
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.success_threshold, 2);
}

#[test]
fn test_health_config_aggressive() {
    let config = HealthConfig {
        enabled: true,
        check_interval_seconds: 5,
        timeout_ms: 1000,
        failure_threshold: 1,
        success_threshold: 1,
    };

    assert!(config.enabled);
    assert_eq!(config.check_interval_seconds, 5);
    assert_eq!(config.failure_threshold, 1);
}

#[test]
fn test_health_config_tolerant() {
    let config = HealthConfig {
        enabled: true,
        check_interval_seconds: 120,
        timeout_ms: 30000,
        failure_threshold: 10,
        success_threshold: 5,
    };

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 5);
    assert!(config.failure_threshold > config.success_threshold);
}

#[test]
fn test_scaling_config_defaults() {
    let config = ScalingConfig::default();

    assert!(!config.enabled); // Disabled by default
    assert_eq!(config.min_instances, 1);
    assert_eq!(config.max_instances, 10);
    assert!((config.target_cpu_percent - 70.0).abs() < 0.001);
    assert!((config.target_memory_percent - 80.0).abs() < 0.001);
    assert_eq!(config.check_interval_seconds, 60);
}

#[test]
fn test_scaling_config_enabled() {
    let config = ScalingConfig {
        enabled: true,
        min_instances: 2,
        max_instances: 50,
        target_cpu_percent: 80.0,
        target_memory_percent: 85.0,
        check_interval_seconds: 30,
    };

    assert!(config.enabled);
    assert_eq!(config.min_instances, 2);
    assert_eq!(config.max_instances, 50);
    assert!(config.min_instances < config.max_instances);
}

#[test]
fn test_scaling_config_validation() {
    let config = ScalingConfig::default();

    assert!(config.min_instances <= config.max_instances);
    assert!(config.target_cpu_percent > 0.0 && config.target_cpu_percent <= 100.0);
    assert!(config.target_memory_percent > 0.0 && config.target_memory_percent <= 100.0);
}

#[test]
fn test_orchestration_config_serialization() -> SongbirdResult<()> {
    let config = OrchestrationConfig::default();

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
    assert!(json.contains("discovery"));
    assert!(json.contains("load_balancing"));
    assert!(json.contains("health"));
    assert!(json.contains("scaling"));

    let deserialized: OrchestrationConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Should deserialize: {}", e),
            debug_info: None,
        })?;
    assert_eq!(config.discovery.enabled, deserialized.discovery.enabled);
    Ok(())
}

#[test]
fn test_orchestration_config_clone() {
    let config = OrchestrationConfig::default();
    let cloned = config.clone();

    assert_eq!(config.discovery.enabled, cloned.discovery.enabled);
    assert_eq!(config.health.enabled, cloned.health.enabled);
    assert_eq!(config.scaling.enabled, cloned.scaling.enabled);
}

#[test]
fn test_orchestration_config_debug() {
    let config = OrchestrationConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("OrchestrationConfig"));
    assert!(debug_str.contains("discovery"));
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
            .map_err(|e| SongbirdError::configuration(format!("Should serialize: {}", e)))?;
        let _deserialized: LoadBalancingStrategy =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Should deserialize: {}", e),
                debug_info: None,
            })?;
    }
    Ok(())
}

#[test]
fn test_service_discovery_disabled() {
    let config = ServiceDiscoveryConfig {
        enabled: false,
        interval_seconds: 0,
        timeout_ms: 1000,
        max_services: 0,
    };

    assert!(!config.enabled);
    assert_eq!(config.max_services, 0);
}

#[test]
fn test_health_config_disabled() {
    let config = HealthConfig {
        enabled: false,
        check_interval_seconds: 0,
        timeout_ms: 0,
        failure_threshold: 0,
        success_threshold: 0,
    };

    assert!(!config.enabled);
}

#[test]
fn test_scaling_extreme_values() {
    let minimal = ScalingConfig {
        enabled: true,
        min_instances: 1,
        max_instances: 1,
        target_cpu_percent: 50.0,
        target_memory_percent: 50.0,
        check_interval_seconds: 10,
    };

    let maximal = ScalingConfig {
        enabled: true,
        min_instances: 10,
        max_instances: 1000,
        target_cpu_percent: 90.0,
        target_memory_percent: 95.0,
        check_interval_seconds: 300,
    };

    assert_eq!(minimal.min_instances, minimal.max_instances);
    assert!(maximal.min_instances < maximal.max_instances);
}
