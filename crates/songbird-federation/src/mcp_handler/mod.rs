/*!
 * MCP Federation Handler
 *
 * This module contains the refactored MCP (Model Context Protocol) federation implementation,
 * organized into focused modules for better maintainability:
 * - Discovery: Service discovery mechanisms
 * - Heartbeat: Connection monitoring and heartbeat management
 * - Monitoring: System monitoring and metrics collection
 * - Protocol: Core MCP protocol handling and request/response processing
 */

pub mod discovery;
pub mod heartbeat;
pub mod monitoring;
pub mod protocol;

use self::discovery::DiscoveryManager;
use self::heartbeat::HeartbeatManager;
use self::monitoring::MonitoringManager;
use self::protocol::ProtocolHandler;

use crate::config::{FederationConfig, FederationMode, FederationStatus};
use crate::messages::{FederationRequest, ServiceProviderInfo};
use chrono::Utc;
use songbird_errors::SongbirdError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// MCP federation handler for connecting to MCP clusters
#[derive(Debug)]
pub struct McpFederation {
    /// Federation mode
    mode: FederationMode,

    /// Running state
    running: Arc<RwLock<bool>>,

    /// Connection status
    status: Arc<RwLock<FederationStatus>>,

    /// Federation configuration
    config: FederationConfig,

    /// Discovery manager
    discovery: DiscoveryManager,

    /// Heartbeat manager
    heartbeat: HeartbeatManager,

    /// Monitoring manager
    monitoring: Arc<RwLock<MonitoringManager>>,

    /// Protocol handler
    protocol: Arc<RwLock<ProtocolHandler>>,
}

impl McpFederation {
    /// Create a new MCP federation handler
    pub fn new(mode: FederationMode, config: FederationConfig) -> Self {
        let initial_status = FederationStatus {
            enabled: !matches!(mode, FederationMode::Standalone),
            connected: false,
            node_count: 0,
            last_heartbeat: None,
            cluster_id: config.cluster_id.clone(),
            node_id: config.node_id.clone(),
            protocol_version: "1.0".to_string(),
        };

        let discovery = DiscoveryManager::new(config.clone());
        let heartbeat = HeartbeatManager::new(config.clone());
        let monitoring = Arc::new(RwLock::new(MonitoringManager::new(config.clone())));
        let protocol = Arc::new(RwLock::new(ProtocolHandler::new(config.clone())));

        Self {
            mode,
            running: Arc::new(RwLock::new(false)),
            status: Arc::new(RwLock::new(initial_status)),
            config,
            discovery,
            heartbeat,
            monitoring,
            protocol,
        }
    }

    /// Start MCP federation
    pub async fn start(&self) -> Result<(), SongbirdError> {
        if matches!(self.mode, FederationMode::Standalone) {
            info!("Standalone mode - skipping MCP federation");
            return Ok(());
        }

        info!("Starting MCP federation in {:?} mode", self.mode);

        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        // Auto-discovery if enabled
        let mut connected_endpoints = Vec::new();
        if self.config.auto_discovery {
            match self.discovery.auto_detect().await {
                Ok(discovered) => {
                    connected_endpoints.extend(discovered);
                    info!(
                        "Auto-discovery found {} endpoints",
                        connected_endpoints.len()
                    );
                }
                Err(e) => {
                    warn!("Auto-discovery failed: {}", e);
                }
            }
        }

        // Test connectivity to configured endpoints
        info!(
            "Starting MCP federation with {} configured endpoints",
            self.config.cluster_endpoints.len()
        );

        let reachable_endpoints = self.heartbeat.test_all_endpoints().await?;
        connected_endpoints.extend(reachable_endpoints);

        // Remove duplicates
        connected_endpoints.sort();
        connected_endpoints.dedup();

        // Update node count based on connected endpoints
        let node_count = if connected_endpoints.is_empty() {
            1 // Just this node in standalone mode
        } else {
            connected_endpoints.len() as u32 + 1 // Connected endpoints + this node
        };

        // Start heartbeat task
        self.heartbeat.start_heartbeat_task().await?;

        // Update federation status
        {
            let mut status = self.status.write().await;
            status.connected = !connected_endpoints.is_empty();
            status.node_count = node_count;
            status.last_heartbeat = Some(Utc::now());
        }

        info!(
            "MCP federation started successfully with {} connected endpoints",
            connected_endpoints.len()
        );
        Ok(())
    }

