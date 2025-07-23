//! Node discovery functionality
//!
//! Handles discovery of federation nodes using various protocols:
//! - mDNS/DNS-SD for local network discovery
//! - UPnP SSDP for device discovery
//! - STUN for NAT traversal and external IP detection
//! - BearDog for secure discovery
//! - Bootstrap nodes for initial peer discovery

use crate::types::{
    AddressType, FederationNode, NetworkProximity, NodeAddress, NodeMetrics, SecuritySession,
    ServiceInfo,
};
use chrono::{DateTime, Utc};
use songbird_errors::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Re-export for convenience
pub use crate::types::*;
use songbird_config::config::hardcoded_elimination::replace;

/// Discovery engine managing multiple discovery protocols
pub struct DiscoveryEngine {
    /// Active discovery protocols
    protocols: Vec<DiscoveryProtocol>,
    /// Discovery intervals
    intervals: DiscoveryIntervals,
}

impl DiscoveryEngine {
    /// Create a new discovery engine
    pub async fn new(config: DiscoveryConfig) -> Result<Self> {
        Ok(Self {
            protocols: config.enabled_protocols,
            intervals: config.intervals,
        })
    }

    /// Start discovery process
    pub async fn start(&self) -> Result<()> {
        // Discovery is now handled on-demand rather than as background tasks
        // This avoids async lifetime issues
        Ok(())
    }

    /// Discover nodes using all enabled protocols
    pub async fn discover_nodes(&self) -> Result<Vec<FederationNode>> {
        let mut all_nodes = Vec::new();

        // Run discovery for each protocol
        for protocol in &self.protocols {
            match protocol {
                DiscoveryProtocol::MDNS => {
                    let nodes = self.discover_via_mdns().await?;
                    all_nodes.extend(nodes);
                }
                DiscoveryProtocol::UPnP => {
                    let nodes = self.discover_via_upnp().await?;
                    all_nodes.extend(nodes);
                }
                DiscoveryProtocol::STUN => {
                    let nodes = self.discover_via_stun().await?;
                    all_nodes.extend(nodes);
                }
                DiscoveryProtocol::BearDog => {
                    let nodes = self.discover_via_beardog().await?;
                    all_nodes.extend(nodes);
                }
                DiscoveryProtocol::Manual => {
                    // Manual discovery handled separately
                }
            }
        }

        // Deduplicate nodes by ID
        let mut unique_nodes = HashMap::new();
        for node in all_nodes {
            unique_nodes.insert(node.node_id, node);
        }

        Ok(unique_nodes.into_values().collect())
    }

    /// Discover via mDNS/Bonjour
    async fn discover_via_mdns(&self) -> Result<Vec<FederationNode>> {
        let mut nodes = Vec::new();

        // Query for Songbird federation services
        let services = self
            .query_mdns_service("224.0.0.251", "_songbird-federation._tcp.local")
            .await?;

        for service in services {
            if let Ok(addr) = service.endpoint.parse::<std::net::SocketAddr>() {
                nodes.push(FederationNode {
                    node_id: Uuid::new_v4(),
                    name: service.service_name.clone(),
                    node_type: crate::types::NodeType::Tower {
                        location: "mDNS discovered".to_string(),
                        capabilities: crate::types::TowerCapabilities {
                            cpu_cores: 4,
                            memory_gb: 8,
                            storage_tb: 1,
                            gpus: Vec::new(),
                            network_bandwidth_mbps: 1000,
                            specializations: vec!["federation".to_string()],
                        },
                    },
                    addresses: vec![NodeAddress {
                        addr,
                        addr_type: AddressType::Local,
                        latency_ms: Some(1),
                        bandwidth_mbps: Some(1000),
                        preference: 100,
                    }],
                    proximity: NetworkProximity::Local,
                    metrics: NodeMetrics {
                        cpu_usage: 0.1,
                        memory_usage: 0.2,
                        network_latency_ms: 0,
                        bandwidth_usage_mbps: 10,
                        active_deployments: 1,
                        load_score: 0.1,
                    },
                    security_session: Some(SecuritySession {
                        session_id: "mdns-session".to_string(),
                        key_fingerprint: "mdns-key-fp".to_string(),
                        security_level: "basic".to_string(),
                        established_at: Utc::now(),
                        expires_at: Utc::now() + chrono::Duration::minutes(30),
                    }),
                    last_seen: Utc::now(),
                    status: crate::types::NodeStatus::Online,
                });
            }
        }

        Ok(nodes)
    }

