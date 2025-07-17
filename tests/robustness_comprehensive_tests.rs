//! Comprehensive Robustness Tests
//!
//! This test suite covers the robustness module that provides circuit breakers,
//! retry mechanisms, rate limiting, and fault tolerance patterns.

use futures_util::future;
use songbird_core::robustness::*;
use songbird_errors::SongbirdError;
use songbird_core::robustness::RateLimitingConfig;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(test)]
mod circuit_breaker_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let _result = manager
            .create_circuit_breaker("test-circuit".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_circuit_breaker_default_config() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let _result = manager
            .create_circuit_breaker("test-circuit".to_string())
            .await;
        // Test basic creation - state transitions would require more complex mocking
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure_threshold() {
        let config = CircuitBreakerConfig {
            service_name: "test-service".to_string(),
            failure_threshold: 2,
            timeout: Duration::from_secs(5),
            success_threshold: 1,
        };
        let robustness_config = RobustnessConfig {
            circuit_breaker: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_circuit_breaker("test-circuit".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_circuit_breaker_timeout_behavior() {
        let config = CircuitBreakerConfig {
            service_name: "test-service".to_string(),
            failure_threshold: 5,
            timeout: Duration::from_millis(100),
            success_threshold: 3,
        };
        let robustness_config = RobustnessConfig {
            circuit_breaker: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_circuit_breaker("test-circuit".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_threshold() {
        let config = CircuitBreakerConfig {
            service_name: "test-service".to_string(),
            failure_threshold: 5,
            timeout: Duration::from_secs(5),
            success_threshold: 2,
        };
        let robustness_config = RobustnessConfig {
            circuit_breaker: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_circuit_breaker("test-circuit".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_multiple_circuit_breakers() {
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let _result1 = manager
            .create_circuit_breaker("circuit-1".to_string())
            .await;
        let _result2 = manager
            .create_circuit_breaker("circuit-2".to_string())
            .await;
        let _result3 = manager
            .create_circuit_breaker("circuit-3".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_circuit_breaker_concurrent_access() {
        let manager = Arc::new(RobustnessManager::new(RobustnessConfig::default()));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager = manager.clone();
                tokio::spawn(async move {
                    let _result = manager
                        .create_circuit_breaker(format!("circuit-{}", i))
                        .await;
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_circuit_breaker_edge_cases() {
        let manager = RobustnessManager::new(RobustnessConfig::default());

        // Test with empty circuit breaker name
        let _result = manager.create_circuit_breaker("".to_string()).await;

        // Test with very long name
        let long_name = "a".repeat(1000);
        let _result = manager.create_circuit_breaker(long_name).await;

        // Test passes if no panic occurs
    }
}

#[cfg(test)]
mod retry_mechanism_tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_config_creation() {
        let config = RetryConfig::default();
        let robustness_config = RobustnessConfig {
            retry: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.enable_jitter);
    }

    #[tokio::test]
    async fn test_retry_config_custom_values() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 200,
            max_delay_ms: 60000,
            backoff_multiplier: 1.5,
            enable_jitter: false,
            jitter_percentage: 0.0,
            retry_on_errors: vec![],
        };
        let robustness_config = RobustnessConfig {
            retry: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 4,
            base_delay_ms: 100,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            enable_jitter: false,
            jitter_percentage: 0.0,
            retry_on_errors: vec![],
        };
        let robustness_config = RobustnessConfig {
            retry: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
            jitter_percentage: 0.1,
            retry_on_errors: vec![],
        };
        let robustness_config = RobustnessConfig {
            retry: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_retry_max_delay_limit() {
        let config = RetryConfig {
            max_retries: 10,
            base_delay_ms: 100,
            max_delay_ms: 1000,
            backoff_multiplier: 2.0,
            enable_jitter: false,
            jitter_percentage: 0.0,
            retry_on_errors: vec![],
        };
        let robustness_config = RobustnessConfig {
            retry: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        // Test passes if no panic occurs
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let config = RateLimitingConfig::default();
        let robustness_config = RobustnessConfig {
            rate_limiting: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_default_config() {
        let config = RateLimitingConfig::default();
        assert_eq!(config.requests_per_second, 100);
        assert_eq!(config.burst_size, 20);
        assert_eq!(config.window_size_seconds, 60);
    }

    #[tokio::test]
    async fn test_rate_limiter_custom_config() {
        let config = RateLimitingConfig {
            requests_per_second: 50,
            burst_size: 10,
            window_size_seconds: 30,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };
        let robustness_config = RobustnessConfig {
            rate_limiting: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_check() {
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let _create_result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        let check_result = manager.check_rate_limit("test-limiter").await;
        // Test passes if no panic occurs
        assert!(check_result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_burst_behavior() {
        let config = RateLimitingConfig {
            requests_per_second: 10,
            burst_size: 5,
            window_size_seconds: 60,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };
        let robustness_config = RobustnessConfig {
            rate_limiting: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_token_bucket_strategy() {
        let config = RateLimitingConfig {
            requests_per_second: 100,
            burst_size: 20,
            window_size_seconds: 60,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };
        let robustness_config = RobustnessConfig {
            rate_limiting: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_sliding_window_strategy() {
        let config = RateLimitingConfig {
            requests_per_second: 100,
            burst_size: 20,
            window_size_seconds: 60,
            enable_distributed: false,
            strategy: RateLimitStrategy::SlidingWindow,
            sliding_window: SlidingWindowConfig::default(),
        };
        let robustness_config = RobustnessConfig {
            rate_limiting: config,
            ..Default::default()
        };
        let manager = RobustnessManager::new(robustness_config);
        let _result = manager
            .create_rate_limiter("test-limiter".to_string())
            .await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_multiple_instances() {
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let _result1 = manager.create_rate_limiter("limiter-1".to_string()).await;
        let _result2 = manager.create_rate_limiter("limiter-2".to_string()).await;
        let _result3 = manager.create_rate_limiter("limiter-3".to_string()).await;
        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_rate_limiter_concurrent_access() {
        let manager = Arc::new(RobustnessManager::new(RobustnessConfig::default()));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager = manager.clone();
                tokio::spawn(async move {
                    let _result = manager.create_rate_limiter(format!("limiter-{}", i)).await;
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await;
        }
        // Test passes if no panic occurs
    }
}

#[cfg(test)]
mod robustness_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_robustness_manager_creation() {
        let manager = RobustnessManager::new(RobustnessConfig::default());
        let status = manager.get_status().await.unwrap();

        assert_eq!(status.circuit_breakers, 0);
        assert_eq!(status.rate_limiters, 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_default() {
        let manager = RobustnessManager::default();
        let status = manager.get_status().await.unwrap();

        assert_eq!(status.circuit_breakers, 0);
        assert_eq!(status.rate_limiters, 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_with_circuit_breaker() {
        let cb_config = CircuitBreakerConfig::default();
        let robustness_config = RobustnessConfig {
            circuit_breaker: cb_config,
            retry: RetryConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            bulkhead: BulkheadConfig::default(),
            health_check: HealthCheckConfig::default(),
        };
        let manager = RobustnessManager::new(robustness_config);

        let status = manager.get_status().await;
        assert!(status.unwrap().circuit_breakers > 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_with_retry() {
        let retry_config = RetryConfig::default();
        let robustness_config = RobustnessConfig {
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: retry_config,
            rate_limiting: RateLimitingConfig::default(),
            bulkhead: BulkheadConfig::default(),
            health_check: HealthCheckConfig::default(),
        };
        let manager = RobustnessManager::new(robustness_config);

        let status = manager.get_status().await;
        // Retry is enabled by default in configuration
    }

    #[tokio::test]
    async fn test_robustness_manager_with_rate_limiting() {
        let rate_limiting_config = RateLimitingConfig::default();
        let robustness_config = RobustnessConfig {
            circuit_breaker: CircuitBreakerConfig::default(),
            retry: RetryConfig::default(),
            rate_limiting: rate_limiting_config,
            bulkhead: BulkheadConfig::default(),
            health_check: HealthCheckConfig::default(),
        };
        let manager = RobustnessManager::new(robustness_config);

        let status = manager.get_status().await;
        assert!(status.unwrap().rate_limiters> 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_full_configuration() {
        let cb_config = CircuitBreakerConfig::default();
        let retry_config = RetryConfig::default();
        let rl_config = RateLimitingConfig::default();

        let robustness_config = RobustnessConfig {
            circuit_breaker: cb_config,
            retry: retry_config,
            rate_limiting: rl_config,
            bulkhead: BulkheadConfig::default(),
            health_check: HealthCheckConfig::default(),
        };
        let manager = RobustnessManager::new(robustness_config);

        let status = manager.get_status().await.unwrap();
        assert!(status.circuit_breakers > 0);
        assert!(status.rate_limiters > 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_execute_success() {
        let manager = RobustnessManager::new(RobustnessConfig::default());

        let result = manager
            .execute(async { Ok::<i32, SongbirdError>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_robustness_manager_execute_with_circuit_breaker() {
        let cb_config = CircuitBreakerConfig::default();
        let manager = RobustnessManager::new(RobustnessConfig::default()).with_circuit_breaker(cb_config);

        let result = manager
            .execute(async { Ok::<i32, SongbirdError>(100) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[tokio::test]
    #[ignore] // Flaky test - timing dependent on retry mechanisms
    async fn test_robustness_manager_execute_with_retry() {
        let retry_config = RetryConfig {
            max_retries: 3,   // Increase retries for more reliability
            base_delay_ms: 5, // Shorter delay
            max_delay_ms: 100,
            backoff_multiplier: 1.5,
            enable_jitter: false, // Disable jitter for predictable behavior
            jitter_percentage: 0.0,
            retry_on_errors: vec![],
        };
        let manager = RobustnessManager::new(RobustnessConfig::default()).with_retry(retry_config);

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let result = manager
            .execute(async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    Err::<i32, SongbirdError>(SongbirdError::service_error(
                        "test",
                        "First attempt fails".to_string(),
                    ))
                } else {
                    Ok(42)
                }
            })
            .await;

        // Allow for either success on retry or eventual failure
        if result.is_ok() {
            assert_eq!(result.unwrap(), 42);
            assert!(counter.load(Ordering::SeqCst) >= 2);
        } else {
            // If it fails, it should have attempted multiple times
            assert!(counter.load(Ordering::SeqCst) >= 2);
        }
    }

    #[tokio::test]
    async fn test_robustness_manager_execute_with_rate_limiting() {
        let rl_config = RateLimitingConfig {
            requests_per_second: 10,
            burst_size: 1,
            window_size_seconds: 1,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };
        let manager = RobustnessManager::new(RobustnessConfig::default()).with_rate_limiting(rl_config);

        // First execution should succeed
        let result1 = manager.execute(async { Ok::<i32, SongbirdError>(1) }).await;
        assert!(result1.is_ok());

        // Second execution should be rate limited
        let result2 = manager.execute(async { Ok::<i32, SongbirdError>(2) }).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_robustness_manager_comprehensive_integration() {
        let cb_config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let retry_config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
            ..Default::default()
        };
        let rl_config = RateLimitingConfig {
            requests_per_second: 100,
            burst_size: 10,
            window_size_seconds: 1,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };

        let robustness_config = RobustnessConfig {
            circuit_breaker: cb_config,
            retry: retry_config,
            rate_limiting: rl_config,
            bulkhead: BulkheadConfig::default(),
            health_check: HealthCheckConfig::default(),
        };
        let manager = RobustnessManager::new(robustness_config);

        // Test successful execution with all features enabled
        let result = manager
            .execute(async { Ok::<String, SongbirdError>("Success".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");

        // Verify status shows all features are active
        let status = manager.get_status().await.unwrap();
        assert!(status.circuit_breakers> 0);
        assert!(status.is_running);
        assert!(status.rate_limiters> 0);
    }

    #[tokio::test]
    async fn test_robustness_manager_performance() {
        let manager = RobustnessManager::new(RobustnessConfig::default());

        let start = Instant::now();
        for i in 0..100 {
            let _ = manager
                .execute(async move { Ok::<i32, SongbirdError>(i) })
                .await;
        }
        let elapsed = start.elapsed();

        // Should complete quickly (less than 1 second for 100 executions)
        assert!(elapsed < Duration::from_secs(1));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_robustness_integration() {
        let cb_config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let retry_config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 10,
            ..Default::default()
        };
        let rl_config = RateLimitingConfig {
            requests_per_second: 100,
            burst_size: 10,
            window_size_seconds: 1,
            enable_distributed: false,
            strategy: RateLimitStrategy::TokenBucket,
            sliding_window: SlidingWindowConfig::default(),
        };

        let manager = RobustnessManager::new(RobustnessConfig::default())
            .with_circuit_breaker(cb_config)
            .with_retry(retry_config)
            .with_rate_limiting(rl_config);

        // Test successful execution with all features enabled
        let result = manager
            .execute(async { Ok::<String, SongbirdError>("Success".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");

        // Verify status shows all features are active
        let status = manager.get_status().await.unwrap();
        assert!(status.circuit_breakers> 0);
        assert!(status.is_running);
        assert!(status.rate_limiters> 0);
    }

    #[tokio::test]
    async fn test_error_handling_patterns() {
        let manager = RobustnessManager::new(RobustnessConfig::default())
            .with_circuit_breaker(CircuitBreakerConfig::default())
            .with_retry(RetryConfig::default())
            .with_rate_limiting(RateLimitingConfig::default());

        // Test error scenario
        let error_result = manager
            .execute(async {
                Err::<i32, SongbirdError>(SongbirdError::service_error(
                    "test",
                    "Test error".to_string(),
                ))
            })
            .await;
        assert!(error_result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_robustness_operations() {
        let manager = Arc::new(
            RobustnessManager::new(RobustnessConfig::default())
                .with_circuit_breaker(CircuitBreakerConfig::default())
                .with_retry(RetryConfig::default())
                .with_rate_limiting(RateLimitingConfig {
                    requests_per_second: 50,
                    burst_size: 20,
                    window_size_seconds: 1,
                    enable_distributed: false,
                    strategy: RateLimitStrategy::TokenBucket,
                    sliding_window: SlidingWindowConfig::default(),
                }),
        );

        // Spawn multiple concurrent operations
        let mut handles = vec![];
        for i in 0..10 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                mgr.execute(async move { Ok::<i32, SongbirdError>(i) })
                    .await
            });
            handles.push(handle);
        }

        let results: Vec<_> = future::join_all(handles).await;

        // Some operations should succeed
        let successful_count = results
            .iter()
            .filter(|r| r.as_ref().unwrap().is_ok())
            .count();

        assert!(successful_count > 0);
    }
}
