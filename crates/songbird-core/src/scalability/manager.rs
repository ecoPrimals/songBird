//! Scalability Manager
//!
//! Core manager implementation for auto-scaling services

use crate::scalability::types::*;
use chrono::Utc;
use songbird_errors::Result;
use std::time::Duration;
use tracing::info;

/// Scalability manager
#[derive(Debug)]
pub struct ScalabilityManager {
    config: ScalabilityConfig,
    current_instances: Vec<String>,
    last_scale_action: Option<chrono::DateTime<Utc>>,
    metrics_history: Vec<ResourceUsage>,
}

impl ScalabilityManager {
    /// Create a new scalability manager
    pub fn new(config: ScalabilityConfig) -> Self {
        Self {
            config,
            current_instances: Vec::new(),
            last_scale_action: None,
            metrics_history: Vec::new(),
        }
    }

    /// Get current instances
    pub async fn get_current_instances(&self) -> Vec<String> {
        self.current_instances.clone()
    }

    /// Check if manager is healthy
    pub async fn is_healthy(&self) -> bool {
        true // Always healthy for now
    }

    /// Add metrics to history
    pub async fn add_metrics(&mut self, metrics: ResourceUsage) {
        self.metrics_history.push(metrics);
        // Keep only last 100 entries
        if self.metrics_history.len() > 100 {
            self.metrics_history.remove(0);
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ScalabilityConfig {
        &self.config
    }

    /// Get metrics history
    pub fn metrics_history(&self) -> &[ResourceUsage] {
        &self.metrics_history
    }

    /// Get last scale action time
    pub fn last_scale_action(&self) -> Option<chrono::DateTime<Utc>> {
        self.last_scale_action
    }

    /// Evaluate scaling decision
    pub async fn evaluate_scaling(&mut self) -> Result<ScalingAction> {
        let current_instances = self.current_instances.len() as u32;

        if let Some(latest_metrics) = self.metrics_history.last() {
            if latest_metrics.cpu_percent > self.config.target_cpu_threshold
                && current_instances < self.config.max_instances
            {
                Ok(ScalingAction {
                    action_type: ScalingActionType::ScaleUp,
                    target_instances: current_instances + 1,
                    reason: format!(
                        "CPU usage {}% > {}%",
                        latest_metrics.cpu_percent, self.config.target_cpu_threshold
                    ),
                    timestamp: Utc::now(),
                })
            } else if latest_metrics.cpu_percent < 30.0
                && current_instances > self.config.min_instances
            {
                Ok(ScalingAction {
                    action_type: ScalingActionType::ScaleDown,
                    target_instances: current_instances - 1,
                    reason: format!("CPU usage {}% < 30%", latest_metrics.cpu_percent),
                    timestamp: Utc::now(),
                })
            } else {
                Ok(ScalingAction {
                    action_type: ScalingActionType::NoAction,
                    target_instances: current_instances,
                    reason: "Metrics within acceptable range".to_string(),
                    timestamp: Utc::now(),
                })
            }
        } else {
            Ok(ScalingAction {
                action_type: ScalingActionType::NoAction,
                target_instances: current_instances,
                reason: "No metrics available".to_string(),
                timestamp: Utc::now(),
            })
        }
    }

    /// Predict future load
    pub async fn predict_future_load(&self, duration: Duration) -> Result<f64> {
        // Simple prediction based on recent trend
        if self.metrics_history.len() < 2 {
            return Ok(0.0);
        }

        let recent = &self.metrics_history[self.metrics_history.len() - 2..];
        let trend = recent[1].cpu_percent - recent[0].cpu_percent;
        let predicted = recent[1].cpu_percent + trend * (duration.as_secs() as f64 / 60.0);

        Ok(predicted.clamp(0.0, 100.0))
    }

    /// Execute scaling action
    pub async fn execute_scaling_action(&mut self, action: ScalingAction) -> Result<()> {
        info!("Executing scaling action: {:?}", action);

        match action.action_type {
            ScalingActionType::ScaleUp => {
                let new_instance = format!("instance-{}", self.current_instances.len());
                self.current_instances.push(new_instance);
                self.last_scale_action = Some(Utc::now());
            }
            ScalingActionType::ScaleDown => {
                if !self.current_instances.is_empty() {
                    self.current_instances.pop();
                    self.last_scale_action = Some(Utc::now());
                }
            }
            ScalingActionType::NoAction => {
                // No action needed
            }
        }

        Ok(())
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ScalabilityConfig) {
        self.config = config;
    }

    /// Clear metrics history
    pub fn clear_metrics_history(&mut self) {
        self.metrics_history.clear();
    }

    /// Get current instance count
    pub fn current_instance_count(&self) -> u32 {
        self.current_instances.len() as u32
    }
}
