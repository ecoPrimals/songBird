// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
        self.checks.write().await.remove(plugin_id);
        self.statuses.write().await.remove(plugin_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HealthCheckConfig, HealthCheckType, PluginId};
    use std::time::Duration;

    fn sample_config() -> HealthCheckConfig {
        HealthCheckConfig {
            check_type: HealthCheckType::HttpEndpoint {
                url: "http://127.0.0.1:1".into(),
                expected_status: 200,
            },
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(5),
            failure_threshold: 1,
            success_threshold: 1,
        }
    }

    #[tokio::test]
    async fn add_and_remove_check_roundtrip() {
        let mon = HealthMonitor::new();
        let id = PluginId::new("p1");
        mon.add_check(id.clone(), sample_config()).await;
        mon.remove_check(&id).await;
        assert!(mon.get_status(&id).await.is_none());
    }

    #[tokio::test]
    async fn get_all_statuses_empty_by_default() {
        let mon = HealthMonitor::new();
        assert!(mon.get_all_statuses().await.is_empty());
    }

    #[tokio::test]
    async fn default_matches_new() {
        let a = HealthMonitor::new();
        let b = HealthMonitor::default();
        assert!(a.get_all_statuses().await.is_empty());
        assert!(b.get_all_statuses().await.is_empty());
    }
}
