//! Observability Traits
//!
//! Universal observability patterns for tracing, metrics, and monitoring

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::errors::Result;

/// Universal request context for tracing and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Unique trace identifier for request correlation
    pub trace_id: String,
    
    /// Span identifier within the trace
    pub span_id: String,
    
    /// Parent span identifier for nested operations
    pub parent_span_id: Option<String>,
    
    /// Request baggage for cross-service context
    pub baggage: HashMap<String, String>,
    
    /// Request start timestamp
    pub started_at: DateTime<Utc>,
    
    /// Service-specific metadata
    pub service_metadata: HashMap<String, serde_json::Value>,
    
    /// User/session context
    pub user_context: Option<UserContext>,
}

impl RequestContext {
    /// Create a new root request context
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
            started_at: Utc::now(),
            service_metadata: HashMap::new(),
            user_context: None,
        }
    }
    
    /// Create a child span from this context
    pub fn child_span(&self, operation: &str) -> Self {
        let mut child = self.clone();
        child.parent_span_id = Some(child.span_id.clone());
        child.span_id = Uuid::new_v4().to_string();
        child.baggage.insert("operation".to_string(), operation.to_string());
        child
    }
    
    /// Add baggage item for cross-service context
    pub fn with_baggage(mut self, key: &str, value: &str) -> Self {
        self.baggage.insert(key.to_string(), value.to_string());
        self
    }
}

/// User context for request attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub session_id: Option<String>,
    pub roles: Vec<String>,
    pub attributes: HashMap<String, String>,
}

/// Span information for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<LogEntry>,
    pub status: SpanStatus,
}

/// Span completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error { message: String },
    Cancelled,
    DeadlineExceeded,
}

/// Log entry within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Universal tracing provider trait
#[async_trait]
pub trait TracingProvider: Send + Sync {
    /// Start a new span
    async fn start_span(&self, context: &RequestContext, operation: &str) -> Result<Span>;
    
    /// Finish a span with status
    async fn finish_span(&self, span: Span, status: SpanStatus) -> Result<()>;
    
    /// Add tags to a span
    async fn add_span_tags(&self, span_id: &str, tags: HashMap<String, String>) -> Result<()>;
    
    /// Log an event within a span
    async fn log_event(&self, span_id: &str, entry: LogEntry) -> Result<()>;
    
    /// Extract context from headers/metadata
    fn extract_context(&self, headers: &HashMap<String, String>) -> Option<RequestContext>;
    
    /// Inject context into headers/metadata
    fn inject_context(&self, context: &RequestContext) -> HashMap<String, String>;
    
    /// Get provider information
    fn provider_info(&self) -> TracingProviderInfo;
}

/// Tracing provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingProviderInfo {
    pub name: String,
    pub version: String,
    pub supports_distributed: bool,
    pub supports_sampling: bool,
    pub backend_type: String,
}

/// Universal metrics provider trait
#[async_trait]
pub trait MetricsProvider: Send + Sync {
    /// Record a counter metric
    async fn record_counter(&self, name: &str, value: u64, tags: &HashMap<String, String>) -> Result<()>;
    
    /// Record a gauge metric
    async fn record_gauge(&self, name: &str, value: f64, tags: &HashMap<String, String>) -> Result<()>;
    
    /// Record a histogram metric
    async fn record_histogram(&self, name: &str, value: f64, tags: &HashMap<String, String>) -> Result<()>;
    
    /// Record a timer metric
    async fn record_timer(&self, name: &str, duration: Duration, tags: &HashMap<String, String>) -> Result<()>;
    
    /// Get current metric values
    async fn get_metrics(&self, filter: Option<&str>) -> Result<HashMap<String, MetricValue>>;
    
    /// Get provider information
    fn provider_info(&self) -> MetricsProviderInfo;
}

/// Metric value enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { 
        count: u64, 
        sum: f64, 
        buckets: Vec<HistogramBucket> 
    },
    Timer {
        count: u64,
        sum: Duration,
        percentiles: HashMap<String, Duration>,
    },
}

/// Histogram bucket for distribution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Metrics provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsProviderInfo {
    pub name: String,
    pub version: String,
    pub supports_histograms: bool,
    pub supports_tags: bool,
    pub backend_type: String,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Tracing configuration
    pub tracing: TracingConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
    
    /// Sampling configuration
    pub sampling: SamplingConfig,
    
    /// Export configuration
    pub export: ExportConfig,
}

/// Tracing-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub provider: String,
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub resource_attributes: HashMap<String, String>,
}

/// Metrics-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub provider: String,
    pub collection_interval: Duration,
    pub export_interval: Duration,
    pub default_tags: HashMap<String, String>,
}

/// Sampling configuration for trace data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub strategy: SamplingStrategy,
    pub rate: f64,
    pub max_traces_per_second: Option<u64>,
}

/// Sampling strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingStrategy {
    Always,
    Never,
    Probabilistic,
    RateLimited,
    Adaptive,
}

/// Export configuration for observability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub endpoint: Option<String>,
    pub headers: HashMap<String, String>,
    pub compression: bool,
    pub batch_size: usize,
    pub export_timeout: Duration,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            tracing: TracingConfig {
                enabled: true,
                provider: "console".to_string(),
                service_name: "songbird-orchestrator".to_string(),
                service_version: "0.1.0".to_string(),
                environment: "development".to_string(),
                resource_attributes: HashMap::new(),
            },
            metrics: MetricsConfig {
                enabled: true,
                provider: "prometheus".to_string(),
                collection_interval: Duration::from_secs(10),
                export_interval: Duration::from_secs(30),
                default_tags: HashMap::new(),
            },
            sampling: SamplingConfig {
                strategy: SamplingStrategy::Probabilistic,
                rate: 0.1,
                max_traces_per_second: Some(100),
            },
            export: ExportConfig {
                endpoint: None,
                headers: HashMap::new(),
                compression: true,
                batch_size: 100,
                export_timeout: Duration::from_secs(10),
            },
        }
    }
} 