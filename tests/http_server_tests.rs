use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
#[allow(dead_code, unused_imports, unused_variables)]
use songbird_gaming_bridge::{
    http_server::{HttpServiceExt, HttpServiceResponse, HttpServiceServer},
    traits::service_id::{
        ResponseStatus, ServiceEndpoint, ServiceInfo, ServiceMetrics, ServiceRequest,
        ServiceResponse, UniversalService,
    },
};

use reqwest;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

/// Test service for HTTP server testing
#[derive(Clone)]
struct TestHttpService {
    id: String,
    counter: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestConfig {
    enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TestHealth {
    status: String,
    request_count: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("Test service error: {message}")]
struct TestError {
    message: String,
}

impl TestHttpService {
    fn new(id: String) -> Self {
        Self {
            id,
            counter: Arc::new(AtomicU64::new(0)),
        }
    }
}

#[async_trait]
impl UniversalService for TestHttpService {
    type Config = TestConfig;
    type Health = TestHealth;
    type Error = TestError;

    async fn initialize(&mut self, _config: Self::Config) -> Result<()> {
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn health_check(&self) -> Result<Self::Health> {
        Ok(TestHealth {
            status: "healthy".to_string(),
            request_count: self.counter.load(Ordering::Relaxed),
        })
    }

    async fn handle_request(
        &self,
        request: ServiceRequest,
    ) -> Result<ServiceResponse> {
        self.counter.fetch_add(1, Ordering::Relaxed);

        let response_data = match request.path.as_str() {
            "/test/echo" => {
                serde_json::json!({
                    "echo": request.body,
                    "service": self.id,
                    "method": request.method
                })
            }
            "/test/counter" => {
                serde_json::json!({
                    "counter": self.counter.load(Ordering::Relaxed),
                    "service": self.id
                })
            }
            "/test/error" => {
                return Ok(ServiceResponse {
                    request_id: request.id,
                    status: ResponseStatus::Error {
                        code: 400,
                        message: "Test error".to_string(),
                    },
                    headers: HashMap::new(),
                    body: serde_json::json!({"error": "Intentional test error"}),
                    timestamp: chrono::Utc::now(),
                    processing_time: Duration::from_millis(1),
                    processing_time: std::time::Duration::from_millis(1),
                    
                });
            }
            _ => {
                return Ok(ServiceResponse {
                    request_id: request.id,
                    status: ResponseStatus::Error {
                        code: 404,
                        message: "Endpoint not found".to_string(),
                    },
                    headers: HashMap::new(),
                    body: serde_json::json!({"error": "Not Found"}),
                    timestamp: chrono::Utc::now(),
                    processing_time: Duration::from_millis(1),
                    processing_time: std::time::Duration::from_millis(1),
                    
                });
            }
        };

        Ok(ServiceResponse {
            request_id: request.id,
            status: ResponseStatus::Success,
            headers: HashMap::new(),
            body: response_data,
            timestamp: chrono::Utc::now(),
            processing_time: Duration::from_millis(5),
            processing_time: std::time::Duration::from_millis(3),
            
        })
    }

    async fn update_config(&mut self, _config: Self::Config) -> Result<()> {
        Ok(())
    }

    async fn get_metrics(&self) -> Result<ServiceMetrics> {
        let mut metrics = ServiceMetrics::default();
        metrics.request_count = self.counter.load(Ordering::Relaxed);
        Ok(metrics)
    }

    async fn can_handle_load(&self) -> Result<bool> {
        Ok(true)
    }

    async fn get_load_factor(&self) -> Result<f64> {
        Ok(0.1)
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            service_service_id: self.id.clone(),
            name: format!("Test HTTP Service {}", self.id),
            version: "1.0.0".to_string(),
            service_type: "http-test".to_string(),
            description: Some("Test service for HTTP server testing").to_string(),
            endpoints: vec![
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/test/echo".to_string(),
                    method: "POST".to_string(),
                    description: Some("Echo back the request").to_string(),
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
                    path: "/test/counter".to_string(),
                    method: "GET".to_string(),
                    description: Some("Get request counter").to_string(),
                    parameters: vec![],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "counter": {"type": "number"},
                            "service": {"type": "string"}
                        }
                    })),
                },
                ServiceEndpoint {
            auth_required: false,
            rate_limit: None,
                    path: "/test/error".to_string(),
                    method: "GET".to_string(),
                    description: Some("Trigger a test error").to_string(),
                    parameters: vec![],
                    response_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "error": {"type": "string"}
                        }
                    })),
                },
            ],
            tags: std::collections::HashMap::new(),
            tags: HashMap::new(),
            
        }
    }
}

