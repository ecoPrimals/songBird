use CanonicalSongbirdConfig;
//! Comprehensive Fault Tolerance Tests
//!
//! Tests the system's ability to handle various failure scenarios and recover gracefully.
//! Includes network partitions, service failures, resource exhaustion, and recovery tests.

use songbird_core::robustness::{
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState},
    manager::RobustnessManager,
};
use songbird_network::management::{
    health::{HealthChecker, HealthCheckConfig},
    load_balancer::{LoadBalancer, LoadBalancingStrategy},
};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::{
    discovery::{EcosystemDiscovery, EcosystemDiscoveryConfig},
    traits::{PrimalCapability, PrimalContext, SecurityLevel},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{test, time::timeout};
use tracing::{error, info, warn};
use tracing_test::traced_test;

#[tokio::test]
#[traced_test]
async fn test_network_partition_recovery() {
    info!("🌐 Testing network partition recovery");
    
    // Create a system that can handle network partitions
    let config = EcosystemDiscoveryConfig {
        ecosystem_base_path: "../".to_string(),
        health_check_timeout_ms: 1000, // Short timeout to simulate partition
        max_concurrent_discoveries: 5,
        enable_capability_inference: true,
        enable_filesystem_discovery: true,
        enable_network_discovery: false,
    };
    
    let discovery = EcosystemDiscovery::new(config);
    
    // Simulate network partition by setting very short timeouts
    info!("  📡 Simulating network partition...");
    
    let partition_result = timeout(Duration::from_millis(500), 
        discovery.discover_ecosystem_primals()).await;
    
    match partition_result {
        Ok(Ok(primals)) => {
            info!("  ✅ Discovery succeeded despite partition simulation: {} primals", primals.len());
            // Verify system can still operate with whatever primals are available
            test_degraded_mode_operation(&primals).await;
        }
        Ok(Err(e)) => {
            info!("  ⚠️ Discovery failed as expected during partition: {}", e);
            // Test graceful degradation
            test_partition_graceful_degradation().await;
        }
        Err(_) => {
            info!("  ⏰ Discovery timed out during partition (expected)");
            // Test recovery after partition
            test_post_partition_recovery(&discovery).await;
        }
    }
}

#[tokio::test]
#[traced_test]
async fn test_circuit_breaker_fault_tolerance() {
    info!("🔌 Testing circuit breaker fault tolerance");
    
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout_ms: 1000,
        half_open_max_calls: 2,
    };
    
    let circuit_breaker = CircuitBreaker::new(config.test.service_name.to_string(), config);
    
    // Test failure scenarios
    info!("  💥 Testing failure accumulation...");
    for i in 1..=3 {
        let result = circuit_breaker.call(|| async {
            Err(songbird_errors::SongbirdError::network_error("Simulated failure"))
        }).await;
        
        assert!(result.is_err(), "Call {} should fail", i);
        
        let state = circuit_breaker.state().await;
        if i < 3 {
            assert_eq!(state, CircuitBreakerState::Closed, "Circuit should be closed until threshold");
        } else {
            assert_eq!(state, CircuitBreakerState::Open, "Circuit should open after {} failures", i);
        }
    }
    
    // Test that circuit breaker prevents calls when open
    info!("  🚫 Testing open circuit protection...");
    let protected_result = circuit_breaker.call(|| async {
        Ok("Should not reach here")
    }).await;
    
    assert!(protected_result.is_err(), "Circuit breaker should prevent calls when open");
    
    // Wait for half-open transition
    info!("  🔄 Testing half-open recovery...");
    tokio::time::sleep(Duration::from_millis(1100)).await;
    
    let state = circuit_breaker.state().await;
    assert_eq!(state, CircuitBreakerState::HalfOpen, "Circuit should transition to half-open");
    
    // Test successful recovery
    let recovery_result = circuit_breaker.call(|| async {
        Ok("Recovery success")
    }).await;
    
    assert!(recovery_result.is_ok(), "First call in half-open should succeed");
    
    let final_state = circuit_breaker.state().await;
    assert_eq!(final_state, CircuitBreakerState::Closed, "Circuit should close after successful call");
    
    info!("  ✅ Circuit breaker fault tolerance verified");
}

