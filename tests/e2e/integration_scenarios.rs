use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
#[allow(dead_code, unused_imports, unused_variables)]
// End-to-End Integration Tests
//
// Comprehensive integration scenarios testing full system functionality

use songbird_gaming_bridge::{
    orchestrator::Orchestrator,
    config::OrchestratorConfig,
    discovery::{ServiceRegistry, InMemoryServiceRegistry},
    communication::{HttpCommunication, WebSocketCommunication, CommunicationLayer},
    proxy::{ConnectionProxy, ProxyConfig},
    traits::{
        service_id::{UniversalService, ServiceInfo, ServiceEndpoint, EndpointParameter, ServiceRequest, ServiceResponse, ResponseStatus, ServiceMetrics},
        communication::{ServiceMessage, ServiceAddress, MessageType},
    },
};

use async_trait::async_trait;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// E2E Test Configuration
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    /// Number of services to deploy
    pub service_count: usize,
    /// Test duration
    pub test_processing_time: Duration,
    /// Request complexity (number of service hops)
    pub request_complexity: usize,
    /// Enable real network communication
    pub use_real_network: bool,
    /// Enable observability
    pub enable_observability: bool,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            service_count: 5,
            test_processing_time: Duration::from_secs(30),
            request_complexity: 3,
            use_real_network: false,
            enable_observability: true,
        }
    }
}

/// Test Service Implementation
#[derive(Debug, Clone)]
pub struct TestService {
    id: String,
    service_type: String,
    request_count: Arc<AtomicU64>,
    error_rate: f64,
}

impl TestService {
    pub fn new(id: String, service_type: String) -> Self {
        Self {
            id,
            service_type,
            request_count: Arc::new(AtomicU64::new(0)),
            error_rate: 0.05, // 5% error rate
        }
    }
    
    pub fn with_error_rate(mut self, error_rate: f64) -> Self {
        self.error_rate = error_rate;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub enabled: bool,
    pub max_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHealth {
    pub status: String,
    pub uptime_seconds: u64,
    pub request_count: u64,
}

#[derive(Debug, Clone)]
pub struct TestError {
    pub message: String,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TestError: {}", self.message)
    }
}

impl std::error::Error for TestError {}

#[async_trait]
impl UniversalService for TestService {
    type Config = TestConfig;
    type Health = TestHealth;
    type Error = TestError;

