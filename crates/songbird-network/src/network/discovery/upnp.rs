//! UPnP client for local network discovery

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error};

use super::super::beardog_integration::PeerCapabilities;
use super::types::{DiscoveryConfig, UPnPDevice};
use songbird_errors::Result;

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
    pub async fn discover_peers(&self) -> Result<Vec<PeerCapabilities>> {
        debug!("Discovering peers via UPnP...");

        let mut peers = Vec::new();

        // Create UPnP multicast socket for SSDP discovery
        let bind_addr = format!(
            "{}:0",
            crate::config::constants::network::production_bind_address()
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

                    peers.push(PeerCapabilities {
                        protocol_support: vec![
                            "UPnP".to_string(),
                            "BSTP".to_string(),
                            "HTTP".to_string(),
                        ],
                        bandwidth_mbps,
                        latency_ms,
                        gaming_optimized: true,
                        security_level: crate::network::beardog_integration::SecurityLevel::Gaming,
                    });
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
    async fn measure_latency(&self, _addr: &SocketAddr) -> Option<u32> {
        // In a real implementation, this would:
        // 1. Send ICMP ping requests
        // 2. Measure round-trip time
        // 3. Return average latency over multiple measurements
        
        // For now, return a mock value based on local network assumptions
        Some(5) // Assume 5ms for local network
    }

    /// Estimate bandwidth to a discovered device
    async fn estimate_bandwidth(&self, _addr: &SocketAddr) -> Option<u32> {
        // In a real implementation, this would:
        // 1. Perform bandwidth tests using UPnP device capabilities
        // 2. Send test data and measure transfer rates
        // 3. Return estimated bandwidth in Mbps
        
        // For now, return a conservative estimate for local networks
        Some(100) // Assume 100 Mbps for local network
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
        debug!("Starting UPnP announcement listener on port {}", self.discovery_port);

        let socket = tokio::net::UdpSocket::bind(format!("0.0.0.0:{}", self.discovery_port)).await?;
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
            "LOCATION: http://0.0.0.0:8080/upnp/desc.xml",
            "NT: urn:schemas-songbird:device:orchestrator:1",
            "NTS: ssdp:alive",
            "USN: uuid:songbird-orchestrator::urn:schemas-songbird:device:orchestrator:1",
            "SERVER: Songbird/1.0 UPnP/1.0",
            "",
            "",
        ]
        .join("\r\n");

        socket.send_to(announcement.as_bytes(), multicast_addr).await?;
        debug!("Sent UPnP device announcement");

        Ok(())
    }
} 