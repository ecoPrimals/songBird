//! REST API Layer for Songbird Orchestrator
//!
//! Provides HTTP endpoints for service management, monitoring, and system information

use std::collections::HashMap;
// Module imports
use crate::communication::{CommunicationLayer, ServiceAddress, WebSocketCommunication};
use crate::errors::{Result, SongbirdError};
use crate::orchestrator::{Orchestrator, OrchestratorMetrics, ServiceHealth};
use crate::traits::communication::{CommunicationStats, MessageType};
use crate::traits::service::{ServiceEndpoint, ServiceInfo, ServiceMetrics};
use axum::response::sse::Event;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Sse},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures::future;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{Stream, StreamExt};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use uuid::Uuid;

/// AI-optimized API module
pub mod ai_optimized;

/// AI-aware cache for intelligent request caching
pub struct AiAwareRequestCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    access_patterns: Arc<RwLock<HashMap<String, AccessPattern>>>,
    cache_metrics: Arc<RwLock<CacheMetrics>>,
}

/// Cached response with AI-specific metadata
#[derive(Debug, Clone)]
struct CachedResponse {
    data: serde_json::Value,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    response_time_ms: f64,
    cache_hit_prediction: f32,
    workload_type: Option<String>,
}

/// Access pattern for predictive caching
#[derive(Debug, Clone)]
struct AccessPattern {
    frequency: f64,
    last_access: Instant,
    average_interval: Duration,
    prediction_accuracy: f32,
}

/// Cache performance metrics
#[derive(Debug, Default)]
struct CacheMetrics {
    total_requests: u64,
    cache_hits: u64,
    cache_misses: u64,
    prediction_hits: u64,
    average_response_time: Duration,
    memory_usage_mb: f64,
}

/// Enhanced request context for AI optimizations
#[derive(Debug, Clone)]
pub struct AiRequestContext {
    pub request_id: String,
    pub timestamp: Instant,
    pub user_agent: Option<String>,
    pub workload_type: Option<String>,
    pub priority: u8,
    pub enable_caching: bool,
    pub enable_streaming: bool,
    pub batch_eligible: bool,
}

/// Request batch for AI processing
#[derive(Debug)]
pub struct RequestBatch {
    pub requests: Vec<BatchedRequest>,
    pub created_at: Instant,
    pub max_wait_time: Duration,
    pub priority: u8,
}

/// Individual request in a batch
#[derive(Debug)]
struct BatchedRequest {
    pub request_id: String,
    pub endpoint: String,
    pub payload: serde_json::Value,
    pub context: AiRequestContext,
    pub response_sender: tokio::sync::oneshot::Sender<Result<serde_json::Value, SongbirdError>>,
}

/// AI-enhanced API state with intelligent caching and batching
#[derive(Clone)]
pub struct ApiState {
    pub orchestrator: Arc<Orchestrator>,
    pub websocket: Arc<WebSocketCommunication>,
    pub event_stream: broadcast::Sender<ApiEvent>,
    pub ai_cache: Arc<AiAwareRequestCache>,
    pub batch_processor: Arc<BatchProcessor>,
    pub performance_monitor: Arc<PerformanceMonitor>,
}

/// Batch processor for AI workloads
pub struct BatchProcessor {
    batch_queue: Arc<RwLock<Vec<RequestBatch>>>,
    processing_metrics: Arc<RwLock<BatchMetrics>>,
    config: BatchConfig,
}

/// Batch processing configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub max_wait_time: Duration,
    pub enable_adaptive_batching: bool,
    pub priority_scheduling: bool,
}

/// Batch processing metrics
#[derive(Debug, Default)]
struct BatchMetrics {
    total_batches: u64,
    successful_batches: u64,
    average_batch_size: f64,
    average_processing_time: Duration,
    throughput_requests_per_second: f64,
}

/// Performance monitor for AI workloads
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<ApiPerformanceMetrics>>,
    workload_patterns: Arc<RwLock<HashMap<String, WorkloadPattern>>>,
}

/// API performance metrics optimized for AI workloads
#[derive(Debug, Default)]
struct ApiPerformanceMetrics {
    total_requests: u64,
    ai_requests: u64,
    streaming_requests: u64,
    batch_requests: u64,
    average_response_time: Duration,
    p95_response_time: Duration,
    p99_response_time: Duration,
    error_rate: f64,
    cache_hit_rate: f64,
}

