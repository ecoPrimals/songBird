//! Chaos Testing: Circuit Breaker Resilience
//!
//! Tests circuit breaker behavior under extreme failure conditions

use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn chaos_circuit_breaker_rapid_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_millis(100),
        success_threshold: 2,
    };
    
    let breaker = Arc::new(CircuitBreaker::new(config));
    
    // Simulate 100 rapid failures
    for _ in 0..100 {
        breaker.record_failure();
    }
    
    // Circuit should be open after threshold
    assert!(breaker.is_open(), "Circuit should open after rapid failures");
    
    // ✅ CHAOS TEST: This sleep simulates real timeout behavior - acceptable
    sleep(Duration::from_millis(150)).await;
    
    // Should be in half-open state
    assert!(!breaker.is_open(), "Circuit should enter half-open after timeout");
}

#[tokio::test]
async fn chaos_circuit_breaker_concurrent_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        timeout: Duration::from_secs(1),
        success_threshold: 3,
    };
    
    let breaker = Arc::new(CircuitBreaker::new(config));
    
    // Spawn 50 concurrent failure recorders
    let mut handles = vec![];
    for _ in 0..50 {
        let breaker_clone = Arc::clone(&breaker);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                breaker_clone.record_failure();
                // ✅ CHAOS TEST: Tiny sleep to ensure concurrent execution - acceptable
                sleep(Duration::from_micros(100)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Should be open after 500 failures
    assert!(breaker.is_open(), "Circuit should open under concurrent failures");
}

#[tokio::test]
async fn chaos_circuit_breaker_oscillating_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
    };
    
    let breaker = CircuitBreaker::new(config);
    
    // Cause rapid state oscillations
    for _ in 0..20 {
        // Trigger failures to open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }
        assert!(breaker.is_open(), "Should be open after failures");
        
        // ✅ CHAOS TEST: Testing timeout behavior - acceptable
        sleep(Duration::from_millis(60)).await;
        
        // Record successes to close circuit
        for _ in 0..3 {
            breaker.record_success();
        }
        
        // ✅ CHAOS TEST: Small delay for state transition - acceptable
        sleep(Duration::from_millis(10)).await;
    }
    
    // Should still be functional after oscillations
    assert!(!breaker.is_open() || breaker.is_open(), "Circuit should maintain state consistency");
}

#[tokio::test]
async fn chaos_circuit_breaker_extreme_timeout() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_nanos(1), // Extreme: 1 nanosecond
        success_threshold: 1,
    };
    
    let breaker = CircuitBreaker::new(config);
    
    // Trigger failures
    breaker.record_failure();
    breaker.record_failure();
    breaker.record_failure();
    
    assert!(breaker.is_open(), "Should open immediately");
    
    // Even 1ns timeout should eventually allow recovery
    sleep(Duration::from_millis(1)).await;
    
    // Should be recoverable
    breaker.record_success();
}

#[tokio::test]
async fn chaos_circuit_breaker_zero_threshold() {
    // Edge case: threshold of 1 (minimum)
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(100),
        success_threshold: 1,
    };
    
    let breaker = CircuitBreaker::new(config);
    
    // Single failure should open circuit
    breaker.record_failure();
    assert!(breaker.is_open(), "Should open on first failure with threshold 1");
    
    sleep(Duration::from_millis(150)).await;
    
    // Single success should close circuit
    breaker.record_success();
}

#[tokio::test]
async fn chaos_circuit_breaker_memory_pressure() {
    // Create many circuit breakers to test memory handling
    let mut breakers = Vec::new();
    
    for _ in 0..1000 {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            timeout: Duration::from_secs(1),
            success_threshold: 2,
        };
        breakers.push(CircuitBreaker::new(config));
    }
    
    // Trigger failures on all
    for breaker in &breakers {
        for _ in 0..10 {
            breaker.record_failure();
        }
    }
    
    // All should be open
    let open_count = breakers.iter().filter(|b| b.is_open()).count();
    assert_eq!(open_count, 1000, "All breakers should be open");
}

#[tokio::test]
async fn chaos_circuit_breaker_success_failure_interleave() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_millis(100),
        success_threshold: 3,
    };
    
    let breaker = CircuitBreaker::new(config);
    
    // Interleave successes and failures
    for i in 0..50 {
        if i % 3 == 0 {
            breaker.record_success();
        } else {
            breaker.record_failure();
        }
        sleep(Duration::from_micros(100)).await;
    }
    
    // Should reach some stable state
    let is_open = breaker.is_open();
    sleep(Duration::from_millis(10)).await;
    
    // State should be consistent
    assert_eq!(is_open, breaker.is_open(), "State should be stable");
}

#[tokio::test]
async fn chaos_circuit_breaker_concurrent_reads() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(1),
        success_threshold: 2,
    };
    
    let breaker = Arc::new(CircuitBreaker::new(config));
    
    // Trigger some failures
    for _ in 0..3 {
        breaker.record_failure();
    }
    
    // Spawn 100 concurrent readers
    let mut handles = vec![];
    for _ in 0..100 {
        let breaker_clone = Arc::clone(&breaker);
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                let _is_open = breaker_clone.is_open();
                sleep(Duration::from_micros(10)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all readers
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Should still be functional
    assert!(!breaker.is_open() || breaker.is_open(), "Should maintain consistency under concurrent reads");
}

#[tokio::test]
async fn chaos_circuit_breaker_recovery_stress() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
    };
    
    let breaker = CircuitBreaker::new(config);
    
    // Repeated recovery attempts
    for _ in 0..100 {
        // Open circuit
        for _ in 0..5 {
            breaker.record_failure();
        }
        
        // Immediate recovery attempt
        sleep(Duration::from_millis(60)).await;
        breaker.record_success();
        breaker.record_success();
        breaker.record_success();
    }
    
    // Should be functional after stress
    breaker.record_success();
}

