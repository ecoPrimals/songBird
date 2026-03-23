// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unified gateway abstraction over `UPnP` IGD and NAT-PMP
//!
//! Tries `UPnP` IGD first (most common), falls back to NAT-PMP.
//! Provides a single interface for all port mapping operations.

use crate::error::{IgdError, Result};
use crate::mapping::{PortMapping, PortMappingRequest, Protocol};
use crate::nat_pmp::NatPmpClient;
use crate::soap::SoapClient;
use crate::ssdp::SsdpClient;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use tracing::{debug, info, warn};

/// Which protocol the gateway supports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayProtocol {
    /// `UPnP` IGD via SSDP/SOAP
    UpnpIgd {
        /// SOAP control URL
        control_url: String,
        /// `UPnP` service type
        service_type: String,
        /// Friendly device name
        device_name: Option<String>,
    },
    /// NAT-PMP binary protocol
    NatPmp,
    /// No supported protocol found
    None,
}

/// Discovered gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    /// Gateway IP address
    pub ip: IpAddr,
    /// Which protocol is available
    pub protocol: GatewayProtocol,
    /// External (WAN) IP if known
    pub external_ip: Option<IpAddr>,
    /// Device name if known
    pub device_name: Option<String>,
    /// Non-IGD `UPnP` devices found (printers, Chromecasts, etc.)
    pub other_devices: Vec<String>,
}

/// `UPnP` discovery status (avoids excessive bools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpnpDiscoveryStatus {
    /// SSDP M-SEARCH was sent
    pub ssdp_sent: bool,
    /// IGD device was found
    pub igd_found: bool,
}

/// NAT-PMP discovery status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatPmpDiscoveryStatus {
    /// NAT-PMP probe was sent
    pub probe_sent: bool,
    /// NAT-PMP responded
    pub responded: bool,
}

/// Discovery diagnostics for user-facing error messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDiagnostics {
    /// Gateway IP
    pub gateway_ip: IpAddr,
    /// Whether gateway is reachable
    pub gateway_reachable: bool,
    /// `UPnP` discovery status
    pub upnp: UpnpDiscoveryStatus,
    /// `UPnP` devices found (including non-IGD)
    pub upnp_devices_found: Vec<String>,
    /// NAT-PMP discovery status
    pub nat_pmp: NatPmpDiscoveryStatus,
    /// Manual configuration instructions
    pub manual_instructions: Vec<String>,
    /// Alternative connectivity tiers
    pub alternative_tiers: Vec<String>,
}

impl Gateway {
    /// Discover the best available gateway and protocol
    ///
    /// # Errors
    ///
    /// Returns an error if default gateway detection fails.
    pub async fn discover() -> Result<Self> {
        info!("Discovering gateway for IGD configuration");

        let gateway_ip = Self::get_default_gateway()?;
        debug!("Default gateway: {}", gateway_ip);

        // Try UPnP IGD first
        match Self::try_upnp_igd(gateway_ip).await {
            Ok(gw) => {
                info!("UPnP IGD gateway found at {}", gw.ip);
                return Ok(gw);
            }
            Err(e) => {
                debug!("UPnP IGD not available: {}", e);
            }
        }

        // Fall back to NAT-PMP
        match Self::try_nat_pmp(gateway_ip).await {
            Ok(gw) => {
                info!("NAT-PMP gateway found at {}", gw.ip);
                return Ok(gw);
            }
            Err(e) => {
                debug!("NAT-PMP not available: {}", e);
            }
        }

        // Nothing available — return gateway with diagnostics
        warn!(
            "No IGD protocol available on gateway {}. Manual port forwarding required.",
            gateway_ip
        );

        Ok(Self {
            ip: gateway_ip,
            protocol: GatewayProtocol::None,
            external_ip: None,
            device_name: None,
            other_devices: Vec::new(),
        })
    }

    /// Discover with full diagnostics
    pub async fn discover_with_diagnostics() -> (Self, DiscoveryDiagnostics) {
        let gateway_ip =
            Self::get_default_gateway().unwrap_or(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));

