// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![expect(clippy::unused_async, reason = "unused bindings/imports in this compilation unit")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::debug;

use super::{HealthStatus, NodeHealth, ServiceHealth};
type Result<T> = SongbirdResult<T>;

/// Health monitor for services and nodes
#[derive(Debug)]
pub struct HealthMonitor {
    services: Arc<RwLock<HashMap<String, ServiceHealth>>>,
    nodes: Arc<RwLock<HashMap<String, NodeHealth>>>,
}

impl HealthMonitor {
    /// Create new health monitor
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service for monitoring
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn register_service(&self, service_id: String) -> Result<()> {
        let health = ServiceHealth {
            service_id: service_id.clone(),
            status: HealthStatus::Unknown,
            last_check: Utc::now(),
            response_time_ms: 0,
            error_message: None,
        };

        self.services.write().unwrap_or_else(std::sync::PoisonError::into_inner).insert(service_id, health);
        Ok(())
    }

    /// Update service health status
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn update_service_health(
        &self,
        service_id: &str,
        status: HealthStatus,
    ) -> Result<()> {
        if let Some(health) = self.services.write().unwrap_or_else(std::sync::PoisonError::into_inner).get_mut(service_id) {
            health.status = status;
            health.last_check = Utc::now();
        }
        Ok(())
    }

    /// Get service health status
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_service_health(&self, service_id: &str) -> Result<Option<ServiceHealth>> {
        let services = self.services.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(services.get(service_id).cloned())
    }

    /// Get all service health statuses
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_all_service_health(&self) -> Result<Vec<ServiceHealth>> {
        let services = self.services.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(services.values().cloned().collect())
    }

    /// Run health checks for all registered services
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn run_health_checks(&self) -> Result<()> {
        debug!("Running health checks");
        // In a real implementation, this would check actual service endpoints
        Ok(())
    }

    /// Get health statistics
    pub async fn get_health_stats(&self) -> HealthStats {
        let services = self.services.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        let healthy_count =
            services.values().filter(|s| matches!(s.status, HealthStatus::Healthy)).count();

        HealthStats {
            total_services: services.len(),
            healthy_services: healthy_count,
            unhealthy_services: services.len() - healthy_count,
        }
    }

    /// Register a node for health monitoring
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn register_node(&self, node_id: String) -> Result<()> {
        let health = NodeHealth {
            node_id: node_id.clone(),
            status: HealthStatus::Unknown,
            last_heartbeat: Utc::now(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
        };
        self.nodes.write().unwrap_or_else(std::sync::PoisonError::into_inner).insert(node_id, health);
        Ok(())
    }

    /// Update node health from a heartbeat
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn update_node_health(
        &self,
        node_id: &str,
        status: HealthStatus,
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
    ) -> Result<()> {
        if let Some(node) = self.nodes.write().unwrap_or_else(std::sync::PoisonError::into_inner).get_mut(node_id) {
            node.status = status;
            node.last_heartbeat = Utc::now();
            node.cpu_usage = cpu_usage;
            node.memory_usage = memory_usage;
            node.disk_usage = disk_usage;
        }
        Ok(())
    }

    /// Get all node health statuses
    ///
    /// # Errors
    ///
    /// This function is currently infallible but returns a Result for future extensibility
    pub async fn get_all_node_health(&self) -> Result<Vec<NodeHealth>> {
        let nodes = self.nodes.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(nodes.values().cloned().collect())
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "stats struct mirrors monitoring dashboard field names"
)]
pub struct HealthStats {
    /// Registered services in the monitor.
    pub total_services: usize,
    /// Count currently marked [`HealthStatus::Healthy`].
    pub healthy_services: usize,
    /// Count not healthy (degraded, unhealthy, or unknown).
    pub unhealthy_services: usize,
}

/// Rollup suitable for top-of-dashboard “cluster green?” widgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealthStatus {
    /// Worst status among monitored services.
    pub status: HealthStatus,
    /// Healthy subset size.
    pub healthy_services: usize,
    /// Total monitored services.
    pub total_services: usize,
    /// When this rollup was computed (UTC).
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        let stats = monitor.get_health_stats().await;
        assert_eq!(stats.total_services, 0);
    }

    #[tokio::test]
    async fn test_service_registration() {
        let monitor = HealthMonitor::new();
        assert!(monitor.register_service(String::from("test-service")).await.is_ok());

        let stats = monitor.get_health_stats().await;
        assert_eq!(stats.total_services, 1);
    }

    #[tokio::test]
    async fn test_health_status_update() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let monitor = HealthMonitor::new();
        monitor.register_service(String::from("test-service")).await?;

        assert!(monitor.update_service_health("test-service", HealthStatus::Healthy).await.is_ok());

        let health = monitor.get_service_health("test-service").await?;
        assert!(health.is_some());
        if let Some(h) = health {
            assert!(matches!(h.status, HealthStatus::Healthy));
        }
        Ok(())
    }
}
