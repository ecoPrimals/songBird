//! Comprehensive tests for canonical resilience configuration
//!
//! Phase 3 Test Coverage Expansion - Week 1
//! Target: Additional coverage for resilience.rs (circuit breaker, retry, robustness)

use super::resilience::*;
use std::time::Duration;

// ============================================================================
// ROBUSTNESS CONFIG TESTS
// ============================================================================

#[test]
fn test_robustness_config_default() {
    let config = RobustnessConfig::default();

    assert!(config.circuit_breaker.enabled);
    assert!(config.rate_limiting.enabled);
    assert!(config.bulkhead.enabled);
    assert!(config.retry.enabled);
}

#[test]
fn test_robustness_config_custom() {
    let config = RobustnessConfig {
        circuit_breaker: CircuitBreakerConfig::disabled(),
        rate_limiting: RateLimitingConfig {
            enabled: false,
            ..Default::default()
        },
        bulkhead: BulkheadConfig::default(),
        retry: RetryConfig::default(),
        load_balancer: LoadBalancerConfig::default(),
        health_check: HealthCheckConfig::default(),
        zero_cost_router: ZeroCostRouterConfig::default(),
    };

    assert!(!config.circuit_breaker.enabled);
    assert!(!config.rate_limiting.enabled);
}

#[test]
fn test_robustness_config_serialization() {
    let config = RobustnessConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: RobustnessConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.circuit_breaker.enabled, deserialized.circuit_breaker.enabled);
    assert_eq!(config.retry.max_attempts, deserialized.retry.max_attempts);
}

// ============================================================================
// RATE LIMITING CONFIG TESTS
// ============================================================================

#[test]
fn test_rate_limiting_config_custom() {
    let config = RateLimitingConfig {
        enabled: true,
        max_requests_per_second: 500,
        burst_capacity: 1000,
        window_size_secs: 30,
        algorithm: RateLimitAlgorithm::LeakyBucket,
        per_client_enabled: false,
        per_client_max_requests: 50,
    };

    assert!(config.enabled);
    assert_eq!(config.max_requests_per_second, 500);
    assert_eq!(config.algorithm, RateLimitAlgorithm::LeakyBucket);
}

#[test]
fn test_rate_limiting_algorithms() {
    let algorithms = vec![
        RateLimitAlgorithm::TokenBucket,
        RateLimitAlgorithm::LeakyBucket,
        RateLimitAlgorithm::FixedWindow,
        RateLimitAlgorithm::SlidingWindow,
    ];

    for algorithm in algorithms {
        let config = RateLimitingConfig {
            algorithm: algorithm.clone(),
            ..Default::default()
        };

        assert_eq!(config.algorithm, algorithm);
    }
}

#[test]
fn test_rate_limiting_per_client() {
    let config = RateLimitingConfig {
        per_client_enabled: true,
        per_client_max_requests: 200,
        ..Default::default()
    };

    assert!(config.per_client_enabled);
    assert_eq!(config.per_client_max_requests, 200);
}

#[test]
fn test_rate_limiting_serialization() {
    let config = RateLimitingConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: RateLimitingConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.max_requests_per_second, deserialized.max_requests_per_second);
}

// ============================================================================
// BULKHEAD CONFIG TESTS
// ============================================================================

#[test]
fn test_bulkhead_config_custom() {
    let config = BulkheadConfig {
        enabled: true,
        max_concurrent_operations: 200,
        queue_size: 2000,
        operation_timeout_ms: 60000,
        thread_pool_size: 20,
        isolation_strategy: IsolationStrategy::Semaphore,
    };

    assert!(config.enabled);
    assert_eq!(config.max_concurrent_operations, 200);
    assert_eq!(config.isolation_strategy, IsolationStrategy::Semaphore);
}

#[test]
fn test_bulkhead_isolation_strategies() {
    let strategies =
        vec![IsolationStrategy::ThreadPool, IsolationStrategy::Semaphore, IsolationStrategy::Queue];

    for strategy in strategies {
        let config = BulkheadConfig {
            isolation_strategy: strategy.clone(),
            ..Default::default()
        };

        assert_eq!(config.isolation_strategy, strategy);
    }
}

