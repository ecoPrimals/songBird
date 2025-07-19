//! HTTP Server Module
//!
//! Provides HTTP server functionality for services to expose endpoints

use axum::{
    extract::State,
    http::Method,
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use songbird_errors::{NetworkError, Result, SongbirdError};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// HTTP server for Songbird networking
pub struct HttpServer {
    addr: SocketAddr,
    service: Arc<dyn NetworkService>,
}

/// Network service trait for dependency injection
pub trait NetworkService: Send + Sync {
    fn get_status(&self) -> NetworkStatus;
    fn get_connections(&self) -> Vec<ConnectionInfo>;
    fn get_metrics(&self) -> NetworkMetrics;
}

/// Network status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub is_running: bool,
    pub uptime: Duration,
    pub connections: u32,
    pub last_updated: SystemTime,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub remote_addr: SocketAddr,
    pub connected_at: SystemTime,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_connections: u64,
    pub active_connections: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub requests_per_second: f64,
    pub error_rate: f64,
}

impl HttpServer {
    /// Create new HTTP server
    pub fn new(addr: SocketAddr, service: Arc<dyn NetworkService>) -> Self {
        Self { addr, service }
    }

    /// Build the router with all routes
    pub fn build_router(&self) -> Router {
        Router::new()
            .route("/", get(root_handler))
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            .route("/connections", get(connections_handler))
            .route("/metrics", get(metrics_handler))
            .route("/api/v1/network/status", get(api_status_handler))
            .route("/api/v1/network/connections", get(api_connections_handler))
            .route("/api/v1/network/metrics", get(api_metrics_handler))
            .layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST])
                    .allow_origin(Any)
                    .allow_headers(Any),
            )
            .with_state(Arc::clone(&self.service))
    }

    /// Start the HTTP server
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                message: format!(
                    "HTTP Server - Failed to bind to address {}: {}",
                    self.addr, e
                ),
                endpoint: Some(self.addr.to_string()),
                port: Some(self.addr.port()),
                protocol: Some("HTTP".to_string()),
            }))
        })?;

        info!("HTTP server starting on {}", self.addr);

        let app = self.build_router();

        axum::serve(listener, app).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                message: format!("HTTP Server - Server error: {}", e),
                endpoint: Some(self.addr.to_string()),
                port: Some(self.addr.port()),
                protocol: Some("HTTP".to_string()),
            }))
        })?;

        Ok(())
    }
}

/// Root handler
async fn root_handler() -> Html<&'static str> {
    Html("<h1>Songbird Network Server</h1><p>Server is running</p>")
}

/// Health check handler
async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

/// Status handler
async fn status_handler(State(service): State<Arc<dyn NetworkService>>) -> Json<NetworkStatus> {
    Json(service.get_status())
}

/// Connections handler
async fn connections_handler(
    State(service): State<Arc<dyn NetworkService>>,
) -> Json<Vec<ConnectionInfo>> {
    Json(service.get_connections())
}

/// Metrics handler
async fn metrics_handler(State(service): State<Arc<dyn NetworkService>>) -> Json<NetworkMetrics> {
    Json(service.get_metrics())
}

/// API status handler
async fn api_status_handler(
    State(service): State<Arc<dyn NetworkService>>,
) -> Json<serde_json::Value> {
    let status = service.get_status();
    Json(json!({
        "api_version": "1.0",
        "network_status": status,
        "endpoints": [
            "/api/v1/network/status",
            "/api/v1/network/connections",
            "/api/v1/network/metrics"
        ]
    }))
}

/// API connections handler
async fn api_connections_handler(
    State(service): State<Arc<dyn NetworkService>>,
) -> Json<serde_json::Value> {
    let connections = service.get_connections();
    Json(json!({
        "connections": connections,
        "total_count": connections.len()
    }))
}

/// API metrics handler
async fn api_metrics_handler(
    State(service): State<Arc<dyn NetworkService>>,
) -> Json<serde_json::Value> {
    let metrics = service.get_metrics();
    Json(json!({
        "metrics": metrics,
        "collected_at": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

/// Mock network service for testing
pub struct MockNetworkService {
    start_time: SystemTime,
}

impl MockNetworkService {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
        }
    }
}

impl NetworkService for MockNetworkService {
    fn get_status(&self) -> NetworkStatus {
        NetworkStatus {
            is_running: true,
            uptime: SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default(),
            connections: 5,
            last_updated: SystemTime::now(),
        }
    }

    fn get_connections(&self) -> Vec<ConnectionInfo> {
        vec![
            ConnectionInfo {
                id: "conn_1".to_string(),
                remote_addr: "127.0.0.1:8080".parse().unwrap(),
                connected_at: SystemTime::now(),
                bytes_sent: 1024,
                bytes_received: 512,
            },
            ConnectionInfo {
                id: "conn_2".to_string(),
                remote_addr: "127.0.0.1:8081".parse().unwrap(),
                connected_at: SystemTime::now(),
                bytes_sent: 2048,
                bytes_received: 1024,
            },
        ]
    }

    fn get_metrics(&self) -> NetworkMetrics {
        NetworkMetrics {
            total_connections: 10,
            active_connections: 5,
            bytes_sent: 10240,
            bytes_received: 5120,
            requests_per_second: 15.5,
            error_rate: 0.05,
        }
    }
}

/// Create a default HTTP server for testing
pub fn create_default_server(addr: SocketAddr) -> HttpServer {
    let service = Arc::new(MockNetworkService::new());
    HttpServer::new(addr, service)
}
