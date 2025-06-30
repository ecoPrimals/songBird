/*!
 * Robustness Demo - Songbird Orchestrator
 *
 * Demonstrates advanced reliability and fault tolerance patterns:
 * - Circuit breaker pattern for fault isolation
 * - Exponential backoff retry logic with jitter
 * - Rate limiting and throttling
 * - Timeout management and adaptive timeouts
 * - Bulkhead pattern for resource isolation
 * - Health check coordination
 * - Comprehensive robustness statistics
 */

use std::time::Duration;
use tokio::time::sleep;

use songbird_gaming_bridge::{
    prelude::*,
    robustness_types::{
        BulkheadConfig, CircuitBreaker, CircuitBreakerConfig, HealthCheckConfig,
        HealthCheckStrategy, RateLimitConfig, RateLimitStrategy, RateLimiter, RetryConfig,
        RetryExecutor, RobustnessConfig, RobustnessManager,
    },
};

// Import TimeoutConfig from robustness module specifically
use songbird_gaming_bridge::robustness::TimeoutConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,songbird_orchestrator=debug")
        .init();

    println!("🛡️ Songbird Orchestrator - Robustness Demo");
    println!("===========================================");

    // Demo 1: Circuit Breaker Pattern
    println!("\n📋 Demo 1: Circuit Breaker Pattern");
    demo_circuit_breaker().await?;

    // Demo 2: Retry Logic with Exponential Backoff
    println!("\n📋 Demo 2: Retry Logic with Exponential Backoff");
    demo_retry_logic().await?;

    // Demo 3: Rate Limiting and Throttling
    println!("\n📋 Demo 3: Rate Limiting and Throttling");
    demo_rate_limiting().await?;

    // Demo 4: Comprehensive Robustness Manager
    println!("\n📋 Demo 4: Comprehensive Robustness Manager");
    demo_robustness_manager().await?;

    // Demo 5: Configuration Options
    println!("\n📋 Demo 5: Configuration Options");
    demo_configuration_options().await?;

    // Demo 6: Robustness Statistics
    println!("\n📋 Demo 6: Robustness Statistics");
    demo_robustness_statistics().await?;

    println!("\n✅ All robustness demos completed successfully!");
    Ok(())
}

/// Demonstrate circuit breaker pattern
async fn demo_circuit_breaker() -> Result<()> {
    println!("Testing circuit breaker pattern...");

    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout_seconds: 2,
        success_threshold: 2,
        minimum_request_threshold: 2,
        failure_rate_window_seconds: 10,
        failure_rate_threshold: 0.5,
    };

    let mut circuit_breaker = CircuitBreaker::new(config);

    println!("Circuit Breaker Configuration:");
    println!("  - Failure Threshold: 3");
    println!("  - Recovery Timeout: 2 seconds");
    println!("  - Success Threshold: 2");
    println!("  - Initial State: {:?}", circuit_breaker.get_state());

    // Simulate operations that will trigger circuit breaker
    println!("\nSimulating operations:");

    for i in 1..=8 {
        if circuit_breaker.can_execute() {
            // Simulate failure for requests 2, 3, 4 to trigger circuit breaker
            if (2..=4).contains(&i) {
                circuit_breaker.record_failure();
                println!(
                    "  Operation {}: ❌ Failed (State: {:?}, Failures: {})",
                    i,
                    circuit_breaker.get_state(),
                    circuit_breaker.get_failure_count()
                );
            } else {
                circuit_breaker.record_success();
                println!(
                    "  Operation {}: ✅ Success (State: {:?}, Failures: {})",
                    i,
                    circuit_breaker.get_state(),
                    circuit_breaker.get_failure_count()
                );
            }
        } else {
            println!("  Operation {}: 🚫 Circuit breaker is open", i);
        }

        // Wait for recovery timeout if circuit is open
        if i == 5 {
            println!("  Waiting for recovery timeout...");
            sleep(Duration::from_secs(3)).await;
        }
    }

    println!("✓ Circuit breaker demo completed");
    Ok(())
}

/// Demonstrate retry logic with exponential backoff
async fn demo_retry_logic() -> Result<()> {
    println!("Testing retry logic with exponential backoff...");

    let config = RetryConfig {
        max_retries: 4,
        base_delay_ms: 100,
        max_delay_ms: 2000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        retryable_errors: vec!["timeout".to_string(), "connection_refused".to_string()],
    };

    let retry_executor = RetryExecutor::new(config);

    println!("Retry Configuration:");
    println!("  - Max Retries: 4");
    println!("  - Base Delay: 100ms");
    println!("  - Backoff Multiplier: 2.0");
    println!("  - Jitter Factor: 0.1");

    // Test successful operation after retries
    println!("\nTesting operation that succeeds after 3 attempts:");
    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = retry_executor
        .execute(|| {
            let current_attempt =
                attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            Box::pin(async move {
                println!("  Attempt {}", current_attempt);
                if current_attempt < 3 {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "Simulated failure",
                    ))
                } else {
                    Ok("Success!")
                }
            })
        })
        .await;

    match result {
        Ok(value) => println!("  Final result: ✅ {}", value),
        Err(e) => println!("  Final result: ❌ {}", e),
    }

    println!("✓ Retry logic demo completed");
    Ok(())
}

