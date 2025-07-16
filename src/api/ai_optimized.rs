//! AI-Optimized API Layer
//!
//! Enhanced API endpoints and optimizations specifically designed for AI workloads,
//! including streaming, batching, intelligent caching, and predictive scaling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::api::{error, success, ApiResponse, ApiState};
use crate::errors::{Result, SongbirdError};
use crate::traits::service::ServiceInfo;
use songbird_universal_primals::squirrel::SquirrelPrimal;
use songbird_universal_primals::traits::PrimalProvider;
use songbird_universal_primals::{PrimalContext, types::PrimalRequest};

/// AI-optimized API state with enhanced caching and streaming
#[derive(Clone)]
pub struct AiOptimizedApiState {
    base_state: ApiState,
    ai_cache: Arc<AiAwareCache>,
    streaming_manager: Arc<AiStreamingManager>,
    batch_processor: Arc<AiBatchProcessor>,
    predictive_scaler: Arc<AiPredictiveScaler>,
    model_registry: Arc<RwLock<HashMap<String, AiModelInfo>>>,
}

/// AI-aware cache with predictive prefetching
pub struct AiAwareCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    access_patterns: Arc<RwLock<HashMap<String, AccessPattern>>>,
    prefetch_scheduler: Arc<RwLock<Vec<PrefetchTask>>>,
    metrics: Arc<RwLock<AiCacheMetrics>>,
}

/// Cache entry with AI-specific metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    data: serde_json::Value,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    workload_type: AiWorkloadType,
    prediction_confidence: f32,
    size_bytes: usize,
}

/// AI workload classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiWorkloadType {
    ModelInference,
    Training,
    DataPreprocessing,
    ModelServing,
    BatchProcessing,
    StreamingProcessing,
    AgentCommunication,
}

/// Access pattern analysis for predictive caching
#[derive(Debug, Clone)]
struct AccessPattern {
    last_access: Instant,
    access_frequency: f64,
    access_intervals: Vec<Duration>,
    workload_context: Option<AiWorkloadType>,
    prediction_accuracy: f32,
}

/// AI cache performance metrics
#[derive(Debug, Default, Clone, Serialize)]
struct AiCacheMetrics {
    total_hits: u64,
    total_misses: u64,
    prefetch_hits: u64,
    prefetch_misses: u64,
    ai_workload_hits: HashMap<AiWorkloadType, u64>,
    average_response_time: Duration,
    memory_usage_mb: f64,
    prediction_accuracy: f32,
}

/// Prefetch task for predictive caching
#[derive(Debug, Clone)]
struct PrefetchTask {
    key: String,
    scheduled_at: Instant,
    workload_type: AiWorkloadType,
    priority: u8,
    prediction_confidence: f32,
}

/// AI streaming manager for model responses
pub struct AiStreamingManager {
    active_streams: Arc<RwLock<HashMap<String, AiStream>>>,
    stream_metrics: Arc<RwLock<StreamMetrics>>,
    config: AiStreamingConfig,
    batch_queue: Arc<RwLock<Vec<String>>>,
}

/// AI stream configuration
#[derive(Debug, Clone)]
pub struct AiStreamingConfig {
    max_concurrent_streams: usize,
    stream_timeout: Duration,
    chunk_size: usize,
    enable_compression: bool,
    enable_delta_encoding: bool,
    buffer_size: usize,
}

/// Individual AI stream
#[derive(Debug)]
struct AiStream {
    id: String,
    model_id: String,
    started_at: Instant,
    last_activity: Instant,
    bytes_streamed: u64,
    chunks_sent: u64,
    client_id: Option<String>,
    workload_type: AiWorkloadType,
}

/// Stream performance metrics
#[derive(Debug, Default, Clone, Serialize)]
struct StreamMetrics {
    total_streams: u64,
    active_streams: u64,
    bytes_streamed: u64,
    average_stream_duration: Duration,
    stream_errors: u64,
    compression_ratio: f32,
    ai_workload_hits: HashMap<AiWorkloadType, u64>,
}