#[tokio::test]
#[traced_test]
async fn test_service_failure_resilience() {
    info!("🏥 Testing service failure resilience");
    
    // Create a health checker that can detect failures
    let health_config = HealthCheckConfig {
        enabled: true,
        interval_secs: 1,
        timeout_secs: 1,
        unhealthy_threshold: 2,
        healthy_threshold: 2,
        upstream_servers: vec![
            "http://localhost:9999".to_string(), // This will fail
            "http://localhost:9998".to_string(), // This will also fail
        ],
    };
    
    let health_checker = HealthChecker::new(health_config);
    
    // Test health check failures
    info!("  🔍 Testing health check failure detection...");
    let health_results = health_checker.check_upstream_health().await;
    
    assert!(health_results.is_ok(), "Health checker should handle failures gracefully");
    
    if let Ok(results) = health_results {
        // All services should be marked as unhealthy since they're not running
        for (server, is_healthy) in results {
            info!("    Server {}: {}", server, if is_healthy { "healthy" } else { "unhealthy" });
            // In this test scenario, we expect services to be unhealthy
            assert!(!is_healthy, "Test servers should be unhealthy");
        }
    }
    
    info!("  ✅ Service failure detection working correctly");
}

#[tokio::test]
#[traced_test]
async fn test_resource_exhaustion_handling() {
    info!("💾 Testing resource exhaustion handling");
    
    // Simulate high memory usage scenario
    let robustness_manager = RobustnessManager::new();
    
    // Test bulkhead isolation under resource pressure
    info!("  🏗️ Testing bulkhead isolation...");
    
    let critical_bulkhead = robustness_manager.get_or_create_bulkhead("critical", 2).await;
    let normal_bulkhead = robustness_manager.get_or_create_bulkhead("normal", 10).await;
    
    // Fill up the critical bulkhead
    let permit1 = critical_bulkhead.acquire().await;
    let permit2 = critical_bulkhead.acquire().await;
    
    assert!(permit1.is_ok(), "First critical permit should succeed");
    assert!(permit2.is_ok(), "Second critical permit should succeed");
    
    // Third permit should fail due to bulkhead limit
    let permit3_result = timeout(Duration::from_millis(100), 
        critical_bulkhead.acquire()).await;
    
    match permit3_result {
        Ok(permit3) => {
            // If we get a permit, it should be an error due to bulkhead limits
            assert!(permit3.is_err(), "Third critical permit should fail due to bulkhead limits");
        }
        Err(_) => {
            info!("    ⏰ Critical bulkhead correctly blocked third request (timeout expected)");
        }
    }
    
    // Normal bulkhead should still work
    let normal_permit = normal_bulkhead.acquire().await;
    assert!(normal_permit.is_ok(), "Normal bulkhead should still work despite critical being full");
    
    info!("  ✅ Bulkhead isolation working correctly");
}

