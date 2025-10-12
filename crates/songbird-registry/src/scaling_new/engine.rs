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
        let mut policies = self.policies.write().await;
        policies.remove(plugin_id);

        let mut instances = self.instances.write().await;
        instances.remove(plugin_id);
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