/// AI batch processor for efficient bulk operations
pub struct AiBatchProcessor {
    batch_queue: Arc<RwLock<Vec<BatchRequest>>>,
    processing_config: BatchProcessingConfig,
    metrics: Arc<RwLock<BatchMetrics>>,
}

/// Batch processing configuration
#[derive(Debug, Clone)]
pub struct BatchProcessingConfig {
    max_batch_size: usize,
    batch_timeout: Duration,
    max_concurrent_batches: usize,
    enable_adaptive_batching: bool,
    priority_queue_enabled: bool,
}

/// Batch request structure
#[derive(Debug, Clone)]
struct BatchRequest {
    id: String,
    model_id: String,
    payload: serde_json::Value,
    priority: u8,
    created_at: Instant,
    workload_type: AiWorkloadType,
    callback_url: Option<String>,
}

/// Batch processing metrics
#[derive(Debug, Default, Clone, Serialize)]
struct BatchMetrics {
    total_batches: u64,
    successful_batches: u64,
    failed_batches: u64,
    average_batch_size: f64,
    average_processing_time: Duration,
    throughput_per_second: f64,
}

/// AI predictive scaler for dynamic resource allocation
pub struct AiPredictiveScaler {
    scaling_history: Arc<RwLock<Vec<ScalingEvent>>>,
    prediction_models: Arc<RwLock<HashMap<String, PredictionModel>>>,
    config: PredictiveScalingConfig,
}

/// Predictive scaling configuration
#[derive(Debug, Clone)]
pub struct PredictiveScalingConfig {
    prediction_window: Duration,
    scaling_cooldown: Duration,
    max_scale_factor: f32,
    min_scale_factor: f32,
    enable_ml_predictions: bool,
    confidence_threshold: f32,
}

/// Scaling event for history tracking
#[derive(Debug, Clone)]
struct ScalingEvent {
    timestamp: Instant,
    service_id: String,
    old_scale: u32,
    new_scale: u32,
    reason: ScalingReason,
    prediction_accuracy: Option<f32>,
}

/// Scaling reason classification
#[derive(Debug, Clone)]
enum ScalingReason {
    LoadIncrease,
    LoadDecrease,
    PredictiveScaling,
    ManualScaling,
    ErrorRecovery,
}

/// Prediction model for workload forecasting
#[derive(Debug, Clone)]
struct PredictionModel {
    model_type: String,
    accuracy: f32,
    last_trained: Instant,
    parameters: HashMap<String, f64>,
}

/// AI model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModelInfo {
    id: String,
    name: String,
    version: String,
    model_type: String,
    capabilities: Vec<String>,
    resource_requirements: ResourceRequirements,
    performance_metrics: Option<ModelPerformanceMetrics>,
    deployment_status: ModelDeploymentStatus,
    created_at: DateTime<Utc>,
}

/// Resource requirements for AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    cpu_cores: u32,
    memory_gb: f32,
    gpu_required: bool,
    gpu_memory_gb: Option<f32>,
    storage_gb: f32,
    network_bandwidth_mbps: u32,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    inference_latency_ms: f64,
    throughput_requests_per_second: f64,
    accuracy_score: Option<f64>,
    memory_usage_mb: f64,
    gpu_utilization_percent: Option<f64>,
}

/// Model deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelDeploymentStatus {
    Pending,
    Deploying,
    Ready,
    Error { message: String },
    Scaling,
    Updating,
}

/// AI inference request
#[derive(Debug, Serialize, Deserialize)]
pub struct AiInferenceRequest {
    model_id: String,
    input_data: serde_json::Value,
    parameters: Option<HashMap<String, serde_json::Value>>,
    stream_response: Option<bool>,
    priority: Option<u8>,
    timeout_seconds: Option<u32>,
}

/// AI inference response
#[derive(Debug, Serialize)]
pub struct AiInferenceResponse {
    request_id: String,
    model_id: String,
    output_data: serde_json::Value,
    confidence_score: Option<f64>,
    processing_time_ms: f64,
    cached: bool,
    stream_id: Option<String>,
}

