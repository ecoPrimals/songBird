//! UPnP client for local network discovery

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

use super::types::{DiscoveryConfig, UPnPDevice};
use songbird_errors::Result;
use songbird_universal_primals::PrimalCapability;

/// UPnP client for local network discovery
pub struct UPnPClient {
    discovery_port: u16,
    timeout: Duration,
    discovered_devices: Arc<RwLock<HashMap<String, UPnPDevice>>>,
}

impl UPnPClient {
    /// Create new UPnP client
    pub fn new(config: &DiscoveryConfig) -> Self {
        Self {
            discovery_port: 1900, // Standard UPnP port
            timeout: config.discovery_timeout,
            discovered_devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Discover peers via UPnP
    pub async fn discover_peers(&self) -> Result<Vec<Vec<PrimalCapability>>> {
        debug!("Discovering peers via UPnP...");

        let mut peers = Vec::new();

        // Create UPnP multicast socket for SSDP discovery
        let bind_addr = format!(
            "{}:0",
            songbird_config::config::constants::network::DEFAULT_BIND_ADDRESS
        );
        let socket = match tokio::net::UdpSocket::bind(&bind_addr).await {
            Ok(socket) => socket,
            Err(e) => {
                debug!("Failed to create UPnP discovery socket: {}", e);
                return Ok(peers);
            }
        };

        // UPnP SSDP discovery message
        let search_request = [
            "M-SEARCH * HTTP/1.1",
            "HOST: 239.255.255.250:1900",
            "MAN: \"ssdp:discover\"",
            "ST: urn:schemas-songbird:device:orchestrator:1",
            "MX: 3",
            "",
            "",
        ]
        .join("\r\n");

        // Send multicast discovery request
        let multicast_addr: SocketAddr = std::env::var("SONGBIRD_MULTICAST_ADDRESS")
            .unwrap_or_else(|_| "239.255.255.250:1900".to_string())
            .parse()
            .unwrap_or_else(|_| {
                "239.255.255.250:1900".parse().unwrap_or_else(|_| {
                    error!("Failed to parse multicast address, using fallback");
                    std::net::SocketAddr::from(([239, 255, 255, 250], 1900))
                })
            });

        match socket
            .send_to(search_request.as_bytes(), multicast_addr)
            .await
        {
            Ok(_) => debug!("UPnP discovery request sent"),
            Err(e) => {
                debug!("Failed to send UPnP discovery request: {}", e);
                return Ok(peers);
            }
        }

        // Listen for responses with timeout
        let mut buffer = [0u8; 1024];
        let timeout_future = tokio::time::timeout(self.timeout, async {
            while let Ok((size, addr)) = socket.recv_from(&mut buffer).await {
                let response = String::from_utf8_lossy(&buffer[..size]);

                // Parse UPnP response for Songbird orchestrators
                if response.contains("urn:schemas-songbird:device:orchestrator:1")
                    && response.contains("HTTP/1.1 200 OK")
                {
                    debug!("Found Songbird orchestrator at: {}", addr);

                    // Extract capabilities from UPnP response
                    let latency_ms = self.measure_latency(&addr).await.unwrap_or(10);
                    let bandwidth_mbps = self.estimate_bandwidth(&addr).await.unwrap_or(100);

                    peers.push(vec![
                        PrimalCapability::NetworkRouting {
                            protocols: vec![
                                "UPnP".to_string(),
                                "BSTP".to_string(),
                                "HTTP".to_string(),
                            ],
                        },
                        PrimalCapability::Custom {
                            name: "Gaming".to_string(),
                            properties: vec![("optimized".to_string(), "true".to_string())],
                        },
                        PrimalCapability::Custom {
                            name: "NetworkConnectivity".to_string(),
                            properties: [
                                ("bandwidth_mbps".to_string(), bandwidth_mbps.to_string()),
                                ("latency_ms".to_string(), latency_ms.to_string()),
                            ]
                            .to_vec(),
                        },
                    ]);
                }
            }
        });

        match timeout_future.await {
            Ok(_) => debug!("UPnP discovery completed"),
            Err(_) => debug!("UPnP discovery timed out"),
        }

        // For UPnP discovery, we'll use a conservative estimate
        if peers.is_empty() {
            debug!("No UPnP peers discovered, providing default estimate");
        }

        Ok(peers)
    }

    /// Measure latency to a discovered device
    async fn measure_latency(&self, addr: &SocketAddr) -> Option<u32> {
        debug!("Measuring latency to UPnP device: {}", addr);

        // Use TCP connection timing as a proxy for latency since ICMP requires privileges
        let start_time = std::time::Instant::now();

        match tokio::time::timeout(
            Duration::from_millis(5000),
            tokio::net::TcpStream::connect(addr),
        )
        .await
        {
            Ok(Ok(_stream)) => {
                let latency_ms = start_time.elapsed().as_millis() as u32;
                debug!("TCP connect latency to {}: {}ms", addr, latency_ms);
                Some(latency_ms)
            }
            Ok(Err(e)) => {
                debug!("TCP connect failed to {}: {}", addr, e);
                // Even failed connections can give us some timing info
                let partial_latency = start_time.elapsed().as_millis() as u32;
                if partial_latency > 1000 {
                    // If it took a long time to fail, the device is likely unreachable
                    None
                } else {
                    // Quick failure might just be a closed port, but device is reachable
                    Some(partial_latency * 2) // Estimate based on partial timing
                }
            }
            Err(_timeout) => {
                debug!("Latency measurement timed out for {}", addr);
                None
            }
        }
    }

    /// Estimate bandwidth to a discovered device
    async fn estimate_bandwidth(&self, addr: &SocketAddr) -> Option<u32> {
        debug!("Estimating bandwidth to UPnP device: {}", addr);

        // Measure latency first to get baseline connectivity info
        let latency_ms = match self.measure_latency(addr).await {
            Some(latency) => latency,
            None => {
                debug!("Cannot estimate bandwidth without connectivity");
                return None;
            }
        };

        // Use a simple bandwidth estimation based on latency and network topology
        let estimated_mbps = match latency_ms {
            // Very low latency suggests local gigabit network
            lat if lat < 2 => 1000,
            // Low latency suggests fast local network
            lat if lat < 5 => 100,
            // Medium latency suggests standard local network
            lat if lat < 15 => 100,
            // Higher latency suggests slower connection or distant device
            lat if lat < 50 => 10,
            // Very high latency suggests limited bandwidth
            _ => 1,
        };

        // Additional heuristics based on IP address ranges
        let adjusted_bandwidth = match addr.ip() {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                match octets {
                    // Local loopback - assume very fast
                    [127, _, _, _] => estimated_mbps.max(1000),
                    // Private Class A (10.x.x.x) - typically enterprise/fast
                    [10, _, _, _] => estimated_mbps.max(100),
                    // Private Class B (172.16-31.x.x) - typically office networks
                    [172, second, _, _] if (16..=31).contains(&second) => estimated_mbps.max(100),
                    // Private Class C (192.168.x.x) - typically home networks
                    [192, 168, _, _] => estimated_mbps.min(100), // Cap at 100 Mbps for home
                    // Public IP - more conservative estimate
                    _ => estimated_mbps.min(50),
                }
            }
            std::net::IpAddr::V6(_) => {
                // IPv6 local networks are typically modern and fast
                estimated_mbps.max(100)
            }
        };

        debug!(
            "Estimated bandwidth to {}: {} Mbps (latency: {}ms)",
            addr, adjusted_bandwidth, latency_ms
        );

        Some(adjusted_bandwidth)
    }

    /// Get all discovered devices
    pub async fn get_discovered_devices(&self) -> HashMap<String, UPnPDevice> {
        self.discovered_devices.read().await.clone()
    }

    /// Add discovered device
    pub async fn add_discovered_device(&self, device: UPnPDevice) {
        let mut devices = self.discovered_devices.write().await;
        devices.insert(device.device_id.clone(), device);
    }

    /// Remove discovered device
    pub async fn remove_discovered_device(&self, device_id: &str) -> Option<UPnPDevice> {
        let mut devices = self.discovered_devices.write().await;
        devices.remove(device_id)
    }

    /// Clear all discovered devices
    pub async fn clear_discovered_devices(&self) {
        let mut devices = self.discovered_devices.write().await;
        devices.clear();
    }

    /// Check if device exists
    pub async fn has_device(&self, device_id: &str) -> bool {
        let devices = self.discovered_devices.read().await;
        devices.contains_key(device_id)
    }

    /// Get device count
    pub async fn device_count(&self) -> usize {
        let devices = self.discovered_devices.read().await;
        devices.len()
    }

    /// Start UPnP device announcement listener
    pub async fn start_announcement_listener(&self) -> Result<()> {
        debug!(
            "Starting UPnP announcement listener on port {}",
            self.discovery_port
        );

        let socket =
            tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", self.discovery_port)).await?;
        socket.join_multicast_v4(
            std::net::Ipv4Addr::new(239, 255, 255, 250),
            std::net::Ipv4Addr::new(0, 0, 0, 0),
        )?;

        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((size, addr)) => {
                        let message = String::from_utf8_lossy(&buffer[..size]);
                        if message.contains("NOTIFY") || message.contains("M-SEARCH") {
                            debug!("Received UPnP announcement from {}: {}", addr, message);
                            // Process UPnP announcements here
                        }
                    }
                    Err(e) => {
                        error!("Error receiving UPnP announcement: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Send UPnP device announcement
    pub async fn send_device_announcement(&self) -> Result<()> {
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        let multicast_addr: SocketAddr = "239.255.255.250:1900".parse()?;

        let announcement = [
            "NOTIFY * HTTP/1.1",
            "HOST: 239.255.255.250:1900",
            "CACHE-CONTROL: max-age=120",
            &format!(
                "LOCATION: http://{}:8080/upnp/desc.xml",
                songbird_config::config::hardcoded_elimination::replace::production_bind_address()
            ),
            "NT: urn:schemas-songbird:device:orchestrator:1",
            "NTS: ssdp:alive",
            "USN: uuid:songbird-orchestrator::urn:schemas-songbird:device:orchestrator:1",
            "SERVER: Songbird/1.0 UPnP/1.0",
            "",
            "",
        ]
        .join("\r\n");

        socket
            .send_to(announcement.as_bytes(), multicast_addr)
            .await?;
        debug!("Sent UPnP device announcement");

        Ok(())
    }
}
