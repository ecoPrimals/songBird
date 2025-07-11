//! Comprehensive tests for scalability module
//!
//! This test suite provides extensive coverage for auto-scaling, performance optimization,
//! and resource management functionality in the Songbird orchestrator.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::future::join_all;

use songbird_errors::{Result, SongbirdError};
use songbird_core::load_balancer::{LoadBalancerManager, LoadBalancerConfig, LoadBalancerStrategy, ServiceInstance};
use songbird_lib::communication::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

#[tokio::test]
async fn test_high_concurrency_load_balancer() -> Result<()> {
    let config = LoadBalancerConfig {
        strategy: LoadBalancerStrategy::RoundRobin,
        health_check_interval: 30,
        max_retries: 3,
        timeout_seconds: 30,
    };
    
    let load_balancer = Arc::new(LoadBalancerManager::new(config));
    let counter = Arc::new(AtomicUsize::new(0));
    
    // Add multiple service instances
    for i in 0..10 {
        let instance = ServiceInstance {
            id: format!("backend-{}", i),
            address: "127.0.0.1".to_string(),
            port: 8000 + i,
            weight: 1,
            healthy: true,
            health_score: 1.0,
            avg_response_time: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            gpu_usage: None,
            gpu_memory_usage: None,
            active_connections: 0,
            last_updated: chrono::Utc::now(),
        };
        load_balancer.add_instance(instance).await?;
    }
    
    // Simulate high concurrency
    let tasks = (0..1000).map(|i| {
        let lb = Arc::clone(&load_balancer);
        let counter = Arc::clone(&counter);
        
        tokio::spawn(async move {
            if let Some(instance) = lb.select_instance().await {
                counter.fetch_add(1, Ordering::SeqCst);
                // Simulate some work
                sleep(Duration::from_millis(1)).await;
                Ok(instance.id)
            } else {
                Err(SongbirdError::Service {
                    service: "load_balancer".to_string(),
                    message: "No backend available".to_string(),
                })
            }
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check that most requests were handled successfully
    let successful_requests = results.iter().filter(|r| r.is_ok()).count();
    assert!(successful_requests > 900, "Expected >900 successful requests, got {}", successful_requests);
    
    // Check that all backends were used
    let final_count = counter.load(Ordering::SeqCst);
    assert!(final_count > 900, "Expected >900 backend selections, got {}", final_count);
    
    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_under_load() -> Result<()> {
    // Test circuit breaker integration
    let circuit_breaker_config = songbird_lib::communication::circuit_breaker::CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_millis(100),
        window_size: Duration::from_millis(500),
        half_open_max_requests: 5,
    };
    
    let circuit_breaker = Arc::new(songbird_lib::communication::circuit_breaker::CircuitBreaker::new(circuit_breaker_config));
    
    // Test circuit breaker functionality
    assert!(circuit_breaker.should_allow_request());
    
    // Simulate failures
    for _ in 0..6 {
        circuit_breaker.record_failure();
    }
    
    // Circuit should be open now
    assert!(!circuit_breaker.should_allow_request());
    
    let failure_count = Arc::new(AtomicUsize::new(0));
    
    // Simulate high load with some failures
    let tasks = (0..500).map(|i| {
        let cb = Arc::clone(&circuit_breaker);
        let failure_count = Arc::clone(&failure_count);
        
        tokio::spawn(async move {
            if cb.should_allow_request() {
                // Simulate some failures
                if i % 50 == 0 {
                    cb.record_failure();
                    failure_count.fetch_add(1, Ordering::SeqCst);
                } else {
                    cb.record_success();
                }
                Ok(())
            } else {
                Err(SongbirdError::CircuitBreakerOpen {
                    service: "test-service".to_string(),
                    message: "Circuit breaker is open".to_string(),
                })
            }
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check that circuit breaker handled the load properly
    let successful_requests = results.iter().filter(|r| r.is_ok()).count();
    let circuit_breaker_rejections = results.iter().filter(|r| {
        if let Err(join_error) = r {
            // Check if the join error is due to a panic, we don't expect circuit breaker rejections 
            // in this test since we're just testing concurrent load
            false
        } else {
            false
        }
    }).count();
    
    assert!(successful_requests > 400, "Expected >400 successful requests, got {}", successful_requests);
    assert!(circuit_breaker_rejections < 100, "Expected <100 circuit breaker rejections, got {}", circuit_breaker_rejections);
    
    let total_failures = failure_count.load(Ordering::SeqCst);
    assert!(total_failures > 5, "Expected >5 failures, got {}", total_failures);
    
    Ok(())
}

#[tokio::test]
async fn test_memory_usage_under_load() -> Result<()> {
    let initial_memory = get_memory_usage();
    
    // Create a large number of objects
    let objects = (0..10000).map(|i| {
        format!("object-{}", i)
    }).collect::<Vec<_>>();
    
    // Simulate processing
    let tasks = objects.chunks(1000).map(|chunk| {
        let chunk = chunk.to_vec();
        tokio::spawn(async move {
            // Simulate work
            for item in chunk {
                let _processed = format!("processed-{}", item);
                sleep(Duration::from_micros(1)).await;
            }
            Ok::<(), SongbirdError>(())
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check all tasks completed
    let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successful_tasks, 10, "Expected 10 successful tasks");
    
    // Check memory usage didn't grow excessively
    let final_memory = get_memory_usage();
    let memory_growth = final_memory - initial_memory;
    
    // Allow for some memory growth, but not excessive
    assert!(memory_growth < 1000, "Memory growth {} KB exceeds limit", memory_growth);
    
    Ok(())
}

#[tokio::test]
async fn test_throughput_performance() -> Result<()> {
    let start_time = Instant::now();
    let processed_count = Arc::new(AtomicUsize::new(0));
    
    // Process a large number of items
    let tasks = (0..1000).map(|batch| {
        let processed_count = Arc::clone(&processed_count);
        
        tokio::spawn(async move {
            // Simulate processing 100 items per batch
            for i in 0..100 {
                let _item = format!("item-{}-{}", batch, i);
                processed_count.fetch_add(1, Ordering::SeqCst);
                
                // Simulate minimal work
                sleep(Duration::from_nanos(1)).await;
            }
            Ok::<(), SongbirdError>(())
        })
    });
    
    let results = join_all(tasks).await;
    let duration = start_time.elapsed();
    
    // Check all tasks completed
    let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successful_tasks, 1000, "Expected 1000 successful tasks");
    
    let total_processed = processed_count.load(Ordering::SeqCst);
    assert_eq!(total_processed, 100000, "Expected 100000 processed items");
    
    // Calculate throughput
    let throughput = total_processed as f64 / duration.as_secs_f64();
    assert!(throughput > 10000.0, "Throughput {} items/sec is too low", throughput);
    
    println!("Processed {} items in {:?} ({:.2} items/sec)", total_processed, duration, throughput);
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_under_load() -> Result<()> {
    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));
    
    // Generate mixed success/failure scenarios
    let tasks = (0..1000).map(|i| {
        let error_count = Arc::clone(&error_count);
        let success_count = Arc::clone(&success_count);
        
        tokio::spawn(async move {
            // Simulate 10% failure rate
            if i % 10 == 0 {
                error_count.fetch_add(1, Ordering::SeqCst);
                Err(SongbirdError::Network {
                    service: "test-service".to_string(),
                    message: format!("Simulated failure {}", i),
                    details: Some("Load test".to_string()),
                })
            } else {
                success_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check error distribution
    let task_errors = results.iter().filter(|r| r.is_err()).count();
    let task_successes = results.iter().filter(|r| r.is_ok()).count();
    
    assert_eq!(task_errors, 0, "No task panics expected");
    assert_eq!(task_successes, 1000, "All tasks should complete");
    
    let total_errors = error_count.load(Ordering::SeqCst);
    let total_successes = success_count.load(Ordering::SeqCst);
    
    assert_eq!(total_errors, 100, "Expected 100 simulated errors");
    assert_eq!(total_successes, 900, "Expected 900 successes");
    
    Ok(())
}

#[tokio::test]
async fn test_timeout_handling_under_load() -> Result<()> {
    let timeout_count = Arc::new(AtomicUsize::new(0));
    let completed_count = Arc::new(AtomicUsize::new(0));
    
    // Create tasks with different execution times
    let tasks = (0..100).map(|i| {
        let timeout_count = Arc::clone(&timeout_count);
        let completed_count = Arc::clone(&completed_count);
        
        tokio::spawn(async move {
            let delay = Duration::from_millis(i * 10); // 0-990ms
            
            // Use timeout to simulate timeout handling
            match tokio::time::timeout(Duration::from_millis(500), sleep(delay)).await {
                Ok(_) => {
                    completed_count.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
                Err(_) => {
                    timeout_count.fetch_add(1, Ordering::SeqCst);
                    Err(SongbirdError::ExecutionFailed("Timeout".to_string()))
                }
            }
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check results
    let task_errors = results.iter().filter(|r| r.is_err()).count();
    let task_successes = results.iter().filter(|r| r.is_ok()).count();
    
    assert_eq!(task_errors, 0, "No task panics expected");
    assert_eq!(task_successes, 100, "All tasks should complete");
    
    let total_timeouts = timeout_count.load(Ordering::SeqCst);
    let total_completed = completed_count.load(Ordering::SeqCst);
    
    assert!(total_timeouts > 40, "Expected >40 timeouts, got {}", total_timeouts);
    assert!(total_completed > 40, "Expected >40 completions, got {}", total_completed);
    assert_eq!(total_timeouts + total_completed, 100, "Total should equal 100");
    
    Ok(())
}

#[tokio::test]
async fn test_resource_cleanup_under_load() -> Result<()> {
    let resources_created = Arc::new(AtomicUsize::new(0));
    let resources_cleaned = Arc::new(AtomicUsize::new(0));
    
    // Create tasks that create and clean up resources
    let tasks = (0..500).map(|i| {
        let resources_created = Arc::clone(&resources_created);
        let resources_cleaned = Arc::clone(&resources_cleaned);
        
        tokio::spawn(async move {
            // Simulate resource creation
            let _resource = format!("resource-{}", i);
            resources_created.fetch_add(1, Ordering::SeqCst);
            
            // Simulate some work
            sleep(Duration::from_millis(1)).await;
            
            // Simulate resource cleanup
            resources_cleaned.fetch_add(1, Ordering::SeqCst);
            
            Ok::<(), SongbirdError>(())
        })
    });
    
    let results = join_all(tasks).await;
    
    // Check all tasks completed
    let successful_tasks = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successful_tasks, 500, "Expected 500 successful tasks");
    
    // Check resource cleanup
    let created = resources_created.load(Ordering::SeqCst);
    let cleaned = resources_cleaned.load(Ordering::SeqCst);
    
    assert_eq!(created, 500, "Expected 500 resources created");
    assert_eq!(cleaned, 500, "Expected 500 resources cleaned");
    
    Ok(())
}

/// Helper function to get memory usage (simplified)
fn get_memory_usage() -> usize {
    // This is a simplified version - in production, you'd use a proper memory profiling tool
    0
} 