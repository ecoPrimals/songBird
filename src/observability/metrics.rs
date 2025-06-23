//! Metrics Collection Module
//!
//! System and application metrics collection using sysinfo and internal tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;
use sysinfo::System;
use std::collections::HashMap;

use crate::errors::{Result, SongbirdError};
use super::ObservabilityConfig;

/// System metrics collected from the host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage: f32,
    /// Memory usage ratio (0.0 - 1.0)
    pub memory_usage: f64,
    /// Memory usage in bytes
    pub memory_used_bytes: u64,
    /// Total memory in bytes
    pub memory_total_bytes: u64,
    /// Disk usage metrics
    pub disk_usage: DiskMetrics,
    /// System uptime
    pub uptime: Duration,
    /// Load average
    pub load_average: LoadAverage,
    /// Network statistics
    pub network_stats: NetworkMetrics,
    /// Number of running processes
    pub process_count: u32,
}

/// Disk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Total disk space in bytes
    pub total_bytes: u64,
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Used disk space in bytes
    pub used_bytes: u64,
    /// Disk usage percentage
    pub usage_percentage: f64,
}

/// Load average metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    /// 1-minute load average
    pub one: f64,
    /// 5-minute load average
    pub five: f64,
    /// 15-minute load average
    pub fifteen: f64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes received
    pub bytes_received: u64,
    /// Bytes transmitted
    pub bytes_transmitted: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets transmitted
    pub packets_transmitted: u64,
    /// Network errors
    pub errors: u64,
}

/// Process-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// CPU usage by this process
    pub cpu_usage: f32,
    /// Memory usage by this process in bytes
    pub memory_bytes: u64,
    /// Virtual memory usage
    pub virtual_memory_bytes: u64,
    /// Number of threads
    pub thread_count: u32,
    /// Number of file descriptors
    pub fd_count: u32,
}

/// Songbird-specific application metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdMetrics {
    /// Number of active services
    pub active_services: u64,
    /// Number of federation nodes
    pub federation_nodes: u64,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Circuit breaker statistics
    pub circuit_breakers: CircuitBreakerMetrics,
    /// Load balancer statistics
    pub load_balancer: LoadBalancerMetrics,
    /// Communication layer statistics
    pub communication: CommunicationMetrics,
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    /// Number of open circuit breakers
    pub open_circuits: u64,
    /// Number of half-open circuit breakers
    pub half_open_circuits: u64,
    /// Total number of circuit breaker trips
    pub total_trips: u64,
    /// Total rejected requests due to circuit breakers
    pub rejected_requests: u64,
}

/// Load balancer metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Active backend instances
    pub active_backends: u64,
    /// Average request duration
    pub avg_request_duration_ms: f64,
}

/// Communication layer metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationMetrics {
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Active connections
    pub active_connections: u64,
    /// Connection errors
    pub connection_errors: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
}

/// Complete metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// System-level metrics
    pub system: SystemMetrics,
    /// Songbird application metrics
    pub songbird: SongbirdMetrics,
    /// Registered services
    pub services: HashMap<String, crate::traits::service::ServiceInfo>,
    /// Timestamp when metrics were collected
    pub timestamp: DateTime<Utc>,
    /// Collection duration
    pub collection_duration_ms: u64,
}

