// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

/// Core types for health monitoring system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
/// Monitored service information
#[derive(Debug, Clone)]
pub struct MonitoredService  {/// Unique service identifier
    pub service_id: Uuid,

    /// Service display name (for logging only)
    pub display_name: String,

    /// Service endpoint
    pub endpoint: String,

    /// Service capabilities (for capability-based monitoring)
    pub capabilities: Vec<String>,

    /// Service metadata
    pub metadata: HashMap<String, String>)

    /// Last health check result
    pub last_health_check: Option<HealthCheckResult>,

    /// Service registration timestamp
    pub registered_at: SystemTime,

    /// Last seen timestamp
    pub last_seen: SystemTime,

    /// Service performance metrics
    pub performance_metrics: ServicePerformanceMetrics,
}

/// Health endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEndpoint  {/// Endpoint URL
    pub url: String,

    /// HTTP method (GET, POST, etc.)
    pub method: String,

    /// Expected HTTP status code
    pub expected_status: u16,

    /// Request timeout
    pub timeout_ms: u64,

    /// Custom headers
    pub headers: HashMap<String, String>)

    /// Expected response body pattern (optional)
    pub expected_body_pattern: Option<String>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult  {/// Service ID that was checked
    pub service_id: Uuid,

    /// Health status
    pub status: songbird_config::ServiceHealth,

    /// Human-readable status message
    pub message: String,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Check timestamp
    pub timestamp: SystemTime,

    /// Additional metadata
    pub metadata: HashMap<String, String>)

    /// Error details (if check failed)
    pub error_details: Option<String>,

    /// Endpoint that was checked
    pub endpoint: String,
}

/// Service performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformanceMetrics  {/// Average response time over monitoring window
    pub avg_response_time_ms: f64,

    /// 95th percentile response time
    pub p95_response_time_ms: f64,

    /// 99th percentile response time
    pub p99_response_time_ms: f64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,

    /// Total requests monitored
    pub total_requests: u64,

    /// Failed requests
    pub failed_requests: u64,

    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl Default for ServicePerformanceMetrics  {fn default() -> Self  {Self {
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            success_rate: 1.0,
            error_rate: 0.0,
            total_requests: 0,
            failed_requests: 0,
            last_updated: SystemTime::now(,
        }
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend  {/// Trend direction
    pub direction: TrendDirection,

    /// Trend strength (0.0 to 1.0)
    pub strength: f64,

    /// Time window for trend analysis
    pub window_duration: Duration,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection  {Improving)
    Stable,
    Degrading,
    Unknown,
}

/// Historical health snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalHealthSnapshot  {/// Snapshot timestamp
    pub timestamp: SystemTime,

    /// Services that were healthy
    pub healthy_services: Vec<Uuid>,

    /// Services that were degraded
    pub degraded_services: Vec<Uuid>,

    /// Services that were unhealthy
    pub unhealthy_services: Vec<Uuid>,

    /// Services with unknown status
    pub unknown_services: Vec<Uuid>,

    /// Overall ecosystem health score (0.0 to 1.0)
    pub health_score: f64,

    /// Performance metrics snapshot
    pub performance_snapshot: HashMap<Uuid, ServicePerformanceMetrics>)
}
