//! CLI Discovery Utilities
//!
//! Network discovery utilities for the CLI

use crate::cli::CliError;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;

/// Discovered node information
#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
    pub version: Option<String>,
    pub node_type: NodeType,
    pub response_time_ms: u64,
}

/// Types of discovered nodes
#[derive(Debug, Clone)]
pub enum NodeType {
    Orchestrator,
    ServiceNode,
    Unknown,
}

/// Network scanner for discovering Songbird nodes
pub struct NetworkScanner {
    timeout: Duration,
}

impl NetworkScanner {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Scan a subnet for Songbird nodes
    pub async fn scan_subnet(&self, subnet: &str) -> Result<Vec<DiscoveredNode>, CliError> {
        // Check if we should use simulation mode
        let simulation_mode = std::env::var("SONGBIRD_DISCOVERY_SIMULATION")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if simulation_mode {
            return Ok(self.generate_simulated_nodes(subnet));
        }

        // Real subnet scanning implementation
        let mut discovered_nodes = Vec::new();
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let common_ports = [
            env_config.bind_port,
            env_config.dashboard_port,
            env_config.metrics_port,
            4000, 5000, 8000  // Keep some common fallback ports
        ];

        // Parse subnet (e.g., "192.168.1" -> scan 192.168.1.1-254)
        let subnet_parts: Vec<&str> = subnet.split('.').collect();
        if subnet_parts.len() != 3 {
            return Err(CliError::Network(
                "Invalid subnet format. Use format like '192.168.1'".to_string(),
            ));
        }

        let mut scan_tasks = Vec::new();

        // Scan each host in the subnet
        for host in 1..=254 {
            let target_ip = format!("{}.{}", subnet, host);

            if let Ok(ip) = target_ip.parse::<IpAddr>() {
                for &port in &common_ports {
                    let task = self.scan_address_with_timeout(ip, port);
                    scan_tasks.push(task);
                }
            }
        }

        // Execute scans with timeout
        let scan_timeout = Duration::from_secs(30); // Total scan timeout
        let results =
            tokio::time::timeout(scan_timeout, futures_util::future::join_all(scan_tasks)).await;

        match results {
            Ok(results) => {
                for node in results.into_iter().filter_map(|r| r.ok()).flatten() {
                    discovered_nodes.push(node);
                }
            }
            Err(_) => {
                return Err(CliError::Network("Subnet scan timed out".to_string()));
            }
        }

        Ok(discovered_nodes)
    }

    /// Scan a specific address
    pub async fn scan_address(
        &self,
        address: IpAddr,
        port: u16,
    ) -> Result<Option<DiscoveredNode>, CliError> {
        // Check if we should use simulation mode
        let simulation_mode = std::env::var("SONGBIRD_DISCOVERY_SIMULATION")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        if simulation_mode {
            // Generate simulated response for this address
            return Ok(Some(DiscoveredNode {
                name: format!("🎭 [SIM] Node-{}", address),
                address,
                port,
                version: Some("1.0.0-sim".to_string()),
                node_type: NodeType::ServiceNode,
                response_time_ms: 10 + (port as u64 % 50), // Deterministic "response time"
            }));
        }

        self.scan_address_with_timeout(address, port).await
    }

    /// Internal method to scan address with timeout
    async fn scan_address_with_timeout(
        &self,
        address: IpAddr,
        port: u16,
    ) -> Result<Option<DiscoveredNode>, CliError> {
        let socket_addr = SocketAddr::new(address, port);
        let start_time = std::time::Instant::now();

        // Try to connect to the address
        match tokio::time::timeout(self.timeout, TcpStream::connect(socket_addr)).await {
            Ok(Ok(_stream)) => {
                let response_time = start_time.elapsed().as_millis() as u64;

                // Try to identify if it's a Songbird node
                match self.identify_songbird_node(address, port).await {
                    Ok(Some(node_info)) => Ok(Some(DiscoveredNode {
                        name: node_info.0,
                        address,
                        port,
                        version: node_info.1,
                        node_type: node_info.2,
                        response_time_ms: response_time,
                    })),
                    Ok(None) => Ok(None), // Not a Songbird node
                    Err(_) => {
                        // Connection succeeded but couldn't identify - assume it's a node
                        Ok(Some(DiscoveredNode {
                            name: format!("Unknown-{}", address),
                            address,
                            port,
                            version: None,
                            node_type: NodeType::Unknown,
                            response_time_ms: response_time,
                        }))
                    }
                }
            }
            Ok(Err(_)) => Ok(None), // Connection failed
            Err(_) => Ok(None),     // Timeout
        }
    }

    /// Try to identify if a node is running Songbird
    async fn identify_songbird_node(
        &self,
        address: IpAddr,
        port: u16,
    ) -> Result<Option<(String, Option<String>, NodeType)>, CliError> {
        // Use the real HTTP implementation for comprehensive service identification
        match self.check_songbird_endpoint(address, port).await? {
            Some((name, version, node_type)) => {
                Ok(Some((name, Some(version), node_type)))
            }
            None => {
                // Fallback: Try basic HTTP endpoints
                let endpoints = [
                    "/health",
                    "/api/v1/health", 
                    "/status",
                    "/api/status",
                    "/songbird/health",
                ];

                for endpoint in &endpoints {
                    let url = format!("http://{}:{}{}", address, port, endpoint);

                    if let Ok(_) = self.simulate_http_check(&url).await {
                        // Found a responsive HTTP service - assume it's a Songbird node
                        let node_type = if endpoint.contains("orchestrator") || port == 8080 {
                            NodeType::Orchestrator
                        } else {
                            NodeType::ServiceNode
                        };
                        
                        return Ok(Some((
                            format!("songbird-{}", address), 
                            Some("unknown".to_string()),
                            node_type
                        )));
                    }
                }

                Ok(None)
            }
        }
    }