        let local_ip = Self::get_local_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let mut diagnostics = DiscoveryDiagnostics {
            gateway_ip,
            gateway_reachable: true, // assumed if we got here
            upnp: UpnpDiscoveryStatus {
                ssdp_sent: false,
                igd_found: false,
            },
            upnp_devices_found: Vec::new(),
            nat_pmp: NatPmpDiscoveryStatus {
                probe_sent: false,
                responded: false,
            },
            manual_instructions: vec![
                format!("1. Open http://{gateway_ip} in a browser"),
                "2. Log in to your router admin panel".to_string(),
                "3. Navigate to Firewall > NAT/Gaming (or Port Forwarding)".to_string(),
                format!("4. Add rule: TCP port 3492 -> {local_ip}:3492"),
                "5. Save and apply".to_string(),
                "6. Optionally: Enable UPnP in the router settings for auto-config".to_string(),
            ],
            alternative_tiers: vec![
                "Sovereign onion: .onion address via onion.start (works everywhere)".to_string(),
                "STUN hole-punch: punch.request (works for non-symmetric NAT)".to_string(),
                "Family relay: mesh via another connected family device".to_string(),
            ],
        };

        // Try SSDP
        diagnostics.upnp.ssdp_sent = true;
        let ssdp = SsdpClient::new();
        let ssdp_results = ssdp.discover_gateways().await.unwrap_or_default();

        for r in &ssdp_results {
            diagnostics.upnp_devices_found.push(format!(
                "{service_type} ({source_addr})",
                service_type = r.service_type,
                source_addr = r.source_addr
            ));
        }

        let igd_responses: Vec<_> = ssdp_results
            .iter()
            .filter(|r| {
                r.service_type.contains("InternetGatewayDevice")
                    || r.service_type.contains("WANIPConnection")
            })
            .collect();

        if !igd_responses.is_empty() {
            diagnostics.upnp.igd_found = true;

            let resp = &igd_responses[0];
            let gateway = Self {
                ip: gateway_ip,
                protocol: GatewayProtocol::UpnpIgd {
                    control_url: resp.location.clone(),
                    service_type: resp.service_type.clone(),
                    device_name: Some(resp.server.clone()),
                },
                external_ip: None,
                device_name: Some(resp.server.clone()),
                other_devices: Vec::new(),
            };

            return (gateway, diagnostics);
        }

        // Try NAT-PMP
        diagnostics.nat_pmp.probe_sent = true;
        let natpmp = NatPmpClient::new(gateway_ip);
        if natpmp.probe().await {
            diagnostics.nat_pmp.responded = true;

            let external_ip = natpmp.get_external_ip().await.ok().map(|r| IpAddr::V4(r.ip));

            let gateway = Self {
                ip: gateway_ip,
                protocol: GatewayProtocol::NatPmp,
                external_ip,
                device_name: None,
                other_devices: Vec::new(),
            };

            return (gateway, diagnostics);
        }

        // Nothing found
        let gateway = Self {
            ip: gateway_ip,
            protocol: GatewayProtocol::None,
            external_ip: None,
            device_name: None,
            other_devices: diagnostics
                .upnp_devices_found
                .iter()
                .filter(|d| !d.contains("InternetGateway"))
                .cloned()
                .collect(),
        };

