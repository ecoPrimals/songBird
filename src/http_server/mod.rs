//! HTTP Server Module
//!
//! Provides HTTP server functionality for services to expose endpoints

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
    body::Bytes,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

use crate::traits::service::{ServiceRequest, UniversalService};
use crate::errors::{Result, SongbirdError};

/// HTTP server for Universal Services
pub struct HttpServiceServer<S> 
where
    S: UniversalService + Clone + Send + Sync + 'static,
{
    service: Arc<S>,
    addr: SocketAddr,
}

/// HTTP request context
#[derive(Debug, Clone)]
pub struct HttpRequestContext {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

/// Standard HTTP response format
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServiceResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
}

impl<S> HttpServiceServer<S>
where
    S: UniversalService + Clone + Send + Sync + 'static,
    S::Error: std::fmt::Display,
{
    /// Create a new HTTP server for a service
    pub fn new(service: S, addr: SocketAddr) -> Self {
        Self {
            service: Arc::new(service),
            addr,
        }
    }

    /// Build the router with service endpoints
    pub fn build_router(&self) -> Router {
        let service_info = self.service.service_info();
        
        // Create base router
        let mut router = Router::new();

        // Add health check endpoint
        router = router.route("/health", get(health_handler::<S>));

        // Add metrics endpoint
        router = router.route("/metrics", get(metrics_handler::<S>));

        // Add service info endpoint
        router = router.route("/info", get(info_handler::<S>));

        // Add service-specific endpoints
        for endpoint in &service_info.endpoints {
            let path = endpoint.path.clone();
            
            match endpoint.method.to_lowercase().as_str() {
                "get" => {
                    router = router.route(&path, get(request_handler::<S>));
                }
                "post" => {
                    router = router.route(&path, post(request_handler::<S>));
                }
                "put" => {
                    router = router.route(&path, axum::routing::put(request_handler::<S>));
                }
                "delete" => {
                    router = router.route(&path, axum::routing::delete(request_handler::<S>));
                }
                "patch" => {
                    router = router.route(&path, axum::routing::patch(request_handler::<S>));
                }
                _ => {
                    warn!("Unsupported HTTP method: {} for endpoint: {}", endpoint.method, path);
                }
            }
        }

        // Add middleware and state
        router
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
                    .into_inner()
            )
            .with_state(Arc::clone(&self.service))
    }

    /// Start the HTTP server
    pub async fn start(&self) -> Result<()> {
        let router = self.build_router();
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| SongbirdError::Network { message: e.to_string() })?;

        info!("HTTP server starting on {}", self.addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| SongbirdError::Network { message: e.to_string() })?;

        Ok(())
    }

    /// Get the server address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Health check handler
async fn health_handler<S>(
    State(service): State<Arc<S>>,
) -> std::result::Result<Json<HttpServiceResponse>, StatusCode>
where
    S: UniversalService + Send + Sync,
    S::Error: std::fmt::Display,
{
    let request_id = uuid::Uuid::new_v4().to_string();
    
    match service.health_check().await {
        Ok(health) => {
            Ok(Json(HttpServiceResponse {
                success: true,
                data: Some(serde_json::to_value(health).unwrap_or_else(|_| serde_json::json!({}))),
                error: None,
                timestamp: chrono::Utc::now(),
                request_id,
            }))
        }
        Err(e) => {
            Ok(Json(HttpServiceResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                timestamp: chrono::Utc::now(),
                request_id,
            }))
        }
    }
}

/// Metrics handler
async fn metrics_handler<S>(
    State(service): State<Arc<S>>,
) -> std::result::Result<Json<HttpServiceResponse>, StatusCode>
where
    S: UniversalService + Send + Sync,
    S::Error: std::fmt::Display,
{
    let request_id = uuid::Uuid::new_v4().to_string();
    
    match service.get_metrics().await {
        Ok(metrics) => {
            Ok(Json(HttpServiceResponse {
                success: true,
                data: Some(serde_json::to_value(metrics).unwrap_or_else(|_| serde_json::json!({}))),
                error: None,
                timestamp: chrono::Utc::now(),
                request_id,
            }))
        }
        Err(e) => {
            Ok(Json(HttpServiceResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                timestamp: chrono::Utc::now(),
                request_id,
            }))
        }
    }
}

