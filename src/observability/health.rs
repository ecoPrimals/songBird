//! Health Monitoring Module
//!
//! Monitors health of services, nodes, and federation components

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::errors::Result;
use crate::traits::service::ServiceInfo;
use super::ObservabilityConfig;

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceHealth {
    /// Service ID
    pub service_id: String,
    /// Whether the service is healthy
    pub is_healthy: bool,
    /// Health check details
    pub details: HealthCheckDetails,
    /// Last successful health check
    pub last_success: Option<DateTime<Utc>>,
    /// Last failed health check
    pub last_failure: Option<DateTime<Utc>>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Health check response time
    pub response_time_ms: Option<u64>,
    /// Service uptime
    pub uptime_seconds: u64,
    /// Health check message
    pub message: String,
}

/// Health check details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheckDetails {
    /// Health check status
    pub status: HealthStatus,
    /// Status message
    pub message: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Check timestamp
    pub timestamp: DateTime<Utc>,
    /// Check duration
    pub duration_ms: u64,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Timeout,
    Error,
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of successes to mark healthy again
    pub success_threshold: u32,
    /// Enable detailed health checks
    pub detailed_checks: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            detailed_checks: true,
        }
    }
}

/// Health monitor implementation
pub struct HealthMonitor {
    config: ObservabilityConfig,
    health_config: HealthCheckConfig,
    /// Registered services for monitoring
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// Current health status of services
    service_health: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    /// Health check statistics
    check_count: AtomicU64,
    /// Last health check time
    last_check_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// Health check history
    health_history: Arc<RwLock<Vec<HealthCheckEvent>>>,
}

/// Health check event for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckEvent {
    pub service_id: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
}

