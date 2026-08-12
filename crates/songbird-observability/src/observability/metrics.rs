// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use std::sync::RwLock;

use super::SystemMetrics;
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Metrics collector for system and application metrics
#[derive(Debug)]
pub struct MetricsCollector {
    current_metrics: Arc<RwLock<Option<MetricsSnapshot>>>,
    collection_count: Arc<std::sync::atomic::AtomicU64>,
}

impl MetricsCollector {
    /// Create new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(None)),
            collection_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Collect all metrics
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn collect_all_metrics(&self) -> Result<MetricsSnapshot> {
        let metrics = MetricsSnapshot {
            system: SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_io: super::NetworkIO {
                    bytes_in: 0,
                    bytes_out: 0,
                    packets_in: 0,
                    packets_out: 0,
                },
                timestamp: Utc::now(),
            },
            songbird: ApplicationMetrics {
                active_services: 0,
                request_rate: 0.0,
                error_rate: 0.0,
                avg_response_time_ms: 0.0,
            },
            collection_duration_ms: 1,
            timestamp: Utc::now(),
        };

        // Update stored metrics
        *self.current_metrics.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
            Some(metrics.clone());

        // Increment collection count
        self.collection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(metrics)
    }

    /// Get current metrics snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection fails when no snapshot exists
    pub async fn get_current_snapshot(&self) -> Result<MetricsSnapshot> {
        let metrics_copy = {
            let current =
                self.current_metrics.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            current.as_ref().cloned()
        };

        match metrics_copy {
            Some(metrics) => Ok(metrics),
            None => self.collect_all_metrics().await,
        }
    }

    /// Get current metrics (alias for compatibility)
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection fails when no snapshot exists
    pub async fn get_current_metrics(&self) -> Result<MetricsSnapshot> {
        self.get_current_snapshot().await
    }

    /// Export metrics in Prometheus format
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection fails
    pub async fn export_prometheus(&self) -> Result<String> {
        let metrics = self.get_current_snapshot().await?;

        let mut output = String::new();

        // System metrics
        output.push_str("# HELP songbird_cpu_usage_percent CPU usage percentage\n");
        output.push_str("# TYPE songbird_cpu_usage_percent gauge\n");
        let _ = writeln!(output, "songbird_cpu_usage_percent {}", metrics.system.cpu_usage);

        output.push_str("# HELP songbird_memory_usage_ratio Memory usage ratio\n");
        output.push_str("# TYPE songbird_memory_usage_ratio gauge\n");
        let _ = writeln!(output, "songbird_memory_usage_ratio {}", metrics.system.memory_usage);

        // Application metrics
        output.push_str("# HELP songbird_active_services Number of active services\n");
        output.push_str("# TYPE songbird_active_services gauge\n");
        let _ = writeln!(output, "songbird_active_services {}", metrics.songbird.active_services);

        Ok(output)
    }

    /// Get collection count
    #[must_use]
    pub fn get_collection_count(&self) -> u64 {
        self.collection_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get last collection time
    #[must_use]
    #[allow(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "instance method for future stateful timing; Option for API stability"
    )]
    pub fn last_collection_time(&self) -> Option<DateTime<Utc>> {
        // In a real implementation, this would track the actual last collection time
        Some(Utc::now())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Single scrape combining host [`SystemMetrics`] and [`ApplicationMetrics`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Host-level gauges from the collector.
    pub system: SystemMetrics,
    /// Application-level counters and latency estimates.
    pub songbird: ApplicationMetrics,
    /// How long the scrape took in milliseconds.
    pub collection_duration_ms: u64,
    /// Wall-clock end time of the snapshot (UTC).
    pub timestamp: DateTime<Utc>,
}

/// Songbird-specific traffic and error signals (complements [`SystemMetrics`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    /// Active logical services reporting in.
    pub active_services: u32,
    /// Requests per second observed in the window.
    pub request_rate: f64,
    /// Error responses per request (0.0–1.0) in the window.
    pub error_rate: f64,
    /// Mean response time in milliseconds.
    pub avg_response_time_ms: f64,
}

#[cfg(test)]
#[path = "metrics_tests.rs"]
mod tests;