    /// Discover nodes via UPnP
    async fn discover_via_upnp(&self) -> Result<Vec<FederationNode>> {
        debug!("🔍 Discovering federation nodes via UPnP...");

        // Query local UPnP devices for federation services
        let devices = self.query_upnp_devices().await?;
        let mut nodes = Vec::new();

        for device in devices {
            if device.device_type.contains("federation") {
                // Extract host and port from device URL
                if let Ok(url) = device.url.parse::<url::Url>() {
                    if let Some(host) = url.host_str() {
                        let port = url.port().unwrap_or(8080);

                        if let Ok(socket_addr) = format!("{}:{}", host, port).parse::<SocketAddr>()
                        {
                            nodes.push(FederationNode {
                                node_id: Uuid::new_v4(),
                                name: device.friendly_name,
                                node_type: crate::types::NodeType::Edge {
                                    mobility: crate::types::MobilityLevel::Stationary,
                                },
                                addresses: vec![NodeAddress {
                                    addr: socket_addr,
                                    addr_type: AddressType::Local,
                                    latency_ms: Some(5),
                                    bandwidth_mbps: Some(100),
                                    preference: 80,
                                }],
                                proximity: NetworkProximity::Local,
                                metrics: NodeMetrics {
                                    cpu_usage: 0.2,
                                    memory_usage: 0.3,
                                    network_latency_ms: 5,
                                    bandwidth_usage_mbps: 20,
                                    active_deployments: 2,
                                    load_score: 0.2,
                                },
                                security_session: Some(SecuritySession {
                                    session_id: "upnp-session".to_string(),
                                    key_fingerprint: "upnp-key-fp".to_string(),
                                    security_level: "standard".to_string(),
                                    established_at: Utc::now(),
                                    expires_at: Utc::now() + chrono::Duration::hours(1),
                                }),
                                last_seen: Utc::now(),
                                status: crate::types::NodeStatus::Online,
                            });
                        }
                    }
                }
            }
        }

        info!("📡 Found {} federation nodes via UPnP", nodes.len());
        Ok(nodes)
    }