    /// Real HTTP check implementation
    async fn simulate_http_check(&self, url: &str) -> Result<(), CliError> {
        // Real HTTP client implementation using hyper
        use hyper::{Client, Uri, Body, Request};
        use hyper_tls::HttpsConnector;
        use std::time::Instant;

        let start = Instant::now();
        
        // Create HTTPS client
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);

        // Parse URL
        let uri: Uri = url.parse()
            .map_err(|e| CliError::Network(format!("Invalid URL {}: {}", url, e)))?;

        // Create GET request
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("User-Agent", "Songbird-Discovery/1.0")
            .header("Accept", "application/json, text/plain")
            .body(Body::empty())
            .map_err(|e| CliError::Network(format!("Failed to build request: {}", e)))?;

        // Execute request with timeout
        let response_future = client.request(req);
        let response = tokio::time::timeout(self.timeout, response_future).await
            .map_err(|_| CliError::Network("HTTP request timeout".to_string()))?
            .map_err(|e| CliError::Network(format!("HTTP request failed: {}", e)))?;

        let elapsed = start.elapsed();
        tracing::debug!("HTTP check for {} completed in {:?}", url, elapsed);

        // Check response status
        if response.status().is_success() {
            Ok(())
        } else {
            Err(CliError::Network(format!("HTTP check failed with status: {}", response.status())))
        }
    }

    /// Real HTTP client implementation for service identification
    async fn check_songbird_endpoint(&self, address: IpAddr, port: u16) -> Result<Option<(String, String, NodeType)>, CliError> {
        let endpoints = [
            format!("http://{}:{}/health", address, port),
            format!("http://{}:{}/api/v1/info", address, port),
            format!("http://{}:{}/status", address, port),
            format!("https://{}:{}/health", address, port),
        ];

        for endpoint in &endpoints {
            match self.try_songbird_endpoint(endpoint).await {
                Ok(Some(info)) => return Ok(Some(info)),
                Ok(None) => continue,
                Err(_) => continue, // Try next endpoint
            }
        }

        Ok(None)
    }

    /// Try a specific Songbird endpoint
    async fn try_songbird_endpoint(&self, url: &str) -> Result<Option<(String, String, NodeType)>, CliError> {
        use hyper::{Client, Uri, Body, Request};
        use hyper_tls::HttpsConnector;

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);

        let uri: Uri = url.parse()
            .map_err(|e| CliError::Network(format!("Invalid URL {}: {}", url, e)))?;

        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("User-Agent", "Songbird-Discovery/1.0")
            .header("Accept", "application/json")
            .body(Body::empty())
            .map_err(|e| CliError::Network(format!("Failed to build request: {}", e)))?;

        let response = tokio::time::timeout(
            Duration::from_secs(2), // Shorter timeout for discovery
            client.request(req)
        ).await
            .map_err(|_| CliError::Network("Request timeout".to_string()))?
            .map_err(|e| CliError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        // Read response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await
            .map_err(|e| CliError::Network(format!("Failed to read response: {}", e)))?;

        let body_str = String::from_utf8_lossy(&body_bytes);

        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
            let name = json.get("service")
                .or_else(|| json.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("songbird-node")
                .to_string();

            let version = self.extract_version_from_response(&body_str)
                .unwrap_or_else(|| "unknown".to_string());

            let node_type = if json.get("orchestrator").is_some() || 
                             json.get("type").and_then(|v| v.as_str()) == Some("orchestrator") {
                NodeType::Orchestrator
            } else {
                NodeType::ServiceNode
            };

            return Ok(Some((name, version, node_type)));
        }

        // Fallback: Check for Songbird signatures in plain text
        if body_str.to_lowercase().contains("songbird") {
            let name = "songbird-node".to_string();
            let version = self.extract_version_from_response(&body_str)
                .unwrap_or_else(|| "unknown".to_string());
            
            return Ok(Some((name, version, NodeType::Unknown)));
        }

        Ok(None)
    }

    /// Extract version from API response
    #[allow(dead_code)]
    fn extract_version_from_response(&self, response: &str) -> Option<String> {
        // Try to parse JSON response for version
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                return Some(version.to_string());
            }
            if let Some(version) = json.get("build_version").and_then(|v| v.as_str()) {
                return Some(version.to_string());
            }
        }

        // Try to find version in plain text
        for line in response.lines() {
            if line.to_lowercase().contains("version") && line.contains(":") {
                if let Some(version_part) = line.split(':').nth(1) {
                    return Some(version_part.trim().to_string());
                }
            }
        }

        None
    }

    /// Generate simulated nodes for testing/demo purposes
    fn generate_simulated_nodes(&self, subnet: &str) -> Vec<DiscoveredNode> {
        let node_count = std::env::var("SONGBIRD_SIM_NODE_COUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3); // Default 3 simulated nodes

        let mut nodes = Vec::new();
        for i in 1..=node_count {
            let host_ip = format!("{}.{}", subnet, 100 + i);
            if let Ok(address) = host_ip.parse::<IpAddr>() {
                // Use environment configuration - NO MORE HARDCODING!
                let env_config = crate::config::environment::EnvironmentConfig::default();
                nodes.push(DiscoveredNode {
                    name: format!("🎭 [SIM] Songbird-Node-{}", i),
                    address,
                    port: env_config.bind_port + (i as u16 % 10),
                    version: Some(format!("1.0.{}-sim", i)),
                    node_type: if i == 1 {
                        NodeType::Orchestrator
                    } else {
                        NodeType::ServiceNode
                    },
                    response_time_ms: 5 + (i * 7) % 30, // Deterministic response times
                });
            }
        }
        nodes
    }
}
