//! Circuit breaker concurrency and performance tests

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use songbird_network::communication::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};
use super::basic_tests::create_fast_circuit_breaker;

#[tokio::test]
async fn test_concurrent_failure_recording() {
    let circuit_breaker = Arc::new(create_fast_circuit_breaker());
    let mut handles = vec![];

    // Spawn multiple tasks to record failures concurrently
    for _ in 0..10 {
        let cb = circuit_breaker.clone();
        let handle = tokio::spawn(async move {
            cb.record_failure().await;
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.expect("Test operation should succeed");
    }

    // Circuit should be open
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
}

#[tokio::test]
async fn test_concurrent_success_recording() {
    let circuit_breaker = Arc::new(create_fast_circuit_breaker());

    // Force to half-open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }
    sleep(Duration::from_millis(150)).await;
    circuit_breaker.should_allow_request().await;

    let mut handles = vec![];

    // Record successes concurrently
    for _ in 0..5 {
        let cb = circuit_breaker.clone();
        let handle = tokio::spawn(async move {
            cb.record_success().await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Test operation should succeed");
    }

    // Should be closed
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));
}

#[tokio::test]
async fn test_high_load_performance() {
    let circuit_breaker = Arc::new(create_fast_circuit_breaker());
    let start = std::time::Instant::now();

    let mut handles = vec![];
    for i in 0..1000 {
        let cb = circuit_breaker.clone();
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                cb.record_success().await;
            } else {
                cb.record_failure().await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Test operation should succeed");
    }

    let duration = start.elapsed();
    println!("High load test completed in: {:?}", duration);
    
    // Should complete in reasonable time
    assert!(duration < Duration::from_secs(5));
}

#[tokio::test]
async fn test_concurrent_state_checks() {
    let circuit_breaker = Arc::new(create_fast_circuit_breaker());
    let mut handles = vec![];

    // Concurrent state checks shouldn't panic
    for _ in 0..50 {
        let cb = circuit_breaker.clone();
        let handle = tokio::spawn(async move {
            let _state = cb.get_state().await;
            let _should_allow = cb.should_allow_request().await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Test operation should succeed");
    }
} 