//! SSDP (Simple Service Discovery Protocol) implementation
//!
//! SSDP is used to discover UPnP devices on the local network via UDP multicast.
//! This module sends M-SEARCH queries to the SSDP multicast address and parses responses.

use crate::error::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, trace, warn};

/// SSDP client for discovering UPnP IGD devices
pub struct SsdpClient {
    /// Discovery timeout
    timeout: Duration,
}

/// SSDP discovery response
#[derive(Debug, Clone)]
pub struct SsdpResponse {
    /// Device description URL (LOCATION header)
    pub location: String,

    /// Server identification string
    pub server: String,

    /// Service type (ST header)
    pub service_type: String,

    /// Unique service name (USN header)
    pub usn: String,

    /// Source address of the response
    pub source_addr: SocketAddr,
}

impl SsdpClient {
    /// Create new SSDP client with default 3-second timeout
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(3),
        }
    }

    /// Create SSDP client with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
        }
    }

    /// Discover UPnP IGD devices on the network
    ///
    /// Sends M-SEARCH multicast and collects responses for 3 seconds (or custom timeout).
    pub async fn discover_gateways(&self) -> Result<Vec<SsdpResponse>> {
        debug!("Starting SSDP discovery for UPnP IGD devices");

        // Bind to any available port
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        // Enable broadcast (required for multicast)
        socket.set_broadcast(true)?;

        // M-SEARCH message targeting InternetGatewayDevice
        let search_msg = format!(
            "M-SEARCH * HTTP/1.1\r\n\
             HOST: {}\r\n\
             MAN: \"ssdp:discover\"\r\n\
             MX: 3\r\n\
             ST: {}\r\n\
             \r\n",
            crate::SSDP_MULTICAST_ADDR,
            crate::IGD_DEVICE_TYPE
        );

        // Send M-SEARCH to multicast address
        socket.send_to(search_msg.as_bytes(), crate::SSDP_MULTICAST_ADDR).await?;

        trace!("Sent SSDP M-SEARCH to {}", crate::SSDP_MULTICAST_ADDR);

        // Also try searching for WANIPConnection service directly
        // Some routers only respond to service-level queries
        let service_search_msg = format!(
            "M-SEARCH * HTTP/1.1\r\n\
             HOST: {}\r\n\
             MAN: \"ssdp:discover\"\r\n\
             MX: 3\r\n\
             ST: {}\r\n\
             \r\n",
            crate::SSDP_MULTICAST_ADDR,
            crate::WANIP_SERVICE_TYPE
        );

        socket.send_to(service_search_msg.as_bytes(), crate::SSDP_MULTICAST_ADDR).await?;

        trace!("Sent SSDP M-SEARCH for WANIPConnection service");

        // Collect responses
        let mut responses = Vec::new();
        let mut buf = [0u8; 2048];
        let deadline = tokio::time::Instant::now() + self.timeout;

        loop {
            tokio::select! {
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            if let Some(resp) = Self::parse_response(&buf[..len], addr) {
                                debug!(
                                    "Found UPnP device: {} at {}",
                                    resp.service_type, resp.source_addr
                                );
                                responses.push(resp);
                            }
                        }
                        Err(e) => {
                            warn!("Error receiving SSDP response: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep_until(deadline) => {
                    debug!("SSDP discovery timeout reached");
                    break;
                }
            }
        }

        if responses.is_empty() {
            debug!("No UPnP IGD devices found via SSDP");
        } else {
            debug!("Found {} UPnP device(s)", responses.len());
        }

        Ok(responses)
    }

    /// Parse SSDP response from raw bytes
    fn parse_response(data: &[u8], source_addr: SocketAddr) -> Option<SsdpResponse> {
        let text = std::str::from_utf8(data).ok()?;

        // Must be HTTP response
        if !text.starts_with("HTTP/1.1 200 OK") {
            return None;
        }

        // Parse headers into HashMap
        let mut headers = HashMap::new();
        for line in text.lines().skip(1) {
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_uppercase(), value.trim().to_string());
            }
        }

        // Extract required fields
        let location = headers.get("LOCATION")?.clone();
        let server = headers.get("SERVER").unwrap_or(&"Unknown".to_string()).clone();
        let service_type = headers.get("ST")?.clone();
        let usn = headers.get("USN").unwrap_or(&"".to_string()).clone();

        // Only return IGD devices or WANIPConnection services
        if !service_type.contains("InternetGatewayDevice")
            && !service_type.contains("WANIPConnection")
        {
            trace!("Ignoring non-IGD device: {}", service_type);
            return None;
        }

        Some(SsdpResponse {
            location,
            server,
            service_type,
            usn,
            source_addr,
        })
    }
}

impl Default for SsdpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_parse_ssdp_response() {
        let response = b"HTTP/1.1 200 OK\r\n\
            CACHE-CONTROL: max-age=1800\r\n\
            LOCATION: http://192.168.1.254:5431/IGD.xml\r\n\
            SERVER: Linux UPnP/1.0 Songbird-IGD/0.1.0\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            USN: uuid:12345678-1234-1234-1234-123456789012::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            \r\n";

        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)), 1900);

        let parsed = SsdpClient::parse_response(response, addr);
        assert!(parsed.is_some());

        let ssdp = parsed.unwrap();
        assert_eq!(ssdp.location, "http://192.168.1.254:5431/IGD.xml");
        assert!(ssdp.service_type.contains("InternetGatewayDevice"));
    }

    #[test]
    fn test_parse_non_igd_response() {
        let response = b"HTTP/1.1 200 OK\r\n\
            LOCATION: http://192.168.1.100:8008/\r\n\
            ST: urn:dial-multiscreen-org:service:dial:1\r\n\
            \r\n";

        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 1900);

        let parsed = SsdpClient::parse_response(response, addr);
        assert!(parsed.is_none(), "Should ignore non-IGD devices");
    }
}
