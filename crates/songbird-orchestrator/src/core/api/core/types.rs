// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// PEDANTIC: Consider adding ErrorSeverity and ErrorClassification for AI automation
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// Import canonical types
use crate::biome::ServiceHealth;
use crate::orchestrator::OrchestratorMetrics;
use crate::traits::communication::{CommunicationStats, MessageType};
use crate::traits::ServiceInfo;

/// Canonical API response structure
///
/// This replaces the generic ApiResponse<T> with a unified structure
/// that uses serde_json::Value for flexible data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct ApiResponse {
    /// Success field
    pub success: bool,
    /// Message field
    pub message: String,
    /// Data field
    pub data: Option<serde_json::Value>,
    /// Timestamp when this was created or last updated
    pub timestamp: DateTime<Utc>,
}

impl ApiResponse {
    /// Create a successful response with data
    #[must_use]
    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            timestamp: Utc::now(),
        }
    }

    /// Create a successful response without data
    #[must_use]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            timestamp: Utc::now(),
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            timestamp: Utc::now(),
        }
    }
}

/// Service registration request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterServiceRequest {
    /// Name identifier
    pub name: String,
    /// Service Type field
    pub service_type: String,
    /// Version string
    pub version: String,
    /// Human-readable description
    pub description: Option<String>,
    /// **MIGRATION COMPLETE**: Using String for endpoints
    pub endpoints: Option<Vec<String>>,
    /// Capabilities field
    pub capabilities: Option<Vec<String>>,
    /// Tags field
    pub tags: Option<HashMap<String, String>>,
    /// Metadata field
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
/// Service operation request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceOperationRequest {
    /// Operation field
    pub operation: String,
    /// Parameters field
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Message send request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SendMessageRequest {
    /// Target Service field
    pub target_service: String,
    /// Message Type field
    pub message_type: MessageType,
    /// Topic field
    pub topic: Option<String>,
    /// Payload field
    pub payload: serde_json::Value,
    /// Headers field
    pub headers: Option<HashMap<String, String>>,
    /// Ttl field
    pub ttl: Option<u64>,
}

/// Broadcast message request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastMessageRequest {
    /// Message Type field
    pub message_type: MessageType,
    /// Topic field
    pub topic: Option<String>,
    /// Payload field
    pub payload: serde_json::Value,
    /// Headers field
    pub headers: Option<HashMap<String, String>>,
    /// Ttl field
    pub ttl: Option<u64>,
}
/// System information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Total Services field
    pub total_services: u64,
    /// Healthy Services field
    pub healthy_services: u64,
    /// Number of currently active connections
    pub active_connections: u64,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Api Endpoints field
    pub api_endpoints: Vec<String>,
}

/// Dashboard data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// System Info field
    pub system_info: SystemInfo,
    /// Orchestrator Metrics field
    pub orchestrator_metrics: OrchestratorMetrics,
    /// Services field
    pub services: Vec<ServiceInfo>,
    /// Communication Stats field
    pub communication_stats: CommunicationStats,
    /// Recent Events field
    pub recent_events: Vec<ApiEvent>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"]
pub struct HealthCheckResponse {
    /// Current status of the operation or entity
    pub status: String,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error Message field
    pub error_message: Option<String>,
}

