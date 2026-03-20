// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling_policy_default_values() {
        let p = ScalingPolicy::default();
        assert_eq!(p.min_instances, 1);
        assert_eq!(p.max_instances, 10);
        assert!(p.cooldown.as_secs() > 0);
    }

    #[test]
    fn scaling_policy_json_roundtrip() {
        let p = ScalingPolicy::default();
        let json = serde_json::to_string(&p).unwrap();
        let q: ScalingPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(q.min_instances, p.min_instances);
        assert_eq!(q.max_instances, p.max_instances);
    }

    #[test]
    fn scaling_action_json_roundtrip() {
        let a = ScalingAction::ScaleUp(3);
        let json = serde_json::to_string(&a).unwrap();
        let b: ScalingAction = serde_json::from_str(&json).unwrap();
        assert!(matches!(b, ScalingAction::ScaleUp(3)));
    }

    #[test]
    fn scaling_action_no_action_roundtrip() {
        let json = serde_json::to_string(&ScalingAction::NoAction).unwrap();
        let a: ScalingAction = serde_json::from_str(&json).unwrap();
        assert!(matches!(a, ScalingAction::NoAction));
    }
}
