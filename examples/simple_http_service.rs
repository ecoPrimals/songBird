use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use songbird_gaming_bridge::prelude::*; // All core types and traits
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

/// A simple HTTP service that provides various endpoints
#[derive(Clone)]
struct SimpleHttpService {
    id: String,
    counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl SimpleHttpService {
    fn new(id: String) -> Self {
        Self {
            id,
            counter: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleConfig {
    enabled: bool,
    max_requests: u64,
}

#[derive(Debug, Clone, Serialize)]
struct SimpleHealth {
    status: String,
    uptime_seconds: u64,
    request_count: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("Simple service error: {message}")]
struct SimpleError {
    message: String,
}

#[async_trait]
impl UniversalService for SimpleHttpService {
    type Config = SimpleConfig;
    type Health = SimpleHealth;
    type Error = SimpleError;

    async fn initialize(&mut self, _config: Self::Config) -> std::result::Result<(), Self::Error> {
        println!("✅ Simple HTTP Service initialized");
        Ok(())
    }

    async fn start(&mut self) -> std::result::Result<(), Self::Error> {
        println!("🚀 Simple HTTP Service started");
        Ok(())
    }

    async fn stop(&mut self) -> std::result::Result<(), Self::Error> {
        println!("🛑 Simple HTTP Service stopped");
        Ok(())
    }

    async fn health_check(&self) -> std::result::Result<Self::Health, Self::Error> {
        Ok(SimpleHealth {
            status: "healthy".to_string(),
            uptime_seconds: 300,
            request_count: self.counter.load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> std::result::Result<ServiceResponse, Self::Error> {
        // Increment request counter
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        println!(
            "🌐 {} handling HTTP request: {} {}",
            self.id, request.method, request.path
        );

        // Handle different endpoints
        let response_data = match request.path.as_str() {
            "/api/hello" => {
                let name = request
                    .body
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("World");

                serde_json::json!({
                    "message": format!("Hello, {}!", name),
                    "service": self.id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "method": request.method
                })
            }
            "/api/echo" => {
                serde_json::json!({
                    "echo": request.body,
                    "headers": request.headers,
                    "service": self.id,
                    "path": request.path
                })
            }
            "/api/status" => {
                serde_json::json!({
                    "service": self.id,
                    "status": "running",
                    "requests_handled": self.counter.load(std::sync::atomic::Ordering::Relaxed),
                    "uptime": "5 minutes"
                })
            }
            "/api/data" => {
                // Simulate some data processing
                let count = request
                    .body
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10);

                let data: Vec<serde_json::Value> = (1..=count)
                    .map(|i| {
                        serde_json::json!({
                            "id": i,
                            "value": format!("Item {}", i),
                            "generated_at": chrono::Utc::now().to_rfc3339()
                        })
                    })
                    .collect();

                serde_json::json!({
                    "data": data,
                    "total": count,
                    "service": self.id
                })
            }
            _ => {
                return Ok(ServiceResponse {
                    request_id: request.id,
                    status: ResponseStatus::Error {
                        code: 404,
                        message: format!("Endpoint not found: {}", request.path),
                    },
                    headers: HashMap::new(),
                    body: serde_json::json!({
                        "error": "Not Found",
                        "path": request.path,
                        "available_endpoints": ["/api/hello", "/api/echo", "/api/status", "/api/data"]
                    }),
                    timestamp: Utc::now(),
                    processing_time: std::time::Duration::from_millis( std::time::Duration::from_millis(1),
                    processing_time: std::time::Duration::from_millis(1),
                    
                });
            }
        };

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: response_data,
            timestamp: Utc::now(),
            processing_time: std::time::Duration::from_millis( std::time::Duration::from_millis(5),
            processing_time: std::time::Duration::from_millis(3),
            
        })
    }

    async fn update_config(
        &mut self,
        _config: Self::Config,
    ) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    async fn get_metrics(&self) -> std::result::Result<ServiceMetrics, Self::Error> {
        let mut metrics = ServiceMetrics::default();
        metrics.request_count = self.counter.load(std::sync::atomic::Ordering::Relaxed);
        metrics.uptime_seconds = 300; // Mock uptime
        Ok(metrics)
    }

    async fn can_handle_load(&self) -> std::result::Result<bool, Self::Error> {
        Ok(true)
    }

    async fn get_load_factor(&self) -> std::result::Result<f64, Self::Error> {
        Ok(0.2) // 20% load
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_id: self.id.clone(),
            name: format!("Simple HTTP Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: "http-api".to_string(),
            description: Some("A simple HTTP service demonstrating various endpoints").to_string(),
            endpoints: vec![
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/api/hello".to_string(),
                    method: "POST".to_string(),
                    description: Some("Say hello to a name").to_string(),
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
                            "message": {"type": "string"},
                            "service": {"type": "string"},
                            "timestamp": {"type": "string"}
                        }
                    })),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/api/echo".to_string(),
                    method: "POST".to_string(),
                    description: Some("Echo back the request data").to_string(),
                    parameters: vec![],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "echo": {"type": "object"},
                            "service": {"type": "string"}
                        }
                    })),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/api/status".to_string(),
                    method: "GET".to_string(),
                    description: Some("Get service status").to_string(),
                    parameters: vec![],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "status": {"type": "string"},
                            "requests_handled": {"type": "number"}
                        }
                    })),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/api/data".to_string(),
                    method: "POST".to_string(),
                    description: Some("Generate sample data").to_string(),
                    parameters: vec![EndpointParameter {
                        name: "count".to_string(),
                        param_type: "number".to_string(),
                        required: false,
                        description: Some("Number of items to generate").to_string(),
                        default: Some(serde_json::json!(10)),
                    }],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "data": {"type": "array"},
                            "total": {"type": "number"}
                        }
                    })),
                },
            ],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🎼 Songbird Orchestrator - HTTP Service Example");
    println!("===============================================");

    // Create HTTP service
    let service = SimpleHttpService::new("simple-http".to_string());
    let _config = SimpleConfig {
        enabled: true,
        max_requests: 1000,
    };

    // Start HTTP server on localhost:3000
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;

    println!("\n🚀 Starting HTTP service on http://{}", addr);
    println!("Available endpoints:");
    println!("  GET  http://{}/health      - Health check", addr);
    println!("  GET  http://{}/metrics     - Service metrics", addr);
    println!("  GET  http://{}/info        - Service information", addr);
    println!(
        "  POST http://{}/api/hello   - Say hello (JSON: {{\"name\": \"Alice\"}})",
        addr
    );
    println!("  POST http://{}/api/echo    - Echo request data", addr);
    println!("  GET  http://{}/api/status  - Service status", addr);
    println!(
        "  POST http://{}/api/data    - Generate data (JSON: {{\"count\": 5}})",
        addr
    );

    println!("\n🔧 Test commands:");
    println!("  curl http://{}/health", addr);
    println!("  curl -X POST http://{}/api/hello -H 'Content-Type: application/json' -d '{{\"name\": \"Alice\"}}'", addr);
    println!("  curl -X POST http://{}/api/data -H 'Content-Type: application/json' -d '{{\"count\": 3}}'", addr);

    // Use the HttpServiceExt trait to serve HTTP
    service.serve_http(addr).await?;

    Ok(())
}
