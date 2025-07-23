//! Auto-scaling engine
//!
//! Provides intelligent auto-scaling capabilities based on metrics and policies

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use crate::service::ServiceMetrics;
use songbird_errors::Result;

/// Auto-scaling engine managing service scaling decisions
pub struct AutoScalingEngine {
    scaling_tasks: HashMap<String, JoinHandle<()>>,
    scaling_decisions: HashMap<String, Vec<ScalingDecision>>,
}

impl AutoScalingEngine {
    pub fn new() -> Self {
        Self {
            scaling_tasks: HashMap::new(),
            scaling_decisions: HashMap::new(),
        }
    }

    /// Start auto-scaling monitoring for a service
    pub async fn start_scaling_monitoring(
        &mut self,
        service_id: String,
        policy: AutoScalingPolicy,
    ) -> Result<()> {
        tracing::info!(
            "Starting auto-scaling monitoring for service: {}",
            service_id
        );

        let service_id_clone = service_id.clone();
        let policy_clone = policy.clone();
        let task = tokio::spawn(async move {
            let mut scale_up_duration = Duration::from_secs(0);
            let mut scale_down_duration = Duration::from_secs(0);
            let mut last_scaling = Instant::now();

            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;

                // Check if we're in cooldown period
                if last_scaling.elapsed() < policy_clone.cooldown_period {
                    continue;
                }

                // Collect current metrics
                match collect_service_metrics(&service_id_clone).await {
                    Ok(metrics) => {
                        // Check scale up conditions
                        if evaluate_scale_up_conditions(
                            &metrics,
                            &policy_clone,
                            &mut scale_up_duration,
                        )
                        .await
                        {
                            let target =
                                calculate_scale_up_target(1, &policy_clone, &metrics).await; // Placeholder current_instances
                            tracing::info!(
                                "Scaling up {} to {} instances",
                                service_id_clone,
                                target
                            );
                            last_scaling = Instant::now();
                            scale_up_duration = Duration::from_secs(0);
                        }
                        // Check scale down conditions
                        else if evaluate_scale_down_conditions(
                            &metrics,
                            &policy_clone,
                            &mut scale_down_duration,
                        )
                        .await
                        {
                            let target =
                                calculate_scale_down_target(1, &policy_clone, &metrics).await; // Placeholder current_instances
                            tracing::info!(
                                "Scaling down {} to {} instances",
                                service_id_clone,
                                target
                            );
                            last_scaling = Instant::now();
                            scale_down_duration = Duration::from_secs(0);
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to collect metrics for {}: {}",
                            service_id_clone,
                            e
                        );
                    }
                }
            }
        });

        self.scaling_tasks.insert(service_id, task);
        Ok(())
    }

    /// Stop auto-scaling monitoring for a service
    pub async fn stop_scaling_monitoring(&mut self, service_id: &str) -> Result<()> {
        if let Some(task) = self.scaling_tasks.remove(service_id) {
            task.abort();
            tracing::info!(
                "Stopped auto-scaling monitoring for service: {}",
                service_id
            );
        }
        Ok(())
    }

    /// Get scaling decision history
    pub fn get_scaling_decisions(&self, service_id: &str) -> Vec<ScalingDecision> {
        self.scaling_decisions
            .get(service_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Record scaling decision
    pub fn record_scaling_decision(&mut self, service_id: String, decision: ScalingDecision) {
        let decisions = self.scaling_decisions.entry(service_id).or_default();
        decisions.push(decision);

        // Keep only last 50 decisions
        if decisions.len() > 50 {
            decisions.drain(0..decisions.len() - 50);
        }
    }
}

impl Default for AutoScalingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Auto-scaling policy defining scaling behavior
#[derive(Debug, Clone)]
pub struct AutoScalingPolicy {
    pub service_id: String,
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub target_request_rate: f64,
    pub scale_up_threshold: ScalingThreshold,
    pub scale_down_threshold: ScalingThreshold,
    pub cooldown_period: Duration,
    pub scaling_strategy: ScalingStrategy,
}

/// Scaling threshold configuration
#[derive(Debug, Clone)]
pub struct ScalingThreshold {
    pub metric_thresholds: HashMap<String, f64>,
    pub sustained_duration: Duration,
    pub scale_factor: f64, // How aggressively to scale (instances to add/remove)
}

/// Scaling strategies
#[derive(Debug, Clone)]
pub enum ScalingStrategy {
    Linear,      // Add/remove instances linearly
    Exponential, // Scale exponentially for rapid response
    Predictive,  // Use historical patterns for scaling
    Gaming,      // Gaming-optimized scaling for session-based workloads
}

/// Scaling direction
#[derive(Debug, Clone)]
pub enum ScalingDirection {
    Up,
    Down,
}

/// Scaling state
#[derive(Debug, Clone)]
pub enum ScalingState {
    Stable,
    ScalingUp { target: u32 },
    ScalingDown { target: u32 },
    Cooldown { until: Instant },
}

/// Scaling decision record
#[derive(Debug, Clone)]
pub struct ScalingDecision {
    pub timestamp: Instant,
    pub trigger_metric: String,
    pub trigger_value: f64,
    pub decision: ScalingDirection,
    pub instances_changed: u32,
    pub reason: String,
}

/// Collect service metrics for scaling decisions
pub async fn collect_service_metrics(service_id: &str) -> Result<ServiceMetrics> {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    tracing::debug!("Collecting metrics for service: {}", service_id);

    // Create system instance with process monitoring
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );
    system.refresh_all();

    // Try to find process(es) matching the service ID
    let mut total_cpu = 0.0;
    let mut total_memory = 0.0;
    let mut process_count = 0;

    for process in system.processes().values() {
        let process_name = process.name();
        // Match processes that contain the service ID in their name
        if process_name
            .to_lowercase()
            .contains(&service_id.to_lowercase())
        {
            total_cpu += process.cpu_usage() as f64;
            total_memory += process.memory() as f64;
            process_count += 1;
        }
    }

    // Calculate averages if processes found, otherwise use system-wide metrics
    let (cpu_util, memory_util) = if process_count > 0 {
        (
            total_cpu / process_count as f64,
            (total_memory / system.total_memory() as f64) * 100.0,
        )
    } else {
        // Fallback to estimated metrics based on system load
        let system_cpu = system.global_cpu_info().cpu_usage() as f64;
        let memory_usage = ((system.used_memory() as f64) / (system.total_memory() as f64)) * 100.0;
        (system_cpu, memory_usage)
    };

    // Estimate other metrics based on system state and service characteristics
    let request_rate = if cpu_util > 80.0 {
        200.0
    } else if cpu_util > 50.0 {
        150.0
    } else {
        100.0
    };
    let response_time = if cpu_util > 80.0 {
        300.0
    } else if cpu_util > 50.0 {
        200.0
    } else {
        150.0
    };
    let error_rate = if cpu_util > 90.0 {
        5.0
    } else if cpu_util > 70.0 {
        3.0
    } else {
        1.0
    };
    let active_connections = (request_rate * 0.5) as u32; // Estimate based on request rate
    let queue_depth = if cpu_util > 80.0 {
        25
    } else if cpu_util > 50.0 {
        15
    } else {
        5
    };

    Ok(ServiceMetrics {
        cpu_utilization: cpu_util,
        memory_utilization: memory_util,
        request_rate,
        response_time_ms: response_time,
        error_rate,
        active_connections,
        queue_depth,
    })
}

