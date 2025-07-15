/*!
 * MCP Federation Heartbeat
 *
 * Handles heartbeat management for MCP federation:
 * - Heartbeat task scheduling
 * - Connection monitoring
 * - Endpoint connectivity testing
 * - Departure notifications
 */

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use sysinfo::{System, SystemExt, CpuExt};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

use crate::config::FederationConfig;
use crate::messages::FederationMessage;
use crate::types::*;
use songbird_errors::{Result, SongbirdError};

#[derive(Debug)]
/// Heartbeat manager for MCP federation
pub struct HeartbeatManager {
    config: FederationConfig,
    running: Arc<RwLock<bool>>,
    heartbeat_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl HeartbeatManager {
    /// Create new heartbeat manager
    pub fn new(config: FederationConfig) -> Self {
        Self {
            config,
            running: Arc::new(RwLock::new(false)),
            heartbeat_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start heartbeat task
    pub async fn start_heartbeat_task(&self) -> Result<()> {
        info!("Starting MCP federation heartbeat task");

        {
            let running = self.running.read().await;
            if *running {
                return Ok(());
            }
        }

        let heartbeat_interval =
            Duration::from_secs(self.config.heartbeat_interval.unwrap_or(30) as u64);

        let cluster_id = self.config.cluster_id.clone();
        let node_id = self.config.node_id.clone();
        let endpoints = self.config.cluster_endpoints.clone();
        let running = Arc::clone(&self.running);

        // Set running flag before spawning task
        {
            let mut running_guard = self.running.write().await;
            *running_guard = true;
        }

        // Start heartbeat task
        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(heartbeat_interval);

            info!(
                "Heartbeat task started with interval: {:?}",
                heartbeat_interval
            );

            loop {
                // Check if still running
                {
                    let running_guard = running.read().await;
                    if !*running_guard {
                        info!("Heartbeat task stopping");
                        break;
                    }
                }

                // Wait for next interval
                interval_timer.tick().await;

                // Send heartbeats to all endpoints
                for endpoint in &endpoints {
                    if let Err(e) =
                        Self::send_heartbeat_to_endpoint(endpoint, &cluster_id, &node_id).await
                    {
                        warn!("Failed to send heartbeat to {}: {}", endpoint, e);
                    }
                }

                debug!(
                    "Heartbeat cycle completed for {} endpoints",
                    endpoints.len()
                );
            }

            info!("Heartbeat task ended");
        });

        // Store the handle
        {
            let mut handle_guard = self.heartbeat_handle.write().await;
            *handle_guard = Some(handle);
        }

        Ok(())
    }

    /// Stop heartbeat task
    pub async fn stop_heartbeat_task(&self) {
        info!("Stopping MCP federation heartbeat task");

        // Signal task to stop
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Wait for task to complete
        {
            let mut handle_guard = self.heartbeat_handle.write().await;
            if let Some(handle) = handle_guard.take() {
                if let Err(e) = handle.await {
                    error!("Error waiting for heartbeat task to complete: {}", e);
                } else {
                    info!("Heartbeat task stopped successfully");
                }
            }
        }
    }

    /// Send heartbeat to specific endpoint
    pub async fn send_heartbeat_to_endpoint(
        endpoint: &str,
        cluster_id: &str,
        node_id: &str,
    ) -> Result<()> {
        debug!("Sending heartbeat to endpoint: {}", endpoint);

        let heartbeat_payload = serde_json::json!({
            "type": "heartbeat",
            "cluster_id": cluster_id,
            "node_id": node_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "protocol_version": "1.0",
            "status": "active"
        });

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SongbirdError::service_error(
                    "heartbeat",
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        let heartbeat_url = format!("{endpoint}/federation/heartbeat");

        let response = timeout(
            Duration::from_secs(5),
            client.post(&heartbeat_url).json(&heartbeat_payload).send(),
        )
        .await
        .map_err(|_| {
            SongbirdError::service_error("heartbeat", "Heartbeat request timed out".to_string())
        })?
        .map_err(|e| {
            SongbirdError::service_error("heartbeat", format!("Heartbeat request failed: {e}"))
        })?;

        if response.status().is_success() {
            debug!("Heartbeat sent successfully to {}", endpoint);

            // Try to parse response for additional info
            if let Ok(response_body) = response.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_body) {
                    if let Some(node_count) = json.get("node_count").and_then(|v| v.as_u64()) {
                        debug!(
                            "Remote endpoint {} reports {} nodes in cluster",
                            endpoint, node_count
                        );
                    }
                }
            }
        } else {
            return Err(SongbirdError::service_error(
                "heartbeat",
                format!("Heartbeat failed with status: {}", response.status()),
            ));
        }

        Ok(())
    }

