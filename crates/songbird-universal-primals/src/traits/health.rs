//! Health monitoring traits and types for Universal Primals
//!
//! Provides comprehensive health checking, performance metrics, and status reporting
//! for primal services with modern Rust idioms and patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;

/// Health status enumeration with clear semantics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[must_use = "Health status should be handled appropriately"]
pub enum HealthStatus {
    /// Service is healthy and operational
    Healthy,
    /// Service is unhealthy but still running
    Unhealthy,
    /// Service status is unknown or not yet determined
    Unknown,
    /// Service is degraded but partially functional
    Degraded,
    /// Service is completely down
    Down,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Unknown => write!(f, "unknown"),
            Self::Degraded => write!(f, "degraded"),
            Self::Down => write!(f, "down"),
        }
    }
}

/// Comprehensive performance metrics for primal services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetrics {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_usage_percent: Option<f64>,
    /// Memory usage in MB
    pub memory_usage_mb: Option<f64>,
    /// Average response time in milliseconds
    pub response_time_ms: Option<f64>,
    /// Throughput in requests per second
    pub throughput_rps: Option<f64>,
    /// Error rate percentage (0.0-100.0)
    pub error_rate: Option<f64>,
    /// Current queue depth
    pub queue_depth: Option<u64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: None,
            memory_usage_mb: None,
            response_time_ms: None,
            throughput_rps: None,
            error_rate: None,
            queue_depth: None,
        }
    }
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cpu_usage_percent: None,
            memory_usage_mb: None,
            response_time_ms: None,
            throughput_rps: None,
            error_rate: None,
            queue_depth: None,
        }
    }

    /// Check if all metrics are within healthy ranges
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        let cpu_healthy = self.cpu_usage_percent.map_or(true, |cpu| cpu < 80.0);
        let memory_healthy = self.memory_usage_mb.map_or(true, |mem| mem < 1000.0);
        let response_healthy = self.response_time_ms.map_or(true, |resp| resp < 1000.0);
        let error_healthy = self.error_rate.map_or(true, |err| err < 5.0);

        cpu_healthy && memory_healthy && response_healthy && error_healthy
    }
}

/// Detailed health information for a primal service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Timestamp of last health check
    pub timestamp: DateTime<Utc>,
    /// Custom health metrics
    pub metrics: HashMap<String, f64>,
    /// Detailed health information
    pub details: Vec<HealthDetail>,
    /// Service uptime in seconds
    pub uptime_seconds: Option<u64>,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

impl Default for PrimalHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            details: Vec::new(),
            uptime_seconds: None,
            last_error: None,
            performance: PerformanceMetrics::default(),
        }
    }
}

impl PrimalHealth {
    /// Create a new healthy status
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            details: Vec::new(),
            uptime_seconds: None,
            last_error: None,
            performance: PerformanceMetrics::default(),
        }
    }

    /// Create an unhealthy status with error message
    #[must_use]
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            details: Vec::new(),
            uptime_seconds: None,
            last_error: Some(error.into()),
            performance: PerformanceMetrics::default(),
        }
    }

    /// Create a degraded status with optional message
    #[must_use]
    pub fn degraded(message: Option<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            details: Vec::new(),
            uptime_seconds: None,
            last_error: message,
            performance: PerformanceMetrics::default(),
        }
    }

    /// Add a health detail
    pub fn add_detail(&mut self, detail: HealthDetail) {
        self.details.push(detail);
    }

    /// Add a custom metric
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }

    /// Update the timestamp to current time
    pub fn refresh_timestamp(&mut self) {
        self.timestamp = Utc::now();
    }

    /// Check if the service is considered healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy) && self.performance.is_healthy()
    }

    /// Get a summary of health issues
    #[must_use]
    pub fn health_summary(&self) -> String {
        match self.status {
            HealthStatus::Healthy => "Service is operating normally".to_string(),
            HealthStatus::Unhealthy => self.last_error.as_ref().map_or_else(
                || "Service is unhealthy".to_string(),
                |err| format!("Service is unhealthy: {err}"),
            ),
            HealthStatus::Degraded => self.last_error.as_ref().map_or_else(
                || "Service is degraded".to_string(),
                |err| format!("Service is degraded: {err}"),
            ),
            HealthStatus::Down => "Service is down".to_string(),
            HealthStatus::Unknown => "Service status is unknown".to_string(),
        }
    }
}