/// Metrics collector implementation
pub struct MetricsCollector {
    config: ObservabilityConfig,
    system: std::sync::Mutex<sysinfo::System>,
    start_time: Instant,
    collection_count: AtomicU64,
    last_collection_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    // Metrics history (for trends)
    history: Arc<RwLock<Vec<MetricsSnapshot>>>,
    // Application metrics tracking
    app_metrics: Arc<RwLock<SongbirdMetrics>>,
    // Application counters
    service_count: AtomicU64,
    request_count: AtomicU64,
    error_count: AtomicU64,
    total_response_time: AtomicU64,
    last_request_count: AtomicU64,
    last_error_count: AtomicU64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: ObservabilityConfig) -> Result<Self> {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        Ok(Self {
            config,
            system: std::sync::Mutex::new(system),
            start_time: Instant::now(),
            collection_count: AtomicU64::new(0),
            last_collection_time: Arc::new(RwLock::new(None)),
            history: Arc::new(RwLock::new(Vec::new())),
            app_metrics: Arc::new(RwLock::new(SongbirdMetrics::default())),
            service_count: AtomicU64::new(0),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_response_time: AtomicU64::new(0),
            last_request_count: AtomicU64::new(0),
            last_error_count: AtomicU64::new(0),
        })
    }

    /// Collect all metrics (system + application)
    pub async fn collect_all_metrics(&self) -> Result<MetricsSnapshot> {
        let collection_start = Instant::now();
        
        // Collect system metrics
        let system_metrics = self.collect_system_metrics().await?;
        
        // Collect application metrics
        let songbird_metrics = self.collect_songbird_metrics().await?;
        
        let collection_duration = collection_start.elapsed();
        let snapshot = MetricsSnapshot {
            system: system_metrics,
            songbird: songbird_metrics,
            services: HashMap::new(),
            timestamp: Utc::now(),
            collection_duration_ms: collection_duration.as_millis() as u64,
        };

        // Update collection tracking
        self.collection_count.fetch_add(1, Ordering::Relaxed);
        *self.last_collection_time.write().await = Some(snapshot.timestamp);

        // Store in history (with limit)
        let mut history = self.history.write().await;
        history.push(snapshot.clone());
        
        // Keep only the last N snapshots
        if history.len() > self.config.max_metric_history {
            history.remove(0);
        }

        debug!("Collected metrics in {:?}", collection_duration);
        Ok(snapshot)
    }

    /// Collect system metrics using sysinfo
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system.lock().map_err(|e| {
            SongbirdError::Service {
                message: format!("Failed to lock system info: {}", e),
            }
        })?;

        // Refresh system information
        system.refresh_all();

        // Get CPU usage - use global CPU info
        let cpu_usage = system.global_cpu_info().cpu_usage();

        // Get memory usage
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = if total_memory > 0 {
            used_memory as f64 / total_memory as f64
        } else {
            0.0
        };

        // Get load average - use static method
        let load_avg = System::load_average();
        let load_average = LoadAverage {
            one: load_avg.one,
            five: load_avg.five,
            fifteen: load_avg.fifteen,
        };

        // Get uptime - use static method
        let uptime = Duration::from_secs(System::uptime());

        // Get disk usage
        let disk_metrics = Self::collect_disk_metrics(&system);

        // Get network metrics
        let network_metrics = Self::collect_network_metrics(&system);

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            memory_used_bytes: used_memory,
            memory_total_bytes: total_memory,
            disk_usage: disk_metrics,
            uptime,
            load_average,
            network_stats: network_metrics,
            process_count: system.processes().len() as u32,
        })
    }

    /// Collect Songbird application metrics
    pub async fn collect_songbird_metrics(&self) -> Result<SongbirdMetrics> {
        let app_metrics = self.app_metrics.read().await.clone();
        Ok(app_metrics)
    }

    /// Get current metrics snapshot
    pub async fn get_current_snapshot(&self) -> Result<MetricsSnapshot> {
        self.collect_all_metrics().await
    }

    /// Get the total number of metrics collections performed
    pub fn get_collection_count(&self) -> u64 {
        self.collection_count.load(Ordering::Relaxed)
    }

    /// Get the timestamp of the last metrics collection
    pub fn last_collection_time(&self) -> Option<DateTime<Utc>> {
        // Use try_read to avoid blocking, return None if lock is held
        self.last_collection_time.try_read().ok().and_then(|guard| *guard)
    }

    /// Get metrics history
    pub async fn get_history(&self) -> Vec<MetricsSnapshot> {
        self.history.read().await.clone()
    }

    /// Update application metrics
    pub async fn update_app_metrics<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut SongbirdMetrics),
    {
        let mut app_metrics = self.app_metrics.write().await;
        updater(&mut app_metrics);
        Ok(())
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> Result<String> {
        let snapshot = self.get_current_snapshot().await?;
        
        let mut output = String::new();
        
        // System metrics
        output.push_str(&format!("# HELP songbird_cpu_usage_percent CPU usage percentage\n"));
        output.push_str(&format!("# TYPE songbird_cpu_usage_percent gauge\n"));
        output.push_str(&format!("songbird_cpu_usage_percent {}\n", snapshot.system.cpu_usage));
        
        output.push_str(&format!("# HELP songbird_memory_usage_ratio Memory usage ratio\n"));
        output.push_str(&format!("# TYPE songbird_memory_usage_ratio gauge\n"));
        output.push_str(&format!("songbird_memory_usage_ratio {}\n", snapshot.system.memory_usage));
        
        output.push_str(&format!("# HELP songbird_disk_usage_percent Disk usage percentage\n"));
        output.push_str(&format!("# TYPE songbird_disk_usage_percent gauge\n"));
        output.push_str(&format!("songbird_disk_usage_percent {}\n", snapshot.system.disk_usage.usage_percentage));
        
        // Application metrics
        output.push_str(&format!("# HELP songbird_active_services Number of active services\n"));
        output.push_str(&format!("# TYPE songbird_active_services gauge\n"));
        output.push_str(&format!("songbird_active_services {}\n", snapshot.songbird.active_services));
        
        output.push_str(&format!("# HELP songbird_request_rate Requests per second\n"));
        output.push_str(&format!("# TYPE songbird_request_rate gauge\n"));
        output.push_str(&format!("songbird_request_rate {}\n", snapshot.songbird.request_rate));
        
        output.push_str(&format!("# HELP songbird_error_rate Errors per second\n"));
        output.push_str(&format!("# TYPE songbird_error_rate gauge\n"));
        output.push_str(&format!("songbird_error_rate {}\n", snapshot.songbird.error_rate));
        
        Ok(output)
    }

    /// Collect disk metrics
    fn collect_disk_metrics(_system: &sysinfo::System) -> DiskMetrics {
        // For now, return default values as sysinfo 0.30 API has changed
        // In a production system, we'd use a different approach or library
        DiskMetrics {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            usage_percentage: 0.0,
        }
    }

    /// Collect network metrics
    fn collect_network_metrics(_system: &sysinfo::System) -> NetworkMetrics {
        // For now, return default values as sysinfo 0.30 API has changed
        // In a production system, we'd use a different approach or library  
        NetworkMetrics {
            bytes_received: 0,
            bytes_transmitted: 0,
            packets_received: 0,
            packets_transmitted: 0,
            errors: 0,
        }
    }

    /// Update service count
    pub fn update_service_count(&self, count: u64) {
        self.service_count.store(count, Ordering::Relaxed);
    }

    /// Increment request count
    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error count
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Add response time
    pub fn add_response_time(&self, response_time_ms: u64) {
        self.total_response_time.fetch_add(response_time_ms, Ordering::Relaxed);
    }

    /// Get current metrics snapshot (alias for compatibility)
    pub async fn get_current_metrics(&self) -> Result<MetricsSnapshot> {
        self.collect_all_metrics().await
    }
}

