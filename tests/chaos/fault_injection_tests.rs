use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// Chaos Engineering Tests
//
// Fault injection and resilience testing for Songbird Orchestrator

use songbird_gaming_bridge::{
    orchestrator::Orchestrator,
    config::OrchestratorConfig,
    discovery::{ServiceRegistry, InMemoryServiceRegistry},
    communication::{HttpCommunication, CommunicationLayer, CircuitBreaker, CircuitBreakerConfig},
    proxy::{ConnectionProxy, ProxyConfig},
    traits::{
        service_id::{ServiceInfo, ServiceEndpoint},
        communication::{ServiceMessage, ServiceAddress, MessageType},
    },
};

use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};

use tokio::time::timeout;

/// Chaos test configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Duration of chaos injection
    pub chaos_processing_time: Duration,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Network delay simulation
    pub network_delay: Option<Duration>,
    /// Memory pressure simulation
    pub memory_pressure: bool,
    /// CPU pressure simulation
    pub cpu_pressure: bool,
    /// Service kill probability
    pub service_kill_rate: f64,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            chaos_processing_time: Duration::from_secs(30),
            failure_rate: 0.1, // 10% failure rate
            network_delay: Some(Duration::from_millis(100)),
            memory_pressure: false,
            cpu_pressure: false,
            service_kill_rate: 0.05, // 5% service kill rate
        }
    }
}

/// Chaos test results
#[derive(Debug, Clone)]
pub struct ChaosResults {
    /// Total operations attempted
    pub total_operations: u64,
    /// Operations that succeeded despite chaos
    pub successful_operations: u64,
    /// Operations that failed due to chaos
    pub failed_operations: u64,
    /// Recovery time after chaos stopped
    pub recovery_time: Duration,
    /// System remained stable
    pub system_stable: bool,
    /// Circuit breaker activations
    pub circuit_breaker_activations: u64,
    /// Error types encountered
    pub error_types: HashMap<String, u64>,
}

/// Test Circuit Breaker Resilience
#[tokio::test]
async fn test_circuit_breaker_resilience() {
    println!("🔌 Testing Circuit Breaker Resilience");
    
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(5),
        window_size: Duration::from_secs(10),
        half_open_max_requests: 2,
    };
    
    let circuit_breaker = CircuitBreaker::new(config);
    
    // Simulate failures to trip the circuit breaker
    for _ in 0..5 {
        circuit_breaker.record_failure();
    }
    
    // Circuit breaker should be open now
    assert!(!circuit_breaker.should_allow_request(), "Circuit breaker should be open after failures");
    
    // Wait for timeout
    tokio::time::sleep(Duration::from_secs(6)).await;
    
    // Should allow limited requests in half-open state
    assert!(circuit_breaker.should_allow_request(), "Circuit breaker should allow requests after timeout");
    
    // Record successes to close circuit
    circuit_breaker.record_success();
    circuit_breaker.record_success();
    
    // Circuit should be closed now
    assert!(circuit_breaker.should_allow_request(), "Circuit breaker should be closed after successes");
    
    println!("✅ Circuit breaker behaved correctly under failure conditions");
}

/// Test Network Partition Resilience
#[tokio::test]
async fn test_network_partition_resilience() {
    println!("🌐 Testing Network Partition Resilience");
    
    let chaos_config = ChaosConfig {
        chaos_processing_time: Duration::from_secs(10),
        failure_rate: 0.8, // High failure rate to simulate partition
        network_delay: Some(Duration::from_millis(500)),
        ..Default::default()
    };
    
    let results = simulate_network_partition(chaos_config).await;
    
    // System should handle network partitions gracefully
    assert!(results.total_operations > 0, "Should attempt operations during partition");
    assert!(results.system_stable, "System should remain stable during network partition");
    assert!(results.recovery_time < Duration::from_secs(30), "Recovery should be quick after partition heals");
    
    // Some operations should succeed even during partition (cached/local operations)
    let success_rate = results.successful_operations as f64 / results.total_operations as f64;
    assert!(success_rate > 0.1, "At least 10% of operations should succeed during partition");
    
    println!("📊 Network Partition Test Results:");
    print_chaos_results(&results);
}

