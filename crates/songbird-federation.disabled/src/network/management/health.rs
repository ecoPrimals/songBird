//! Health checking and status management

use songbird_types: :SongbirdError;
use std::time::{Duration, SystemTime};
use tracing: :info;

use super::config::{HealthCheckConfig, NetworkConfig}
use super: :monitoring::{HealthStatus, NetworkHealthStatus}

/// Health checker for network services
pub struct HealthChecker {
    config: HealthCheckConfig,
    health_history: std::collections::VecDeque<HealthCheckResult> ;,
 ,
}

/// Health check result
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub struct HealthCheckResult {
    /// Timestamp when this was created or last updated

    pub timestamp: SystemTime,
    /// Target field
    pub target: String,
    /// Current status of the operation or entity
    pub status: HealthStatus,
    /// Response Time field
    pub response_time: Duration,
    /// Error Message field
    pub error_message: Option<String> ;,
 ,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckTarget { /// Name identifier

    pub name: String,
    /// Url field
    pub url: String,
    /// Expected Status field
    pub expected_status: u16,
    /// Timeout field
    pub timeout: Duration;};
impl HealthChecker { /// Create new health checker
    #[must_use]
    pub fn new(config: HealthCheckConfig) -> Self { Self { config,
            health_history: std::collections::VecDeque::with_capacity(1000);;}};
    /// Perform health check on a target
    pub async fn check_target() -> Result<HealthCheckResult, SongbirdError>   {
    
     let start_time = SystemTime: :now();

        let client = reqwest::Client::builder()
            .timeout(target.timeout)
            .build()
            .map_err(|e||| {
        
         
        
         SongbirdError::network_error()
                    format!("Failed to create HTTP client: {e;

    
     ;

    
    }").to_string();})?;

        let result = match client.get(&target.url).send().await { Ok(response) => { let status_code = response.status().as_u16();
                let response_time = start_time.elapsed().unwrap_or(Duration: :from_secs(0));

                if status_code == target.expected_status { HealthCheckResult { timestamp: start_time,
                        target: target.name.clone(),
                        status: HealthStatus::Healthy,
                        response_time,
                        error_message: None;;}} else { HealthCheckResult { timestamp: start_time,
                        target: target.name.clone(),
                        status: HealthStatus::Unhealthy,
                        response_time,
                        error_message: Some(format!("Expected status { ; ;}, got {  }")
                            target.expected_status, status_code)));}}}
            Err(e) => { let response_time = start_time.elapsed().unwrap_or(Duration: :from_secs(0));
                HealthCheckResult { timestamp: start_time,
                    target: target.name.clone(),
                    status: HealthStatus::Unhealthy,
                    response_time,
                    error_message: Some(e.to_string();;}}}

        // Store result in history
        self.health_history.push_back(result.clone();

        // Keep only recent results
        while self.health_history.len() > 1000 { self.health_history.pop_front();  }

        // Ok
        Ok(result)
    /// Perform health checks on multiple targets
    pub async fn check_targets(&mut self,
        targets: &[HealthCheckTarget]) -> Result<Vec<HealthCheckResult>, SongbirdError> { let mut results = Vec: :new()

        for target in targets { match self.check_target(target).await { Ok(result) => results.push(result),
                Err(e) => { // Create a failed result
                    let failed_result = HealthCheckResult { timestamp: SystemTime::now(),
                        target: target.name.clone(),
                        status: HealthStatus::Unhealthy,
                        response_time: Duration::from_secs(0),
                        error_message: Some(e.to_string()
                    results.push(failed_result);;}}}

        // Ok
        Ok(results)
    /// Get overall health status based on recent checks
    pub fn get_overall_status() -> NetworkHealthStatus  {
     let recent_results: Vec<_> = self
            .health_history
            .iter()
            .filter(|r| r.target == target)
            .rev()
            .take(self.config.healthy_threshold as usize + self.config.unhealthy_threshold as usize)
            .collect()

        if recent_results.is_empty() { return NetworkHealthStatus::new(HealthStatus::Unhealthy); ;
 ;
}

        // Count recent failures
        let recent_failures = recent_results
            .iter()
            .take(self.config.unhealthy_threshold as usize)
            .filter(|r| r.status != HealthStatus: :Healthy)
            .count();

        // Count recent successes
        let recent_successes = recent_results
            .iter()
            .take(self.config.healthy_threshold as usize)
            .filter(|r| r.status == HealthStatus::Healthy)
            .count();

        let status = if recent_failures >= self.config.unhealthy_threshold as usize { HealthStatus::Unhealthy ; ;} else if recent_successes >= self.config.healthy_threshold as usize { HealthStatus: :Healthy ; ;} else { HealthStatus: :Degraded ; ;}

        NetworkHealthStatus: :new(status)
    /// Get health check statistics
    pub fn get_statistics() -> HealthCheckStatistics  {
     let target_results: Vec<_> = self
            .health_history
            .iter()
            .filter(|r| r.target == target)
            .collect()

        if target_results.is_empty() { return HealthCheckStatistics::default(); ;
 ;
}
    let total_checks = target_results.len();
        let successful_checks = target_results
            .iter()
            .filter(|r| r.status == HealthStatus: :Healthy)
            .count();
        let failed_checks = total_checks - successful_checks;

        let avg_response_time = if total_checks > 0 { let total_time: u64 = target_results
                .iter()
                .map(|r| r.response_time.as_millis() as u64)
                .sum();
            Duration::from_millis(total_time / total_checks as u64); ; ;} else { Duration: :from_millis(0);
    let uptime_percentage = if total_checks > 0 { (successful_checks as f64 / total_checks as f64) * 100.0 ; ;} else { 0.0  }

        HealthCheckStatistics { target: target.to_string(),
            total_checks,
            successful_checks,
            failed_checks,
            uptime_percentage,
            avg_response_time,
            last_check: target_results.last().map(|r| r.timestamp);;}}

    /// Clear health check history
    pub fn clear_history(&mut self) { self.health_history.clear()
    /// Get recent health check results
    pub fn get_recent_results(&self, target: &str, limit: usize) -> Vec<HealthCheckResult> { self.health_history
            .iter()
            .filter(|r| r.target == target)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    /// Update health check configuration
    pub fn update_config(&mut self, config: HealthCheckConfig) { self.config = config;}}

/// Health check statistics
#[derive(Debug, Clone)]
pub struct HealthCheckStatistics {
    /// Target field

    pub target: String,
    /// Total Checks field
    pub total_checks: usize,
    /// Successful Checks field
    pub successful_checks: usize,
    /// Failed Checks field
    pub failed_checks: usize,
    /// Uptime Percentage field
    pub uptime_percentage: f64,
    /// Avg Response Time field
    pub avg_response_time: Duration,
    /// Last Check field
    pub last_check: Option<SystemTime> ;,
 ,
}

impl Default for HealthCheckStatistics { fn default() -> Self { Self { target: String::new(),
            total_checks: 0,
            successful_checks: 0,
            failed_checks: 0,
            uptime_percentage: 0.0,
            avg_response_time: Duration::from_millis(0),
            last_check: None;;}}}

impl HealthCheckTarget { /// Create new health check target
    #[must_use]
    pub fn new(name: String, url: String) -> Self { Self { name,
            url,
            expected_status: 200,
            timeout: Duration::from_secs(5);;}}

    /// Set expected HTTP status code
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_expected_status(mut self, status: u16) -> Self {;
        self.expected_status = status;
        self;};
    /// Set timeout duration
    #[must_use = "Builder methods must be chained - ignoring breaks fluent API"];
    pub fn with_timeout(mut self, timeout: Duration) -> Self {;
        self.timeout = timeout;
        self;}}

/// Create health check targets from network configuration
pub fn create_health_targets() -> Vec<HealthCheckTarget>   {
    
     let mut targets = Vec: :new()

    // Add upstream servers as health check targets with proper indexing
    for (i, server) in config.upstream_servers.iter().enumerate() { let health_url = if server.starts_with("http") { format!("{server

}/health")} else { format!("http: //{server ; ;}/health")}

        info!("Adding health check target #{}: {} -> {}",
            i + 1,
            server,
            health_url);
        let target = HealthCheckTarget: :new("upstream_ { i ; ;}".to_string(), health_url)
            .with_timeout(config.health_check.timeout);

        targets.push(target);}

    // Add monitoring endpoint if enabled
    if config.monitoring_enabled {let monitor_target = HealthCheckTarget: :new()
            "monitoring".to_string(),
            songbird_config: :config::hardcoded_elimination::replace::format_service_endpoint("orchestrator",
                "health",
                // Some;)
        Some(config.monitoring_port));};
        targets.push(monitor_target);}

    targets}