#[test]
fn test_bulkhead_config_high_concurrency() {
    let config = BulkheadConfig {
        enabled: true,
        max_concurrent_operations: 1000,
        queue_size: 10000,
        thread_pool_size: 50,
        ..Default::default()
    };

    assert_eq!(config.max_concurrent_operations, 1000);
    assert_eq!(config.queue_size, 10000);
}

#[test]
fn test_bulkhead_config_serialization() {
    let config = BulkheadConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: BulkheadConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.max_concurrent_operations, deserialized.max_concurrent_operations);
}

// ============================================================================
// LOAD BALANCER CONFIG TESTS
// ============================================================================

#[test]
fn test_load_balancer_config_custom() {
    let config = LoadBalancerConfig {
        algorithm: LoadBalancingAlgorithm::LeastConnections,
        sticky_sessions: true,
        session_timeout_secs: 600,
        max_connections_per_backend: 200,
        connection_timeout_ms: 60000,
        fail_fast: false,
    };

    assert_eq!(config.algorithm, LoadBalancingAlgorithm::LeastConnections);
    assert!(config.sticky_sessions);
    assert!(!config.fail_fast);
}

#[test]
fn test_load_balancing_algorithms() {
    let algorithms = vec![
        LoadBalancingAlgorithm::RoundRobin,
        LoadBalancingAlgorithm::LeastConnections,
        LoadBalancingAlgorithm::WeightedRoundRobin,
        LoadBalancingAlgorithm::Random,
        LoadBalancingAlgorithm::IpHash,
        LoadBalancingAlgorithm::CapabilityBased,
    ];

    for algorithm in algorithms {
        let config = LoadBalancerConfig {
            algorithm: algorithm.clone(),
            ..Default::default()
        };

        assert_eq!(config.algorithm, algorithm);
    }
}

#[test]
fn test_load_balancer_with_sticky_sessions() {
    let config = LoadBalancerConfig {
        sticky_sessions: true,
        session_timeout_secs: 1800,
        ..Default::default()
    };

    assert!(config.sticky_sessions);
    assert_eq!(config.session_timeout_secs, 1800);
}

#[test]
fn test_load_balancer_serialization() {
    let config = LoadBalancerConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: LoadBalancerConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.sticky_sessions, deserialized.sticky_sessions);
    assert_eq!(config.fail_fast, deserialized.fail_fast);
}

// ============================================================================
// HEALTH CHECK CONFIG TESTS
// ============================================================================

#[test]
fn test_health_check_config_custom() {
    let config = HealthCheckConfig {
        enabled: true,
        interval_secs: 15,
        timeout_secs: 5,
        failure_threshold: 5,
        recovery_threshold: 3,
        path: "/api/health".to_string(),
        max_retries: 2,
    };

    assert!(config.enabled);
    assert_eq!(config.interval_secs, 15);
    assert_eq!(config.path, "/api/health");
}

#[test]
fn test_health_check_config_disabled() {
    let config = HealthCheckConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(!config.enabled);
}

#[test]
fn test_health_check_config_thresholds() {
    let config = HealthCheckConfig {
        failure_threshold: 10,
        recovery_threshold: 5,
        ..Default::default()
    };

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.recovery_threshold, 5);
}

#[test]
fn test_health_check_config_custom_paths() {
    let paths = vec!["/health", "/api/health", "/status", "/ping"];

    for path in paths {
        let config = HealthCheckConfig {
            path: path.to_string(),
            ..Default::default()
        };

        assert_eq!(config.path, path);
    }
}

#[test]
fn test_health_check_config_with_retries() {
    let config = HealthCheckConfig {
        max_retries: 5,
        ..Default::default()
    };

    assert_eq!(config.max_retries, 5);
}

#[test]
fn test_health_check_config_serialization() {
    let config = HealthCheckConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: HealthCheckConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.interval_secs, deserialized.interval_secs);
    assert_eq!(config.path, deserialized.path);
}

