// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use serde::de::DeserializeOwned;

fn serde_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let decoded: T = serde_json::from_str(&json).expect("deserialize");
    let reencoded = serde_json::to_string(&decoded).expect("re-serialize");
    assert_eq!(json, reencoded);
}

fn rate_limiting_invariants(config: &RateLimitingConfig) -> bool {
    config.max_requests_per_second > 0
        && config.burst_capacity > 0
        && config.window_size_secs > 0
        && config.per_client_max_requests > 0
}

fn bulkhead_invariants(config: &BulkheadConfig) -> bool {
    config.max_concurrent_operations > 0
        && config.queue_size > 0
        && config.operation_timeout_ms > 0
        && config.thread_pool_size > 0
}

#[test]
fn test_circuit_breaker_config_default() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert!(config.enabled);
    assert!(config.is_valid());
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert!(config.enabled);
    assert!(config.is_valid());
}

#[test]
fn test_retry_delay_calculation() {
    let config = RetryConfig::default();

    let delay1 = config.calculate_delay(1);
    let delay2 = config.calculate_delay(2);
    let delay3 = config.calculate_delay(3);

    assert_eq!(delay1, Duration::from_millis(100));
    assert_eq!(delay2, Duration::from_millis(200));
    assert_eq!(delay3, Duration::from_millis(400));
}

#[test]
fn test_disabled_configs() {
    let cb_config = CircuitBreakerConfig::disabled();
    let retry_config = RetryConfig::disabled();

    assert!(!cb_config.enabled);
    assert!(!retry_config.enabled);

    let delay = retry_config.calculate_delay(1);
    assert_eq!(delay, Duration::from_millis(0));
}

#[test]
fn test_circuit_breaker_states() {
    assert_eq!(CircuitBreakerState::default(), CircuitBreakerState::Closed);
    assert_eq!(
        RetryStrategy::default(),
        RetryStrategy::ExponentialBackoff {
            jitter: true
        }
    );
}

// --- RateLimitingConfig ---

#[test]
fn test_rate_limiting_config_default() {
    let config = RateLimitingConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_requests_per_second, 1000);
    assert_eq!(config.burst_capacity, 2000);
    assert_eq!(config.window_size_secs, 60);
    assert_eq!(config.algorithm, RateLimitAlgorithm::TokenBucket);
    assert!(config.per_client_enabled);
    assert_eq!(config.per_client_max_requests, 100);
    assert!(rate_limiting_invariants(&config));
}

#[test]
fn test_rate_limiting_config_invalid_invariants() {
    let zero_rps = RateLimitingConfig {
        max_requests_per_second: 0,
        ..Default::default()
    };
    assert!(!rate_limiting_invariants(&zero_rps));

    let zero_burst = RateLimitingConfig {
        burst_capacity: 0,
        ..Default::default()
    };
    assert!(!rate_limiting_invariants(&zero_burst));
}

#[test]
fn test_rate_limiting_config_serde_roundtrip() {
    serde_roundtrip(&RateLimitingConfig::default());
}

// --- BulkheadConfig ---

#[test]
fn test_bulkhead_config_default() {
    let config = BulkheadConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_concurrent_operations, 100);
    assert_eq!(config.queue_size, 1000);
    assert_eq!(config.operation_timeout_ms, 30000);
    assert_eq!(config.thread_pool_size, 10);
    assert_eq!(config.isolation_strategy, IsolationStrategy::ThreadPool);
    assert!(bulkhead_invariants(&config));
}

#[test]
fn test_bulkhead_config_invalid_invariants() {
    let zero_concurrency = BulkheadConfig {
        max_concurrent_operations: 0,
        ..Default::default()
    };
    assert!(!bulkhead_invariants(&zero_concurrency));

    let zero_queue = BulkheadConfig {
        queue_size: 0,
        ..Default::default()
    };
    assert!(!bulkhead_invariants(&zero_queue));
}

#[test]
fn test_bulkhead_config_serde_roundtrip() {
    serde_roundtrip(&BulkheadConfig::default());
}

// --- LoadBalancerConfig ---

#[test]
fn test_load_balancer_config_default() {
    let config = LoadBalancerConfig::default();
    assert_eq!(config.algorithm, LoadBalancingAlgorithm::RoundRobin);
    assert!(!config.sticky_sessions);
    assert_eq!(config.session_timeout_secs, 300);
    assert_eq!(config.max_connections_per_backend, 100);
    assert_eq!(config.connection_timeout_ms, 30000);
    assert!(config.fail_fast);
}

