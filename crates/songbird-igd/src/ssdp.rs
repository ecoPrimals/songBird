// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! SSDP (Simple Service Discovery Protocol) implementation
//!
//! SSDP is used to discover `UPnP` devices on the local network via UDP multicast.
//! This module sends M-SEARCH queries to the SSDP multicast address and parses responses.

use crate::error::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, trace, warn};

/// SSDP client for discovering `UPnP` IGD devices
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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            timeout: Duration::from_secs(3),
        }
    }

    /// Create SSDP client with custom timeout
    #[must_use]
    pub const fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
        }
    }

    /// Discover `UPnP` IGD devices on the network
    ///
    /// Sends M-SEARCH multicast and collects responses for 3 seconds (or custom timeout).
    ///
    /// # Errors
    ///
    /// Returns an error if socket binding, broadcast enable, or send fails.
    pub async fn discover_gateways(&self) -> Result<Vec<SsdpResponse>> {
        debug!("Starting SSDP discovery for UPnP IGD devices");

        let socket = UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR).await?;

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
                () = tokio::time::sleep_until(deadline) => {
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
        let server = headers.get("SERVER").cloned().unwrap_or_else(|| String::from("Unknown"));
        let service_type = headers.get("ST")?.clone();
        let usn = headers.get("USN").cloned().unwrap_or_default();

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
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn ssdp_client_new_and_default_match() {
        let a = SsdpClient::new();
        let b = SsdpClient::default();
        assert_eq!(a.timeout, b.timeout, "Default and new() should agree");
    }

    #[test]
    fn ssdp_client_with_timeout_overrides_default() {
        let c = SsdpClient::with_timeout(Duration::from_secs(7));
        assert_eq!(c.timeout, Duration::from_secs(7));
    }

    #[test]
    fn parse_ssdp_response_igd_device() {
        let response = b"HTTP/1.1 200 OK\r\n\
            CACHE-CONTROL: max-age=1800\r\n\
            LOCATION: http://192.168.1.254:5431/IGD.xml\r\n\
            SERVER: Linux UPnP/1.0 Songbird-IGD/0.1.0\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            USN: uuid:12345678-1234-1234-1234-123456789012::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            \r\n";

        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)), 1900);

        let parsed = SsdpClient::parse_response(response, addr);
        assert!(parsed.is_some(), "IGD ST should be accepted");

        let ssdp = parsed.expect("parsed IGD response");
        assert_eq!(ssdp.location, "http://192.168.1.254:5431/IGD.xml");
        assert!(ssdp.service_type.contains("InternetGatewayDevice"));
        assert_eq!(ssdp.source_addr, addr);
        assert!(!ssdp.usn.is_empty(), "USN should be captured when present");
    }

    #[test]
    fn parse_ssdp_response_wanip_service_st_header() {
        let response = b"HTTP/1.1 200 OK\r\n\
            LOCATION: http://192.168.0.1:49000/wanip.xml\r\n\
            ST: urn:schemas-upnp-org:service:WANIPConnection:1\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)), 1900);
        let ssdp = SsdpClient::parse_response(response, addr).expect("WANIP ST should qualify");
        assert!(ssdp.service_type.contains("WANIPConnection"));
        assert_eq!(ssdp.server, "Unknown", "missing SERVER defaults to Unknown");
    }

    #[test]
    fn parse_ssdp_response_rejects_non_200() {
        let response = b"HTTP/1.1 404 Not Found\r\n\r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(
            SsdpClient::parse_response(response, addr).is_none(),
            "non-200 SSDP should be ignored"
        );
    }

    #[test]
    fn parse_ssdp_response_rejects_non_utf8() {
        let bad = [0xFFu8, 0xFE, 0xFD];
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(SsdpClient::parse_response(&bad, addr).is_none());
    }

    #[test]
    fn parse_ssdp_response_requires_location() {
        let response = b"HTTP/1.1 200 OK\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(SsdpClient::parse_response(response, addr).is_none());
    }

    #[test]
    fn parse_non_igd_response_filtered_out() {
        let response = b"HTTP/1.1 200 OK\r\n\
            LOCATION: http://192.168.1.100:8008/\r\n\
            ST: urn:dial-multiscreen-org:service:dial:1\r\n\
            \r\n";

        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 1900);

        let parsed = SsdpClient::parse_response(response, addr);
        assert!(parsed.is_none(), "non-IGD ST should be ignored");
    }

    #[test]
    fn parse_ssdp_response_rejects_http_200_without_ok_phrase() {
        let response = b"HTTP/1.1 200\r\n\
            LOCATION: http://192.168.1.1/desc.xml\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(
            SsdpClient::parse_response(response, addr).is_none(),
            "parser requires exact 'HTTP/1.1 200 OK' line"
        );
    }

    #[test]
    fn parse_ssdp_response_requires_st_header_even_with_location() {
        let response = b"HTTP/1.1 200 OK\r\n\
            LOCATION: http://192.168.1.1/desc.xml\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(SsdpClient::parse_response(response, addr).is_none());
    }

    #[test]
    fn parse_ssdp_response_is_case_sensitive_on_status_line() {
        let response = b"http/1.1 200 OK\r\n\
            LOCATION: http://192.168.1.1/desc.xml\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        assert!(SsdpClient::parse_response(response, addr).is_none());
    }

    #[test]
    fn parse_ssdp_response_header_names_are_case_insensitive() {
        let response = b"HTTP/1.1 200 OK\r\n\
            location: http://192.168.1.50:8080/desc.xml\r\n\
            st: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            server: TestRouter/1.0\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)), 1900);
        let ssdp = SsdpClient::parse_response(response, addr).expect("lowercase headers");
        assert_eq!(ssdp.location, "http://192.168.1.50:8080/desc.xml");
        assert_eq!(ssdp.server, "TestRouter/1.0");
    }

    #[test]
    fn parse_ssdp_response_usn_defaults_to_empty_when_missing() {
        let response = b"HTTP/1.1 200 OK\r\n\
            LOCATION: http://192.168.1.1/desc.xml\r\n\
            ST: urn:schemas-upnp-org:service:WANIPConnection:1\r\n\
            \r\n";
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1900);
        let ssdp = SsdpClient::parse_response(response, addr).expect("WANIP without USN");
        assert!(ssdp.usn.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn discover_gateways_returns_empty_when_no_responses() {
        let client = SsdpClient::with_timeout(Duration::from_millis(1));
        let responses = client
            .discover_gateways()
            .await
            .expect("discovery should not error when no devices reply");
        assert!(
            responses.is_empty(),
            "no SSDP replies should yield empty list, got {:?}",
            responses
        );
    }
}