/// Query parameters for metrics endpoints
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    /// Service Id field
    pub service_id: Option<String>,
    /// Start Time field
    pub start_time: Option<String>,
    /// End Time field
    pub end_time: Option<String>,
    /// Interval field
    pub interval: Option<String>,
}
/// API events for real-time streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiEvent {
    /// Service started event
    ServiceStarted {
        /// Service ID
        service_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Service stopped event
    ServiceStopped {
        /// Service ID
        service_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Service health changed event
    ServiceHealthChanged {
        /// Service ID
        service_id: String,
        // health: ServiceHealth,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Metrics update event
    MetricsUpdate {
        /// Metrics
        metrics: OrchestratorMetrics,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    // AI-specific events
    /// AI cache hit event
    AiCacheHit {
        /// Cache key
        key: String,
        /// Workload type
        workload_type: Option<String>,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Batch processed event
    BatchProcessed {
        /// Batch ID
        batch_id: String,
        /// Batch size
        batch_size: usize,
        /// Processing time in milliseconds
        processing_time_ms: f64,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
    /// Workload pattern detected event
    WorkloadPatternDetected {
        /// Pattern type
        pattern_type: String,
        /// Confidence
        confidence: f32,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
}

/// Canonical AI request configuration
///
/// Modern unified interface for AI-powered service operations
/// that replaces fragmented configuration patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequestContext {
    /// Request Id field
    pub request_id: String,
    /// Timestamp when this was created or last updated
    pub timestamp: SystemTime,
    /// User Agent field
    pub user_agent: Option<String>,
    /// Workload Type field
    pub workload_type: Option<String>,
    /// Priority field
    pub priority: u8,
    /// Enable Caching field
    pub enable_caching: bool,
    /// Enable Streaming field
    pub enable_streaming: bool,
    /// Batch Eligible field
    pub batch_eligible: bool,
}

/// Request batch for AI processing
#[derive(Debug)]
pub struct RequestBatch {
    /// Requests field
    pub requests: Vec<BatchedRequest>,
    /// Created At field
    pub created_at: SystemTime,
    /// Max Wait Time field
    pub max_wait_time: Duration,
    /// Priority field
    pub priority: u8,
}

/// Individual request in a batch
#[derive(Debug)]
pub struct BatchedRequest {
    /// Request Id field
    pub request_id: String,
    /// Endpoint field
    pub endpoint: String,
    /// Payload field
    pub payload: serde_json::Value,
    /// Context field
    pub context: AiRequestContext,
    /// Response Sender field
    pub response_sender: tokio::sync::oneshot::Sender<SongbirdResult<serde_json::Value>>,
}
/// Cache performance metrics
#[derive(Debug, Default, Clone, Serialize)]
pub struct CacheMetrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Cache Hits field
    pub cache_hits: u64,
    /// Cache Misses field
    pub cache_misses: u64,
    /// Prediction Hits field
    pub prediction_hits: u64,
    /// Average response time
    pub average_response_time: Duration,
    /// Memory Usage Mb field
    pub memory_usage_mb: f64,
}

/// Batch processing metrics
#[derive(Debug, Default, Clone, Serialize)]
pub struct BatchMetrics {
    /// Total Batches field
    pub total_batches: u64,
    /// Successful Batches field
    pub successful_batches: u64,
    /// Average Batch Size field
    pub average_batch_size: f64,
    /// Average Processing Time field
    pub average_processing_time: Duration,
    /// Throughput Requests Per Second field
    pub throughput_requests_per_second: f64,
}

/// API performance metrics optimized for AI workloads
#[derive(Debug, Default, Clone, Serialize)]
pub struct ApiPerformanceMetrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Ai Requests field
    pub ai_requests: u64,
    /// Streaming Requests field
    pub streaming_requests: u64,
    /// Batch Requests field
    pub batch_requests: u64,
    /// Average Response Time field
    pub average_response_time: Duration,
    /// P95 Response Time field
    pub p95_response_time: Duration,
    /// P99 Response Time field
    pub p99_response_time: Duration,
    /// Error Rate field
    pub error_rate: f64,
    /// Cache Hit Rate field
    pub cache_hit_rate: f64,
    /// Requests Per Second field
    pub requests_per_second: f64,
    /// Number of currently active connections
    pub active_connections: u64,
}

/// Workload pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPattern {
    /// Request Rate field
    pub request_rate: f64,
    /// Peak Hours field
    pub peak_hours: Vec<u8>,
    /// Resource Usage field
    pub resource_usage: ResourceUsage,
    /// Prediction Model field
    pub prediction_model: Option<String>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Cpu Percent field
    pub cpu_percent: f64,
    /// Memory Mb field
    pub memory_mb: f64,
    /// Network Mbps field
    pub network_mbps: f64,
    /// Gpu Percent field
    pub gpu_percent: Option<f64>,
}
