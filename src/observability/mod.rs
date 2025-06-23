//! Observability Module
//!
//! Built-in observability for Songbird Orchestrator including:
//! - System metrics collection (CPU, memory, disk)
//! - Service health monitoring
//! - Performance metrics tracking
//! - Optional simple web dashboard

pub mod metrics;
pub mod health;
pub mod dashboard;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, info, warn};
use std::collections::HashMap;

pub use metrics::*;
pub use health::*;
pub use dashboard::*;

use crate::config::ObservabilityConfig;
use crate::errors::{Result, SongbirdError};
use crate::traits::service::ServiceInfo;

/// Cluster status overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    /// Node health information
    pub nodes: Vec<NodeHealth>,
    /// Service health information
    pub services: Vec<ServiceHealth>,
    /// Federation health (if enabled)
    pub federation: Option<FederationHealth>,
    /// Current system and application metrics
    pub metrics: MetricsSnapshot,
    /// Overall cluster status
    pub overall_status: ClusterHealthStatus,
    /// Status timestamp
    pub timestamp: DateTime<Utc>,
}

impl ClusterStatus {
    /// Count healthy nodes
    pub fn healthy_nodes(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_healthy).count()
    }

    /// Count total nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Count running services
    pub fn running_services(&self) -> usize {
        self.services.iter().filter(|s| s.is_healthy).count()
    }

    /// Count total services
    pub fn total_services(&self) -> usize {
        self.services.len()
    }
}

/// Overall cluster health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Node health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: String,
    pub node_type: String,
    pub is_healthy: bool,
    pub last_seen: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub issues: Vec<String>,
}

/// Federation health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationHealth {
    pub connected_nodes: u32,
    pub total_discovered_nodes: u32,
    pub federation_active: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub sync_errors: u32,
}

/// Main observability engine
pub struct ObservabilityEngine {
    config: ObservabilityConfig,
    metrics_collector: Arc<MetricsCollector>,
    health_monitor: Arc<HealthMonitor>,
    dashboard: Option<Arc<SimpleDashboard>>,
    running: Arc<AtomicBool>,
    event_sender: broadcast::Sender<ObservabilityEvent>,
    start_time: Instant,
}

/// Observability events - Updated to match demo expectations
#[derive(Debug, Clone)]
pub enum ObservabilityEvent {
    MetricsCollected { 
        timestamp: DateTime<Utc>,
        duration_ms: u64,
    },
    HealthCheckCompleted {
        service_id: String,
        is_healthy: bool,
        response_time_ms: u64,
    },
    DashboardStarted {
        port: u16,
    },
    SystemAlert {
        message: String,
        severity: String,
    },
}

/// Alert levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

impl ObservabilityEngine {
    /// Create a new observability engine
    pub fn new(config: ObservabilityConfig) -> Result<Self> {
        let metrics_collector = Arc::new(MetricsCollector::new(config.clone())?);
        let health_monitor = Arc::new(HealthMonitor::new(config.clone()));
        let (event_sender, _) = broadcast::channel(1000);
        
        let dashboard = if config.enable_dashboard {
            Some(Arc::new(SimpleDashboard::new(
                config.dashboard_port,
                Arc::clone(&metrics_collector),
                Arc::clone(&health_monitor),
            )))
        } else {
            None
        };

        Ok(Self {
            config,
            metrics_collector,
            health_monitor,
            dashboard,
            running: Arc::new(AtomicBool::new(false)),
            event_sender,
            start_time: Instant::now(),
        })
    }