/// Test helper to find available port with global synchronization
static PORT_COUNTER: Mutex<u16> = Mutex::new(8000);

async fn find_available_port() -> u16 {
    let mut port = PORT_COUNTER.lock().expect("Test assertion failed");
    loop {
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", *port))
            .await
            .is_ok()
        {
            let result = *port;
            *port += 1;
            return result;
        }
        *port += 1;
        if *port > 9000 {
            *port = 8000;
        }
    }
}

/// Test helper to start HTTP server and return client
async fn start_test_server() -> (u16, reqwest::Client) {
    let port = find_available_port().await;
    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("Test assertion failed");
    let service = TestHttpService::new("test-service".to_string());

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = service.serve_http(addr).await {
            eprintln!("Server error on port {}: {}", addr.port(), e);
        }
    });

    // Wait longer for server to start and verify it's actually running
    sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Test assertion failed");

    // Verify server is actually running by attempting connection
    for _ in 0..10 {
        if client
            .get(&format!("http://127.0.0.1:{}/health", port))
            .send()
            .await
            .is_ok()
        {
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }

    (port, client)
}

#[tokio::test]
async fn test_http_server_health_endpoint() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/health", port);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);
    assert!(json.data.is_some());
    assert!(json.error.is_none());
    assert!(!json.request_id.is_empty());

    // Check that health data contains expected fields
    let health_data = json.data.expect("Test assertion failed");
    assert!(health_data.get("status").is_some());
    assert!(health_data.get("request_count").is_some());
}

#[tokio::test]
async fn test_http_server_metrics_endpoint() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/metrics", port);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);
    assert!(json.data.is_some());
    assert!(json.error.is_none());

    // Check that metrics data contains expected fields
    let metrics_data = json.data.expect("Test assertion failed");
    assert!(metrics_data.get("request_count").is_some());
}

#[tokio::test]
async fn test_http_server_info_endpoint() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/info", port);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);
    assert!(json.data.is_some());
    assert!(json.error.is_none());

    // Check that service info contains expected fields
    let info_data = json.data.expect("Test assertion failed");
    assert_eq!(
        info_data
            .get("id")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "test-service"
    );
    assert_eq!(
        info_data
            .get("service_type")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "http-test"
    );
    assert!(info_data.get("endpoints").is_some());
}

#[tokio::test]
async fn test_http_server_custom_endpoint_post() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/test/echo", port);

    let test_data = serde_json::json!({
        "message": "hello",
        "number": 42
    });

    let response = client
        .post(&url)
        .json(&test_data)
        .send()
        .await
        .expect("Test assertion failed");

    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);
    assert!(json.data.is_some());
    assert!(json.error.is_none());

    // Check that echo response contains our data
    let response_data = json.data.expect("Test assertion failed");
    assert_eq!(
        response_data
            .get("service")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "test-service"
    );
    assert_eq!(
        response_data
            .get("method")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "POST"
    );
    assert!(response_data.get("echo").is_some());
}

#[tokio::test]
async fn test_http_server_custom_endpoint_get() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/test/counter", port);

    // Make first request
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);

    let response_data = json.data.expect("Test assertion failed");
    let counter1 = response_data
        .get("counter")
        .expect("Test assertion failed")
        .as_u64()
        .expect("Test assertion failed");

    // Make second request - counter should increment
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    let response_data = json.data.expect("Test assertion failed");
    let counter2 = response_data
        .get("counter")
        .expect("Test assertion failed")
        .as_u64()
        .expect("Test assertion failed");

    assert!(counter2 > counter1);
}

