//! Tests for configuration types module

use super::*;
use std::time::Duration;

#[test]
fn test_security_config_creation() {
    let config = SecurityConfig {
        enabled: true,
        level: SecurityLevel::High,
        authentication_required: true,
        tls_enabled: true,
        certificate_path: Some("/path/to/cert".to_string()),
    };

    assert!(config.enabled);
    assert_eq!(config.level, SecurityLevel::High);
    assert!(config.tls_enabled);
}

#[test]
fn test_load_balancing_strategy_default() {
    let strategy = LoadBalancingStrategy::default();
    assert_eq!(strategy, LoadBalancingStrategy::RoundRobin);
}

#[test]
fn test_load_balancing_strategy_variants() {
    let strategies = [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::WeightedRoundRobin,
    ];
    assert_eq!(strategies.len(), 4);
}

#[test]
fn test_load_balancing_config_default() {
    let config = LoadBalancingConfig::default();
    assert_eq!(config.strategy, LoadBalancingStrategy::RoundRobin);
    assert!(config.health_check_enabled);
    assert_eq!(config.connection_timeout_ms, 5000);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_load_balancing_config_custom() {
    let config = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        health_check_enabled: false,
        connection_timeout_ms: 10000,
        max_retries: 5,
    };

    assert_eq!(config.strategy, LoadBalancingStrategy::LeastConnections);
    assert!(!config.health_check_enabled);
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.backoff_multiplier, 2.0);
}

#[test]
fn test_retry_config_custom() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(30),
        backoff_multiplier: 1.5,
        jitter: true,
        enabled: true,
    };

    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.backoff_multiplier, 1.5);
}

#[test]
fn test_circuit_breaker_config_default() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.half_open_max_requests, 10);
    assert!(config.enabled);
}

#[test]
fn test_circuit_breaker_config_custom() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        timeout: Duration::from_secs(120),
        success_threshold: 3,
        half_open_max_requests: 5,
        enabled: true,
    };

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 3);
}

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();
    assert_eq!(config.interval_secs, 30);
    assert_eq!(config.timeout_secs, 10);
    assert_eq!(config.recovery_threshold, 2);
    assert_eq!(config.failure_threshold, 3);
    assert!(config.enabled);
}

#[test]
fn test_health_check_config_custom() {
    let config = HealthCheckConfig {
        enabled: true,
        interval_secs: 60,
        timeout_secs: 10,
        failure_threshold: 5,
        recovery_threshold: 3,
        path: "/health".to_string(),
        max_retries: 3,
    };

    assert_eq!(config.interval_secs, 60);
    assert_eq!(config.failure_threshold, 5);
}

#[test]
fn test_feature_flags_creation() {
    let flags = FeatureFlags {
        experimental_features: true,
        verbose_logging: false,
        metrics_enabled: true,
        tracing_enabled: true,
        auto_discovery: false,
    };

    assert!(flags.experimental_features);
    assert!(!flags.verbose_logging);
    assert!(flags.metrics_enabled);
    assert!(flags.tracing_enabled);
    assert!(!flags.auto_discovery);
}

#[test]
fn test_load_balancing_config_clone() {
    let config = LoadBalancingConfig::default();
    let cloned = config.clone();
    assert_eq!(config.strategy, cloned.strategy);
}

#[test]
fn test_retry_config_clone() {
    let config = RetryConfig::default();
    let cloned = config.clone();
    assert_eq!(config.max_attempts, cloned.max_attempts);
}

#[test]
fn test_circuit_breaker_config_clone() {
    let config = CircuitBreakerConfig::default();
    let cloned = config.clone();
    assert_eq!(config.failure_threshold, cloned.failure_threshold);
}

#[test]
fn test_health_check_config_clone() {
    let config = HealthCheckConfig::default();
    let cloned = config.clone();
    assert_eq!(config.interval_secs, cloned.interval_secs);
}

#[test]
fn test_feature_flags_clone() {
    let flags = FeatureFlags {
        experimental_features: true,
        verbose_logging: true,
        metrics_enabled: true,
        tracing_enabled: false,
        auto_discovery: true,
    };
    let cloned = flags.clone();
    assert_eq!(flags.experimental_features, cloned.experimental_features);
}
