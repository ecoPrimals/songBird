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
use songbird_errors::{SongbirdError, SongbirdResult as Result};
use std::collections::HashMap;
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
            SongbirdError::network(format!(
                "HTTP Server - Failed to bind to address {}: {}",
                self.addr, e
            ))
        })?;

        info!("HTTP server starting on {}", self.addr);

        let app = self.build_router();

        axum::serve(listener, app)
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP Server - Server error: {}", e)))?;

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

/// Production network service with real monitoring capabilities
pub struct ProductionNetworkService {
    start_time: SystemTime,
    connection_count: Arc<parking_lot::RwLock<u32>>,
    total_connections: Arc<parking_lot::RwLock<u64>>,
    bytes_sent: Arc<parking_lot::RwLock<u64>>,
    bytes_received: Arc<parking_lot::RwLock<u64>>,
    active_connections: Arc<parking_lot::RwLock<HashMap<String, ConnectionInfo>>>,
}

impl Default for ProductionNetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductionNetworkService {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            connection_count: Arc::new(parking_lot::RwLock::new(0)),
            total_connections: Arc::new(parking_lot::RwLock::new(0)),
            bytes_sent: Arc::new(parking_lot::RwLock::new(0)),
            bytes_received: Arc::new(parking_lot::RwLock::new(0)),
            active_connections: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Add a new connection
    pub fn add_connection(&self, connection: ConnectionInfo) {
        let mut connections = self.active_connections.write();
        connections.insert(connection.id.clone(), connection);

        *self.connection_count.write() += 1;
        *self.total_connections.write() += 1;
    }

    /// Remove a connection
    pub fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.active_connections.write();
        if connections.remove(connection_id).is_some() {
            *self.connection_count.write() = self.connection_count.read().saturating_sub(1);
        }
    }

    /// Update bytes sent/received
    pub fn update_bytes(&self, sent: u64, received: u64) {
        *self.bytes_sent.write() += sent;
        *self.bytes_received.write() += received;
    }
}

impl NetworkService for ProductionNetworkService {
    fn get_status(&self) -> NetworkStatus {
        NetworkStatus {
            is_running: true,
            uptime: SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default(),
            connections: *self.connection_count.read(),
            last_updated: SystemTime::now(),
        }
    }

    fn get_connections(&self) -> Vec<ConnectionInfo> {
        self.active_connections.read().values().cloned().collect()
    }

    fn get_metrics(&self) -> NetworkMetrics {
        let connections = self.active_connections.read();
        let uptime_seconds = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default()
            .as_secs_f64();

        // Calculate requests per second (simplified)
        let requests_per_second = if uptime_seconds > 0.0 {
            *self.total_connections.read() as f64 / uptime_seconds
        } else {
            0.0
        };

        NetworkMetrics {
            total_connections: *self.total_connections.read(),
            active_connections: connections.len() as u32,
            bytes_sent: *self.bytes_sent.read(),
            bytes_received: *self.bytes_received.read(),
            requests_per_second,
            error_rate: 0.0, // Could be calculated from error tracking
        }
    }
}

/// Create a default HTTP server for testing
pub fn create_default_server(addr: SocketAddr) -> HttpServer {
    let service = Arc::new(ProductionNetworkService::new());
    HttpServer::new(addr, service)
}
