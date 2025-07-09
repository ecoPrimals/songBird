//! REST API Layer for Songbird Orchestrator
//!
//! Provides HTTP endpoints for service management, monitoring, and system information

use std::collections::HashMap;
// Module imports
use axum::response::sse::Event;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Sse},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;
use crate::communication::{CommunicationLayer, WebSocketCommunication};
use crate::errors::{Result, SongbirdError};
use crate::orchestrator::{Orchestrator, OrchestratorMetrics, ServiceHealth};
use crate::traits::communication::{
    MessageType, ServiceAddress, ServiceMessage, CommunicationStats,
};
use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceMetrics};
/// API server state containing the orchestrator and communication layer
#[derive(Clone)]
pub struct ApiState {
    pub orchestrator: Arc<Orchestrator>,
    pub websocket: Arc<WebSocketCommunication>,
    pub event_stream: broadcast::Sender<ApiEvent>,
}
/// API events for real-time streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiEvent {
    ServiceStarted {
        service_id: String,
        timestamp: DateTime<Utc>,
    },
    ServiceStopped {
        service_id: String,
        timestamp: DateTime<Utc>,
    },
    ServiceHealthChanged {
        service_id: String,
        health: ServiceHealth,
        timestamp: DateTime<Utc>,
    },
    MetricsUpdate {
        metrics: OrchestratorMetrics,
        timestamp: DateTime<Utc>,
    },
}
/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}
/// Service registration request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterServiceRequest {
    pub name: String,
    pub service_type: String,
    pub version: String,
    pub description: Option<String>,
    pub endpoints: Option<Vec<ServiceEndpoint>>,
    pub capabilities: Option<Vec<String>>,
    pub tags: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
/// Service operation request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceOperationRequest {
    pub operation: String,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}
/// Message send request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendMessageRequest {
    pub target_service: String,
    pub message_type: MessageType,
    pub topic: Option<String>,
    pub payload: serde_json::Value,
    pub headers: Option<HashMap<String, String>>,
    pub ttl: Option<u64>,
}
/// Broadcast message request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastMessageRequest {
    pub message_type: MessageType,
    pub topic: Option<String>,
    pub payload: serde_json::Value,
    pub headers: Option<HashMap<String, String>>,
    pub ttl: Option<u64>,
}
/// System information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub uptime_seconds: u64,
    pub total_services: u64,
    pub healthy_services: u64,
    pub active_connections: u64,
    pub total_requests: u64,
    pub api_endpoints: Vec<String>,
}
/// Dashboard data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub system_info: SystemInfo,
    pub orchestrator_metrics: OrchestratorMetrics,
    pub services: Vec<ServiceInfo>,
    pub communication_stats: CommunicationStats,
    pub recent_events: Vec<ApiEvent>,
}
/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub checks: HashMap<String, String>,
}
/// Query parameters for metrics endpoints
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub service_id: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub interval: Option<String>,
}
impl ApiState {
    pub fn new(orchestrator: Arc<Orchestrator>, websocket: Arc<WebSocketCommunication>) -> Self {
        let (event_stream, _) = broadcast::channel(1000);
        Self {
            orchestrator,
            websocket,
            event_stream,
        }
    }
    /// Broadcast an API event
    pub fn broadcast_event(&self, event: ApiEvent) {
        let _ = self.event_stream.send(event);
    }
}
/// Create the API router with all endpoints
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Health and system endpoints
        .route("/health", get(health_check))
        .route("/health/detailed", get(detailed_health_check))
        .route("/system/info", get(get_system_info))
        .route("/system/metrics", get(get_system_metrics))
        // Service management endpoints
        .route("/services", get(list_services))
        .route("/services", post(register_service))
        .route("/services/:id", get(get_service))
        .route("/services/:id", put(update_service))
        .route("/services/:id", delete(unregister_service))
        .route("/services/:id/start", post(start_service))
        .route("/services/:id/stop", post(stop_service))
        .route("/services/:id/restart", post(restart_service))
        .route("/services/:id/health", get(get_service_health))
        .route("/services/:id/metrics", get(get_service_metrics))
        // Communication endpoints
        .route("/communication/send", post(send_message))
        .route("/communication/broadcast", post(broadcast_message))
        .route("/communication/stats", get(get_communication_stats))
        .route("/communication/connections", get(get_connections))
        // Metrics and monitoring endpoints
        .route("/metrics", get(get_orchestrator_metrics))
        .route("/metrics/prometheus", get(prometheus_metrics))
        .route("/metrics/services", get(get_all_service_metrics))
        // Real-time streams
        .route("/stream/events", get(events_stream))
        .route("/stream/metrics", get(metrics_stream))
        // Dashboard endpoint
        .route("/dashboard", get(get_dashboard_data))
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
/// Start the API server
pub async fn start_server(
    orchestrator: Arc<Orchestrator>,
    websocket: Arc<WebSocketCommunication>,
    bind_addr: SocketAddr,
) -> Result<()> {
    let state = ApiState::new(orchestrator, websocket);
    let app = create_router(state);
    info!("Starting API server on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .map_err(|e| SongbirdError::Network { 
            service: "api-server".to_string(),
            message: e.to_string(),
            details: None 
        })?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("API server error: {}", e);
        }
    });
    Ok(())
}
// Helper functions for responses
pub fn success<T>(data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }),
    )
}

