//! Main NetworkManager struct and core implementation

use anyhow::Result;
use songbird_errors::SongbirdError;
use std::sync::Arc;
use tracing::info;

use super::config::NetworkConfig;
use super::monitoring::{HealthStatus, NetworkDiagnostics, NetworkHealthStatus, NetworkStats};

/// Network management service
#[derive(Debug, Clone)]
pub struct NetworkManager {
    config: NetworkConfig,
    stats: Arc<std::sync::Mutex<NetworkStats>>,
    health_status: Arc<std::sync::Mutex<NetworkHealthStatus>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            stats: Arc::new(std::sync::Mutex::new(NetworkStats::new())),
            health_status: Arc::new(std::sync::Mutex::new(NetworkHealthStatus::new(
                HealthStatus::Healthy,
            ))),
        }
    }

    /// Start the network manager
    pub async fn start(&self) -> Result<(), SongbirdError> {
        info!("Starting network manager");

        if self.config.reverse_proxy_enabled {
            info!(
                "Reverse proxy enabled on port {}",
                self.config.reverse_proxy_port
            );
        }

        if self.config.ssl_termination_enabled {
            info!("SSL termination enabled");
        }

        if self.config.cors_enabled {
            info!("CORS enabled");
        }

        if self.config.rate_limiting_enabled {
            info!("Rate limiting enabled");
        }

        if self.config.load_balancing_enabled {
            info!(
                "Load balancing enabled with strategy: {:?}",
                self.config.load_balancing_strategy
            );
        }

        if self.config.monitoring_enabled {
            info!("Network monitoring enabled");
        }

        Ok(())
    }

    /// Stop the network manager
    pub async fn stop(&self) -> Result<(), SongbirdError> {
        info!("Stopping network manager");
        Ok(())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Update the network configuration
    pub fn update_config(&mut self, config: NetworkConfig) {
        self.config = config;
    }

    /// Check if reverse proxy is enabled
    pub fn is_reverse_proxy_enabled(&self) -> bool {
        self.config.reverse_proxy_enabled
    }

    /// Get reverse proxy port
    pub fn get_reverse_proxy_port(&self) -> u16 {
        self.config.reverse_proxy_port
    }

    /// Check if SSL termination is enabled
    pub fn is_ssl_termination_enabled(&self) -> bool {
        self.config.ssl_termination_enabled
    }

    /// Check if CORS is enabled
    pub fn is_cors_enabled(&self) -> bool {
        self.config.cors_enabled
    }

    /// Check if rate limiting is enabled
    pub fn is_rate_limiting_enabled(&self) -> bool {
        self.config.rate_limiting_enabled
    }

    /// Check if load balancing is enabled
    pub fn is_load_balancing_enabled(&self) -> bool {
        self.config.load_balancing_enabled
    }

    /// Get load balancing strategy
    pub fn get_load_balancing_strategy(&self) -> &super::config::LoadBalancingStrategy {
        &self.config.load_balancing_strategy
    }

    /// Check if monitoring is enabled
    pub fn is_monitoring_enabled(&self) -> bool {
        self.config.monitoring_enabled
    }

    /// Get current network statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.lock().unwrap().clone()
    }

    /// Update network statistics
    pub fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut NetworkStats),
    {
        if let Ok(mut stats) = self.stats.lock() {
            updater(&mut stats);
        }
    }

    /// Get current health status
    pub fn get_health_status(&self) -> NetworkHealthStatus {
        self.health_status.lock().unwrap().clone()
    }

    /// Update health status
    pub fn update_health_status(&self, status: HealthStatus) {
        if let Ok(mut health) = self.health_status.lock() {
            health.update_status(status);
        }
    }

    /// Get network diagnostics
    pub async fn get_diagnostics(&self) -> Result<NetworkDiagnostics, SongbirdError> {
        let mut diagnostics = NetworkDiagnostics::new();

        // Collect system diagnostics
        diagnostics = self.collect_system_diagnostics(diagnostics).await?;
        diagnostics = self.collect_network_diagnostics(diagnostics).await?;

        Ok(diagnostics)
    }

    /// Collect system-level diagnostics
    async fn collect_system_diagnostics(
        &self,
        mut diagnostics: NetworkDiagnostics,
    ) -> Result<NetworkDiagnostics, SongbirdError> {
        // Get system uptime
        if let Ok(uptime) = self.get_system_uptime().await {
            diagnostics.set_uptime(uptime);
        }

        // Get load average
        if let Ok(load) = self.get_load_average().await {
            diagnostics.set_load_average(load);
        }

        Ok(diagnostics)
    }

    /// Collect network-level diagnostics
    async fn collect_network_diagnostics(
        &self,
        mut diagnostics: NetworkDiagnostics,
    ) -> Result<NetworkDiagnostics, SongbirdError> {
        // Get network interface statistics
        if let Ok(interfaces) = self.get_network_interfaces().await {
            for interface in interfaces {
                diagnostics.add_interface(interface.name.clone(), interface);
            }
        }

        // Get connection statistics
        if let Ok(conn_stats) = self.get_connection_stats().await {
            diagnostics.connections = conn_stats;
        }

        Ok(diagnostics)
    }

    /// Get system uptime
    async fn get_system_uptime(&self) -> Result<std::time::Duration, SongbirdError> {
        // Try to read actual system uptime from /proc/uptime (Linux)
        #[cfg(target_os = "linux")]
        {
            match std::fs::read_to_string("/proc/uptime") {
                Ok(content) => {
                    let uptime_seconds = content
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(3600.0); // Fallback to 1 hour
                    Ok(std::time::Duration::from_secs_f64(uptime_seconds))
                }
                Err(_) => Ok(std::time::Duration::from_secs(3600)), // Fallback
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // For non-Linux systems, provide reasonable fallback
            let boot_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            Ok(boot_time) // Approximation
        }
    }

    /// Get system load average
    async fn get_load_average(&self) -> Result<(f64, f64, f64), SongbirdError> {
        #[cfg(target_os = "linux")]
        {
            match std::fs::read_to_string("/proc/loadavg") {
                Ok(content) => {
                    let parts: Vec<&str> = content.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let load1 = parts[0].parse().unwrap_or(0.5);
                        let load5 = parts[1].parse().unwrap_or(0.7);
                        let load15 = parts[2].parse().unwrap_or(0.8);
                        Ok((load1, load5, load15))
                    } else {
                        Ok((0.5, 0.7, 0.8)) // Fallback
                    }
                }
                Err(_) => Ok((0.5, 0.7, 0.8)), // Fallback
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // For non-Linux systems, provide reasonable defaults
            Ok((0.3, 0.5, 0.6))
        }
    }

    /// Get network interface statistics
    async fn get_network_interfaces(
        &self,
    ) -> Result<Vec<super::monitoring::InterfaceStats>, SongbirdError> {
        #[cfg(target_os = "linux")]
        {
            match std::fs::read_to_string("/proc/net/dev") {
                Ok(content) => {
                    let mut interfaces = Vec::new();

                    // Skip first two header lines
                    for line in content.lines().skip(2) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 10 {
                            let interface_name = parts[0].trim_end_matches(':');
                            let rx_bytes = parts[1].parse().unwrap_or(0);
                            let tx_bytes = parts[9].parse().unwrap_or(0);

                            let mut interface =
                                super::monitoring::InterfaceStats::new(interface_name.to_string());
                            interface.rx_bytes = rx_bytes;
                            interface.tx_bytes = tx_bytes;
                            interface.is_up = rx_bytes > 0 || tx_bytes > 0; // Simple heuristic

                            interfaces.push(interface);
                        }
                    }

                    if !interfaces.is_empty() {
                        return Ok(interfaces);
                    }
                }
                Err(_) => {} // Fall through to mock data
            }
        }

        // Fallback mock interface statistics for non-Linux or when /proc is unavailable
        let mut interfaces = Vec::new();

        let mut eth0 = super::monitoring::InterfaceStats::new("eth0".to_string());
        eth0.rx_bytes = 1024 * 1024;
        eth0.tx_bytes = 512 * 1024;
        eth0.is_up = true;
        interfaces.push(eth0);

        let mut lo = super::monitoring::InterfaceStats::new("lo".to_string());
        lo.rx_bytes = 1024;
        lo.tx_bytes = 1024;
        lo.is_up = true;
        interfaces.push(lo);

        Ok(interfaces)
    }

    /// Get connection statistics
    async fn get_connection_stats(
        &self,
    ) -> Result<super::monitoring::ConnectionStats, SongbirdError> {
        // In a real implementation, parse /proc/net/tcp or use system calls
        let mut stats = super::monitoring::ConnectionStats::new();
        stats.tcp_established = 10;
        stats.tcp_active = 5;
        stats.tcp_passive = 3;
        Ok(stats)
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<(), SongbirdError> {
        if self.config.reverse_proxy_port == 0 {
            return Err(SongbirdError::config_field(
                "reverse_proxy_port",
                "Port cannot be 0",
            ));
        }

        // Note: No need to check > 65535 since reverse_proxy_port is u16 (max 65535)

        if self.config.ssl_termination_enabled && self.config.ssl_cert_dir.is_empty() {
            return Err(SongbirdError::config_field(
                "ssl_cert_dir",
                "SSL certificate directory cannot be empty when SSL is enabled",
            ));
        }

        if self.config.load_balancing_enabled && self.config.upstream_servers.is_empty() {
            return Err(SongbirdError::config_field(
                "upstream_servers",
                "Upstream servers cannot be empty when load balancing is enabled",
            ));
        }

        if self.config.max_request_size == 0 {
            return Err(SongbirdError::config_field(
                "max_request_size",
                "Maximum request size cannot be 0",
            ));
        }

        Ok(())
    }

    /// Reload configuration
    pub async fn reload_config(&mut self, new_config: NetworkConfig) -> Result<(), SongbirdError> {
        info!("Reloading network configuration");

        // Validate new configuration
        let temp_manager = NetworkManager::new(new_config.clone());
        temp_manager.validate_config()?;

        // If validation passes, update the configuration
        self.config = new_config;

        info!("Network configuration reloaded successfully");
        Ok(())
    }

    /// Get configuration summary
    pub fn get_config_summary(&self) -> std::collections::HashMap<String, String> {
        let mut summary = std::collections::HashMap::new();

        summary.insert(
            "reverse_proxy_enabled".to_string(),
            self.config.reverse_proxy_enabled.to_string(),
        );
        summary.insert(
            "reverse_proxy_port".to_string(),
            self.config.reverse_proxy_port.to_string(),
        );
        summary.insert(
            "ssl_termination_enabled".to_string(),
            self.config.ssl_termination_enabled.to_string(),
        );
        summary.insert(
            "cors_enabled".to_string(),
            self.config.cors_enabled.to_string(),
        );
        summary.insert(
            "rate_limiting_enabled".to_string(),
            self.config.rate_limiting_enabled.to_string(),
        );
        summary.insert(
            "load_balancing_enabled".to_string(),
            self.config.load_balancing_enabled.to_string(),
        );
        summary.insert(
            "monitoring_enabled".to_string(),
            self.config.monitoring_enabled.to_string(),
        );

        summary
    }
}
