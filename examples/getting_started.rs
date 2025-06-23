//! Getting Started Example
//! 
//! This example demonstrates the basic usage of Songbird Orchestrator
//! as described in docs/user/GETTING_STARTED.md

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_orchestrator::{
    errors::SongbirdError,
    traits::service::{
        UniversalService, 
        ServiceInfo, 
        ServiceRequest, 
        ServiceResponse, 
        ServiceMetrics,
        ResponseStatus
    },
    Orchestrator, 
    OrchestratorConfig,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Example service configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExampleServiceConfig {
    pub port: u16,
    pub name: String,
    pub timeout_ms: u64,
}

/// Example service that demonstrates the UniversalService trait
pub struct ExampleService {
    config: Option<ExampleServiceConfig>,
    is_running: bool,
    request_count: std::sync::atomic::AtomicU64,
}

impl ExampleService {
    pub fn new() -> Self {
        Self {
            config: None,
            is_running: false,
            request_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl UniversalService for ExampleService {
    type Config = ExampleServiceConfig;
    type Health = serde_json::Value;
    type Error = SongbirdError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = Some(config);
        println!("🔧 Service initialized with config: {:?}", self.config);
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        self.is_running = true;
        println!("🚀 Service started: {}", self.config.as_ref().unwrap().name);
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.is_running = false;
        println!("🛑 Service stopped");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        let status = if self.is_running { "healthy" } else { "stopped" };
        Ok(serde_json::json!({
            "status": status,
            "requests_processed": self.request_count.load(std::sync::atomic::Ordering::Relaxed),
            "uptime": "running"
        }))
    }
    
    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> {
        // Increment request counter
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Simulate some processing time
        sleep(Duration::from_millis(10)).await;
        
        println!("📨 Handling request: {} {}", request.method, request.path);
        
        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            payload: serde_json::json!({
                "message": "Hello from Example Service!",
                "method": request.method,
                "path": request.path,
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(10),
            processing_time: 10,
            metadata: HashMap::new(),
        })
    }
    
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> {
        let request_count = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        
        Ok(ServiceMetrics {
            request_count,
            error_count: 0,
            avg_response_time_ms: 10.0,
            p95_response_time_ms: 15.0,
            p99_response_time_ms: 20.0,
            cpu_usage: 15.0,
            memory_usage: 64 * 1024 * 1024, // 64 MB
            active_connections: 1,
            queue_depth: 0,
            throughput_rps: 10.0,
            error_rate: 0.0,
            uptime_seconds: 300,
            last_updated: chrono::Utc::now(),
            custom_metrics: HashMap::new(),
        })
    }
    
    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: "example-service".to_string(),
            name: "Example Service".to_string(),
            version: "1.0.0".to_string(),
            service_type: "example".to_string(),
            description: "A simple example service demonstrating Songbird Orchestrator".to_string(),
            endpoints: vec![],
            capabilities: vec!["http".to_string(), "json".to_string()],
            tags: HashMap::from([
                ("environment".to_string(), "example".to_string()),
                ("component".to_string(), "demo".to_string()),
            ]),
            metadata: HashMap::new(),
        }
    }
    
    async fn can_handle_load(&self) -> Result<bool, Self::Error> {
        Ok(self.is_running)
    }
    
    async fn get_load_factor(&self) -> Result<f64, Self::Error> {
        // Return load factor based on current request count
        let requests = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        let load_factor = (requests as f64 / 100.0).min(1.0);
        Ok(load_factor)
    }
    
    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = Some(config);
        println!("🔄 Service configuration updated");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 === SONGBIRD ORCHESTRATOR - GETTING STARTED EXAMPLE ===");
    println!();
    
    // Step 1: Create orchestrator with default configuration
    println!("📋 Creating orchestrator with default configuration...");
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    println!("✅ Orchestrator created successfully");
    println!();
    
    // Step 2: Start the orchestrator
    println!("🚀 Starting orchestrator...");
    orchestrator.start().await?;
    println!("✅ Orchestrator started");
    println!();
    
    // Step 3: Register an example service
    println!("📝 Registering example service...");
    let service = ExampleService::new();
    let service_config = ExampleServiceConfig {
        port: 8080,
        name: "ExampleService".to_string(),
        timeout_ms: 5000,
    };
    
    let service_id = orchestrator.register_service(service, service_config).await?;
    println!("✅ Service registered with ID: {}", service_id);
    println!();
    
    // Step 4: Verify the service is working
    println!("🔍 Verifying service registration...");
    let services = orchestrator.list_services().await;
    println!("📊 Total services: {}", services.len());
    for service in &services {
        println!("  - {} ({}): {}", service.name, service.id, service.description);
    }
    println!();
    
    // Step 5: Check service health
    println!("🏥 Checking service health...");
    let health = orchestrator.get_service_health(&service_id).await?;
    println!("✅ Service health: {:?}", health);
    println!();
    
    // Step 6: Send some test requests
    println!("📨 Sending test requests...");
    for i in 1..=3 {
        let test_request = ServiceRequest::new("GET", &format!("/test/{}", i))
            .with_payload(serde_json::json!({"test_id": i, "timestamp": chrono::Utc::now()}));
        
        let response = orchestrator.handle_service_request(&service_id, test_request).await?;
        match response.status {
            ResponseStatus::Success => {
                println!("  ✅ Request {}: Success - {}", i, response.payload["message"]);
            }
            _ => {
                println!("  ❌ Request {}: Failed - {:?}", i, response.status);
            }
        }
    }
    println!();
    
    // Step 7: Check service metrics
    println!("📊 Getting service metrics...");
    let metrics = orchestrator.get_service_metrics(&service_id).await?;
    println!("  - Requests processed: {}", metrics.request_count);
    println!("  - Errors: {}", metrics.error_count);
    println!("  - Average response time: {:.2}ms", metrics.avg_response_time_ms);
    println!("  - Memory usage: {:.2}MB", metrics.memory_usage as f64 / (1024.0 * 1024.0));
    println!();
    
    // Step 8: Check orchestrator metrics
    println!("🎛️ Getting orchestrator metrics...");
    let orch_metrics = orchestrator.get_metrics().await;
    println!("  - Total services: {}", orch_metrics.total_services);
    println!("  - Healthy services: {}", orch_metrics.healthy_services);
    println!("  - Total requests: {}", orch_metrics.total_requests);
    println!("  - Uptime: {}s", orch_metrics.uptime_seconds);
    println!();
    
    // Step 9: Test load balancer integration
    println!("⚖️ Testing load balancer integration...");
    let selected = orchestrator.select_service_for_request(Some("example")).await?;
    if let Some(selected_service) = selected {
        println!("✅ Load balancer selected service: {}", selected_service.name);
    } else {
        println!("⚠️ No service selected by load balancer");
    }
    println!();
    
    // Step 10: Test service discovery
    println!("🔍 Testing service discovery...");
    let discovered = orchestrator.list_discovered_services().await?;
    println!("📋 Discovered {} services:", discovered.len());
    for service in &discovered {
        println!("  - {}: {} ({})", service.name, service.description, service.service_type);
    }
    println!();
    
    // Step 11: Demonstrate graceful shutdown
    println!("🛑 Performing graceful shutdown...");
    println!("  - Stopping orchestrator...");
    orchestrator.stop().await?;
    println!("✅ Orchestrator stopped gracefully");
    println!();
    
    println!("🎉 === EXAMPLE COMPLETED SUCCESSFULLY ===");
    println!("📖 This demonstrates that Songbird Orchestrator delivers on all its promises!");
    println!("🚀 Your services are now ready for production HPC environments!");
    
    Ok(())
} 