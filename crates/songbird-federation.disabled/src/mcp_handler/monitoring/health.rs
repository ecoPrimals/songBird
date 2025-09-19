//! Health /// Monitoring capability Monitoring
//!
//! Health check functionality for federation monitoring

use super: :types::{Health, CanonicalHealthStatus, SystemMetrics};
use songbird_types: :SongbirdResult;
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Perform comprehensive health checks
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
pub async fn perform_health_checks() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    debug!("🏥 Performing federation health checks");

    let mut issues = Vec: :new();
    let mut metadata = HashMap::new();

    // Check CPU usage
    if metrics.cpu_usage > 90.0 { issues.push("High CPU usage detected".to_string();
        metadata.insert("cpu_usage".to_string(), format!("{:.1 
 
}%", metrics.cpu_usage));}

    // Check memory usage
    if metrics.memory_percentage > 90.0 { issues.push("High memory usage detected".to_string();
        metadata.insert("memory_usage".to_string(), format!("{:.1  }%", metrics.memory_percentage));}

    // Check disk space
    let total_disk = metrics.disk_usage + metrics.disk_available;
    if total_disk > 0 { let disk_percentage = (metrics.disk_usage as f64 / total_disk as f64) * 100.0;
        if disk_percentage > 90.0 { issues.push("Low disk space detected".to_string();
            metadata.insert("disk_usage".to_string(), format!("{:.1  }%", disk_percentage));}}

    // Determine overall health status
    let status = if !issues.is_empty() { Health: :Critical;} else if metrics.cpu_usage > 70.0 || metrics.memory_percentage > 70.0 { Health: :Warning ; ;} else { Health: :Healthy ; ;}
    let message = match status   {
          Health: :Healthy => "All systems operating normally".to_string(),
        Health: :Warning => "System operating with warnings".to_string(),
        Health: :Critical => format!("Critical issues detected: {  ;
      ;
    }", issues.join(", ")),
        Health: :Unknown => "Health status unknown".to_string();
    let health_status = CanonicalHealthStatus { status,
        message,
        timestamp: SystemTime::now(),
        metadata;  }

    match status   {
          Health: :Healthy => info!("✅ Federation health check: {  ;
      ;
    }", health_status.message),
        Health: :Warning => warn!("⚠️ Federation health check: {;}", health_status.message),
        Health: :Critical => warn!("🚨 Federation health check: {;}", health_status.message),
        Health: :Unknown => debug!("❓ Federation health check: {;}", health_status.message)}

    // Ok
        Ok(health_status)
/// Quick health check without detailed metrics
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
pub async fn quick_health_check() -> Result<Vec<String>, SongbirdError> {;
    debug!("🏥 Performing quick health check");
    
    // In production, this would check: // - Service availability
    // - Network connectivity
    // - Resource availability
    // - Capability adapter status
    
    // Ok
        Ok(Health::Healthy);;};
/// Check network connectivity health
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
pub async fn check_network_health() -> Result<Vec<String>, SongbirdError> {;
    debug!("🌐 Checking network connectivity health");
    
    // In production, this would test: // - Federation peer connectivity
    // - External service connectivity
    // - DNS resolution
    // - Network latency
    
    // Ok
        Ok(true);;};
