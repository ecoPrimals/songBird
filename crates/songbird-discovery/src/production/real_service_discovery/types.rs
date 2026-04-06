// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration and registry types for production service discovery.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use crate::discovery::core::ServiceInstance;

/// Configuration for production service discovery
#[derive(Debug, Clone)]
pub struct ProductionDiscoveryConfig {
    /// Health Check Interval field
    pub health_check_interval: Duration,
    /// Service Timeout field
    pub service_timeout: Duration,
    /// Max Retry Attempts field
    pub max_retry_attempts: u32,
    /// Enable Health Checks field
    pub enable_health_checks: bool,
}

impl Default for ProductionDiscoveryConfig {
    fn default() -> Self {
        // ✅ DEEP DEBT EVOLUTION (Feb 3, 2026): Use TimeoutConfig
        // Replaces hardcoded Duration::from_secs with configurable timeouts
        let timeout_config = songbird_config::timeouts::TimeoutConfig::from_env();

        Self {
            health_check_interval: timeout_config.health_check,
            service_timeout: timeout_config.discovery,
            max_retry_attempts: 3,
            enable_health_checks: true,
        }
    }
}

/// Registered service with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Instance field
    pub instance: ServiceInstance,
    /// Registered At field
    pub registered_at: SystemTime,
    /// Last Heartbeat field
    pub last_heartbeat: Option<SystemTime>,
    /// Health Status field
    pub health_status: ServiceHealthStatus,
    /// Retry Count field
    pub retry_count: u32,
}

/// Health record for services
#[derive(Debug, Clone)]
pub struct HealthRecord {
    /// Service Id field
    pub service_id: String,
    /// Current status of the operation or entity
    pub status: ServiceHealthStatus,
    /// Last Check field
    pub last_check: SystemTime,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Error Message field
    pub error_message: Option<String>,
}

/// Service health status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