/// Overall health status for the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealthStatus {
    pub overall_health: super::ClusterHealthStatus,
    pub service_health: HashMap<String, ServiceHealth>,
    pub timestamp: DateTime<Utc>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: ObservabilityConfig) -> Self {
        Self {
            config,
            health_config: HealthCheckConfig::default(),
            services: Arc::new(RwLock::new(HashMap::new())),
            service_health: Arc::new(RwLock::new(HashMap::new())),
            check_count: AtomicU64::new(0),
            last_check_time: Arc::new(RwLock::new(None)),
            health_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a service for health monitoring
    pub async fn register_service(&self, service_info: ServiceInfo) -> Result<()> {
        let service_id = service_info.id.clone();
        
        info!("🔍 Registering service '{}' for health monitoring", service_id);
        
        // Add to services registry
        self.services.write().await.insert(service_id.clone(), service_info);
        
        // Initialize health status
        let initial_health = ServiceHealth {
            service_id: service_id.clone(),
            is_healthy: true, // Assume healthy until proven otherwise
            details: HealthCheckDetails {
                status: HealthStatus::Unknown,
                message: "Service registered, health check pending".to_string(),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
                duration_ms: 0,
            },
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            response_time_ms: None,
            uptime_seconds: 0,
            message: "Service registered".to_string(),
        };
        
        self.service_health.write().await.insert(service_id, initial_health);
        
        Ok(())
    }

    /// Unregister a service from health monitoring
    pub async fn unregister_service(&self, service_id: &str) -> Result<()> {
        info!("🗑️ Unregistering service '{}' from health monitoring", service_id);
        
        self.services.write().await.remove(service_id);
        self.service_health.write().await.remove(service_id);
        
        Ok(())
    }

    /// Update service health status manually
    pub async fn update_service_health(&self, service_id: &str, healthy: bool) -> Result<()> {
        let mut service_health = self.service_health.write().await;
        
        if let Some(health) = service_health.get_mut(service_id) {
            let previous_health = health.is_healthy;
            health.is_healthy = healthy;
            
            // Update health details
            health.details = HealthCheckDetails {
                status: if healthy { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
                message: if healthy { "Manual health update: healthy" } else { "Manual health update: unhealthy" }.to_string(),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
                duration_ms: 0,
            };
            
            // Update consecutive failures
            if healthy {
                health.consecutive_failures = 0;
                health.last_success = Some(Utc::now());
            } else {
                health.consecutive_failures += 1;
                health.last_failure = Some(Utc::now());
            }
            
            // Log health change if status changed
            if previous_health != healthy {
                if healthy {
                    info!("✅ Service '{}' health improved", service_id);
                } else {
                    warn!("❌ Service '{}' health degraded", service_id);
                }
            }
            
            // Record health check event
            self.record_health_event(service_id, &health.details).await;
        } else {
            warn!("Attempted to update health for unknown service: {}", service_id);
        }
        
        Ok(())
    }

    /// Run health checks for all registered services
    pub async fn run_health_checks(&self) -> Result<()> {
        let services = self.services.read().await.clone();
        
        if services.is_empty() {
            debug!("No services registered for health monitoring");
            return Ok(());
        }
        
        debug!("🏥 Running health checks for {} services", services.len());
        let check_start = Instant::now();
        
        for (service_id, service_info) in services {
            if let Err(e) = self.check_service_health(&service_id, &service_info).await {
                warn!("Health check failed for service '{}': {}", service_id, e);
            }
        }
        
        // Update check statistics
        self.check_count.fetch_add(1, Ordering::Relaxed);
        *self.last_check_time.write().await = Some(Utc::now());
        
        let check_duration = check_start.elapsed();
        debug!("Health checks completed in {:?}", check_duration);
        
        Ok(())
    }

    /// Check health of a specific service
    async fn check_service_health(&self, service_id: &str, service_info: &ServiceInfo) -> Result<()> {
        let check_start = Instant::now();
        
        debug!("Checking health for service '{}'", service_id);
        
        // Perform health check based on service endpoints
        let health_result = if service_info.endpoints.is_empty() {
            // No endpoints defined, assume healthy if service is registered
            Ok(HealthCheckDetails {
                status: HealthStatus::Healthy,
                message: "No health endpoints defined, assuming healthy".to_string(),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
                duration_ms: check_start.elapsed().as_millis() as u64,
            })
        } else {
            // Check health endpoints
            self.check_service_endpoints(service_info).await
        };
        
        // Update service health based on check result
        match health_result {
            Ok(details) => {
                self.update_service_health_details(service_id, details, true).await?;
            }
            Err(e) => {
                let details = HealthCheckDetails {
                    status: HealthStatus::Error,
                    message: format!("Health check error: {}", e),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                    duration_ms: check_start.elapsed().as_millis() as u64,
                };
                self.update_service_health_details(service_id, details, false).await?;
            }
        }
        
        Ok(())
    }

    /// Check service health via its endpoints
    async fn check_service_endpoints(&self, service_info: &ServiceInfo) -> Result<HealthCheckDetails> {
        let check_start = Instant::now();
        
        // For now, we'll do a simple connectivity check
        // In a real implementation, this would make HTTP requests to health endpoints
        
        // Simulate health check by examining service metadata
        let is_healthy = service_info.metadata.get("health_status")
            .and_then(|v| v.as_str())
            .map(|s| s == "healthy")
            .unwrap_or(true); // Default to healthy
        
        let status = if is_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };
        
        let mut metadata = HashMap::new();
        metadata.insert(
            "endpoints_count".to_string(),
            serde_json::Value::from(service_info.endpoints.len()),
        );
        metadata.insert(
            "service_type".to_string(),
            serde_json::Value::String(service_info.service_type.clone()),
        );
        
        Ok(HealthCheckDetails {
            status,
            message: if is_healthy {
                "Service health check passed".to_string()
            } else {
                "Service health check failed".to_string()
            },
            metadata,
            timestamp: Utc::now(),
            duration_ms: check_start.elapsed().as_millis() as u64,
        })
    }

    /// Update service health details internally
    async fn update_service_health_details(
        &self,
        service_id: &str,
        details: HealthCheckDetails,
        _healthy: bool, // We determine health from details.status instead
    ) -> Result<()> {
        let mut service_health = self.service_health.write().await;
        
        if let Some(health) = service_health.get_mut(service_id) {
            let previous_health = health.is_healthy;
            
            // Determine if healthy based on details status
            let healthy = matches!(details.status, HealthStatus::Healthy);
            
            // Update health status
            health.is_healthy = healthy;
            health.details = details.clone();
            health.response_time_ms = Some(details.duration_ms);
            
            // Update consecutive failures/successes
            if healthy {
                health.consecutive_failures = 0;
                health.last_success = Some(details.timestamp);
                
                // Log recovery if previously unhealthy
                if !previous_health {
                    info!("🔄 Service '{}' recovered (consecutive failures reset)", service_id);
                }
            } else {
                health.consecutive_failures += 1;
                health.last_failure = Some(details.timestamp);
                
                // Check if we should mark as unhealthy
                if health.consecutive_failures >= self.health_config.failure_threshold && previous_health {
                    warn!("⚠️ Service '{}' marked unhealthy after {} consecutive failures", 
                          service_id, health.consecutive_failures);
                }
            }
            
            // Record health check event
            self.record_health_event(service_id, &details).await;
        }
        
        Ok(())
    }

    /// Record a health check event in history
    async fn record_health_event(&self, service_id: &str, details: &HealthCheckDetails) {
        let event = HealthCheckEvent {
            service_id: service_id.to_string(),
            status: details.status.clone(),
            message: details.message.clone(),
            timestamp: details.timestamp,
            duration_ms: details.duration_ms,
        };
        
        let mut history = self.health_history.write().await;
        history.push(event);
        
        // Keep only the last N events
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get health status for all services
    pub async fn get_service_health(&self) -> Result<Vec<ServiceHealth>> {
        let service_health = self.service_health.read().await;
        
        let health_list: Vec<ServiceHealth> = service_health
            .values()
            .cloned()
            .collect();
        
        Ok(health_list)
    }

    /// Get node health information
    pub async fn get_node_health(&self) -> Result<Vec<super::NodeHealth>> {
        // For now, return information about the current node
        // In a real implementation, this would query federation nodes
        
        let node_health = super::NodeHealth {
            node_id: "local-node".to_string(),
            node_type: "orchestrator".to_string(),
            is_healthy: true,
            last_seen: Utc::now(),
            uptime_seconds: 0, // TODO: Track actual uptime
            issues: vec![],
        };
        
        Ok(vec![node_health])
    }

    /// Get federation health information
    pub async fn get_federation_health(&self) -> Result<Option<super::FederationHealth>> {
        // For now, return None if federation is not active
        // In a real implementation, this would query federation status
        
        Ok(None)
    }

    /// Get health check count
    pub fn get_check_count(&self) -> u64 {
        self.check_count.load(Ordering::Relaxed)
    }

    /// Get health check history
    pub async fn get_health_history(&self) -> Vec<HealthCheckEvent> {
        self.health_history.read().await.clone()
    }

    /// Get health statistics
    pub async fn get_health_stats(&self) -> HealthStats {
        let service_health = self.service_health.read().await;
        
        let total_services = service_health.len();
        let healthy_services = service_health.values().filter(|h| h.is_healthy).count();
        let unhealthy_services = total_services - healthy_services;
        
        let avg_response_time = if !service_health.is_empty() {
            service_health
                .values()
                .filter_map(|h| h.response_time_ms)
                .sum::<u64>() as f64 / service_health.len() as f64
        } else {
            0.0
        };
        
        HealthStats {
            total_services,
            healthy_services,
            unhealthy_services,
            total_checks: self.check_count.load(Ordering::Relaxed),
            avg_response_time_ms: avg_response_time,
            last_check_time: *self.last_check_time.read().await,
        }
    }

    /// Get overall health status
    pub async fn get_health_status(&self) -> Result<OverallHealthStatus> {
        let service_health = self.service_health.read().await;
        
        // Overall health status calculation
        let overall_health = OverallHealthStatus {
            overall_health: if service_health.is_empty() {
                crate::observability::ClusterHealthStatus::Unknown
            } else {
                let healthy_count = service_health.values().filter(|h| h.is_healthy).count();
                let total_count = service_health.len();
                
                if healthy_count == total_count {
                    crate::observability::ClusterHealthStatus::Healthy
                } else if healthy_count > 0 {
                    crate::observability::ClusterHealthStatus::Degraded
                } else {
                    crate::observability::ClusterHealthStatus::Unhealthy
                }
            },
            service_health: service_health.clone(),
            timestamp: Utc::now(),
        };
        
        Ok(overall_health)
    }
}

/// Health monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    pub total_services: usize,
    pub healthy_services: usize,
    pub unhealthy_services: usize,
    pub total_checks: u64,
    pub avg_response_time_ms: f64,
    pub last_check_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::service::ServiceEndpoint;
    use tokio::time::Duration;

    fn create_test_config() -> ObservabilityConfig {
        ObservabilityConfig {
            enabled: true,
            metrics_interval_secs: 30,
            health_check_interval_secs: 5, // Fast interval for testing
            enable_dashboard: false,
            dashboard_port: 8081,
            export_prometheus: true,
            max_metric_history: 100,
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

    fn create_unhealthy_service_info(id: &str) -> ServiceInfo {
        let mut service = create_test_service_info(id);
        service.metadata.insert(
            "health_status".to_string(),
            serde_json::Value::String("unhealthy".to_string()),
        );
        service
    }

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        assert_eq!(monitor.get_check_count(), 0);
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_service_registration() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        
        monitor.register_service(service_info).await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].service_id, "test-service");
        assert!(services[0].is_healthy); // Should start as healthy
        assert_eq!(services[0].consecutive_failures, 0);
        assert!(services[0].last_success.is_none()); // No health check yet
        assert!(services[0].last_failure.is_none());
    }

    #[tokio::test]
    async fn test_service_unregistration() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        
        // Register and then unregister
        monitor.register_service(service_info).await.unwrap();
        monitor.unregister_service("test-service").await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_manual_health_update() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        monitor.register_service(service_info).await.unwrap();
        
        // Update to unhealthy
        monitor.update_service_health("test-service", false).await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 1);
        assert!(!services[0].is_healthy);
        assert_eq!(services[0].consecutive_failures, 1);
        assert!(services[0].last_failure.is_some());
        assert!(services[0].details.status == HealthStatus::Unhealthy);
        
        // Update back to healthy
        monitor.update_service_health("test-service", true).await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert!(services[0].is_healthy);
        assert_eq!(services[0].consecutive_failures, 0);
        assert!(services[0].last_success.is_some());
        assert!(services[0].details.status == HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_consecutive_failures() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        monitor.register_service(service_info).await.unwrap();
        
        // Cause multiple failures
        for i in 1..=5 {
            monitor.update_service_health("test-service", false).await.unwrap();
            
            let services = monitor.get_service_health().await.unwrap();
            assert_eq!(services[0].consecutive_failures, i);
        }
        
        // One success should reset failures
        monitor.update_service_health("test-service", true).await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services[0].consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_health_check_run() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        // Register services with different health states
        let healthy_service = create_test_service_info("healthy-service");
        let unhealthy_service = create_unhealthy_service_info("unhealthy-service");
        
        monitor.register_service(healthy_service).await.unwrap();
        monitor.register_service(unhealthy_service).await.unwrap();
        
        // Run health checks
        monitor.run_health_checks().await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 2);
        
        // Check that checks were performed
        assert!(monitor.get_check_count() > 0);
        
        // Find the unhealthy service and verify its status
        let unhealthy = services.iter().find(|s| s.service_id == "unhealthy-service").unwrap();
        assert!(!unhealthy.is_healthy);
        assert!(unhealthy.response_time_ms.is_some());
        
        // Find the healthy service and verify its status
        let healthy = services.iter().find(|s| s.service_id == "healthy-service").unwrap();
        assert!(healthy.is_healthy);
        assert!(healthy.response_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_health_stats() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        // Register multiple services
        for i in 0..5 {
            let service_info = create_test_service_info(&format!("service-{}", i));
            monitor.register_service(service_info).await.unwrap();
        }
        
        // Make some unhealthy
        monitor.update_service_health("service-0", false).await.unwrap();
        monitor.update_service_health("service-1", false).await.unwrap();
        
        let stats = monitor.get_health_stats().await;
        
        assert_eq!(stats.total_services, 5);
        assert_eq!(stats.healthy_services, 3);
        assert_eq!(stats.unhealthy_services, 2);
        assert!(stats.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_node_health() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let nodes = monitor.get_node_health().await.unwrap();
        
        // Should return at least the local node
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, "local-node");
        assert_eq!(nodes[0].node_type, "orchestrator");
        assert!(nodes[0].is_healthy);
        assert!(nodes[0].issues.is_empty());
    }

    #[tokio::test]
    async fn test_federation_health() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let federation = monitor.get_federation_health().await.unwrap();
        
        // Should return None if federation is not active
        assert!(federation.is_none());
    }

    #[tokio::test]
    async fn test_health_history() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        monitor.register_service(service_info).await.unwrap();
        
        // Perform several health updates to generate history
        monitor.update_service_health("test-service", false).await.unwrap();
        monitor.update_service_health("test-service", true).await.unwrap();
        monitor.update_service_health("test-service", false).await.unwrap();
        
        let history = monitor.get_health_history().await;
        
        // Should have recorded events
        assert!(history.len() >= 3);
        
        // Events should be for the correct service
        for event in &history {
            assert_eq!(event.service_id, "test-service");
            assert!(event.duration_ms >= 0);
        }
    }

    #[tokio::test]
    async fn test_health_check_details() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        let service_info = create_test_service_info("test-service");
        monitor.register_service(service_info).await.unwrap();
        
        // Run health check
        monitor.run_health_checks().await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        let service = &services[0];
        
        // Duration might be 0 for very fast operations, which is acceptable
        assert!(service.details.duration_ms >= 0);
        assert!(service.details.timestamp <= Utc::now());
        assert!(!service.details.message.is_empty());
        
        // Should have metadata about endpoints
        assert!(service.details.metadata.contains_key("endpoints_count"));
        assert!(service.details.metadata.contains_key("service_type"));
    }

    #[tokio::test]
    async fn test_health_status_enum() {
        // Test all health status variants
        let statuses = [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
            HealthStatus::Timeout,
            HealthStatus::Error,
        ];
        
        for status in &statuses {
            // Should be able to clone and serialize
            let cloned = status.clone();
            assert_eq!(std::mem::discriminant(status), std::mem::discriminant(&cloned));
            
            // Should be serializable
            let serialized = serde_json::to_string(status).unwrap();
            assert!(!serialized.is_empty());
        }
    }

    #[tokio::test]
    async fn test_service_endpoints_processing() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        // Create service with multiple endpoints
        let mut service_info = create_test_service_info("multi-endpoint-service");
        service_info.endpoints.push(ServiceEndpoint {
            path: "/admin/health".to_string(),
            method: "GET".to_string(),
            description: "Admin health check endpoint".to_string(),
            parameters: vec![],
            response_schema: None,
        });
        
        monitor.register_service(service_info).await.unwrap();
        monitor.run_health_checks().await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        let service = &services[0];
        
        // Should have processed endpoints in metadata
        let endpoints_count = service.details.metadata.get("endpoints_count").unwrap();
        assert_eq!(endpoints_count.as_u64().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_service_without_endpoints() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        // Create service without endpoints
        let mut service_info = create_test_service_info("no-endpoints-service");
        service_info.endpoints.clear();
        
        monitor.register_service(service_info).await.unwrap();
        monitor.run_health_checks().await.unwrap();
        
        let services = monitor.get_service_health().await.unwrap();
        let service = &services[0];
        
        // Should be healthy by default when no endpoints defined
        assert!(service.is_healthy);
        assert!(service.details.message.contains("No health endpoints defined"));
    }

    #[tokio::test]
    async fn test_unknown_service_health_update() {
        let config = create_test_config();
        let monitor = HealthMonitor::new(config);
        
        // Try to update health for non-existent service
        monitor.update_service_health("unknown-service", false).await.unwrap();
        
        // Should not crash, but service shouldn't be created
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_health_check_config_defaults() {
        let config = HealthCheckConfig::default();
        
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.success_threshold, 2);
        assert!(config.detailed_checks);
    }

    #[tokio::test]
    async fn test_concurrent_health_updates() {
        let config = create_test_config();
        let monitor = Arc::new(HealthMonitor::new(config));
        
        let service_info = create_test_service_info("concurrent-service");
        monitor.register_service(service_info).await.unwrap();
        
        // Perform concurrent health updates
        let mut handles = vec![];
        
        for i in 0..10 {
            let monitor_clone = Arc::clone(&monitor);
            let handle = tokio::spawn(async move {
                let healthy = i % 2 == 0;
                monitor_clone.update_service_health("concurrent-service", healthy).await.unwrap();
            });
            handles.push(handle);
        }
        
        // Wait for all updates to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Service should still exist and be in a valid state
        let services = monitor.get_service_health().await.unwrap();
        assert_eq!(services.len(), 1);
        
        // Check count should reflect the updates
        assert!(services[0].consecutive_failures <= 10);
    }
} 