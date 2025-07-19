//! Circuit breaker configuration tests

use std::time::Duration;
use songbird_network::communication::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

#[test]
fn test_circuit_breaker_config_default() {
    let config = CircuitBreakerConfig::default();

    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.timeout_duration, Duration::from_secs(60));
}

#[test]
fn test_circuit_breaker_config_custom() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 5,
        timeout_duration: Duration::from_secs(300),
    };

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 5);
    assert_eq!(config.timeout_duration, Duration::from_secs(300));
}

#[test]
fn test_circuit_breaker_config_edge_cases() {
    // Zero thresholds
    let config = CircuitBreakerConfig {
        failure_threshold: 0,
        success_threshold: 0,
        timeout_duration: Duration::from_millis(1),
    };

    let circuit_breaker = CircuitBreaker::new(config);
    // Should handle gracefully
    assert!(circuit_breaker.get_state().try_wait().is_ok());
}

#[test]
fn test_circuit_breaker_config_large_values() {
    let config = CircuitBreakerConfig {
        failure_threshold: u32::MAX,
        success_threshold: u32::MAX,
        timeout_duration: Duration::from_secs(u64::MAX),
    };

    let _circuit_breaker = CircuitBreaker::new(config);
    // Should create without panic
} 