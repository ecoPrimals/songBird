//! Scaling policy types
//!
//! Defines scaling policies and actions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Scaling policy for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Minimum number of instances
    pub min_instances: u32,

    /// Maximum number of instances
    pub max_instances: u32,

    /// Target CPU utilization (0.0 to 1.0)
    pub target_cpu: f64,

    /// Target memory utilization (0.0 to 1.0)
    pub target_memory: f64,

    /// Cooldown period between scaling actions
    pub cooldown: Duration,
}

impl Default for ScalingPolicy {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            target_cpu: 0.7,
            target_memory: 0.8,
            cooldown: Duration::from_secs(300),
        }
    }
}

/// Scaling action to take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAction {
    /// Scale up by N instances
    ScaleUp(u32),

    /// Scale down by N instances
    ScaleDown(u32),

    /// No action needed
    NoAction,
}