    /// Start the observability engine
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Observability disabled by configuration");
            return Ok(());
        }

        if self.running.load(Ordering::Relaxed) {
            warn!("Observability engine already running");
            return Ok(());
        }

        info!("🔍 Starting Songbird Observability Engine");
        self.running.store(true, Ordering::Relaxed);

        // Start metrics collection
        if self.config.enable_system_metrics || self.config.enable_service_metrics {
            self.start_metrics_collection().await?;
        }

        // Start health monitoring
        self.start_health_monitoring().await?;

        // Start dashboard if enabled
        if let Some(dashboard) = &self.dashboard {
            dashboard.start().await?;
            
            // Send dashboard started event
            let _ = self.event_sender.send(ObservabilityEvent::DashboardStarted {
                port: self.config.dashboard_port,
            });
        }

        info!("✅ Observability engine started successfully");
        Ok(())
    }

    /// Stop the observability engine
    pub async fn stop(&self) -> Result<()> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("🛑 Stopping Songbird Observability Engine");
        self.running.store(false, Ordering::Relaxed);

        // Stop dashboard if running
        if let Some(dashboard) = &self.dashboard {
            dashboard.stop().await?;
        }

        info!("✅ Observability engine stopped successfully");
        Ok(())
    }

    /// Get current cluster status
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let nodes = self.health_monitor.get_node_health().await?;
        let services = self.health_monitor.get_service_health().await?;
        let federation = self.health_monitor.get_federation_health().await?;
        let metrics = self.metrics_collector.get_current_snapshot().await?;

        // Determine overall cluster health
        let overall_status = self.calculate_overall_health(&nodes, &services);

        Ok(ClusterStatus {
            nodes,
            services,
            federation,
            metrics,
            overall_status,
            timestamp: Utc::now(),
        })
    }

    /// Get metrics collector
    pub fn metrics_collector(&self) -> &Arc<MetricsCollector> {
        &self.metrics_collector
    }

    /// Get health monitor
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }

    /// Subscribe to observability events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ObservabilityEvent> {
        self.event_sender.subscribe()
    }

    /// Export metrics in Prometheus format (if enabled)
    pub async fn export_prometheus(&self) -> Result<String> {
        if !self.config.export_prometheus {
            return Err(SongbirdError::Configuration {
                field: "export_prometheus".to_string(),
                message: "Prometheus export is disabled".to_string(),
            });
        }

        self.metrics_collector.export_prometheus().await
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Register a service for monitoring
    pub async fn register_service(&self, service_info: &ServiceInfo) -> Result<()> {
        self.health_monitor.register_service(service_info.clone()).await
    }

    /// Unregister a service from monitoring
    pub async fn unregister_service(&self, service_id: &str) -> Result<()> {
        self.health_monitor.unregister_service(service_id).await
    }

    /// Update service health status
    pub async fn update_service_health(&self, service_id: &str, healthy: bool) -> Result<()> {
        self.health_monitor.update_service_health(service_id, healthy).await?;
        
        // Broadcast health change event
        let _ = self.event_sender.send(ObservabilityEvent::HealthCheckCompleted {
            service_id: service_id.to_string(),
            is_healthy: healthy,
            response_time_ms: 0, // TODO: Get actual response time
        });

        Ok(())
    }

    /// Start background metrics collection
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let running = Arc::clone(&self.running);
        let event_sender = self.event_sender.clone();
        let interval_secs = self.config.metrics_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            info!("📊 Starting metrics collection (interval: {}s)", interval_secs);
            
            while running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                let collection_start = Instant::now();
                match metrics_collector.collect_all_metrics().await {
                    Ok(metrics) => {
                        let duration_ms = collection_start.elapsed().as_millis() as u64;
                        debug!("Collected metrics: CPU={:.1}%, Memory={:.1}%", 
                               metrics.system.cpu_usage, 
                               metrics.system.memory_usage * 100.0);
                        
                        // Broadcast metrics event
                        let _ = event_sender.send(ObservabilityEvent::MetricsCollected {
                            timestamp: Utc::now(),
                            duration_ms,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to collect metrics: {}", e);
                        
                        // Send alert for metrics collection failure
                        let _ = event_sender.send(ObservabilityEvent::SystemAlert {
                            message: format!("Metrics collection failed: {}", e),
                            severity: "warning".to_string(),
                        });
                    }
                }
            }
            
            debug!("Metrics collection stopped");
        });

        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) -> Result<()> {
        let health_monitor = Arc::clone(&self.health_monitor);
        let running = Arc::clone(&self.running);
        let event_sender = self.event_sender.clone();
        let interval_secs = self.config.health_check_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            info!("🏥 Starting health monitoring (interval: {}s)", interval_secs);
            
            while running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                if let Err(e) = health_monitor.run_health_checks().await {
                    warn!("Health check failed: {}", e);
                    
                    // Send alert for health check failures
                    let _ = event_sender.send(ObservabilityEvent::SystemAlert {
                        message: format!("Health check failed: {}", e),
                        severity: "warning".to_string(),
                    });
                }
            }
            
            debug!("Health monitoring stopped");
        });

        Ok(())
    }

    /// Calculate overall cluster health
    fn calculate_overall_health(&self, nodes: &[NodeHealth], services: &[ServiceHealth]) -> ClusterHealthStatus {
        let healthy_nodes = nodes.iter().filter(|n| n.is_healthy).count();
        let healthy_services = services.iter().filter(|s| s.is_healthy).count();
        
        let node_health_ratio = if nodes.is_empty() { 
            1.0 
        } else { 
            healthy_nodes as f64 / nodes.len() as f64 
        };
        
        let service_health_ratio = if services.is_empty() { 
            1.0 
        } else { 
            healthy_services as f64 / services.len() as f64 
        };

        // Overall health is the minimum of node and service health
        let overall_ratio = node_health_ratio.min(service_health_ratio);

        if overall_ratio >= 0.9 {
            ClusterHealthStatus::Healthy
        } else if overall_ratio >= 0.7 {
            ClusterHealthStatus::Degraded
        } else if overall_ratio > 0.0 {
            ClusterHealthStatus::Unhealthy
        } else {
            ClusterHealthStatus::Unknown
        }
    }

    /// Get observability statistics
    pub async fn get_stats(&self) -> ObservabilityStats {
        ObservabilityStats {
            uptime_seconds: self.uptime_seconds(),
            metrics_collected: self.metrics_collector.get_collection_count(),
            health_checks_performed: self.health_monitor.get_check_count(),
            dashboard_enabled: self.dashboard.is_some(),
            prometheus_enabled: self.config.export_prometheus,
            last_collection_time: self.metrics_collector.last_collection_time(),
        }
    }

    /// Get current metrics snapshot (alias for compatibility)
    pub async fn get_metrics(&self) -> Result<MetricsSnapshot> {
        self.metrics_collector.get_current_metrics().await
    }

    /// Get current health status (alias for compatibility)
    pub async fn get_health_status(&self) -> Result<health::OverallHealthStatus> {
        self.health_monitor.get_health_status().await
    }

    /// Get dashboard data for display
    pub async fn get_dashboard_data(&self) -> Result<Vec<serde_json::Value>> {
        let mut data = Vec::new();
        
        // Add metrics data
        if let Ok(metrics) = self.get_metrics().await {
            data.push(serde_json::json!({
                "type": "metrics",
                "data": metrics
            }));
        }

        // Add health data
        if let Ok(health) = self.get_health_status().await {
            data.push(serde_json::json!({
                "type": "health",
                "data": health
            }));
        }

        // Add cluster status
        if let Ok(cluster) = self.get_cluster_status().await {
            data.push(serde_json::json!({
                "type": "cluster",
                "data": cluster
            }));
        }

        Ok(data)
    }
}

