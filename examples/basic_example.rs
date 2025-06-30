use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use songbird_gaming_bridge::prelude::*;
use songbird_gaming_bridge::traits::service_id::{
    EndpointParameter, ResponseStatus, ServiceEndpoint, ServiceInfo, ServiceMetrics,
    ServiceRequest, ServiceResponse, UniversalService,
};
use std::collections::HashMap;

/// Example service that provides greeting functionality
#[derive(Clone)]
struct GreetingService {
    id: String,
    greeting_prefix: String,
}

impl GreetingService {
    fn new(id: String, prefix: String) -> Self {
        Self {
            id,
            greeting_prefix: prefix,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GreetingConfig {
    prefix: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct GreetingHealth {
    status: String,
    uptime: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("Greeting service error: {message}")]
struct GreetingError {
    message: String,
}

#[async_trait]
impl UniversalService for GreetingService {
    type Config = GreetingConfig;
    type Health = GreetingHealth;
    type Error = GreetingError;

    async fn initialize(&mut self, config: Self::Config) -> std::result::Result<(), Self::Error> {
        self.greeting_prefix = config.prefix;
        println!(
            "🚀 Initialized {} with prefix: {}",
            self.id, self.greeting_prefix
        );
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        println!("▶️  Started greeting service_id: {}", self.id);
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        println!("⏹️  Stopped greeting service_id: {}", self.id);
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        Ok(GreetingHealth {
            status: "healthy".to_string(),
            uptime: 100, // mock uptime
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error> {
        // Extract name from request payload
        let name = request
            .body
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        let greeting = format!("{} {}!", self.greeting_prefix, name);

        println!("🗣️  {} handling request - greeting: {}", self.id, greeting);

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: serde_json::json!({
                "greeting": greeting,
                "service_id": self.id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            timestamp: Utc::now(),
            processing_time: std::time::Duration::from_millis( std::time::Duration::from_millis(5),
            processing_time: std::time::Duration::from_millis(3),
            
        })
    }

    async fn update_config(
        &mut self,
        config: Self::Config,
    ) -> std::result::Result<(), Self::Error> {
        self.greeting_prefix = config.prefix;
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics::default())
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        Ok(0.3) // 30% load
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_id: self.id.clone(),
            name: format!("Greeting Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: "greeting".to_string(),
            description: Some("A friendly greeting service").to_string(),
            endpoints: vec![ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                path: "/greet".to_string(),
                method: "POST".to_string(),
                description: Some("Generate a greeting").to_string(),
                parameters: vec![EndpointParameter {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: Some("Name to greet").to_string(),
                    default: Some(serde_json::json!("World")),
                }],
                response_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "greeting": {"type": "string"},
                        "service_id": {"type": "string"},
                        "timestamp": {"type": "string"}
                    }
                })),
            }],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎼 Songbird Orchestrator - Basic Example");
    println!("========================================");

    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;

    // Create greeting services with different prefixes
    let hello_service = GreetingService::new("hello-service".to_string(), "Hello".to_string());
    let hey_service = GreetingService::new("hey-service".to_string(), "Hey there".to_string());
    let bonjour_service =
        GreetingService::new("bonjour-service".to_string(), "Bonjour".to_string());

    // Register services
    println!("\n📝 Registering services...");

    let hello_id = orchestrator
        .register_service(
            hello_service,
            GreetingConfig {
                prefix: "Hello".to_string(),
                enabled: true,
            },
        )
        .await?;

    let hey_id = orchestrator
        .register_service(
            hey_service,
            GreetingConfig {
                prefix: "Hey there".to_string(),
                enabled: true,
            },
        )
        .await?;

    let bonjour_id = orchestrator
        .register_service(
            bonjour_service,
            GreetingConfig {
                prefix: "Bonjour".to_string(),
                enabled: true,
            },
        )
        .await?;

    println!("✅ Registered {} services", 3);

    // Show orchestrator metrics
    let metrics = orchestrator.get_config().await;
    println!("\n📊 Orchestrator Metrics:");
    println!("   Total services: {}", metrics.total_services);
    println!("   Healthy services: {}", metrics.healthy_services);

    // List all services
    println!("\n📋 Registered Services:");
    let services = orchestrator.list_services().await;
    for service in &services {
        println!(
            "   - {} ({}): {}",
            service.id, service.service_type, service.description
        );
    }

    // Create test requests
    println!("\n🔄 Testing Service Requests:");

    let test_requests = vec![
        ("Alice", &hello_id),
        ("Bob", &hey_id),
        ("Marie", &bonjour_id),
        ("Charlie", &hello_id), // Test load balancing by reusing hello service
    ];

    for (name, service_id) in test_requests {
        let request = ServiceRequest {
            id: format!("req-{}", uuid::Uuid::new_v4()),
            method: "POST".to_string(),
            path: "/greet".to_string(),
            headers: HashMap::new(),
            body: serde_json::json!({"name": name}),
            timestamp: Utc::now(),
            timeout: Some(std::time::Duration::from_secs(5)),
            client_info: None,
            
        };

        match orchestrator
            .handle_service_request(service_id, request)
            .await
        {
            Ok(response) => {
                if let ResponseStatus::Success = response.status {
                    let greeting = response.body["greeting"]
                        .as_str()
                        .unwrap_or("(no greeting)");
                    let service_id = response.body["service_id"]
                        .as_str()
                        .unwrap_or("(unknown)");
                    println!("   ✅ {}: \"{}\" (from {})", name, greeting, service_id);
                } else {
                    println!("   ❌ {}: Request failed: {:?}", name, response.status);
                }
            }
            Err(e) => {
                println!("   ⚠️  {}: Communication error: {}", name, e);
                println!("      (This is expected since we're using mock communication)");
            }
        }
    }

    // Show request metrics
    println!("\n📈 Request Metrics:");
    let request_metrics = orchestrator.get_request_metrics();
    let total = request_metrics
        .total_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let successful = request_metrics
        .successful_requests
        .load(std::sync::atomic::Ordering::Relaxed);
    let failed = request_metrics
        .failed_requests
        .load(std::sync::atomic::Ordering::Relaxed);

    println!("   Total requests: {}", total);
    println!("   Successful: {}", successful);
    println!("   Failed: {}", failed);

    // Show service health
    println!("\n🏥 Service Health Status:");
    for service in &services {
        match orchestrator.get_service_health(&service.id).await {
            Ok(health) => println!("   {} health: {:?}", service.id, health),
            Err(e) => println!("   {} health check failed: {}", service.id, e),
        }
    }

    println!("\n🎯 Example completed successfully!");
    println!("   ✅ Orchestrator setup and configuration");
    println!("   ✅ Service registration and discovery");
    println!("   ✅ Request routing infrastructure");
    println!("   ✅ Load balancing and metrics");
    println!("   ✅ Health monitoring");

    Ok(())
}