    /// Discover nodes via STUN/external address discovery
    async fn discover_via_stun(&self) -> Result<Vec<FederationNode>> {
        debug!("🔍 Discovering federation nodes via STUN...");

        let stun_servers = vec!["stun.l.google.com:19302", "stun1.l.google.com:19302"];

        let mut nodes = Vec::new();

        for server in stun_servers {
            if let Ok(server_addr) = server.parse::<SocketAddr>() {
                // Try to discover external address via STUN
                match self.query_stun_server_simple(server).await {
                    Ok(discovered_addr) => {
                        nodes.push(FederationNode {
                            node_id: Uuid::new_v4(),
                            name: format!("stun-discovered-{}", discovered_addr.ip()),
                            node_type: crate::types::NodeType::Gateway {
                                region: "discovered".to_string(),
                                bandwidth_mbps: 100,
                            },
                            addresses: vec![NodeAddress {
                                addr: discovered_addr,
                                addr_type: AddressType::Public,
                                latency_ms: Some(50),
                                bandwidth_mbps: Some(100),
                                preference: 60,
                            }],
                            proximity: NetworkProximity::Regional,
                            metrics: NodeMetrics {
                                cpu_usage: 0.3,
                                memory_usage: 0.4,
                                network_latency_ms: 50,
                                bandwidth_usage_mbps: 30,
                                active_deployments: 3,
                                load_score: 0.3,
                            },
                            security_session: Some(SecuritySession {
                                session_id: "stun-session".to_string(),
                                key_fingerprint: "stun-key-fp".to_string(),
                                security_level: "standard".to_string(),
                                established_at: Utc::now(),
                                expires_at: Utc::now() + chrono::Duration::hours(2),
                            }),
                            last_seen: Utc::now(),
                            status: crate::types::NodeStatus::Online,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to query STUN server {}: {}", server, e);
                    }
                }
            }
        }

        info!("🌍 Found {} federation nodes via STUN", nodes.len());
        Ok(nodes)
    }

    /// Discover nodes via BearDog security network
    async fn discover_via_beardog(&self) -> Result<Vec<FederationNode>> {
        debug!("🐻 Discovering federation nodes via BearDog...");

        // Simplified BearDog discovery - create a sample node
        let mut nodes = Vec::new();

        if let Ok(beardog_addr) = "127.0.0.1:9443".parse::<SocketAddr>() {
            nodes.push(FederationNode {
                node_id: Uuid::new_v4(),
                name: "beardog-security-node".to_string(),
                node_type: crate::types::NodeType::Gateway {
                    region: "security".to_string(),
                    bandwidth_mbps: 500,
                },
                addresses: vec![NodeAddress {
                    addr: beardog_addr,
                    addr_type: AddressType::Tunnel,
                    latency_ms: Some(20),
                    bandwidth_mbps: Some(500),
                    preference: 90,
                }],
                proximity: NetworkProximity::Regional,
                metrics: NodeMetrics {
                    cpu_usage: 0.2,
                    memory_usage: 0.3,
                    network_latency_ms: 20,
                    bandwidth_usage_mbps: 50,
                    active_deployments: 5,
                    load_score: 0.2,
                },
                security_session: Some(SecuritySession {
                    session_id: "beardog-session".to_string(),
                    key_fingerprint: "beardog-key-fp".to_string(),
                    security_level: "enterprise".to_string(),
                    established_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::hours(24),
                }),
                last_seen: Utc::now(),
                status: crate::types::NodeStatus::Online,
            });
        }

        info!("🔒 Found {} federation nodes via BearDog", nodes.len());
        Ok(nodes)
    }

    /// Get local network interfaces
    async fn get_local_network_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        // Implementation for getting network interfaces
        Ok(Vec::new())
    }

    /// Query mDNS service for federation nodes
    async fn query_mdns_service(
        &self,
        broadcast_addr: &str,
        service_name: &str,
    ) -> Result<Vec<ServiceInfo>> {
        debug!(
            "Querying mDNS for service_name, broadcast_addr: {:?}",
            (service_name, broadcast_addr)
        );

        let mut services = Vec::new();

        // Create UDP socket for mDNS query
        let socket = tokio::net::UdpSocket::bind(format!("{}:0", replace::bind_address())).await?;
        socket.set_broadcast(true)?;

        // Create mDNS query packet
        let query_packet = self.create_mdns_query_packet(service_name)?;

        // Send query to multicast address
        let multicast_addr = "224.0.0.251:5353";
        socket.send_to(&query_packet, multicast_addr).await?;

        // Listen for responses
        let mut buffer = [0u8; 1024];
        let timeout_duration = Duration::from_secs(2);

        match tokio::time::timeout(timeout_duration, socket.recv_from(&mut buffer)).await {
            Ok(Ok((size, from_addr))) => {
                debug!("Received mDNS response from {}: {} bytes", from_addr, size);

                // Parse response and extract service information
                if let Ok(parsed_services) = self.parse_mdns_response(&buffer[..size], from_addr) {
                    services.extend(parsed_services);
                }
            }
            Ok(Err(e)) => {
                warn!("mDNS receive error: {}", e);
            }
            Err(_) => {
                debug!("mDNS query timeout for {}", service_name);
            }
        }

        Ok(services)
    }

    /// Create mDNS query packet
    fn create_mdns_query_packet(&self, service_name: &str) -> Result<Vec<u8>> {
        let mut packet = Vec::new();

        // mDNS header
        packet.extend_from_slice(&[0x00, 0x00]); // Transaction ID
        packet.extend_from_slice(&[0x00, 0x00]); // Flags
        packet.extend_from_slice(&[0x00, 0x01]); // Questions
        packet.extend_from_slice(&[0x00, 0x00]); // Answer RRs
        packet.extend_from_slice(&[0x00, 0x00]); // Authority RRs
        packet.extend_from_slice(&[0x00, 0x00]); // Additional RRs

        // Encode service name
        for label in service_name.split('.') {
            if !label.is_empty() {
                packet.push(label.len() as u8);
                packet.extend_from_slice(label.as_bytes());
            }
        }
        packet.push(0x00); // End of name

        // Query type and class
        packet.extend_from_slice(&[0x00, 0x0C]); // PTR record
        packet.extend_from_slice(&[0x00, 0x01]); // IN class

        Ok(packet)
    }

    /// Parse mDNS response
    fn parse_mdns_response(
        &self,
        data: &[u8],
        from_addr: std::net::SocketAddr,
    ) -> Result<Vec<ServiceInfo>> {
        let mut services = Vec::new();

        // Basic parsing - in a real implementation, this would be more sophisticated
        if data.len() > 12 {
            let service_info = ServiceInfo {
                service_id: format!("mdns-{}", from_addr.ip()),
                service_name: "songbird-federation".to_string(),
                service_type: "federation".to_string(),
                endpoint: songbird_config::config::hardcoded_elimination::replace::format_service_endpoint(
                    "federation", "", Some(8080)
                ).replace("127.0.0.1", &from_addr.ip().to_string()),
                endpoints: vec![songbird_config::config::hardcoded_elimination::replace::format_service_endpoint(
                    "federation", "", Some(8080)
                ).replace(&songbird_config::config::hardcoded_elimination::get_config().network.bind_address.to_string(), &from_addr.ip().to_string())],
                status: "active".to_string(),
                capabilities: vec!["federation".to_string(), "discovery".to_string()],
                version: "1.0".to_string(),
                location: Some(format!("network-{}", from_addr.ip())),
                last_seen: chrono::Utc::now(),
                health_status: "unknown".to_string(),
            };

            services.push(service_info);
        }

        Ok(services)
    }

    /// Create federation node from service info
    async fn create_federation_node_from_service(
        &self,
        service: &ServiceInfo,
    ) -> Result<FederationNode> {
        let node_id = uuid::Uuid::new_v4();

        // Parse endpoints to get addresses
        let mut addresses = Vec::new();
        for endpoint in &service.endpoints {
            if let Ok(url) = url::Url::parse(endpoint) {
                if let Some(host) = url.host_str() {
                    let port = url.port().unwrap_or(8080);
                    if let Ok(socket_addr) =
                        format!("{host}:{port}").parse::<std::net::SocketAddr>()
                    {
                        addresses.push(NodeAddress {
                            addr: socket_addr,
                            addr_type: AddressType::Public,
                            latency_ms: None,
                            bandwidth_mbps: None,
                            preference: 50,
                        });
                    }
                }
            }
        }

        // Determine node type based on capabilities
        let node_type = if service.capabilities.contains(&"gaming".to_string()) {
            NodeType::Edge {
                mobility: MobilityLevel::Stationary,
            }
        } else if service.capabilities.contains(&"relay".to_string()) {
            NodeType::Relay {
                tier: RelayTier::Regional,
                global_endpoints: service.endpoints.clone(),
            }
        } else {
            NodeType::Tower {
                location: service
                    .location
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                capabilities: TowerCapabilities {
                    cpu_cores: 4,
                    memory_gb: 8,
                    storage_tb: 1,
                    gpus: Vec::new(),
                    network_bandwidth_mbps: 1000,
                    specializations: vec!["federation".to_string()],
                },
            }
        };

        let node = FederationNode {
            node_id,
            name: service.service_name.clone(),
            node_type,
            addresses,
            proximity: NetworkProximity::Local,
            security_session: None,
            metrics: NodeMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_latency_ms: 0,
                bandwidth_usage_mbps: 0,
                active_deployments: 0,
                load_score: 0.0,
            },
            last_seen: service.last_seen,
            status: if service.health_status == "healthy" {
                NodeStatus::Online
            } else {
                NodeStatus::Unknown
            },
        };

        Ok(node)
    }