#[test]
fn test_load_balancer_config_serde_roundtrip() {
    serde_roundtrip(&LoadBalancerConfig::default());
}

// --- HealthCheckConfig ---

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();
    assert!(config.enabled);
    assert_eq!(config.interval_secs, 30);
    assert_eq!(config.timeout_secs, 10);
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.recovery_threshold, 2);
    assert_eq!(config.path, "/health");
    assert_eq!(config.max_retries, 0);
}

#[test]
fn test_health_check_config_serde_roundtrip() {
    serde_roundtrip(&HealthCheckConfig::default());
}

#[test]
fn test_health_check_config_serde_empty_object_uses_defaults() {
    let config: HealthCheckConfig =
        serde_json::from_str("{}").expect("empty object should deserialize");
    let defaults = HealthCheckConfig::default();
    assert_eq!(config.enabled, defaults.enabled);
    assert_eq!(config.interval_secs, defaults.interval_secs);
    assert_eq!(config.timeout_secs, defaults.timeout_secs);
    assert_eq!(config.failure_threshold, defaults.failure_threshold);
    assert_eq!(config.recovery_threshold, defaults.recovery_threshold);
    assert_eq!(config.path, defaults.path);
    assert_eq!(config.max_retries, defaults.max_retries);
}

// --- ZeroCostRouterConfig ---

#[test]
fn test_zero_cost_router_config_default() {
    let config = ZeroCostRouterConfig::default();
    assert!(config.enabled);
    assert_eq!(config.route_cache_size, 10000);
    assert_eq!(config.route_cache_ttl_secs, 300);
    assert!(config.optimize_routes);
    assert_eq!(config.max_route_depth, 10);
    assert_eq!(config.discovery_timeout_ms, 30000);
}

#[test]
fn test_zero_cost_router_config_serde_roundtrip() {
    serde_roundtrip(&ZeroCostRouterConfig::default());
}

// --- RobustnessConfig ---

#[test]
fn test_robustness_config_default_composition() {
    let config = RobustnessConfig::default();
    assert!(config.circuit_breaker.is_valid());
    assert!(config.retry.is_valid());
    assert!(config.rate_limiting.enabled);
    assert!(config.bulkhead.enabled);
    assert_eq!(config.load_balancer.algorithm, LoadBalancingAlgorithm::RoundRobin);
    assert!(config.health_check.enabled);
    assert!(config.zero_cost_router.enabled);
}

#[test]
fn test_robustness_config_serde_roundtrip() {
    serde_roundtrip(&RobustnessConfig::default());
}

#[test]
fn test_robustness_config_serde_empty_object_uses_defaults() {
    let config: RobustnessConfig =
        serde_json::from_str("{}").expect("empty object should deserialize");
    let defaults = RobustnessConfig::default();
    assert!(config.circuit_breaker.is_valid());
    assert!(config.retry.is_valid());
    assert_eq!(config.rate_limiting.enabled, defaults.rate_limiting.enabled);
    assert_eq!(config.bulkhead.enabled, defaults.bulkhead.enabled);
    assert_eq!(config.load_balancer.algorithm, defaults.load_balancer.algorithm);
    assert_eq!(config.health_check.path, defaults.health_check.path);
    assert_eq!(
        config.zero_cost_router.route_cache_size,
        defaults.zero_cost_router.route_cache_size
    );
}

// --- CircuitBreakerConfig::is_valid edge cases ---

