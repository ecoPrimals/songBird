//! Canonical Universal Adapter Tests
//!
//! Comprehensive tests for the canonical adapter configuration and supporting types.
//! These tests focus on ensuring all config structs, enums, and metrics work correctly.

#![allow(clippy::similar_names)]
#![allow(clippy::uninlined_format_args)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use songbird_types::adapters::canonical::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;

// ============================================================================
// ADAPTER CONFIG TESTS
// ============================================================================

#[test]
fn test_canonical_adapter_config_default() {
    let config = CanonicalAdapterConfig::default();

    // All sub-configs should have reasonable defaults
    assert!(config.discovery.interval > Duration::from_secs(0));
    assert!(config.discovery.timeout > Duration::from_secs(0));
    assert!(config.load_balancing.health_weight >= 0.0);
    assert!(config.circuit_breaker.failure_threshold > 0);
    assert!(config.retry.max_attempts > 0);
    assert!(config.timeouts.request_timeout > Duration::from_secs(0));
    assert!(config.health_check.interval > Duration::from_secs(0));
    assert!(config.monitoring.collection_interval > Duration::from_secs(0));
}

#[test]
fn test_canonical_adapter_config_clone() -> SongbirdResult<()> {
    let config1 = CanonicalAdapterConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.discovery.interval, config2.discovery.interval);
    assert!(
        (config1.load_balancing.health_weight - config2.load_balancing.health_weight).abs() < 1e-6
    );
    assert_eq!(
        config1.circuit_breaker.failure_threshold,
        config2.circuit_breaker.failure_threshold
    );

    Ok(())
}

#[test]
fn test_canonical_adapter_config_debug() -> SongbirdResult<()> {
    let config = CanonicalAdapterConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("CanonicalAdapterConfig"));

    Ok(())
}

// ============================================================================
// DISCOVERY CONFIG TESTS
// ============================================================================

#[test]
fn test_discovery_config_default() -> SongbirdResult<()> {
    let config = CanonicalDiscoveryConfig::default();

    assert_eq!(config.interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.max_services_per_capability, 10);
    assert_eq!(config.service_ttl, Duration::from_secs(300));

    Ok(())
}

#[test]
fn test_discovery_config_custom() -> SongbirdResult<()> {
    let config = CanonicalDiscoveryConfig {
        interval: Duration::from_secs(60),
        timeout: Duration::from_secs(20),
        max_services_per_capability: 50,
        service_ttl: Duration::from_secs(600),
    };

    assert_eq!(config.interval, Duration::from_secs(60));
    assert_eq!(config.max_services_per_capability, 50);

    Ok(())
}

#[test]
fn test_discovery_config_serialization() -> SongbirdResult<()> {
    let config = CanonicalDiscoveryConfig::default();

    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;

    let deserialized: CanonicalDiscoveryConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {}", e)))?;

    assert_eq!(config.interval, deserialized.interval);
    assert_eq!(config.timeout, deserialized.timeout);

    Ok(())
}

// ============================================================================
// LOAD BALANCING CONFIG TESTS
// ============================================================================

#[test]
fn test_load_balancing_config_default() {
    let config = CanonicalLoadBalancingConfig::default();

    assert!((config.health_weight - 0.4).abs() < 1e-6);
    assert!((config.performance_weight - 0.4).abs() < 1e-6);
    assert!((config.availability_weight - 0.2).abs() < 1e-6);
}

#[test]
fn test_load_balancing_config_weights_sum() -> SongbirdResult<()> {
    let config = CanonicalLoadBalancingConfig::default();
    let sum = config.health_weight + config.performance_weight + config.availability_weight;

    // Weights should sum to approximately 1.0
    assert!((sum - 1.0).abs() < 0.01);

    Ok(())
}

