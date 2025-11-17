use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
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
        let metrics_copy = current.as_ref().cloned();
        drop(current); // Explicitly drop the read lock before potentially acquiring write lock

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
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
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
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.get_collection_count(), 0);
    }

    #[tokio::test]
    async fn test_metrics_collector_default() {
        let collector = MetricsCollector::default();
        assert_eq!(collector.get_collection_count(), 0);
    }

    #[tokio::test]
    async fn test_collect_all_metrics() -> Result<()> {
        let collector = MetricsCollector::new();
        let result = collector.collect_all_metrics().await;

        assert!(result.is_ok());
        let metrics =
            result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.songbird.active_services, 0);
        assert_eq!(collector.get_collection_count(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_current_snapshot_after_collection() -> Result<()> {
        let collector = MetricsCollector::new();

        // First collection
        let _ = collector.collect_all_metrics().await;

        // Get snapshot
        let result = collector.get_current_snapshot().await;
        assert!(result.is_ok());
        let snapshot =
            result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert!((snapshot.system.cpu_usage - 0.0).abs() < f64::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_current_snapshot_without_prior_collection() {
        let collector = MetricsCollector::new();

        // Get snapshot without prior collection - should auto-collect
        let result = collector.get_current_snapshot().await;
        assert!(result.is_ok());
        assert_eq!(collector.get_collection_count(), 1);
    }

    #[tokio::test]
    async fn test_get_current_metrics_alias() -> Result<()> {
        let collector = MetricsCollector::new();

        let result = collector.get_current_metrics().await;
        assert!(result.is_ok());
        let metrics =
            result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn test_collection_count_increments() {
        let collector = MetricsCollector::new();

        assert_eq!(collector.get_collection_count(), 0);
        let _ = collector.collect_all_metrics().await;
        assert_eq!(collector.get_collection_count(), 1);
        let _ = collector.collect_all_metrics().await;
        assert_eq!(collector.get_collection_count(), 2);
        let _ = collector.collect_all_metrics().await;
        assert_eq!(collector.get_collection_count(), 3);
    }

    #[tokio::test]
    async fn test_export_prometheus_format() -> Result<()> {
        let collector = MetricsCollector::new();
        let _ = collector.collect_all_metrics().await;

        let result = collector.export_prometheus().await;
        assert!(result.is_ok());

        let output =
            result.map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        assert!(output.contains("songbird_cpu_usage_percent"));
        assert!(output.contains("songbird_memory_usage_ratio"));
        assert!(output.contains("songbird_active_services"));
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        Ok(())
    }

    #[tokio::test]
    async fn test_export_prometheus_without_prior_collection() {
        let collector = MetricsCollector::new();

        // Export should auto-collect if needed
        let result = collector.export_prometheus().await;
        assert!(result.is_ok());
        assert_eq!(collector.get_collection_count(), 1);
    }

    #[tokio::test]
    async fn test_last_collection_time() {
        let collector = MetricsCollector::new();
        let time = collector.last_collection_time();
        assert!(time.is_some());
    }

    #[tokio::test]
    async fn test_metrics_snapshot_contains_timestamp() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        // Timestamp should be recent (within last second)
        let now = Utc::now();
        let diff = now.signed_duration_since(metrics.timestamp);
        assert!(diff.num_seconds() < 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_snapshot_system_metrics() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        assert!((metrics.system.cpu_usage - 0.0).abs() < f64::EPSILON);
        assert!((metrics.system.memory_usage - 0.0).abs() < f64::EPSILON);
        assert!((metrics.system.disk_usage - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.system.network_io.bytes_in, 0);
        assert_eq!(metrics.system.network_io.bytes_out, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_snapshot_application_metrics() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        assert_eq!(metrics.songbird.active_services, 0);
        assert!((metrics.songbird.request_rate - 0.0).abs() < f64::EPSILON);
        assert!((metrics.songbird.error_rate - 0.0).abs() < f64::EPSILON);
        assert!((metrics.songbird.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_snapshot_clone() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        let cloned = metrics.clone();
        assert!((metrics.system.cpu_usage - cloned.system.cpu_usage).abs() < f64::EPSILON);
        assert_eq!(metrics.songbird.active_services, cloned.songbird.active_services);
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_snapshot_serialization() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        let serialized =
            serde_json::to_string(&metrics).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {e}"),
                debug_info: None,
            })?;
        let deserialized: MetricsSnapshot =
            serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {e}"),
                debug_info: None,
            })?;

        assert!((metrics.system.cpu_usage - deserialized.system.cpu_usage).abs() < f64::EPSILON);
        assert_eq!(metrics.songbird.active_services, deserialized.songbird.active_services);
        Ok(())
    }

    #[tokio::test]
    async fn test_application_metrics_serialization() -> Result<()> {
        let app_metrics = ApplicationMetrics {
            active_services: 5,
            request_rate: 100.5,
            error_rate: 0.01,
            avg_response_time_ms: 25.3,
        };

        let serialized =
            serde_json::to_string(&app_metrics).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Serialization failed: {e}"),
                debug_info: None,
            })?;
        let deserialized: ApplicationMetrics =
            serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Parsing failed: {e}"),
                debug_info: None,
            })?;

        assert_eq!(app_metrics.active_services, deserialized.active_services);
        assert!((app_metrics.request_rate - deserialized.request_rate).abs() < f64::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_collections() -> Result<()> {
        let collector = Arc::new(MetricsCollector::new());

        let mut handles = vec![];
        for _ in 0..10 {
            let collector_clone = Arc::clone(&collector);
            let handle = tokio::spawn(async move { collector_clone.collect_all_metrics().await });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.map_err(|e| {
                SongbirdError::configuration(format!("Failed to join concurrent metrics task: {e}"))
            })?;
            assert!(result.is_ok());
        }

        assert_eq!(collector.get_collection_count(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_collector_debug() {
        let collector = MetricsCollector::new();
        let debug_str = format!("{collector:?}");
        assert!(debug_str.contains("MetricsCollector"));
    }

    #[tokio::test]
    async fn test_metrics_snapshot_debug() -> Result<()> {
        let collector = MetricsCollector::new();
        let metrics = collector
            .collect_all_metrics()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;
        let debug_str = format!("{metrics:?}");
        assert!(debug_str.contains("MetricsSnapshot"));
        Ok(())
    }

    #[tokio::test]
    async fn test_prometheus_export_format_correctness() -> Result<()> {
        let collector = MetricsCollector::new();
        let output = collector
            .export_prometheus()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Operation failed: {e}")))?;

        // Check format: metric name followed by value
        let mut found_metric_line = false;

        for line in output.lines() {
            if line.starts_with("songbird_cpu_usage_percent") && !line.starts_with('#') {
                found_metric_line = true;
                // Should have format: metric_name value
                assert_eq!(line.split_whitespace().count(), 2);
            }
        }

        assert!(found_metric_line, "Should have at least one metric line");
        Ok(())
    }
}