#[test]
fn test_circuit_breaker_is_valid_rejects_zero_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 0,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_circuit_breaker_is_valid_rejects_zero_success_threshold() {
    let config = CircuitBreakerConfig {
        success_threshold: 0,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_circuit_breaker_is_valid_rejects_zero_half_open_max() {
    let config = CircuitBreakerConfig {
        half_open_max_requests: 0,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_circuit_breaker_is_valid_rejects_zero_timeout() {
    let config = CircuitBreakerConfig {
        timeout: Duration::ZERO,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_circuit_breaker_new_is_valid() {
    let config = CircuitBreakerConfig::new(10, Duration::from_secs(30), 2, 5);
    assert!(config.enabled);
    assert!(config.is_valid());
}

#[test]
fn test_circuit_breaker_config_serde_roundtrip() {
    serde_roundtrip(&CircuitBreakerConfig::default());
}

// --- RetryConfig::is_valid edge cases ---

#[test]
fn test_retry_is_valid_rejects_zero_max_attempts() {
    let config = RetryConfig {
        max_attempts: 0,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_retry_is_valid_rejects_zero_initial_delay() {
    let config = RetryConfig {
        initial_delay: Duration::ZERO,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_retry_is_valid_rejects_zero_max_delay() {
    let config = RetryConfig {
        max_delay: Duration::ZERO,
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_retry_is_valid_rejects_non_positive_backoff() {
    let config = RetryConfig {
        backoff_multiplier: 0.0,
        ..Default::default()
    };
    assert!(!config.is_valid());

    let negative = RetryConfig {
        backoff_multiplier: -1.0,
        ..Default::default()
    };
    assert!(!negative.is_valid());
}

#[test]
fn test_retry_is_valid_rejects_max_delay_less_than_initial() {
    let config = RetryConfig {
        initial_delay: Duration::from_secs(10),
        max_delay: Duration::from_secs(1),
        ..Default::default()
    };
    assert!(!config.is_valid());
}

#[test]
fn test_retry_calculate_delay_respects_max_cap() {
    let config = RetryConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(250),
        backoff_multiplier: 2.0,
        ..Default::default()
    };
    assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
    assert_eq!(config.calculate_delay(2), Duration::from_millis(200));
    assert_eq!(config.calculate_delay(3), Duration::from_millis(250));
}

#[test]
fn test_retry_config_serde_roundtrip() {
    serde_roundtrip(&RetryConfig::default());
}

// --- Enum serialization roundtrips ---

#[test]
fn test_rate_limit_algorithm_serde_roundtrip() {
    for algo in [
        RateLimitAlgorithm::TokenBucket,
        RateLimitAlgorithm::LeakyBucket,
        RateLimitAlgorithm::FixedWindow,
        RateLimitAlgorithm::SlidingWindow,
    ] {
        serde_roundtrip(&algo);
    }
}

#[test]
fn test_isolation_strategy_serde_roundtrip() {
    for strategy in
        [IsolationStrategy::ThreadPool, IsolationStrategy::Semaphore, IsolationStrategy::Queue]
    {
        serde_roundtrip(&strategy);
    }
}

#[test]
fn test_load_balancing_algorithm_serde_roundtrip() {
    for algo in [
        LoadBalancingAlgorithm::RoundRobin,
        LoadBalancingAlgorithm::LeastConnections,
        LoadBalancingAlgorithm::WeightedRoundRobin,
        LoadBalancingAlgorithm::Random,
        LoadBalancingAlgorithm::IpHash,
        LoadBalancingAlgorithm::CapabilityBased,
    ] {
        serde_roundtrip(&algo);
    }
}

#[test]
fn test_backoff_strategy_serde_roundtrip() {
    for strategy in [
        BackoffStrategy::Fixed,
        BackoffStrategy::Linear,
        BackoffStrategy::Exponential,
        BackoffStrategy::ExponentialWithJitter,
    ] {
        serde_roundtrip(&strategy);
    }
}

#[test]
fn test_circuit_breaker_state_serde_roundtrip() {
    for state in
        [CircuitBreakerState::Closed, CircuitBreakerState::Open, CircuitBreakerState::HalfOpen]
    {
        serde_roundtrip(&state);
    }
}

#[test]
fn test_retry_strategy_serde_roundtrip() {
    for strategy in [
        RetryStrategy::Fixed,
        RetryStrategy::ExponentialBackoff {
            jitter: false,
        },
        RetryStrategy::LinearBackoff,
        RetryStrategy::Custom {
            name: String::from("custom"),
        },
    ] {
        serde_roundtrip(&strategy);
    }
}

#[test]
fn test_retry_calculate_delay_zero_when_disabled() {
    let config = RetryConfig::disabled();
    assert_eq!(config.calculate_delay(5), Duration::from_millis(0));
}

#[test]
fn test_retry_calculate_delay_zero_for_attempt_zero() {
    let config = RetryConfig::default();
    assert_eq!(config.calculate_delay(0), Duration::from_millis(0));
}
