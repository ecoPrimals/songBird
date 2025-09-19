//! AutoScaler Implementation
//!
//! Auto-scaler for managing service instances based on metrics

use crate::scalability::types::*;
use chrono::{DateTime, Utc};
use songbird_errors::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::info;

/// Auto-scaler for managing service instances
pub struct AutoScaler {
    config: ServiceScalingConfig,
    stats: ScalabilityStats,
    resource_pool: ResourcePool,
    scaling_history: Vec<ScalingEvent>,
    last_scaling_time: Option<DateTime<Utc>>,
    cooldown_period: Duration,
}

impl AutoScaler {
    #[must_use]
    pub fn new(config: ServiceScalingConfig, resource_pool: ResourcePool) -> Self {
        Self {
            config,
            stats: ScalabilityStats {
                total_scale_events: 0,
                scale_up_events: 0,
                scale_down_events: 0,
                average_response_time: 0.0,
                current_load: 0.0,
                resource_utilization: ResourceUsage::default(),
            },
            resource_pool,
            scaling_history: Vec::new(),
            last_scaling_time: None,
            cooldown_period: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Evaluate scaling decision based on current metrics
    ///
    /// # Errors
    ///
    /// Returns an error if the scaling evaluation fails due to resource
    /// constraints or invalid service configuration.
    pub fn evaluate_scaling(
        &mut self,
        service_id: &str,
        current_instances: u32,
        resource_usage: &ResourceUsage,
        request_rate: f64,
    ) -> SongbirdResult<ScalingDecision> {
        // Check cooldown period
        if let Some(last_time) = self.last_scaling_time {
            let elapsed = Utc::now() - last_time;
            if elapsed
                < chrono::Duration::from_std(self.cooldown_period)
                    .unwrap_or(chrono::Duration::zero())
            {
                return Ok(ScalingDecision::NoAction);
            }
        }

        // Determine scaling decision based on resource usage
        let decision = if resource_usage.cpu_percent > self.config.scale_up_threshold
            || resource_usage.memory_percent > self.config.scale_up_threshold
        {
            if current_instances < self.config.max_instances {
                let scale_factor = Self::calculate_scale_factor(resource_usage, request_rate);
                let target_instances =
                    (current_instances + scale_factor as u32).min(self.config.max_instances);
                ScalingDecision::ScaleUp(target_instances - current_instances)
            } else {
                ScalingDecision::NoAction
            }
        } else if resource_usage.cpu_percent < self.config.scale_down_threshold
            && resource_usage.memory_percent < self.config.scale_down_threshold
        {
            if current_instances > self.config.min_instances {
                ScalingDecision::ScaleDown(1)
            } else {
                ScalingDecision::NoAction
            }
        } else {
            ScalingDecision::NoAction
        };

        // Record scaling event
        if !matches!(decision, ScalingDecision::NoAction) {
            let target_instances = match decision {
                ScalingDecision::ScaleUp(delta) => current_instances + delta,
                ScalingDecision::ScaleDown(delta) => current_instances - delta,
                ScalingDecision::NoAction => current_instances,
            };

            let event = ScalingEvent {
                timestamp: Utc::now(),
                service_id: service_id.to_string(),
                decision: decision.clone(),
                reason: Self::generate_scaling_reason(
                    current_instances,
                    resource_usage,
                    request_rate,
                    &decision,
                ),
                current_instances,
                target_instances,
            };

            self.scaling_history.push(event);
            self.last_scaling_time = Some(Utc::now());

            // Update statistics
            self.stats.total_scale_events += 1;
            match decision {
                ScalingDecision::ScaleUp(_) => self.stats.scale_up_events += 1,
                ScalingDecision::ScaleDown(_) => self.stats.scale_down_events += 1,
                ScalingDecision::NoAction => {}
            }
        }

        // Update current metrics
        self.stats.resource_utilization = resource_usage.clone();
        self.stats.current_load = request_rate;

        Ok(decision)
    }

    /// Calculate a scale factor based on resource usage and request rate
    fn calculate_scale_factor(resource_usage: &ResourceUsage, request_rate: f64) -> f64 {
        // Simple scaling algorithm - can be made more sophisticated
        let cpu_factor: f64 = if resource_usage.cpu_percent > 90.0 {
            2.0
        } else {
            1.0
        };
        let memory_factor: f64 = if resource_usage.memory_percent > 90.0 {
            2.0
        } else {
            1.0
        };
        let load_factor: f64 = if request_rate > 1000.0 { 2.0 } else { 1.0 };

        cpu_factor.max(memory_factor).max(load_factor)
    }

    /// Determine scaling direction based on current state
    #[allow(dead_code)]
    fn determine_scale_direction(
        current_instances: u32,
        resource_usage: &ResourceUsage,
        request_rate: f64,
    ) -> ScaleDirection {
        if resource_usage.cpu_percent > 80.0
            || resource_usage.memory_percent > 80.0
            || request_rate > 500.0
        {
            ScaleDirection::Up
        } else if resource_usage.cpu_percent < 20.0
            && resource_usage.memory_percent < 20.0
            && request_rate < 100.0
            && current_instances > 1
        {
            ScaleDirection::Down
        } else {
            ScaleDirection::None
        }
    }

    /// Generate reason for scaling decision
    fn generate_scaling_reason(
        _current_instances: u32,
        resource_usage: &ResourceUsage,
        _request_rate: f64,
        decision: &ScalingDecision,
    ) -> String {
        match decision {
            ScalingDecision::ScaleUp(_) => {
                format!(
                    "High resource usage: CPU {}%, Memory {}%",
                    resource_usage.cpu_percent, resource_usage.memory_percent
                )
            }
            ScalingDecision::ScaleDown(_) => {
                format!(
                    "Low resource usage: CPU {}%, Memory {}%",
                    resource_usage.cpu_percent, resource_usage.memory_percent
                )
            }
            ScalingDecision::NoAction => "No scaling required".to_string(),
        }
    }

    /// Execute scaling action
    ///
    /// # Errors
    ///
    /// Returns an error if the scaling operation fails due to resource
    /// constraints or infrastructure issues.
    pub async fn execute_scaling(
        &mut self,
        service_id: &str,
        decision: &ScalingDecision,
    ) -> SongbirdResult<()> {
        match decision {
            ScalingDecision::ScaleUp(instances) => {
                info!(
                    "Scaling up service {}: adding {} instances",
                    service_id, instances
                );
                self.scale_up_service(service_id, *instances).await?;
            }
            ScalingDecision::ScaleDown(instances) => {
                info!(
                    "Scaling down service {}: removing {} instances",
                    service_id, instances
                );
                self.scale_down_service(service_id, *instances).await?;
            }
            ScalingDecision::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Scale up service instances
    async fn scale_up_service(&mut self, service_id: &str, instances: u32) -> SongbirdResult<()> {
        // Check if we have enough resources
        let required_cpu = f64::from(instances) * 0.5; // Assume 0.5 CPU per instance
        let required_memory = instances * 512; // Assume 512MB per instance

        if f64::from(self.resource_pool.available_cpu_cores) < required_cpu {
            return Err(SongbirdError::configuration(format!(
                "Insufficient CPU cores: need {}, have {}",
                required_cpu, self.resource_pool.available_cpu_cores
            )));
        }

        if self.resource_pool.available_memory_mb < required_memory {
            return Err(SongbirdError::configuration(format!(
                "Insufficient memory: need {}MB, have {}MB",
                required_memory, self.resource_pool.available_memory_mb
            )));
        }

        // Allocate resources
        self.resource_pool.available_cpu_cores -=
            u32::try_from(required_cpu as u64).unwrap_or(self.resource_pool.available_cpu_cores);
        self.resource_pool.available_memory_mb -=
            required_memory.min(self.resource_pool.available_memory_mb);

        info!(
            "Scaled up service {}: added {} instances",
            service_id, instances
        );
        Ok(())
    }

    /// Scale down service instances
    async fn scale_down_service(&mut self, service_id: &str, instances: u32) -> SongbirdResult<()> {
        // Free up resources
        let freed_cpu = f64::from(instances) * 0.5; // Assume 0.5 CPU per instance
        let freed_memory = instances * 512; // Assume 512MB per instance

        self.resource_pool.available_cpu_cores += u32::try_from(freed_cpu as u64).unwrap_or(0);
        self.resource_pool.available_memory_mb += freed_memory;

        // Ensure we don't exceed total resources
        self.resource_pool.available_cpu_cores = self
            .resource_pool
            .available_cpu_cores
            .min(self.resource_pool.total_cpu_cores);
        self.resource_pool.available_memory_mb = self
            .resource_pool
            .available_memory_mb
            .min(self.resource_pool.total_memory_mb);

        info!(
            "Scaled down service {}: removed {} instances",
            service_id, instances
        );
        Ok(())
    }

    /// Get scaling statistics
    #[must_use]
    pub fn get_stats(&self) -> &ScalabilityStats {
        &self.stats
    }

    /// Get scaling history
    #[must_use]
    pub fn get_scaling_history(&self) -> &[ScalingEvent] {
        &self.scaling_history
    }

    /// Get resource pool status
    #[must_use]
    pub fn get_resource_pool(&self) -> &ResourcePool {
        &self.resource_pool
    }

    /// Update resource pool
    pub fn update_resource_pool(&mut self, resource_pool: ResourcePool) {
        self.resource_pool = resource_pool;
    }

    /// Set cooldown period
    pub fn set_cooldown_period(&mut self, duration: Duration) {
        self.cooldown_period = duration;
    }

    /// Get current configuration
    pub fn config(&self) -> &ServiceScalingConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ServiceScalingConfig) {
        self.config = config;
    }
}
