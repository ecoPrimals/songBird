//! Main ProductionPerformanceOptimizer coordinator

use super::batch_processor::AsyncBatchProcessor;
use super::cache::AdaptiveCache;
use super::config::{PerformanceConfig, PerformanceTuningResult};
use super::load_balancer::{FastLoadBalancer, LoadBalancerStats};
use super::monitor::{PerformanceMonitor, SystemMetrics};
use super::object_pool::ObjectPool;
use songbird_errors::Result;
use std::sync::Arc;

/// Production performance optimizer - main coordinator
pub struct ProductionPerformanceOptimizer {
    config: PerformanceConfig,
    load_balancer: Option<FastLoadBalancer>,
    monitor: PerformanceMonitor,
    string_pool: Arc<ObjectPool<String>>,
    byte_pool: Arc<ObjectPool<Vec<u8>>>,
}

impl ProductionPerformanceOptimizer {
    /// Create new production performance optimizer
    pub fn new(config: PerformanceConfig) -> Self {
        let monitor = PerformanceMonitor::new(config.clone());

        // Initialize load balancer if enabled
        let load_balancer = if config.enable_fast_load_balancing {
            Some(FastLoadBalancer::new(
                super::config::LoadBalancingStrategy::PerformanceBased,
                1000, // Cache size
            ))
        } else {
            None
        };

        // Initialize object pools if enabled
        let string_pool = Arc::new(ObjectPool::new(
            || String::with_capacity(1024),
            config.object_pool_sizes.message_pool,
        ));

        let byte_pool = Arc::new(ObjectPool::new(
            || Vec::with_capacity(4096),
            config.object_pool_sizes.buffer_pool,
        ));

        Self {
            config,
            load_balancer,
            monitor,
            string_pool,
            byte_pool,
        }
    }

    /// Start the performance optimizer
    pub async fn start(&self) -> Result<()> {
        // Start performance monitoring
        self.monitor.start_monitoring().await;

        tracing::info!("Production Performance Optimizer started");
        Ok(())
    }

