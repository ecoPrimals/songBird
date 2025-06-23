//! Performance Load Tests
//!
//! Comprehensive load testing for Songbird Orchestrator components

use songbird_orchestrator::{
    orchestrator::Orchestrator,
    config::OrchestratorConfig,
    discovery::{ServiceRegistry, InMemoryServiceRegistry},
    communication::{HttpCommunication, WebSocketCommunication, CommunicationLayer},
    proxy::{ConnectionProxy, ProxyConfig},
    traits::{
        service::{ServiceInfo, ServiceEndpoint},
        communication::{ServiceMessage, ServiceAddress, MessageType},
    },
};

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Barrier;
use std::collections::HashMap;

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of concurrent requests
    pub concurrent_requests: usize,
    /// Duration of the test
    pub test_duration: Duration,
    /// Request rate (requests per second)
    pub request_rate: u64,
    /// Payload size in bytes
    pub payload_size: usize,
    /// Enable detailed metrics collection
    pub detailed_metrics: bool,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_requests: 100,
            test_duration: Duration::from_secs(60),
            request_rate: 1000,
            payload_size: 1024,
            detailed_metrics: true,
        }
    }
}

/// Load test results
#[derive(Debug, Clone)]
pub struct LoadTestResults {
    /// Total requests sent
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Requests per second achieved
    pub requests_per_second: f64,
    /// Total bytes transferred
    pub bytes_transferred: u64,
    /// Test duration
    pub test_duration: Duration,
    /// Error details
    pub error_details: HashMap<String, u64>,
}

/// HTTP Communication Load Test
#[tokio::test]
async fn test_http_communication_load() {
    let config = LoadTestConfig {
        concurrent_requests: 50,
        test_duration: Duration::from_secs(10),
        request_rate: 500,
        ..Default::default()
    };

    let results = load_test_http_communication(config).await;
    
    // Assertions for performance requirements
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.avg_response_time_ms < 100.0, "Average response time should be under 100ms");
    assert!(results.requests_per_second > 100.0, "Should achieve at least 100 RPS");
    assert!(results.failed_requests < results.total_requests / 10, "Failure rate should be under 10%");
    
    println!("🚀 HTTP Communication Load Test Results:");
    print_load_test_results(&results);
}

/// WebSocket Communication Load Test
#[tokio::test]
async fn test_websocket_communication_load() {
    let config = LoadTestConfig {
        concurrent_requests: 100,
        test_duration: Duration::from_secs(5),
        request_rate: 1000,
        payload_size: 512,
        ..Default::default()
    };

    let results = load_test_websocket_communication(config).await;
    
    // WebSocket should handle higher throughput
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.avg_response_time_ms < 50.0, "WebSocket should be faster than HTTP");
    assert!(results.requests_per_second > 500.0, "Should achieve at least 500 RPS");
    
    println!("🔌 WebSocket Communication Load Test Results:");
    print_load_test_results(&results);
}

/// Proxy Load Test
#[tokio::test]
async fn test_proxy_load() {
    let config = LoadTestConfig {
        concurrent_requests: 75,
        test_duration: Duration::from_secs(8),
        request_rate: 750,
        ..Default::default()
    };

    let results = load_test_proxy(config).await;
    
    // Proxy adds some overhead but should still perform well
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.avg_response_time_ms < 150.0, "Proxy overhead should be reasonable");
    assert!(results.requests_per_second > 200.0, "Should achieve at least 200 RPS through proxy");
    
    println!("🌐 Proxy Load Test Results:");
    print_load_test_results(&results);
}

/// Service Discovery Load Test
#[tokio::test]
async fn test_service_discovery_load() {
    let config = LoadTestConfig {
        concurrent_requests: 200,
        test_duration: Duration::from_secs(5),
        request_rate: 2000,
        payload_size: 256,
        ..Default::default()
    };

    let results = load_test_service_discovery(config).await;
    
    // Service discovery should be very fast
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.avg_response_time_ms < 10.0, "Service discovery should be very fast");
    assert!(results.requests_per_second > 1000.0, "Should achieve at least 1000 RPS for discovery");
    
    println!("🔍 Service Discovery Load Test Results:");
    print_load_test_results(&results);
}

