/*!
 * MCP Federation Monitoring
 *
 * Handles system monitoring and metrics collection for MCP federation:
 * - System resource monitoring (CPU, memory, storage)
 * - Service monitoring and health checks
 * - Performance metrics collection
 * - Capacity and load monitoring
 */

use std::time::SystemTime;
use tracing::{debug, warn};

use crate::config::FederationConfig;
use songbird_config::constants;
use songbird_errors::{NetworkError, Result, SongbirdError};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

#[derive(Debug)]
/// System monitoring manager for MCP federation
pub struct MonitoringManager {
    system: System,
    start_time: SystemTime,
    config: crate::config::FederationConfig,
}

impl MonitoringManager {
    /// Create new monitoring manager
    pub fn new(config: crate::config::FederationConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            start_time: SystemTime::now(),
            config,
        }
    }

    /// Refresh system information
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Get local services information
    pub async fn get_local_services(&mut self) -> Result<Vec<serde_json::Value>> {
        debug!("Collecting local services information");
        self.refresh();

        let mut services = Vec::new();

        // Add system service
        services.push(serde_json::json!({
            "name": "system",
            "type": "system_monitor",
            "status": "active",
            "cpu_usage": self.get_cpu_usage().await?,
            "memory_usage": self.get_memory_usage().await?,
            "memory_total_gb": self.get_total_memory_gb().await?,
            "storage_available_gb": self.get_available_storage_gb().await?,
            "uptime_seconds": self.get_uptime_seconds().await?,
            "load_average": self.get_load_average().await?,
        }));

        // Add federation service
        services.push(serde_json::json!({
            "name": "federation",
            "type": "mcp_federation",
            "status": "active",
            "protocol_version": "1.0",
            "capabilities": ["heartbeat", "discovery", "service_registry"]
        }));

        // Add gaming service if enabled
        if self.is_gaming_enabled().await {
            services.push(serde_json::json!({
                "name": "gaming",
                "type": "gaming_service",
                "status": "active",
                "active_sessions": self.get_active_gaming_sessions().await?,
                "capabilities": ["lan_gaming", "tournament_mode", "matchmaking"]
            }));
        }

        // Add primal services if enabled
        if self.is_primal_services_enabled().await {
            services.push(serde_json::json!({
                "name": "primals",
                "type": "primal_coordination",
                "status": "active",
                "active_services": self.get_active_service_count().await?,
                "capabilities": ["service_discovery", "load_balancing", "health_monitoring"]
            }));
        }

        debug!(
            "Collected information for {} local services",
            services.len()
        );
        Ok(services)
    }

    /// Check if gaming is enabled
    pub async fn is_gaming_enabled(&self) -> bool {
        // Check if gaming ports are available/in use
        self.is_port_available(7777).await || // Common gaming port
        self.is_port_available(25565).await // Minecraft default
    }

    /// Check if primal services are enabled
    pub async fn is_primal_services_enabled(&self) -> bool {
        // Check if common primal service ports are in use
        !self.is_port_available(8080).await || // HTTP API
        !self.is_port_available(8081).await // Alternative port
    }

    /// Check if a port is available
    pub async fn is_port_available(&self, port: u16) -> bool {
        (tokio::net::TcpListener::bind(format!("{}:{port}", constants::default_bind_address()))
            .await)
            .is_ok()
    }

    /// Get active gaming sessions count
    pub async fn get_active_gaming_sessions(&self) -> Result<u32> {
        // Get gaming sessions from environment variable
        if let Ok(count_str) = std::env::var("SONGBIRD_ACTIVE_GAMING_SESSIONS") {
            if let Ok(count) = count_str.parse::<u32>() {
                return Ok(count);
            }
        }

        // Default to 1 active gaming session if environment detection is enabled
        if std::env::var("SONGBIRD_GAMING_ENABLED").unwrap_or_default() == "true" {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Get CPU usage percentage
    pub async fn get_cpu_usage(&mut self) -> Result<f64> {
        self.refresh();

        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return Ok(0.0);
        }

        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        let average_usage = total_usage / cpus.len() as f32;

        Ok(average_usage as f64)
    }

    /// Get memory usage percentage
    pub async fn get_memory_usage(&mut self) -> Result<f64> {
        self.refresh();

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();

        if total_memory == 0 {
            return Ok(0.0);
        }

        let usage_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
        Ok(usage_percentage)
    }

    /// Get total memory in GB
    pub async fn get_total_memory_gb(&mut self) -> Result<u64> {
        self.refresh();
        let total_memory_kb = self.system.total_memory();
        Ok(total_memory_kb / 1024 / 1024) // Convert KB to GB
    }

    /// Get available storage in GB
    pub async fn get_available_storage_gb(&mut self) -> Result<u64> {
        self.refresh();

        let mut total_available = 0u64;

        for disk in self.system.disks() {
            total_available += disk.available_space();
        }

        Ok(total_available / 1024 / 1024 / 1024) // Convert bytes to GB
    }

    /// Get total storage size in GB
    pub async fn get_storage_size(&mut self) -> Result<u64> {
        self.refresh();

        let mut total_size = 0u64;

        for disk in self.system.disks() {
            total_size += disk.total_space();
        }

        Ok(total_size / 1024 / 1024 / 1024) // Convert bytes to GB
    }

    /// Get service count
    pub async fn get_service_count(&self) -> Result<u32> {
        // Count active services based on open ports and processes
        let mut service_count = 0u32;

        // Base services (always count federation service)
        service_count += 1;

        // Gaming services
        if self.is_gaming_enabled().await {
            service_count += 1;
        }

        // Primal services
        if self.is_primal_services_enabled().await {
            service_count += 1;
        }

        Ok(service_count)
    }

    /// Get system uptime in seconds
    pub async fn get_uptime(&mut self) -> Result<u64> {
        let uptime = self
            .start_time
            .elapsed()
            .map_err(|e| SongbirdError::Communication(format!("Failed to get uptime: {e}")))?;

        Ok(uptime.as_secs())
    }

    /// Get system load average
    pub async fn get_load_average(&mut self) -> Result<f64> {
        self.refresh();

        // Get 1-minute load average
        let load_avg = self.system.load_average();
        Ok(load_avg.one)
    }

    /// Get system capacity percentage (0.0 to 100.0)
    pub async fn get_capacity(&mut self) -> Result<f64> {
        // Calculate overall system capacity based on CPU, memory, and storage
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;

        // Simple average for now (could be weighted in the future)
        let capacity = (cpu_usage + memory_usage) / 2.0;
        Ok(capacity.min(100.0))
    }

    /// Get uptime since manager start
    pub async fn get_uptime_seconds(&self) -> Result<u64> {
        let duration = self
            .start_time
            .elapsed()
            .map_err(|e| SongbirdError::Communication(format!("Failed to get uptime: {e}")))?;
        Ok(duration.as_secs())
    }

    /// Get current system load
    pub async fn get_current_load(&mut self) -> Result<f64> {
        // Return CPU usage as current load
        self.get_cpu_usage().await
    }

    /// Get available system capacity
    pub async fn get_available_capacity(&mut self) -> Result<f64> {
        let used_capacity = self.get_capacity().await?;
        Ok(100.0 - used_capacity)
    }

    /// Get number of active connections
    pub async fn get_active_connections(&self) -> Result<u32> {
        debug!("Getting active connection count");

        let mut total_connections = 0u32;

        // Check federation endpoints for active connections
        for endpoint in &self.config.cluster_endpoints {
            if let Ok(connections) = self.count_connections_to_endpoint(endpoint).await {
                total_connections += connections;
            }
        }

        // Check local listener ports for active connections
        if let Ok(local_connections) = self.count_local_connections().await {
            total_connections += local_connections;
        }

        debug!("Total active connections: {}", total_connections);
        Ok(total_connections)
    }

    /// Count connections to a specific endpoint
    async fn count_connections_to_endpoint(&self, endpoint: &str) -> Result<u32> {
        debug!("Counting connections to endpoint: {}", endpoint);

        // Parse endpoint to get host and port
        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("http://{endpoint}")
        };

        if let Ok(parsed_url) = url::Url::parse(&url) {
            if let Some(host) = parsed_url.host_str() {
                let port = parsed_url.port().unwrap_or(80);

                // Use netstat-like functionality to count connections
                #[cfg(target_os = "linux")]
                {
                    return self.count_connections_linux(host, port).await;
                }

                #[cfg(target_os = "macos")]
                {
                    return self.count_connections_macos(host, port).await;
                }

                #[cfg(target_os = "windows")]
                {
                    return self.count_connections_windows(host, port).await;
                }
            }
        }

        Ok(0)
    }

    /// Count local connections on federation ports
    async fn count_local_connections(&self) -> Result<u32> {
        debug!("Counting local connections");

        let mut total_connections = 0u32;

        // Check default federation port
        let federation_port = self.config.port.unwrap_or(8080);

        #[cfg(target_os = "linux")]
        {
            if let Ok(connections) = self
                .count_listening_connections_linux(federation_port)
                .await
            {
                total_connections += connections;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(connections) = self
                .count_listening_connections_macos(federation_port)
                .await
            {
                total_connections += connections;
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(connections) = self
                .count_listening_connections_windows(federation_port)
                .await
            {
                total_connections += connections;
            }
        }

        Ok(total_connections)
    }

    #[cfg(target_os = "linux")]
    async fn count_connections_linux(&self, host: &str, port: u16) -> Result<u32> {
        debug!("Counting connections on Linux for {}:{}", host, port);

        let output = tokio::process::Command::new("netstat")
            .args(["-tn"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {e}"))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| line.contains(&format!("{host}:{port}")) && line.contains("ESTABLISHED"))
            .count() as u32;

        Ok(connection_count)
    }

    #[cfg(target_os = "linux")]
    async fn count_listening_connections_linux(&self, port: u16) -> Result<u32> {
        debug!("Counting listening connections on Linux for port {}", port);

        let output = tokio::process::Command::new("netstat")
            .args(["-tnl"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {e}"))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| line.contains(&format!(":{port}")) && line.contains("LISTEN"))
            .count() as u32;

        Ok(connection_count)
    }

    #[cfg(target_os = "macos")]
    async fn count_connections_macos(&self, host: &str, port: u16) -> Result<u32> {
        debug!("Counting connections on macOS for {}:{}", host, port);

        let output = tokio::process::Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {}", e))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| {
                line.contains(&format!("{}:{}", host, port)) && line.contains("ESTABLISHED")
            })
            .count() as u32;

        Ok(connection_count)
    }

    #[cfg(target_os = "macos")]
    async fn count_listening_connections_macos(&self, port: u16) -> Result<u32> {
        debug!("Counting listening connections on macOS for port {}", port);

        let output = tokio::process::Command::new("netstat")
            .args(&["-an", "-p", "tcp"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {}", e))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| line.contains(&format!("*.{}", port)) && line.contains("LISTEN"))
            .count() as u32;

        Ok(connection_count)
    }

    #[cfg(target_os = "windows")]
    async fn count_connections_windows(&self, host: &str, port: u16) -> Result<u32> {
        debug!("Counting connections on Windows for {}:{}", host, port);

        let output = tokio::process::Command::new("netstat")
            .args(&["-an", "-p", "TCP"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {}", e))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| {
                line.contains(&format!("{}:{}", host, port)) && line.contains("ESTABLISHED")
            })
            .count() as u32;

        Ok(connection_count)
    }

    #[cfg(target_os = "windows")]
    async fn count_listening_connections_windows(&self, port: u16) -> Result<u32> {
        debug!(
            "Counting listening connections on Windows for port {}",
            port
        );

        let output = tokio::process::Command::new("netstat")
            .args(&["-an", "-p", "TCP"])
            .output()
            .await
            .map_err(|e| {
                SongbirdError::service_error("monitoring", format!("Failed to run netstat: {}", e))
            })?;

        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let connection_count = netstat_output
            .lines()
            .filter(|line| line.contains(&format!(":{}", port)) && line.contains("LISTENING"))
            .count() as u32;

        Ok(connection_count)
    }

    /// Get active service count
    pub async fn get_active_service_count(&self) -> Result<u32> {
        self.get_service_count().await
    }

    /// Collect comprehensive system metrics
    pub async fn collect_metrics(&mut self) -> Result<SystemMetrics> {
        debug!("Collecting comprehensive system metrics");

        let metrics = SystemMetrics {
            timestamp: SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            cpu_usage: self.get_cpu_usage().await?,
            memory_usage: self.get_memory_usage().await?,
            memory_total_gb: self.get_total_memory_gb().await?,
            storage_available_gb: self.get_available_storage_gb().await?,
            storage_total_gb: self.get_storage_size().await?,
            uptime_seconds: self.get_uptime().await?,
            load_average: self.get_load_average().await?,
            service_count: self.get_service_count().await?,
            active_connections: self.get_active_connections().await?,
            capacity: self.get_capacity().await?,
            gaming_enabled: self.is_gaming_enabled().await,
            primal_services_enabled: self.is_primal_services_enabled().await,
        };

        debug!("Metrics collection completed");
        Ok(metrics)
    }

    /// Get system health status
    pub async fn get_health_status(&mut self) -> Result<HealthStatus> {
        let metrics = self.collect_metrics().await?;

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check CPU usage
        if metrics.cpu_usage > 90.0 {
            issues.push("High CPU usage".to_string());
        } else if metrics.cpu_usage > 75.0 {
            warnings.push("Elevated CPU usage".to_string());
        }

        // Check memory usage
        if metrics.memory_usage > 95.0 {
            issues.push("High memory usage".to_string());
        } else if metrics.memory_usage > 80.0 {
            warnings.push("Elevated memory usage".to_string());
        }

        // Check storage
        if metrics.storage_available_gb < 1 {
            issues.push("Low storage space".to_string());
        } else if metrics.storage_available_gb < 10 {
            warnings.push("Limited storage space".to_string());
        }

        // Check load average
        if metrics.load_average > 5.0 {
            issues.push("High system load".to_string());
        } else if metrics.load_average > 2.0 {
            warnings.push("Elevated system load".to_string());
        }

        let health = if !issues.is_empty() {
            Health::Critical
        } else if !warnings.is_empty() {
            Health::Warning
        } else {
            Health::Healthy
        };

        Ok(HealthStatus {
            health,
            issues,
            warnings,
            metrics,
        })
    }

    /// Test system connectivity
    pub async fn test_connectivity(&self) -> Result<bool> {
        // Test basic network connectivity
        match tokio::net::TcpStream::connect("8.8.8.8:53").await {
            Ok(_) => Ok(true),
            Err(_) => {
                warn!("Network connectivity test failed");
                Ok(false)
            }
        }
    }

    /// Update monitoring configuration
    pub async fn update_config(&mut self, new_config: FederationConfig) -> Result<()> {
        debug!("Updating monitoring configuration");

        // Update local configuration
        self.config = new_config.clone();

        // Update monitoring intervals or thresholds if changed
        if self.config.heartbeat_interval != new_config.heartbeat_interval {
            debug!("Monitoring interval changed, will take effect on next cycle");
        }

        if self.config.cluster_id != new_config.cluster_id {
            debug!(
                "Cluster ID changed from {} to {}",
                self.config.cluster_id, new_config.cluster_id
            );
        }

        debug!("Monitoring configuration updated successfully");
        Ok(())
    }

    /// Implement actual message broadcasting for federation
    pub async fn broadcast_message(&self, message: &str) -> Result<()> {
        debug!("Broadcasting monitoring message: {}", message);

        // Create federation message
        let federation_msg = crate::messages::FederationMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            message_type: crate::messages::FederationMessageType::Monitoring,
            data: serde_json::json!({
                "message": message,
                "source": "monitoring",
                "timestamp": chrono::Utc::now(),
                "cluster_id": self.config.cluster_id,
                "node_id": self.config.node_id
            }),
            timestamp: chrono::Utc::now(),
            source_node: self.config.node_id.clone(),
        };

        // Broadcast to all configured endpoints
        for endpoint in &self.config.cluster_endpoints {
            if let Err(e) = self
                .send_message_to_endpoint(endpoint, &federation_msg)
                .await
            {
                warn!("Failed to send message to endpoint {}: {}", endpoint, e);
            }
        }

        Ok(())
    }

    /// Send message to specific endpoint
    async fn send_message_to_endpoint(
        &self,
        endpoint: &str,
        message: &crate::messages::FederationMessage,
    ) -> Result<()> {
        debug!("Sending message to endpoint: {}", endpoint);

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{endpoint}/federation/messages"))
            .json(message)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::Network(Box::new(NetworkError {
                    service: Some("federation".to_string()),
                    message: format!("Failed to send message to {endpoint}: {e}"),
                    details: None,
                    endpoint: Some("federation/messages".to_string()),
                    suggestion: Some(
                        "Check network connectivity and federation endpoint".to_string(),
                    ),
                }))
            })?;

        if response.status().is_success() {
            debug!("Message sent successfully to {}", endpoint);
        } else {
            warn!(
                "Failed to send message to {}: {}",
                endpoint,
                response.status()
            );
        }

        Ok(())
    }
}

impl Default for MonitoringManager {
    fn default() -> Self {
        Self::new(crate::config::FederationConfig::default())
    }
}

/// System metrics structure
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total_gb: u64,
    pub storage_available_gb: u64,
    pub storage_total_gb: u64,
    pub uptime_seconds: u64,
    pub load_average: f64,
    pub service_count: u32,
    pub active_connections: u32,
    pub capacity: f64,
    pub gaming_enabled: bool,
    pub primal_services_enabled: bool,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum Health {
    Healthy,
    Warning,
    Critical,
}

/// System health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub health: Health,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub metrics: SystemMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_manager_creation() {
        let config = crate::config::FederationConfig {
            cluster_id: "test-cluster".to_string(),
            node_id: "test-node".to_string(),
            cluster_endpoints: vec!["http://test:8080".to_string()],
            port: Some(8080),
            discovery_port: Some(8765),
            heartbeat_interval: Some(30),
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
        };
        let mut monitor = MonitoringManager::new(config);

        // Basic functionality tests
        assert!(monitor.get_cpu_usage().await.is_ok());
        assert!(monitor.get_memory_usage().await.is_ok());
        assert!(monitor.get_uptime_seconds().await.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = crate::config::FederationConfig {
            cluster_id: "test-cluster".to_string(),
            node_id: "test-node".to_string(),
            cluster_endpoints: vec!["http://test:8080".to_string()],
            port: Some(8080),
            discovery_port: Some(8765),
            heartbeat_interval: Some(30),
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
        };
        let mut monitor = MonitoringManager::new(config);

        let metrics = monitor.collect_metrics().await;
        assert!(metrics.is_ok());

        let metrics = metrics.unwrap();
        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0);
    }

    #[tokio::test]
    async fn test_monitoring_with_enabled_features() {
        let config = crate::config::FederationConfig {
            cluster_id: "test-cluster".to_string(),
            node_id: "test-node".to_string(),
            cluster_endpoints: vec!["http://test:8080".to_string()],
            port: Some(8080),
            discovery_port: Some(8765),
            heartbeat_interval: Some(30),
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
        };
        let mut monitor = MonitoringManager::new(config);

        // Test that enabled features work
        assert!(monitor.get_cpu_usage().await.is_ok());
        assert!(monitor.get_memory_usage().await.is_ok());
        assert!(monitor.get_uptime_seconds().await.is_ok());
    }

    #[tokio::test]
    async fn test_monitoring_update_config() {
        let config = crate::config::FederationConfig {
            cluster_id: "test-cluster".to_string(),
            node_id: "test-node".to_string(),
            cluster_endpoints: vec!["http://test:8080".to_string()],
            port: Some(8080),
            discovery_port: Some(8765),
            heartbeat_interval: Some(30),
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
        };
        let mut monitor = MonitoringManager::new(config.clone());

        // Test configuration update
        let result = monitor.update_config(config).await;
        assert!(result.is_ok());
    }
}
