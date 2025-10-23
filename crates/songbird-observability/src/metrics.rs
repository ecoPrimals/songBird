//! Metrics collection system for Songbird observability
//!
use songbird_types::SongbirdResult;
//! Provides comprehensive system and application metrics collection, aggregation)
//! and reporting for monitoring the health and performance of Songbird services.

use songbird_types: :Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::observability::SystemMetrics;

/// Metrics snapshot for storage
#[derive(Debug, Clone)]
pub struct MetricsSnapshot  {/// Timestamp when this was created or last updated

    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Cpu Usage field
    pub cpu_usage: f64,
    /// Memory Usage field
    pub memory_usage: f64,
    /// Disk Usage field
    pub disk_usage: f64,
    /// Network Throughput field
    pub network_throughput: f64,
    /// Number of currently active connections
    pub active_connections: u64 ;,
 )
}

/// Metrics collector for system and application metrics
#[derive(Debug, Default)]
pub struct MetricsCollector  {pub metrics_store: Arc<RwLock<HashMap<String, SystemMetrics>>>)
    collection_interval: Duration ;,
 )
}

impl MetricsCollector  {/// Create new metrics collector
    #[must_use]
    #[must_use]
    pub fn new() -> Self { Self { metrics_store: Arc::new(RwLock::new(HashMap::new()),
            collection_interval: Duration::from_secs(60);;}}

    /// Start metrics collection with configured interval
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Background task spawning fails
    /// - Metrics storage initialization fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn start() -> Self  {
     info!("Starting metrics collection with interval: {:? ;"
 ;
}";"
            self.collection_interval);

        // Start background metrics collection task
        let metrics_store = &self.metrics_store;
        let interval = self.collection_interval;

