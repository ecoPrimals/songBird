use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};

use songbird_orchestrator::config::OrchestratorConfig;
use songbird_orchestrator::orchestrator::Orchestrator;
use songbird_orchestrator::traits::service::{
    ServiceRequest, ServiceResponse, ResponseStatus, UniversalService, 
    ServiceInfo, ServiceEndpoint, ServiceMetrics
};
use songbird_orchestrator::errors::SongbirdError;

/// A real HTTP service that runs an actual HTTP server
#[derive(Clone)]
pub struct RealHttpService {
    id: String,
    name: String,
    port: u16,
    server_handle: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl RealHttpService {
    pub fn new(id: String, port: u16) -> Self {
        Self {
            name: format!("Real HTTP Service {}", id),
            id,
            port,
            server_handle: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealHttpConfig {
    pub port: u16,
    pub bind_address: String,
}

// HTTP request/response types for the actual service
#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
    timestamp: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    echo: String,
    service: String,
    port: u16,
    processed_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    port: u16,
    uptime: String,
}

#[async_trait::async_trait]
impl UniversalService for RealHttpService {
    type Config = RealHttpConfig;
    type Health = String;
    type Error = SongbirdError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.port = config.port;
        println!("🔧 Initializing {} on {}:{}", self.name, config.bind_address, self.port);
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("🚀 Starting {} HTTP server on port {}", self.name, self.port);
        
        // Create HTTP server
        let app = self.create_http_app();
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| SongbirdError::Service {
                message: format!("Service startup failed: {}", e),
            })?;

        let service_name = self.name.clone();
        let service_port = self.port;
        
        // Start server in background
        let handle = tokio::spawn(async move {
            println!("✅ {} is listening on {}", service_name, addr);
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("❌ HTTP server error for {}: {}", service_name, e);
            }
        });

        // Store handle for cleanup
        *self.server_handle.lock().await = Some(handle);
        
        // Give server time to start
        sleep(Duration::from_millis(100)).await;
        
        println!("✅ {} is now running and ready to accept connections", self.name);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        println!("🛑 Stopping {} HTTP server", self.name);
        
        if let Some(handle) = self.server_handle.lock().await.take() {
            handle.abort();
        }
        
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health, Self::Error> {
        Ok(format!("healthy on port {}", self.port))
    }

    async fn handle_request(&self, request: ServiceRequest) -> Result<ServiceResponse, Self::Error> {
        // This method is for internal orchestrator communication
        // The actual HTTP endpoints are handled by the Axum server
        println!("📨 {} received internal request: {} {}", self.name, request.method, request.path);
        
        let response_payload = serde_json::json!({
            "service": self.name,
            "port": self.port,
            "message": "Internal request handled",
            "request_id": request.id
        });

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            payload: response_payload,
            timestamp: chrono::Utc::now(),
            duration: Duration::from_millis(5),
            processing_time: 5,
            metadata: HashMap::new(),
        })
    }

    async fn update_config(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.port = config.port;
        println!("🔧 Updated config for {}", self.name);
        Ok(())
    }

    async fn can_handle_load(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    async fn get_load_factor(&self) -> Result<f64, Self::Error> {
        Ok(0.2) // 20% load
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            service_type: "http".to_string(),
            description: "Real HTTP service with actual endpoints".to_string(),
            endpoints: vec![
                ServiceEndpoint {
                    path: format!("http://127.0.0.1:{}/echo", self.port),
                    method: "POST".to_string(),
                    description: "Echo endpoint".to_string(),
                    parameters: Vec::new(),
                    response_schema: None,
                },
                ServiceEndpoint {
                    path: format!("http://127.0.0.1:{}/health", self.port),
                    method: "GET".to_string(),
                    description: "Health check endpoint".to_string(),
                    parameters: Vec::new(),
                    response_schema: None,
                }
            ],
            capabilities: vec!["http".to_string(), "echo".to_string(), "health".to_string()],
            tags: {
                let mut tags = HashMap::new();
                tags.insert("type".to_string(), "real-http".to_string());
                tags.insert("port".to_string(), self.port.to_string());
                tags
            },
            metadata: HashMap::new(),
        }
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error> {
        Ok(ServiceMetrics::default())
    }
}

