//! Canonical Orchestrator Types
//!
//! This module provides unified orchestrator type definitions that replace
//! fragmented types across different modules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical Orchestrator Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    /// Number of active services
    pub active_services: usize,
    /// Total number of nodes in the cluster
    pub total_nodes: usize,
    /// Number of healthy nodes
    pub healthy_nodes: usize,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
    /// Orchestrator uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Overall system status
    pub system_status: SystemStatus,
}

/// Canonical Orchestrator Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorMetrics {
    /// Number of active services
    pub active_services: usize,
    /// Number of healthy services
    pub healthy_services: usize,
    /// Total number of services ever registered
    pub total_services: usize,
    /// Orchestrator uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in megabytes
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Network I/O statistics
    pub network_bytes_in: u64,
    pub network_bytes_out: u64,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
    /// Number of failed health checks
    pub failed_health_checks: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
}

/// System Status Enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SystemStatus {
    /// System is healthy and operational
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is in critical state
    Critical,
    /// System is down
    Down,
    /// System status is unknown
    #[default]
    Unknown,
}

impl Default for OrchestratorStatus {
    fn default() -> Self {
        Self {
            active_services: 0,
            total_nodes: 0,
            healthy_nodes: 0,
            last_health_check: Utc::now(),
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            system_status: SystemStatus::Unknown,
        }
    }
}

impl Default for OrchestratorMetrics {
    fn default() -> Self {
        Self {
            active_services: 0,
            healthy_services: 0,
            total_services: 0,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            network_bytes_in: 0,
            network_bytes_out: 0,
            last_health_check: Utc::now(),
            failed_health_checks: 0,
            avg_response_time_ms: 0.0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

impl OrchestratorStatus {
    /// Create a new orchestrator status
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the orchestrator is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.system_status, SystemStatus::Healthy)
    }

    /// Update service counts
    pub fn update_services(&mut self, active: usize, healthy: usize) {
        self.active_services = active;
        self.healthy_nodes = healthy;
        self.last_health_check = Utc::now();
    }

    /// Update system resources
    pub fn update_resources(&mut self, memory_mb: f64, cpu_percent: f64) {
        self.memory_usage_mb = memory_mb;
        self.cpu_usage_percent = cpu_percent;
    }

    /// Update uptime
    pub fn update_uptime(&mut self, seconds: u64) {
        self.uptime_seconds = seconds;
    }

    /// Set system status
    pub fn set_status(&mut self, status: SystemStatus) {
        self.system_status = status;
    }
}

impl OrchestratorMetrics {
    /// Create new orchestrator metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update service counts
    pub fn update_service_counts(&mut self, active: usize, healthy: usize, total: usize) {
        self.active_services = active;
        self.healthy_services = healthy;
        self.total_services = total;
        self.last_health_check = Utc::now();
    }

    /// Record a health check
    pub fn record_health_check(&mut self, success: bool) {
        self.last_health_check = Utc::now();
        if !success {
            self.failed_health_checks += 1;
        }
    }

    /// Update system resource metrics
    pub fn update_system_metrics(&mut self, memory_mb: f64, cpu_percent: f64) {
        self.memory_usage_mb = memory_mb;
        self.cpu_usage_percent = cpu_percent;
    }

    /// Update network metrics
    pub fn update_network_metrics(&mut self, bytes_in: u64, bytes_out: u64) {
        self.network_bytes_in = bytes_in;
        self.network_bytes_out = bytes_out;
    }

    /// Update request metrics
    pub fn update_request_metrics(&mut self, total: u64, successful: u64, failed: u64) {
        self.total_requests = total;
        self.successful_requests = successful;
        self.failed_requests = failed;
    }

    /// Update average response time
    pub fn update_response_time(&mut self, response_time_ms: f64) {
        // Simple moving average (could be improved with more sophisticated algorithm)
        self.avg_response_time_ms = (self.avg_response_time_ms + response_time_ms) / 2.0;
    }

    /// Update uptime
    pub fn update_uptime(&mut self, seconds: u64) {
        self.uptime_seconds = seconds;
    }

    /// Get success rate
    pub fn get_success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }
}