/// Detailed health check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetail {
    /// Name of the health check
    pub name: String,
    /// Health status for this specific check
    pub status: HealthStatus,
    /// Descriptive message
    pub message: String,
    /// Measured value (if applicable)
    pub value: Option<f64>,
    /// Threshold value for comparison
    pub threshold: Option<f64>,
    /// Timestamp when this detail was created
    pub timestamp: DateTime<Utc>,
}

impl HealthDetail {
    /// Create a new health detail
    #[must_use]
    pub fn new(name: impl Into<String>, status: HealthStatus, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            value: None,
            threshold: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a health detail with value and threshold
    #[must_use]
    pub fn with_threshold(
        name: impl Into<String>,
        status: HealthStatus,
        message: impl Into<String>,
        value: f64,
        threshold: f64,
    ) -> Self {
        Self {
            name: name.into(),
            status,
            message: message.into(),
            value: Some(value),
            threshold: Some(threshold),
            timestamp: Utc::now(),
        }
    }

    /// Check if this detail indicates a healthy state
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthStatus::Healthy)
    }

    /// Check if the value exceeds the threshold (if both are present)
    #[must_use]
    pub fn exceeds_threshold(&self) -> Option<bool> {
        match (self.value, self.threshold) {
            (Some(value), Some(threshold)) => Some(value > threshold),
            _ => None,
        }
    }
}

/// Health monitoring trait for primal services
#[async_trait::async_trait]
pub trait PrimalHealthMonitor: Send + Sync {
    /// Get the current health status
    async fn get_health(&self) -> SongbirdResult<PrimalHealth>;

    /// Perform a comprehensive health check
    async fn health_check(&self) -> SongbirdResult<PrimalHealth>;

    /// Get performance metrics
    async fn get_metrics(&self) -> SongbirdResult<PerformanceMetrics>;

    /// Check if the service is ready to handle requests
    async fn is_ready(&self) -> SongbirdResult<bool>;

    /// Check if the service is alive (basic connectivity)
    async fn is_alive(&self) -> SongbirdResult<bool>;
}

/// Default implementation for basic health monitoring
#[derive(Debug, Clone)]
pub struct DefaultHealthMonitor {
    service_name: String,
    start_time: DateTime<Utc>,
}

impl DefaultHealthMonitor {
    /// Create a new default health monitor
    #[must_use]
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            start_time: Utc::now(),
        }
    }

    /// Calculate uptime in seconds
    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds().max(0) as u64
    }
}

#[async_trait::async_trait]
impl PrimalHealthMonitor for DefaultHealthMonitor {
    async fn get_health(&self) -> SongbirdResult<PrimalHealth> {
        let mut health = PrimalHealth::healthy();
        health.uptime_seconds = Some(self.uptime_seconds());
        health.add_detail(HealthDetail::new(
            "uptime",
            HealthStatus::Healthy,
            format!(
                "Service {} has been running for {} seconds",
                self.service_name,
                self.uptime_seconds()
            ),
        ));
        Ok(health)
    }

    async fn health_check(&self) -> SongbirdResult<PrimalHealth> {
        self.get_health().await
    }

    async fn get_metrics(&self) -> SongbirdResult<PerformanceMetrics> {
        Ok(PerformanceMetrics::default())
    }

    async fn is_ready(&self) -> SongbirdResult<bool> {
        Ok(true)
    }

    async fn is_alive(&self) -> SongbirdResult<bool> {
        Ok(true)
    }
}