impl Default for SongbirdMetrics {
    fn default() -> Self {
        Self {
            active_services: 0,
            federation_nodes: 0,
            request_rate: 0.0,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
            circuit_breakers: CircuitBreakerMetrics::default(),
            load_balancer: LoadBalancerMetrics::default(),
            communication: CommunicationMetrics::default(),
        }
    }
}

impl Default for CircuitBreakerMetrics {
    fn default() -> Self {
        Self {
            open_circuits: 0,
            half_open_circuits: 0,
            total_trips: 0,
            rejected_requests: 0,
        }
    }
}

impl Default for LoadBalancerMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            failed_requests: 0,
            active_backends: 0,
            avg_request_duration_ms: 0.0,
        }
    }
}

impl Default for CommunicationMetrics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            active_connections: 0,
            connection_errors: 0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    fn create_test_config() -> ObservabilityConfig {
        ObservabilityConfig {
            enabled: true,
            metrics_interval_secs: 30,
            health_check_interval_secs: 60,
            enable_dashboard: false,
            dashboard_port: 8081,
            export_prometheus: true,
            max_metric_history: 100,
            enable_system_metrics: true,
            enable_service_metrics: true,
        }
    }

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        assert_eq!(collector.get_collection_count(), 0);
        assert!(collector.last_collection_time().is_none());
    }

    #[tokio::test]
    async fn test_system_metrics_collection() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        let metrics = collector.collect_system_metrics().await.unwrap();
        
        // Basic validation of system metrics
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0 && metrics.memory_usage <= 1.0);
        assert!(metrics.memory_total_bytes > 0);
        assert!(metrics.memory_used_bytes <= metrics.memory_total_bytes);
        
        // Load average should have values
        assert!(metrics.load_average.one >= 0.0);
        assert!(metrics.load_average.five >= 0.0);
        assert!(metrics.load_average.fifteen >= 0.0);
        
        // Uptime should be positive
        assert!(metrics.uptime.as_secs() > 0);
        
        // Process count should be positive
        assert!(metrics.process_count > 0);
        
        // Disk metrics (currently returning defaults due to API limitations)
        assert_eq!(metrics.disk_usage.total_bytes, 0);
        assert_eq!(metrics.disk_usage.usage_percentage, 0.0);
        
        // Network metrics (currently returning defaults due to API limitations)
        assert_eq!(metrics.network_stats.bytes_received, 0);
        assert_eq!(metrics.network_stats.bytes_transmitted, 0);
    }

    #[tokio::test]
    async fn test_songbird_metrics_collection() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        let metrics = collector.collect_songbird_metrics().await.unwrap();
        
        // Should start with default values
        assert_eq!(metrics.active_services, 0);
        assert_eq!(metrics.federation_nodes, 0);
        assert_eq!(metrics.request_rate, 0.0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.avg_response_time_ms, 0.0);
        
        // Circuit breaker metrics
        assert_eq!(metrics.circuit_breakers.open_circuits, 0);
        assert_eq!(metrics.circuit_breakers.half_open_circuits, 0);
        assert_eq!(metrics.circuit_breakers.total_trips, 0);
        assert_eq!(metrics.circuit_breakers.rejected_requests, 0);
        
        // Load balancer metrics
        assert_eq!(metrics.load_balancer.total_requests, 0);
        assert_eq!(metrics.load_balancer.failed_requests, 0);
        assert_eq!(metrics.load_balancer.active_backends, 0);
        assert_eq!(metrics.load_balancer.avg_request_duration_ms, 0.0);
        
        // Communication metrics
        assert_eq!(metrics.communication.messages_sent, 0);
        assert_eq!(metrics.communication.messages_received, 0);
        assert_eq!(metrics.communication.active_connections, 0);
        assert_eq!(metrics.communication.connection_errors, 0);
        assert_eq!(metrics.communication.bytes_sent, 0);
        assert_eq!(metrics.communication.bytes_received, 0);
    }

    #[tokio::test]
    async fn test_complete_metrics_collection() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        let snapshot = collector.collect_all_metrics().await.unwrap();
        
        // Verify snapshot structure
        assert!(snapshot.collection_duration_ms > 0);
        assert!(snapshot.timestamp <= Utc::now());
        
        // Verify system metrics are present
        assert!(snapshot.system.cpu_usage >= 0.0);
        assert!(snapshot.system.memory_usage >= 0.0);
        
        // Verify Songbird metrics are present
        assert_eq!(snapshot.songbird.active_services, 0);
        
        // Collection count should be incremented
        assert_eq!(collector.get_collection_count(), 1);
        assert!(collector.last_collection_time().is_some());
    }

    #[tokio::test]
    async fn test_metrics_history() {
        let mut config = create_test_config();
        config.max_metric_history = 3; // Small limit for testing
        
        let collector = MetricsCollector::new(config).unwrap();
        
        // Collect multiple snapshots
        for _ in 0..5 {
            collector.collect_all_metrics().await.unwrap();
            sleep(Duration::from_millis(10)).await; // Small delay to ensure different timestamps
        }
        
        let history = collector.get_history().await;
        
        // Should be limited to max_metric_history
        assert_eq!(history.len(), 3);
        
        // Should be in chronological order
        for i in 1..history.len() {
            assert!(history[i].timestamp >= history[i-1].timestamp);
        }
    }

    #[tokio::test]
    async fn test_app_metrics_update() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        // Update application metrics
        collector.update_app_metrics(|metrics| {
            metrics.active_services = 5;
            metrics.federation_nodes = 3;
            metrics.request_rate = 100.5;
            metrics.error_rate = 2.1;
            metrics.avg_response_time_ms = 45.7;
        }).await.unwrap();
        
        let metrics = collector.collect_songbird_metrics().await.unwrap();
        
        assert_eq!(metrics.active_services, 5);
        assert_eq!(metrics.federation_nodes, 3);
        assert_eq!(metrics.request_rate, 100.5);
        assert_eq!(metrics.error_rate, 2.1);
        assert_eq!(metrics.avg_response_time_ms, 45.7);
    }

    #[tokio::test]
    async fn test_service_count_tracking() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        // Update service count
        collector.update_service_count(10);
        
        // Update application metrics to reflect the change
        collector.update_app_metrics(|metrics| {
            metrics.active_services = 10;
        }).await.unwrap();
        
        let metrics = collector.collect_songbird_metrics().await.unwrap();
        assert_eq!(metrics.active_services, 10);
    }

    #[tokio::test]
    async fn test_request_and_error_tracking() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        // Simulate requests and errors
        for _ in 0..10 {
            collector.increment_request_count();
        }
        
        for _ in 0..2 {
            collector.increment_error_count();
        }
        
        // Add some response times
        collector.add_response_time(100);
        collector.add_response_time(200);
        collector.add_response_time(50);
        
        // The actual calculation would happen in a real metrics update cycle
        // For now, we just verify the methods work without panicking
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        // Update some metrics
        collector.update_app_metrics(|metrics| {
            metrics.active_services = 5;
            metrics.request_rate = 100.0;
            metrics.error_rate = 2.0;
        }).await.unwrap();
        
        let prometheus_output = collector.export_prometheus().await.unwrap();
        
        // Verify Prometheus format
        assert!(prometheus_output.contains("# HELP songbird_cpu_usage_percent"));
        assert!(prometheus_output.contains("# TYPE songbird_cpu_usage_percent gauge"));
        assert!(prometheus_output.contains("songbird_cpu_usage_percent"));
        
        assert!(prometheus_output.contains("# HELP songbird_memory_usage_ratio"));
        assert!(prometheus_output.contains("# TYPE songbird_memory_usage_ratio gauge"));
        assert!(prometheus_output.contains("songbird_memory_usage_ratio"));
        
        assert!(prometheus_output.contains("# HELP songbird_disk_usage_percent"));
        assert!(prometheus_output.contains("# TYPE songbird_disk_usage_percent gauge"));
        assert!(prometheus_output.contains("songbird_disk_usage_percent"));
        
        assert!(prometheus_output.contains("# HELP songbird_active_services"));
        assert!(prometheus_output.contains("# TYPE songbird_active_services gauge"));
        assert!(prometheus_output.contains("songbird_active_services 5"));
        
        assert!(prometheus_output.contains("# HELP songbird_request_rate"));
        assert!(prometheus_output.contains("# TYPE songbird_request_rate gauge"));
        assert!(prometheus_output.contains("songbird_request_rate 100"));
        
        assert!(prometheus_output.contains("# HELP songbird_error_rate"));
        assert!(prometheus_output.contains("# TYPE songbird_error_rate gauge"));
        assert!(prometheus_output.contains("songbird_error_rate 2"));
    }

    #[tokio::test]
    async fn test_disk_metrics_default() {
        let system = sysinfo::System::new();
        let disk_metrics = MetricsCollector::collect_disk_metrics(&system);
        
        // Currently returns defaults due to API changes
        assert_eq!(disk_metrics.total_bytes, 0);
        assert_eq!(disk_metrics.used_bytes, 0);
        assert_eq!(disk_metrics.available_bytes, 0);
        assert_eq!(disk_metrics.usage_percentage, 0.0);
    }

    #[tokio::test]
    async fn test_network_metrics_default() {
        let system = sysinfo::System::new();
        let network_metrics = MetricsCollector::collect_network_metrics(&system);
        
        // Currently returns defaults due to API changes
        assert_eq!(network_metrics.bytes_received, 0);
        assert_eq!(network_metrics.bytes_transmitted, 0);
        assert_eq!(network_metrics.packets_received, 0);
        assert_eq!(network_metrics.packets_transmitted, 0);
        assert_eq!(network_metrics.errors, 0);
    }

    #[tokio::test]
    async fn test_load_average() {
        let load_avg = sysinfo::System::load_average();
        
        // Load average should be reasonable values
        assert!(load_avg.one >= 0.0);
        assert!(load_avg.five >= 0.0);
        assert!(load_avg.fifteen >= 0.0);
    }

    #[tokio::test]
    async fn test_system_uptime() {
        let uptime = sysinfo::System::uptime();
        
        // Uptime should be positive
        assert!(uptime > 0);
    }

    #[tokio::test]
    async fn test_memory_metrics_consistency() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        let metrics = collector.collect_system_metrics().await.unwrap();
        
        // Memory usage should be consistent
        assert!(metrics.memory_used_bytes <= metrics.memory_total_bytes);
        assert!(metrics.memory_usage <= 1.0);
        
        // Memory usage ratio should match bytes calculation
        let calculated_ratio = metrics.memory_used_bytes as f64 / metrics.memory_total_bytes as f64;
        assert!((metrics.memory_usage - calculated_ratio).abs() < 0.001);
    }
} 