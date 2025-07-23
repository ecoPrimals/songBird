//! Health monitoring and alerting
//!
//! Provides comprehensive health checking capabilities for services

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use songbird_errors::Result;

/// Health monitor managing service health checks
pub struct HealthMonitor {
    check_tasks: HashMap<String, JoinHandle<()>>,
    health_history: HashMap<String, Vec<HealthCheckResult>>,
    alert_thresholds: HealthAlertThresholds,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            check_tasks: HashMap::new(),
            health_history: HashMap::new(),
            alert_thresholds: HealthAlertThresholds::default(),
        }
    }

    /// Start health monitoring for a service
    pub async fn start_monitoring(
        &mut self,
        service_id: String,
        policy: HealthCheckPolicy,
    ) -> Result<()> {
        tracing::info!("Starting health monitoring for service: {}", service_id);

        let service_id_clone = service_id.clone();
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(policy.check_interval);
            loop {
                interval.tick().await;

                match perform_health_check(&service_id_clone, &policy).await {
                    Ok(result) => {
                        tracing::debug!(
                            "Health check passed for {}: score={}",
                            service_id_clone,
                            result.health_score
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Health check failed for {}: {}", service_id_clone, e);
                    }
                }
            }
        });

        self.check_tasks.insert(service_id, task);
        Ok(())
    }

    /// Stop health monitoring for a service
    pub async fn stop_monitoring(&mut self, service_id: &str) -> Result<()> {
        if let Some(task) = self.check_tasks.remove(service_id) {
            task.abort();
            tracing::info!("Stopped health monitoring for service: {}", service_id);
        }
        Ok(())
    }

    /// Get health history for a service
    pub fn get_health_history(&self, service_id: &str) -> Vec<HealthCheckResult> {
        self.health_history
            .get(service_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Record health check result
    pub fn record_health_result(&mut self, service_id: String, result: HealthCheckResult) {
        let history = self.health_history.entry(service_id).or_default();
        history.push(result);

        // Keep only last 100 results
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check policy defining how to check service health
#[derive(Debug, Clone)]
pub struct HealthCheckPolicy {
    pub service_id: String,
    pub check_interval: Duration,
    pub timeout: Duration,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
    pub health_check_path: Option<String>,
    pub custom_health_checks: Vec<CustomHealthCheck>,
    pub failure_action: HealthFailureAction,
}

/// Custom health check definition
#[derive(Debug, Clone)]
pub struct CustomHealthCheck {
    pub name: String,
    pub check_type: HealthCheckType,
    pub parameters: HashMap<String, String>,
    pub weight: f64, // Weight in overall health calculation
}

/// Types of health checks supported
#[derive(Debug, Clone)]
pub enum HealthCheckType {
    HttpEndpoint { path: String, expected_status: u16 },
    TcpConnect { port: u16 },
    ProcessCheck { process_name: String },
    MemoryUsage { max_percentage: f64 },
    CpuUsage { max_percentage: f64 },
    CustomScript { script_path: String },
}

/// Actions to take when health check fails
#[derive(Debug, Clone)]
pub enum HealthFailureAction {
    Alert,
    Restart,
    Scale,
    Migrate,
    Custom { action: String },
}

/// Result of a health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub timestamp: Instant,
    pub success: bool,
    pub response_time: Duration,
    pub health_score: f64,
    pub details: HashMap<String, String>,
}

/// Health alert thresholds
pub struct HealthAlertThresholds {
    pub critical_health_score: f64,
    pub warning_health_score: f64,
    pub max_consecutive_failures: u32,
}

impl Default for HealthAlertThresholds {
    fn default() -> Self {
        Self {
            critical_health_score: 0.3,
            warning_health_score: 0.7,
            max_consecutive_failures: 3,
        }
    }
}

/// Perform a health check for a service
async fn perform_health_check(
    service_id: &str,
    policy: &HealthCheckPolicy,
) -> Result<HealthCheckResult> {
    let start_time = Instant::now();
    let mut success = true;
    let mut health_score = 1.0;
    let mut details = HashMap::new();

    // Perform custom health checks
    for check in &policy.custom_health_checks {
        match perform_custom_check(check).await {
            Ok(score) => {
                details.insert(check.name.clone(), format!("Passed (score: {score:.2})"));
                health_score = (health_score + score * check.weight) / (1.0 + check.weight);
            }
            Err(e) => {
                details.insert(check.name.clone(), format!("Failed: {e}"));
                success = false;
                health_score *= 0.5; // Reduce score on failure
            }
        }
    }

    // HTTP endpoint check if configured
    if let Some(path) = &policy.health_check_path {
        match perform_http_check(service_id, path).await {
            Ok(()) => {
                details.insert("http_endpoint".to_string(), "Passed".to_string());
            }
            Err(e) => {
                details.insert("http_endpoint".to_string(), format!("Failed: {e}"));
                success = false;
                health_score *= 0.7;
            }
        }
    }

    let response_time = start_time.elapsed();

    Ok(HealthCheckResult {
        timestamp: start_time,
        success,
        response_time,
        health_score,
        details,
    })
}

/// Perform a custom health check
async fn perform_custom_check(check: &CustomHealthCheck) -> Result<f64> {
    match &check.check_type {
        HealthCheckType::HttpEndpoint {
            path,
            expected_status,
        } => {
            // Simulate HTTP check
            tracing::debug!("Performing HTTP check on {}", path);
            if expected_status == &200 {
                Ok(1.0)
            } else {
                Ok(0.8)
            }
        }
        HealthCheckType::TcpConnect { port } => {
            // Simulate TCP connection check
            tracing::debug!("Performing TCP check on port {}", port);
            Ok(1.0)
        }
        HealthCheckType::ProcessCheck { process_name } => {
            use sysinfo::{ProcessRefreshKind, RefreshKind, System};

            tracing::debug!("Checking process: {}", process_name);

            let mut system = System::new_with_specifics(
                RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
            );
            system.refresh_processes();

            // Check if process exists and is healthy
            let process_found = system.processes().values().any(|process| {
                process
                    .name()
                    .to_lowercase()
                    .contains(&process_name.to_lowercase())
            });

            Ok(if process_found { 1.0 } else { 0.0 })
        }
        HealthCheckType::MemoryUsage { max_percentage } => {
            use sysinfo::System;

            let mut system = System::new_all();
            system.refresh_memory();

            let current_usage =
                ((system.used_memory() as f64) / (system.total_memory() as f64)) * 100.0;
            tracing::debug!("Current memory usage: {:.2}%", current_usage);

            if current_usage < *max_percentage {
                Ok(1.0 - (current_usage / 100.0))
            } else {
                Ok(0.0)
            }
        }
        HealthCheckType::CpuUsage { max_percentage } => {
            use sysinfo::System;

            let mut system = System::new_all();
            system.refresh_cpu();

            // Wait a bit for accurate CPU measurement
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            system.refresh_cpu();

            let current_usage = system.global_cpu_info().cpu_usage() as f64;
            tracing::debug!("Current CPU usage: {:.2}%", current_usage);

            if current_usage < *max_percentage {
                Ok(1.0 - (current_usage / 100.0))
            } else {
                Ok(0.0)
            }
        }
        HealthCheckType::CustomScript { script_path } => {
            // Simulate custom script execution
            tracing::debug!("Running custom script: {}", script_path);
            Ok(0.9)
        }
    }
}

/// Perform HTTP health check
async fn perform_http_check(service_id: &str, path: &str) -> Result<()> {
    use reqwest;
    use tokio::time::{timeout, Duration};

    tracing::debug!("HTTP health check for {} at {}", service_id, path);

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            songbird_errors::SongbirdError::service_error(
                "registry",
                format!("Failed to create HTTP client: {e}"),
            )
        })?;

    // Perform the health check request
    let response = timeout(Duration::from_secs(10), client.get(path).send())
        .await
        .map_err(|_| {
            songbird_errors::SongbirdError::service_error(
                "registry",
                "HTTP health check timeout".to_string(),
            )
        })?
        .map_err(|e| {
            songbird_errors::SongbirdError::service_error(
                "registry",
                format!("HTTP health check failed: {e}"),
            )
        })?;

    if response.status().is_success() {
        tracing::debug!(
            "HTTP health check for {} passed: {}",
            service_id,
            response.status()
        );
        Ok(())
    } else {
        Err(songbird_errors::SongbirdError::service_error(
            "registry",
            format!(
                "HTTP health check for {} failed: {}",
                service_id,
                response.status()
            ),
        ))
    }
}

/// Execute health failure action
pub async fn execute_health_failure_action(
    service_id: &str,
    action: &HealthFailureAction,
) -> Result<()> {
    match action {
        HealthFailureAction::Alert => {
            tracing::warn!("Health alert triggered for service: {}", service_id);
        }
        HealthFailureAction::Restart => {
            tracing::info!("Restarting service due to health failure: {}", service_id);
        }
        HealthFailureAction::Scale => {
            tracing::info!("Scaling service due to health failure: {}", service_id);
        }
        HealthFailureAction::Migrate => {
            tracing::info!("Migrating service due to health failure: {}", service_id);
        }
        HealthFailureAction::Custom { action } => {
            tracing::info!(
                "Executing custom action '{}' for service: {}",
                action,
                service_id
            );
        }
    }
    Ok(())
}
