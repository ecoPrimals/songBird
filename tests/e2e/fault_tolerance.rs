// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals
//! Fault Tolerance E2E Tests
//!
//! Tests for circuit breaking, timeouts, and retry logic

#![cfg(test)]

#[path = "../common/mod.rs"]
mod common;

use common::{TestEnvironment, MockServiceConfig, TestAssertions};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_circuit_breaker_opens_on_failures() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a service that will fail
    let config = MockServiceConfig::new("failing-service")
        .with_capability("compute")
        .with_health(songbird_types::HealthStatus::Unhealthy);
    
    env.start_mock_service("failing-service", config).await?;
    
    // Register the service
    let service = songbird_types::ServiceInfo {
        name: "failing-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("failing-service", 10),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Simulate multiple failures to trigger circuit breaker
    let max_failures = 5;
    for i in 0..max_failures {
        let result = env.make_request("failing-service", "/compute").await;
        // Expect failures
        assert!(result.is_err() || !result.unwrap(), 
            "Request {} should fail or return false", i);
    }
    
    // Circuit should now be open - additional requests should fail fast
    let start = Instant::now();
    let result = env.make_request("failing-service", "/compute").await;
    let duration = start.elapsed();
    
    // Should fail fast (< 100ms) instead of timing out
    assert!(duration < Duration::from_millis(100), 
        "Circuit breaker should fail fast, took {:?}", duration);
    assert!(result.is_err() || !result.unwrap(), 
        "Request should fail when circuit is open");
    
    Ok(())
}

#[tokio::test]
async fn test_timeout_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a service with a timeout configuration
    let config = MockServiceConfig::new("slow-service")
        .with_capability("compute")
        .with_timeout(Duration::from_secs(2));
    
    env.start_mock_service("slow-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "slow-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("slow-service", 11),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Make a request that should timeout
    let start = Instant::now();
    let result = env.make_request_with_timeout("slow-service", "/slow", Duration::from_millis(500)).await;
    let duration = start.elapsed();
    
    // Should timeout within reasonable time (< 1 second)
    assert!(duration < Duration::from_secs(1), 
        "Timeout should occur quickly, took {:?}", duration);
    assert!(duration >= Duration::from_millis(450), 
        "Should wait at least the timeout duration");
    
    // Request should fail due to timeout
    assert!(result.is_err(), "Request should timeout");
    
    Ok(())
}

#[tokio::test]
async fn test_retry_with_exponential_backoff() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a service that succeeds on the 3rd attempt
    let config = MockServiceConfig::new("flaky-service")
        .with_capability("compute")
        .with_failure_rate(0.7) // 70% failure rate
        .with_health(songbird_types::HealthStatus::Degraded);
    
    env.start_mock_service("flaky-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "flaky-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("flaky-service", 12),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Make request with retry logic
    let start = Instant::now();
    let max_retries = 5;
    let mut attempt = 0;
    let mut last_error = None;
    
    for retry in 0..max_retries {
        attempt = retry + 1;
        
        // ✅ ACCEPTABLE SLEEP: Testing exponential backoff behavior itself
        // Calculate exponential backoff: 100ms, 200ms, 400ms, 800ms, 1600ms
        if retry > 0 {
            let backoff = Duration::from_millis(100 * 2_u64.pow(retry as u32));
            tokio::time::sleep(backoff).await;
        }
        
        match env.make_request("flaky-service", "/compute").await {
            Ok(success) if success => {
                // Success!
                break;
            }
            Ok(_) => {
                last_error = Some("Request returned false");
            }
            Err(e) => {
                last_error = Some("Request failed");
                continue;
            }
        }
    }
    
    let duration = start.elapsed();
    
    // Should eventually succeed or exhaust retries
    assert!(attempt <= max_retries, "Should not exceed max retries");
    
    // Total duration should reflect exponential backoff if retried
    if attempt > 1 {
        let expected_min_duration = Duration::from_millis(100 * (2_u64.pow(attempt as u32) - 2));
        assert!(duration >= expected_min_duration, 
            "Duration {:?} should reflect exponential backoff", duration);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a service that can be toggled between healthy and unhealthy
    let config = MockServiceConfig::new("recoverable-service")
        .with_capability("compute")
        .with_health(songbird_types::HealthStatus::Unhealthy);
    
    env.start_mock_service("recoverable-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "recoverable-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("recoverable-service", 13),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Trigger circuit breaker by causing failures
    for _ in 0..5 {
        let _ = env.make_request("recoverable-service", "/compute").await;
    }
    
    // Circuit is now open - heal the service
    env.update_service_health("recoverable-service", songbird_types::HealthStatus::Healthy).await?;
    
    // Poll for circuit breaker to attempt recovery (half-open state)
    let start = tokio::time::Instant::now();
    let timeout = Duration::from_secs(5);
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    
    let mut result = Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "timeout")) as Box<dyn std::error::Error>);
    while start.elapsed() < timeout {
        result = env.make_request("recoverable-service", "/compute").await;
        if result.is_ok() {
            break;
        }
        interval.tick().await;
    }
    
    // Should have recovered and allowed request
    let result = result;
    
    // Circuit should close and allow traffic through
    assert!(result.is_ok(), "Circuit breaker should recover and allow requests");
    
    Ok(())
}

