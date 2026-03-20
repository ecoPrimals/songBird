// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! CLI Discovery Utilities
//!
//! Network discovery utilities for the CLI

#![allow(missing_docs, reason = "discovery probe structs are internal to the CLI")]

use crate::errors::{CliError, SongbirdResult};
use songbird_types::SafeEnv;
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
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
        }
    }

    /// Scan a subnet for Songbird nodes
    pub async fn scan_subnet(&self, subnet: &str) -> SongbirdResult<Vec<DiscoveredNode>> {
        // Check if we should use simulation mode
        let simulation_mode = SafeEnv::get_bool("SONGBIRD_DISCOVERY_SIMULATION", false);

        if simulation_mode {
            return Ok(self.generate_simulated_nodes(subnet));
        }

        // Real subnet scanning implementation
        let mut discovered_nodes = Vec::new();
        // Common service ports - should be discovered via capability endpoints
        let common_ports = [
            SafeEnv::get_port(
                "DISCOVERY_PORT_1",
                songbird_config::defaults::ports::orchestrator_port(),
            ),
            SafeEnv::get_port("DISCOVERY_PORT_2", songbird_config::defaults::ports::metrics_port()),
            SafeEnv::get_port(
                "DISCOVERY_PORT_3",
                songbird_config::defaults::ports::dashboard_port(),
            ),
            4000, // External service port
            5000, // External service port
            8000, // External service port
        ];

        // Parse subnet (e.g., "192.168.1" -> scan 192.168.1.1-254)"
        let subnet_parts: Vec<&str> = subnet.split('.').collect();
        if subnet_parts.len() != 3 {
            return Err(CliError::Network {
                message: "Invalid subnet format. Use format like '192.168.1'".to_string(),
                interface: Some(subnet.to_string()),
                suggestion: Some("Provide a valid subnet in the format 'xxx.xxx.xxx'".to_string()),
            }
            .into());
        }

        let mut scan_tasks = Vec::new();

        // Scan each host in the subnet
        for host in 1..=254 {
            let target_ip = format!("{subnet}.{host}");

            if let Ok(ip) = target_ip.parse::<IpAddr>() {
                for &port in &common_ports {
                    let task = self.scan_address_with_timeout(ip, port);
                    scan_tasks.push(task);
                }
            }
        }

        // Execute scans with timeout
        // Use configurable timeout instead of hardcoded 30 seconds
        let scan_timeout = std::time::Duration::from_millis(
            songbird_config::canonical::constants::get_connection_timeout_ms(),
        );
        let results =
            tokio::time::timeout(scan_timeout, futures_util::future::join_all(scan_tasks)).await;

        match results {
            Ok(results) => {
                for node in results.into_iter().filter_map(|r| r.ok().flatten()) {
                    discovered_nodes.push(node);
                }
            }
            Err(_) => {
                return Err(CliError::Network {
                    message: "Subnet scan timed out".to_string(),
                    interface: Some(subnet.to_string()),
                    suggestion: Some(
                        "Try increasing the timeout or checking network connectivity".to_string(),
                    ),
                }
                .into());
            }
        }

        Ok(discovered_nodes)
    }

    /// Scan a specific address
    pub async fn scan_address(
        &self,
        address: IpAddr,
        port: u16,
    ) -> SongbirdResult<Option<DiscoveredNode>> {
        // Check if we should use simulation mode
        let simulation_mode = SafeEnv::get_bool("SONGBIRD_DISCOVERY_SIMULATION", false);

        if simulation_mode {
            // Generate simulated response for this address
            return Ok(Some(DiscoveredNode {
                name: format!("🎭 [SIM] Node-{address}"),
                address,
                port,
                version: Some("1.0.0-sim".to_string()),
                node_type: NodeType::ServiceNode,
                response_time_ms: 10 + (u64::from(port) % 50), // Deterministic "response time"
            }));
        }

        self.scan_address_with_timeout(address, port).await
    }

    /// Internal method to scan address with timeout
    async fn scan_address_with_timeout(
        &self,
        address: IpAddr,
        port: u16,
    ) -> SongbirdResult<Option<DiscoveredNode>> {
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
                            name: format!("Unknown-{address}"),
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
    ) -> SongbirdResult<Option<(String, Option<String>, NodeType)>> {
        // Try common Songbird endpoints
        let endpoints = ["/health", "/api/v1/health", "/status", "/api/status", "/songbird/health"];

        for endpoint in &endpoints {
            let url = format!("http://{address}:{port}{endpoint}");

            // For now, we'll simulate the HTTP check since we don't have the HTTP client implemented
            // In a real implementation, you would use an HTTP client here
            if let Ok(_response) = self.simulate_http_check(&url).await {
                let node_type = NodeType::ServiceNode;
                let version = Some("1.0.0".to_string());

                return Ok(Some((format!("Songbird-{address}"), version, node_type)));
            }
        }

        Ok(None)
    }

    /// Simulate HTTP check for now
    async fn simulate_http_check(&self, _url: &str) -> SongbirdResult<()> {
        // This is a placeholder - in real implementation would use HTTP client
        Err(CliError::Network {
            message: "HTTP client not implemented".to_string(),
            interface: Some("http_client".to_string()),
            suggestion: Some("This feature is not yet implemented".to_string()),
        }
        .into())
    }

    /// Extract version from API response
    #[expect(
        dead_code,
        reason = "reserved for version parsing when discovery HTTP client is wired"
    )]
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
            if line.to_lowercase().contains("version")
                && line.contains(':')
                && let Some(version_part) = line.split(':').nth(1)
            {
                return Some(version_part.trim().to_string());
            }
        }

        None
    }

    /// Generate simulated nodes for testing/demo purposes
    fn generate_simulated_nodes(&self, subnet: &str) -> Vec<DiscoveredNode> {
        let node_count = SafeEnv::get_usize("SONGBIRD_SIM_NODE_COUNT", 3); // Default 3 simulated nodes

        let mut nodes = Vec::new();
        for i in 1..=node_count {
            let host_ip = format!("{}.{}", subnet, 100 + i);
            if let Ok(address) = host_ip.parse::<IpAddr>() {
                // Use canonical configuration - NO MORE HARDCODING!
                let config = songbird_types::config::CanonicalSongbirdConfig::default();
                nodes.push(DiscoveredNode {
                    name: format!("🎭 [SIM] Songbird-Node-{i}"),
                    address,
                    port: config.network.base_port + (i as u16 % 10),
                    version: Some(format!("1.0.{i}-sim")),
                    node_type: if i == 1 {
                        NodeType::Orchestrator
                    } else {
                        NodeType::ServiceNode
                    },
                    response_time_ms: (5 + (i * 7) % 30) as u64, // Deterministic response times
                });
            }
        }
        nodes
    }
}