#[tokio::test]
async fn test_http_server_error_handling() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/test/error", port);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 200); // HTTP layer is OK, but service returns error

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(!json.success); // Service-level error
    assert!(json.data.is_some());
    assert!(json.error.is_some());

    let error = json.error.expect("Test assertion failed");
    assert!(error.contains("Test error"));
}

#[tokio::test]
async fn test_http_server_404_endpoint() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/nonexistent", port);

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    assert_eq!(response.status(), 404); // Axum router 404, not service 404
}

#[tokio::test]
async fn test_http_server_query_parameters() {
    let (port, client) = start_test_server().await;
    let url = format!(
        "http://127.0.0.1:{}/test/echo?param1=value1&param2=value2",
        port
    );

    let test_data = serde_json::json!({
        "body_field": "test"
    });

    let response = client
        .post(&url)
        .json(&test_data)
        .send()
        .await
        .expect("Test assertion failed");

    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);

    let response_data = json.data.expect("Test assertion failed");
    let echo = response_data.get("echo").expect("Test assertion failed");

    // Query parameters should be included in the payload
    assert!(echo.get("query_params").is_some());
    let query_params = echo.get("query_params").expect("Test assertion failed");
    assert_eq!(
        query_params
            .get("param1")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "value1"
    );
    assert_eq!(
        query_params
            .get("param2")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "value2"
    );
}

#[tokio::test]
async fn test_http_server_request_id_tracking() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/health", port);

    // Make multiple requests
    let response1 = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    let json1: HttpServiceResponse = response1.json().await.expect("Test assertion failed");

    let response2 = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    let json2: HttpServiceResponse = response2.json().await.expect("Test assertion failed");

    // Request IDs should be different
    assert_ne!(json1.request_id, json2.request_id);
    assert!(!json1.request_id.is_empty());
    assert!(!json2.request_id.is_empty());
}

#[tokio::test]
async fn test_http_server_content_type_handling() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/test/echo", port);

    // Test with JSON content type
    let test_data = r#"{"message": "json test"}"#;

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(test_data)
        .send()
        .await
        .expect("Test assertion failed");

    assert_eq!(response.status(), 200);

    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    assert!(json.success);

    let response_data = json.data.expect("Test assertion failed");
    let echo = response_data.get("echo").expect("Test assertion failed");
    assert_eq!(
        echo.get("message")
            .expect("Test assertion failed")
            .as_str()
            .expect("Test assertion failed"),
        "json test"
    );
}

#[tokio::test]
async fn test_http_service_server_creation() {
    let service = TestHttpService::new("test".to_string());
    let addr: SocketAddr = "127.0.0.1:0".parse().expect("Test assertion failed"); // Use port 0 for testing

    let server = HttpServiceServer::new(service.clone(), addr);
    assert_eq!(server.addr(), addr);

    // Test creating via trait
    let server2 = service.create_http_server(addr);
    assert_eq!(server2.addr(), addr);
}

#[tokio::test]
async fn test_http_server_concurrent_requests() {
    let (port, client) = start_test_server().await;
    let url = format!("http://127.0.0.1:{}/test/counter", port);

    // Make multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let url = url.clone();
        let handle = tokio::spawn(async move {
            client
                .get(&url)
                .send()
                .await
                .expect("Test assertion failed")
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.expect("Test assertion failed");
        assert_eq!(response.status(), 200);
    }

    // Final counter check - should be at least 10
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Test assertion failed");
    let json: HttpServiceResponse = response.json().await.expect("Test assertion failed");
    let response_data = json.data.expect("Test assertion failed");
    let counter = response_data
        .get("counter")
        .expect("Test assertion failed")
        .as_u64()
        .expect("Test assertion failed");

    assert!(counter >= 10);
}