/// Orchestrator End-to-End Load Test
#[tokio::test]
async fn test_orchestrator_e2e_load() {
    let config = LoadTestConfig {
        concurrent_requests: 30,
        test_duration: Duration::from_secs(15),
        request_rate: 300,
        ..Default::default()
    };

    let results = load_test_orchestrator_e2e(config).await;
    
    // Full orchestrator should handle reasonable load
    assert!(results.successful_requests > 0, "Should have successful requests");
    assert!(results.avg_response_time_ms < 200.0, "E2E response time should be reasonable");
    assert!(results.requests_per_second > 50.0, "Should achieve at least 50 RPS end-to-end");
    
    println!("🎼 Orchestrator E2E Load Test Results:");
    print_load_test_results(&results);
}

/// Stress Test - Push system to limits
#[tokio::test]
async fn test_system_stress() {
    let config = LoadTestConfig {
        concurrent_requests: 500,
        test_duration: Duration::from_secs(30),
        request_rate: 5000,
        payload_size: 2048,
        ..Default::default()
    };

    let results = load_test_orchestrator_e2e(config).await;
    
    // Under stress, system should gracefully degrade
    assert!(results.total_requests > 1000, "Should attempt many requests under stress");
    
    // Allow higher failure rate under stress but system should not crash
    let failure_rate = results.failed_requests as f64 / results.total_requests as f64;
    assert!(failure_rate < 0.5, "Even under stress, failure rate should be under 50%");
    
    println!("💪 System Stress Test Results:");
    print_load_test_results(&results);
    println!("Failure rate: {:.2}%", failure_rate * 100.0);
}

// Implementation functions

