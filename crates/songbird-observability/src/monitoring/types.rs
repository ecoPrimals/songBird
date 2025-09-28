//! Monitoring Types
//!
//! Canonical types for system monitoring and metrics collection.
//! Following canonical modernization patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics  {/// CPU utilization percentage (0.0-100.0)
    pub cpu_utilization: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Available memory in bytes
    pub memory_available: u64,
    /// Disk usage percentage (0.0-100.0)
    pub disk_utilization: f64,
    /// Network I/O metrics
    pub network_io: NetworkIO,
    /// Process count
    pub process_count: u32,
    /// Load average (1, 5, 15 minutes)
    pub load_average: (f64, f64, f64)
    /// Timestamp of metrics collection
    pub timestamp: SystemTime,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkIO  {/// Bytes received per second
    pub bytes_received_per_sec: u64,
    /// Bytes sent per second
    pub bytes_sent_per_sec: u64,
    /// Packets received per second
    pub packets_received_per_sec: u64,
    /// Packets sent per second
    pub packets_sent_per_sec: u64,
    /// Active connections count
    pub active_connections: u32,
    /// Network errors count
    pub network_errors: u32,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics  {/// Request latency percentiles
    pub latency_percentiles: LatencyPercentiles,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
    /// Error rates
    pub error_rates: ErrorRates,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Latency percentile measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles  {/// 50th percentile (median) in milliseconds
    pub p50: f64,
    /// 90th percentile in milliseconds
    pub p90: f64,
    /// 95th percentile in milliseconds
    pub p95: f64,
    /// 99th percentile in milliseconds
    pub p99: f64,
    /// 99.9th percentile in milliseconds
    pub p999: f64,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics  {/// Requests per second
    pub requests_per_second: f64,
    /// Operations per second
    pub operations_per_second: f64,
    /// Data processed per second (bytes)
    pub data_processed_per_second: u64,
}

/// Error rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRates  {/// Total error rate (0.0-1.0)
    pub total_error_rate: f64,
    /// 4xx error rate (0.0-1.0)
    pub client_error_rate: f64,
    /// 5xx error rate (0.0-1.0)
    pub server_error_rate: f64,
    /// Timeout error rate (0.0-1.0)
    pub timeout_error_rate: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization  {/// CPU utilization (0.0-1.0)
    pub cpu: f64,
    /// Memory utilization (0.0-1.0)
    pub memory: f64,
    /// Disk utilization (0.0-1.0)
    pub disk: f64,
    /// Network utilization (0.0-1.0)
    pub network: f64,
}

/// Metrics snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot  {/// Snapshot ID
    pub id: String,
    /// System metrics
    pub system_metrics: SystemMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>)
    /// Collection timestamp
    pub timestamp: SystemTime,
    /// Collection duration
    pub collection_duration: Duration,
}

impl Default for SystemMetrics  {fn default() -> Self  {Self {
            cpu_utilization: 0.0,
            memory_usage: 0,
            memory_available: 0,
            disk_utilization: 0.0,
            network_io: NetworkIO::default(),
            process_count: 0,
            load_average: (0.0, 0.0, 0.0)
            timestamp: SystemTime::now(,
        }
    }
}

impl Default for PerformanceMetrics  {fn default() -> Self  {Self {
            latency_percentiles: LatencyPercentiles::default(),
            throughput: ThroughputMetrics::default(),
            error_rates: ErrorRates::default(),
            resource_utilization: ResourceUtilization::default(),
            timestamp: SystemTime::now(,
        }
    }
}

impl Default for LatencyPercentiles  {fn default() -> Self  {Self {
            p50: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
            p999: 0.0,
        }
    }
}

impl Default for ThroughputMetrics  {fn default() -> Self  {Self {
            requests_per_second: 0.0,
            operations_per_second: 0.0,
            data_processed_per_second: 0,
        }
    }
}

impl Default for ErrorRates  {fn default() -> Self  {Self {
            total_error_rate: 0.0,
            client_error_rate: 0.0,
            server_error_rate: 0.0,
            timeout_error_rate: 0.0,
        }
    }
}

impl Default for ResourceUtilization  {fn default() -> Self  {Self {
            cpu: 0.0,
            memory: 0.0,
            disk: 0.0,
            network: 0.0,
        }
    }
}