/// Evaluate if scale up conditions are met
pub async fn evaluate_scale_up_conditions(
    metrics: &ServiceMetrics,
    policy: &AutoScalingPolicy,
    scale_up_duration: &mut Duration,
) -> bool {
    let cpu_above_threshold = metrics.cpu_utilization > policy.target_cpu_utilization;
    let memory_above_threshold = metrics.memory_utilization > policy.target_memory_utilization;
    let requests_above_threshold = metrics.request_rate > policy.target_request_rate;

    if cpu_above_threshold || memory_above_threshold || requests_above_threshold {
        *scale_up_duration += Duration::from_secs(30);
        *scale_up_duration >= policy.scale_up_threshold.sustained_duration
    } else {
        *scale_up_duration = Duration::from_secs(0);
        false
    }
}

/// Evaluate if scale down conditions are met
pub async fn evaluate_scale_down_conditions(
    metrics: &ServiceMetrics,
    policy: &AutoScalingPolicy,
    scale_down_duration: &mut Duration,
) -> bool {
    let cpu_below_threshold = metrics.cpu_utilization < policy.target_cpu_utilization * 0.7;
    let memory_below_threshold =
        metrics.memory_utilization < policy.target_memory_utilization * 0.7;
    let requests_below_threshold = metrics.request_rate < policy.target_request_rate * 0.7;

    if cpu_below_threshold && memory_below_threshold && requests_below_threshold {
        *scale_down_duration += Duration::from_secs(30);
        *scale_down_duration >= policy.scale_down_threshold.sustained_duration
    } else {
        *scale_down_duration = Duration::from_secs(0);
        false
    }
}

/// Calculate target instances for scale up
pub async fn calculate_scale_up_target(
    current_instances: u32,
    policy: &AutoScalingPolicy,
    _metrics: &ServiceMetrics,
) -> u32 {
    let target = match policy.scaling_strategy {
        ScalingStrategy::Linear => current_instances + 1,
        ScalingStrategy::Exponential => current_instances * 2,
        ScalingStrategy::Predictive => current_instances + 2, // Simplified
        ScalingStrategy::Gaming => current_instances + 3,     // Aggressive for gaming
    };

    target.min(policy.max_instances)
}

/// Calculate target instances for scale down
pub async fn calculate_scale_down_target(
    current_instances: u32,
    policy: &AutoScalingPolicy,
    _metrics: &ServiceMetrics,
) -> u32 {
    let target = match policy.scaling_strategy {
        ScalingStrategy::Linear => current_instances.saturating_sub(1),
        ScalingStrategy::Exponential => current_instances / 2,
        ScalingStrategy::Predictive => current_instances.saturating_sub(1),
        ScalingStrategy::Gaming => current_instances.saturating_sub(2), // Conservative for gaming
    };

    target.max(policy.min_instances)
}