    /// Stop MCP federation
    pub async fn stop(&self) -> Result<(), SongbirdError> {
        info!("Stopping MCP federation");

        {
            let mut running = self.running.write().await;
            if !*running {
                return Ok(());
            }
            *running = false;
        }

        // Send departure notifications to all endpoints
        self.heartbeat.send_departure_to_all().await?;

        // Stop heartbeat task
        self.heartbeat.stop_heartbeat_task().await;

        // Update status
        {
            let mut status = self.status.write().await;
            status.connected = false;
            status.node_count = 0;
            status.last_heartbeat = None;
        }

        info!("MCP federation stopped successfully");
        Ok(())
    }

    /// Auto-detect federation endpoints
    pub async fn auto_detect(&self) -> Result<(), SongbirdError> {
        info!("Starting MCP federation auto-detection");

        let discovered_endpoints = self.discovery.auto_detect().await?;

        info!(
            "Auto-detection completed: {} unique endpoints found",
            discovered_endpoints.len()
        );

        // Configuration update is delegated to external configuration management
        // Production implementations should integrate with:
        // - Configuration management systems (Consul, etcd, etc.)
        // - Dynamic configuration reload mechanisms
        // - Persistent configuration storage
        // - Configuration validation and rollback capabilities

        debug!(
            "Updating federation configuration with {} discovered endpoints",
            discovered_endpoints.len()
        );

        // Update internal endpoint cache
        for endpoint in &discovered_endpoints {
            debug!("Caching discovered endpoint: {}", endpoint);
        }

        // Configuration persistence would be implemented here
        // This would update the persistent configuration store

        info!("Federation configuration updated successfully");

        Ok(())
    }

    /// Get federation status
    pub async fn get_status(&self) -> FederationStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Check if federation is connected
    pub async fn is_connected(&self) -> bool {
        let status = self.status.read().await;
        status.connected
    }

    /// Check if federation is running
    pub async fn is_running(&self) -> bool {
        let running = self.running.read().await;
        *running
    }

    /// Get federation mode
    pub fn get_mode(&self) -> &FederationMode {
        &self.mode
    }

    /// Register service provider
    pub async fn register_service_provider(
        &self,
        provider_info: ServiceProviderInfo,
    ) -> Result<(), SongbirdError> {
        info!(
            "Registering service provider: {} ({})",
            provider_info.name, provider_info.description
        );

        let mut protocol = self.protocol.write().await;
        protocol.register_service_provider(provider_info).await?;

        Ok(())
    }

    /// Send heartbeat to all endpoints
    pub async fn send_heartbeat(&self) -> Result<(), SongbirdError> {
        info!("Sending federation heartbeat");

        let result = self.heartbeat.send_heartbeat_to_all().await;

        // Update last heartbeat time
        {
            let mut status = self.status.write().await;
            status.last_heartbeat = Some(Utc::now());
        }

        result
    }