pub fn error<T>(code: StatusCode, message: String) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        code,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }),
    )
}

// Health and system endpoints
async fn health_check() -> (StatusCode, Json<ApiResponse<&'static str>>) {
    success("healthy")
}

async fn detailed_health_check(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<HealthCheckResponse>>) {
    let metrics = state.orchestrator.get_metrics().await;
    // Mock websocket connection status for now
    let websocket_connected = true;
    let mut checks = HashMap::new();
    checks.insert("orchestrator".to_string(), "healthy".to_string());
    checks.insert(
        "websocket".to_string(),
        if websocket_connected {
            "healthy"
        } else {
            "unhealthy"
        }.to_string(),
    );
    checks.insert(
        "services".to_string(),
        format!("{}/{}", metrics.healthy_services, metrics.total_services),
    );
    let response = HealthCheckResponse {
        status: if websocket_connected && metrics.healthy_services == metrics.total_services {
            "healthy"
        } else if websocket_connected || metrics.healthy_services > 0 {
            "degraded"
        } else {
            "unhealthy"
        }.to_string(),
        checks,
    };
    success(response)
}

async fn get_system_info(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<SystemInfo>>) {
    let metrics = state.orchestrator.get_metrics().await;
    // Mock communication stats for now
    let comm_stats = CommunicationStats::default();
    let endpoints = vec![
        "/health".to_string(),
        "/services".to_string(),
        "/metrics".to_string(),
        "/communication/send".to_string(),
        "/dashboard".to_string(),
    ];
    let system_info = SystemInfo {
        uptime_seconds: metrics.uptime_seconds,
        total_services: metrics.total_services,
        healthy_services: metrics.healthy_services,
        active_connections: comm_stats.active_connections,
        total_requests: metrics.total_requests,
        api_endpoints: endpoints,
    };
    success(system_info)
}