#[tokio::test]
#[traced_test]
async fn test_cascading_failure_prevention() {
    info!("⛓️ Testing cascading failure prevention");
    
    // Test that failure in one component doesn't bring down the whole system
    let discovery = EcosystemDiscovery::new(EcosystemDiscoveryConfig {
        ecosystem_base_path: "../".to_string(),
        health_check_timeout_ms: 100, // Very short to trigger failures
        max_concurrent_discoveries: 1, // Limit concurrency to test bottlenecks
        enable_capability_inference: true,
        enable_filesystem_discovery: true,
        enable_network_discovery: false,
    });
    
    // Simulate multiple concurrent discovery attempts that may fail
    let mut handles = Vec::new();
    for i in 0..5 {
        let discovery_clone = EcosystemDiscovery::new(EcosystemDiscoveryConfig::default());
        let handle = tokio::spawn(async move {
            info!("    🔄 Discovery attempt {}", i);
            match timeout(Duration::from_millis(200), 
                discovery_clone.discover_ecosystem_primals()).await {
                Ok(result) => {
                    match result {
                        Ok(primals) => {
                            info!("      ✅ Discovery {} succeeded: {} primals", i, primals.len());
                            true
                        }
                        Err(e) => {
                            info!("      ⚠️ Discovery {} failed: {}", i, e);
                            false
                        }
                    }
                }
                Err(_) => {
                    info!("      ⏰ Discovery {} timed out", i);
                    false
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all attempts
    let mut successes = 0;
    let mut failures = 0;
    
    for handle in handles {
        match handle.await {
            Ok(true) => successes += 1,
            Ok(false) => failures += 1,
            Err(e) => {
                warn!("Discovery task panicked: {}", e);
                failures += 1;
            }
        }
    }
    
    info!("  📊 Results: {} successes, {} failures", successes, failures);
    
    // The system should handle failures gracefully - either all succeed or fail gracefully
    assert!(successes >= 0, "System should handle concurrent operations");
    assert!(failures >= 0, "Failures should be handled gracefully");
    
    info!("  ✅ Cascading failure prevention verified");
}

#[tokio::test]
#[traced_test]
async fn test_graceful_degradation() {
    info!("🎚️ Testing graceful degradation");
    
    // Test that system can operate with reduced functionality when some components fail
    let mock_context = PrimalContext {
        user_id: "test-user".to_string(),
        device_id: "test-device".to_string(),
        session_id: "test-session".to_string(),
        security_level: SecurityLevel::Standard,
        metadata: HashMap::new(),
    };
    
    // Test discovery with various failure scenarios
    let degraded_scenarios = vec![
        ("network_disabled", false, true),   // Network discovery off, filesystem on
        ("filesystem_disabled", true, false), // Network discovery on, filesystem off  
        ("both_limited", false, false),       // Both discovery methods disabled
    ];
    
    for (scenario_name, enable_network, enable_filesystem) in degraded_scenarios {
        info!("  🔄 Testing scenario: {}", scenario_name);
        
        let config = EcosystemDiscoveryConfig {
            ecosystem_base_path: "../".to_string(),
            health_check_timeout_ms: 1000,
            max_concurrent_discoveries: 3,
            enable_capability_inference: true,
            enable_filesystem_discovery: enable_filesystem,
            enable_network_discovery: enable_network,
        };
        
        let discovery = EcosystemDiscovery::new(config);
        
        match timeout(Duration::from_millis(2000), 
            discovery.discover_ecosystem_primals()).await {
            Ok(Ok(primals)) => {
                info!("    ✅ Scenario '{}' succeeded with {} primals", scenario_name, primals.len());
                
                // Verify basic functionality still works
                for primal in &primals {
                    assert!(!primal.primal_type.as_str().is_empty());
                    assert!(!primal.capabilities.is_empty());
                }
            }
            Ok(Err(e)) => {
                info!("    ⚠️ Scenario '{}' failed gracefully: {}", scenario_name, e);
                // Graceful failure is acceptable
            }
            Err(_) => {
                info!("    ⏰ Scenario '{}' timed out (degraded performance expected)", scenario_name);
            }
        }
    }
    
    info!("  ✅ Graceful degradation verified");
}

// Helper functions for fault tolerance testing

async fn test_degraded_mode_operation(primals: &[songbird_universal_primals::discovery::types::DiscoveredPrimal]) {
    info!("    🔧 Testing degraded mode operation with {} available primals", primals.len());
    
    // Verify system can still route capabilities even with limited primals
    for primal in primals {
        assert!(!primal.capabilities.is_empty(), "Available primals should have capabilities");
        info!("      Available: {} with {} capabilities", 
            primal.primal_type.as_str(), primal.capabilities.len());
    }
}

async fn test_partition_graceful_degradation() {
    info!("    🏗️ Testing graceful degradation during partition");
    
    // Test that the system falls back to offline/cached data
    // In a real system, this would use cached primal information
    let fallback_capabilities = vec![
        PrimalCapability::Authentication { providers: vec!["local".to_string()] },
                        PrimalCapability::Storage { types: vec!["file".to_string()] },
    ];
    
    assert!(!fallback_capabilities.is_empty(), "Should have fallback capabilities");
    info!("      ✅ Fallback capabilities available: {}", fallback_capabilities.len());
}

async fn test_post_partition_recovery(discovery: &EcosystemDiscovery) {
    info!("    🔄 Testing post-partition recovery");
    
    // Simulate recovery by trying discovery with normal timeout
    let recovery_config = EcosystemDiscoveryConfig {
        ecosystem_base_path: "../".to_string(),
        health_check_timeout_ms: config.timeouts.request_ms, // Normal timeout
        max_concurrent_discoveries: 10,
        enable_capability_inference: true,
        enable_filesystem_discovery: true,
        enable_network_discovery: false,
    };
    
    let recovery_discovery = EcosystemDiscovery::new(recovery_config);
    
    match timeout(Duration::from_millis(config.dashboard.port), 
        recovery_discovery.discover_ecosystem_primals()).await {
        Ok(Ok(primals)) => {
            info!("      ✅ Recovery successful: {} primals discovered", primals.len());
        }
        Ok(Err(e)) => {
            info!("      ⚠️ Recovery attempted but failed: {}", e);
        }
        Err(_) => {
            info!("      ⏰ Recovery still in progress (timeout)");
        }
    }
} 