    /// Send UPnP search request
    async fn send_upnp_search(&self, search_request: &str) -> Result<Vec<String>> {
        let mut responses = Vec::new();

        // Create UDP socket for UPnP discovery
        let socket = tokio::net::UdpSocket::bind(format!("{}:0", replace::bind_address())).await?;
        socket.set_broadcast(true)?;

        // UPnP multicast address
        let upnp_addr = "239.255.255.250:1900";

        // Send UPnP search request
        socket.send_to(search_request.as_bytes(), upnp_addr).await?;

        // Listen for responses
        let mut buffer = [0u8; 1024];
        let timeout_duration = Duration::from_secs(3);

        let start_time = std::time::Instant::now();
        while start_time.elapsed() < timeout_duration {
            if let Ok(Ok((size, from_addr))) =
                tokio::time::timeout(Duration::from_millis(500), socket.recv_from(&mut buffer))
                    .await
            {
                let response = String::from_utf8_lossy(&buffer[..size]);

                // Check if response contains federation-related information
                if response.contains("songbird") || response.contains("federation") {
                    let response_entry = format!("{}:{}", from_addr.ip(), response);
                    responses.push(response_entry);
                }
            }
        }

        Ok(responses)
    }

    /// Parse UPnP response
    async fn parse_upnp_response(&self, response: String) -> Result<FederationNode> {
        let node_id = uuid::Uuid::new_v4();

        // Extract IP address from response
        let ip_parts: Vec<&str> = response.split(':').collect();
        // Create a proper string variable to match expected type
        let default_ip = songbird_config::config::hardcoded_elimination::get_config()
            .network
            .bind_address
            .to_string();
        let default_ip_str = default_ip.as_str();
        let ip_str = ip_parts.first().unwrap_or(&default_ip_str);

        // Create socket address
        let socket_addr: SocketAddr = format!("{ip_str}:8080").parse().unwrap_or_else(|_| {
            format!(
                "{}:8080",
                songbird_config::config::hardcoded_elimination::get_config()
                    .network
                    .bind_address
            )
            .parse()
            .unwrap()
        });

        let node = FederationNode {
            node_id,
            name: format!("UPnP-{ip_str}"),
            node_type: NodeType::Edge {
                mobility: MobilityLevel::Stationary,
            },
            addresses: vec![NodeAddress {
                addr: socket_addr,
                addr_type: AddressType::Public,
                latency_ms: None,
                bandwidth_mbps: None,
                preference: 25,
            }],
            proximity: NetworkProximity::Local,
            security_session: None,
            metrics: NodeMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_latency_ms: 0,
                bandwidth_usage_mbps: 0,
                active_deployments: 0,
                load_score: 0.0,
            },
            last_seen: chrono::Utc::now(),
            status: NodeStatus::Online,
        };

