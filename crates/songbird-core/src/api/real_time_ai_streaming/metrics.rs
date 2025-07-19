//! Performance monitoring and metrics collection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System metrics snapshot for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetricsSnapshot {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage percentage (0.0 - 100.0)  
    pub memory_usage_percent: f64,
    /// Network usage in Mbps
    pub network_usage_mbps: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Messages per second throughput
    pub messages_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate percentage (0.0 - 100.0)
    pub error_rate_percent: f64,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
}

/// Response time percentile measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePercentiles {
    /// 50th percentile response time
    pub p50_ms: f64,
    /// 95th percentile response time
    pub p95_ms: f64,
    /// 99th percentile response time
    pub p99_ms: f64,
}

/// Streaming performance monitor
pub struct StreamingPerformanceMonitor {
    /// Performance metrics history
    pub metrics_history: Vec<SystemMetricsSnapshot>,
    /// Current active sessions
    pub active_sessions: HashMap<String, SessionMetrics>,
    /// Connection quality tracking
    pub connection_quality: HashMap<String, super::connection::ConnectionQualityMetrics>,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

/// Metrics for individual sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Session identifier
    pub session_id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Number of participants
    pub participant_count: u32,
    /// Messages exchanged in session
    pub messages_exchanged: u64,
    /// Total session duration (seconds)
    pub duration_seconds: u64,
    /// Average response time in session
    pub avg_response_time_ms: f64,
    /// Session efficiency score (0.0 - 1.0)
    pub efficiency_score: f64,
    /// Collaboration quality score (0.0 - 1.0)
    pub collaboration_quality: f64,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage alert threshold (percentage)
    pub cpu_usage_threshold: f64,
    /// Memory usage alert threshold (percentage)
    pub memory_usage_threshold: f64,
    /// Response time alert threshold (milliseconds)
    pub response_time_threshold: f64,
    /// Error rate alert threshold (percentage)
    pub error_rate_threshold: f64,
    /// Connection quality threshold (0.0 - 1.0)
    pub connection_quality_threshold: f64,
}

/// Performance alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert identifier
    pub alert_id: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert severity
    pub severity: super::service_mesh::EventSeverity,
    /// Alert message
    pub message: String,
    /// Metric that triggered the alert
    pub triggering_metric: String,
    /// Current metric value
    pub current_value: f64,
    /// Threshold that was exceeded
    pub threshold_value: f64,
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighCPUUsage,
    HighMemoryUsage,
    HighResponseTime,
    HighErrorRate,
    LowConnectionQuality,
    ConnectionLoss,
    ServiceUnavailable,
    CapacityExceeded,
}

/// Connection health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHealthMetrics {
    /// Connection identifier
    pub connection_id: String,
    /// Connection uptime in seconds
    pub uptime_seconds: u64,
    /// Number of disconnections
    pub disconnection_count: u32,
    /// Average reconnection time (seconds)
    pub avg_reconnection_time_seconds: f64,
    /// Data transfer metrics
    pub data_transfer: DataTransferMetrics,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
}

/// Data transfer metrics for connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransferMetrics {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Transfer rate (bytes per second)
    pub transfer_rate_bps: f64,
}