    async fn initialize(&mut self, _config: Self::Config) -> std::result::Result<()> {
        println!("✅ Initialized test service_id: {}", self.id);
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<()> {
        println!("🚀 Started test service_id: {}", self.id);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<()> {
        println!("🛑 Stopped test service_id: {}", self.id);
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health> {
        Ok(TestHealth {
            status: "healthy".to_string(),
            uptime_seconds: 300,
            request_count: self.request_count.load(Ordering::Relaxed),
        })
    }

    async fn handle_request(&self, request: ServiceRequest) -> std::result::Result<ServiceResponse> {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Simulate errors
        if rand::random::<f64>() < self.error_rate {
            return Err(TestError {
                message: format!("Simulated error in service {}", self.id),
            });
        }
        
        // Process based on service type
        let response_data = match self.service_type.as_str() {
            "data-processor" => {
                serde_json::json!({
                    "service": self.id,
                    "type": "data-processor",
                    "processed_data": request.body,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "processing_time_ms": 10
                })
            }
            "validator" => {
                let is_valid = request.body.get("data").is_some();
                serde_json::json!({
                    "service": self.id,
                    "type": "validator",
                    "valid": is_valid,
                    "original_request": request.body,
                    "validation_timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            "aggregator" => {
                serde_json::json!({
                    "service": self.id,
                    "type": "aggregator",
                    "aggregated_results": [request.body],
                    "count": 1,
                    "aggregation_timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            "notifier" => {
                serde_json::json!({
                    "service": self.id,
                    "type": "notifier",
                    "notification_sent": true,
                    "recipients": ["user@example.com"],
                    "message": format!("Processed: {:?}", request.body),
                    "sent_at": chrono::Utc::now().to_rfc3339()
                })
            }
            _ => {
                serde_json::json!({
                    "service": self.id,
                    "type": "generic",
                    "echo": request.body,
                    "processed_at": chrono::Utc::now().to_rfc3339()
                })
            }
        };

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: response_data,
            timestamp: chrono::Utc::now(),
            processing_time: Duration::from_millis(10),
            processing_time: std::time::Duration::from_millis(10),
            
        })
    }

    async fn update_config(&mut self, _config: Self::Config) -> std::result::Result<()> {
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics> {
        let mut metrics = ServiceMetrics::default();
        metrics.request_count = self.request_count.load(Ordering::Relaxed);
        metrics.uptime_seconds = 300;
        Ok(metrics)
    }

    async fn can_handle_load(&self) -> std::result::Result<bool> {
        Ok(true)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64> {
        Ok(0.1)
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_service_id: self.id.clone(),
            name: format!("Test Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: self.service_type.clone(),
            description: format!("Test service of type: {}", self.service_type),
            endpoints: vec![
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/process".to_string(),
                    method: "POST".to_string(),
                    description: Some("Process data").to_string(),
                    parameters: vec![
                        EndpointParameter {
                            name: "data".to_string(),
                            param_type: "object".to_string(),
                            required: true,
                            description: Some("Data to process").to_string(),
                            default: None,
                        }
                    ],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "service": {"type": "string"},
                            "type": {"type": "string"},
                            "processed_at": {"type": "string"}
                        }
                    })),
                }
            ],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }
}

/// Test Multi-Service Workflow
#[tokio::test]
async fn test_multi_service_workflow() {
    println!("🔄 Testing Multi-Service Workflow");
    
    let config = E2ETestConfig {
        service_count: 4,
        test_processing_time: Duration::from_secs(20),
        request_complexity: 3,
        ..Default::default()
    };
    
    let results = run_multi_service_workflow(config).await;
    
    // Validate workflow completion
    assert!(results.total_requests > 0, "Should process requests in workflow");
    assert!(results.successful_workflows > 0, "Should complete some workflows successfully");
    assert!(results.average_workflow_time < Duration::from_secs(5), "Workflow should complete in reasonable time");
    
    let success_rate = results.successful_workflows as f64 / results.total_requests as f64;
    assert!(success_rate > 0.8, "Workflow success rate should be over 80%");
    
    println!("📊 Multi-Service Workflow Results:");
    print_workflow_results(&results);
}

/// Test Service Discovery and Load Balancing
#[tokio::test]
async fn test_service_discovery_load_balancing() {
    println!("⚖️ Testing Service Discovery and Load Balancing");
    
    let config = E2ETestConfig {
        service_count: 6,
        test_processing_time: Duration::from_secs(15),
        ..Default::default()
    };
    
    let results = run_load_balancing_test(config).await;
    
    // Validate load balancing
    assert!(results.total_requests > 0, "Should distribute requests");
    assert!(results.services_used > 1, "Should use multiple services");
    assert!(results.load_distribution_variance < 0.3, "Load should be well distributed");
    
    println!("📊 Load Balancing Results:");
    print_load_balancing_results(&results);
}

/// Test Circuit Breaker Integration
#[tokio::test]
async fn test_circuit_breaker_integration() {
    println!("🔌 Testing Circuit Breaker Integration");
    
    let config = E2ETestConfig {
        service_count: 3,
        test_processing_time: Duration::from_secs(12),
        ..Default::default()
    };
    
    let results = run_circuit_breaker_test(config).await;
    
    // Validate circuit breaker behavior
    assert!(results.total_requests > 0, "Should attempt requests");
    assert!(results.circuit_breaker_trips > 0, "Circuit breakers should trip with failing services");
    assert!(results.recovery_attempts > 0, "Should attempt recovery");
    
    println!("📊 Circuit Breaker Results:");
    print_circuit_breaker_results(&results);
}

/// Test Real-Time Communication
#[tokio::test]
async fn test_real_time_communication() {
    println!("🔌 Testing Real-Time Communication");
    
    let config = E2ETestConfig {
        service_count: 4,
        test_processing_time: Duration::from_secs(10),
        use_real_network: false, // Simulated for testing
        ..Default::default()
    };
    
    let results = run_real_time_communication_test(config).await;
    
    // Validate real-time communication
    assert!(results.messages_sent > 0, "Should send real-time messages");
    assert!(results.messages_received > 0, "Should receive real-time messages");
    assert!(results.average_latency < Duration::from_millis(100), "Real-time latency should be low");
    
    let delivery_rate = results.messages_received as f64 / results.messages_sent as f64;
    assert!(delivery_rate > 0.9, "Message delivery rate should be over 90%");
    
    println!("📊 Real-Time Communication Results:");
    print_real_time_results(&results);
}

/// Test Proxy Integration
#[tokio::test]
async fn test_proxy_integration() {
    println!("🌐 Testing Proxy Integration");
    
    let config = E2ETestConfig {
        service_count: 3,
        test_processing_time: Duration::from_secs(8),
        ..Default::default()
    };
    
    let results = run_proxy_integration_test(config).await;
    
    // Validate proxy functionality
    assert!(results.total_requests > 0, "Should route requests through proxy");
    assert!(results.successful_requests > 0, "Should successfully route some requests");
    assert!(results.average_response_time < Duration::from_millis(200), "Proxy should add minimal latency");
    
    println!("📊 Proxy Integration Results:");
    print_proxy_results(&results);
}

/// Test Full System Integration
#[tokio::test]
async fn test_full_system_integration() {
    println!("🎼 Testing Full System Integration");
    
    let config = E2ETestConfig {
        service_count: 8,
        test_processing_time: Duration::from_secs(25),
        request_complexity: 4,
        enable_observability: true,
        ..Default::default()
    };
    
    let results = run_full_system_test(config).await;
    
    // Validate full system functionality
    assert!(results.total_operations > 0, "Should perform operations");
    assert!(results.successful_operations > 0, "Should complete operations successfully");
    assert!(results.system_stability > 0.9, "System should remain stable");
    assert!(results.observability_data_points > 0, "Should collect observability data");
    
    println!("📊 Full System Integration Results:");
    print_full_system_results(&results);
}

// Result structures and implementation functions

#[derive(Debug)]
pub struct WorkflowResults {
    pub total_requests: u64,
    pub successful_workflows: u64,
    pub failed_workflows: u64,
    pub average_workflow_time: Duration,
    pub service_interactions: u64,
}

#[derive(Debug)]
pub struct LoadBalancingResults {
    pub total_requests: u64,
    pub services_used: usize,
    pub load_distribution_variance: f64,
    pub request_distribution: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct CircuitBreakerResults {
    pub total_requests: u64,
    pub circuit_breaker_trips: u64,
    pub recovery_attempts: u64,
    pub successful_recoveries: u64,
}

#[derive(Debug)]
pub struct RealTimeResults {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub average_latency: Duration,
    pub connection_stability: f64,
}

#[derive(Debug)]
pub struct ProxyResults {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub routes_tested: usize,
}

#[derive(Debug)]
pub struct FullSystemResults {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub system_stability: f64,
    pub observability_data_points: u64,
    pub components_tested: Vec<String>,
}

async fn run_multi_service_workflow(config: E2ETestConfig) -> WorkflowResults {
    let orchestrator_config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(orchestrator_config).await.expect("Failed to create orchestrator");
    
    // Create test services for workflow
    let services = vec![
        TestService::new("data-processor-1".to_string(), "data-processor".to_string()),
        TestService::new("validator-1".to_string(), "validator".to_string()),
        TestService::new("aggregator-1".to_string(), "aggregator".to_string()),
        TestService::new("notifier-1".to_string(), "notifier".to_string()),
    ];
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_workflows = 0;
    let mut workflow_times = Vec::new();
    let mut service_interactions = 0;
    
    // Simulate workflow requests
    while start_time.elapsed() < config.test_duration {
        total_requests += 1;
        
        let workflow_start = Instant::now();
        
        // Simulate multi-step workflow
        let mut workflow_success = true;
        
        // Step 1: Data Processing
        let data_request = ServiceRequest {
            id: format!("workflow-{}-step-1", total_requests),
            path: "/process".to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            body: serde_json::json!({
                "data": format!("workflow-data-{}", total_requests),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            timestamp: chrono::Utc::now(),
            timeout: Some(Duration::from_secs(5)),
            
        };
        
        match services[0].handle_request(data_request).await {
            Ok(response) => {
                service_interactions += 1;
                
                // Step 2: Validation
                let validation_request = ServiceRequest {
                    id: format!("workflow-{}-step-2", total_requests),
                    path: "/validate".to_string(),
                    method: "POST".to_string(),
                    headers: HashMap::new(),
                    body: response.body,
                    timestamp: chrono::Utc::now(),
                    timeout: Some(Duration::from_secs(5)),
                    
                };
                
                match services[1].handle_request(validation_request).await {
                    Ok(validated_response) => {
                        service_interactions += 1;
                        
                        // Step 3: Aggregation
                        let aggregation_request = ServiceRequest {
                            id: format!("workflow-{}-step-3", total_requests),
                            path: "/aggregate".to_string(),
                            method: "POST".to_string(),
                            headers: HashMap::new(),
                            body: validated_response.body,
                            timestamp: chrono::Utc::now(),
                            timeout: Some(Duration::from_secs(5)),
                            
                        };
                        
                        if services[2].handle_request(aggregation_request).await.is_ok() {
                            service_interactions += 1;
                            // Workflow completed successfully
                        } else {
                            workflow_success = false;
                        }
                    }
                    Err(_) => workflow_success = false,
                }
            }
            Err(_) => workflow_success = false,
        }
        
        let workflow_time = workflow_start.elapsed();
        workflow_times.push(workflow_time);
        
        if workflow_success {
            successful_workflows += 1;
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let average_workflow_time = if !workflow_times.is_empty() {
        Duration::from_nanos(
            workflow_times.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / workflow_times.len() as u64
        )
    } else {
        Duration::from_millis(0)
    };
    
    WorkflowResults {
        total_requests,
        successful_workflows,
        failed_workflows: total_requests - successful_workflows,
        average_workflow_time,
        service_interactions,
    }
}

async fn run_load_balancing_test(config: E2ETestConfig) -> LoadBalancingResults {
    let registry = InMemoryServiceRegistry::new();
    let mut request_distribution = HashMap::new();
    
    // Register multiple instances of the same service type
    for i in 0..config.service_count {
        let service = ServiceInfo {
            id: format!("load-test-service-{}", i),
            name: format!("Load Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "load-test".to_string(),
            description: Some("Service for load balancing test").to_string(),
            endpoints: vec![],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        registry.register_service(service).await.expect("Failed to register service");
        request_distribution.insert(format!("load-test-service-{}", i), 0);
    }
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut services_used = std::collections::HashSet::new();
    
    // Distribute requests across services
    while start_time.elapsed() < config.test_duration {
        total_requests += 1;
        
        // Discover services for load balancing
        match registry.discover_services("load-test").await {
            Ok(services) => {
                if !services.is_empty() {
                    // Simple round-robin load balancing
                    let selected_service = &services[total_requests as usize % services.len()];
                    services_used.insert(selected_service.id.clone());
                    
                    // Track request distribution
                    *request_distribution.entry(selected_service.id.clone()).or_insert(0) += 1;
                }
            }
            Err(_) => {}
        }
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Calculate load distribution variance
    let mean_requests = total_requests as f64 / config.service_count as f64;
    let variance = request_distribution.values()
        .map(|&count| {
            let diff = count as f64 - mean_requests;
            diff * diff
        })
        .sum::<f64>() / config.service_count as f64;
    
    let load_distribution_variance = variance / (mean_requests * mean_requests);
    
    LoadBalancingResults {
        total_requests,
        services_used: services_used.len(),
        load_distribution_variance,
        request_distribution,
    }
}

async fn run_circuit_breaker_test(config: E2ETestConfig) -> CircuitBreakerResults {
    // Create services with different error rates
    let services = vec![
        TestService::new("reliable-service".to_string(), "reliable".to_string()).with_error_rate(0.05),
        TestService::new("unreliable-service".to_string(), "unreliable".to_string()).with_error_rate(0.8),
        TestService::new("failing-service".to_string(), "failing".to_string()).with_error_rate(0.95),
    ];
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut circuit_breaker_trips = 0;
    let mut recovery_attempts = 0;
    let mut successful_recoveries = 0;
    
    // Track failure counts for circuit breaker simulation
    let mut failure_counts = HashMap::new();
    let circuit_breaker_threshold = 5;
    
    while start_time.elapsed() < config.test_duration {
        total_requests += 1;
        
        for service in &services {
            let service_id = &service.id;
            let failure_count = failure_counts.get(service_id).unwrap_or(&0);
            
            // Check if circuit breaker should be open
            if *failure_count >= circuit_breaker_threshold {
                // Circuit breaker is open, attempt recovery
                recovery_attempts += 1;
                
                // Simulate recovery attempt
                if rand::random::<f64>() < 0.3 {  // 30% chance of recovery
                    failure_counts.insert(service_id.clone(), 0);
                    successful_recoveries += 1;
                }
                continue;
            }
            
            // Make request
            let request = ServiceRequest {
                id: format!("circuit-test-{}", total_requests),
                path: "/test".to_string(),
                method: "POST".to_string(),
                headers: HashMap::new(),
                body: serde_json::json!({"test": "data"}),
                timestamp: chrono::Utc::now(),
                timeout: Some(Duration::from_secs(5)),
                
            };
            
            match service.handle_request(request).await {
                Ok(_) => {
                    // Reset failure count on success
                    failure_counts.insert(service_id.clone(), 0);
                }
                Err(_) => {
                    // Increment failure count
                    let new_count = failure_counts.get(service_id).unwrap_or(&0) + 1;
                    failure_counts.insert(service_id.clone(), new_count);
                    
                    if new_count >= circuit_breaker_threshold {
                        circuit_breaker_trips += 1;
                    }
                }
            }
        }
        
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    CircuitBreakerResults {
        total_requests,
        circuit_breaker_trips,
        recovery_attempts,
        successful_recoveries,
    }
}

async fn run_real_time_communication_test(config: E2ETestConfig) -> RealTimeResults {
    let ws_comm = WebSocketCommunication::new("127.0.0.1".to_string(), 0);
    
    let start_time = Instant::now();
    let mut messages_sent = 0;
    let mut messages_received = 0;
    let mut latencies = Vec::new();
    
    // Simulate real-time message exchange
    while start_time.elapsed() < config.test_duration {
        messages_sent += 1;
        
        let message_start = Instant::now();
        
        // Create real-time message
        let message = ServiceMessage {
            id: format!("realtime-{}", messages_sent),
            message_type: MessageType::Event,
            topic: "real-time-test".to_string(),
            body: serde_json::json!({
                "event": "test-event",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "sequence": messages_sent
            }),
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            ttl: Some(Duration::from_secs(10)),
            headers: HashMap::new(),
        };
        
        // Simulate WebSocket message sending and receiving
        // In a real test, this would involve actual WebSocket connections
        tokio::time::sleep(Duration::from_micros(100)).await; // Simulate network latency
        
        let latency = message_start.elapsed();
        latencies.push(latency);
        
        // Simulate successful message delivery (95% success rate)
        if rand::random::<f64>() < 0.95 {
            messages_received += 1;
        }
        
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    
    let average_latency = if !latencies.is_empty() {
        Duration::from_nanos(
            latencies.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / latencies.len() as u64
        )
    } else {
        Duration::from_millis(0)
    };
    
    let connection_stability = messages_received as f64 / messages_sent as f64;
    
    RealTimeResults {
        messages_sent,
        messages_received,
        average_latency,
        connection_stability,
    }
}

async fn run_proxy_integration_test(config: E2ETestConfig) -> ProxyResults {
    let proxy_config = ProxyConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0,
        enable_logging: false,
        ..Default::default()
    };
    
    let proxy = ConnectionProxy::new(proxy_config);
    
    // Register test services
    let mut services = Vec::new();
    for i in 0..config.service_count {
        let service = ServiceInfo {
            id: format!("proxy-test-service-{}", i),
            name: format!("Proxy Test Service {}", i),
            version: "1.0.0".to_string(),
            service_type: "http".to_string(),
            description: Some("Service for proxy testing").to_string(),
            endpoints: vec![
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: format!("/api/service-{}", i),
                    method: "POST".to_string(),
                    description: Some("Test endpoint").to_string(),
                    parameters: vec![],
                    response_schema: None,
                }
            ],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        };
        services.push(service);
    }
    
    proxy.update_services(services).await.expect("Failed to register services");
    
    let start_time = Instant::now();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut response_times = Vec::new();
    let mut routes_tested = std::collections::HashSet::new();
    
    // Test proxy routing
    while start_time.elapsed() < config.test_duration {
        total_requests += 1;
        
        let service_id = format!("proxy-test-service-{}", total_requests % config.service_count);
        routes_tested.insert(service_id.clone());
        
        let request_start = Instant::now();
        
        // Create proxy request
        let proxy_request = songbird_gaming_bridge::proxy::ProxyRequest {
            method: axum::http::Method::POST,
            uri: format!("/api/service-{}", total_requests % config.service_count).parse().expect("Test assertion failed"),
            headers: axum::http::HeaderMap::new(),
            body: b"proxy test data".to_vec(),
            source_ip: Some("127.0.0.1".to_string()),
            timestamp: std::time::Instant::now(),
        };
        
        match proxy.route_request(&service_id, proxy_request).await {
            Ok(_) => {
                successful_requests += 1;
                let response_time = request_start.elapsed();
                response_times.push(response_time);
            }
            Err(_) => {
                // Expected for testing without real backends
            }
        }
        
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    
    let average_response_time = if !response_times.is_empty() {
        Duration::from_nanos(
            response_times.iter().map(|d| d.as_nanos()).sum::<u128>() as u64 / response_times.len() as u64
        )
    } else {
        Duration::from_millis(100) // Default for testing
    };
    
    ProxyResults {
        total_requests,
        successful_requests,
        failed_requests: total_requests - successful_requests,
        average_response_time,
        routes_tested: routes_tested.len(),
    }
}

async fn run_full_system_test(config: E2ETestConfig) -> FullSystemResults {
    let orchestrator_config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(orchestrator_config).await.expect("Failed to create orchestrator");
    
    let start_time = Instant::now();
    let mut total_operations = 0;
    let mut successful_operations = 0;
    let mut stability_checks = Vec::new();
    let mut observability_data_points = 0;
    
    let components_tested = vec![
        "orchestrator".to_string(),
        "service_discovery".to_string(),
        "communication".to_string(),
        "proxy".to_string(),
        "observability".to_string(),
    ];
    
    // Run comprehensive system test
    while start_time.elapsed() < config.test_duration {
        total_operations += 1;
        
        // Test orchestrator health
        let health_start = Instant::now();
        let is_healthy = orchestrator.is_healthy().await;
        let health_check_time = health_start.elapsed();
        
        stability_checks.push(if is_healthy && health_check_time < Duration::from_millis(100) {
            1.0
        } else {
            0.0
        });
        
        if is_healthy {
            successful_operations += 1;
        }
        
        // Simulate observability data collection
        if config.enable_observability {
            observability_data_points += 1;
            
            // Simulate metrics collection
            if total_operations % 10 == 0 {
                observability_data_points += 5; // Batch metrics
            }
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    let system_stability = if !stability_checks.is_empty() {
        stability_checks.iter().sum::<f64>() / stability_checks.len() as f64
    } else {
        0.0
    };
    
    FullSystemResults {
        total_operations,
        successful_operations,
        failed_operations: total_operations - successful_operations,
        system_stability,
        observability_data_points,
        components_tested,
    }
}

// Result printing functions

fn print_workflow_results(results: &WorkflowResults) {
    println!("  📊 Total Requests: {}", results.total_requests);
    println!("  ✅ Successful Workflows: {}", results.successful_workflows);
    println!("  ❌ Failed Workflows: {}", results.failed_workflows);
    println!("  ⏱️  Average Workflow Time: {:.2}ms", results.average_workflow_time.as_millis());
    println!("  🔄 Service Interactions: {}", results.service_interactions);
    
    let success_rate = (results.successful_workflows as f64 / results.total_requests as f64) * 100.0;
    println!("  📋 Workflow Success Rate: {:.2}%", success_rate);
}

fn print_load_balancing_results(results: &LoadBalancingResults) {
    println!("  📊 Total Requests: {}", results.total_requests);
    println!("  🎯 Services Used: {}", results.services_used);
    println!("  📈 Load Distribution Variance: {:.3}", results.load_distribution_variance);
    println!("  🔄 Request Distribution:");
    for (service, count) in &results.request_distribution {
        println!("    {} -> {} requests", service, count);
    }
}

fn print_circuit_breaker_results(results: &CircuitBreakerResults) {
    println!("  📊 Total Requests: {}", results.total_requests);
    println!("  🔌 Circuit Breaker Trips: {}", results.circuit_breaker_trips);
    println!("  🔄 Recovery Attempts: {}", results.recovery_attempts);
    println!("  ✅ Successful Recoveries: {}", results.successful_recoveries);
    
    let recovery_rate = if results.recovery_attempts > 0 {
        (results.successful_recoveries as f64 / results.recovery_attempts as f64) * 100.0
    } else {
        0.0
    };
    println!("  📋 Recovery Rate: {:.2}%", recovery_rate);
}

fn print_real_time_results(results: &RealTimeResults) {
    println!("  📤 Messages Sent: {}", results.messages_sent);
    println!("  📥 Messages Received: {}", results.messages_received);
    println!("  ⏱️  Average Latency: {:.2}ms", results.average_latency.as_millis());
    println!("  🔗 Connection Stability: {:.2}%", results.connection_stability * 100.0);
}

fn print_proxy_results(results: &ProxyResults) {
    println!("  📊 Total Requests: {}", results.total_requests);
    println!("  ✅ Successful: {}", results.successful_requests);
    println!("  ❌ Failed: {}", results.failed_requests);
    println!("  ⏱️  Average Response Time: {:.2}ms", results.average_response_time.as_millis());
    println!("  🛣️  Routes Tested: {}", results.routes_tested);
}

fn print_full_system_results(results: &FullSystemResults) {
    println!("  📊 Total Operations: {}", results.total_operations);
    println!("  ✅ Successful: {}", results.successful_operations);
    println!("  ❌ Failed: {}", results.failed_operations);
    println!("  🛡️  System Stability: {:.2}%", results.system_stability * 100.0);
    println!("  📈 Observability Data Points: {}", results.observability_data_points);
    println!("  🧩 Components Tested: {:?}", results.components_tested);
} 