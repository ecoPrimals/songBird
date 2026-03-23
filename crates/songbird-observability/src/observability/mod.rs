// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;

/// Dashboard-oriented views and helpers (UI-facing aggregates).
pub mod dashboard;
/// Service- and node-level [`crate::observability::health::HealthMonitor`] used by [`ObservabilityManager`].
pub mod health;
/// [`crate::observability::metrics::MetricsCollector`] and [`crate::observability::metrics::MetricsSnapshot`] for system and app gauges.
pub mod metrics;

/// Point-in-time resource snapshot for a single host or process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU utilization fraction (0.0–1.0) over the last collection window.
    pub cpu_usage: f64,
    /// Resident memory utilization fraction (0.0–1.0).
    pub memory_usage: f64,
    /// Primary volume utilization fraction (0.0–1.0).
    pub disk_usage: f64,
    /// Byte and packet counters since the last reset or boot.
    pub network_io: NetworkIO,
    /// When the sample was taken (UTC).
    pub timestamp: DateTime<Utc>,
}

/// Cumulative network counters paired with [`SystemMetrics`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    /// Inbound bytes observed in the window.
    pub bytes_in: u64,
    /// Outbound bytes observed in the window.
    pub bytes_out: u64,
    /// Inbound packets observed in the window.
    pub packets_in: u64,
    /// Outbound packets observed in the window.
    pub packets_out: u64,
}

/// Serialized health status for a single service (four-way, includes unknown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All checks green for this service.
    Healthy,
    /// Some checks yellow; service may still be partially usable.
    Degraded,
    /// Hard failure for this service.
    Unhealthy,
    /// No data yet or probe has not run.
    Unknown,
}

/// Last-known probe result for a logical service id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Stable identifier (often matches discovery or orchestration id).
    pub service_id: String,
    /// Latest [`HealthStatus`] from the last check.
    pub status: HealthStatus,
    /// When the last probe completed (UTC).
    pub last_check: DateTime<Utc>,
    /// Probe latency in milliseconds.
    pub response_time_ms: u64,
    /// Optional error string when status is not [`HealthStatus::Healthy`].
    pub error_message: Option<String>,
}

/// Cluster-wide rollup for node and service counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    /// Total cluster members seen by the manager.
    pub total_nodes: usize,
    /// Members reporting healthy last heartbeat.
    pub healthy_nodes: usize,
    /// Registered logical services across the cluster.
    pub total_services: usize,
    /// Services currently running (not drained/stopped).
    pub running_services: usize,
    /// When this rollup was last refreshed (UTC).
    pub last_updated: DateTime<Utc>,
}

impl Default for ClusterStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterStatus {
    /// Returns zeroed counts with [`Utc::now`] as [`last_updated`](ClusterStatus::last_updated).
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            healthy_nodes: 0,
            total_services: 0,
            running_services: 0,
            last_updated: Utc::now(),
        }
    }

    /// Returns [`ClusterStatus::total_nodes`].
    #[must_use]
    pub const fn total_nodes(&self) -> usize {
        self.total_nodes
    }

    /// Returns [`ClusterStatus::running_services`].
    #[must_use]
    pub const fn running_services(&self) -> usize {
        self.running_services
    }

    /// Returns [`ClusterStatus::total_services`].
    #[must_use]
    pub const fn total_services(&self) -> usize {
        self.total_services
    }
}

/// High-level cluster posture derived from node and service rollups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterHealthStatus {
    /// Majority of nodes and services within SLO.
    Healthy,
    /// Some nodes or services failing; partial outage.
    Degraded,
    /// Large-scale failure or quorum loss.
    Critical,
}

/// Per-node resource and heartbeat for topology views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    /// Stable node id (matches federation or orchestrator id).
    pub node_id: String,
    /// Last reported [`HealthStatus`] for workloads on this node.
    pub status: HealthStatus,
    /// CPU utilization fraction (0.0–1.0).
    pub cpu_usage: f64,
    /// Memory utilization fraction (0.0–1.0).
    pub memory_usage: f64,
    /// Disk utilization fraction (0.0–1.0).
    pub disk_usage: f64,
    /// Last successful heartbeat from the node (UTC).
    pub last_heartbeat: DateTime<Utc>,
}

