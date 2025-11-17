//! Observability configuration structures
//!
//! ⚠️ **DEPRECATED**: This module has been consolidated into `canonical::observability`.
//!
//! **Migration Path**:
//! ```rust,ignore
//! // OLD (deprecated):
//! use songbird_config::unified::observability::{UnifiedObservabilityConfig, DashboardConfig};
//!
//! // NEW (canonical):
//! use songbird_config::canonical::observability::{UnifiedObservabilityConfig, DashboardConfig};
//! ```
//!
//! **All 5 observability types** are now available in `canonical::observability`:
//! - `UnifiedObservabilityConfig`
//! - `DashboardConfig`
//! - `LoggingConfig`
//! - `LogRotationConfig`
//! - `TracingConfig`
//!
//! **Removal Timeline**: This module will be removed in Q2 2026.
//!
//! **Phase**: Config Consolidation Phase 3B (November 2025)

#![deprecated(
    since = "0.1.0",
    note = "Use `canonical::observability` module instead. All types consolidated. Migration: `unified::observability::UnifiedObservabilityConfig` → `canonical::observability::UnifiedObservabilityConfig`"
)]

use serde::{Deserialize, Serialize};
use std::env;

/// Unified configuration for observability features
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedObservabilityConfig {
    /// Dashboard configuration
    pub dashboard: DashboardConfig,
    /// Health check configuration
    pub health_checks: HealthCheckConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub port: u16,
    pub refresh_interval_ms: u64,
    pub max_alerts: usize,
    pub enable_real_time: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_DASHBOARD_ENABLED").is_ok(),
            port: env::var("SONGBIRD_DASHBOARD_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            refresh_interval_ms: 1000,
            max_alerts: 100,
            enable_real_time: true,
        }
    }
}

/// Health check configuration - Re-export from robustness module
/// **UNIFICATION COMPLETE**: Uses the comprehensive `HealthCheckConfig` from robustness.rs
pub use super::robustness::HealthCheckConfig;

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: env::var("SONGBIRD_LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            format: env::var("SONGBIRD_LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            output: env::var("SONGBIRD_LOG_OUTPUT").unwrap_or_else(|_| "stdout".to_string()),
            rotation: LogRotationConfig::default(),
        }
    }
}

/// Log rotation configuration
// ✅ CONSOLIDATED: Re-export from canonical location
pub use crate::canonical::observability::LogRotationConfig;

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_TRACING_ENABLED").is_ok(),
            endpoint: env::var("SONGBIRD_TRACING_ENDPOINT").ok(),
            sample_rate: 0.1,
        }
    }
}
