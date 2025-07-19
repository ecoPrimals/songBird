//! Performance monitor for real-time monitoring

use super::config::{PerformanceConfig, PerformanceTuningResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance monitor for real-time metrics collection
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    metrics: Arc<RwLock<SystemMetrics>>,
    start_time: Instant,
}

/// System performance metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: usize,
    pub network_throughput_mbps: f64,
    pub active_connections: usize,
    pub request_latency_ms: f64,
    pub cache_hit_ratio: f64,
    pub error_rate: f64,
    pub last_updated: Instant,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            network_throughput_mbps: 0.0,
            active_connections: 0,
            request_latency_ms: 0.0,
            cache_hit_ratio: 0.0,
            error_rate: 0.0,
            last_updated: Instant::now(),
        }
    }
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            start_time: Instant::now(),
        }
    }

    /// Start performance monitoring
    pub async fn start_monitoring(&self) {
        let metrics = self.metrics.clone();
        let interval = self.config.monitoring_interval;

        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);

            loop {
                timer.tick().await;

                // Collect system metrics
                let mut metrics_guard = metrics.write().await;

                // Update CPU usage
                metrics_guard.cpu_usage_percent = Self::collect_cpu_usage().await;

                // Update memory usage
                metrics_guard.memory_usage_mb = Self::collect_memory_usage().await;

                // Update network metrics
                metrics_guard.network_throughput_mbps = Self::collect_network_throughput().await;

                // Update timestamp
                metrics_guard.last_updated = Instant::now();
            }
        });
    }

    /// Get current system metrics
    pub async fn get_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }

    /// Update specific metric
    pub async fn update_metric(&self, update: MetricUpdate) {
        let mut metrics = self.metrics.write().await;

        match update {
            MetricUpdate::CpuUsage(value) => metrics.cpu_usage_percent = value,
            MetricUpdate::MemoryUsage(value) => metrics.memory_usage_mb = value,
            MetricUpdate::NetworkThroughput(value) => metrics.network_throughput_mbps = value,
            MetricUpdate::ActiveConnections(value) => metrics.active_connections = value,
            MetricUpdate::RequestLatency(value) => {
                // Use exponential moving average
                metrics.request_latency_ms = metrics.request_latency_ms * 0.9 + value * 0.1;
            }
            MetricUpdate::CacheHitRatio(value) => metrics.cache_hit_ratio = value,
            MetricUpdate::ErrorRate(value) => {
                // Use exponential moving average
                metrics.error_rate = metrics.error_rate * 0.9 + value * 0.1;
            }
        }

        metrics.last_updated = Instant::now();
    }

    /// Generate performance tuning recommendations
    pub async fn generate_tuning_recommendations(&self) -> PerformanceTuningResult {
        let metrics = self.metrics.read().await;
        let mut result = PerformanceTuningResult::new();

        result.cache_hit_ratio = metrics.cache_hit_ratio;
        result.avg_response_time = Duration::from_millis(metrics.request_latency_ms as u64);
        result.memory_usage_mb = metrics.memory_usage_mb;
        result.cpu_usage_percent = metrics.cpu_usage_percent;

        // Generate recommendations based on metrics
        if metrics.cpu_usage_percent > 80.0 {
            result.add_recommendation("High CPU usage detected. Consider scaling horizontally or optimizing CPU-intensive operations.".to_string());
        }

        if metrics.memory_usage_mb > (self.config.cache_size_mb * 2) {
            result.add_recommendation("High memory usage detected. Consider increasing available memory or reducing cache size.".to_string());
        }

        if metrics.cache_hit_ratio < 0.8 {
            result.add_recommendation("Low cache hit ratio. Consider increasing cache size or improving cache key strategies.".to_string());
        }

        if metrics.request_latency_ms > 100.0 {
            result.add_recommendation("High request latency detected. Consider optimizing database queries or enabling caching.".to_string());
        }

        if metrics.error_rate > 0.05 {
            result.add_recommendation(
                "Elevated error rate detected. Review application logs and error handling."
                    .to_string(),
            );
        }

        if metrics.active_connections < 10 && self.config.enable_fast_load_balancing {
            result.add_recommendation("Low connection utilization. Consider reducing load balancer overhead for small workloads.".to_string());
        }

        result
    }

    /// Check if system performance is healthy
    pub async fn is_healthy(&self) -> bool {
        let metrics = self.metrics.read().await;

        metrics.cpu_usage_percent < 80.0
            && metrics.request_latency_ms < 100.0
            && metrics.error_rate < 0.05
            && metrics.cache_hit_ratio > 0.8
    }

    /// Get uptime duration
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get performance score (0.0-1.0)
    pub async fn get_performance_score(&self) -> f64 {
        let metrics = self.metrics.read().await;

        let cpu_score = (100.0 - metrics.cpu_usage_percent) / 100.0;
        let latency_score = if metrics.request_latency_ms <= 10.0 {
            1.0
        } else {
            1.0 / (metrics.request_latency_ms / 10.0)
        };
        let cache_score = metrics.cache_hit_ratio;
        let error_score = 1.0 - metrics.error_rate.min(1.0);

        (cpu_score + latency_score + cache_score + error_score) / 4.0
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = SystemMetrics::default();
    }

    /// Collect CPU usage (mock implementation)
    async fn collect_cpu_usage() -> f64 {
        // In production, this would use system APIs like:
        // - /proc/stat on Linux
        // - Windows Performance Counters
        // - macOS system calls

        // Mock implementation
        fastrand::f64() * 100.0
    }

    /// Collect memory usage (mock implementation)
    async fn collect_memory_usage() -> usize {
        // In production, this would use system APIs like:
        // - /proc/meminfo on Linux
        // - GlobalMemoryStatusEx on Windows
        // - mach_host_self() on macOS

        // Mock implementation
        (fastrand::u32(100..2048)) as usize
    }

    /// Collect network throughput (mock implementation)
    async fn collect_network_throughput() -> f64 {
        // In production, this would use system APIs like:
        // - /proc/net/dev on Linux
        // - GetIfTable on Windows
        // - getifaddrs on macOS

        // Mock implementation
        fastrand::f64() * 1000.0
    }
}

/// Metric update enumeration
pub enum MetricUpdate {
    CpuUsage(f64),
    MemoryUsage(usize),
    NetworkThroughput(f64),
    ActiveConnections(usize),
    RequestLatency(f64),
    CacheHitRatio(f64),
    ErrorRate(f64),
}

impl SystemMetrics {
    /// Check if metrics are stale
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_updated.elapsed() > max_age
    }

    /// Get resource utilization score (0.0-1.0)
    pub fn resource_utilization_score(&self) -> f64 {
        let cpu_util = self.cpu_usage_percent / 100.0;
        let memory_util = (self.memory_usage_mb as f64 / 4096.0).min(1.0); // Assume 4GB max
        let network_util = (self.network_throughput_mbps / 1000.0).min(1.0); // Assume 1Gbps max

        (cpu_util + memory_util + network_util) / 3.0
    }

    /// Get service quality score (0.0-1.0)
    pub fn service_quality_score(&self) -> f64 {
        let latency_score = if self.request_latency_ms <= 10.0 {
            1.0
        } else {
            1.0 / (self.request_latency_ms / 10.0)
        };
        let cache_score = self.cache_hit_ratio;
        let reliability_score = 1.0 - self.error_rate.min(1.0);

        (latency_score + cache_score + reliability_score) / 3.0
    }
}