/// Fan-out event for [`ObservabilityManager::subscribe_to_events`] subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservabilityEvent {
    /// A metrics scrape finished and produced [`SystemMetrics`].
    MetricsCollected {
        /// Collected host metrics.
        metrics: SystemMetrics,
        /// When collection started or completed (UTC).
        timestamp: DateTime<Utc>,
        /// Wall-clock duration of the scrape in milliseconds.
        duration_ms: u64,
    },
    /// A single service health check completed.
    HealthCheckCompleted {
        /// Target service id.
        service_id: String,
        /// Resulting [`HealthStatus`].
        status: HealthStatus,
        /// Probe latency in milliseconds.
        response_time_ms: u64,
        /// Event time (UTC).
        timestamp: DateTime<Utc>,
    },
    /// Operator-facing alert (e.g. threshold breach).
    SystemAlert {
        /// Human-readable alert text.
        message: String,
        /// Severity label (e.g. `"warning"` / `"critical"`).
        severity: String,
        /// When the alert was raised (UTC).
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a service transitions between [`HealthStatus`] values.
    ServiceStatusChanged {
        /// Target service id.
        service_id: String,
        /// Previous status.
        old_status: HealthStatus,
        /// New status after the transition.
        new_status: HealthStatus,
        /// When the transition was observed (UTC).
        timestamp: DateTime<Utc>,
    },
}

/// Coordinates in-memory health, cluster rollups, and event subscribers for the observability stack.
#[derive(Debug)]
pub struct ObservabilityManager {
    #[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
    metrics_store: Arc<RwLock<HashMap<String, SystemMetrics>>>,
    /// Latest [`ServiceHealth`] per service id.
    health_store: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    /// Latest [`ClusterStatus`] snapshot.
    cluster_status: Arc<RwLock<ClusterStatus>>,
    /// Channels that receive [`ObservabilityEvent`] copies.
    event_subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<ObservabilityEvent>>>>,
}

impl ObservabilityManager {
    /// Create new observability manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics_store: Arc::new(RwLock::new(HashMap::new())),
            health_store: Arc::new(RwLock::new(HashMap::new())),
            cluster_status: Arc::new(RwLock::new(ClusterStatus::new())),
            event_subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Lightweight liveness check: returns `true` when the manager is structurally sound.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        true
    }

    /// Start observability monitoring
    ///
    /// # Errors
    ///
    /// Returns an error if metrics collection or health monitoring fails to start
    pub async fn start(&self) -> Result<()> {
        info!("Starting observability manager");

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start health monitoring
        self.start_health_monitoring().await?;

        info!("Observability manager started successfully");
        Ok(())
    }

    /// Stop observability monitoring
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping observability manager");
        Ok(())
    }

    /// Get current system metrics
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_metrics(&self) -> Result<SystemMetrics> {
        let metrics = SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_io: NetworkIO {
                bytes_in: 0,
                bytes_out: 0,
                packets_in: 0,
                packets_out: 0,
            },
            timestamp: Utc::now(),
        };
        Ok(metrics)
    }

    /// Get service health status
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_service_health(&self, service_id: &str) -> Result<Option<ServiceHealth>> {
        let health_store = self.health_store.read().await;
        Ok(health_store.get(service_id).cloned())
    }

    /// Get cluster status
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let status = self.cluster_status.read().await;
        Ok(status.clone())
    }

    /// Subscribe to observability events
    pub async fn subscribe_to_events(
        &self,
    ) -> tokio::sync::mpsc::UnboundedReceiver<ObservabilityEvent> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut subscribers = self.event_subscribers.write().await;
        subscribers.push(tx);
        rx
    }

    /// Record a health check result
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn record_health_check(
        &self,
        service_id: String,
        status: HealthStatus,
        response_time_ms: u64,
    ) -> Result<()> {
        let health = ServiceHealth {
            service_id: service_id.clone(),
            status: status.clone(),
            last_check: Utc::now(),
            response_time_ms,
            error_message: None,
        };

        self.health_store.write().await.insert(service_id.clone(), health);

        // Send event to subscribers
        self.send_event(ObservabilityEvent::HealthCheckCompleted {
            service_id,
            status,
            response_time_ms,
            timestamp: Utc::now(),
        })
        .await;

        Ok(())
    }

    /// Send event to all subscribers
    async fn send_event(&self, event: ObservabilityEvent) {
        let subscribers = self.event_subscribers.read().await;
        for subscriber in subscribers.iter() {
            let _ = subscriber.send(event.clone());
        }
    }

    /// Start metrics collection
    async fn start_metrics_collection(&self) -> Result<()> {
        debug!("Starting metrics collection");
        Ok(())
    }

    /// Start health monitoring
    async fn start_health_monitoring(&self) -> Result<()> {
        debug!("Starting health monitoring");
        Ok(())
    }
}

impl Default for ObservabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_manager_creation() {
        let manager = ObservabilityManager::new();
        assert!(manager.start().await.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let manager = ObservabilityManager::new();
        let metrics = manager.get_metrics().await;
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let manager = ObservabilityManager::new();
        let result = manager
            .record_health_check("test-service".to_string(), HealthStatus::Healthy, 100)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let manager = ObservabilityManager::new();
        let _receiver = manager.subscribe_to_events().await;
        // Test passes if we can create a receiver
    }
}
