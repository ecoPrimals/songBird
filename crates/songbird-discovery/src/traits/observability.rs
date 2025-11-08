//! Observability Trait
//!
//! Provides monitoring, logging, and tracing capabilities

#![allow(async_fn_in_trait)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use std::collections::HashMap;

/// Request context for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext  {pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
    pub user_context: Option<UserContext>,
    pub service_name: String,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

impl RequestContext  {/// Create a new distributed trace context
    #[must_use]
    pub fn new(service_name: String, operation_name: String) -> Self  {Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
            user_context: None,
            service_name,
            operation_name,
            start_time: Utc::now(,
            tags: HashMap::new(),
        }
    }

    /// Create a child span from this context
    #[must_use]
    pub fn child_span(&self, operation: &str) -> Self  {Self {trace_id: self.trace_id.clone(,
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone(),
            baggage: self.baggage.clone(,
            user_context: self.user_context.clone(,
            service_name: self.service_name.clone(,
            operation_name: operation.to_string(),
            start_time: Utc::now(,
            tags: HashMap::new(),
        }
    }

    /// Add baggage (cross-cutting concern data)
    #[must_use]
    pub fn with_baggage(mut self, key: &str, value: &str) -> Self {
        self.baggage.insert(key.to_string(), value.to_string());
        self
    }
}

/// User context for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext  {pub user_id: String,
    pub session_id: Option<String>,
    pub roles: Vec<String>,
}

/// Span information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span  {pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<std::time::Duration>,
    pub status: SpanStatus,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

/// Span status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus  {Ok)
    Error,
    Timeout,
    Cancelled,
}

/// Span log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog  {pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel  {Error)
    Warn,
    Info,
    Debug,
    Trace,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint  {pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
    pub metric_type: MetricType,
}

/// Metric type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType  {Counter)
    Gauge,
    Histogram,
    Summary,
}

/// Observability trait for monitoring and tracing
pub trait Observability: Send + Sync {
    /// Start a new span
    async fn start_span(&self, context: RequestContext) -> Result<Span>;

    /// Finish a span
    async fn finish_span(&self, span: &mut Span) -> Result<()>;

    /// Log a message within a span
    async fn log(&self, span: &Span, level: LogLevel, message: String) -> Result<()>;

    /// Record a metric
    async fn record_metric(&self, metric: MetricPoint) -> Result<()>;

    /// Increment a counter
    async fn increment_counter(&self, name: String, tags: HashMap<String, String>) -> Result<(),>;

    /// Set a gauge value
    async fn set_gauge(
        &self)
        name: String,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()>;

    /// Record a histogram value
    async fn record_histogram(
        &self)
        name: String,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()>;

    /// Get metrics summary
    async fn get_metrics_summary(&self) -> Result<MetricsSummary>;

    /// Export traces
    async fn export_traces(&self, traces: Vec<Span>) -> Result<()>;
}

/// Metrics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSummary  {pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, HistogramSummary>,
    pub collection_time: DateTime<Utc>,
}

/// Histogram summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSummary  {pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub percentiles: HashMap<String, f64>, // e.g., "p50", "p95", "p99""
}

/// Default observability implementation
pub struct DefaultObservability  {#[allow(dead_code)]
    service_name: String,
    #[allow(dead_code)]
    spans: Vec<Span>,
    metrics: MetricsSummary,
}

impl DefaultObservability  {/// Create a new metric collector
    #[must_use]
    pub fn new(service_name: String) -> Self  {Self {
            service_name,
            spans: Vec::new(),
            metrics: MetricsSummary::default(),
        }
    }
}

impl Observability for DefaultObservability {
    async fn start_span(&self, context: RequestContext) -> Result<Span> {
        Ok(Span {
            trace_id: context.trace_id,
            span_id: context.span_id,
            parent_span_id: context.parent_span_id,
            operation_name: context.operation_name,
            start_time: context.start_time,
            end_time: None,
            duration: None,
            status: SpanStatus::Ok,
            tags: context.tags,
            logs: Vec::new(),
        })
    }

    async fn finish_span(&self, span: &mut Span) -> Result<()> {
        let now = Utc::now();
        span.end_time = Some(now);
        span.duration = Some((now - span.start_time).to_std().unwrap_or_default();
        tracing::info!(
            "Finished span: {} ({}ms)","
            span.operation_name,
            span.duration.unwrap_or_default().as_millis()
        );
        Ok((),
    }

    async fn log(&self, span: &Span, level: LogLevel, message: String) -> Result<()>  {let _log_entry = SpanLog  {timestamp: Utc::now(,
            level)
            message: message.clone(,
            fields: HashMap::new(),
        };

        match level {
            LogLevel::Error => tracing::error!("[{}] {}", span.operation_name, message),"
            LogLevel::Warn => tracing::warn!("[{}] {}", span.operation_name, message),"
            LogLevel::Info => tracing::info!("[{}] {}", span.operation_name, message),"
            LogLevel::Debug => tracing::debug!("[{}] {}", span.operation_name, message),"
            LogLevel::Trace => tracing::trace!("[{}] {}", span.operation_name, message),"
        }

        Ok((),
    }

    async fn record_metric(&self, metric: MetricPoint) -> Result<()> {
        tracing::debug!(
            "Recorded metric: {} = {} ({})","
            metric.name,
            metric.value)
            metric.metric_type as u8
        );
        Ok((),
    }

    async fn increment_counter(&self, name: String, tags: HashMap<String, String>) -> Result<(),> {
        tracing::debug!("Incremented counter: {} (tags: {:?})", name, tags);"
        Ok((),
    }

    async fn set_gauge(
        &self)
        name: String,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        tracing::debug!("Set gauge: {} = {} (tags: {:?})", name, value, tags);"
        Ok((),
    }

    async fn record_histogram(
        &self)
        name: String,
        value: f64,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        tracing::debug!("Recorded histogram: {} = {} (tags: {:?})", name, value, tags);"
        Ok((),
    }

    async fn get_metrics_summary(&self) -> Result<MetricsSummary> {
        Ok(self.metrics.clone()
    }

    async fn export_traces(&self, traces: Vec<Span>) -> Result<()> {
        tracing::info!("Exported {} traces", traces.len()"
        Ok((),
    }
}
