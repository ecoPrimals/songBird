//! Node discovery functionality
//!
//! Handles discovery of federation nodes using various protocols:
//! - mDNS/DNS-SD for local network discovery
//! - UPnP SSDP for device discovery
//! - STUN for NAT traversal and external IP detection
//! - BearDog for secure discovery
//! - Bootstrap nodes for initial peer discovery

use songbird_errors::Result;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn};

use crate::types::*;

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

    /// Discover local nodes on the LAN
    async fn discover_via_mdns(&self) -> Result<Vec<FederationNode>> {
        let interfaces = self.get_local_network_interfaces().await?;
        let mut nodes = Vec::new();

        for interface in &interfaces {
            let services = self
                .query_mdns_service(&interface.broadcast_ip, "_songbird._tcp.local")
                .await?;
            for service in services {
                if let Ok(node) = self.create_federation_node_from_service(&service).await {
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }

    /// Discover nodes via UPnP SSDP
    async fn discover_via_upnp(&self) -> Result<Vec<FederationNode>> {
        let search_request = "M-SEARCH * HTTP/1.1\r\n\
                             HOST: 239.255.255.250:1900\r\n\
                             MAN: \"ssdp:discover\"\r\n\
                             ST: urn:schemas-songbird:device:federation:1\r\n\
                             MX: 3\r\n\r\n";

        let responses = self.send_upnp_search(search_request).await?;
        let mut nodes = Vec::new();

        for response in responses {
            if let Ok(node) = self.parse_upnp_response(response).await {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// Discover nodes via STUN servers
    async fn discover_via_stun(&self) -> Result<Vec<FederationNode>> {
        let stun_servers = vec![
            "stun.l.google.com:19302",
            "stun1.l.google.com:19302",
            "stun2.l.google.com:19302",
        ];

        let mut nodes = Vec::new();
        for server in stun_servers {
            if let Ok(external_info) = self.query_stun_server(server).await {
                let discovered_nodes = self.discover_peers_via_external_ip(&external_info).await?;
                nodes.extend(discovered_nodes);
            }
        }

        Ok(nodes)
    }

    /// Discover nodes via BearDog secure discovery
    async fn discover_via_beardog(&self) -> Result<Vec<FederationNode>> {
        // BearDog discovery implementation
        // This would integrate with the BearDog security system
        Ok(Vec::new())
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
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
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
                endpoint: format!("http://{}:8080", from_addr.ip()),
                endpoints: vec![format!("http://{}:8080", from_addr.ip())],
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
                    gpus: vec![],
                    network_bandwidth_mbps: 1000,
                    specializations: service.capabilities.clone(),
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
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
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
        let ip_str = ip_parts.first().unwrap_or(&"127.0.0.1");

        // Create socket address
        let socket_addr = format!("{ip_str}:8080")
            .parse::<std::net::SocketAddr>()
            .unwrap_or_else(|_| "127.0.0.1:8080".parse().unwrap());

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

    /// Query STUN server
    async fn query_stun_server(&self, server: &str) -> Result<ExternalIPInfo> {
        debug!("Querying STUN server: {}", server);

        // Create UDP socket for STUN query
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

        // Parse STUN server address
        let server_addr = if server.contains(':') {
            server.to_string()
        } else {
            format!("{server}:3478") // Default STUN port
        };

        // Create STUN binding request
        let stun_request = self.create_stun_request()?;

        // Send STUN request
        socket.send_to(&stun_request, &server_addr).await?;

        // Listen for response
        let mut buffer = [0u8; 1024];
        let timeout_duration = Duration::from_secs(5);

        match tokio::time::timeout(timeout_duration, socket.recv_from(&mut buffer)).await {
            Ok(Ok((size, _))) => {
                let response = &buffer[..size];
                self.parse_stun_response(response, server).await
            }
            Ok(Err(e)) => {
                warn!("STUN receive error: {}", e);
                Ok(ExternalIPInfo {
                    external_ip: "0.0.0.0".to_string(),
                    external_port: 0,
                    server: server.to_string(),
                    nat_type: NATType::Unknown,
                    region: "unknown".to_string(),
                })
            }
            Err(_) => {
                debug!("STUN query timeout for server: {}", server);
                Ok(ExternalIPInfo {
                    external_ip: "0.0.0.0".to_string(),
                    external_port: 0,
                    server: server.to_string(),
                    nat_type: NATType::Timeout,
                    region: "unknown".to_string(),
                })
            }
        }
    }

    /// Create STUN binding request
    fn create_stun_request(&self) -> Result<Vec<u8>> {
        let mut request = Vec::new();

        // STUN header
        request.extend_from_slice(&[0x00, 0x01]); // Binding request
        request.extend_from_slice(&[0x00, 0x00]); // Length (0 for now)
        request.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]); // Magic cookie

        // Transaction ID (12 bytes)
        for _ in 0..12 {
            request.push(rand::random::<u8>());
        }

        // Update length
        request[2] = 0x00;
        request[3] = 0x00;

        Ok(request)
    }

    /// Parse STUN response
    async fn parse_stun_response(&self, response: &[u8], server: &str) -> Result<ExternalIPInfo> {
        if response.len() < 20 {
            return Ok(ExternalIPInfo {
                external_ip: "0.0.0.0".to_string(),
                external_port: 0,
                server: server.to_string(),
                nat_type: NATType::Unknown,
                region: "unknown".to_string(),
            });
        }

        // Basic STUN response parsing
        let mut external_ip = "0.0.0.0".to_string();
        let mut external_port = 0u16;

        // Look for XOR-MAPPED-ADDRESS attribute (0x0020)
        let mut pos = 20; // Skip STUN header
        while pos + 4 < response.len() {
            let attr_type = u16::from_be_bytes([response[pos], response[pos + 1]]);
            let attr_length = u16::from_be_bytes([response[pos + 2], response[pos + 3]]);

            if attr_type == 0x0020 && attr_length >= 8 {
                // XOR-MAPPED-ADDRESS found
                let port_bytes = [response[pos + 6], response[pos + 7]];
                external_port = u16::from_be_bytes(port_bytes) ^ 0x2112;

                let ip_bytes = [
                    response[pos + 8] ^ 0x21,
                    response[pos + 9] ^ 0x12,
                    response[pos + 10] ^ 0xA4,
                    response[pos + 11] ^ 0x42,
                ];

                external_ip = format!(
                    "{}.{}.{}.{}",
                    ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]
                );
                break;
            }

            pos += 4 + attr_length as usize;
        }

        Ok(ExternalIPInfo {
            external_ip,
            external_port,
            server: server.to_string(),
            nat_type: NATType::Symmetric,
            region: "unknown".to_string(),
        })
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
                for port in [8080, 8081, 8082, 8083] {
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

        match tokio::time::timeout(Duration::from_secs(2), client.get(&health_url).send()).await {
            Ok(Ok(resp)) => resp.status().is_success(),
            _ => false,
        }
    }

    /// Create federation node from endpoint
    async fn create_node_from_endpoint(&self, endpoint: &str) -> Result<FederationNode> {
        let node_id = uuid::Uuid::new_v4();

        // Parse endpoint to get address
        let url = url::Url::parse(endpoint)?;
        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(8080);
        let socket_addr = format!("{host}:{port}")
            .parse::<std::net::SocketAddr>()
            .unwrap_or_else(|_| "127.0.0.1:8080".parse().unwrap());

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
}