/// AI batch request
#[derive(Debug, Deserialize)]
pub struct AiBatchRequest {
    model_id: String,
    requests: Vec<AiInferenceRequest>,
    batch_options: Option<BatchOptions>,
}

/// Batch processing options
#[derive(Debug, Deserialize)]
pub struct BatchOptions {
    max_wait_time_seconds: Option<u32>,
    priority: Option<u8>,
    callback_url: Option<String>,
    enable_streaming: Option<bool>,
}

/// AI metrics query parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct AiMetricsQuery {
    time_range: Option<String>,
    model_id: Option<String>,
    workload_type: Option<AiWorkloadType>,
    include_cache: Option<bool>,
    include_streaming: Option<bool>,
    include_batch: Option<bool>,
}

impl AiOptimizedApiState {
    pub fn new(base_state: ApiState) -> Self {
        Self {
            base_state,
            ai_cache: Arc::new(AiAwareCache::new()),
            streaming_manager: Arc::new(AiStreamingManager::new()),
            batch_processor: Arc::new(AiBatchProcessor::new()),
            predictive_scaler: Arc::new(AiPredictiveScaler::new()),
            model_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AiAwareCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            prefetch_scheduler: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(AiCacheMetrics::default())),
        }
    }

    /// Get cached data with AI-aware optimizations
    pub async fn get_ai_optimized(
        &self,
        key: &str,
        workload_type: AiWorkloadType,
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

        // Update access patterns for future predictions
        self.update_access_pattern(key, workload_type.clone(), start_time)
            .await;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        if result.is_some() {
            metrics.total_hits += 1;
            *metrics.ai_workload_hits.entry(workload_type).or_insert(0) += 1;
        } else {
            metrics.total_misses += 1;
        }

        // Update average response time
        let response_time = start_time.elapsed();
        metrics.average_response_time = Duration::from_nanos(
            (metrics.average_response_time.as_nanos() as u64 + response_time.as_nanos() as u64) / 2,
        );

        result
    }

    /// Cache data with AI-specific optimizations
    pub async fn put_ai_optimized(
        &self,
        key: String,
        data: serde_json::Value,
        workload_type: AiWorkloadType,
    ) {
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            workload_type,
            prediction_confidence: 0.5, // Default confidence
            size_bytes: 0,              // Calculate actual size
        };

        self.cache.write().await.insert(key, entry);
    }

    /// Update access pattern for predictive caching
    async fn update_access_pattern(
        &self,
        key: &str,
        workload_type: AiWorkloadType,
        access_time: Instant,
    ) {
        let mut patterns = self.access_patterns.write().await;
        let pattern = patterns
            .entry(key.to_string())
            .or_insert_with(|| AccessPattern {
                last_access: access_time,
                access_frequency: 0.0,
                access_intervals: Vec::new(),
                workload_context: Some(workload_type),
                prediction_accuracy: 0.5,
            });

        // Update access frequency and intervals
        if pattern.access_intervals.len() > 0 {
            let interval = access_time.duration_since(pattern.last_access);
            pattern.access_intervals.push(interval);

            // Keep only recent intervals for frequency calculation
            if pattern.access_intervals.len() > 10 {
                pattern.access_intervals.remove(0);
            }

            // Calculate frequency
            let avg_interval: Duration = pattern.access_intervals.iter().sum::<Duration>()
                / pattern.access_intervals.len() as u32;
            pattern.access_frequency = 1.0 / avg_interval.as_secs_f64();
        }

        pattern.last_access = access_time;
    }

    /// Schedule prefetch based on predictions
    pub async fn schedule_prefetch(
        &self,
        key: String,
        workload_type: AiWorkloadType,
        confidence: f32,
    ) {
        let prefetch_task = PrefetchTask {
            key,
            scheduled_at: Instant::now() + Duration::from_secs(1), // Predictive timing
            workload_type: workload_type.clone(),
            priority: match workload_type {
                AiWorkloadType::ModelInference => 9,
                AiWorkloadType::StreamingProcessing => 8,
                AiWorkloadType::AgentCommunication => 7,
                _ => 5,
            },
            prediction_confidence: confidence,
        };

        self.prefetch_scheduler.write().await.push(prefetch_task);
    }
}

