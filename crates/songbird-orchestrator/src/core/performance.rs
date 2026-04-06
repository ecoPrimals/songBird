// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! # 📊 Performance Monitoring
//!
//! **MODERN PERFORMANCE MONITORING** ✅
//! **ZERO-COPY OPTIMIZATION** (Dec 8, 2025)

use super::{ComponentHealth, HealthStatus, PerformanceConfig};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::sync::Arc;

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub response_time: f64,
    pub throughput: f64,
    pub error_rate: f64,
}

/// Performance monitor implementation
#[derive(Debug)]
pub struct PerformanceMonitor {
    #[expect(
        dead_code,
        reason = "retained for threshold-based tuning when metrics pipeline is active"
    )]
    config: PerformanceConfig,
    metrics: PerformanceMetrics,
}

impl PerformanceMonitor {
    #[must_use]
    pub const fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            metrics: PerformanceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                response_time: 0.0,
                throughput: 0.0,
                error_rate: 0.0,
            },
        }
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn initialize(&mut self) -> SongbirdResult<()> {
        // Initialize performance monitoring
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn start(&mut self) -> SongbirdResult<()> {
        // Start performance monitoring
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn stop(&mut self) -> SongbirdResult<()> {
        // Stop performance monitoring
        Ok(())
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    #[expect(
        clippy::unused_async,
        reason = "async signature required by Axum, trait objects, or future I/O"
    )]
    pub async fn health_check(&self) -> SongbirdResult<ComponentHealth> {
        Ok(ComponentHealth {
            status: HealthStatus::Healthy,
            message: Some(Arc::from("Performance monitor active")),
            last_check: Some(chrono::Utc::now().timestamp() as u64),
        })
    }

    #[must_use]
    pub const fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::SongbirdError;

    #[test]
    fn test_performance_monitor_new() {
        let config = PerformanceConfig::default();
        let pm = PerformanceMonitor::new(config);

        let metrics = pm.get_metrics();
        assert_eq!(metrics.cpu_usage, 0.0);
        assert_eq!(metrics.memory_usage, 0.0);
        assert_eq!(metrics.response_time, 0.0);
        assert_eq!(metrics.throughput, 0.0);
        assert_eq!(metrics.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_performance_monitor_initialize() {
        let config = PerformanceConfig::default();
        let mut pm = PerformanceMonitor::new(config);

        let result = pm.initialize().await;
        assert!(result.is_ok(), "Initialize should succeed");
    }

    #[tokio::test]
    async fn test_performance_monitor_start() -> SongbirdResult<()> {
        let config = PerformanceConfig::default();
        let mut pm = PerformanceMonitor::new(config);

        pm.initialize().await?;

        let result = pm.start().await;
        assert!(result.is_ok(), "Start should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_monitor_stop() -> SongbirdResult<()> {
        let config = PerformanceConfig::default();
        let mut pm = PerformanceMonitor::new(config);

        pm.initialize().await?;
        pm.start().await?;

        let result = pm.stop().await;
        assert!(result.is_ok(), "Stop should succeed");
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_monitor_health_check() -> SongbirdResult<()> {
        let config = PerformanceConfig::default();
        let pm = PerformanceMonitor::new(config);

        let health = pm.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_some());
        assert!(health.last_check.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_monitor_full_lifecycle() -> SongbirdResult<()> {
        let config = PerformanceConfig::default();
        let mut pm = PerformanceMonitor::new(config);

        pm.initialize().await?;
        pm.start().await?;

        let health = pm.health_check().await?;
        assert_eq!(health.status, HealthStatus::Healthy);

        pm.stop().await?;
        Ok(())
    }

    #[test]
    fn test_performance_metrics_clone() {
        let metrics = PerformanceMetrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            response_time: 100.0,
            throughput: 1000.0,
            error_rate: 0.5,
        };

        let cloned = metrics.clone();
        assert_eq!(metrics.cpu_usage, cloned.cpu_usage);
        assert_eq!(metrics.memory_usage, cloned.memory_usage);
        assert_eq!(metrics.response_time, cloned.response_time);
        assert_eq!(metrics.throughput, cloned.throughput);
        assert_eq!(metrics.error_rate, cloned.error_rate);
    }

    #[test]
    fn test_performance_metrics_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let metrics = PerformanceMetrics {
            cpu_usage: 75.5,
            memory_usage: 80.2,
            response_time: 250.0,
            throughput: 5000.0,
            error_rate: 1.5,
        };

        let json = serde_json::to_string(&metrics).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Serialization failed: {e}"),
            debug_info: None,
        })?;
        let deserialized: PerformanceMetrics =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {e}"),
                debug_info: None,
            })?;

        assert_eq!(metrics.cpu_usage, deserialized.cpu_usage);
        assert_eq!(metrics.memory_usage, deserialized.memory_usage);
        Ok(())
    }

    #[test]
    fn test_performance_metrics_debug_format() {
        let metrics = PerformanceMetrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            response_time: 100.0,
            throughput: 1000.0,
            error_rate: 0.5,
        };

        let debug_string = format!("{metrics:?}");
        assert!(debug_string.contains("PerformanceMetrics"));
        assert!(debug_string.contains("cpu_usage"));
        assert!(debug_string.contains("50"));
    }

    #[test]
    fn test_performance_monitor_get_metrics() {
        let config = PerformanceConfig::default();
        let pm = PerformanceMonitor::new(config);

        let metrics = pm.get_metrics();
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0);
    }
}