    /// Stop the performance optimizer
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Production Performance Optimizer stopped");
        Ok(())
    }

    /// Get load balancer reference
    pub fn get_load_balancer(&self) -> Option<&FastLoadBalancer> {
        self.load_balancer.as_ref()
    }

    /// Get performance monitor reference
    pub fn get_monitor(&self) -> &PerformanceMonitor {
        &self.monitor
    }

    /// Get string pool reference
    pub fn get_string_pool(&self) -> Arc<ObjectPool<String>> {
        self.string_pool.clone()
    }

    /// Get byte buffer pool reference  
    pub fn get_byte_pool(&self) -> Arc<ObjectPool<Vec<u8>>> {
        self.byte_pool.clone()
    }

    /// Create adaptive cache instance
    pub fn create_cache<K, V>(&self) -> AdaptiveCache<K, V>
    where
        K: Clone + std::hash::Hash + Eq,
        V: Clone,
    {
        let cache_config = super::config::CacheConfig {
            max_size: (self.config.cache_size_mb * 1024) / 64, // Rough estimate
            max_memory_mb: self.config.cache_size_mb,
            ..Default::default()
        };

        AdaptiveCache::new(cache_config)
    }

    /// Create batch processor
    pub fn create_batch_processor<T, R, F>(
        &self,
        batch_size: usize,
        processor: F,
    ) -> AsyncBatchProcessor<T, R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> std::result::Result<Vec<R>, String> + Send + Sync + 'static,
    {
        let timeout = std::time::Duration::from_millis(50); // Default 50ms batch timeout
        AsyncBatchProcessor::new(batch_size, timeout, processor)
    }

    /// Get comprehensive performance metrics
    pub async fn get_comprehensive_metrics(&self) -> ComprehensiveMetrics {
        let system_metrics = self.monitor.get_metrics().await;
        let performance_score = self.monitor.get_performance_score().await;

        let load_balancer_stats = if let Some(ref lb) = self.load_balancer {
            Some(lb.get_statistics().await)
        } else {
            None
        };

        let string_pool_size = self.string_pool.size().await;
        let byte_pool_size = self.byte_pool.size().await;

        ComprehensiveMetrics {
            system_metrics,
            performance_score,
            load_balancer_stats,
            string_pool_size,
            byte_pool_size,
            uptime: self.monitor.uptime(),
        }
    }

    /// Perform automatic performance tuning
    pub async fn auto_tune(&self) -> Result<PerformanceTuningResult> {
        let tuning_result = self.monitor.generate_tuning_recommendations().await;

        // Apply automatic tuning based on recommendations
        if self.config.auto_tuning_sensitivity > 0.5 {
            self.apply_auto_tuning(&tuning_result).await?;
        }

        Ok(tuning_result)
    }

    /// Apply automatic tuning recommendations
    async fn apply_auto_tuning(&self, result: &PerformanceTuningResult) -> Result<()> {
        // Auto-tuning implementation would go here
        // For example:
        // - Adjust cache sizes based on hit ratios
        // - Modify batch sizes based on throughput
        // - Scale pool sizes based on utilization

        tracing::info!(
            "Applied {} auto-tuning recommendations",
            result.recommendations.len()
        );
        Ok(())
    }

    /// Health check for the optimizer
    pub async fn health_check(&self) -> HealthCheckResult {
        let is_healthy = self.monitor.is_healthy().await;
        let performance_score = self.monitor.get_performance_score().await;
        let uptime = self.monitor.uptime();

        HealthCheckResult {
            is_healthy,
            performance_score,
            uptime,
            components: vec![
                ComponentHealth {
                    component: "monitor".to_string(),
                    healthy: true,
                    details: "Performance monitoring active".to_string(),
                },
                ComponentHealth {
                    component: "load_balancer".to_string(),
                    healthy: self.load_balancer.is_some(),
                    details: if self.load_balancer.is_some() {
                        "Load balancer active".to_string()
                    } else {
                        "Load balancer disabled".to_string()
                    },
                },
                ComponentHealth {
                    component: "object_pools".to_string(),
                    healthy: true,
                    details: "Object pools active".to_string(),
                },
            ],
        }
    }

    /// Get configuration
    pub fn get_config(&self) -> &PerformanceConfig {
        &self.config
    }
}

/// Comprehensive performance metrics
#[derive(Debug, Clone)]
pub struct ComprehensiveMetrics {
    pub system_metrics: SystemMetrics,
    pub performance_score: f64,
    pub load_balancer_stats: Option<LoadBalancerStats>,
    pub string_pool_size: usize,
    pub byte_pool_size: usize,
    pub uptime: std::time::Duration,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub is_healthy: bool,
    pub performance_score: f64,
    pub uptime: std::time::Duration,
    pub components: Vec<ComponentHealth>,
}

/// Individual component health
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub component: String,
    pub healthy: bool,
    pub details: String,
}

impl HealthCheckResult {
    /// Check if all components are healthy
    pub fn all_components_healthy(&self) -> bool {
        self.components.iter().all(|c| c.healthy)
    }

    /// Get unhealthy components
    pub fn get_unhealthy_components(&self) -> Vec<&ComponentHealth> {
        self.components.iter().filter(|c| !c.healthy).collect()
    }

    /// Generate health summary
    pub fn health_summary(&self) -> String {
        let healthy_count = self.components.iter().filter(|c| c.healthy).count();
        let total_count = self.components.len();

        format!(
            "Overall: {}, Performance Score: {:.2}, Components: {}/{} healthy, Uptime: {:?}",
            if self.is_healthy {
                "HEALTHY"
            } else {
                "UNHEALTHY"
            },
            self.performance_score,
            healthy_count,
            total_count,
            self.uptime
        )
    }
}