    /// Test connectivity to an endpoint
    pub async fn test_endpoint_connectivity(&self, endpoint: &str) -> Result<()> {
        debug!("Testing connectivity to endpoint: {}", endpoint);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SongbirdError::service_error(
                    "heartbeat",
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        // Test basic connectivity with status endpoint
        let status_url = format!("{endpoint}/federation/status");

        let response = timeout(Duration::from_secs(5), client.get(&status_url).send())
            .await
            .map_err(|_| {
                SongbirdError::service_error("heartbeat", "Connection test timed out".to_string())
            })?
            .map_err(|e| {
                SongbirdError::service_error("heartbeat", format!("Connection test failed: {e}"))
            })?;

        if response.status().is_success() {
            debug!("Successfully connected to {}", endpoint);

            // Validate response is from a Songbird federation endpoint
            if let Ok(response_body) = response.text().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_body) {
                    if json.get("cluster_id").is_some() && json.get("node_id").is_some() {
                        info!("Validated Songbird federation endpoint: {}", endpoint);
                        return Ok(());
                    }
                }
            }

            Err(SongbirdError::service_error(
                "heartbeat",
                "Endpoint is not a valid Songbird federation endpoint".to_string(),
            ))
        } else {
            Err(SongbirdError::service_error(
                "heartbeat",
                format!("Connection test failed with status: {}", response.status()),
            ))
        }
    }

    /// Send departure notification to endpoint
    pub async fn send_departure_notification(&self, endpoint: &str) -> Result<()> {
        info!("Sending departure notification to: {}", endpoint);

        let departure_payload = serde_json::json!({
            "type": "departure",
            "cluster_id": self.config.cluster_id,
            "node_id": self.config.node_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "reason": "graceful_shutdown"
        });

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                SongbirdError::service_error(
                    "heartbeat",
                    format!("Failed to create HTTP client: {e}"),
                )
            })?;

        let departure_url = format!("{endpoint}/federation/departure");

        let response = timeout(
            Duration::from_secs(5),
            client.post(&departure_url).json(&departure_payload).send(),
        )
        .await
        .map_err(|_| {
            SongbirdError::service_error(
                "heartbeat",
                "Departure notification timed out".to_string(),
            )
        })?
        .map_err(|e| {
            SongbirdError::service_error("heartbeat", format!("Departure notification failed: {e}"))
        })?;

        if response.status().is_success() {
            info!("Departure notification sent successfully to {}", endpoint);
        } else {
            warn!(
                "Departure notification failed with status: {}",
                response.status()
            );
        }

        Ok(())
    }

    /// Send heartbeat to all configured endpoints
    pub async fn send_heartbeat_to_all(&self) -> Result<()> {
        debug!("Sending heartbeat to all federation endpoints");

        let mut successful_heartbeats = 0;
        let mut failed_heartbeats = 0;

        // Create heartbeat message with current node status
        let heartbeat_message = self.create_heartbeat_message().await?;

        // Send heartbeat to all configured endpoints
        for endpoint in &self.config.cluster_endpoints {
            match self.send_heartbeat_data_to_endpoint(endpoint, &heartbeat_message).await {
                Ok(_) => {
                    successful_heartbeats += 1;
                    debug!("Successfully sent heartbeat to: {}", endpoint);
                }
                Err(e) => {
                    failed_heartbeats += 1;
                    warn!("Failed to send heartbeat to {}: {}", endpoint, e);
                }
            }
        }

        info!(
            "Heartbeat cycle completed: {} successful, {} failed",
            successful_heartbeats, failed_heartbeats
        );

        if successful_heartbeats > 0 {
            Ok(())
        } else {
            Err(SongbirdError::Network {
                service: Some("federation".to_string()),
                message: "All heartbeat attempts failed".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and endpoint configuration".to_string()),
            })
        }
    }

    /// Create heartbeat message with current node status
    async fn create_heartbeat_message(&self) -> Result<serde_json::Value> {
        // Collect system metrics
        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let cpu_usage = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / system.cpus().len() as f32;
        let memory_usage = (system.used_memory() as f64 / system.total_memory() as f64) * 100.0;
        let uptime = system.uptime();

        // Create heartbeat payload
        let heartbeat_data = serde_json::json!({
            "node_id": self.config.node_id,
            "cluster_id": self.config.cluster_id,
            "timestamp": chrono::Utc::now(),
            "status": "online",
            "metrics": {
                "cpu_usage": cpu_usage,
                "memory_usage": memory_usage,
                "uptime_seconds": uptime,
                "total_memory_gb": system.total_memory() / 1024 / 1024 / 1024,
                "available_memory_gb": system.available_memory() / 1024 / 1024 / 1024,
            },
            "capabilities": [
                "service_discovery",
                "heartbeat",
                "federation",
                "load_balancing"
            ],
            "version": env!("CARGO_PKG_VERSION"),
            "protocol_version": "1.0"
        });

        Ok(heartbeat_data)
    }

    /// Send heartbeat to a specific endpoint
    async fn send_heartbeat_data_to_endpoint(&self, endpoint: &str, heartbeat_data: &serde_json::Value) -> Result<()> {
        let client = reqwest::Client::new();
        let heartbeat_url = format!("{}/federation/heartbeat", endpoint);

        let response = tokio::time::timeout(
            Duration::from_secs(5),
            client.post(&heartbeat_url)
                .json(heartbeat_data)
                .send()
        ).await;

        match response {
            Ok(Ok(resp)) => {
                if resp.status().is_success() {
                    debug!("Heartbeat sent successfully to: {}", endpoint);
                    Ok(())
                } else {
                    Err(SongbirdError::Network {
                        service: Some("federation".to_string()),
                        message: format!("Heartbeat failed with status: {}", resp.status()),
                        details: None,
                        endpoint: Some(endpoint.to_string()),
                        suggestion: Some("Check federation endpoint health".to_string()),
                    })
                }
            }
            Ok(Err(e)) => {
                Err(SongbirdError::Network {
                    service: Some("federation".to_string()),
                    message: format!("Failed to send heartbeat: {}", e),
                    details: None,
                    endpoint: Some(endpoint.to_string()),
                    suggestion: Some("Check network connectivity".to_string()),
                })
            }
            Err(_) => {
                Err(SongbirdError::Network {
                    service: Some("federation".to_string()),
                    message: "Heartbeat request timed out".to_string(),
                    details: None,
                    endpoint: Some(endpoint.to_string()),
                    suggestion: Some("Check network connectivity and endpoint responsiveness".to_string()),
                })
            }
        }
    }

    /// Send heartbeat to a specific endpoint
    pub async fn send_heartbeat_to_endpoint_public(&self, endpoint: &str) -> Result<()> {
        let heartbeat_data = self.create_heartbeat_message().await?;
        self.send_heartbeat_data_to_endpoint(endpoint, &heartbeat_data).await
    }

    /// Process incoming heartbeat from another node
    pub async fn process_heartbeat(&self, heartbeat_data: &serde_json::Value) -> Result<()> {
        debug!("Processing incoming heartbeat");

        // Extract node information from heartbeat
        let node_id = heartbeat_data.get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let cluster_id = heartbeat_data.get("cluster_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = heartbeat_data.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Validate cluster ID
        if cluster_id != self.config.cluster_id {
            return Err(SongbirdError::Configuration {
                field: "cluster_id".to_string(),
                message: format!("Heartbeat from wrong cluster: expected {}, got {}", self.config.cluster_id, cluster_id),
                suggestion: Some("Check cluster configuration".to_string()),
            });
        }

        // Update node status in registry
        self.update_node_status(node_id, status, heartbeat_data).await?;

        info!("Processed heartbeat from node: {} (status: {})", node_id, status);
        Ok(())
    }

    /// Update node status in the federation registry
    async fn update_node_status(&self, node_id: &str, status: &str, heartbeat_data: &serde_json::Value) -> Result<()> {
        debug!("Updating node status for: {} -> {}", node_id, status);

        // In a real implementation, this would update a persistent registry
        // For now, we'll just log the update
        if let Some(metrics) = heartbeat_data.get("metrics") {
            let cpu_usage = metrics.get("cpu_usage").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let memory_usage = metrics.get("memory_usage").and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            debug!("Node {} metrics: CPU {}%, Memory {}%", node_id, cpu_usage, memory_usage);
        }

        // Update last seen timestamp
        let timestamp = chrono::Utc::now();
        debug!("Node {} last seen: {}", node_id, timestamp);

        Ok(())
    }

    /// Check if heartbeat is healthy (within expected interval)
    pub async fn is_heartbeat_healthy(&self, last_heartbeat: chrono::DateTime<chrono::Utc>) -> bool {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(last_heartbeat);
        
        // Consider heartbeat healthy if it's within 2x the configured interval
        let max_interval = Duration::from_secs(
            (self.config.heartbeat_interval.unwrap_or(30) * 2) as u64
        );
        
        duration.to_std().unwrap_or(Duration::from_secs(999)) < max_interval
    }

    /// Get heartbeat statistics
    pub async fn get_heartbeat_stats(&self) -> Result<serde_json::Value> {
        // In a real implementation, this would return actual statistics
        // For now, return basic information
        Ok(serde_json::json!({
            "heartbeat_interval": self.config.heartbeat_interval.unwrap_or(30),
            "total_endpoints": self.config.cluster_endpoints.len(),
            "last_heartbeat_cycle": chrono::Utc::now(),
            "status": "active"
        }))
    }

    /// Test connectivity to all configured endpoints
    pub async fn test_all_endpoints(&self) -> Result<Vec<String>> {
        info!("Testing connectivity to all configured endpoints");

        let mut connected_endpoints = Vec::new();

        for endpoint in &self.config.cluster_endpoints {
            match self.test_endpoint_connectivity(endpoint).await {
                Ok(()) => {
                    connected_endpoints.push(endpoint.to_string());
                    info!("Successfully connected to: {}", endpoint);
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", endpoint, e);
                }
            }
        }

        info!(
            "Connectivity test completed: {}/{} endpoints reachable",
            connected_endpoints.len(),
            self.config.cluster_endpoints.len()
        );

        Ok(connected_endpoints)
    }

    /// Check if heartbeat task is running
    pub async fn is_heartbeat_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }

    /// Get heartbeat interval
    pub fn get_heartbeat_interval(&self) -> Duration {
        Duration::from_secs(self.config.heartbeat_interval.unwrap_or(30) as u64)
    }

    /// Update heartbeat configuration
    pub async fn update_config(
        &mut self,
        new_config: FederationConfig,
    ) -> Result<()> {
        info!("Updating heartbeat configuration");

        // Update local configuration
        self.config = new_config.clone();

        // Restart heartbeat if interval changed
        if self.config.heartbeat_interval != new_config.heartbeat_interval {
            info!("Heartbeat interval changed, restarting heartbeat");

            // Stop current heartbeat task
            self.stop_heartbeat_task().await;

            // Start new heartbeat task if we have endpoints
            if !self.config.cluster_endpoints.is_empty() {
                self.start_heartbeat_task().await?;
            }
        }

        info!("Heartbeat configuration updated successfully");
        Ok(())
    }

    /// Send departure notifications to all endpoints
    pub async fn send_departure_to_all(&self) -> Result<()> {
        info!("Sending departure notifications to all endpoints");

        for endpoint in &self.config.cluster_endpoints {
            if let Err(e) = self.send_departure_notification(endpoint).await {
                warn!(
                    "Failed to send departure notification to {}: {}",
                    endpoint, e
                );
            }
        }

        Ok(())
    }
}