/// Demonstrate rate limiting and throttling
async fn demo_rate_limiting() -> Result<()> {
    println!("Testing rate limiting and throttling...");

    let config = RateLimitConfig {
        max_requests_per_second: 5,
        burst_capacity: 10,
        window_seconds: 1,
        strategy: RateLimitStrategy::TokenBucket,
    };

    let rate_limiter = RateLimiter::new(config);

    println!("Rate Limiting Configuration:");
    println!("  - Max Requests/Second: 5");
    println!("  - Burst Capacity: 10");
    println!("  - Strategy: Token Bucket");

    println!("\nTesting rate limiting (20 rapid requests):");
    let mut allowed_count = 0;
    let mut denied_count = 0;

    for i in 1..=20 {
        if rate_limiter.is_allowed().await {
            allowed_count += 1;
            println!("  Request {}: ✅ Allowed", i);
        } else {
            denied_count += 1;
            println!("  Request {}: 🚫 Rate limited", i);
        }

        // Small delay to simulate rapid requests
        sleep(Duration::from_millis(50)).await;
    }

    println!(
        "Results: {} allowed, {} denied",
        allowed_count, denied_count
    );

    println!("✓ Rate limiting demo completed");
    Ok(())
}

/// Demonstrate comprehensive robustness manager
async fn demo_robustness_manager() -> Result<()> {
    println!("Testing comprehensive robustness manager...");

    let config = RobustnessConfig {
        circuit_breaker: CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout_seconds: 1,
            success_threshold: 1,
            minimum_request_threshold: 1,
            failure_rate_window_seconds: 5,
            failure_rate_threshold: 0.5,
        },
        retry: RetryConfig {
            max_retries: 2,
            base_delay_ms: 50,
            max_delay_ms: 500,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable_errors: vec!["timeout".to_string()],
        },
        rate_limiting: RateLimitConfig {
            max_requests_per_second: 10,
            burst_capacity: 15,
            window_seconds: 1,
            strategy: RateLimitStrategy::TokenBucket,
        },
        timeout: TimeoutConfig::default(),
        bulkhead: BulkheadConfig::default(),
        health_check: HealthCheckConfig::default(),
    };

    let robustness_manager = RobustnessManager::new(config);

    println!("Robustness Manager Configuration:");
    println!("  - Circuit Breaker: Failure threshold 2, Recovery 1s");
    println!("  - Retry: Max 2 retries, 50ms base delay");
    println!("  - Rate Limiting: 10 req/s, burst 15");

    // Test operations with different services
    let services = vec![
        "service-a".to_string(),
        "service-b".to_string(),
        "service-c".to_string(),
    ];

    println!("\nTesting operations with robustness patterns:");

    for service in &services {
        println!("  Testing {}:", service);

        // Simulate some operations
        for i in 1..=5 {
            let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let attempt_count_clone = attempt_count.clone();
            let service_name = service.clone();

            let result = robustness_manager
                .execute_with_robustness(&service_name, || {
                    let current_attempt =
                        attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    let service_inner = service_name.clone();
                    Box::pin(async move {
                        // Simulate different failure patterns per service
                        match service_inner.as_str() {
                            "service-a" => {
                                if i <= 2 && current_attempt == 1 {
                                    Err(std::io::Error::new(
                                        std::io::ErrorKind::TimedOut,
                                        "Timeout",
                                    ))
                                } else {
                                    Ok(format!("Success from {}", service_inner))
                                }
                            }
                            "service-b" => {
                                if i == 3 {
                                    Err(std::io::Error::new(
                                        std::io::ErrorKind::ConnectionRefused,
                                        "Connection failed",
                                    ))
                                } else {
                                    Ok(format!("Success from {}", service_inner))
                                }
                            }
                            _ => Ok(format!("Success from {}", service_inner)),
                        }
                    })
                })
                .await;

            match result {
                Ok(value) => println!("    Operation {}: ✅ {}", i, value),
                Err(e) => println!("    Operation {}: ❌ {}", i, e),
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    println!("✓ Robustness manager demo completed");
    Ok(())
}

/// Demonstrate configuration options
async fn demo_configuration_options() -> Result<()> {
    println!("Demonstrating configuration options...");

    // Default configuration
    let default_config = RobustnessConfig::default();
    println!("Default Configuration:");
    println!(
        "  - Circuit Breaker Threshold: {}",
        default_config.circuit_breaker.failure_threshold
    );
    println!("  - Max Retries: {}", default_config.retry.max_retries);
    println!(
        "  - Rate Limit: {} req/s",
        default_config.rate_limiting.max_requests_per_second
    );
    println!(
        "  - Default Timeout: {}s",
        default_config.timeout.default_timeout_seconds
    );

    // Custom high-availability configuration
    let ha_config = RobustnessConfig {
        circuit_breaker: CircuitBreakerConfig {
            failure_threshold: 10,
            recovery_timeout_seconds: 5,
            success_threshold: 5,
            minimum_request_threshold: 20,
            failure_rate_window_seconds: 60,
            failure_rate_threshold: 0.2,
        },
        retry: RetryConfig {
            max_retries: 5,
            base_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_refused".to_string(),
                "service_unavailable".to_string(),
                "internal_server_error".to_string(),
            ],
        },
        rate_limiting: RateLimitConfig {
            max_requests_per_second: 1000,
            burst_capacity: 2000,
            window_seconds: 1,
            strategy: RateLimitStrategy::SlidingWindowCounter,
        },
        timeout: TimeoutConfig {
            default_timeout_seconds: 60,
            health_check_timeout_seconds: 10,
            startup_timeout_seconds: 120,
            shutdown_timeout_seconds: 60,
            connection_timeout_seconds: 30,
            request_timeout_seconds: 45,
            adaptive_timeout_enabled: true,
            adaptive_timeout_percentile: 0.99,
        },
        bulkhead: BulkheadConfig {
            max_concurrent_per_service_id: 50,
            max_concurrent_global: 500,
            queue_size: 200,
            resource_pools: [
                ("database".to_string(), 20),
                ("cache".to_string(), 30),
                ("external_api".to_string(), 10),
            ]
            .into_iter()
            .collect(),
        },
        health_check: HealthCheckConfig {
            interval_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 2,
            success_threshold: 3,
            adaptive_enabled: true,
            strategies: [
                (
                    "http".to_string(),
                    HealthCheckStrategy::HttpEndpoint("/health".to_string()),
                ),
                ("tcp".to_string(), HealthCheckStrategy::Ping),
            ]
            .into_iter()
            .collect(),
        },
    };

    println!("\nHigh-Availability Configuration:");
    println!(
        "  - Circuit Breaker Threshold: {}",
        ha_config.circuit_breaker.failure_threshold
    );
    println!("  - Max Retries: {}", ha_config.retry.max_retries);
    println!(
        "  - Rate Limit: {} req/s",
        ha_config.rate_limiting.max_requests_per_second
    );
    println!(
        "  - Request Timeout: {}s",
        ha_config.timeout.request_timeout_seconds
    );
    println!(
        "  - Max Concurrent/Service: {}",
        ha_config.bulkhead.max_concurrent_per_service
    );
    println!(
        "  - Health Check Interval: {}s",
        ha_config.health_check.interval_seconds
    );

    println!("✓ Configuration options demo completed");
    Ok(())
}

/// Demonstrate robustness statistics
async fn demo_robustness_statistics() -> Result<()> {
    println!("Demonstrating robustness statistics...");

    let config = RobustnessConfig::default();
    let robustness_manager = RobustnessManager::new(config);

    // Generate some activity to create statistics
    let services = vec![
        "web-api".to_string(),
        "database".to_string(),
        "cache".to_string(),
        "auth-service".to_string(),
    ];

    println!("Generating activity for statistics...");
    for service in &services {
        for i in 1..=3 {
            let service_name = service.clone();
            let result = robustness_manager
                .execute_with_robustness(&service_name, || {
                    let service_inner = service_name.clone();
                    Box::pin(async move {
                        // Simulate different success rates
                        if service_inner == "database" && i == 2 {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "DB timeout",
                            ))
                        } else {
                            Ok(format!("Success from {}", service_inner))
                        }
                    })
                })
                .await;

            match result {
                Ok(_value) => println!("  {} operation {}: ✅", service, i),
                Err(e) => println!("  {} operation {}: ❌", service, e),
            }
        }
    }

    // Get and display statistics
    let stats = robustness_manager.get_stats().await;

    println!("\nRobustness Statistics:");
    println!(
        "  📊 Total Circuit Breakers: {}",
        stats.total_circuit_breakers
    );
    println!(
        "  🔴 Open Circuit Breakers: {}",
        stats.open_circuit_breakers
    );
    println!("  🔄 Circuit Breaker States:");

    for (service, state) in &stats.circuit_breaker_states {
        println!("    - {}: {:?}", service, state);
    }

    // Calculate overall health score
    let health_score = if stats.total_circuit_breakers > 0 {
        ((stats.total_circuit_breakers - stats.open_circuit_breakers) as f64
            / stats.total_circuit_breakers as f64)
            * 100.0
    } else {
        100.0
    };

    println!("  🎯 Overall Health Score: {:.1}%", health_score);

    println!("✓ Robustness statistics demo completed");
    Ok(())
}
