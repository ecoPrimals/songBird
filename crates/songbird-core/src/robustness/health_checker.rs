//! Health checking implementation for service monitoring

use super::config::HealthCheckConfig;
use super::error_types::HealthStatus;
use std::time::{Duration, Instant};

/// Health checker instance for monitoring service health
#[derive(Debug)]
pub struct HealthCheckerInstance {
    pub id: String,
    pub config: HealthCheckConfig,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub last_check_time: Option<Instant>,
    pub last_check_duration: Option<Duration>,
    pub health_status: HealthStatus,
}

impl HealthCheckerInstance {
    pub fn new(id: String, config: HealthCheckConfig) -> Self {
        Self {
            id,
            config,
            consecutive_failures: 0,
            consecutive_successes: 0,
            total_checks: 0,
            successful_checks: 0,
            failed_checks: 0,
            last_check_time: None,
            last_check_duration: None,
            health_status: HealthStatus::Unknown,
        }
    }

    /// Perform a health check
    pub async fn perform_health_check<F, Fut>(&mut self, check_fn: F) -> HealthStatus
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start_time = Instant::now();
        self.total_checks += 1;
        self.last_check_time = Some(start_time);

        // Execute the health check with timeout
        let check_result = tokio::time::timeout(self.config.check_timeout, check_fn()).await;

        let duration = start_time.elapsed();
        self.last_check_duration = Some(duration);

        let is_healthy = match check_result {
            Ok(result) => result,
            Err(_) => false, // Timeout
        };

        self.record_check_result(is_healthy);
        self.update_health_status();

        self.health_status.clone()
    }

    /// Record the result of a health check
    fn record_check_result(&mut self, is_healthy: bool) {
        if is_healthy {
            self.successful_checks += 1;
            self.consecutive_successes += 1;
            self.consecutive_failures = 0;
        } else {
            self.failed_checks += 1;
            self.consecutive_failures += 1;
            self.consecutive_successes = 0;
        }
    }

    /// Update health status based on consecutive results
    fn update_health_status(&mut self) {
        match self.health_status {
            HealthStatus::Healthy => {
                if self.consecutive_failures >= self.config.failure_threshold {
                    self.health_status = HealthStatus::Unhealthy;
                }
            }
            HealthStatus::Unhealthy => {
                if self.consecutive_successes >= self.config.success_threshold {
                    self.health_status = HealthStatus::Healthy;
                } else if self.consecutive_successes > 0 {
                    self.health_status = HealthStatus::Degraded;
                }
            }
            HealthStatus::Degraded => {
                if self.consecutive_successes >= self.config.success_threshold {
                    self.health_status = HealthStatus::Healthy;
                } else if self.consecutive_failures >= self.config.failure_threshold {
                    self.health_status = HealthStatus::Unhealthy;
                }
            }
            HealthStatus::Unknown => {
                if self.consecutive_successes >= self.config.success_threshold {
                    self.health_status = HealthStatus::Healthy;
                } else if self.consecutive_failures >= self.config.failure_threshold {
                    self.health_status = HealthStatus::Unhealthy;
                }
            }
        }
    }

    /// Get the current health status
    pub fn get_health_status(&self) -> HealthStatus {
        self.health_status.clone()
    }

    /// Check if the service is considered healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health_status, HealthStatus::Healthy)
    }

    /// Get the success rate of health checks
    pub fn get_success_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.successful_checks as f64 / self.total_checks as f64
        }
    }

    /// Check if it's time for the next health check
    pub fn should_check(&self) -> bool {
        match self.last_check_time {
            Some(last_check) => last_check.elapsed() >= self.config.check_interval,
            None => true, // Never checked before
        }
    }

    /// Reset health check statistics
    pub fn reset_stats(&mut self) {
        self.consecutive_failures = 0;
        self.consecutive_successes = 0;
        self.total_checks = 0;
        self.successful_checks = 0;
        self.failed_checks = 0;
        self.health_status = HealthStatus::Unknown;
    }
}