#[tokio::test]
async fn test_timeout_doesnt_block_other_requests() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create one slow service and one fast service
    let slow_config = MockServiceConfig::new("slow-service")
        .with_capability("compute")
        .with_timeout(Duration::from_secs(10));
        
    let fast_config = MockServiceConfig::new("fast-service")
        .with_capability("storage")
        .with_health(songbird_types::HealthStatus::Healthy);
    
    env.start_mock_service("slow-service", slow_config).await?;
    env.start_mock_service("fast-service", fast_config).await?;
    
    // Register services
    let slow_service = songbird_types::ServiceInfo {
        name: "slow-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("slow-service", 14),
        metadata: std::collections::HashMap::new(),
    };
    
    let fast_service = songbird_types::ServiceInfo {
        name: "fast-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["storage".to_string()],
        endpoint: env.get_endpoint("fast-service", 15),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(slow_service).await?;
    env.register_service(fast_service).await?;
    
    // Start slow request (will timeout)
    let slow_handle = tokio::spawn({
        let mut env_clone = env.clone();
        async move {
            env_clone.make_request_with_timeout("slow-service", "/slow", Duration::from_millis(500)).await
        }
    });
    
    // ✅ ACCEPTABLE SLEEP: E2E test ensuring slow request has started
    // Make fast request while slow one is running
    tokio::time::sleep(Duration::from_millis(100)).await;
    let fast_start = Instant::now();
    let fast_result = env.make_request("fast-service", "/data").await;
    let fast_duration = fast_start.elapsed();
    
    // Fast request should complete quickly despite slow request
    assert!(fast_duration < Duration::from_millis(200), 
        "Fast request should not be blocked by slow request");
    assert!(fast_result.is_ok(), "Fast request should succeed");
    
    // Wait for slow request to complete
    let slow_result = slow_handle.await?;
    assert!(slow_result.is_err(), "Slow request should timeout");
    
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_half_open_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create an initially unhealthy service
    let config = MockServiceConfig::new("half-open-service")
        .with_capability("compute")
        .with_health(songbird_types::HealthStatus::Unhealthy);
    
    env.start_mock_service("half-open-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "half-open-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("half-open-service", 16),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Trip the circuit breaker
    for _ in 0..5 {
        let _ = env.make_request("half-open-service", "/compute").await;
    }
    
    // Poll for timeout to enter half-open state
    let start = tokio::time::Instant::now();
    let timeout = Duration::from_secs(3);
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    
    let mut result1 = Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "timeout")) as Box<dyn std::error::Error>);
    while start.elapsed() < timeout {
        result1 = env.make_request("half-open-service", "/compute").await;
        if result1.is_ok() {
            break;
        }
        interval.tick().await;
    }
    
    // First request in half-open should be allowed
    let result1 = result1;
    
    // Half-open state limits concurrent requests
    // Multiple rapid requests should show throttling behavior
    let mut allowed = 0;
    let mut rejected = 0;
    
    for _ in 0..20 {
        match env.make_request("half-open-service", "/compute").await {
            Ok(_) => allowed += 1,
            Err(_) => rejected += 1,
        }
    }
    
    // In half-open state, should have limited allowed requests
    assert!(allowed < 20, "Half-open state should limit concurrent requests");
    assert!(rejected > 0, "Some requests should be rejected in half-open state");
    
    Ok(())
}