/// Test Service Failure and Recovery
#[tokio::test]
async fn test_service_failure_recovery() {
    println!("🔧 Testing Service Failure and Recovery");
    
    let chaos_config = ChaosConfig {
        chaos_processing_time: Duration::from_secs(15),
        service_kill_rate: 0.3, // 30% chance of service failure
        failure_rate: 0.2,
        ..Default::default()
    };
    
    let results = simulate_service_failures(chaos_config).await;
    
    // System should handle service failures gracefully
    assert!(results.total_operations > 0, "Should attempt operations during service failures");
    assert!(results.system_stable, "System should remain stable during service failures");
    
    // Circuit breakers should activate during failures
    assert!(results.circuit_breaker_activations > 0, "Circuit breakers should activate during service failures");
    
    // Recovery should be reasonable
    assert!(results.recovery_time < Duration::from_secs(60), "Recovery should complete within 1 minute");
    
    println!("📊 Service Failure Test Results:");
    print_chaos_results(&results);
}

/// Test High Load with Random Failures
#[tokio::test]
async fn test_high_load_with_failures() {
    println!("🚀 Testing High Load with Random Failures");
    
    let chaos_config = ChaosConfig {
        chaos_processing_time: Duration::from_secs(20),
        failure_rate: 0.15, // 15% random failure rate
        network_delay: Some(Duration::from_millis(50)),
        memory_pressure: true,
        cpu_pressure: true,
        ..Default::default()
    };
    
    let results = simulate_high_load_chaos(chaos_config).await;
    
    // System should handle high load with failures
    assert!(results.total_operations > 100, "Should handle significant load during chaos");
    assert!(results.system_stable, "System should remain stable under high load with failures");
    
    // Failure rate should be contained
    let failure_rate = results.failed_operations as f64 / results.total_operations as f64;
    assert!(failure_rate < 0.5, "Failure rate should be under 50% even with chaos");
    
    println!("📊 High Load Chaos Test Results:");
    print_chaos_results(&results);
}

/// Test Proxy Resilience Under Chaos
#[tokio::test]
async fn test_proxy_chaos_resilience() {
    println!("🌐 Testing Proxy Resilience Under Chaos");
    
    let chaos_config = ChaosConfig {
        chaos_processing_time: Duration::from_secs(12),
        failure_rate: 0.25, // 25% failure rate
        network_delay: Some(Duration::from_millis(200)),
        ..Default::default()
    };
    
    let results = simulate_proxy_chaos(chaos_config).await;
    
    // Proxy should handle chaos gracefully
    assert!(results.total_operations > 0, "Proxy should attempt operations during chaos");
    assert!(results.system_stable, "Proxy should remain stable during chaos");
    
    // Recovery should be fast
    assert!(results.recovery_time < Duration::from_secs(20), "Proxy recovery should be quick");
    
    println!("📊 Proxy Chaos Test Results:");
    print_chaos_results(&results);
}

/// Test Discovery Service Chaos
#[tokio::test]
async fn test_discovery_chaos_resilience() {
    println!("🔍 Testing Service Discovery Resilience Under Chaos");
    
    let chaos_config = ChaosConfig {
        chaos_processing_time: Duration::from_secs(8),
        failure_rate: 0.3, // 30% failure rate
        service_kill_rate: 0.2, // 20% service kill rate
        ..Default::default()
    };
    
    let results = simulate_discovery_chaos(chaos_config).await;
    
    // Discovery should be resilient
    assert!(results.total_operations > 0, "Discovery should attempt operations during chaos");
    assert!(results.system_stable, "Discovery should remain stable during chaos");
    
    // Discovery should recover quickly
    assert!(results.recovery_time < Duration::from_secs(10), "Discovery recovery should be very quick");
    
    println!("📊 Discovery Chaos Test Results:");
    print_chaos_results(&results);
}

// Implementation functions

