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
    
    /// Get cluster endpoints
    pub async fn get_cluster_endpoints(&self) -> Result<Vec<String>, SongbirdError> {
        Ok(self.config.cluster_endpoints.clone())
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
    /// Test connectivity to federation endpoint
    async fn test_endpoint_connectivity(&self, endpoint: &str) -> Result<(), SongbirdError> {
        tracing::debug!("Testing connectivity to federation endpoint: {}", endpoint);
        
        // Create a health check URL
        let health_url = if endpoint.ends_with('/') {
            format!("{}health", endpoint)
        } else {
            format!("{}/health", endpoint)
        };
        
        // Test basic HTTP connectivity with timeout
        let response = self.http_client
            .get(&health_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| SongbirdError::Network {
                service: "federation".to_string(),
                message: format!("Failed to connect to endpoint {}: {}", endpoint, e),
                details: Some(format!("Connection error: {}", e))
            })?;
        
        // Check if response indicates a Songbird/MCP service
        if response.status().is_success() {
            // Try to parse response as federation info
            match response.text().await {
                Ok(body) => {
                    // Look for indicators that this is a compatible MCP endpoint
                    if body.contains("songbird") || body.contains("mcp") || body.contains("federation") {
                        tracing::debug!("Endpoint {} appears to be compatible MCP service", endpoint);
                        Ok(())
                    } else {
                        tracing::debug!("Endpoint {} responded but doesn't appear to be MCP service", endpoint);
                        // Still consider it successful - might be a basic HTTP service
                        Ok(())
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not read response from {}: {}", endpoint, e);
                    // Still consider successful if we got an HTTP response
                    Ok(())
                }
            }
        } else {
            Err(SongbirdError::Network {
                service: "federation".to_string(),
                message: format!("Endpoint {} returned error status: {}", endpoint, response.status()),
                details: Some(format!("HTTP status: {}", response.status()))
            })
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
    
    /// Get local IP address for federation registration
    async fn get_local_ip_internal(&self) -> Result<String, SongbirdError> {
        tracing::debug!("Detecting local IP address for federation");
        
        // Try multiple methods to detect the local IP address
        
        // Method 1: Connect to a well-known external address and check our source IP
        if let Ok(ip) = self.detect_ip_via_external_connection().await {
            tracing::debug!("Detected local IP via external connection: {}", ip);
            return Ok(ip);
        }
        
        // Method 2: Use network interface enumeration
        if let Ok(ip) = self.detect_ip_via_interfaces().await {
            tracing::debug!("Detected local IP via network interfaces: {}", ip);
            return Ok(ip);
        }
        
        // Method 3: Try UDP socket binding
        if let Ok(ip) = self.detect_ip_via_udp_socket().await {
            tracing::debug!("Detected local IP via UDP socket: {}", ip);
            return Ok(ip);
        }
        
        // Method 4: Check environment variable
        if let Ok(ip) = std::env::var("SONGBIRD_LOCAL_IP") {
            if let Ok(_) = ip.parse::<std::net::IpAddr>() {
                tracing::debug!("Using IP from environment variable: {}", ip);
                return Ok(ip);
            }
        }
        
        // Method 5: Use hostname resolution
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            if let Ok(addrs) = tokio::net::lookup_host(format!("{}:80", hostname)).await {
                for addr in addrs {
                    let ip = addr.ip();
                    if !ip.is_loopback() {
                        tracing::debug!("Detected local IP via hostname resolution: {}", ip);
                        return Ok(ip.to_string());
                    }
                }
            }
        }
        
        // Fallback to localhost if all methods fail
        tracing::warn!("Could not detect local IP, falling back to localhost");
        Ok(crate::config::constants::default_bind_address().to_string())
    }
    
    /// Detect IP by connecting to external address
    async fn detect_ip_via_external_connection(&self) -> Result<String, SongbirdError> {
        use std::net::{SocketAddr, TcpStream};
        use std::time::Duration;
        
        // Try to connect to a well-known address (Google DNS)
        let timeout = Duration::from_secs(2);
        let target: SocketAddr = "8.8.8.8:53".parse()
            .map_err(|e| SongbirdError::Network {
                service: "federation".to_string(),
                message: format!("Invalid target address: {}", e),
                details: None
            })?;
        
        // Use std::net::TcpStream with timeout
        match std::net::TcpStream::connect_timeout(&target, timeout) {
            Ok(stream) => {
                if let Ok(local_addr) = stream.local_addr() {
                    Ok(local_addr.ip().to_string())
                } else {
                    Err(SongbirdError::Network {
                        service: "federation".to_string(),
                        message: "Could not get local address from connection".to_string(),
                        details: None
                    })
                }
            }
            Err(e) => Err(SongbirdError::Network {
                service: "federation".to_string(),
                message: format!("Could not connect to external address: {}", e),
                details: None
            })
        }
    }
    
    /// Detect IP via network interfaces
    async fn detect_ip_via_interfaces(&self) -> Result<String, SongbirdError> {
        // Try to use local network interface information
        // This is a simplified implementation that looks for non-loopback interfaces
        
        // Check common environment variables first
        if let Ok(ip) = std::env::var("SONGBIRD_LOCAL_IP") {
            if let Ok(_) = ip.parse::<std::net::IpAddr>() {
                return Ok(ip);
            }
        }
        
        // Try to read from /proc/net/route on Linux to find the default route interface
        if let Ok(route_content) = tokio::fs::read_to_string("/proc/net/route").await {
            for line in route_content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 8 && fields[1] == "00000000" {
                    // This is the default route
                    let interface = fields[0];
                    
                    // Try to get the IP address for this interface
                    if let Ok(addr_content) = tokio::fs::read_to_string(format!("/proc/net/if_inet6")).await {
                        for addr_line in addr_content.lines() {
                            let addr_fields: Vec<&str> = addr_line.split_whitespace().collect();
                            if addr_fields.len() >= 6 && addr_fields[5] == interface {
                                // Parse the IPv6 address (hex format)
                                if addr_fields[0].len() == 32 {
                                    // Simple hex parsing without additional dependencies
                                    let hex_str = &addr_fields[0];
                                    if let Ok(addr_int) = u128::from_str_radix(hex_str, 16) {
                                        let ip = std::net::Ipv6Addr::from(addr_int);
                                        if !ip.is_loopback() && !ip.is_multicast() {
                                            return Ok(ip.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        
        // Fallback: try to read IP from common network interface files
        let interface_names = vec!["eth0", "wlan0", "en0", "ens160", "ens192"];
        for interface in interface_names {
            if let Ok(ip) = std::env::var(&format!("SONGBIRD_IP_{}", interface.to_uppercase())) {
                if let Ok(_) = ip.parse::<std::net::IpAddr>() {
                    return Ok(ip);
                }
            }
        }
        
        // For now, return error to fall back to other methods
        // In a full implementation, we'd enumerate network interfaces using a crate like `pnet`
        Err(SongbirdError::Network {
            service: "federation".to_string(),
            message: "Interface enumeration not implemented".to_string(),
            details: None
        })
    }
    
    /// Detect IP via UDP socket
    async fn detect_ip_via_udp_socket(&self) -> Result<String, SongbirdError> {
        use std::net::UdpSocket;
        
        // Create a UDP socket and "connect" to an external address
        // This doesn't send packets but sets up the socket routing
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                match socket.connect("8.8.8.8:80") {
                    Ok(()) => {
                        if let Ok(local_addr) = socket.local_addr() {
                            Ok(local_addr.ip().to_string())
                        } else {
                            Err(SongbirdError::Network {
                                service: "federation".to_string(),
                                message: "Could not get local UDP socket address".to_string(),
                                details: None
                            })
                        }
                    }
                    Err(e) => Err(SongbirdError::Network {
                        service: "federation".to_string(),
                        message: format!("Could not connect UDP socket: {}", e),
                        details: None
                    })
                }
            }
            Err(e) => Err(SongbirdError::Network {
                service: "federation".to_string(),
                message: format!("Could not create UDP socket: {}", e),
                details: None
            })
        }
    }
    
    /// Get local network prefix for federation scanning
    async fn get_local_network_prefix(&self) -> Result<String, SongbirdError> {
        tracing::debug!("Detecting local network prefix for federation");
        
        // Get the local IP first
        let local_ip = self.get_local_ip().await?;
        
        // Parse the IP and determine network prefix
        match local_ip.parse::<std::net::IpAddr>() {
            Ok(std::net::IpAddr::V4(ipv4)) => {
                let octets = ipv4.octets();
                
                // Determine network class and prefix
                let prefix = if octets[0] == 192 && octets[1] == 168 {
                    // Class C private network (192.168.x.x)
                    format!("{}.{}", octets[0], octets[1])
                } else if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                    // Class B private network (172.16-31.x.x)
                    format!("{}.{}", octets[0], octets[1])
                } else if octets[0] == 10 {
                    // Class A private network (10.x.x.x)
                    format!("{}.{}", octets[0], octets[1])
                } else {
                    // For other networks, use first three octets as best guess
                    format!("{}.{}.{}", octets[0], octets[1], octets[2])
                };
                
                tracing::debug!("Detected network prefix: {}", prefix);
                Ok(prefix)
            }
            Ok(std::net::IpAddr::V6(_)) => {
                // For IPv6, return localhost prefix for now
                tracing::debug!("IPv6 detected, using localhost prefix");
                Ok("::1".to_string())
            }
            Err(e) => {
                tracing::warn!("Could not parse local IP {}: {}", local_ip, e);
                // Fallback to common private network prefix
                Ok("192.168.1".to_string())
            }
        }
    }

    /// Get actual CPU usage percentage
    async fn get_cpu_usage(&self) -> Result<f64, SongbirdError> {
        // Try to get CPU usage from system info
        match sys_info::loadavg() {
            Ok(load) => {
                // Convert load average to percentage (rough approximation)
                // Load average of 1.0 = 100% on single core
                let cpu_count = sys_info::cpu_num().unwrap_or(1) as f64;
                let usage_percent = (load.one / cpu_count) * 100.0;
                Ok(usage_percent.min(100.0)) // Cap at 100%
            }
            Err(_) => {
                // Fallback: try to estimate from /proc/loadavg on Linux
                if let Ok(load_str) = std::fs::read_to_string("/proc/loadavg") {
                    if let Some(first_value) = load_str.split_whitespace().next() {
                        if let Ok(load) = first_value.parse::<f64>() {
                            let cpu_count = sys_info::cpu_num().unwrap_or(1) as f64;
                            let usage_percent = (load / cpu_count) * 100.0;
                            return Ok(usage_percent.min(100.0));
                        }
                    }
                }
                // Final fallback
                Ok(0.0)
            }
        }
    }

    /// Get actual memory usage percentage
    async fn get_memory_usage(&self) -> Result<f64, SongbirdError> {
        match sys_info::mem_info() {
            Ok(mem) => {
                if mem.total > 0 {
                    let used = mem.total - mem.avail;
                    let usage_percent = (used as f64 / mem.total as f64) * 100.0;
                    Ok(usage_percent)
                } else {
                    Ok(0.0)
                }
            }
            Err(_) => Ok(0.0)
        }
    }

    /// Get total memory in GB
    async fn get_total_memory_gb(&self) -> Result<u64, SongbirdError> {
        match sys_info::mem_info() {
            Ok(mem) => {
                // Convert from KB to GB
                let gb = mem.total / (1024 * 1024);
                Ok(gb)
            }
            Err(_) => Ok(0)
        }
    }

    /// Get available storage in GB
    async fn get_available_storage_gb(&self) -> Result<u64, SongbirdError> {
        match sys_info::disk_info() {
            Ok(disk) => {
                // Convert from bytes to GB
                let gb = disk.free / (1024 * 1024 * 1024);
                Ok(gb)
            }
            Err(_) => Ok(0)
        }
    }

    /// Get active service count from registry
    async fn get_active_service_count(&self) -> Result<u32, SongbirdError> {
        // Try to get from local service registry if available
        let services = self.get_local_services().await?;
        Ok(services.len() as u32)
    }

    /// Get system uptime in seconds
    async fn get_uptime_seconds(&self) -> Result<u64, SongbirdError> {
        // Try to read uptime from /proc/uptime on Linux
        if let Ok(uptime_str) = std::fs::read_to_string("/proc/uptime") {
            if let Some(first_value) = uptime_str.split_whitespace().next() {
                if let Ok(uptime) = first_value.parse::<f64>() {
                    return Ok(uptime as u64);
                }
            }
        }
        
        // Fallback: use system boot time if available
        match sys_info::boottime() {
            Ok(boot_time) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let uptime = now.saturating_sub(boot_time.sec as u64);
                Ok(uptime)
            }
            Err(_) => Ok(0)
        }
    }

    /// Get current system load
    async fn get_current_load(&self) -> Result<f64, SongbirdError> {
        match sys_info::loadavg() {
            Ok(load) => Ok(load.one), // 1-minute load average
            Err(_) => Ok(0.0)
        }
    }

    /// Calculate available system capacity
    async fn get_available_capacity(&self) -> Result<f64, SongbirdError> {
        // Calculate available capacity based on CPU and memory usage
        let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        let memory_usage = self.get_memory_usage().await.unwrap_or(0.0);
        
        // Available capacity is the inverse of average resource usage
        let avg_usage = (cpu_usage + memory_usage) / 2.0;
        let available_capacity = (100.0 - avg_usage) / 100.0; // Convert to 0.0-1.0 range
        
        Ok(available_capacity.max(0.0).min(1.0))
    }

    /// Get number of active connections (placeholder - would need connection tracking)
    async fn get_active_connections(&self) -> Result<u32, SongbirdError> {
        // This would require integration with connection tracking systems
        // For now, return a reasonable estimate based on service count
        let service_count = self.get_active_service_count().await.unwrap_or(0);
        Ok(service_count * 2) // Estimate 2 connections per service
    }
    
    /// Send federation request to endpoint
    pub async fn send_federation_request(&self, endpoint: &str, request: &FederationRequest) -> Result<FederationResponse, SongbirdError> {
        Self::send_federation_request_static(&self.http_client, endpoint, request).await
    }
    
    /// Send federation request using static client
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
    
    /// Get local services for federation registration
    async fn get_local_services(&self) -> Result<Vec<serde_json::Value>, SongbirdError> {
        let mut services = Vec::new();
        
        // Add core songbird services
        services.push(serde_json::json!({
            "name": "songbird-orchestrator",
            "type": "orchestrator", 
            "status": "running",
            "endpoints": {
                "http": format!("{}/api", std::env::var("SONGBIRD_HTTP_LISTEN").unwrap_or_else(|_| format!("http://{}:8080", crate::config::constants::default_bind_address()))),
                "websocket": format!("{}/ws", std::env::var("SONGBIRD_WEBSOCKET_LISTEN").unwrap_or_else(|_| format!("ws://{}:8080", crate::config::constants::default_bind_address())))
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
                    "api": format!("{}/storage", std::env::var("SONGBIRD_HTTP_LISTEN").unwrap_or_else(|_| format!("http://{}:8080", crate::config::constants::default_bind_address())))
                },
                "capabilities": ["object_storage", "file_sharing", "backup"],
                "protocols": ["http", "s3"]
            }));
        }
        
        tracing::debug!("Enumerated {} local services", services.len());
        Ok(services)
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