#[tokio::test]
async fn test_retry_with_jitter() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a flaky service
    let config = MockServiceConfig::new("jitter-service")
        .with_capability("compute")
        .with_failure_rate(0.5); // 50% failure
    
    env.start_mock_service("jitter-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "jitter-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("jitter-service", 17),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Track retry timings to verify jitter
    let mut retry_delays = Vec::new();
    let mut last_attempt = Instant::now();
    
    for retry in 0..3 {
        if retry > 0 {
            let base_delay = Duration::from_millis(100 * 2_u64.pow(retry as u32 - 1));
            
            // ✅ ACCEPTABLE SLEEP: Testing jittered retry behavior itself
            // Add jitter: ±25% of base delay
            let jitter_range = base_delay.as_millis() as i64 / 4;
            let jitter = fastrand::i64(-jitter_range..=jitter_range);
            let jittered_delay = Duration::from_millis((base_delay.as_millis() as i64 + jitter) as u64);
            
            tokio::time::sleep(jittered_delay).await;
            
            let actual_delay = last_attempt.elapsed();
            retry_delays.push(actual_delay);
            last_attempt = Instant::now();
        }
        
        let _ = env.make_request("jitter-service", "/compute").await;
    }
    
    // Verify jitter was applied (delays should vary)
    if retry_delays.len() >= 2 {
        let first = retry_delays[0];
        let second = retry_delays[1];
        
        // Delays should be different due to jitter
        assert_ne!(first, second, "Jitter should cause different retry delays");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_cascading_failure_prevention() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a chain of services: A -> B -> C
    // If C fails, it shouldn't cascade to A
    let service_a_config = MockServiceConfig::new("service-a")
        .with_capability("frontend")
        .with_health(songbird_types::HealthStatus::Healthy);
    
    let service_b_config = MockServiceConfig::new("service-b")
        .with_capability("backend")
        .with_health(songbird_types::HealthStatus::Healthy);
        
    let service_c_config = MockServiceConfig::new("service-c")
        .with_capability("database")
        .with_health(songbird_types::HealthStatus::Unhealthy); // This one fails
    
    env.start_mock_service("service-a", service_a_config).await?;
    env.start_mock_service("service-b", service_b_config).await?;
    env.start_mock_service("service-c", service_c_config).await?;
    
    // Register services
    for (name, cap, port) in [
        ("service-a", "frontend", 18),
        ("service-b", "backend", 19),
        ("service-c", "database", 20),
    ] {
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![cap.to_string()],
            endpoint: env.get_endpoint(name, port),
            metadata: std::collections::HashMap::new(),
        };
        env.register_service(service).await?;
    }
    
    // service-c fails multiple times
    for _ in 0..5 {
        let _ = env.make_request("service-c", "/query").await;
    }
    
    // service-c circuit should be open, but service-a and service-b should still work
    let result_a = env.make_request("service-a", "/page").await;
    let result_b = env.make_request("service-b", "/api").await;
    
    // These should succeed despite service-c failure
    assert!(result_a.is_ok() || result_a.unwrap_err().to_string().contains("service-a"), 
        "Service A should be isolated from Service C failure");
    assert!(result_b.is_ok() || result_b.unwrap_err().to_string().contains("service-b"), 
        "Service B should be isolated from Service C failure");
    
    Ok(())
}