impl RealHttpService {
    /// Create the actual HTTP application with real endpoints
    fn create_http_app(&self) -> Router {
        let service_name = self.name.clone();
        let service_port = self.port;
        let service_id = self.id.clone();

        Router::new()
            .route("/health", get({
                let service_name = service_name.clone();
                move || async move {
                    let response = HealthResponse {
                        status: "healthy".to_string(),
                        service: service_name,
                        port: service_port,
                        uptime: "unknown".to_string(),
                    };
                    (StatusCode::OK, ResponseJson(response))
                }
            }))
            .route("/echo", post({
                let service_name = service_name.clone();
                move |Json(payload): Json<EchoRequest>| async move {
                    let response = EchoResponse {
                        echo: payload.message,
                        service: service_name,
                        port: service_port,
                        processed_at: chrono::Utc::now().timestamp(),
                    };
                    (StatusCode::OK, ResponseJson(response))
                }
            }))
            .route("/info", get({
                let service_name = service_name.clone();
                let service_id = service_id.clone();
                move || async move {
                    let info = serde_json::json!({
                        "service_id": service_id,
                        "service_name": service_name,
                        "port": service_port,
                        "status": "running",
                        "endpoints": ["/health", "/echo", "/info"]
                    });
                    (StatusCode::OK, ResponseJson(info))
                }
            }))
    }
}

/// Demo function to test real service communication
async fn test_real_communication() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎼 Testing Real HTTP Service Communication");
    println!("========================================");
    
    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).await?;
    
    // Start orchestrator
    orchestrator.start().await?;
    
    // Create and register real HTTP services
    let service1 = RealHttpService::new("http-service-1".to_string(), 9001);
    let config1 = RealHttpConfig {
        port: 9001,
        bind_address: "127.0.0.1".to_string(),
    };
    
    let service2 = RealHttpService::new("http-service-2".to_string(), 9002);
    let config2 = RealHttpConfig {
        port: 9002,
        bind_address: "127.0.0.1".to_string(),
    };
    
    println!("\n📝 Registering real HTTP services...");
    let service1_id = orchestrator.register_service(service1, config1).await?;
    let service2_id = orchestrator.register_service(service2, config2).await?;
    
    // Wait for services to fully start
    sleep(Duration::from_secs(2)).await;
    
    // Test direct HTTP communication to services
    println!("\n🌐 Testing direct HTTP communication...");
    
    let client = reqwest::Client::new();
    
    // Test service 1 health
    match client.get("http://127.0.0.1:9001/health").send().await {
        Ok(response) => {
            let body: serde_json::Value = response.json().await?;
            println!("✅ Service 1 health: {}", body);
        }
        Err(e) => println!("❌ Service 1 health failed: {}", e),
    }
    
    // Test service 2 echo
    let echo_request = EchoRequest {
        message: "Hello from test client!".to_string(),
        timestamp: Some(chrono::Utc::now().timestamp()),
    };
    
    match client.post("http://127.0.0.1:9002/echo")
        .json(&echo_request)
        .send().await {
            Ok(response) => {
                let body: serde_json::Value = response.json().await?;
                println!("✅ Service 2 echo: {}", body);
            }
            Err(e) => println!("❌ Service 2 echo failed: {}", e),
    }
    
    // Now test orchestrator communication
    println!("\n🎼 Testing orchestrator-mediated communication...");
    
    // This will test if the orchestrator can route requests to our real HTTP services
    let request = ServiceRequest {
        id: "test-orchestrator-request".to_string(),
        method: "POST".to_string(),
        path: "/echo".to_string(),
        headers: HashMap::new(),
        payload: serde_json::json!({
            "message": "Hello through orchestrator!",
            "timestamp": chrono::Utc::now().timestamp()
        }),
        timestamp: chrono::Utc::now(),
        timeout: Some(Duration::from_secs(10)),
        client_info: None,
        metadata: HashMap::new(),
    };
    
    match orchestrator.handle_service_request(&service1_id, request).await {
        Ok(response) => {
            println!("✅ Orchestrator communication successful: {:?}", response.status);
            println!("   Response: {}", response.payload);
        }
        Err(e) => {
            println!("⚠️  Orchestrator communication failed: {}", e);
            println!("   This is expected - we need to wire the communication layer to our HTTP endpoints");
        }
    }
    
    // Show metrics
    let metrics = orchestrator.get_metrics().await;
    println!("\n📊 Final Metrics:");
    println!("   - Total services: {}", metrics.total_services);
    println!("   - Healthy services: {}", metrics.healthy_services);
    
    // Cleanup
    println!("\n🧹 Cleaning up...");
    orchestrator.unregister_service(&service1_id).await?;
    orchestrator.unregister_service(&service2_id).await?;
    
    println!("✅ Test completed!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_real_communication().await
} 