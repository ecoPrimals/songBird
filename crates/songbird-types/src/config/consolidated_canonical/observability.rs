// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Observability Configuration Module
//!
//! **CANONICAL OBSERVABILITY CONFIGURATION** ✅
//!
//! This module provides monitoring and observability configuration structures for the Songbird ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// OBSERVABILITY CONFIGURATION
// ============================================================================

/// **CANONICAL**: Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalObservabilityConfig {
    /// Enable observability features
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval: u64,
    /// Health check configuration
    pub health_checks: CanonicalHealthCheckConfig,
    /// Metrics configuration
    pub metrics: CanonicalMetricsConfig,
    /// Tracing configuration
    pub tracing: CanonicalTracingConfig,
}

/// **CANONICAL**: Health Check Configuration - replaces 12+ duplicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Health check endpoint path
    pub endpoint: String,
    /// Expected status codes for healthy response
    pub expected_status_codes: Vec<u16>,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
    /// Custom health check headers
    pub headers: HashMap<String, String>,
    /// Enable detailed health reporting
    pub detailed_reporting: bool,
}

/// **CANONICAL**: Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Metrics export endpoints
    pub export_endpoints: Vec<String>,
    /// Custom metrics labels
    pub labels: HashMap<String, String>,
}

/// **CANONICAL**: Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalTracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,
    /// Tracing level (trace, debug, info, warn, error,
    pub level: String,
    /// Sampling rate (0.0 to 1.0,
    pub sampling_rate: f64,
    /// Trace export endpoints
    pub export_endpoints: Vec<String>,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for CanonicalObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: 60,
            health_checks: CanonicalHealthCheckConfig::default(),
            metrics: CanonicalMetricsConfig::default(),
            tracing: CanonicalTracingConfig::default(),
        }
    }
}

impl Default for CanonicalHealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            endpoint: "/health".to_string(),
            expected_status_codes: vec![200, 204],
            failure_threshold: 3,
            success_threshold: 2,
            headers: HashMap::new(),
            detailed_reporting: false,
        }
    }
}

impl Default for CanonicalMetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: Duration::from_secs(60),
            retention_period: Duration::from_secs(86400), // 24 hours
            export_endpoints: vec![],
            labels: HashMap::new(),
        }
    }
}

impl Default for CanonicalTracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            sampling_rate: 1.0,
            export_endpoints: vec![],
        }
    }
}