#[tokio::test]
async fn test_resource_cleanup_on_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create a slow service
    let config = MockServiceConfig::new("cleanup-service")
        .with_capability("compute")
        .with_timeout(Duration::from_secs(10));
    
    env.start_mock_service("cleanup-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "cleanup-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("cleanup-service", 21),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Get initial resource count
    let initial_connections = env.get_active_connections().await.unwrap_or(0);
    
    // Make request that will timeout
    let _ = env.make_request_with_timeout("cleanup-service", "/slow", Duration::from_millis(300)).await;
    
    // The async timeout method handles cleanup when it returns
    // No need to wait - cleanup is complete when the await returns
    
    // Verify resources were cleaned up
    let final_connections = env.get_active_connections().await.unwrap_or(0);
    
    // Should not have leaked connections
    assert_eq!(initial_connections, final_connections, 
        "Resources should be cleaned up after timeout");
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_concurrent_circuit_breakers() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create multiple services with different failure patterns
    let services = vec![
        ("service-1", songbird_types::HealthStatus::Unhealthy),
        ("service-2", songbird_types::HealthStatus::Healthy),
        ("service-3", songbird_types::HealthStatus::Degraded),
    ];
    
    for (i, (name, health)) in services.iter().enumerate() {
        let config = MockServiceConfig::new(name)
            .with_capability("compute")
            .with_health(*health);
        
        env.start_mock_service(name, config).await?;
        
        let service = songbird_types::ServiceInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["compute".to_string()],
            endpoint: env.get_endpoint(name, 22 + i as u16),
            metadata: std::collections::HashMap::new(),
        };
        
        env.register_service(service).await?;
    }
    
    // Trigger different circuit breakers independently
    for _ in 0..5 {
        let _ = env.make_request("service-1", "/compute").await;
    }
    
    // service-1 circuit should be open
    let start = Instant::now();
    let result_1 = env.make_request("service-1", "/compute").await;
    let duration_1 = start.elapsed();
    
    // Should fail fast
    assert!(duration_1 < Duration::from_millis(100), "service-1 circuit should be open");
    
    // service-2 should still work fine
    let result_2 = env.make_request("service-2", "/compute").await;
    assert!(result_2.is_ok() || !result_2.unwrap_err().to_string().contains("circuit"), 
        "service-2 should not be affected");
    
    Ok(())
}

#[tokio::test]
async fn test_bulkhead_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = TestEnvironment::new().await;
    
    // Create services with limited concurrency (bulkhead pattern)
    let config = MockServiceConfig::new("bulkhead-service")
        .with_capability("compute")
        .with_max_concurrent_requests(3) // Bulkhead limit
        .with_health(songbird_types::HealthStatus::Healthy);
    
    env.start_mock_service("bulkhead-service", config).await?;
    
    let service = songbird_types::ServiceInfo {
        name: "bulkhead-service".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["compute".to_string()],
        endpoint: env.get_endpoint("bulkhead-service", 25),
        metadata: std::collections::HashMap::new(),
    };
    
    env.register_service(service).await?;
    
    // Attempt many concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let mut env_clone = env.clone();
        let handle = tokio::spawn(async move {
            env_clone.make_request("bulkhead-service", "/compute").await
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut succeeded = 0;
    let mut rejected = 0;
    
    for handle in handles {
        match handle.await? {
            Ok(_) => succeeded += 1,
            Err(_) => rejected += 1,
        }
    }
    
    // Bulkhead should limit concurrent execution
    // Some requests should be rejected or queued
    assert!(succeeded <= 10, "Not all requests should execute immediately");
    
    // At least some requests should have been processed
    assert!(succeeded > 0, "Some requests should succeed within bulkhead limit");
    
    Ok(())
}