impl StreamingPerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            active_sessions: HashMap::new(),
            connection_quality: HashMap::new(),
            alert_thresholds: AlertThresholds::default(),
        }
    }

    /// Record system metrics snapshot
    pub fn record_metrics(&mut self, metrics: SystemMetricsSnapshot) {
        // Keep only last 1000 snapshots
        if self.metrics_history.len() >= 1000 {
            self.metrics_history.remove(0);
        }
        self.metrics_history.push(metrics);
    }

    /// Update session metrics
    pub fn update_session_metrics(&mut self, session_id: String, metrics: SessionMetrics) {
        self.active_sessions.insert(session_id, metrics);
    }

    /// Remove session metrics (when session ends)
    pub fn remove_session(&mut self, session_id: &str) {
        self.active_sessions.remove(session_id);
    }

    /// Update connection quality
    pub fn update_connection_quality(
        &mut self,
        connection_id: String,
        quality: super::connection::ConnectionQualityMetrics,
    ) {
        self.connection_quality.insert(connection_id, quality);
    }

    /// Check for alerts based on current metrics
    pub fn check_alerts(&self, current_metrics: &SystemMetricsSnapshot) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // CPU usage alert
        if current_metrics.cpu_usage_percent > self.alert_thresholds.cpu_usage_threshold {
            alerts.push(PerformanceAlert {
                alert_id: uuid::Uuid::new_v4().to_string(),
                alert_type: AlertType::HighCPUUsage,
                severity: super::service_mesh::EventSeverity::Warning,
                message: format!(
                    "High CPU usage detected: {:.1}%",
                    current_metrics.cpu_usage_percent
                ),
                triggering_metric: "cpu_usage_percent".to_string(),
                current_value: current_metrics.cpu_usage_percent,
                threshold_value: self.alert_thresholds.cpu_usage_threshold,
                timestamp: Utc::now(),
                suggested_actions: vec![
                    "Check for resource-intensive processes".to_string(),
                    "Consider scaling up resources".to_string(),
                ],
            });
        }

        // Memory usage alert
        if current_metrics.memory_usage_percent > self.alert_thresholds.memory_usage_threshold {
            alerts.push(PerformanceAlert {
                alert_id: uuid::Uuid::new_v4().to_string(),
                alert_type: AlertType::HighMemoryUsage,
                severity: super::service_mesh::EventSeverity::Warning,
                message: format!(
                    "High memory usage detected: {:.1}%",
                    current_metrics.memory_usage_percent
                ),
                triggering_metric: "memory_usage_percent".to_string(),
                current_value: current_metrics.memory_usage_percent,
                threshold_value: self.alert_thresholds.memory_usage_threshold,
                timestamp: Utc::now(),
                suggested_actions: vec![
                    "Check for memory leaks".to_string(),
                    "Consider increasing memory allocation".to_string(),
                ],
            });
        }

        // Response time alert
        if current_metrics.avg_response_time_ms > self.alert_thresholds.response_time_threshold {
            alerts.push(PerformanceAlert {
                alert_id: uuid::Uuid::new_v4().to_string(),
                alert_type: AlertType::HighResponseTime,
                severity: super::service_mesh::EventSeverity::Error,
                message: format!(
                    "High response time detected: {:.1}ms",
                    current_metrics.avg_response_time_ms
                ),
                triggering_metric: "avg_response_time_ms".to_string(),
                current_value: current_metrics.avg_response_time_ms,
                threshold_value: self.alert_thresholds.response_time_threshold,
                timestamp: Utc::now(),
                suggested_actions: vec![
                    "Check for performance bottlenecks".to_string(),
                    "Review database query performance".to_string(),
                ],
            });
        }

        alerts
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let latest_metrics = self.metrics_history.last();

        PerformanceSummary {
            total_sessions: self.active_sessions.len() as u32,
            active_connections: latest_metrics.map(|m| m.active_connections).unwrap_or(0),
            avg_response_time_ms: latest_metrics
                .map(|m| m.avg_response_time_ms)
                .unwrap_or(0.0),
            system_health_score: self.calculate_health_score(),
            last_updated: Utc::now(),
        }
    }

    /// Calculate overall system health score (0.0 - 1.0)
    fn calculate_health_score(&self) -> f64 {
        if let Some(latest) = self.metrics_history.last() {
            let cpu_score = (100.0 - latest.cpu_usage_percent) / 100.0;
            let memory_score = (100.0 - latest.memory_usage_percent) / 100.0;
            let error_score = (100.0 - latest.error_rate_percent) / 100.0;

            (cpu_score + memory_score + error_score) / 3.0
        } else {
            0.0
        }
    }
}

/// Performance summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total active sessions
    pub total_sessions: u32,
    /// Active connections count
    pub active_connections: u32,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Overall system health score (0.0 - 1.0)
    pub system_health_score: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_threshold: 80.0,         // 80% CPU
            memory_usage_threshold: 85.0,      // 85% Memory
            response_time_threshold: 1000.0,   // 1 second
            error_rate_threshold: 5.0,         // 5% error rate
            connection_quality_threshold: 0.8, // 80% quality
        }
    }
}

impl Default for SystemMetricsSnapshot {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            network_usage_mbps: 0.0,
            active_connections: 0,
            messages_per_second: 0.0,
            avg_response_time_ms: 0.0,
            error_rate_percent: 0.0,
            timestamp: Utc::now(),
        }
    }
}