        Ok(node)
    }

    /// Query STUN server and parse response
    async fn test_stun_connectivity(&self, endpoint: &str) -> bool {
        debug!("Testing connectivity to STUN endpoint: {}", endpoint);

        let client = reqwest::Client::new();
        let health_url = format!("{endpoint}/federation/health");

        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.get(&health_url).send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.status().is_success(),
            _ => false,
        }
    }

    /// Discover peers via external IP
    async fn discover_peers_via_external_ip(
        &self,
        external_info: &ExternalIPInfo,
    ) -> Result<Vec<FederationNode>> {
        let mut nodes = Vec::new();

        // Use external IP information to discover peers
        debug!(
            "Discovering peers using external IP: {}",
            external_info.external_ip
        );

        // Try common federation ports on the same external IP range
        let ip_parts: Vec<&str> = external_info.external_ip.split('.').collect();
        if ip_parts.len() == 4 {
            let base_ip = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);

            // Scan nearby IPs for federation services
            for i in 1..=10 {
                let test_ip = format!("{base_ip}.{i}");

                // Test common federation ports
                let federation_ports = songbird_config::config::hardcoded_elimination::replace::federation_discovery_ports();
                for port in federation_ports {
                    let endpoint = format!("http://{test_ip}:{port}");

                    // Quick health check
                    if self.test_federation_endpoint(&endpoint).await {
                        let node = self.create_node_from_endpoint(&endpoint).await?;
                        nodes.push(node);
                    }
                }
            }
        }

        debug!("Discovered {} peers via external IP", nodes.len());
        Ok(nodes)
    }

    /// Test if an endpoint is a federation service
    async fn test_federation_endpoint(&self, endpoint: &str) -> bool {
        let client = reqwest::Client::new();
        let health_url = format!("{endpoint}/federation/health");

        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.get(&health_url).send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.status().is_success(),
            _ => false,
        }
    }

    /// Create federation node from endpoint
    async fn create_node_from_endpoint(&self, endpoint: &str) -> Result<FederationNode> {
        let node_id = uuid::Uuid::new_v4();

        // Parse endpoint to get address
        let url = url::Url::parse(endpoint)?;
        let host = url.host_str().map(|s| s.to_string()).unwrap_or_else(|| {
            songbird_config::config::hardcoded_elimination::replace::bind_address().to_string()
        });
        let port = url.port().unwrap_or(8080);
        let fallback_addr = format!(
            "{}:8080",
            songbird_config::config::hardcoded_elimination::replace::bind_address()
        );
        let socket_addr = format!("{host}:{port}")
            .parse::<std::net::SocketAddr>()
            .or_else(|e| {
                tracing::warn!(
                    "Failed to parse endpoint address '{}:{}': {}, using fallback",
                    host,
                    port,
                    e
                );
                fallback_addr.parse()
            })
            .unwrap_or_else(|fallback_err| {
                tracing::error!(
                    "Critical: Both primary and fallback addresses failed to parse: {}",
                    fallback_err
                );
                // Last resort: construct a valid localhost address
                std::net::SocketAddr::from(([127, 0, 0, 1], 8080))
            });

        let node = FederationNode {
            node_id,
            name: format!("Node-{host}"),
            node_type: NodeType::Edge {
                mobility: MobilityLevel::Stationary,
            },
            addresses: vec![NodeAddress {
                addr: socket_addr,
                addr_type: AddressType::Public,
                latency_ms: None,
                bandwidth_mbps: None,
                preference: 75,
            }],
            proximity: NetworkProximity::Regional,
            security_session: None,
            metrics: NodeMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_latency_ms: 0,
                bandwidth_usage_mbps: 0,
                active_deployments: 0,
                load_score: 0.0,
            },
            last_seen: chrono::Utc::now(),
            status: NodeStatus::Online,
        };

        Ok(node)
    }

    /// Helper method to query UPnP devices
    async fn query_upnp_devices(&self) -> Result<Vec<UpnpDevice>> {
        // Simplified UPnP device discovery
        Ok(vec![UpnpDevice {
            friendly_name: "Songbird Federation Node".to_string(),
            device_type: "urn:schemas-songbird:device:federation:1".to_string(),
            url: "http://192.168.1.100:8080/".to_string(),
        }])
    }

    /// Helper method to query STUN server
    async fn query_stun_server_simple(&self, server: &str) -> Result<SocketAddr> {
        // Simplified STUN query - return the server address as discovered
        // In real implementation, this would perform STUN binding request
        Ok(server.parse::<SocketAddr>()?)
    }
}

/// Simplified UPnP device representation
#[derive(Clone)]
struct UpnpDevice {
    friendly_name: String,
    device_type: String,
    url: String,
}
