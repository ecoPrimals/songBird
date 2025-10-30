//! Health monitoring implementation
//!
//! Monitors plugin health and triggers alerts.

use crate::types::{HealthCheckConfig, HealthStatus, PluginId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health monitor for plugins
pub struct HealthMonitor {
    /// Health check configurations
    checks: Arc<RwLock<HashMap<PluginId, HealthCheckConfig>>>,

    /// Current health statuses
    statuses: Arc<RwLock<HashMap<PluginId, HealthStatus>>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a health check for a plugin
    pub async fn add_check(&self, plugin_id: PluginId, config: HealthCheckConfig) {
        let mut checks = self.checks.write().await;
        checks.insert(plugin_id, config);
    }

    /// Remove a health check for a plugin
    pub async fn remove_check(&self, plugin_id: &PluginId) {
        let mut checks = self.checks.write().await;
        checks.remove(plugin_id);

        let mut statuses = self.statuses.write().await;
        statuses.remove(plugin_id);
    }

    /// Get current health status for a plugin
    pub async fn get_status(&self, plugin_id: &PluginId) -> Option<HealthStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(plugin_id).cloned()
    }

    /// Get all health statuses
    pub async fn get_all_statuses(&self) -> HashMap<PluginId, HealthStatus> {
        let statuses = self.statuses.read().await;
        statuses.clone()
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
