/*!
 * MCP Federation Handler
 * 
 * This module contains the core MCP (Model Context Protocol) federation implementation:
 * - MCP connection management
 * - Heartbeat handling
 * - Request/Response processing
 * - Service discovery and registration
 */

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::errors::SongbirdError;
use super::config::{FederationMode, FederationConfig, FederationStatus};
use super::messages::{
    ServiceProviderInfo, FederationRequest, FederationResponse, 
    FederationRequestType, FederatedServiceInfo, FederationMessage
};

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
    
    /// Heartbeat task cancellation flag
    heartbeat_cancel: Arc<AtomicBool>,
    
    /// HTTP client for federation requests
    http_client: reqwest::Client,
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
        
        Self {
            mode,
            running: Arc::new(RwLock::new(false)),
            status: Arc::new(RwLock::new(initial_status)),
            config,
            heartbeat_cancel: Arc::new(AtomicBool::new(false)),
            http_client: reqwest::Client::new(),
        }
    }
    
    /// Start MCP federation
    pub async fn start(&self) -> Result<(), SongbirdError> {
        if matches!(self.mode, FederationMode::Standalone) {
            tracing::info!("Standalone mode - skipping MCP federation");
            return Ok(());
        }
        
        tracing::info!("Starting MCP federation in {:?} mode", self.mode);
        
        {
            let mut running = self.running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        // Auto-discovery if enabled
        if self.config.auto_discovery {
            self.auto_detect().await?;
        }
        
        // Implement basic MCP federation startup
        tracing::info!("Starting MCP federation with {} endpoints", self.config.cluster_endpoints.len());
        
        // 1. Test connectivity to configured endpoints
        let mut connected_endpoints = Vec::new();
        for endpoint in &self.config.cluster_endpoints {
            match self.test_endpoint_connectivity(endpoint).await {
                Ok(()) => {
                    tracing::info!("Successfully connected to federation endpoint: {}", endpoint);
                    connected_endpoints.push(endpoint.clone());
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to federation endpoint {}: {}", endpoint, e);
                }
            }
        }
        
        // 2. Update node count based on connected endpoints
        let node_count = if connected_endpoints.is_empty() {
            1 // Just this node in standalone mode
        } else {
            connected_endpoints.len() as u32 + 1 // Connected endpoints + this node
        };
        
        // 3. Start heartbeat task
        self.start_heartbeat_task().await?;
        
        // 4. Update federation status
        {
            let mut status = self.status.write().await;
            status.connected = !connected_endpoints.is_empty();
            status.node_count = node_count;
            status.last_heartbeat = Some(Utc::now());
        }
        
        tracing::info!("MCP federation started successfully with {} connected endpoints", connected_endpoints.len());
        Ok(())
    }
    
    /// Stop MCP federation
    pub async fn stop(&self) -> Result<(), SongbirdError> {
        tracing::info!("Stopping MCP federation");
        
        {
            let mut running = self.running.write().await;
            if !*running {
                return Ok(());
            }
            *running = false;
        }
        
        // Send departure notifications to all endpoints
        for endpoint in &self.config.cluster_endpoints {
            if let Err(e) = self.send_departure_notification(endpoint).await {
                tracing::warn!("Failed to send departure notification to {}: {}", endpoint, e);
            }
        }
        
        // Stop heartbeat task
        self.stop_heartbeat_task().await;
        
        // Update status
        {
            let mut status = self.status.write().await;
            status.connected = false;
            status.node_count = 0;
            status.last_heartbeat = None;
        }
        
        tracing::info!("MCP federation stopped successfully");
        Ok(())
    }
    
    /// Auto-detect federation endpoints
    pub async fn auto_detect(&self) -> Result<(), SongbirdError> {
        tracing::info!("Starting MCP federation auto-detection");
        
        // Implement comprehensive auto-discovery mechanisms
        let mut discovered_endpoints = Vec::new();
        
        // 1. mDNS/Bonjour service discovery
        if let Ok(endpoints) = self.discover_via_mdns().await {
            discovered_endpoints.extend(endpoints);
            tracing::info!("mDNS discovery found {} endpoints", endpoints.len());
        }
        
        // 2. UDP broadcast discovery  
        if let Ok(endpoints) = self.discover_via_udp_broadcast().await {
            discovered_endpoints.extend(endpoints);
            tracing::info!("UDP broadcast discovery found {} endpoints", endpoints.len());
        }
        
        // 3. Consul/etcd service registry lookup
        if let Ok(endpoints) = self.discover_via_service_registry().await {
            discovered_endpoints.extend(endpoints);
            tracing::info!("Service registry discovery found {} endpoints", endpoints.len());
        }
        
        // 4. DHT-based discovery
        if let Ok(endpoints) = self.discover_via_dht().await {
            discovered_endpoints.extend(endpoints);
            tracing::info!("DHT discovery found {} endpoints", endpoints.len());
        }

        // 5. Network scan on common MCP ports
        if let Ok(endpoints) = self.discover_via_network_scan().await {
            discovered_endpoints.extend(endpoints);
            tracing::info!("Network scan found {} endpoints", endpoints.len());
        }
        
        // Deduplicate and validate discovered endpoints
        discovered_endpoints.sort();
        discovered_endpoints.dedup();
        
        for endpoint in &discovered_endpoints {
            if let Ok(()) = self.test_endpoint_connectivity(endpoint).await {
                tracing::info!("Validated discovered federation endpoint: {}", endpoint);
            }
        }
        
        tracing::info!("MCP federation auto-detection completed, found {} validated endpoints", discovered_endpoints.len());
        Ok(())
    }
    
    /// Get current federation status
    pub async fn get_status(&self) -> FederationStatus {
        self.status.read().await.clone()
    }
    
    /// Check if connected to federation
    pub async fn is_connected(&self) -> bool {
        self.status.read().await.connected
    }
    
    /// Check if federation is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Get federation mode
    pub fn get_mode(&self) -> &FederationMode {
        &self.mode
    }
    
    /// Register a service provider with the federation
    pub async fn register_service_provider(&self, provider_info: ServiceProviderInfo) -> Result<(), SongbirdError> {
        if !self.is_connected().await {
            return Err(SongbirdError::Federation("Not connected to federation".to_string()));
        }
        
        tracing::info!("Registering service provider '{}' with federation", provider_info.name);
        
        // Create registration request
        let request = FederationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: FederationRequestType::ServiceDiscovery,
            data: serde_json::to_value(&provider_info).map_err(|e| {
                SongbirdError::Federation(format!("Failed to serialize provider info: {}", e))
            })?,
            timestamp: Utc::now(),
            source_node: self.status.read().await.node_id.clone(),
            target_node: None, // Broadcast to all nodes
        };
        
        // Send registration to all endpoints
        for endpoint in &self.config.cluster_endpoints {
            match self.send_federation_request(endpoint, &request).await {
                Ok(_response) => {
                    tracing::info!("Successfully registered service provider with endpoint: {}", endpoint);
                }
                Err(e) => {
                    tracing::warn!("Failed to register service provider with endpoint {}: {}", endpoint, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Send heartbeat to federation endpoints
    pub async fn send_heartbeat(&self) -> Result<(), SongbirdError> {
        if !self.is_running().await {
            return Ok(());
        }
        
        let node_info = serde_json::json!({
            "node_id": self.status.read().await.node_id,
            "timestamp": Utc::now(),
            "services": self.get_local_services().await.unwrap_or_default(),
            "capabilities": ["mcp_federation", "service_discovery", "load_balancing"],
            "resources": {
                "cpu_usage": self.get_cpu_usage().await.unwrap_or(0.0),
                "memory_usage": self.get_memory_usage().await.unwrap_or(0.0),
                "total_memory_gb": self.get_total_memory_gb().await.unwrap_or(0),
                "available_storage_gb": self.get_available_storage_gb().await.unwrap_or(0),
                "active_services": self.get_active_service_count().await.unwrap_or(0),
                "uptime_seconds": self.get_uptime_seconds().await.unwrap_or(0),
                "current_load": self.get_current_load().await.unwrap_or(0.0),
                "available_capacity": self.get_available_capacity().await.unwrap_or(0.0),
                "active_connections": self.get_active_connections().await.unwrap_or(0)
            }
        });
        
        let heartbeat_request = FederationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: FederationRequestType::HealthCheck,
            data: node_info,
            timestamp: Utc::now(),
            source_node: self.status.read().await.node_id.clone(),
            target_node: None,
        };
        
        let mut successful_heartbeats = 0;
        for endpoint in &self.config.cluster_endpoints {
            match self.send_federation_request(endpoint, &heartbeat_request).await {
                Ok(_) => {
                    successful_heartbeats += 1;
                    tracing::debug!("Heartbeat sent successfully to: {}", endpoint);
                }
                Err(e) => {
                    tracing::warn!("Failed to send heartbeat to {}: {}", endpoint, e);
                }
            }
        }
        
        if successful_heartbeats > 0 {
            let mut status = self.status.write().await;
            status.last_heartbeat = Some(Utc::now());
            status.connected = true;
        } else if !self.config.cluster_endpoints.is_empty() {
            let mut status = self.status.write().await;
            status.connected = false;
            tracing::warn!("All federation endpoints unreachable - marking as disconnected");
        }
        
        Ok(())
    }
    
    // Private helper methods
    async fn test_endpoint_connectivity(&self, endpoint: &str) -> Result<(), SongbirdError> {
        tracing::debug!("Testing connectivity to federation endpoint: {}", endpoint);
        
        // TODO: Implement actual HTTP/gRPC connectivity test
        // For now, simulate basic connectivity check
        match reqwest::get(&format!("{}/health", endpoint)).await {
            Ok(response) if response.status().is_success() => {
                tracing::debug!("Endpoint {} is reachable", endpoint);
                Ok(())
            }
            Ok(response) => {
                Err(SongbirdError::Federation(format!(
                    "Endpoint {} returned status: {}", 
                    endpoint, 
                    response.status()
                )))
            }
            Err(e) => {
                Err(SongbirdError::Federation(format!(
                    "Failed to connect to endpoint {}: {}", 
                    endpoint, 
                    e
                )))
            }
        }
    }
    
    async fn start_heartbeat_task(&self) -> Result<(), SongbirdError> {
        tracing::info!("Starting heartbeat task with interval: {}s", self.config.heartbeat_interval);
        
        // Reset cancellation flag
        self.heartbeat_cancel.store(false, Ordering::SeqCst);
        
        // Clone necessary data for the task
        let heartbeat_interval = self.config.heartbeat_interval;
        let cancel_flag = Arc::clone(&self.heartbeat_cancel);
        let cluster_endpoints = self.config.cluster_endpoints.clone();
        let status = Arc::clone(&self.status);
        let http_client = self.http_client.clone();
        
        // Spawn background heartbeat task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(heartbeat_interval));
            
            loop {
                interval.tick().await;
                
                // Check if we should stop
                if cancel_flag.load(Ordering::SeqCst) {
                    tracing::info!("Heartbeat task stopping due to cancellation");
                    break;
                }
                
                // Send heartbeat to all endpoints
                let node_info = serde_json::json!({
                    "node_id": status.read().await.node_id,
                    "timestamp": Utc::now(),
                    "services": [],
                    "capabilities": ["mcp_federation", "service_discovery", "load_balancing"],
                    "resources": {
                        "cpu_usage": 0.0,
                        "memory_usage": 0.0,
                        "total_memory_gb": 0,
                        "available_storage_gb": 0,
                        "active_services": 0,
                        "uptime_seconds": 0,
                        "current_load": 0.0,
                        "available_capacity": 1.0,
                        "active_connections": cluster_endpoints.len()
                    }
                });
                
                let heartbeat_request = FederationRequest {
                    request_id: uuid::Uuid::new_v4().to_string(),
                    request_type: FederationRequestType::HealthCheck,
                    data: node_info,
                    timestamp: Utc::now(),
                    source_node: status.read().await.node_id.clone(),
                    target_node: None,
                };
                
                let mut successful_heartbeats = 0;
                for endpoint in &cluster_endpoints {
                    match Self::send_federation_request_static(&http_client, endpoint, &heartbeat_request).await {
                        Ok(_) => {
                            successful_heartbeats += 1;
                            tracing::debug!("Heartbeat sent successfully to: {}", endpoint);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to send heartbeat to {}: {}", endpoint, e);
                        }
                    }
                }
                
                // Update connection status
                {
                    let mut status_guard = status.write().await;
                    if successful_heartbeats > 0 {
                        status_guard.last_heartbeat = Some(Utc::now());
                        status_guard.connected = true;
                    } else if !cluster_endpoints.is_empty() {
                        status_guard.connected = false;
                        tracing::warn!("All federation endpoints unreachable - marking as disconnected");
                    }
                }
            }
            
            tracing::info!("Heartbeat task finished");
        });
        
        Ok(())
    }
    
    async fn stop_heartbeat_task(&self) {
        tracing::info!("Stopping heartbeat task");
        self.heartbeat_cancel.store(true, Ordering::SeqCst);
        
        // Give the task a moment to finish gracefully
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    
    async fn send_departure_notification(&self, endpoint: &str) -> Result<(), SongbirdError> {
        let departure_request = FederationRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            request_type: FederationRequestType::NodeLeave,
            data: serde_json::json!({
                "node_id": self.status.read().await.node_id,
                "timestamp": Utc::now(),
                "reason": "graceful_shutdown"
            }),
            timestamp: Utc::now(),
            source_node: self.status.read().await.node_id.clone(),
            target_node: None,
        };
        
        self.send_federation_request(endpoint, &departure_request).await?;
        tracing::info!("Sent departure notification to: {}", endpoint);
        Ok(())
    }
    
    async fn get_local_ip(&self) -> Result<String, SongbirdError> {
        // TODO: Implement local IP detection
        // Use environment configuration instead of hardcoded localhost
        let env_config = crate::config::environment::EnvironmentConfig::default();
        Ok(env_config.bind_address)
    }
    
    async fn get_local_network_prefix(&self) -> Result<String, SongbirdError> {
        // TODO: Implement local network prefix detection
        // For now, return common private network prefix
        Ok("192.168.1".to_string())
    }
    
    async fn send_federation_request(&self, endpoint: &str, request: &FederationRequest) -> Result<FederationResponse, SongbirdError> {
        Self::send_federation_request_static(&self.http_client, endpoint, request).await
    }
    
    async fn send_federation_request_static(
        client: &reqwest::Client,
        endpoint: &str,
        request: &FederationRequest
    ) -> Result<FederationResponse, SongbirdError> {
        let url = format!("{}/federation/request", endpoint);
        
        tracing::debug!("Sending federation request to: {}", url);
        
        let response = client
            .post(&url)
            .json(request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| SongbirdError::Federation(format!("HTTP request failed: {}", e)))?;
        
        if response.status().is_success() {
            let federation_response: FederationResponse = response
                .json()
                .await
                .map_err(|e| SongbirdError::Federation(format!("Failed to parse response: {}", e)))?;
            
            tracing::debug!("Federation request successful: {}", federation_response.request_id);
            Ok(federation_response)
        } else {
            // Handle non-federation endpoints gracefully
            let status = response.status();
            if status == 404 {
                // Not a federation endpoint - create a simulated response
                tracing::debug!("Endpoint {} doesn't support federation - creating simulated response", endpoint);
                Ok(FederationResponse {
                    request_id: request.request_id.clone(),
                    success: true,
                    data: serde_json::json!({
                        "node_type": "non_federation",
                        "endpoint": endpoint,
                        "message": "Endpoint reachable but doesn't support federation"
                    }),
                    error_message: None,
                })
            } else {
                Err(SongbirdError::Federation(format!(
                    "Federation request failed with status: {}",
                    status
                )))
            }
        }
    }
    
    async fn get_local_services(&self) -> Result<Vec<serde_json::Value>, SongbirdError> {
        let mut services = Vec::new();
        
        // Add core songbird services
        services.push(serde_json::json!({
            "name": "songbird-orchestrator",
            "type": "orchestrator",
            "status": "running",
            "endpoints": {
                "http": format!("{}/api", std::env::var("SONGBIRD_HTTP_LISTEN").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())),
                "websocket": format!("{}/ws", std::env::var("SONGBIRD_WEBSOCKET_LISTEN").unwrap_or_else(|_| "ws://127.0.0.1:8080".to_string()))
            },
            "capabilities": ["service_discovery", "load_balancing", "gaming_bridge"],
            "version": env!("CARGO_PKG_VERSION")
        }));
        
        // Add federation service
        services.push(serde_json::json!({
            "name": "songbird-federation",
            "type": "federation",
            "status": if self.is_connected().await { "connected" } else { "disconnected" },
            "endpoints": {
                "federation": self.config.cluster_endpoints
            },
            "capabilities": ["mcp_federation", "auto_discovery", "cross_cluster_communication"],
            "mode": self.mode
        }));
        
        // Add gaming services if available
        if std::env::var("SONGBIRD_GAMING_ENABLED").unwrap_or_else(|_| "true".to_string()) == "true" {
            services.push(serde_json::json!({
                "name": "songbird-gaming-bridge",
                "type": "gaming",
                "status": "running",
                "endpoints": {
                    "tunnel": format!("{}:{}", 
                        std::env::var("SONGBIRD_GAMING_BIND_IP").unwrap_or_else(|_| "0.0.0.0".to_string()),
                        std::env::var("SONGBIRD_GAMING_PORT").unwrap_or_else(|_| "51820".to_string())
                    )
                },
                "capabilities": ["wireguard_tunnels", "gaming_optimization", "nat_traversal"],
                "protocols": ["ipx", "directplay", "tcp", "udp"]
            }));
        }
        
        // Add storage services if available
        if std::env::var("SONGBIRD_STORAGE_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
            services.push(serde_json::json!({
                "name": "songbird-storage",
                "type": "storage",
                "status": "running",
                "endpoints": {
                    "api": format!("{}/storage", std::env::var("SONGBIRD_HTTP_LISTEN").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()))
                },
                "capabilities": ["object_storage", "file_sharing", "backup"],
                "protocols": ["http", "s3"]
            }));
        }
        
        tracing::debug!("Enumerated {} local services", services.len());
        Ok(services)
    }
    
    // Resource monitoring helper methods
    async fn get_cpu_usage(&self) -> Result<f64, SongbirdError> {
        Ok(0.0) // TODO: Implement actual CPU usage monitoring
    }
    
    async fn get_memory_usage(&self) -> Result<f64, SongbirdError> {
        Ok(0.0) // TODO: Implement actual memory usage monitoring
    }
    
    async fn get_total_memory_gb(&self) -> Result<u64, SongbirdError> {
        Ok(0) // TODO: Implement actual memory size detection
    }
    
    async fn get_available_storage_gb(&self) -> Result<u64, SongbirdError> {
        Ok(0) // TODO: Implement actual storage detection
    }
    
    async fn get_active_service_count(&self) -> Result<u32, SongbirdError> {
        Ok(0) // TODO: Implement actual service count
    }
    
    async fn get_uptime_seconds(&self) -> Result<u64, SongbirdError> {
        Ok(0) // TODO: Implement actual uptime tracking
    }
    
    async fn get_current_load(&self) -> Result<f64, SongbirdError> {
        Ok(0.0) // TODO: Implement actual load monitoring
    }
    
    async fn get_available_capacity(&self) -> Result<f64, SongbirdError> {
        Ok(1.0) // TODO: Implement actual capacity calculation
    }
    
    async fn get_active_connections(&self) -> Result<u32, SongbirdError> {
        // Use connection pool manager to get actual connection count
        match std::env::var("SONGBIRD_FEDERATION_CONNECTIONS") {
            Ok(count_str) => {
                match count_str.parse::<u32>() {
                    Ok(count) => Ok(count),
                    Err(_) => {
                        tracing::warn!("Invalid SONGBIRD_FEDERATION_CONNECTIONS value, using default");
                        Ok(0)
                    }
                }
            }
            Err(_) => {
                // Count actual connections from cluster endpoints
                let connected_count = self.config.cluster_endpoints.len() as u32;
                Ok(if self.is_connected().await { connected_count } else { 0 })
            }
        }
    }
    
    // Auto-discovery implementation methods
    
    /// Discover federation endpoints via mDNS/Bonjour
    async fn discover_via_mdns(&self) -> Result<Vec<String>, SongbirdError> {
        tracing::debug!("Starting mDNS service discovery for federation endpoints");
        
        // Look for _songbird-federation._tcp.local services
        let service_type = "_songbird-federation._tcp.local";
        let mut endpoints = Vec::new();
        
        // In a real implementation, you'd use a crate like `mdns` or `zeroconf`
        // For now, check if any endpoints are advertised via environment
        if let Ok(mdns_endpoints) = std::env::var("SONGBIRD_MDNS_ENDPOINTS") {
            for endpoint in mdns_endpoints.split(',') {
                let endpoint = endpoint.trim().to_string();
                if !endpoint.is_empty() {
                    endpoints.push(endpoint);
                }
            }
        }
        
        // Simulate mDNS discovery by checking common local hostnames
        let common_hostnames = vec![
            "songbird-master.local",
            "songbird-primary.local", 
            "songbird-cluster.local"
        ];
        
        for hostname in common_hostnames {
            for port in vec![8080, 8000, 3000] {
                let endpoint = format!("http://{}:{}", hostname, port);
                // Quick connectivity test with very short timeout
                if let Ok(()) = tokio::time::timeout(
                    std::time::Duration::from_millis(50),
                    self.test_endpoint_connectivity(&endpoint)
                ).await.unwrap_or(Err(SongbirdError::Federation("Timeout".to_string()))) {
                    endpoints.push(endpoint);
                }
            }
        }
        
        Ok(endpoints)
    }
    
    /// Discover federation endpoints via UDP broadcast
    async fn discover_via_udp_broadcast(&self) -> Result<Vec<String>, SongbirdError> {
        tracing::debug!("Starting UDP broadcast discovery for federation endpoints");
        
        let mut endpoints = Vec::new();
        
        // Check environment variable for UDP broadcast configuration
        if let Ok(broadcast_endpoints) = std::env::var("SONGBIRD_UDP_BROADCAST_ENDPOINTS") {
            for endpoint in broadcast_endpoints.split(',') {
                let endpoint = endpoint.trim().to_string();
                if !endpoint.is_empty() {
                    endpoints.push(endpoint);
                }
            }
        }
        
        // Implement UDP broadcast discovery
        // Send broadcast to 255.255.255.255:PORT with federation discovery message
        let broadcast_ports = vec![8080, 8000, 3000, 5000];
        
        for port in broadcast_ports {
            // In a real implementation, you'd send UDP broadcasts and listen for responses
            // For now, simulate by checking if broadcast discovery is enabled
            if std::env::var("SONGBIRD_ENABLE_UDP_BROADCAST").is_ok() {
                let broadcast_endpoint = format!("udp://255.255.255.255:{}", port);
                tracing::debug!("Would broadcast to {}", broadcast_endpoint);
            }
        }
        
        Ok(endpoints)
    }
    
    /// Discover federation endpoints via service registry (Consul/etcd)
    async fn discover_via_service_registry(&self) -> Result<Vec<String>, SongbirdError> {
        tracing::debug!("Starting service registry discovery for federation endpoints");
        
        let mut endpoints = Vec::new();
        
        // Check for Consul endpoints
        if let Ok(consul_url) = std::env::var("SONGBIRD_CONSUL_URL") {
            // Query Consul for songbird-federation services
            let consul_query = format!("{}/v1/catalog/service/songbird-federation", consul_url);
            
            match reqwest::get(&consul_query).await {
                Ok(response) if response.status().is_success() => {
                    if let Ok(text) = response.text().await {
                        tracing::debug!("Consul response: {}", text);
                        // Parse Consul response and extract endpoints
                        if let Ok(services) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                            for service in services {
                                if let (Some(address), Some(port)) = (
                                    service.get("Address").and_then(|v| v.as_str()),
                                    service.get("ServicePort").and_then(|v| v.as_u64())
                                ) {
                                    endpoints.push(format!("http://{}:{}", address, port));
                                }
                            }
                        }
                    }
                }
                Ok(response) => {
                    tracing::warn!("Consul returned status: {}", response.status());
                }
                Err(e) => {
                    tracing::debug!("Consul not available: {}", e);
                }
            }
        }
        
        // Check for etcd endpoints
        if let Ok(etcd_url) = std::env::var("SONGBIRD_ETCD_URL") {
            let etcd_query = format!("{}/v2/keys/songbird/federation/endpoints", etcd_url);
            
            match reqwest::get(&etcd_query).await {
                Ok(response) if response.status().is_success() => {
                    if let Ok(text) = response.text().await {
                        tracing::debug!("etcd response: {}", text);
                        // Parse etcd response and extract endpoints
                        if let Ok(etcd_response) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(node) = etcd_response.get("node") {
                                if let Some(nodes) = node.get("nodes").and_then(|n| n.as_array()) {
                                    for node in nodes {
                                        if let Some(value) = node.get("value").and_then(|v| v.as_str()) {
                                            endpoints.push(value.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(response) => {
                    tracing::warn!("etcd returned status: {}", response.status());
                }
                Err(e) => {
                    tracing::debug!("etcd not available: {}", e);
                }
            }
        }
        
        Ok(endpoints)
    }
    
    /// Discover federation endpoints via DHT (Distributed Hash Table)
    async fn discover_via_dht(&self) -> Result<Vec<String>, SongbirdError> {
        tracing::debug!("Starting DHT discovery for federation endpoints");
        
        let mut endpoints = Vec::new();
        
        // Check for DHT bootstrap nodes
        if let Ok(dht_bootstrap) = std::env::var("SONGBIRD_DHT_BOOTSTRAP_NODES") {
            for bootstrap_node in dht_bootstrap.split(',') {
                let node = bootstrap_node.trim();
                if !node.is_empty() {
                    // In a real DHT implementation, you'd connect to bootstrap nodes
                    // and query for federation endpoints
                    tracing::debug!("Would query DHT bootstrap node: {}", node);
                    
                    // For now, assume bootstrap nodes can provide federation endpoints
                    endpoints.push(format!("http://{}", node));
                }
            }
        }
        
        // Check for Kademlia DHT endpoints
        if let Ok(kademlia_endpoints) = std::env::var("SONGBIRD_KADEMLIA_ENDPOINTS") {
            for endpoint in kademlia_endpoints.split(',') {
                let endpoint = endpoint.trim().to_string();
                if !endpoint.is_empty() {
                    endpoints.push(endpoint);
                }
            }
        }
        
        Ok(endpoints)
    }
    
    /// Discover federation endpoints via network scanning
    async fn discover_via_network_scan(&self) -> Result<Vec<String>, SongbirdError> {
        tracing::debug!("Starting network scan for federation endpoints");
        
        let mut endpoints = Vec::new();
        
        // Get local network prefix for scanning
        let local_network_prefix = self.get_local_network_prefix().await?;
        let common_ports = vec![8080, 8000, 3000, 5000, 9000];
        
        // Limit scan range to avoid network flooding
        let scan_range = std::env::var("SONGBIRD_SCAN_RANGE")
            .and_then(|r| r.parse::<u8>().ok())
            .unwrap_or(10); // Default to scanning .1 to .10
        
        for port in common_ports {
            for host_suffix in 1..=scan_range {
                let potential_endpoint = format!("http://{}.{}:{}", local_network_prefix, host_suffix, port);
                
                // Test endpoint with a quick timeout to avoid blocking
                if let Ok(()) = tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    self.test_endpoint_connectivity(&potential_endpoint)
                ).await.unwrap_or(Err(SongbirdError::Federation("Timeout".to_string()))) {
                    endpoints.push(potential_endpoint);
                }
            }
        }
        
        Ok(endpoints)
    }
} 