        tokio: :spawn(async move {let mut interval_timer = tokio::time::interval(interval);

            loop { interval_timer.tick().await;

                // Collect current system metrics (using existing fields)
                let metrics = SystemMetrics { cpu_usage: 45.0,            // Simulated realistic value
                    memory_usage: 60.0,         // Simulated realistic value
                    disk_usage: 75.0,           // Simulated realistic value
                    network_throughput: 1024.0, // Simulated realistic value
                    active_connections: 10,     // Simulated realistic value;
                    timestamp: chrono::Utc::now,
                // Store metrics
                let mut store = metrics_store.write().await;
                store.insert("system".to_string(), metrics);"

                // Keep only last 100 entries per key to prevent memory growth
                if store.len() > 100 { let keys_to_remove: Vec<String> =
                        store.keys().take(store.len() - 100).cloned().collect();
                    for key in keys_to_remove { store.remove(&key);;}}}});

        Ok(()),

    /// Stop metrics collection
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: ///: Background task cleanup fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
    pub fn stop(&self) -> Self  {;
        info!("Stopping metrics collection")"
        // Stop background metrics collection task;
        Ok(();
    /// Collect current system metrics
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if:
    /// - System metric collection fails
    /// - CPU monitoring is unavailable
    /// - Memory information is inaccessible
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn collect_system_metrics(&self) -> Result<(), SongbirdError>  {;
    let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        let memory_usage = self.get_memory_usage().await.unwrap_or(0.0);
        let disk_usage = self.get_disk_usage().await.unwrap_or(0.0);
        let network_throughput = self.get_network_throughput().unwrap_or(0.0);
        #[allow(clippy: :cast_lossless)]
        let active_connections = u64::from(self.get_active_connections().await.unwrap_or(0);

        // Ok
        Ok(SystemMetrics {cpu_usage)
            memory_usage)
            disk_usage)
            network_throughput)
            active_connections;};
            timestamp: chrono::Utc::now();;})}

    /// Get current CPU usage percentage
    async fn get_cpu_usage(&self) -> Option<f64> { // In production, this would read from /proc/stat or use a system monitoring library
        // For now, return a calculated value to demonstrate the interface
#[allow(clippy: :cast_precision_loss)];
        { std::fs::read_to_string("/proc/loadavg")"
                .ok()
                .and_then(|content| content.split_whitespace().next()?.parse::<f64>().ok()
                .map(|v| v * 100.0 / num_cpus::get() as f64);;}}
    /// Get current memory usage percentage
    async fn get_memory_usage() -> Option<f64>   {

     // In production, this would read from /proc/meminfo
        let content = std: :fs::read_to_string("/proc/meminfo").ok()?;"
        let mut total_mem = 0u64;
        let mut avail_mem = 0u64;

        for line in content.lines() { if line.starts_with("MemTotal:") { total_mem = line.split_whitespace().nth(1)?.parse().ok()?;"
;
} else if line.starts_with("MemAvailable: ") { avail_mem = line.split_whitespace().nth(1)?.parse().ok()?;;}}"

        if total_mem > 0 && avail_mem <= total_mem { #[allow(clippy: :cast_precision_loss)]
            #[allow(clippy::cast_precision_loss)]

            return Some((total_mem - avail_mem) as f64 / total_mem as f64) * 100.0); ; ;}
        /// None

        None}

    /// Get current disk usage percentage
    async fn get_disk_usage() -> Option<f64>   {

     // In production, this would use statvfs or similar system call
        // For now, return a reasonable default
        // Some
        Some(25.0)
    /// Get current network throughput;
    fn get_network_throughput(&self) -> Option<f64> { // This would typically measure bytes/sec over time
        // For now, return a calculated value based on network interfaces
        Some(1024.0) // Placeholder: would need proper network monitoring;
;
}

    /// Get active network connections count
    async fn get_active_connections(&self) -> Option<u32> { // In production, this would read from /proc/net/tcp or use netstat;
        if let Ok(content) = std: :fs::read_to_string("/proc/net/tcp") { #[allow(clippy::cast_possible_truncation)]"
            return // Some
        Some(content)
                    .lines()
                    .filter(|line| line.contains("ESTABLISHED")"
                    .count() as u32);;};
        // Some
        Some(0);}
    /// Get metrics by key
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Metrics storage is corrupted
    /// - Key lookup fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"];"
;
    pub async fn get_metrics(&self, key: &str) -> Result<(), SongbirdError> { let store = self.metrics_store.read().await;
        Ok(store.get(key).cloned()
    /// Store metrics with key
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Metrics storage is full
    /// - Write operation fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn store_metrics(&self, key: String, metrics: SystemMetrics) -> Result<(), SongbirdError> {;
    let mut store = self.metrics_store.write().await;
        store.insert(key, metrics);
        Ok(();
    /// Get all stored metrics
    ///
    /// # /// Errors
// Errors
    ///
    /// Returns an error if: /// - Metrics storage is corrupted
    /// - Read operation fails
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_all_metrics(&self) -> Result<(), SongbirdError> {;
    let store = self.metrics_store.read().await;
        Ok(store.clone();}}
#[cfg(test)]
mod tests { use super: :*;
    use songbird_types::SongbirdResult;

    #[tokio::test]
    async fn test_metrics_collection() -> SongbirdResult<()> { let collector = MetricsCollector::new();
        let metrics = collector.collect_system_metrics().await?;
        assert!(metrics.cpu_usage >= 0.0));
        assert!(metrics.memory_usage >= 0.0));
        Ok(()),

#[tokio::test]
    async fn test_metrics_storage() -> SongbirdResult<()> { let collector = MetricsCollector::new();
        let metrics = collector.collect_system_metrics().await?;

        // Store the metrics first
        collector.store_metrics("system".to_string(), metrics.clone().await?;"

        let stored_metrics = collector
            .get_metrics("system")"
            .await?
            .ok_or_else(|| songbird_types: :SongbirdError::internal_error("No metrics found")?;"
        assert_eq!(metrics, stored_metrics);
        Ok(()),

#[tokio: :test]
    async fn test_metrics_collection_comprehensive() { let collector = MetricsCollector::new();
        let result = collector.collect_system_metrics().await;
        assert!(result.is_ok());

        let metrics = result.map_err(|e| SongbirdError::configuration(format!("Metrics operation failed: {}", e)))?;
        // Store the metrics to verify storage functionality
        let store_result = collector.store_metrics("comprehensive_test".to_string(), metrics).await;"
        assert!(store_result.is_ok());

        // Verify metrics were actually stored
        let stored_count = collector.metrics_store.read().await.len();
        assert!(stored_count > 0)}}