/// Observability statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityStats {
    pub uptime_seconds: u64,
    pub metrics_collected: u64,
    pub health_checks_performed: u64,
    pub dashboard_enabled: bool,
    pub prometheus_enabled: bool,
    pub last_collection_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::service::ServiceEndpoint;
    use tokio::time::timeout;

    fn create_test_config() -> ObservabilityConfig {
        ObservabilityConfig {
            enabled: true,
            metrics_interval_secs: 1, // Fast interval for testing
            health_check_interval_secs: 1,
            enable_dashboard: false, // Disable dashboard for tests
            dashboard_port: 8082,
            export_prometheus: true,
            max_metric_history: 10,
            enable_system_metrics: true,
            enable_service_metrics: true,
        }
    }

    fn create_test_service_info(id: &str) -> ServiceInfo {
        ServiceInfo {
            id: id.to_string(),
            name: format!("Test Service {}", id),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: format!("Test service {}", id),
            endpoints: vec![ServiceEndpoint {
                path: "/health".to_string(),
                method: "GET".to_string(),
                description: "Health check endpoint".to_string(),
                parameters: vec![],
                response_schema: None,
            }],
            capabilities: vec!["health-check".to_string()],
            tags: {
                let mut tags = HashMap::new();
                tags.insert("environment".to_string(), "test".to_string());
                tags
            },
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("environment".to_string(), serde_json::Value::String("test".to_string()));
                metadata
            },
        }
    }

    #[tokio::test]
    async fn test_observability_engine_creation() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        assert!(!engine.running.load(Ordering::Relaxed));
        assert!(engine.dashboard.is_none()); // Dashboard disabled in test config
        assert_eq!(engine.uptime_seconds(), 0); // Should be 0 or 1 since just created
    }

    #[tokio::test]
    async fn test_observability_engine_start_stop() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Test start
        engine.start().await.unwrap();
        assert!(engine.running.load(Ordering::Relaxed));
        
        // Test stop
        engine.stop().await.unwrap();
        assert!(!engine.running.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_disabled_observability() {
        let mut config = create_test_config();
        config.enabled = false;
        
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Should start but not actually run anything
        engine.start().await.unwrap();
        assert!(!engine.running.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_service_registration() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let service_info = create_test_service_info("test-service");
        
        // Register service
        engine.register_service(&service_info).await.unwrap();
        
        // Check that service is registered in health monitor
        let services = engine.health_monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service_id, "test-service");
        assert!(services[0].is_healthy);
    }

    #[tokio::test]
    async fn test_service_unregistration() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let service_info = create_test_service_info("test-service");
        
        // Register and then unregister service
        engine.register_service(&service_info).await.unwrap();
        engine.unregister_service("test-service").await.unwrap();
        
        // Check that service is no longer registered
        let services = engine.health_monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_service_health_update() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let service_info = create_test_service_info("test-service");
        
        // Register service
        engine.register_service(&service_info).await.unwrap();
        
        // Update health to unhealthy
        engine.update_service_health("test-service", false).await.unwrap();
        
        // Check health status
        let services = engine.health_monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 1);
        assert!(!services[0].is_healthy);
        
        // Update back to healthy
        engine.update_service_health("test-service", true).await.unwrap();
        
        let services = engine.health_monitor.get_service_health().await.unwrap();
        assert!(services[0].is_healthy);
    }

    #[tokio::test]
    async fn test_cluster_status() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let service_info = create_test_service_info("test-service");
        engine.register_service(&service_info).await.unwrap();
        
        // Get cluster status
        let status = engine.get_cluster_status().await.unwrap();
        
        assert_eq!(status.services.len(), 1);
        assert_eq!(status.nodes.len(), 1); // Default local node
        assert!(matches!(status.overall_status, ClusterHealthStatus::Healthy));
        assert_eq!(status.total_services(), 1);
        assert_eq!(status.running_services(), 1);
    }

    #[tokio::test]
    async fn test_cluster_health_calculation() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Register multiple services
        for i in 0..10 {
            let service_info = create_test_service_info(&format!("service-{}", i));
            engine.register_service(&service_info).await.unwrap();
        }
        
        // All healthy (100%) - should be Healthy
        let status = engine.get_cluster_status().await.unwrap();
        assert!(matches!(status.overall_status, ClusterHealthStatus::Healthy));
        
        // Make 1 service unhealthy (90% healthy) - should still be Healthy
        engine.update_service_health("service-0", false).await.unwrap();
        
        let status = engine.get_cluster_status().await.unwrap();
        assert!(matches!(status.overall_status, ClusterHealthStatus::Healthy));
        
        // Make 2 more services unhealthy (70% healthy) - should be Degraded
        engine.update_service_health("service-1", false).await.unwrap();
        engine.update_service_health("service-2", false).await.unwrap();
        
        let status = engine.get_cluster_status().await.unwrap();
        assert!(matches!(status.overall_status, ClusterHealthStatus::Degraded));
        
        // Make 4 more services unhealthy (30% healthy) - should be Unhealthy
        engine.update_service_health("service-3", false).await.unwrap();
        engine.update_service_health("service-4", false).await.unwrap();
        engine.update_service_health("service-5", false).await.unwrap();
        engine.update_service_health("service-6", false).await.unwrap();
        
        let status = engine.get_cluster_status().await.unwrap();
        assert!(matches!(status.overall_status, ClusterHealthStatus::Unhealthy));
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Get initial metrics
        let metrics = engine.metrics_collector.get_current_snapshot().await.unwrap();
        
        // Verify basic metrics structure
        assert!(metrics.system.cpu_usage >= 0.0);
        assert!(metrics.system.memory_usage >= 0.0 && metrics.system.memory_usage <= 1.0);
        assert!(metrics.system.memory_total_bytes > 0);
        assert!(metrics.collection_duration_ms > 0);
        
        // Verify application metrics defaults
        assert_eq!(metrics.songbird.active_services, 0);
        assert_eq!(metrics.songbird.request_rate, 0.0);
        assert_eq!(metrics.songbird.error_rate, 0.0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Export Prometheus metrics
        let prometheus_output = engine.export_prometheus().await.unwrap();
        
        // Verify Prometheus format
        assert!(prometheus_output.contains("# HELP songbird_cpu_usage_percent"));
        assert!(prometheus_output.contains("# TYPE songbird_cpu_usage_percent gauge"));
        assert!(prometheus_output.contains("songbird_cpu_usage_percent"));
        assert!(prometheus_output.contains("songbird_memory_usage_ratio"));
        assert!(prometheus_output.contains("songbird_active_services"));
    }

    #[tokio::test]
    async fn test_prometheus_export_disabled() {
        let mut config = create_test_config();
        config.export_prometheus = false;
        
        let engine = ObservabilityEngine::new(config).unwrap();
        
        // Should return error when Prometheus export is disabled
        let result = engine.export_prometheus().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_observability_events() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let mut event_receiver = engine.subscribe_events();
        
        let service_info = create_test_service_info("test-service");
        engine.register_service(&service_info).await.unwrap();
        
        // Update service health to trigger event
        engine.update_service_health("test-service", false).await.unwrap();
        
        // Receive event
        let event = timeout(Duration::from_millis(100), event_receiver.recv()).await;
        assert!(event.is_ok());
        
        if let Ok(Ok(ObservabilityEvent::HealthCheckCompleted { service_id, is_healthy, .. })) = event {
            assert_eq!(service_id, "test-service");
            assert!(!is_healthy);
        } else {
            panic!("Expected HealthCheckCompleted event");
        }
    }

    #[tokio::test]
    async fn test_observability_stats() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let stats = engine.get_stats().await;
        
        assert_eq!(stats.uptime_seconds, 0); // Just created
        assert_eq!(stats.metrics_collected, 0); // No collection yet
        assert_eq!(stats.health_checks_performed, 0); // No checks yet
        assert!(!stats.dashboard_enabled); // Disabled in test config
        assert!(stats.prometheus_enabled);
        assert!(stats.last_collection_time.is_none());
    }

    #[tokio::test]
    async fn test_multiple_event_subscribers() {
        let config = create_test_config();
        let engine = ObservabilityEngine::new(config).unwrap();
        
        let mut receiver1 = engine.subscribe_events();
        let mut receiver2 = engine.subscribe_events();
        
        let service_info = create_test_service_info("test-service");
        engine.register_service(&service_info).await.unwrap();
        
        // Update service health to trigger event
        engine.update_service_health("test-service", false).await.unwrap();
        
        // Both receivers should get the event
        let event1 = timeout(Duration::from_millis(100), receiver1.recv()).await;
        let event2 = timeout(Duration::from_millis(100), receiver2.recv()).await;
        
        assert!(event1.is_ok());
        assert!(event2.is_ok());
    }
} 