/// Service info handler
async fn info_handler<S>(
    State(service): State<Arc<S>>,
) -> Json<HttpServiceResponse>
where
    S: UniversalService + Send + Sync,
{
    let request_id = uuid::Uuid::new_v4().to_string();
    let info = service.service_info();
    
    Json(HttpServiceResponse {
        success: true,
        data: Some(serde_json::to_value(info).unwrap_or_else(|_| serde_json::json!({}))),
        error: None,
        timestamp: chrono::Utc::now(),
        request_id,
    })
}

/// Main request handler
async fn request_handler<S>(
    State(service): State<Arc<S>>,
    uri: axum::http::Uri,
    method: axum::http::Method,
    headers: HeaderMap,
    Query(query_params): Query<HashMap<String, String>>,
    body: Bytes,
) -> std::result::Result<Json<HttpServiceResponse>, StatusCode>
where
    S: UniversalService + Send + Sync,
    S::Error: std::fmt::Display,
{
    let request_id = uuid::Uuid::new_v4().to_string();

    // Convert headers to HashMap
    let mut header_map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(name.to_string(), value_str.to_string());
        }
    }

    // Parse request body as JSON
    let payload = if body.is_empty() {
        serde_json::json!({})
    } else {
        // Convert bytes to string first
        let body_str = String::from_utf8_lossy(&body);
        serde_json::from_str(&body_str).unwrap_or_else(|_| serde_json::json!({"raw": body_str}))
    };

    // Add query parameters to payload
    let mut enhanced_payload = payload;
    if enhanced_payload.is_object() {
        if !query_params.is_empty() {
            enhanced_payload["query_params"] = serde_json::to_value(query_params).unwrap_or_else(|_| serde_json::json!({}));
        }
    }

    // Create ServiceRequest
    let service_request = ServiceRequest {
        id: request_id.clone(),
        method: method.to_string(),
        path: uri.path().to_string(),
        headers: header_map,
        payload: enhanced_payload,
        timestamp: chrono::Utc::now(),
        timeout: Some(std::time::Duration::from_secs(30)),
        client_info: None,
        metadata: HashMap::new(),
    };

    // Handle the request
    match service.handle_request(service_request).await {
        Ok(response) => {
            let success = matches!(response.status, crate::traits::service::ResponseStatus::Success);
            
            Ok(Json(HttpServiceResponse {
                success,
                data: Some(response.payload),
                error: if success { None } else { 
                    match response.status {
                        crate::traits::service::ResponseStatus::Error { message, .. } => Some(message),
                        _ => Some("Request failed".to_string()),
                    }
                },
                timestamp: response.timestamp,
                request_id: response.request_id,
            }))
        }
        Err(e) => {
            error!("Service request failed: {}", e);
            Ok(Json(HttpServiceResponse {
                success: false,
                data: None,
                error: Some(e.to_string()),
                timestamp: chrono::Utc::now(),
                request_id,
            }))
        }
    }
}

/// Helper trait to add HTTP server capability to any UniversalService
#[async_trait::async_trait]
pub trait HttpServiceExt: UniversalService + Clone + Send + Sync + 'static 
where
    Self::Error: std::fmt::Display,
{
    /// Create and start an HTTP server for this service
    async fn serve_http(&self, addr: SocketAddr) -> Result<()> {
        let server = HttpServiceServer::new(self.clone(), addr);
        server.start().await
    }

    /// Create an HTTP server without starting it
    fn create_http_server(&self, addr: SocketAddr) -> HttpServiceServer<Self> {
        HttpServiceServer::new(self.clone(), addr)
    }
}

// Blanket implementation for all UniversalServices
impl<T> HttpServiceExt for T 
where 
    T: UniversalService + Clone + Send + Sync + 'static,
    T::Error: std::fmt::Display,
{} 