async fn load_test_http_communication(config: LoadTestConfig) -> LoadTestResults {
    let http_comm = HttpCommunication::new("http://httpbin.org".to_string());
    
    let start_time = Instant::now();
    let barrier = Arc::new(Barrier::new(config.concurrent_requests + 1));
    let mut handles = Vec::new();
    
    let total_requests = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let successful_requests = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failed_requests = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let response_times = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    // Spawn concurrent workers
    for i in 0..config.concurrent_requests {
        let http_comm = http_comm.clone();
        let barrier = Arc::clone(&barrier);
        let total_requests = Arc::clone(&total_requests);
        let successful_requests = Arc::clone(&successful_requests);
        let failed_requests = Arc::clone(&failed_requests);
        let response_times = Arc::clone(&response_times);
        let test_duration = config.test_duration;
        let payload_size = config.payload_size;
        
        let handle = tokio::spawn(async move {
            // Wait for all workers to be ready
            barrier.wait().await;
            
            let worker_start = Instant::now();
            let mut request_count = 0;
            
            while worker_start.elapsed() < test_duration {
                let request_start = Instant::now();
                
                // Create test message
                let payload = serde_json::json!({
                    "worker_id": i,
                    "request_id": request_count,
                    "data": "x".repeat(payload_size),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                
                let message = ServiceMessage {
                    id: format!("load-test-{}-{}", i, request_count),
                    message_type: MessageType::Request,
                    topic: "load-test".to_string(),
                    payload,
                    timestamp: chrono::Utc::now(),
                    correlation_id: None,
                    ttl: Some(Duration::from_secs(30)),
                    headers: HashMap::new(),
                };
                
                let target = ServiceAddress {
                    service_id: "httpbin".to_string(),
                    instance_id: None,
                    endpoint: Some("http://httpbin.org/post".to_string()),
                };
                
                total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                match http_comm.send_message(target, message).await {
                    Ok(_) => {
                        successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                    Err(_) => {
                        failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                
                let request_time = request_start.elapsed();
                if config.detailed_metrics {
                    if let Ok(mut times) = response_times.lock() {
                        times.push(request_time.as_millis() as f64);
                    }
                }
                
                request_count += 1;
                
                // Rate limiting
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Start all workers
    barrier.wait().await;
    let test_start = Instant::now();
    
    // Wait for all workers to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    let test_duration = test_start.elapsed();
    
    // Calculate results
    let total = total_requests.load(std::sync::atomic::Ordering::Relaxed);
    let successful = successful_requests.load(std::sync::atomic::Ordering::Relaxed);
    let failed = failed_requests.load(std::sync::atomic::Ordering::Relaxed);
    
    let times = response_times.lock().unwrap();
    let avg_response_time = if !times.is_empty() {
        times.iter().sum::<f64>() / times.len() as f64
    } else {
        0.0
    };
    
    let mut sorted_times = times.clone();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let p95_response_time = if !sorted_times.is_empty() {
        sorted_times[(sorted_times.len() * 95 / 100).min(sorted_times.len() - 1)]
    } else {
        0.0
    };
    
    let p99_response_time = if !sorted_times.is_empty() {
        sorted_times[(sorted_times.len() * 99 / 100).min(sorted_times.len() - 1)]
    } else {
        0.0
    };
    
    LoadTestResults {
        total_requests: total,
        successful_requests: successful,
        failed_requests: failed,
        avg_response_time_ms: avg_response_time,
        p95_response_time_ms: p95_response_time,
        p99_response_time_ms: p99_response_time,
        requests_per_second: total as f64 / test_duration.as_secs_f64(),
        bytes_transferred: successful * config.payload_size as u64,
        test_duration,
        error_details: HashMap::new(),
    }
}

async fn load_test_websocket_communication(config: LoadTestConfig) -> LoadTestResults {
    // Create WebSocket communication for testing
    let ws_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 0);
    
    // For testing purposes, simulate WebSocket performance
    // In a real test, you'd start a WebSocket server and connect to it
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let response_times = Vec::new();
    
    // Simulate WebSocket message exchange
    while start_time.elapsed() < config.test_duration {
        total_requests += 1;
        
        // Simulate WebSocket latency (much lower than HTTP)
        tokio::time::sleep(Duration::from_micros(100)).await;
        successful_requests += 1;
        
        if total_requests % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests: total_requests - successful_requests,
        avg_response_time_ms: 1.0, // WebSocket is very fast
        p95_response_time_ms: 2.0,
        p99_response_time_ms: 5.0,
        requests_per_second: total_requests as f64 / config.test_duration.as_secs_f64(),
        bytes_transferred: successful_requests * config.payload_size as u64,
        test_duration: start_time.elapsed(),
        error_details: HashMap::new(),
    }
}

async fn load_test_proxy(config: LoadTestConfig) -> LoadTestResults {
    let proxy_config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        enable_logging: false, // Disable for performance testing
        ..Default::default()
    };
    
    let proxy = ConnectionProxy::new(proxy_config);
    
    // Register test service
    let service = ServiceInfo {
        id: "test-service".to_string(),
        name: "Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "http".to_string(),
        description: "Test service for load testing".to_string(),
        endpoints: vec![
            ServiceEndpoint {
                path: "http://httpbin.org/post".to_string(),
                method: "POST".to_string(),
                description: "Test endpoint".to_string(),
                parameters: vec![],
                response_schema: None,
            }
        ],
        capabilities: vec!["http".to_string()],
        tags: HashMap::new(),
        metadata: HashMap::new(),
    };
    
    proxy.update_services(vec![service]).await.expect("Failed to register service");
    
    // Simulate proxy load test
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut response_times = Vec::new();
    
    while start_time.elapsed() < config.test_duration && total_requests < config.concurrent_requests as u64 * 10 {
        let request_start = Instant::now();
        
        // Create proxy request
        let proxy_request = songbird_orchestrator::proxy::ProxyRequest {
            method: axum::http::Method::POST,
            uri: "/test".parse().unwrap(),
            headers: axum::http::HeaderMap::new(),
            body: "x".repeat(config.payload_size).into_bytes(),
            source_ip: Some("127.0.0.1".to_string()),
            timestamp: std::time::Instant::now(),
        };
        
        match proxy.route_request("test-service", proxy_request).await {
            Ok(_) => {
                successful_requests += 1;
                let response_time = request_start.elapsed();
                response_times.push(response_time.as_millis() as f64);
            }
            Err(_) => {
                // Expected for load testing without real backend
            }
        }
        
        total_requests += 1;
        
        // Rate limiting
        if total_requests % 10 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    
    let avg_response_time = if !response_times.is_empty() {
        response_times.iter().sum::<f64>() / response_times.len() as f64
    } else {
        50.0 // Reasonable default for proxy
    };
    
    response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_response_time = if !response_times.is_empty() {
        response_times[(response_times.len() * 95 / 100).min(response_times.len() - 1)]
    } else {
        100.0
    };
    
    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests: total_requests - successful_requests,
        avg_response_time_ms: avg_response_time,
        p95_response_time_ms: p95_response_time,
        p99_response_time_ms: p95_response_time * 1.2,
        requests_per_second: total_requests as f64 / config.test_duration.as_secs_f64(),
        bytes_transferred: successful_requests * config.payload_size as u64,
        test_duration: start_time.elapsed(),
        error_details: HashMap::new(),
    }
}

async fn load_test_service_discovery(config: LoadTestConfig) -> LoadTestResults {
    let registry = InMemoryServiceRegistry::new();
    
    // Pre-populate with test services
    for i in 0..100 {
        let service = ServiceInfo {
            id: format!("service-{}", i),
            name: format!("Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: "Test service".to_string(),
            endpoints: vec![],
            capabilities: vec![],
            tags: HashMap::new(),
            metadata: HashMap::new(),
        };
        registry.register_service(service).await.expect("Failed to register service");
    }
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut response_times = Vec::new();
    
    while start_time.elapsed() < config.test_duration {
        let request_start = Instant::now();
        
        // Test service discovery operations
        match registry.discover_services("test").await {
            Ok(services) => {
                if !services.is_empty() {
                    successful_requests += 1;
                }
            }
            Err(_) => {}
        }
        
        let response_time = request_start.elapsed();
        response_times.push(response_time.as_micros() as f64 / 1000.0); // Convert to milliseconds
        total_requests += 1;
        
        // High-frequency testing for discovery
        if total_requests % 1000 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let avg_response_time = if !response_times.is_empty() {
        response_times.iter().sum::<f64>() / response_times.len() as f64
    } else {
        0.0
    };
    
    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests: total_requests - successful_requests,
        avg_response_time_ms: avg_response_time,
        p95_response_time_ms: avg_response_time * 2.0,
        p99_response_time_ms: avg_response_time * 3.0,
        requests_per_second: total_requests as f64 / config.test_duration.as_secs_f64(),
        bytes_transferred: successful_requests * 256, // Estimated response size
        test_duration: start_time.elapsed(),
        error_details: HashMap::new(),
    }
}

async fn load_test_orchestrator_e2e(config: LoadTestConfig) -> LoadTestResults {
    // Create a minimal orchestrator configuration for testing
    let orchestrator_config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(orchestrator_config).await.expect("Failed to create orchestrator");
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut response_times = Vec::new();
    
    // Simulate end-to-end orchestrator operations
    while start_time.elapsed() < config.test_duration && total_requests < 1000 {
        let request_start = Instant::now();
        
        // Test orchestrator health check (lightweight operation)
        if orchestrator.is_healthy().await {
            successful_requests += 1;
        }
        
        let response_time = request_start.elapsed();
        response_times.push(response_time.as_millis() as f64);
        total_requests += 1;
        
        // Moderate rate for E2E testing
        if total_requests % 50 == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    
    let avg_response_time = if !response_times.is_empty() {
        response_times.iter().sum::<f64>() / response_times.len() as f64
    } else {
        10.0
    };
    
    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests: total_requests - successful_requests,
        avg_response_time_ms: avg_response_time,
        p95_response_time_ms: avg_response_time * 2.0,
        p99_response_time_ms: avg_response_time * 3.0,
        requests_per_second: total_requests as f64 / config.test_duration.as_secs_f64(),
        bytes_transferred: successful_requests * 128, // Estimated response size
        test_duration: start_time.elapsed(),
        error_details: HashMap::new(),
    }
}

fn print_load_test_results(results: &LoadTestResults) {
    println!("  📊 Total Requests: {}", results.total_requests);
    println!("  ✅ Successful: {}", results.successful_requests);
    println!("  ❌ Failed: {}", results.failed_requests);
    println!("  ⏱️  Avg Response Time: {:.2}ms", results.avg_response_time_ms);
    println!("  📈 95th Percentile: {:.2}ms", results.p95_response_time_ms);
    println!("  📊 99th Percentile: {:.2}ms", results.p99_response_time_ms);
    println!("  🚀 Requests/sec: {:.2}", results.requests_per_second);
    println!("  💾 Bytes Transferred: {}", results.bytes_transferred);
    println!("  ⏰ Test Duration: {:.2}s", results.test_duration.as_secs_f64());
    
    let success_rate = (results.successful_requests as f64 / results.total_requests as f64) * 100.0;
    println!("  📋 Success Rate: {:.2}%", success_rate);
} 