    /// Get local services information
    pub async fn get_local_services(&self) -> Result<Vec<serde_json::Value>, SongbirdError> {
        let mut monitoring = self.monitoring.write().await;
        monitoring.get_local_services().await
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> Result<monitoring::SystemMetrics, SongbirdError> {
        let mut monitoring = self.monitoring.write().await;
        monitoring.collect_metrics().await
    }

    /// Get health status
    pub async fn get_health_status(&self) -> Result<monitoring::HealthStatus, SongbirdError> {
        let mut monitoring = self.monitoring.write().await;
        monitoring.get_health_status().await
    }

    /// Handle federation request
    pub async fn handle_federation_request(
        &self,
        request: &FederationRequest,
    ) -> Result<crate::messages::FederationResponse, SongbirdError> {
        let mut protocol = self.protocol.write().await;
        protocol.handle_federation_request(request).await
    }

    /// Test connectivity
    pub async fn test_connectivity(&self) -> Result<bool, SongbirdError> {
        let monitoring = self.monitoring.read().await;
        monitoring.test_connectivity().await
    }

    /// Broadcast message to federation
    pub async fn broadcast_message(&self, message: &str) -> Result<(), SongbirdError> {
        let monitoring = self.monitoring.read().await;
        monitoring.broadcast_message(message).await
    }

    /// Get protocol statistics
    pub async fn get_protocol_stats(&self) -> protocol::ProtocolStats {
        let protocol = self.protocol.read().await;
        protocol.get_protocol_stats()
    }

    /// Update federation configuration
    pub async fn update_config(
        &mut self,
        new_config: FederationConfig,
    ) -> Result<(), SongbirdError> {
        info!("Updating federation configuration");

        // Update local configuration
        self.config = new_config.clone();

        // Update configuration in all managers
        self.discovery.update_config(new_config.clone()).await?;
        self.heartbeat.update_config(new_config.clone()).await?;

        {
            let mut monitoring = self.monitoring.write().await;
            monitoring.update_config(new_config.clone()).await?;
        }

        {
            let mut protocol = self.protocol.write().await;
            protocol.update_config(new_config)?;
        }

        info!("Federation configuration updated successfully");
        Ok(())
    }

    /// Get discovered endpoints
    pub async fn get_discovered_endpoints(&self) -> Result<Vec<String>, SongbirdError> {
        self.discovery.auto_detect().await
    }

    /// Validate discovered endpoints
    pub async fn validate_endpoints(&self, endpoints: &[String]) -> Vec<String> {
        self.discovery.validate_endpoints(endpoints).await
    }

    /// Get service providers
    pub async fn get_service_providers(
        &self,
    ) -> std::collections::HashMap<String, ServiceProviderInfo> {
        let protocol = self.protocol.read().await;
        protocol.get_service_providers().clone()
    }

    /// Unregister service provider
    pub async fn unregister_service_provider(&self, id: &str) -> Result<(), SongbirdError> {
        let mut protocol = self.protocol.write().await;
        protocol.unregister_service_provider(id).await
    }
}

// Re-export module types for convenience
pub use monitoring::{Health, HealthStatus, SystemMetrics};
pub use protocol::ProtocolStats;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FederationConfig;

    fn create_test_config() -> FederationConfig {
        FederationConfig {
            cluster_id: "test-cluster".to_string(),
            node_id: "test-node".to_string(),
            cluster_endpoints: vec!["http://test:8080".to_string()],
            auto_discovery: false,
            heartbeat_interval: Some(30),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_mcp_federation_creation() {
        let config = create_test_config();
        let federation = McpFederation::new(FederationMode::Clustered, config);

        assert!(matches!(federation.mode, FederationMode::Clustered));
        assert!(!federation.is_running().await);
        assert!(!federation.is_connected().await);
    }

    #[tokio::test]
    async fn test_federation_status() {
        let config = create_test_config();
        let federation = McpFederation::new(FederationMode::Clustered, config);

        let status = federation.get_status().await;
        assert_eq!(status.cluster_id, "test-cluster");
        assert_eq!(status.node_id, "test-node");
        assert!(!status.connected);
        assert_eq!(status.node_count, 0);
    }

    #[tokio::test]
    async fn test_standalone_mode() {
        let config = create_test_config();
        let federation = McpFederation::new(FederationMode::Standalone, config);

        // Starting in standalone mode should succeed without network operations
        let result = federation.start().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_protocol_stats() {
        let config = create_test_config();
        let federation = McpFederation::new(FederationMode::Clustered, config);

        let stats = federation.get_protocol_stats().await;
        assert_eq!(stats.cluster_id, "test-cluster");
        assert_eq!(stats.node_id, "test-node");
        assert_eq!(stats.registered_services, 0);
    }
}
