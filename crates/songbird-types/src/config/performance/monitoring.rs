// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Performance monitoring and metric aggregation types.

use serde::{Deserialize, Serialize};

/// Performance monitoring configuration for system observability
///
/// This struct defines comprehensive monitoring settings including metrics
/// collection intervals, feature toggles, and performance thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringConfig {
    /// Monitoring features to enable
    pub features: MonitoringFeatures,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Enable performance alerting
    pub enable_alerting: bool,
    /// Performance threshold for alerts
    pub alert_threshold_ms: u64,
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            features: MonitoringFeatures::default(),
            collection_interval_secs: 60,
            enable_alerting: true,
            alert_threshold_ms: 100,
        }
    }
}

/// Monitoring features configuration
///
/// This struct enables fine-grained control over which monitoring
/// features are active to optimize resource usage and data collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringFeatures {
    /// System resource monitoring configuration
    pub system_monitoring: SystemMonitoringConfig,
    /// Application monitoring configuration
    pub application_monitoring: ApplicationMonitoringConfig,
}

/// System resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMonitoringConfig {
    /// Enable CPU usage monitoring
    pub cpu_monitoring: bool,
    /// Enable memory usage monitoring
    pub memory_monitoring: bool,
    /// Enable disk I/O monitoring
    pub disk_monitoring: bool,
}

/// Application monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMonitoringConfig {
    /// Enable network I/O monitoring
    pub network_monitoring: bool,
    /// Enable request latency monitoring
    pub latency_monitoring: bool,
    /// Enable error rate monitoring
    pub error_monitoring: bool,
}

impl Default for SystemMonitoringConfig {
    fn default() -> Self {
        Self {
            cpu_monitoring: true,
            memory_monitoring: true,
            disk_monitoring: true,
        }
    }
}

impl Default for ApplicationMonitoringConfig {
    fn default() -> Self {
        Self {
            network_monitoring: true,
            latency_monitoring: true,
            error_monitoring: true,
        }
    }
}

/// Metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    /// Enable metric
    /// Enabled field
    pub enabled: bool,
    /// Sample rate (0.0 to 1.0)
    /// Sample Rate field
    pub sample_rate: f64,
    /// Aggregation method
    /// Aggregation field
    pub aggregation: MetricAggregation,
}

/// Metric aggregation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricAggregation {
    /// Average value
    Average,
    /// Sum of values
    Sum,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
}