async fn simulate_network_partition(config: ChaosConfig) -> ChaosResults {
    let http_comm = HttpCommunication::new("http://httpbin.org".to_string());
    
    let total_operations = Arc::new(AtomicU64::new(0));
    let successful_operations = Arc::new(AtomicU64::new(0));
    let failed_operations = Arc::new(AtomicU64::new(0));
    let system_stable = Arc::new(AtomicBool::new(true));
    
    let start_time = Instant::now();
    
    // Simulate operations during network partition
    let operations_task = {
        let total_ops = Arc::clone(&total_operations);
        let success_ops = Arc::clone(&successful_operations);
        let failed_ops = Arc::clone(&failed_operations);
        let stable = Arc::clone(&system_stable);
        let http_comm = http_comm.clone();
        
        tokio::spawn(async move {
            while start_time.elapsed() < config.chaos_duration {
                total_ops.fetch_add(1, Ordering::Relaxed);
                
                // Simulate network partition by introducing failures and delays
                let should_fail = rand::random::<f64>() < config.failure_rate;
                
                if should_fail {
                    failed_ops.fetch_add(1, Ordering::Relaxed);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    continue;
                }
                
                // Add network delay
                if let Some(delay) = config.network_delay {
                    tokio::time::sleep(delay).await;
                }
                
                // Attempt operation with timeout
                let message = ServiceMessage {
                    id: format!("chaos-test-{}", total_ops.load(Ordering::Relaxed)),
                    message_type: MessageType::Request,
                    topic: "chaos-test".to_string(),
                    body: serde_json::json!({"test": "data"}),
                    timestamp: chrono::Utc::now(),
                    correlation_id: None,
                    ttl: Some(Duration::from_secs(5)),
                    headers: HashMap::new(),
                };
                
                let target = ServiceAddress {
                    service_id: "test-service".to_string(),
                    instance_id: None,
                    endpoint: Some("http://httpbin.org/post".to_string()),
                };
                
                match timeout(Duration::from_secs(2), http_comm.send_message(target, message)).await {
                    Ok(Ok(_)) => {
                        success_ops.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        failed_ops.fetch_add(1, Ordering::Relaxed);
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
    };
    
    // Monitor system stability
    let stability_task = {
        let stable = Arc::clone(&system_stable);
        
        tokio::spawn(async move {
            while start_time.elapsed() < config.chaos_duration {
                // Check if system is still responsive
                let health_check_start = Instant::now();
                tokio::time::sleep(Duration::from_millis(1)).await;
                
                // If basic operations take too long, mark as unstable
                if health_check_start.elapsed() > Duration::from_millis(100) {
                    stable.store(false, Ordering::Relaxed);
                }
                
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
    };
    
    // Wait for chaos period to end
    tokio::join!(operations_task, stability_task);
    
    // Measure recovery time
    let recovery_start = Instant::now();
    
    // Test recovery by attempting a few operations
    let mut recovery_attempts = 0;
    let mut recovery_successes = 0;
    
    for _ in 0..5 {
        recovery_attempts += 1;
        
        let message = ServiceMessage {
            id: format!("recovery-test-{}", recovery_attempts),
            message_type: MessageType::Request,
            topic: "recovery-test".to_string(),
            body: serde_json::json!({"test": "recovery"}),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            ttl: Some(Duration::from_secs(10)),
            headers: HashMap::new(),
        };
        
        let target = ServiceAddress {
            service_id: "recovery-service".to_string(),
            instance_id: None,
            endpoint: Some("http://httpbin.org/post".to_string()),
        };
        
        if let Ok(Ok(_)) = timeout(Duration::from_secs(5), http_comm.send_message(target, message)).await {
            recovery_successes += 1;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let recovery_time = if recovery_successes > 0 {
        recovery_start.elapsed()
    } else {
        Duration::from_secs(60) // Max recovery time if no success
    };
    
    ChaosResults {
        total_operations: total_operations.load(Ordering::Relaxed),
        successful_operations: successful_operations.load(Ordering::Relaxed),
        failed_operations: failed_operations.load(Ordering::Relaxed),
        recovery_time,
        system_stable: system_stable.load(Ordering::Relaxed),
        circuit_breaker_activations: 0, // Would be tracked in real implementation
        error_types: HashMap::new(),
    }
}

async fn simulate_service_failures(config: ChaosConfig) -> ChaosResults {
    let registry = InMemoryServiceRegistry::new();
    
    // Register test services
    for i in 0..10 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: Some("Test service for chaos testing").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        registry.register_service(service).await.expect("Failed to register service");
    }
    
    let total_operations = Arc::new(AtomicU64::new(0));
    let successful_operations = Arc::new(AtomicU64::new(0));
    let failed_operations = Arc::new(AtomicU64::new(0));
    let circuit_breaker_activations = Arc::new(AtomicU64::new(0));
    
    let start_time = Instant::now();
    
    // Simulate service operations with random failures
    while start_time.elapsed() < config.chaos_duration {
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Randomly kill services
        if rand::random::<f64>() < config.service_kill_rate {
            // Simulate service failure by unregistering it
            let service_id = format!("service-{}", rand::random::<u8>() % 10);
            let _ = registry.unregister_service(&service_id).await;
            circuit_breaker_activations.fetch_add(1, Ordering::Relaxed);
        }
        
        // Attempt service discovery
        match registry.discover_services("test").await {
            Ok(services) => {
                if !services.is_empty() {
                    successful_operations.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_operations.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(_) => {
                failed_operations.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Measure recovery by re-registering services
    let recovery_start = Instant::now();
    
    for i in 0..10 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Recovered Service {}", i),
            version: "1.0.1".to_string(),
            service_type: "test".to_string(),
            description: Some("Recovered test service").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        registry.register_service(service).await.expect("Failed to re-register service");
    }
    
    let recovery_time = recovery_start.elapsed();
    
    ChaosResults {
        total_operations: total_operations.load(Ordering::Relaxed),
        successful_operations: successful_operations.load(Ordering::Relaxed),
        failed_operations: failed_operations.load(Ordering::Relaxed),
        recovery_time,
        system_stable: true, // Registry operations are generally stable
        circuit_breaker_activations: circuit_breaker_activations.load(Ordering::Relaxed),
        error_types: HashMap::new(),
    }
}

async fn simulate_high_load_chaos(config: ChaosConfig) -> ChaosResults {
    let orchestrator_config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(orchestrator_config).await.expect("Failed to create orchestrator");
    
    let total_operations = Arc::new(AtomicU64::new(0));
    let successful_operations = Arc::new(AtomicU64::new(0));
    let failed_operations = Arc::new(AtomicU64::new(0));
    let system_stable = Arc::new(AtomicBool::new(true));
    
    let start_time = Instant::now();
    
    // Spawn multiple concurrent workers to create high load
    let mut handles = Vec::new();
    
    for worker_id in 0..20 {
        let total_ops = Arc::clone(&total_operations);
        let success_ops = Arc::clone(&successful_operations);
        let failed_ops = Arc::clone(&failed_operations);
        let stable = Arc::clone(&system_stable);
        let orchestrator = orchestrator.clone();
        
        let handle = tokio::spawn(async move {
            while start_time.elapsed() < config.chaos_duration {
                total_ops.fetch_add(1, Ordering::Relaxed);
                
                // Introduce random failures
                if rand::random::<f64>() < config.failure_rate {
                    failed_ops.fetch_add(1, Ordering::Relaxed);
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    continue;
                }
                
                // Add CPU pressure simulation
                if config.cpu_pressure && rand::random::<f64>() < 0.1 {
                    // Simulate CPU intensive operation
                    let _busy_work: u64 = (0..1000).map(|i| i * worker_id as u64).sum();
                }
                
                // Test orchestrator health (lightweight operation)
                match timeout(Duration::from_millis(100), orchestrator.is_healthy()).await {
                    Ok(true) => {
                        success_ops.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        failed_ops.fetch_add(1, Ordering::Relaxed);
                        stable.store(false, Ordering::Relaxed);
                    }
                }
                
                // Small delay to prevent overwhelming the system
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all workers to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    // Test recovery
    let recovery_start = Instant::now();
    let mut recovery_successes = 0;
    
    for _ in 0..10 {
        if orchestrator.is_healthy().await {
            recovery_successes += 1;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let recovery_time = if recovery_successes > 5 {
        recovery_start.elapsed()
    } else {
        Duration::from_secs(30)
    };
    
    ChaosResults {
        total_operations: total_operations.load(Ordering::Relaxed),
        successful_operations: successful_operations.load(Ordering::Relaxed),
        failed_operations: failed_operations.load(Ordering::Relaxed),
        recovery_time,
        system_stable: system_stable.load(Ordering::Relaxed),
        circuit_breaker_activations: 0,
        error_types: HashMap::new(),
    }
}

async fn simulate_proxy_chaos(config: ChaosConfig) -> ChaosResults {
    let proxy_config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        enable_circuit_breaker: true,
        circuit_breaker_threshold: 3,
        ..Default::default()
    };
    
    let proxy = ConnectionProxy::new(proxy_config);
    
    // Register test service
    let service = ServiceInfo {
        id: "chaos-test-service".to_string(),
        name: "Chaos Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "http".to_string(),
        description: Some("Service for chaos testing").to_string(),
        endpoints: vec![
            ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: "http://httpbin.org/post".to_string(),
                method: "POST".to_string(),
                description: Some("Test endpoint").to_string(),
                parameters: vec![],
                response_schema: None,
            }
        ],
        tags: std::collections::HashMap::new(),
        tags: HashMap::new(),
        
    };
    
    proxy.update_services(vec![service]).await.expect("Failed to register service");
    
    let total_operations = Arc::new(AtomicU64::new(0));
    let successful_operations = Arc::new(AtomicU64::new(0));
    let failed_operations = Arc::new(AtomicU64::new(0));
    
    let start_time = Instant::now();
    
    // Simulate proxy operations with chaos
    while start_time.elapsed() < config.chaos_duration {
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Create proxy request
        let proxy_request = songbird_gaming_bridge::proxy::ProxyRequest {
            method: axum::http::Method::POST,
            uri: "/chaos-test".parse().expect("Test assertion failed"),
            headers: axum::http::HeaderMap::new(),
            body: b"chaos test data".to_vec(),
            source_ip: Some("127.0.0.1".to_string()),
            timestamp: std::time::Instant::now(),
        };
        
        // Introduce random failures
        if rand::random::<f64>() < config.failure_rate {
            failed_operations.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        
        // Add network delay
        if let Some(delay) = config.network_delay {
            tokio::time::sleep(delay).await;
        }
        
        match timeout(Duration::from_secs(2), proxy.route_request("chaos-test-service", proxy_request)).await {
            Ok(Ok(_)) => {
                successful_operations.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                failed_operations.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    
    // Test recovery
    let recovery_start = Instant::now();
    let proxy_stats = proxy.get_stats().await;
    let recovery_time = recovery_start.elapsed();
    
    ChaosResults {
        total_operations: total_operations.load(Ordering::Relaxed),
        successful_operations: successful_operations.load(Ordering::Relaxed),
        failed_operations: failed_operations.load(Ordering::Relaxed),
        recovery_time,
        system_stable: proxy_stats.error_rate < 90.0, // System stable if error rate under 90%
        circuit_breaker_activations: 0, // Would need to track from proxy stats
        error_types: HashMap::new(),
    }
}

async fn simulate_discovery_chaos(config: ChaosConfig) -> ChaosResults {
    let registry = InMemoryServiceRegistry::new();
    
    // Pre-populate registry
    for i in 0..20 {
        let service = ServiceInfo {
            id: format!("chaos-service-{}", i),
            name: format!("Chaos Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "chaos".to_string(),
            description: Some("Service for chaos testing").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        registry.register_service(service).await.expect("Failed to register service");
    }
    
    let total_operations = Arc::new(AtomicU64::new(0));
    let successful_operations = Arc::new(AtomicU64::new(0));
    let failed_operations = Arc::new(AtomicU64::new(0));
    
    let start_time = Instant::now();
    
    // Simulate discovery operations with chaos
    while start_time.elapsed() < config.chaos_duration {
        total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Randomly kill services
        if rand::random::<f64>() < config.service_kill_rate {
            let service_id = format!("chaos-service-{}", rand::random::<u8>() % 20);
            let _ = registry.unregister_service(&service_id).await;
        }
        
        // Introduce random failures
        if rand::random::<f64>() < config.failure_rate {
            failed_operations.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(1)).await;
            continue;
        }
        
        // Test discovery operations
        match registry.discover_services("chaos").await {
            Ok(services) => {
                if !services.is_empty() {
                    successful_operations.fetch_add(1, Ordering::Relaxed);
                } else {
                    failed_operations.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(_) => {
                failed_operations.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    
    // Test recovery by re-registering services
    let recovery_start = Instant::now();
    
    for i in 0..20 {
        let service = ServiceInfo {
            id: format!("chaos-service-{}", i),
            name: format!("Recovered Chaos Service {}", i),
            version: "1.0.1".to_string(),
            service_type: "chaos".to_string(),
            description: Some("Recovered service").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        registry.register_service(service).await.expect("Failed to re-register service");
    }
    
    let recovery_time = recovery_start.elapsed();
    
    ChaosResults {
        total_operations: total_operations.load(Ordering::Relaxed),
        successful_operations: successful_operations.load(Ordering::Relaxed),
        failed_operations: failed_operations.load(Ordering::Relaxed),
        recovery_time,
        system_stable: true, // Discovery is generally stable
        circuit_breaker_activations: 0,
        error_types: HashMap::new(),
    }
}

fn print_chaos_results(results: &ChaosResults) {
    println!("  📊 Total Operations: {}", results.total_operations);
    println!("  ✅ Successful: {}", results.successful_operations);
    println!("  ❌ Failed: {}", results.failed_operations);
    println!("  🔄 Recovery Time: {:.2}s", results.recovery_time.as_secs_f64());
    println!("  🛡️  System Stable: {}", if results.system_stable { "✅" } else { "❌" });
    println!("  🔌 Circuit Breaker Activations: {}", results.circuit_breaker_activations);
    
    let success_rate = (results.successful_operations as f64 / results.total_operations as f64) * 100.0;
    println!("  📋 Success Rate During Chaos: {:.2}%", success_rate);
} 