#[test]
fn test_load_balancing_strategy_variants() -> SongbirdResult<()> {
    use CanonicalLoadBalancingStrategy::*;

    let strategies = vec![
        RoundRobin,
        WeightedRoundRobin,
        LeastConnections,
        LeastResponseTime,
        Random,
        ConsistentHash,
        HealthAware,
    ];

    for strategy in strategies {
        let debug = format!("{:?}", strategy);
        assert!(!debug.is_empty());
    }

    Ok(())
}

#[test]
fn test_load_balancing_strategy_equality() {
    assert_eq!(
        CanonicalLoadBalancingStrategy::RoundRobin,
        CanonicalLoadBalancingStrategy::RoundRobin
    );
    assert_ne!(CanonicalLoadBalancingStrategy::RoundRobin, CanonicalLoadBalancingStrategy::Random);
}

#[test]
fn test_load_balancing_strategy_default() {
    let config = CanonicalLoadBalancingConfig::default();
    assert_eq!(config.strategy, CanonicalLoadBalancingStrategy::HealthAware);
}

// ============================================================================
// CIRCUIT BREAKER CONFIG TESTS
// ============================================================================

#[test]
fn test_circuit_breaker_config_default() {
    let config = CanonicalCircuitBreakerConfig::default();

    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.reset_timeout, Duration::from_secs(30));
}

#[test]
fn test_circuit_breaker_config_custom() -> SongbirdResult<()> {
    let config = CanonicalCircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 5,
        timeout: Duration::from_secs(120),
        reset_timeout: Duration::from_secs(60),
    };

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 5);

    Ok(())
}

#[test]
fn test_circuit_state_variants() -> SongbirdResult<()> {
    use CanonicalCircuitState::*;

    let states = vec![Closed, Open, HalfOpen];

    for state in states {
        let debug = format!("{:?}", state);
        assert!(!debug.is_empty());
    }

    Ok(())
}

#[test]
fn test_circuit_state_equality() {
    assert_eq!(CanonicalCircuitState::Closed, CanonicalCircuitState::Closed);
    assert_ne!(CanonicalCircuitState::Closed, CanonicalCircuitState::Open);
}

// ============================================================================
// RETRY CONFIG TESTS
// ============================================================================

#[test]
fn test_retry_config_default() {
    let config = CanonicalRetryConfig::default();

    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    // Use epsilon comparison for floats to avoid clippy::float_cmp
    assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_retry_config_backoff_multiplier() {
    let config = CanonicalRetryConfig::default();
    assert!(config.backoff_multiplier >= 1.0);
}

#[test]
fn test_retry_config_delays_sensible() {
    let config = CanonicalRetryConfig::default();
    // Max delay should be greater than or equal to initial delay
    assert!(config.max_delay >= config.initial_delay);
}

#[test]
fn test_retry_config_has_max_attempts() {
    let config = CanonicalRetryConfig::default();
    assert!(config.max_attempts > 0, "Should have at least one retry attempt");
}

// ============================================================================
// TIMEOUT CONFIG TESTS
// ============================================================================

#[test]
fn test_timeout_config_default() {
    let config = CanonicalTimeoutConfig::default();

    assert_eq!(config.request_timeout, Duration::from_secs(30));
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
    assert_eq!(config.health_check_timeout, Duration::from_secs(5));
    assert_eq!(config.discovery_timeout, Duration::from_secs(10));
}

#[test]
fn test_timeout_config_all_positive() {
    let config = CanonicalTimeoutConfig::default();

    assert!(config.request_timeout > Duration::from_secs(0));
    assert!(config.connection_timeout > Duration::from_secs(0));
    assert!(config.health_check_timeout > Duration::from_secs(0));
    assert!(config.discovery_timeout > Duration::from_secs(0));
}

#[test]
fn test_timeout_config_clone() {
    let config1 = CanonicalTimeoutConfig::default();
    let config2 = config1.clone();

    assert_eq!(config1.request_timeout, config2.request_timeout);
    assert_eq!(config1.connection_timeout, config2.connection_timeout);
}

// ============================================================================
// HEALTH CHECK CONFIG TESTS
// ============================================================================

#[test]
fn test_health_check_config_default() {
    let config = CanonicalHealthCheckConfig::default();

    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.unhealthy_threshold, 3);
    assert_eq!(config.healthy_threshold, 2);
}