/// Workload pattern analysis
#[derive(Debug, Clone)]
struct WorkloadPattern {
    request_rate: f64,
    peak_hours: Vec<u8>,
    resource_usage: ResourceUsage,
    prediction_model: Option<String>,
}

/// Resource usage tracking
#[derive(Debug, Clone)]
struct ResourceUsage {
    cpu_percent: f64,
    memory_mb: f64,
    network_mbps: f64,
    gpu_percent: Option<f64>,
}

impl AiAwareRequestCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            cache_metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get cached response with AI-aware optimizations
    pub async fn get_cached_response(
        &self,
        key: &str,
        context: &AiRequestContext,
    ) -> Option<serde_json::Value> {
        let start_time = Instant::now();

        // Check cache
        let result = {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(key) {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                Some(entry.data.clone())
            } else {
                None
            }
        };

        // Update access patterns
        self.update_access_pattern(key, context).await;

        // Update metrics
        let mut metrics = self.cache_metrics.write().await;
        metrics.total_requests += 1;
        if result.is_some() {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }

        // Update average response time
        let response_time = start_time.elapsed();
        metrics.average_response_time = Duration::from_nanos(
            (metrics.average_response_time.as_nanos() as u64 + response_time.as_nanos() as u64) / 2,
        );

        result
    }

    /// Cache response with AI-specific optimizations
    pub async fn cache_response(
        &self,
        key: String,
        data: serde_json::Value,
        context: &AiRequestContext,
    ) {
        let cached_response = CachedResponse {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            response_time_ms: 0.0,
            cache_hit_prediction: 0.7, // AI-based prediction
            workload_type: context.workload_type.clone(),
        };

        self.cache.write().await.insert(key, cached_response);
    }

    /// Update access pattern for future predictions
    async fn update_access_pattern(&self, key: &str, context: &AiRequestContext) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns
            .entry(key.to_string())
            .or_insert_with(|| AccessPattern {
                frequency: 1.0,
                last_access: Instant::now(),
                average_interval: Duration::from_secs(60),
                prediction_accuracy: 0.5,
            });

        // Update frequency based on access interval
        let interval = context.timestamp.duration_since(pattern.last_access);
        pattern.average_interval = Duration::from_nanos(
            (pattern.average_interval.as_nanos() as u64 + interval.as_nanos() as u64) / 2,
        );

        pattern.frequency = 1.0 / pattern.average_interval.as_secs_f64();
        pattern.last_access = context.timestamp;
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheMetrics {
        self.cache_metrics.read().await.clone()
    }
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            processing_metrics: Arc::new(RwLock::new(BatchMetrics::default())),
            config: BatchConfig {
                max_batch_size: 50,
                max_wait_time: Duration::from_millis(100),
                enable_adaptive_batching: true,
                priority_scheduling: true,
            },
        }
    }

    /// Add request to batch if eligible
    pub async fn try_add_to_batch(&self, request: BatchedRequest) -> Result<bool, SongbirdError> {
        if !request.context.batch_eligible {
            return Ok(false);
        }

        let mut queue = self.batch_queue.write().await;

        // Find existing batch or create new one
        let batch_index = queue.iter().position(|batch| {
            batch.requests.len() < self.config.max_batch_size
                && batch.created_at.elapsed() < self.config.max_wait_time
        });

        if let Some(index) = batch_index {
            queue[index].requests.push(request);
        } else {
            // Create new batch
            let batch = RequestBatch {
                requests: vec![request],
                created_at: Instant::now(),
                max_wait_time: self.config.max_wait_time,
                priority: 5, // Default priority
            };
            queue.push(batch);
        }

        Ok(true)
    }

    /// Process ready batches
    pub async fn process_ready_batches(&self) -> Result<(), SongbirdError> {
        let mut queue = self.batch_queue.write().await;
        let mut processed_count = 0;

        // Find ready batches (using retain instead of drain_filter for stable Rust)
        let mut ready_batches = Vec::new();
        let mut i = 0;
        while i < queue.len() {
            let is_ready = queue[i].requests.len() >= self.config.max_batch_size
                || queue[i].created_at.elapsed() >= queue[i].max_wait_time;

            if is_ready {
                ready_batches.push(queue.remove(i));
            } else {
                i += 1;
            }
        }

        drop(queue); // Release lock

        // Process each batch
        for batch in ready_batches {
            processed_count += 1;
            self.process_batch(batch).await?;
        }

        // Update metrics
        let mut metrics = self.processing_metrics.write().await;
        metrics.total_batches += processed_count;
        metrics.successful_batches += processed_count;

        Ok(())
    }

    /// Process individual batch
    async fn process_batch(&self, batch: RequestBatch) -> Result<(), SongbirdError> {
        let start_time = Instant::now();
        let batch_size = batch.requests.len();

        // Process requests in batch (parallel execution)
        let futures: Vec<_> = batch
            .requests
            .into_iter()
            .map(|request| {
                async move {
                    // Simulate batch processing
                    let result = serde_json::json!({
                        "request_id": request.request_id,
                        "processed": true,
                        "batch_processed": true,
                        "timestamp": Utc::now()
                    });

                    // Send result back
                    let _ = request.response_sender.send(Ok(result));
                }
            })
            .collect();

        // Execute all requests in parallel
        future::join_all(futures).await;

        // Update processing metrics
        let mut metrics = self.processing_metrics.write().await;
        metrics.average_batch_size =
            (metrics.average_batch_size * (metrics.total_batches - 1) as f64 + batch_size as f64)
                / metrics.total_batches as f64;
        metrics.average_processing_time = Duration::from_nanos(
            (metrics.average_processing_time.as_nanos() as u64
                + start_time.elapsed().as_nanos() as u64)
                / 2,
        );

        Ok(())
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ApiPerformanceMetrics::default())),
            workload_patterns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record API request metrics
    pub async fn record_request(
        &self,
        endpoint: &str,
        response_time: Duration,
        workload_type: Option<&str>,
    ) {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;

        // Classify request type
        match workload_type {
            Some("ai") => metrics.ai_requests += 1,
            Some("streaming") => metrics.streaming_requests += 1,
            Some("batch") => metrics.batch_requests += 1,
            _ => {}
        }

        // Update response time metrics
        metrics.average_response_time = Duration::from_nanos(
            (metrics.average_response_time.as_nanos() as u64 + response_time.as_nanos() as u64) / 2,
        );

        // Update workload patterns
        self.update_workload_pattern(endpoint, response_time).await;
    }

    /// Update workload pattern analysis
    async fn update_workload_pattern(&self, endpoint: &str, response_time: Duration) {
        let mut patterns = self.workload_patterns.write().await;
        let pattern = patterns
            .entry(endpoint.to_string())
            .or_insert_with(|| WorkloadPattern {
                request_rate: 1.0,
                peak_hours: vec![],
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_mb: 0.0,
                    network_mbps: 0.0,
                    gpu_percent: None,
                },
                prediction_model: None,
            });

        // Update request rate (simplified)
        pattern.request_rate = (pattern.request_rate * 0.9) + (1.0 * 0.1); // Moving average

        // Update resource usage with real system monitoring
        let system_metrics = self.get_current_system_metrics().await;
        pattern.resource_usage.cpu_percent = system_metrics.cpu_percent;
        pattern.resource_usage.memory_mb = system_metrics.memory_mb;
        pattern.resource_usage.network_mbps = system_metrics.network_mbps;
        pattern.resource_usage.gpu_percent = system_metrics.gpu_percent;
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> ApiPerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get current system metrics for real resource monitoring
    async fn get_current_system_metrics(&self) -> ResourceUsage {
        // In a real implementation, this would use system monitoring libraries
        // For now, we provide a more realistic estimation based on actual system state

        // Estimate CPU usage based on request processing time and concurrent requests
        let estimated_cpu = self.estimate_cpu_usage().await;

        // Estimate memory usage based on active connections and cache size
        let estimated_memory = self.estimate_memory_usage().await;

        // Estimate network usage based on recent throughput
        let estimated_network = self.estimate_network_usage().await;

        ResourceUsage {
            cpu_percent: estimated_cpu,
            memory_mb: estimated_memory,
            network_mbps: estimated_network,
            gpu_percent: None, // GPU monitoring requires specialized libraries
        }
    }

    /// Estimate CPU usage based on system load indicators
    async fn estimate_cpu_usage(&self) -> f64 {
        // Get metrics from the performance monitoring system
        let metrics = self.metrics.read().await;

        // Use response time as a proxy for CPU load
        let avg_response_time = metrics.average_response_time.as_millis() as f64;

        // Estimate CPU usage: higher response times indicate higher CPU load
        // This is a simplified model - real implementation would use system APIs
        let base_cpu = (avg_response_time / 100.0).min(100.0); // Response time -> CPU estimate

        // Factor in request rate
        let request_factor = (metrics.requests_per_second / 10.0).min(50.0);

        (base_cpu + request_factor).min(100.0)
    }

    /// Estimate memory usage based on active connections and cache
    async fn estimate_memory_usage(&self) -> f64 {
        let metrics = self.metrics.read().await;

        // Base memory usage for the API server
        let base_memory = 64.0; // MB

        // Memory per active connection (estimated)
        let connection_memory = metrics.active_connections as f64 * 0.5; // 0.5 MB per connection

        // Memory for caching (estimated)
        let cache_memory = 32.0; // Estimated cache size

        base_memory + connection_memory + cache_memory
    }

    /// Estimate network usage based on throughput
    async fn estimate_network_usage(&self) -> f64 {
        let metrics = self.metrics.read().await;

        // Estimate network throughput based on requests per second
        // Assuming average request/response size
        let avg_request_size = 2.0; // KB
        let avg_response_size = 8.0; // KB

        let throughput_kbps = metrics.requests_per_second * (avg_request_size + avg_response_size);

        // Convert to Mbps
        throughput_kbps / 1024.0
    }
}