impl Drop for HeartbeatManager {
    fn drop(&mut self) {
        // Ensure heartbeat task is stopped when manager is dropped
        // Note: This is a best-effort cleanup since we can't await in Drop
        if let Ok(running) = self.running.try_read() {
            if *running {
                warn!("HeartbeatManager dropped while heartbeat task was still running");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FederationConfig;

    #[tokio::test]
    async fn test_heartbeat_manager_creation() {
        let config = FederationConfig::default();
        let heartbeat = HeartbeatManager::new(config);

        assert!(!heartbeat.is_heartbeat_running().await);
    }

    #[tokio::test]
    async fn test_heartbeat_interval() {
        let mut config = FederationConfig::default();
        config.heartbeat_interval = Some(60);

        let heartbeat = HeartbeatManager::new(config);
        assert_eq!(heartbeat.get_heartbeat_interval(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_heartbeat_task_lifecycle() {
        let config = FederationConfig::default();
        let heartbeat = HeartbeatManager::new(config);

        // Start heartbeat task
        assert!(heartbeat.start_heartbeat_task().await.is_ok());
        assert!(heartbeat.is_heartbeat_running().await);

        // Stop heartbeat task
        heartbeat.stop_heartbeat_task().await;
        assert!(!heartbeat.is_heartbeat_running().await);
    }
}