async fn get_system_metrics(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<OrchestratorMetrics>>) {
    let metrics = state.orchestrator.get_metrics().await;
    success(metrics)
}
// Service management endpoints
async fn list_services(
    State(_state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<Vec<ServiceInfo>>>) {
    // Mock implementation - return empty list for now
    let services = vec![];
    success(services)
}
async fn register_service(
    State(state): State<ApiState>,
    Json(request): Json<RegisterServiceRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let service_info = ServiceInfo {
        service_id: Uuid::new_v4().to_string(),
        name: request.name,
        service_type: request.service_type,
        version: request.version,
        description: request.description,
        endpoints: request.endpoints.unwrap_or_default(),
        tags: request.tags.unwrap_or_default().into_values().collect(),
        metadata: request.metadata.unwrap_or_default(),
        health_check_endpoint: None,
        dependencies: vec![],
        status: crate::traits::service::ServiceStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: Uuid::new_v4().to_string(),
        host: "localhost".to_string(),
        port: 8080,
    };
    state.broadcast_event(ApiEvent::ServiceStarted {
        service_id: service_info.service_id.clone(),
        timestamp: chrono::Utc::now(),
    });
    success(service_info.service_id)
}
async fn get_service(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceInfo>>) {
    // Mock implementation - service not found for now
        error(StatusCode::NOT_FOUND, format!("Service {} not found", id))
}
async fn update_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(_request): Json<RegisterServiceRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // Implementation would update service configuration
    success(format!("Service {} updated", id))
}
async fn unregister_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    // Mock implementation
            state.broadcast_event(ApiEvent::ServiceStopped {
                service_id: id,
        timestamp: chrono::Utc::now(),
            });
            success(())
}
async fn start_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // Implementation would start the service
    state.broadcast_event(ApiEvent::ServiceStarted {
        service_id: id.clone(),
        timestamp: Utc::now(),
    });
    success(format!("Service {} started", id))
}
async fn stop_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // Implementation would stop the service
    state.broadcast_event(ApiEvent::ServiceStopped {
        service_id: id.clone(),
        timestamp: Utc::now(),
    });
    success(format!("Service {} stopped", id))
}
async fn restart_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // Implementation would restart the service
    state.broadcast_event(ApiEvent::ServiceStopped {
        service_id: id.clone(),
        timestamp: Utc::now(),
    });
    success(format!("Service {} restarted", id))
}
async fn get_service_health(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceHealth>>) {
    // Mock implementation
    let health = ServiceHealth {
        service_id: id,
        status: "healthy".to_string(),
        last_check: chrono::Utc::now(),
        response_time_ms: 100,
        error_count: 0,
        details: HashMap::new(),
    };
    success(health)
}
async fn get_service_metrics(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceMetrics>>) {
    // Mock implementation
    let metrics = ServiceMetrics {
        request_count: 0,
        error_count: 0,
        average_response_time: 0.0,
        uptime: std::time::Duration::from_secs(0),
        memory_usage: Some(0),
        cpu_usage: Some(0.0),
        active_connections: 0,
        custom_metrics: HashMap::new(),
        queue_depth: 0,
        throughput_rps: 0.0,
        error_rate: 0.0,
        uptime_seconds: 0,
        last_updated: chrono::Utc::now(),
    };
    success(metrics)
}
// Communication endpoints
async fn send_message(
    State(_state): State<ApiState>,
    Json(request): Json<SendMessageRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let message = ServiceMessage {
        id: Uuid::new_v4().to_string(),
        message_type: request.message_type,
        topic: request.topic,
        payload: request.payload,
        headers: request.headers.unwrap_or_default(),
        timestamp: chrono::Utc::now(),
        correlation_id: Some(Uuid::new_v4().to_string()),
        reply_to: None,
        ttl: request.ttl,
    };
    // Mock implementation - just return success
    success(message.id)
}
async fn broadcast_message(
    State(_state): State<ApiState>,
    Json(request): Json<BroadcastMessageRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let message = ServiceMessage {
        id: Uuid::new_v4().to_string(),
        message_type: request.message_type,
        topic: request.topic,
        payload: request.payload,
        headers: request.headers.unwrap_or_default(),
        timestamp: chrono::Utc::now(),
        correlation_id: Some(Uuid::new_v4().to_string()),
        reply_to: None,
        ttl: request.ttl,
    };
    // Mock implementation - just return success
    success(message.id)
}
async fn get_communication_stats(
    State(_state): State<ApiState>,
) -> (
    StatusCode,
    Json<ApiResponse<CommunicationStats>>,
) {
    // Mock implementation
    let comm_stats = CommunicationStats::default();
    success(comm_stats)
}
async fn get_connections(State(_state): State<ApiState>) -> (StatusCode, Json<ApiResponse<usize>>) {
    // Mock implementation
    let count = 0;
    success(count)
}
// Metrics endpoints
async fn get_orchestrator_metrics(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<OrchestratorMetrics>>) {
    let metrics = state.orchestrator.get_metrics().await;
    success(metrics)
}
async fn get_all_service_metrics(
    State(_state): State<ApiState>,
) -> (
    StatusCode,
    Json<ApiResponse<HashMap<String, ServiceMetrics>>>,
) {
    // Mock implementation - return empty map for now
    let metrics_map = HashMap::new();
    success(metrics_map)
}
async fn prometheus_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let metrics = state.orchestrator.get_metrics().await;
    let comm_stats = CommunicationStats::default();
    
    let prometheus_output = format!(
        "# HELP songbird_services_total Total number of services\n\
         # TYPE songbird_services_total gauge\n\
         songbird_services_total {}\n\
         # HELP songbird_services_healthy Number of healthy services\n\
         # TYPE songbird_services_healthy gauge\n\
         songbird_services_healthy {}\n\
         # HELP songbird_requests_total Total number of requests\n\
         # TYPE songbird_requests_total counter\n\
         songbird_requests_total {}\n\
         # HELP songbird_connections_active Active WebSocket connections\n\
         # TYPE songbird_connections_active gauge\n\
         songbird_connections_active {}\n\
         # HELP songbird_messages_sent_total Total messages sent\n\
         # TYPE songbird_messages_sent_total counter\n\
         songbird_messages_sent_total {}\n",
        metrics.total_services,
        metrics.healthy_services,
        metrics.total_requests,
        comm_stats.active_connections,
        comm_stats.messages_sent
    );
    (
        [("content-type", "text/plain; version=0.0.4")],
        prometheus_output,
    )
}
// Real-time streams
async fn events_stream(State(state): State<ApiState>) -> impl IntoResponse {
    let mut receiver = state.event_stream.subscribe();
    let stream = async_stream::stream! {
        while let Ok(event) = receiver.recv().await {
            let data = serde_json::to_string(&event).unwrap_or_default();
            yield Ok::<_, Infallible>(Event::default().data(data));
        }
    };
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}
async fn metrics_stream(State(state): State<ApiState>) -> impl IntoResponse {
    let orchestrator = Arc::clone(&state.orchestrator);
    let stream = async_stream::stream! {
        loop {
            let metrics = orchestrator.get_metrics().await;
            let event = ApiEvent::MetricsUpdate {
                metrics,
                timestamp: chrono::Utc::now(),
            };
            let data = serde_json::to_string(&event).unwrap_or_default();
            yield Ok::<_, Infallible>(Event::default().data(data));
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    };
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    )
}
// Dashboard endpoint
async fn get_dashboard_data(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<DashboardData>>) {
    let metrics = state.orchestrator.get_metrics().await;
    let comm_stats = CommunicationStats::default();
    let services = vec![]; // Mock empty services list
    
    let system_info = SystemInfo {
        uptime_seconds: metrics.uptime_seconds,
        total_services: metrics.total_services,
        healthy_services: metrics.healthy_services,
        active_connections: comm_stats.active_connections,
        total_requests: metrics.total_requests,
        api_endpoints: vec![],
    };
    
    let dashboard = DashboardData {
        system_info,
        orchestrator_metrics: metrics,
        services,
        communication_stats: comm_stats,
        recent_events: vec![],
    };
    success(dashboard)
}
pub mod byob;
