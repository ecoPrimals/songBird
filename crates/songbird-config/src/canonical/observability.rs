// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical observability configuration
//!
//! Unified observability configuration for logging, tracing, metrics, and dashboards.
//! Consolidated from `unified::observability` in Config Phase 3B (November 2025).

use serde::{Deserialize, Serialize};

/// Unified observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedObservabilityConfig {
    /// Dashboard configuration
    pub dashboard: DashboardConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Enable dashboard
    pub enabled: bool,

    /// Dashboard host
    pub host: String,

    /// Dashboard port
    pub port: u16,

    /// Enable real-time updates
    pub realtime_updates: bool,

    /// Update interval in seconds
    pub update_interval_secs: u64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: songbird_types::constants::PRODUCTION_BIND_ADDRESS.to_string(),
            port: songbird_types::defaults::ports::DEFAULT_DASHBOARD_PORT,
            realtime_updates: true,
            update_interval_secs: 5,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable logging
    pub enabled: bool,

    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty, compact)
    pub format: String,

    /// Log rotation configuration
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: "pretty".to_string(),
            rotation: LogRotationConfig::default(),
        }
    }
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Enable log rotation
    pub enabled: bool,

    /// Maximum log file size in MB
    pub max_size_mb: u64,

    /// Maximum number of log files to keep
    pub max_files: u32,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 100,
            max_files: 10,
        }
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,

    /// Tracing endpoint (e.g., Jaeger, Zipkin)
    pub endpoint: Option<String>,

    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: None,
            sample_rate: 0.1,
        }
    }
}

#[cfg(test)]
#[path = "observability_tests.rs"]
mod tests;