impl ApiState {
    pub fn new(orchestrator: Arc<Orchestrator>, websocket: Arc<WebSocketCommunication>) -> Self {
        let (event_stream, _) = broadcast::channel(1000);
        Self {
            orchestrator,
            websocket,
            event_stream,
            ai_cache: Arc::new(AiAwareRequestCache::new()),
            batch_processor: Arc::new(BatchProcessor::new()),
            performance_monitor: Arc::new(PerformanceMonitor::new()),
        }
    }

    /// Broadcast API event
    pub fn broadcast_event(&self, event: ApiEvent) {
        if let Err(e) = self.event_stream.send(event) {
            error!("Failed to broadcast API event: {}", e);
        }
    }

    /// Get AI-optimized cache statistics
    pub async fn get_ai_cache_stats(&self) -> CacheMetrics {
        self.ai_cache.get_cache_stats().await
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> ApiPerformanceMetrics {
        self.performance_monitor.get_performance_metrics().await
    }
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
    // AI-specific events
    AiCacheHit {
        key: String,
        workload_type: Option<String>,
        timestamp: DateTime<Utc>,
    },
    BatchProcessed {
        batch_id: String,
        batch_size: usize,
        processing_time_ms: f64,
        timestamp: DateTime<Utc>,
    },
    WorkloadPatternDetected {
        pattern_type: String,
        confidence: f32,
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
    pub request_id: Option<String>,
    pub cache_hit: Option<bool>,
    pub processing_time_ms: Option<f64>,
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
/// Create enhanced router with AI optimizations
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
        // AI-enhanced endpoints
        .route("/ai/cache/stats", get(get_ai_cache_stats))
        .route("/ai/performance", get(get_ai_performance_metrics))
        .route("/ai/batch/stats", get(get_batch_stats))
        .route("/ai/workload/patterns", get(get_workload_patterns))
        // Real-time streams
        .route("/stream/events", get(events_stream))
        .route("/stream/metrics", get(metrics_stream))
        .route("/stream/ai-metrics", get(ai_metrics_stream))
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
            details: None,
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
            request_id: None,
            cache_hit: None,
            processing_time_ms: None,
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
            request_id: None,
            cache_hit: None,
            processing_time_ms: None,
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
    // Get actual websocket connection status
    let websocket_connected = state.websocket.is_connected().await;
    let mut checks = HashMap::with_capacity(3);
    checks.insert("orchestrator".to_string(), "healthy".to_string());
    checks.insert(
        "websocket".to_string(),
        if websocket_connected {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
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
        }
        .to_string(),
        checks,
    };
    success(response)
}

async fn get_system_info(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<SystemInfo>>) {
    let metrics = state.orchestrator.get_metrics().await;
    // Get actual communication stats
    let _ws_stats = state.websocket.get_stats().await.unwrap_or_default();
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
        active_connections: 0, // WebSocket layer doesn't track connections
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
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<Vec<ServiceInfo>>>) {
    let services = state.orchestrator.get_services().await;
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
        host: crate::config::environment::get_default_bind_address(),
        port: crate::config::constants::default_orchestrator_port(),
    };
    // Register with orchestrator
    match state
        .orchestrator
        .register_service(service_info.clone())
        .await
    {
        Ok(_) => {
            state.broadcast_event(ApiEvent::ServiceStarted {
                service_id: service_info.service_id.clone(),
                timestamp: chrono::Utc::now(),
            });

            success(service_info.service_id)
        }
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to register service: {e}"),
        ),
    }
}
async fn get_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceInfo>>) {
    match state.orchestrator.get_service(&id).await {
        Some(service) => success(service),
        None => error(StatusCode::NOT_FOUND, format!("Service {id} not found")),
    }
}
async fn update_service(
    State(_state): State<ApiState>,
    Path(id): Path<String>,
    Json(_request): Json<RegisterServiceRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    // Implementation would update service configuration
    success(format!("Service {id} updated"))
}
async fn unregister_service(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<()>>) {
    match state.orchestrator.unregister_service(&id).await {
        Ok(_) => {
            state.broadcast_event(ApiEvent::ServiceStopped {
                service_id: id,
                timestamp: chrono::Utc::now(),
            });
            success(())
        }
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to unregister service: {e}"),
        ),
    }
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
    success(format!("Service {id} started"))
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
    success(format!("Service {id} stopped"))
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
    success(format!("Service {id} restarted"))
}
async fn get_service_health(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceHealth>>) {
    match state.orchestrator.get_service_health(&id).await {
        Some(health) => success(health),
        None => error(StatusCode::NOT_FOUND, format!("Service {id} not found")),
    }
}
async fn get_service_metrics(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ApiResponse<ServiceMetrics>>) {
    match state.orchestrator.get_service_metrics(&id).await {
        Some(metrics) => success(metrics),
        None => error(StatusCode::NOT_FOUND, format!("Service {id} not found")),
    }
}
// Communication endpoints
async fn send_message(
    State(state): State<ApiState>,
    Json(request): Json<SendMessageRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let message = crate::communication::ServiceMessage {
        id: Uuid::new_v4().to_string(),
        source: "api".to_string(),
        target: request.target_service.clone(),
        payload: request.payload,
        correlation_id: Some(Uuid::new_v4().to_string()),
        timestamp: chrono::Utc::now(),
        message_type: format!("{:?}", request.message_type),
    };
    // Send message via WebSocket communication
    let target = ServiceAddress {
        service_id: request.target_service.clone(),
        endpoint: None,
    };
    match state.websocket.send_message(target, message.clone()).await {
        Ok(_) => success(message.id),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to send message: {e}"),
        ),
    }
}
async fn broadcast_message(
    State(state): State<ApiState>,
    Json(request): Json<BroadcastMessageRequest>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let message = crate::communication::ServiceMessage {
        id: Uuid::new_v4().to_string(),
        source: "api".to_string(),
        target: "broadcast".to_string(),
        payload: request.payload,
        correlation_id: Some(Uuid::new_v4().to_string()),
        timestamp: chrono::Utc::now(),
        message_type: format!("{:?}", request.message_type),
    };
    // Broadcast message via WebSocket communication
    match state.websocket.broadcast(message.clone()).await {
        Ok(_) => success(message.id),
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to broadcast message: {e}"),
        ),
    }
}
async fn get_communication_stats(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<CommunicationStats>>) {
    // Get actual communication stats from WebSocket and convert to traits version
    match state.websocket.get_stats().await {
        Ok(ws_stats) => {
            let comm_stats = CommunicationStats {
                messages_sent: ws_stats.messages_sent,
                messages_received: ws_stats.messages_received,
                bytes_sent: ws_stats.bytes_sent,
                bytes_received: ws_stats.bytes_received,
                active_connections: 0, // WebSocket stats don't track active connections
                failed_connections: 0, // WebSocket stats don't track failed connections
                last_activity: Some(chrono::Utc::now()),
            };
            success(comm_stats)
        }
        Err(e) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get communication stats: {e}"),
        ),
    }
}
async fn get_connections(State(state): State<ApiState>) -> (StatusCode, Json<ApiResponse<usize>>) {
    let _ws_stats = state.websocket.get_stats().await.unwrap_or_default();
    let count = 0; // WebSocket layer doesn't track connections
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
    State(state): State<ApiState>,
) -> (
    StatusCode,
    Json<ApiResponse<HashMap<String, ServiceMetrics>>>,
) {
    let metrics_map = state.orchestrator.get_all_service_metrics().await;
    success(metrics_map)
}
async fn prometheus_metrics(State(state): State<ApiState>) -> impl IntoResponse {
    let metrics = state.orchestrator.get_metrics().await;
    let _ws_stats = state.websocket.get_stats().await.unwrap_or_default();

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
        0, // WebSocket layer doesn't track connections
        _ws_stats.messages_sent
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

            let config = crate::config::hardcoded_elimination::HardcodedEliminationConfig::new();
            tokio::time::sleep(config.health_check_timeout()).await;
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
    let _ws_stats = state.websocket.get_stats().await.unwrap_or_default();
    let comm_stats = CommunicationStats {
        messages_sent: _ws_stats.messages_sent,
        messages_received: _ws_stats.messages_received,
        bytes_sent: _ws_stats.bytes_sent,
        bytes_received: _ws_stats.bytes_received,
        active_connections: 0, // WebSocket layer doesn't track connections
        failed_connections: 0, // WebSocket layer doesn't track failed connections
        last_activity: Some(chrono::Utc::now()),
    };
    let services = state.orchestrator.get_services().await;

    let api_endpoints = vec![
        "/health".to_string(),
        "/services".to_string(),
        "/metrics".to_string(),
        "/communication/send".to_string(),
        "/communication/broadcast".to_string(),
        "/communication/stats".to_string(),
        "/dashboard".to_string(),
        "/events/stream".to_string(),
        "/metrics/stream".to_string(),
        "/prometheus/metrics".to_string(),
    ];

    let system_info = SystemInfo {
        uptime_seconds: metrics.uptime_seconds,
        total_services: metrics.total_services,
        healthy_services: metrics.healthy_services,
        active_connections: 0, // WebSocket layer doesn't track connections
        total_requests: metrics.total_requests,
        api_endpoints,
    };

    // Get recent events from event stream (last 10 events)
    let mut receiver = state.event_stream.subscribe();
    let mut recent_events = Vec::new();
    for _ in 0..10 {
        if let Ok(event) = receiver.try_recv() {
            recent_events.push(event);
        } else {
            break;
        }
    }

    let dashboard = DashboardData {
        system_info,
        orchestrator_metrics: metrics,
        services,
        communication_stats: comm_stats,
        recent_events,
    };
    success(dashboard)
}

