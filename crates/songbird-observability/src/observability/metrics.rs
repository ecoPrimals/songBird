use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

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
        let mut current = self.current_metrics.write().await;
        *current = Some(metrics.clone());

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
        let current = self.current_metrics.read().await;
        match current.as_ref() {
            Some(metrics) => Ok(metrics.clone()),
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
        output.push_str(&format!("songbird_cpu_usage_percent {}\n", metrics.system.cpu_usage));

        output.push_str("# HELP songbird_memory_usage_ratio Memory usage ratio\n");
        output.push_str("# TYPE songbird_memory_usage_ratio gauge\n");
        output.push_str(&format!("songbird_memory_usage_ratio {}\n", metrics.system.memory_usage));

        // Application metrics
        output.push_str("# HELP songbird_active_services Number of active services\n");
        output.push_str("# TYPE songbird_active_services gauge\n");
        output
            .push_str(&format!("songbird_active_services {}\n", metrics.songbird.active_services));

        Ok(output)
    }

    /// Get collection count
    #[must_use]
    pub fn get_collection_count(&self) -> u64 {
        self.collection_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get last collection time
    #[must_use]
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

/// Complete metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub system: SystemMetrics,
    pub songbird: ApplicationMetrics,
    pub collection_duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Application-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub active_services: u32,
    pub request_rate: f64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.get_collection_count(), 0);
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        let metrics = collector.collect_all_metrics().await;
        assert!(metrics.is_ok());
        assert_eq!(collector.get_collection_count(), 1);
    }

    #[tokio::test]
    async fn test_prometheus_export() -> Result<()> {
        let collector = MetricsCollector::new();
        collector.collect_all_metrics().await?;

        let prometheus_output = collector.export_prometheus().await;
        assert!(prometheus_output.is_ok());

        let output = prometheus_output?;
        assert!(output.contains("songbird_cpu_usage_percent"));
        assert!(output.contains("songbird_memory_usage_ratio"));
        assert!(output.contains("songbird_active_services"));
        Ok(())
    }
}
