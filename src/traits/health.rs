use crate::errors::SongbirdError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Type alias for custom health check functions
pub type HealthCheckFn =
    Box<dyn Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub response_time: Duration,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub retry_delay: Duration,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            retry_delay: Duration::from_secs(1),
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self, service_id: &str) -> Result<HealthCheckResult, SongbirdError>;
    fn get_config(&self) -> &HealthCheckConfig;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait HealthMonitor: Send + Sync {
    async fn register_health_check(
        &mut self,
        service_id: String,
        health_check: Box<dyn HealthCheck>,
    ) -> Result<(), SongbirdError>;
    async fn unregister_health_check(&mut self, service_id: &str) -> Result<(), SongbirdError>;
    async fn get_health_status(&self, service_id: &str)
        -> Result<HealthCheckResult, SongbirdError>;
    async fn get_all_health_statuses(
        &self,
    ) -> Result<HashMap<String, HealthCheckResult>, SongbirdError>;
    async fn start_monitoring(&mut self) -> Result<(), SongbirdError>;
    async fn stop_monitoring(&mut self) -> Result<(), SongbirdError>;
}

pub struct HttpHealthCheck {
    config: HealthCheckConfig,
    url: String,
    name: String,
}

impl HttpHealthCheck {
    pub fn new(name: String, url: String, config: HealthCheckConfig) -> Self {
        Self { config, url, name }
    }
}

#[async_trait]
impl HealthCheck for HttpHealthCheck {
    async fn check(&self, _service_id: &str) -> Result<HealthCheckResult, SongbirdError> {
        let start = std::time::Instant::now();

        // For now, simulate a health check - in real implementation, this would make an HTTP request
        let status = if self.url.contains("unhealthy") {
            HealthStatus::Unhealthy
        } else if self.url.contains("degraded") {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let response_time = start.elapsed();
        let mut details = HashMap::new();
        details.insert(
            "url".to_string(),
            serde_json::Value::String(self.url.clone()),
        );
        details.insert(
            "method".to_string(),
            serde_json::Value::String("GET".to_string()),
        );

        Ok(HealthCheckResult {
            status,
            message: format!("HTTP health check for {}", self.url),
            timestamp: Utc::now(),
            response_time,
            details,
        })
    }

    fn get_config(&self) -> &HealthCheckConfig {
        &self.config
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct TcpHealthCheck {
    config: HealthCheckConfig,
    address: String,
    name: String,
}

impl TcpHealthCheck {
    pub fn new(name: String, address: String, config: HealthCheckConfig) -> Self {
        Self {
            config,
            address,
            name,
        }
    }
}

#[async_trait]
impl HealthCheck for TcpHealthCheck {
    async fn check(&self, _service_id: &str) -> Result<HealthCheckResult, SongbirdError> {
        let start = std::time::Instant::now();

        // For now, simulate a TCP health check
        let status = HealthStatus::Healthy;
        let response_time = start.elapsed();
        let mut details = HashMap::new();
        details.insert(
            "address".to_string(),
            serde_json::Value::String(self.address.clone()),
        );

        Ok(HealthCheckResult {
            status,
            message: format!("TCP health check for {}", self.address),
            timestamp: Utc::now(),
            response_time,
            details,
        })
    }

    fn get_config(&self) -> &HealthCheckConfig {
        &self.config
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct CustomHealthCheck {
    config: HealthCheckConfig,
    name: String,
    check_fn: HealthCheckFn,
}

impl CustomHealthCheck {
    pub fn new<F>(name: String, config: HealthCheckConfig, check_fn: F) -> Self
    where
        F: Fn(&str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            config,
            name,
            check_fn: Box::new(check_fn),
        }
    }
}

#[async_trait]
impl HealthCheck for CustomHealthCheck {
    async fn check(&self, service_id: &str) -> Result<HealthCheckResult, SongbirdError> {
        let start = std::time::Instant::now();

        let healthy = (self.check_fn)(service_id).map_err(|e| SongbirdError::HealthCheck {
            message: format!("Health check request failed: {}", e),
        })?;

        let status = if healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        let response_time = start.elapsed();
        let mut details = HashMap::new();
        details.insert(
            "service_id".to_string(),
            serde_json::Value::String(service_id.to_string()),
        );

        Ok(HealthCheckResult {
            status,
            message: format!("Custom health check for {}", service_id),
            timestamp: Utc::now(),
            response_time,
            details,
        })
    }

    fn get_config(&self) -> &HealthCheckConfig {
        &self.config
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct DefaultHealthMonitor {
    health_checks: HashMap<String, Box<dyn HealthCheck>>,
    health_statuses: HashMap<String, HealthCheckResult>,
    monitoring_active: bool,
}

impl Default for DefaultHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHealthMonitor {
    pub fn new() -> Self {
        Self {
            health_checks: HashMap::new(),
            health_statuses: HashMap::new(),
            monitoring_active: false,
        }
    }
}

#[async_trait]
impl HealthMonitor for DefaultHealthMonitor {
    async fn register_health_check(
        &mut self,
        service_id: String,
        health_check: Box<dyn HealthCheck>,
    ) -> Result<(), SongbirdError> {
        self.health_checks.insert(service_id, health_check);
        Ok(())
    }

    async fn unregister_health_check(&mut self, service_id: &str) -> Result<(), SongbirdError> {
        self.health_checks.remove(service_id);
        self.health_statuses.remove(service_id);
        Ok(())
    }

    async fn get_health_status(
        &self,
        service_id: &str,
    ) -> Result<HealthCheckResult, SongbirdError> {
        if let Some(health_check) = self.health_checks.get(service_id) {
            health_check.check(service_id).await
        } else {
            Err(SongbirdError::HealthCheck {
                message: format!("No health check registered for service: {}", service_id),
            })
        }
    }

    async fn get_all_health_statuses(
        &self,
    ) -> Result<HashMap<String, HealthCheckResult>, SongbirdError> {
        let mut results = HashMap::new();

        for (service_id, health_check) in &self.health_checks {
            match health_check.check(service_id).await {
                Ok(result) => {
                    results.insert(service_id.clone(), result);
                }
                Err(e) => {
                    tracing::error!("Health check failed for service {}: {}", service_id, e);
                    results.insert(
                        service_id.clone(),
                        HealthCheckResult {
                            status: HealthStatus::Unknown,
                            message: format!("Health check error: {}", e),
                            timestamp: Utc::now(),
                            response_time: Duration::from_millis(0),
                            details: HashMap::new(),
                        },
                    );
                }
            }
        }

        Ok(results)
    }

    async fn start_monitoring(&mut self) -> Result<(), SongbirdError> {
        self.monitoring_active = true;
        // In a real implementation, this would start background tasks
        Ok(())
    }

    async fn stop_monitoring(&mut self) -> Result<(), SongbirdError> {
        self.monitoring_active = false;
        Ok(())
    }
}