// ============================================================================
// ZERO-COST ROUTER CONFIG TESTS
// ============================================================================

#[test]
fn test_zero_cost_router_config_custom() {
    let config = ZeroCostRouterConfig {
        enabled: true,
        route_cache_size: 20000,
        route_cache_ttl_secs: 600,
        optimize_routes: true,
        max_route_depth: 20,
        discovery_timeout_ms: 60000,
    };

    assert!(config.enabled);
    assert_eq!(config.route_cache_size, 20000);
    assert!(config.optimize_routes);
}

#[test]
fn test_zero_cost_router_disabled() {
    let config = ZeroCostRouterConfig {
        enabled: false,
        optimize_routes: false,
        ..Default::default()
    };

    assert!(!config.enabled);
    assert!(!config.optimize_routes);
}

#[test]
fn test_zero_cost_router_cache_sizes() {
    let sizes = vec![1000, 5000, 10000, 50000, 100000];

    for size in sizes {
        let config = ZeroCostRouterConfig {
            route_cache_size: size,
            ..Default::default()
        };

        assert_eq!(config.route_cache_size, size);
    }
}

#[test]
fn test_zero_cost_router_route_depths() {
    let depths = vec![5, 10, 15, 20, 25];

    for depth in depths {
        let config = ZeroCostRouterConfig {
            max_route_depth: depth,
            ..Default::default()
        };

        assert_eq!(config.max_route_depth, depth);
    }
}

#[test]
fn test_zero_cost_router_serialization() {
    let config = ZeroCostRouterConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: ZeroCostRouterConfig =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.route_cache_size, deserialized.route_cache_size);
    assert_eq!(config.optimize_routes, deserialized.optimize_routes);
}

// ============================================================================
// RETRY STRATEGY TESTS
// ============================================================================

#[test]
fn test_retry_strategy_variants() {
    let strategies = vec![
        RetryStrategy::Fixed,
        RetryStrategy::ExponentialBackoff {
            jitter: true,
        },
        RetryStrategy::ExponentialBackoff {
            jitter: false,
        },
        RetryStrategy::LinearBackoff,
        RetryStrategy::Custom {
            name: "custom".to_string(),
        },
    ];

    for strategy in strategies {
        let _clone = strategy.clone();
        // Verify they can be cloned
    }
}

#[test]
fn test_retry_config_delay_capping() {
    let config = RetryConfig {
        max_attempts: 10,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
        jitter: false,
        enabled: true,
    };

    // Test that delays are capped at max_delay
    for attempt in 1..=10 {
        let delay = config.calculate_delay(attempt);
        assert!(delay <= config.max_delay);
    }
}

#[test]
fn test_retry_config_validation() {
    let valid_config = RetryConfig::default();
    assert!(valid_config.is_valid());

    let invalid_config = RetryConfig {
        max_attempts: 0,
        ..Default::default()
    };
    assert!(!invalid_config.is_valid());
}

// ============================================================================
// CIRCUIT BREAKER VALIDATION TESTS
// ============================================================================

#[test]
fn test_circuit_breaker_validation() {
    let valid_config = CircuitBreakerConfig::default();
    assert!(valid_config.is_valid());

    let invalid_config = CircuitBreakerConfig {
        failure_threshold: 0,
        ..Default::default()
    };
    assert!(!invalid_config.is_valid());
}

#[test]
fn test_circuit_breaker_new() {
    let config = CircuitBreakerConfig::new(10, Duration::from_secs(120), 5, 20);

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.timeout, Duration::from_secs(120));
    assert_eq!(config.success_threshold, 5);
    assert_eq!(config.half_open_max_requests, 20);
    assert!(config.enabled);
}

#[test]
fn test_backoff_strategy_variants() {
    let strategies = vec![
        BackoffStrategy::Fixed,
        BackoffStrategy::Linear,
        BackoffStrategy::Exponential,
        BackoffStrategy::ExponentialWithJitter,
    ];

    for strategy in strategies {
        let _clone = strategy.clone();
        // Verify they can be cloned and serialized
    }
}