        (gateway, diagnostics)
    }

    /// Map a port through the gateway
    ///
    /// # Errors
    ///
    /// Returns an error if protocol is invalid, local IP detection fails, or the gateway request fails.
    pub async fn map_port(
        &self,
        external_port: u16,
        internal_port: u16,
        protocol: &str,
        ttl: u32,
    ) -> Result<PortMapping> {
        let proto = Protocol::from_str(protocol)
            .ok_or_else(|| IgdError::InvalidParameter(format!("Invalid protocol: {protocol}")))?;

        let local_ip = Self::get_local_ip()?;

        let req = PortMappingRequest::new(external_port, internal_port, local_ip, proto)
            .with_description("Songbird sovereign beacon".to_string())
            .with_lease_duration(ttl);

        match &self.protocol {
            GatewayProtocol::UpnpIgd {
                control_url,
                service_type,
                ..
            } => {
                let soap = SoapClient::new(control_url.clone(), service_type.clone());
                soap.add_port_mapping(&req).await?;

                let external_ip = soap.get_external_ip().await.ok();
                let mut mapping = PortMapping::from_request(&req);
                if let Some(ip) = external_ip {
                    mapping = mapping.with_external_ip(ip);
                }

                Ok(mapping)
            }
            GatewayProtocol::NatPmp => {
                let natpmp = NatPmpClient::new(self.ip);
                let resp = natpmp.map_port(internal_port, external_port, proto, ttl).await?;

                let external_ip = natpmp.get_external_ip().await.ok().map(|r| IpAddr::V4(r.ip));

                let mut mapping = PortMapping::from_request(&req);
                mapping.external_port = resp.external_port;
                mapping.lease_duration = resp.lifetime;
                if let Some(ip) = external_ip {
                    mapping = mapping.with_external_ip(ip);
                }

                Ok(mapping)
            }
            GatewayProtocol::None => Err(IgdError::ProtocolNotSupported(
                "No IGD protocol available. Manual port forwarding required.".to_string(),
            )),
        }
    }

    /// Remove a port mapping
    ///
    /// # Errors
    ///
    /// Returns an error if protocol is invalid or the gateway request fails.
    pub async fn unmap_port(&self, external_port: u16, protocol: &str) -> Result<()> {
        let proto = Protocol::from_str(protocol)
            .ok_or_else(|| IgdError::InvalidParameter(format!("Invalid protocol: {protocol}")))?;

        match &self.protocol {
            GatewayProtocol::UpnpIgd {
                control_url,
                service_type,
                ..
            } => {
                let soap = SoapClient::new(control_url.clone(), service_type.clone());
                soap.delete_port_mapping(external_port, protocol).await
            }
            GatewayProtocol::NatPmp => {
                let natpmp = NatPmpClient::new(self.ip);
                natpmp.delete_mapping(external_port, proto).await
            }
            GatewayProtocol::None => {
                Err(IgdError::ProtocolNotSupported("No IGD protocol available".to_string()))
            }
        }
    }

    /// Get external IP from gateway
    ///
    /// # Errors
    ///
    /// Returns an error if the gateway request fails.
    pub async fn get_external_ip(&self) -> Result<IpAddr> {
        match &self.protocol {
            GatewayProtocol::UpnpIgd {
                control_url,
                service_type,
                ..
            } => {
                let soap = SoapClient::new(control_url.clone(), service_type.clone());
                soap.get_external_ip().await
            }
            GatewayProtocol::NatPmp => {
                let natpmp = NatPmpClient::new(self.ip);
                let resp = natpmp.get_external_ip().await?;
                Ok(IpAddr::V4(resp.ip))
            }
            GatewayProtocol::None => {
                Err(IgdError::ProtocolNotSupported("No IGD protocol available".to_string()))
            }
        }
    }

    /// Check if any IGD protocol is available
    #[must_use]
    pub const fn is_available(&self) -> bool {
        !matches!(self.protocol, GatewayProtocol::None)
    }

    /// Get default gateway IP from /proc/net/route (Linux)
    fn get_default_gateway() -> Result<IpAddr> {
        let contents = std::fs::read_to_string("/proc/net/route").map_err(|e| {
            IgdError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Cannot read /proc/net/route: {e}"),
            ))
        })?;

        // Parse routing table: find default route (Destination == 00000000)
        for line in contents.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 3 && fields[1] == "00000000" {
                // Gateway is in hex, little-endian (on x86/ARM Linux)
                let gw_hex = fields[2];
                if let Ok(gw_int) = u32::from_str_radix(gw_hex, 16) {
                    let ip = Ipv4Addr::new(
                        (gw_int & 0xFF) as u8,
                        ((gw_int >> 8) & 0xFF) as u8,
                        ((gw_int >> 16) & 0xFF) as u8,
                        ((gw_int >> 24) & 0xFF) as u8,
                    );
                    debug!("Default gateway from /proc/net/route: {}", ip);
                    return Ok(IpAddr::V4(ip));
                }
            }
        }

        Err(IgdError::NoGatewayFound)
    }

    /// Get local IP address (the one facing the gateway)
    fn get_local_ip() -> Result<IpAddr> {
        // Connect a UDP socket to the gateway to determine our local IP
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(IgdError::Io)?;

        // Connect to a well-known address to determine local interface
        // This doesn't actually send data, just determines the route
        socket.connect("8.8.8.8:53").map_err(IgdError::Io)?;

        let local_addr = socket.local_addr().map_err(IgdError::Io)?;
        Ok(local_addr.ip())
    }

    /// Try `UPnP` IGD discovery
    ///
    /// 1. SSDP M-SEARCH to find IGD devices
    /// 2. HTTP GET on LOCATION URL to fetch device description XML
    /// 3. Parse XML for controlURL and serviceType
    async fn try_upnp_igd(gateway_ip: IpAddr) -> Result<Self> {
        let ssdp = SsdpClient::new();
        let responses = ssdp.discover_gateways().await?;

        let igd = responses
            .iter()
            .find(|r| {
                r.service_type.contains("InternetGatewayDevice")
                    || r.service_type.contains("WANIPConnection")
            })
            .ok_or_else(|| IgdError::SsdpError("No IGD device found via SSDP".to_string()))?;

        // Step 2: Fetch device description XML from LOCATION URL
        let location = &igd.location;
        debug!("Fetching device description from: {}", location);

        let (control_url, service_type, friendly_name) =
            match Self::fetch_device_description(location).await {
                Ok(desc) => desc,
                Err(e) => {
                    // If we can't fetch the description, use the LOCATION URL as fallback
                    // Some routers serve SOAP directly at the LOCATION URL
                    debug!(
                        "Could not fetch device description: {}. Using LOCATION as control URL.",
                        e
                    );
                    (location.clone(), igd.service_type.clone(), Some(igd.server.clone()))
                }
            };

        Ok(Self {
            ip: gateway_ip,
            protocol: GatewayProtocol::UpnpIgd {
                control_url,
                service_type,
                device_name: friendly_name.clone(),
            },
            external_ip: None,
            device_name: friendly_name,
            other_devices: Vec::new(),
        })
    }

    /// Fetch and parse `UPnP` device description XML
    ///
    /// Returns (controlURL, serviceType, friendlyName)
    async fn fetch_device_description(
        location_url: &str,
    ) -> Result<(String, String, Option<String>)> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        // Parse URL
        let url = location_url
            .strip_prefix("http://")
            .or_else(|| location_url.strip_prefix("HTTP://"))
            .ok_or_else(|| {
                IgdError::InvalidResponse(format!("Expected http:// URL: {location_url}"))
            })?;

        let (host_port, path) = url.find('/').map_or((url, "/"), |idx| (&url[..idx], &url[idx..]));

        let (host, port) = host_port.rfind(':').map_or((host_port, 80u16), |idx| {
            let port = host_port[idx + 1..].parse::<u16>().unwrap_or(80);
            (&host_port[..idx], port)
        });

        // HTTP GET request
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");

        let addr = format!("{host}:{port}");
        let mut stream =
            tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(&addr))
                .await
                .map_err(|_| IgdError::Timeout)?
                .map_err(|e| IgdError::SoapError(format!("Failed to connect to {addr}: {e}")))?;

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| IgdError::SoapError(format!("Failed to send HTTP GET: {e}")))?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|e| IgdError::SoapError(format!("Failed to read device description: {e}")))?;

        let body = String::from_utf8_lossy(&response);

        // Skip HTTP headers
        let xml = body.find("\r\n\r\n").map_or_else(|| &body[..], |i| &body[i + 4..]);

        debug!("Device description XML length: {} bytes", xml.len());

        // Parse XML for WANIPConnection service
        let control_url = Self::extract_control_url(xml, location_url)?;
        let service_type = Self::extract_xml_value(xml, "serviceType")
            .filter(|st| st.contains("WANIPConnection") || st.contains("WANPPPConnection"))
            .unwrap_or_else(|| crate::WANIP_SERVICE_TYPE.to_string());
        let friendly_name = Self::extract_xml_value(xml, "friendlyName");

        Ok((control_url, service_type, friendly_name))
    }

    /// Extract controlURL from device description XML
    ///
    /// Finds the `WANIPConnection` or `WANPPPConnection` service block and
    /// extracts its controlURL. The controlURL may be relative (needs
    /// base URL prepended) or absolute.
    fn extract_control_url(xml: &str, base_url: &str) -> Result<String> {
        // Find the WANIPConnection or WANPPPConnection service section
        let wan_markers = ["WANIPConnection", "WANPPPConnection"];

        for marker in &wan_markers {
            if let Some(service_pos) = xml.find(marker) {
                // Look for controlURL after this marker
                let after_marker = &xml[service_pos..];
                if let Some(ctl) = Self::extract_xml_value(after_marker, "controlURL") {
                    // controlURL might be relative — make it absolute
                    if ctl.starts_with("http://") || ctl.starts_with("https://") {
                        return Ok(ctl);
                    }

                    // Build absolute URL from base
                    let base = base_url
                        .strip_prefix("http://")
                        .or_else(|| base_url.strip_prefix("HTTP://"))
                        .unwrap_or(base_url);

                    let host_port = base.find('/').map_or(base, |idx| &base[..idx]);

                    let absolute = if ctl.starts_with('/') {
                        format!("http://{host_port}{ctl}")
                    } else {
                        format!("http://{host_port}/{ctl}")
                    };

                    debug!("Resolved controlURL: {} -> {}", ctl, absolute);
                    return Ok(absolute);
                }
            }
        }

        Err(IgdError::InvalidResponse(
            "No WANIPConnection controlURL found in device description".to_string(),
        ))
    }

    /// Extract a simple XML element value
    ///
    /// Finds `<tag>value</tag>` and returns value.
    /// Not a full XML parser — just enough for `UPnP` device descriptions.
    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{tag}");
        let close_tag = format!("</{tag}>");

        if let Some(start) = xml.find(&open_tag) {
            // Find the closing > of the opening tag
            let after_open = &xml[start + open_tag.len()..];
            if let Some(gt_pos) = after_open.find('>') {
                let content_start = start + open_tag.len() + gt_pos + 1;
                let content = &xml[content_start..];
                if let Some(end) = content.find(&close_tag) {
                    let value = content[..end].trim().to_string();
                    if !value.is_empty() {
                        return Some(value);
                    }
                }
            }
        }
        None
    }

    /// Try NAT-PMP discovery
    async fn try_nat_pmp(gateway_ip: IpAddr) -> Result<Self> {
        let natpmp = NatPmpClient::new(gateway_ip);

        if !natpmp.probe().await {
            return Err(IgdError::NatPmpError("Gateway not responding on port 5351".to_string()));
        }

        let external_ip = natpmp.get_external_ip().await.ok().map(|r| IpAddr::V4(r.ip));

        Ok(Self {
            ip: gateway_ip,
            protocol: GatewayProtocol::NatPmp,
            external_ip,
            device_name: None,
            other_devices: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::IgdError;
    use crate::mapping::Protocol;

    #[test]
    fn test_gateway_protocol_variants() {
        let none = GatewayProtocol::None;
        let gw = Gateway {
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: none,
            external_ip: None,
            device_name: None,
            other_devices: Vec::new(),
        };
        assert!(!gw.is_available());
    }

    #[test]
    fn test_gateway_upnp_available() {
        let gw = Gateway {
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
            protocol: GatewayProtocol::UpnpIgd {
                control_url: "http://192.168.1.254:5431/ctl/IPConn".to_string(),
                service_type: crate::WANIP_SERVICE_TYPE.to_string(),
                device_name: Some("BGW320-505".to_string()),
            },
            external_ip: Some(IpAddr::V4(Ipv4Addr::new(162, 226, 225, 148))),
            device_name: Some("BGW320-505".to_string()),
            other_devices: Vec::new(),
        };
        assert!(gw.is_available());
    }

    #[test]
    fn test_local_ip_detection() {
        // This test requires network but should work on any Linux/macOS
        let ip = Gateway::get_local_ip();
        // Don't assert specific IP, just that it works or fails gracefully
        if let Ok(ip) = ip {
            assert!(!ip.is_loopback());
        }
    }

    #[test]
    fn test_extract_xml_value() {
        let xml = r"<root>
            <friendlyName>BGW320-505</friendlyName>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>/ctl/IPConn</controlURL>
        </root>";

        assert_eq!(Gateway::extract_xml_value(xml, "friendlyName"), Some("BGW320-505".to_string()));
        assert_eq!(Gateway::extract_xml_value(xml, "controlURL"), Some("/ctl/IPConn".to_string()));
        assert_eq!(Gateway::extract_xml_value(xml, "nonexistent"), None);
    }

    #[test]
    fn test_extract_control_url_relative() {
        let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>/ctl/IPConn</controlURL>
        </service>";

        let result = Gateway::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://192.168.1.254:5431/ctl/IPConn");
    }

    #[test]
    fn test_extract_control_url_absolute() {
        let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>http://192.168.1.254:5431/ctl/IPConn</controlURL>
        </service>";

        let result = Gateway::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://192.168.1.254:5431/ctl/IPConn");
    }

    #[test]
    fn test_extract_control_url_wanppp() {
        // Some ISP routers use WANPPPConnection instead of WANIPConnection
        let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANPPPConnection:1</serviceType>
            <controlURL>/upnp/control/ppp</controlURL>
        </service>";

        let result = Gateway::extract_control_url(xml, "http://192.168.0.1:49000/rootDesc.xml");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://192.168.0.1:49000/upnp/control/ppp");
    }

    #[test]
    fn test_extract_control_url_no_wan_service() {
        let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:Layer3Forwarding:1</serviceType>
            <controlURL>/ctl/L3F</controlURL>
        </service>";

        let result = Gateway::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_xml_value_with_attributes() {
        // XML tags might have attributes
        let xml = r#"<controlURL xmlns="urn:schemas-upnp-org:device-1-0">/ctl/IPConn</controlURL>"#;
        assert_eq!(Gateway::extract_xml_value(xml, "controlURL"), Some("/ctl/IPConn".to_string()));
    }

    fn sample_upnp_gateway() -> Gateway {
        Gateway {
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 254)),
            protocol: GatewayProtocol::UpnpIgd {
                control_url: "http://192.168.1.254:5431/ctl/IPConn".to_string(),
                service_type: crate::WANIP_SERVICE_TYPE.to_string(),
                device_name: Some("TestRouter".to_string()),
            },
            external_ip: None,
            device_name: Some("TestRouter".to_string()),
            other_devices: Vec::new(),
        }
    }

    #[test]
    fn test_port_mapping_request_builder_fields() {
        let local = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
        let req = crate::mapping::PortMappingRequest::new(3492, 8080, local, Protocol::Udp)
            .with_description("Songbird test mapping".to_string())
            .with_lease_duration(120);

        assert_eq!(req.external_port, 3492);
        assert_eq!(req.internal_port, 8080);
        assert_eq!(req.internal_client, local);
        assert_eq!(req.protocol, Protocol::Udp);
        assert_eq!(req.lease_duration, 120);
        assert!(req.description.contains("Songbird test mapping"));
    }

    #[tokio::test]
    async fn map_port_rejects_invalid_protocol_string() {
        let gw = sample_upnp_gateway();
        let err = gw.map_port(80, 80, "ICMP", 3600).await.unwrap_err();
        assert!(matches!(err, IgdError::InvalidParameter(_)));
    }

    #[tokio::test]
    async fn map_port_unsupported_when_no_igd_protocol() {
        let gw = Gateway {
            ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
            protocol: GatewayProtocol::None,
            external_ip: None,
            device_name: None,
            other_devices: Vec::new(),
        };
        let err = gw.map_port(3492, 3492, "TCP", 3600).await.unwrap_err();
        assert!(matches!(err, IgdError::ProtocolNotSupported(_)));
    }

    #[tokio::test]
    async fn get_external_ip_errors_when_protocol_none() {
        let gw = Gateway {
            ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            protocol: GatewayProtocol::None,
            external_ip: None,
            device_name: None,
            other_devices: Vec::new(),
        };
        let err = gw.get_external_ip().await.unwrap_err();
        assert!(matches!(err, IgdError::ProtocolNotSupported(_)));
    }

    #[tokio::test]
    async fn unmap_port_rejects_bad_protocol() {
        let gw = sample_upnp_gateway();
        let err = gw.unmap_port(443, "QUIC").await.unwrap_err();
        assert!(matches!(err, IgdError::InvalidParameter(_)));
    }
}