#[test]
fn test_health_check_config_thresholds_positive() {
    let config = CanonicalHealthCheckConfig::default();

    assert!(config.unhealthy_threshold > 0);
    assert!(config.healthy_threshold > 0);
}

#[test]
fn test_health_check_config_custom() {
    let config = CanonicalHealthCheckConfig {
        interval: Duration::from_secs(15),
        timeout: Duration::from_secs(10),
        unhealthy_threshold: 5,
        healthy_threshold: 3,
    };

    assert_eq!(config.unhealthy_threshold, 5);
}

// ============================================================================
// MONITORING CONFIG TESTS
// ============================================================================

#[test]
fn test_monitoring_config_default() {
    let config = CanonicalMonitoringConfig::default();

    assert!(config.enabled);
    assert_eq!(config.collection_interval, Duration::from_secs(60));
    assert_eq!(config.retention_period, Duration::from_secs(3600));
    assert_eq!(config.history_size, 1000);
}

#[test]
fn test_monitoring_config_retention_greater_than_interval() {
    let config = CanonicalMonitoringConfig::default();
    assert!(config.retention_period > config.collection_interval);
}

#[test]
fn test_monitoring_config_history_size_positive() {
    let config = CanonicalMonitoringConfig::default();
    assert!(config.history_size > 0);
}

#[test]
fn test_monitoring_config_custom() {
    let config = CanonicalMonitoringConfig {
        enabled: false,
        collection_interval: Duration::from_secs(120),
        retention_period: Duration::from_secs(7200),
        history_size: 500,
    };

    assert!(!config.enabled);
    assert_eq!(config.history_size, 500);
}

// ============================================================================
// SERVICE PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_service_performance_default() {
    let perf = CanonicalServicePerformance::default();

    assert_eq!(perf.total_requests, 0);
    assert_eq!(perf.successful_requests, 0);
    assert_eq!(perf.failed_requests, 0);
    // Use epsilon comparison for floats
    assert!((perf.success_rate - 1.0).abs() < f64::EPSILON);
    assert_eq!(perf.avg_response_time, Duration::from_millis(100));
}

#[test]
fn test_service_performance_success_rate_in_range() -> SongbirdResult<()> {
    let perf = CanonicalServicePerformance::default();
    assert!(perf.success_rate >= 0.0 && perf.success_rate <= 1.0);
    Ok(())
}

#[test]
fn test_service_performance_clone() -> SongbirdResult<()> {
    let perf1 = CanonicalServicePerformance::default();
    let perf2 = perf1.clone();

    assert_eq!(perf1.total_requests, perf2.total_requests);
    assert!((perf1.success_rate - perf2.success_rate).abs() < f64::EPSILON);

    Ok(())
}

#[test]
fn test_service_performance_serialization() -> SongbirdResult<()> {
    let perf = CanonicalServicePerformance::default();

    let json = serde_json::to_string(&perf)
        .map_err(|e| SongbirdError::configuration(format!("Failed: {}", e)))?;

    let deserialized: CanonicalServicePerformance = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed: {}", e)))?;

    assert_eq!(perf.total_requests, deserialized.total_requests);
    assert!((perf.success_rate - deserialized.success_rate).abs() < f64::EPSILON);

    Ok(())
}

// ============================================================================
// ADAPTER METRICS TESTS
// ============================================================================

#[test]
fn test_adapter_metrics_default() {
    let metrics = CanonicalAdapterMetrics::default();

    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.circuit_breaker_activations, 0);
}

