// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Metrics collection and aggregation
//!
//! This module provides metrics collection from primals in the ecosystem using
//! capability-based adapters. `ComputeMetrics` is a point-in-time snapshot;
//! `ComputeMetricsCounters` tracks thread-safe totals as collections run.

#![expect(
    async_fn_in_trait,
    reason = "native async MetricsCapabilityAdapter; concrete UniversalMetricsAdapter only"
)]

pub mod capability_adapters;

pub use capability_adapters::{MetricsError, UniversalMetricsAdapter};

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe counters incremented when metrics are collected or observed.
#[derive(Debug, Default)]
pub struct ComputeMetricsCounters {
    /// Number of successful `collect_compute_metrics` calls.
    pub collections_total: AtomicU64,
    /// Rolling count of zero-copy operations reported by sources (if available).
    pub zero_copy_ops_observed: AtomicU64,
    /// Last observed queue depth hint (jobs waiting across primals).
    pub queued_jobs_hint: AtomicU64,
}

impl ComputeMetricsCounters {
    /// Creates a new counter bundle (typically wrapped in [`std::sync::Arc`] for sharing).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Point-in-time compute snapshot (serializable for APIs and telemetry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMetrics {
    /// CPU usage as a percentage (0–100).
    pub cpu_usage_percent: f64,
    /// CPU usage as a fraction (0.0–1.0), kept for older call sites.
    pub cpu_usage: f64,
    /// One-minute load average (platform-dependent; 0.0 if unknown).
    pub load_average: f64,
    /// Memory usage as a fraction of address space (0.0–1.0) when known.
    pub memory_usage: f64,
    /// Bytes of memory currently in use (workload / host reported).
    pub memory_usage_bytes: u64,
    /// Bytes of memory considered available to the workload.
    pub memory_available_bytes: u64,
    /// Active containers / workload units (from discovery or runtime).
    pub active_containers: u32,
    /// Jobs queued for execution.
    pub queued_jobs: u32,
    /// Normalized performance score (0.0–1.0).
    pub performance_score: f64,
    /// Observed zero-copy operations per second (last sample).
    pub zero_copy_operations_per_sec: u64,
    /// Logical name for this metric sample (e.g. host or cluster id).
    pub metric_name: String,
    /// When this snapshot was produced.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ComputeMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 15.0,
            cpu_usage: 0.15,
            load_average: 0.0,
            memory_usage: 0.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 0,
            queued_jobs: 0,
            performance_score: 0.8,
            zero_copy_operations_per_sec: 1000,
            metric_name: "songbird.compute".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Capability adapters expose compute metrics snapshots and async collection.
pub trait MetricsCapabilityAdapter: Send + Sync {
    /// Return the last computed snapshot (cheap; may reflect discovery/counters only).
    fn get_compute_metrics(&self) -> ComputeMetrics;

    /// Collect a fresh snapshot (increments collection counters).
    ///
    /// # Errors
    ///
    /// Returns an error if collection fails (e.g. future HTTP delegation).
    async fn collect_compute_metrics(
        &self,
    ) -> Result<ComputeMetrics, Box<dyn std::error::Error + Send + Sync>>;
}

impl MetricsCapabilityAdapter for UniversalMetricsAdapter {
    fn get_compute_metrics(&self) -> ComputeMetrics {
        self.snapshot_compute_metrics()
    }

    async fn collect_compute_metrics(
        &self,
    ) -> Result<ComputeMetrics, Box<dyn std::error::Error + Send + Sync>> {
        self.metrics_counters.collections_total.fetch_add(1, Ordering::Relaxed);
        Ok(self.snapshot_compute_metrics())
    }
}
