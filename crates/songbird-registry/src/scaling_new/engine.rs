// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Scaling engine implementation
//!
//! Manages automatic scaling of plugins based on policies.

use super::policy::ScalingPolicy;
use crate::types::PluginId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Auto-scaling engine
pub struct ScalingEngine {
    /// Scaling policies per plugin
    policies: Arc<RwLock<HashMap<PluginId, ScalingPolicy>>>,

    /// Current instance counts
    instances: Arc<RwLock<HashMap<PluginId, u32>>>,
}

impl ScalingEngine {
    /// Create a new scaling engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a scaling policy for a plugin
    pub async fn add_policy(&self, plugin_id: PluginId, policy: ScalingPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert(plugin_id, policy);
    }

    /// Remove a scaling policy for a plugin
    pub async fn remove_policy(&self, plugin_id: &PluginId) {
        self.policies.write().await.remove(plugin_id);
        self.instances.write().await.remove(plugin_id);
    }

    /// Get current instance count for a plugin
    pub async fn get_instance_count(&self, plugin_id: &PluginId) -> u32 {
        let instances = self.instances.read().await;
        instances.get(plugin_id).copied().unwrap_or(1)
    }
}

impl Default for ScalingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::policy::ScalingPolicy;
    use super::*;
    use crate::types::PluginId;

    #[tokio::test]
    async fn add_remove_policy_and_instance_default() {
        let engine = ScalingEngine::new();
        let id = PluginId::new("scale-test");
        engine.add_policy(id.clone(), ScalingPolicy::default()).await;
        assert_eq!(engine.get_instance_count(&id).await, 1);
        engine.remove_policy(&id).await;
        assert_eq!(engine.get_instance_count(&id).await, 1);
    }

    #[tokio::test]
    async fn default_matches_new() {
        let a = ScalingEngine::new();
        let b = ScalingEngine::default();
        let id = PluginId::new("x");
        assert_eq!(a.get_instance_count(&id).await, b.get_instance_count(&id).await);
    }

    #[tokio::test]
    async fn get_instance_count_unknown_returns_one() {
        let engine = ScalingEngine::new();
        let id = PluginId::new("unknown-plugin");
        assert_eq!(engine.get_instance_count(&id).await, 1);
    }
}