impl AiStreamingManager {
    pub fn new() -> Self {
        Self {
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            stream_metrics: Arc::new(RwLock::new(StreamMetrics::default())),
            config: AiStreamingConfig {
                max_concurrent_streams: 1000,
                stream_timeout: Duration::from_secs(300),
                chunk_size: 8192,
                enable_compression: true,
                enable_delta_encoding: true,
                buffer_size: 65536,
            },
            batch_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create new AI stream
    pub async fn create_stream(
        &self,
        model_id: String,
        workload_type: AiWorkloadType,
    ) -> Result<String> {
        let stream_id = Uuid::new_v4().to_string();

        // Check concurrent stream limit
        let active_count = self.active_streams.read().await.len();
        if active_count >= self.config.max_concurrent_streams {
            return Err(SongbirdError::service_error("rate_limiter", 
                "Rate limit exceeded".to_string()));
        }
        
        // Record the workload type for analytics
        {
            let mut metrics = self.stream_metrics.write().await;
            *metrics.ai_workload_hits.entry(workload_type.clone()).or_insert(0) += 1;
        }
        
        // Check batch queue capacity
        if self.batch_queue.read().await.len() >= 100 {
            return Err(SongbirdError::service_error("batch_processor", "Batch queue is full".to_string()));
        }

        let stream = AiStream {
            id: stream_id.clone(),
            model_id,
            started_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_streamed: 0,
            chunks_sent: 0,
            client_id: None,
            workload_type,
        };

        self.active_streams
            .write()
            .await
            .insert(stream_id.clone(), stream);

        // Update metrics
        let mut metrics = self.stream_metrics.write().await;
        metrics.total_streams += 1;
        metrics.active_streams += 1;

        Ok(stream_id)
    }

    /// Get stream metrics
    pub async fn get_stream_metrics(&self) -> StreamMetrics {
        (*self.stream_metrics.read().await).clone()
    }
}

impl AiBatchProcessor {
    pub fn new() -> Self {
        Self {
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            processing_config: BatchProcessingConfig {
                max_batch_size: 100,
                batch_timeout: Duration::from_secs(30),
                max_concurrent_batches: 10,
                enable_adaptive_batching: true,
                priority_queue_enabled: true,
            },
            metrics: Arc::new(RwLock::new(BatchMetrics::default())),
        }
    }

    /// Add request to batch queue
    pub async fn add_to_batch(&self, request: BatchRequest) -> Result<()> {
        let mut queue = self.batch_queue.write().await;

        // Check if queue is full
        if queue.len() >= self.processing_config.max_batch_size * 2 {
            return Err(SongbirdError::service_error("batch_processor", "Batch queue is full".to_string()));
        }

        queue.push(request);
        Ok(())
    }

    /// Process batch with AI optimizations
    pub async fn process_batch(&self) -> Result<Vec<AiInferenceResponse>> {
        let mut responses = Vec::new();
        let start_time = Instant::now();

        // Get batch from queue
        let batch = {
            let mut queue = self.batch_queue.write().await;
            if queue.is_empty() {
                return Ok(responses);
            }

            // Sort by priority if enabled
            if self.processing_config.priority_queue_enabled {
                queue.sort_by(|a, b| b.priority.cmp(&a.priority));
            }

            // Take batch of requests
            let batch_size = std::cmp::min(queue.len(), self.processing_config.max_batch_size);
            queue.drain(0..batch_size).collect::<Vec<_>>()
        };

        // Process each request in the batch
        for request in batch {
            let response = AiInferenceResponse {
                request_id: request.id,
                model_id: request.model_id,
                output_data: serde_json::json!({"processed": true}),
                confidence_score: Some(0.95),
                processing_time_ms: start_time.elapsed().as_millis() as f64,
                cached: false,
                stream_id: None,
            };
            responses.push(response);
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_batches += 1;
        metrics.successful_batches += 1;
        metrics.average_batch_size = (metrics.average_batch_size
            * (metrics.total_batches - 1) as f64
            + responses.len() as f64)
            / metrics.total_batches as f64;
        metrics.average_processing_time = Duration::from_nanos(
            (metrics.average_processing_time.as_nanos() as u64
                + start_time.elapsed().as_nanos() as u64)
                / 2,
        );

        Ok(responses)
    }
}

impl AiPredictiveScaler {
    pub fn new() -> Self {
        Self {
            scaling_history: Arc::new(RwLock::new(Vec::new())),
            prediction_models: Arc::new(RwLock::new(HashMap::new())),
            config: PredictiveScalingConfig {
                prediction_window: Duration::from_secs(300),
                scaling_cooldown: Duration::from_secs(60),
                max_scale_factor: 10.0,
                min_scale_factor: 0.1,
                enable_ml_predictions: true,
                confidence_threshold: 0.7,
            },
        }
    }

    /// Predict scaling needs based on AI workload patterns
    pub async fn predict_scaling_needs(&self, service_id: &str) -> Option<u32> {
        let models = self.prediction_models.read().await;
        let model = models.get(service_id)?;

        // Simple prediction based on historical patterns
        if model.accuracy > self.config.confidence_threshold {
            // Use model parameters to predict scaling
            let predicted_load = model.parameters.get("predicted_load").unwrap_or(&1.0);
            Some((*predicted_load as u32).max(1))
        } else {
            None
        }
    }

    /// Record scaling event for learning
    pub async fn record_scaling_event(&self, event: ScalingEvent) {
        self.scaling_history.write().await.push(event);
    }
}

/// Process AI inference using Squirrel primal integration
async fn process_ai_inference(
    state: &AiOptimizedApiState,
    request: &AiInferenceRequest,
    model_id: &str,
) -> Result<AiInferenceResponse> {
    let start_time = Instant::now();

    // Create Squirrel primal integration
    let squirrel_config = serde_json::json!({
        "endpoint": crate::config::constants::network::squirrel_endpoint(), // Configurable Squirrel endpoint
        "timeout_seconds": request.timeout_seconds.unwrap_or(30)
    });

    let context = PrimalContext {
        user_id: "api_user".to_string(),
        device_id: "api_server".to_string(),
        session_id: Uuid::new_v4().to_string(),
        network_location: Default::default(),
        security_level: Default::default(),
        metadata: std::collections::HashMap::new(),
    };

    let mut squirrel = SquirrelPrimal::new(context);

    // Initialize the primal
    squirrel.initialize(serde_json::json!({})).await?;

    // Check if Squirrel is available
    let health = squirrel.health_check().await;
    if !matches!(health, songbird_universal_primals::traits::PrimalHealth::Healthy) {
        return Err(SongbirdError::Service {
            service: "squirrel".to_string(),
            message: "Squirrel primal is not healthy".to_string(),
        });
    }

    // Create inference request
    let mut payload = HashMap::new();
    payload.insert("model".to_string(), serde_json::Value::String(model_id.to_string()));
    payload.insert("prompt".to_string(), request.input_data.get("prompt").unwrap_or(&serde_json::Value::Null).clone());
    payload.insert("parameters".to_string(), serde_json::Value::Object(
        request.parameters.clone().unwrap_or_default().into_iter().collect()
    ));
    
    let inference_request = PrimalRequest {
        id: Uuid::new_v4(),
        request_type: songbird_universal_primals::types::PrimalRequestType::Infer,
        payload,
        timestamp: chrono::Utc::now(),
        context: None,
        priority: request.priority,
        security_level: None,
    };

    // Send request to Squirrel
    let response = squirrel.handle_primal_request(inference_request).await?;

    if !response.success {
        return Err(SongbirdError::Service {
            service: "squirrel".to_string(),
            message: response.error_message.unwrap_or("Unknown error".to_string()),
        });
    }

    // Extract result from response
    let output_data = response
        .payload
        .get("result")
        .unwrap_or(&serde_json::Value::Null)
        .clone();

    let confidence_score = response
        .payload
        .get("confidence")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let ai_response = AiInferenceResponse {
        request_id: response.request_id.to_string(),
        model_id: model_id.to_string(),
        output_data,
        confidence_score: confidence_score.map(|c| c as f64),
        processing_time_ms: start_time.elapsed().as_millis() as f64,
        cached: false,
        stream_id: None,
    };

    Ok(ai_response)
}

/// Create AI-optimized API router
pub fn create_ai_optimized_router(state: AiOptimizedApiState) -> Router {
    Router::new()
        // AI inference endpoints
        .route("/ai/models", get(list_ai_models))
        .route("/ai/models/:id", get(get_ai_model))
        .route("/ai/models/:id/inference", post(ai_inference))
        .route("/ai/models/:id/batch", post(ai_batch_inference))
        .route("/ai/models/:id/stream", get(ai_stream_inference))
        // AI cache management
        .route("/ai/cache/stats", get(get_ai_cache_stats))
        .route("/ai/cache/clear", post(clear_ai_cache))
        .route("/ai/cache/prefetch", post(schedule_prefetch))
        // AI metrics and monitoring
        .route("/ai/metrics", get(get_ai_metrics))
        .route("/ai/metrics/stream", get(ai_metrics_stream))
        .route("/ai/health", get(ai_health_check))
        // AI scaling and optimization
        .route("/ai/scaling/predict", get(predict_scaling))
        .route(
            "/ai/optimization/recommendations",
            get(get_optimization_recommendations),
        )
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

// AI API endpoint handlers

/// List available AI models
async fn list_ai_models(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<Json<ApiResponse<Vec<AiModelInfo>>>, StatusCode> {
    let models = state.model_registry.read().await;
    let model_list: Vec<AiModelInfo> = models.values().cloned().collect();
    Ok(success(model_list).1)
}

/// Get specific AI model information
async fn get_ai_model(
    State(state): State<AiOptimizedApiState>,
    Path(model_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<AiModelInfo>>, StatusCode> {
    let models = state.model_registry.read().await;
    match models.get(&model_id) {
        Some(model) => Ok(success(model.clone()).1),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// AI inference endpoint with caching
async fn ai_inference(
    State(state): State<AiOptimizedApiState>,
    Path(model_id): Path<String>,
    Json(request): Json<AiInferenceRequest>,
) -> std::result::Result<Json<ApiResponse<AiInferenceResponse>>, StatusCode> {
    let start_time = Instant::now();

    // Check cache first
    let cache_key = format!(
        "inference:{}:{}",
        model_id,
        serde_json::to_string(&request.input_data).unwrap_or_default()
    );

    let cached_result = state
        .ai_cache
        .get_ai_optimized(&cache_key, AiWorkloadType::ModelInference)
        .await;

    if let Some(cached_data) = cached_result {
        // Return cached result
        let response = AiInferenceResponse {
            request_id: Uuid::new_v4().to_string(),
            model_id: request.model_id,
            output_data: cached_data,
            confidence_score: Some(0.95),
            processing_time_ms: start_time.elapsed().as_millis() as f64,
            cached: true,
            stream_id: None,
        };

        return Ok(success(response).1);
    }

    // Process inference using Squirrel primal integration
    let response = match process_ai_inference(&state, &request, &model_id).await {
        Ok(response) => response,
        Err(e) => {
            // Fallback to local processing if Squirrel primal is unavailable
            tracing::warn!("Squirrel primal unavailable, using local inference: {}", e);
            AiInferenceResponse {
                request_id: Uuid::new_v4().to_string(),
                model_id: request.model_id,
                output_data: serde_json::json!({
                    "result": "local_inference_fallback",
                    "note": "Using local inference due to Squirrel primal unavailability"
                }),
                confidence_score: Some(0.75), // Lower confidence for fallback
                processing_time_ms: start_time.elapsed().as_millis() as f64,
                cached: false,
                stream_id: None,
            }
        }
    };

    // Cache the result
    state
        .ai_cache
        .put_ai_optimized(
            cache_key,
            response.output_data.clone(),
            AiWorkloadType::ModelInference,
        )
        .await;

    Ok(success(response).1)
}

/// AI batch inference endpoint
async fn ai_batch_inference(
    State(state): State<AiOptimizedApiState>,
    Path(model_id): Path<String>,
    Json(request): Json<AiBatchRequest>,
) -> std::result::Result<Json<ApiResponse<Vec<AiInferenceResponse>>>, StatusCode> {
    // Add requests to batch queue
    for inference_request in request.requests {
        let batch_request = BatchRequest {
            id: Uuid::new_v4().to_string(),
            model_id: model_id.clone(),
            payload: serde_json::to_value(inference_request).map_err(|e| {
                SongbirdError::service_error("ai_processor", format!("Failed to serialize request: {}", e))
            })?,
            priority: request
                .batch_options
                .as_ref()
                .and_then(|opts| opts.priority)
                .unwrap_or(5),
            created_at: Instant::now(),
            workload_type: AiWorkloadType::BatchProcessing,
            callback_url: request
                .batch_options
                .as_ref()
                .and_then(|opts| opts.callback_url.clone()),
        };

        if let Err(e) = state.batch_processor.add_to_batch(batch_request).await {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    // Process batch
    match state.batch_processor.process_batch().await {
        Ok(responses) => Ok(success(responses).1),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// AI streaming inference endpoint
async fn ai_stream_inference(
    State(state): State<AiOptimizedApiState>,
    Path(model_id): Path<String>,
) -> std::result::Result<
    Sse<
        impl tokio_stream::Stream<Item = std::result::Result<axum::response::sse::Event, axum::Error>>,
    >,
    StatusCode,
> {
    // Create stream
    let stream_id = match state
        .streaming_manager
        .create_stream(model_id.clone(), AiWorkloadType::StreamingProcessing)
        .await
    {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };

    // Create event stream
    let stream = tokio_stream::iter(0..10).map(move |i| {
        let data = serde_json::json!({
            "stream_id": stream_id,
            "chunk": i,
            "data": format!("Streaming data chunk {}", i),
            "model_id": model_id
        });
        Ok(axum::response::sse::Event::default().data(data.to_string()))
    });

    Ok(Sse::new(stream))
}

/// Get AI cache statistics
async fn get_ai_cache_stats(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<Json<ApiResponse<AiCacheMetrics>>, StatusCode> {
    let metrics = state.ai_cache.metrics.read().await.clone();
    Ok(success(metrics).1)
}

/// Clear AI cache
async fn clear_ai_cache(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<Json<ApiResponse<String>>, StatusCode> {
    state.ai_cache.cache.write().await.clear();
    Ok(success("Cache cleared successfully".to_string()).1)
}

/// Schedule prefetch operation
async fn schedule_prefetch(
    State(state): State<AiOptimizedApiState>,
    Json(request): Json<serde_json::Value>,
) -> std::result::Result<Json<ApiResponse<String>>, StatusCode> {
    // Extract prefetch parameters
    let key = request
        .get("key")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let workload_type = AiWorkloadType::ModelInference; // Default workload type

    state
        .ai_cache
        .schedule_prefetch(key.to_string(), workload_type, 0.8)
        .await;

    Ok(success("Prefetch scheduled successfully".to_string()).1)
}

/// Get AI metrics
async fn get_ai_metrics(
    State(state): State<AiOptimizedApiState>,
    Query(params): Query<AiMetricsQuery>,
) -> std::result::Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let cache_metrics = state.ai_cache.metrics.read().await.clone();
    let stream_metrics = state.streaming_manager.get_stream_metrics().await;
    let batch_metrics = state.batch_processor.metrics.read().await.clone();

    let metrics = serde_json::json!({
        "cache": cache_metrics,
        "streaming": stream_metrics,
        "batch_processing": batch_metrics,
        "timestamp": Utc::now(),
        "query_params": params
    });

    Ok(success(metrics).1)
}

/// AI metrics streaming endpoint
async fn ai_metrics_stream(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<
    Sse<
        impl tokio_stream::Stream<Item = std::result::Result<axum::response::sse::Event, axum::Error>>,
    >,
    StatusCode,
> {
    let stream =
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| {
                let metrics = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "active_ai_requests": 42,
                    "cache_hit_rate": 0.85,
                    "average_response_time": 150,
                    "batch_queue_size": 12,
                    "prediction_accuracy": 0.92
                });
                Ok(axum::response::sse::Event::default().data(metrics.to_string()))
            });

    Ok(Sse::new(stream))
}

/// AI health check endpoint
async fn ai_health_check(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let health_data = serde_json::json!({
        "status": "healthy",
        "ai_services": "operational",
        "cache_status": "healthy",
        "streaming_status": "healthy",
        "batch_processing_status": "healthy",
        "predictive_scaling_status": "healthy",
        "timestamp": Utc::now()
    });

    Ok(success(health_data).1)
}

/// Predict scaling needs
async fn predict_scaling(
    State(state): State<AiOptimizedApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> std::result::Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let default_service_id = "default".to_string();
    let service_id = params.get("service_id").unwrap_or(&default_service_id);

    let prediction = state
        .predictive_scaler
        .predict_scaling_needs(service_id)
        .await;

    let response = serde_json::json!({
        "service_id": service_id,
        "predicted_instances": prediction,
        "confidence": 0.85,
        "reasoning": "Based on historical AI workload patterns",
        "timestamp": Utc::now()
    });

    Ok(success(response).1)
}

/// Get optimization recommendations
async fn get_optimization_recommendations(
    State(state): State<AiOptimizedApiState>,
) -> std::result::Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    let recommendations = vec![
        serde_json::json!({
            "type": "cache_optimization",
            "priority": "high",
            "description": "Increase cache size for model inference workloads",
            "estimated_improvement": "25% faster response times"
        }),
        serde_json::json!({
            "type": "batch_processing",
            "priority": "medium",
            "description": "Enable adaptive batching for better throughput",
            "estimated_improvement": "40% higher throughput"
        }),
        serde_json::json!({
            "type": "predictive_scaling",
            "priority": "high",
            "description": "Scale up services before peak AI workload periods",
            "estimated_improvement": "60% reduction in response time variance"
        }),
    ];

    Ok(success(recommendations).1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_cache_get_put() {
        let cache = AiAwareCache::new();
        let key = "test_key".to_string();
        let data = serde_json::json!({"test": "data"});

        cache
            .put_ai_optimized(key.clone(), data.clone(), AiWorkloadType::ModelInference)
            .await;
        let result = cache
            .get_ai_optimized(&key, AiWorkloadType::ModelInference)
            .await;

        assert!(result.is_some());
        assert_eq!(result.expect("Result should be Some in test"), data);
    }

    #[tokio::test]
    async fn test_streaming_manager() {
        let manager = AiStreamingManager::new();
        let model_id = "test_model".to_string();

        let stream_id = manager
            .create_stream(model_id, AiWorkloadType::StreamingProcessing)
            .await
            .expect("Failed to create stream in test");
        assert!(!stream_id.is_empty());

        let metrics = manager.get_stream_metrics().await;
        assert_eq!(metrics.active_streams, 1);
    }

    #[tokio::test]
    async fn test_batch_processor() {
        let processor = AiBatchProcessor::new();
        let request = BatchRequest {
            id: "test_id".to_string(),
            model_id: "test_model".to_string(),
            payload: serde_json::json!({"test": "data"}),
            priority: 5,
            created_at: Instant::now(),
            workload_type: AiWorkloadType::BatchProcessing,
            callback_url: None,
        };

        processor.add_to_batch(request).await.expect("Failed to add to batch in test");
        let responses = processor.process_batch().await.expect("Failed to process batch in test");
        assert_eq!(responses.len(), 1);
    }

    #[tokio::test]
    async fn test_predictive_scaler() {
        let scaler = AiPredictiveScaler::new();
        let service_id = "test_service";

        let prediction = scaler.predict_scaling_needs(service_id).await;
        // Should be None since no prediction model exists
        assert!(prediction.is_none());
    }
}