#[test]
fn test_adapter_metrics_hashmaps_empty() -> SongbirdResult<()> {
    let metrics = CanonicalAdapterMetrics::default();

    assert!(metrics.requests_by_capability.is_empty());
    assert!(metrics.requests_by_service_type.is_empty());
    assert!(metrics.load_balancing_decisions.is_empty());

    Ok(())
}

#[test]
fn test_adapter_metrics_clone() -> SongbirdResult<()> {
    let metrics1 = CanonicalAdapterMetrics::default();
    let metrics2 = metrics1.clone();

    assert_eq!(metrics1.total_requests, metrics2.total_requests);
    assert_eq!(metrics1.circuit_breaker_activations, metrics2.circuit_breaker_activations);

    Ok(())
}

#[test]
fn test_adapter_metrics_debug() -> SongbirdResult<()> {
    let metrics = CanonicalAdapterMetrics::default();
    let debug_str = format!("{:?}", metrics);

    assert!(debug_str.contains("CanonicalAdapterMetrics"));

    Ok(())
}

// ============================================================================
// REQUEST PRIORITY TESTS
// ============================================================================

#[test]
fn test_request_priority_variants() -> SongbirdResult<()> {
    use CanonicalRequestPriority::*;

    let priorities = vec![Low, Normal, High, Critical];

    for priority in priorities {
        let debug = format!("{:?}", priority);
        assert!(!debug.is_empty());
    }

    Ok(())
}

#[test]
fn test_request_priority_ordering() -> SongbirdResult<()> {
    use CanonicalRequestPriority::*;

    assert!(Low < Normal);
    assert!(Normal < High);
    assert!(High < Critical);

    Ok(())
}

#[test]
fn test_request_priority_equality() -> SongbirdResult<()> {
    assert_eq!(CanonicalRequestPriority::Normal, CanonicalRequestPriority::Normal);
    assert_ne!(CanonicalRequestPriority::Low, CanonicalRequestPriority::High);

    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_full_config_serialization_roundtrip() -> SongbirdResult<()> {
    let config = CanonicalAdapterConfig::default();

    // Serialize
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed: {}", e)))?;

    // Deserialize
    let deserialized: CanonicalAdapterConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed: {}", e)))?;

    // Verify key values match
    assert_eq!(config.discovery.interval, deserialized.discovery.interval);
    assert_eq!(config.retry.max_attempts, deserialized.retry.max_attempts);
    assert_eq!(
        config.circuit_breaker.failure_threshold,
        deserialized.circuit_breaker.failure_threshold
    );

    Ok(())
}

#[test]
fn test_all_configs_have_debug() -> SongbirdResult<()> {
    let adapter_config = CanonicalAdapterConfig::default();
    let discovery = CanonicalDiscoveryConfig::default();
    let load_balancing = CanonicalLoadBalancingConfig::default();
    let circuit_breaker = CanonicalCircuitBreakerConfig::default();
    let retry = CanonicalRetryConfig::default();
    let timeouts = CanonicalTimeoutConfig::default();
    let health_check = CanonicalHealthCheckConfig::default();
    let monitoring = CanonicalMonitoringConfig::default();

    // All should have non-empty debug output
    assert!(!format!("{:?}", adapter_config).is_empty());
    assert!(!format!("{:?}", discovery).is_empty());
    assert!(!format!("{:?}", load_balancing).is_empty());
    assert!(!format!("{:?}", circuit_breaker).is_empty());
    assert!(!format!("{:?}", retry).is_empty());
    assert!(!format!("{:?}", timeouts).is_empty());
    assert!(!format!("{:?}", health_check).is_empty());
    assert!(!format!("{:?}", monitoring).is_empty());

    Ok(())
}

#[test]
fn test_all_configs_are_cloneable() {
    let config = CanonicalAdapterConfig::default();
    let cloned = config.clone();

    // If this compiles and runs, all nested configs are cloneable
    assert_eq!(config.retry.max_attempts, cloned.retry.max_attempts);
}