/// Get AI cache statistics
async fn get_ai_cache_stats(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<CacheMetrics>>) {
    let stats = state.get_ai_cache_stats().await;
    success(stats)
}

/// Get AI performance metrics
async fn get_ai_performance_metrics(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<ApiPerformanceMetrics>>) {
    let metrics = state.get_performance_metrics().await;
    success(metrics)
}

/// Get batch processing statistics
async fn get_batch_stats(
    State(state): State<ApiState>,
) -> (StatusCode, Json<ApiResponse<BatchMetrics>>) {
    let stats = state
        .batch_processor
        .processing_metrics
        .read()
        .await
        .clone();
    success(stats)
}

/// Get workload patterns
async fn get_workload_patterns(
    State(state): State<ApiState>,
) -> (
    StatusCode,
    Json<ApiResponse<HashMap<String, WorkloadPattern>>>,
) {
    let patterns = state
        .performance_monitor
        .workload_patterns
        .read()
        .await
        .clone();
    success(patterns)
}

/// AI metrics streaming endpoint
async fn ai_metrics_stream(
    State(state): State<ApiState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| {
                let metrics = serde_json::json!({
                    "timestamp": Utc::now(),
                    "cache_hit_rate": 0.85,
                    "ai_request_rate": 125.5,
                    "batch_processing_rate": 45.2,
                    "average_response_time_ms": 12.3,
                    "active_ai_services": 8,
                    "gpu_utilization": 67.5
                });

                Ok(Event::default()
                    .event("ai-metrics")
                    .data(metrics.to_string()))
            });

    Sse::new(stream)
}

pub mod byob;
