//! SOAP (Simple Object Access Protocol) control for UPnP IGD
//!
//! Implements SOAP XML envelope construction and parsing for UPnP port mapping operations.
//! Uses Songbird's HTTP client for the actual HTTP POSTs.

use crate::error::{IgdError, Result, SoapErrorCode};
use crate::mapping::PortMappingRequest;
use std::net::IpAddr;
use tracing::{debug, trace};

/// SOAP client for UPnP IGD control
pub struct SoapClient {
    /// Control URL for SOAP actions
    control_url: String,
    
    /// Service type (for SOAP namespace)
    service_type: String,
}

impl SoapClient {
    /// Create new SOAP client
    pub fn new(control_url: String, service_type: String) -> Self {
        Self {
            control_url,
            service_type,
        }
    }

    /// Add a port mapping
    pub async fn add_port_mapping(&self, req: &PortMappingRequest) -> Result<()> {
        debug!(
            "Adding port mapping: {}:{} -> {}:{} ({})",
            req.external_port,
            req.protocol.as_str(),
            req.internal_client,
            req.internal_port,
            req.description
        );

        let body = self.build_add_port_mapping_xml(req);
        let response = self.send_soap_action("AddPortMapping", &body).await?;
        
        // Check for errors in response
        if let Some(error_code) = Self::parse_soap_error(&response) {
            return Err(Self::map_soap_error(error_code, req.external_port));
        }

        debug!("Port mapping added successfully");
        Ok(())
    }

    /// Delete a port mapping
    pub async fn delete_port_mapping(&self, external_port: u16, protocol: &str) -> Result<()> {
        debug!("Deleting port mapping: {}:{}", external_port, protocol);

        let body = self.build_delete_port_mapping_xml(external_port, protocol);
        let response = self.send_soap_action("DeletePortMapping", &body).await?;
        
        // Check for errors
        if let Some(error_code) = Self::parse_soap_error(&response) {
            return Err(IgdError::SoapError(format!(
                "Delete failed with code {}: {}",
                error_code,
                SoapErrorCode::from_code(error_code)
                    .map(|e| e.description())
                    .unwrap_or("Unknown error")
            )));
        }

        debug!("Port mapping deleted successfully");
        Ok(())
    }

    /// Get external IP address from gateway
    pub async fn get_external_ip(&self) -> Result<IpAddr> {
        debug!("Querying external IP address");

        let body = self.build_get_external_ip_xml();
        let response = self.send_soap_action("GetExternalIPAddress", &body).await?;
        
        // Parse IP from response
        let ip = Self::parse_external_ip(&response)?;
        debug!("External IP: {}", ip);
        
        Ok(ip)
    }

    /// Build AddPortMapping SOAP XML
    fn build_add_port_mapping_xml(&self, req: &PortMappingRequest) -> String {
        format!(
            r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:AddPortMapping xmlns:u="{}">
      <NewRemoteHost></NewRemoteHost>
      <NewExternalPort>{}</NewExternalPort>
      <NewProtocol>{}</NewProtocol>
      <NewInternalPort>{}</NewInternalPort>
      <NewInternalClient>{}</NewInternalClient>
      <NewEnabled>1</NewEnabled>
      <NewPortMappingDescription>{}</NewPortMappingDescription>
      <NewLeaseDuration>{}</NewLeaseDuration>
    </u:AddPortMapping>
  </s:Body>
</s:Envelope>"#,
            self.service_type,
            req.external_port,
            req.protocol.as_str(),
            req.internal_port,
            req.internal_client,
            xml_escape(&req.description),
            req.lease_duration
        )
    }

    /// Build DeletePortMapping SOAP XML
    fn build_delete_port_mapping_xml(&self, external_port: u16, protocol: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:DeletePortMapping xmlns:u="{}">
      <NewRemoteHost></NewRemoteHost>
      <NewExternalPort>{}</NewExternalPort>
      <NewProtocol>{}</NewProtocol>
    </u:DeletePortMapping>
  </s:Body>
</s:Envelope>"#,
            self.service_type, external_port, protocol
        )
    }

    /// Build GetExternalIPAddress SOAP XML
    fn build_get_external_ip_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:GetExternalIPAddress xmlns:u="{}">
    </u:GetExternalIPAddress>
  </s:Body>
</s:Envelope>"#,
            self.service_type
        )
    }

    /// Send SOAP action via HTTP POST to the control URL
    ///
    /// UPnP SOAP calls are plain HTTP (not HTTPS) to local LAN addresses,
    /// so we use raw TCP rather than songbird-http-client (which is for TLS).
    async fn send_soap_action(&self, action: &str, body: &str) -> Result<String> {
        trace!("SOAP Action: {} to {}", action, self.control_url);
        trace!("SOAP Body:\n{}", body);

        // Parse the control URL to extract host, port, and path
        let (host, port, path) = Self::parse_url(&self.control_url)?;

        let soap_action = format!("\"{}#{}\"", self.service_type, action);
        let content_length = body.len();

        // Build HTTP POST request
        let request = format!(
            "POST {} HTTP/1.1\r\n\
             Host: {}:{}\r\n\
             Content-Type: text/xml; charset=\"utf-8\"\r\n\
             Content-Length: {}\r\n\
             SOAPAction: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            path, host, port, content_length, soap_action, body
        );

        // Connect and send
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let addr = format!("{}:{}", host, port);
        let mut stream = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| IgdError::Timeout)?
        .map_err(|e| {
            IgdError::SoapError(format!("Failed to connect to {}: {}", addr, e))
        })?;

        stream.write_all(request.as_bytes()).await.map_err(|e| {
            IgdError::SoapError(format!("Failed to send SOAP request: {}", e))
        })?;

        // Read response
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.map_err(|e| {
            IgdError::SoapError(format!("Failed to read SOAP response: {}", e))
        })?;

        let response_str = String::from_utf8_lossy(&response).to_string();
        trace!("SOAP Response:\n{}", response_str);

        // Extract body from HTTP response (skip headers)
        let body_start = response_str
            .find("\r\n\r\n")
            .map(|i| i + 4)
            .unwrap_or(0);
        let response_body = &response_str[body_start..];

        // Check HTTP status
        if let Some(status_line) = response_str.lines().next() {
            if !status_line.contains("200") && !status_line.contains("OK") {
                // UPnP sends 500 for SOAP faults — the body contains the error details
                if status_line.contains("500") {
                    debug!("SOAP fault received (HTTP 500), parsing error details");
                    return Ok(response_body.to_string());
                }
                return Err(IgdError::SoapError(format!(
                    "HTTP error: {}",
                    status_line
                )));
            }
        }

        Ok(response_body.to_string())
    }

    /// Parse a URL into (host, port, path) components
    ///
    /// Handles: `http://192.168.1.254:5431/ctl/IPConn`
    fn parse_url(url: &str) -> Result<(String, u16, String)> {
        let url = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("HTTP://"))
            .ok_or_else(|| {
                IgdError::InvalidParameter(format!(
                    "Expected http:// URL, got: {}",
                    url
                ))
            })?;

        let (host_port, path) = if let Some(idx) = url.find('/') {
            (&url[..idx], &url[idx..])
        } else {
            (url, "/")
        };

        let (host, port) = if let Some(idx) = host_port.rfind(':') {
            let port = host_port[idx + 1..]
                .parse::<u16>()
                .map_err(|_| IgdError::InvalidParameter(format!("Invalid port in URL")))?;
            (&host_port[..idx], port)
        } else {
            (host_port, 80u16)
        };

        Ok((host.to_string(), port, path.to_string()))
    }

    /// Parse SOAP error code from response
    fn parse_soap_error(response: &str) -> Option<u16> {
        // Look for <errorCode>718</errorCode> or similar
        if let Some(start) = response.find("<errorCode>") {
            let start = start + 11;
            if let Some(end) = response[start..].find("</errorCode>") {
                if let Ok(code) = response[start..start + end].parse::<u16>() {
                    return Some(code);
                }
            }
        }
        None
    }

    /// Parse external IP from GetExternalIPAddress response
    fn parse_external_ip(response: &str) -> Result<IpAddr> {
        // Look for <NewExternalIPAddress>1.2.3.4</NewExternalIPAddress>
        if let Some(start) = response.find("<NewExternalIPAddress>") {
            let start = start + 22;
            if let Some(end) = response[start..].find("</NewExternalIPAddress>") {
                let ip_str = &response[start..start + end];
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Ok(ip);
                }
            }
        }
        
        Err(IgdError::InvalidResponse(
            "Could not parse external IP from SOAP response".to_string(),
        ))
    }

    /// Map SOAP error code to IgdError
    fn map_soap_error(code: u16, port: u16) -> IgdError {
        match SoapErrorCode::from_code(code) {
            Some(SoapErrorCode::ConflictInMappingEntry) => {
                IgdError::MappingConflict(port, "unknown host".to_string())
            }
            Some(err) => IgdError::SoapError(format!(
                "SOAP error {}: {}",
                code,
                err.description()
            )),
            None => IgdError::SoapError(format!("Unknown SOAP error code: {}", code)),
        }
    }
}

/// Escape XML special characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Protocol;
    use std::net::Ipv4Addr;

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("test & <test>"), "test &amp; &lt;test&gt;");
        assert_eq!(xml_escape("Songbird's mapping"), "Songbird&apos;s mapping");
    }

    #[test]
    fn test_build_add_port_mapping_xml() {
        let soap = SoapClient::new(
            "http://192.168.1.254:5431/ctl/IPConn".to_string(),
            crate::WANIP_SERVICE_TYPE.to_string(),
        );

        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );

        let xml = soap.build_add_port_mapping_xml(&req);
        
        assert!(xml.contains("<NewExternalPort>3492</NewExternalPort>"));
        assert!(xml.contains("<NewInternalPort>3492</NewInternalPort>"));
        assert!(xml.contains("<NewInternalClient>192.168.1.144</NewInternalClient>"));
        assert!(xml.contains("<NewProtocol>TCP</NewProtocol>"));
    }

    #[test]
    fn test_parse_soap_error() {
        let error_response = r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
  <s:Body>
    <s:Fault>
      <faultcode>s:Client</faultcode>
      <faultstring>UPnPError</faultstring>
      <detail>
        <UPnPError xmlns="urn:schemas-upnp-org:control-1-0">
          <errorCode>718</errorCode>
          <errorDescription>ConflictInMappingEntry</errorDescription>
        </UPnPError>
      </detail>
    </s:Fault>
  </s:Body>
</s:Envelope>"#;

        let code = SoapClient::parse_soap_error(error_response);
        assert_eq!(code, Some(718));
    }

    #[test]
    fn test_parse_external_ip() {
        let response = r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
  <s:Body>
    <u:GetExternalIPAddressResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
      <NewExternalIPAddress>162.226.225.148</NewExternalIPAddress>
    </u:GetExternalIPAddressResponse>
  </s:Body>
</s:Envelope>"#;

        let ip = SoapClient::parse_external_ip(response).unwrap();
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(162, 226, 225, 148)));
    }

    #[test]
    fn test_parse_url() {
        let (host, port, path) =
            SoapClient::parse_url("http://192.168.1.254:5431/ctl/IPConn").unwrap();
        assert_eq!(host, "192.168.1.254");
        assert_eq!(port, 5431);
        assert_eq!(path, "/ctl/IPConn");
    }

    #[test]
    fn test_parse_url_default_port() {
        let (host, port, path) =
            SoapClient::parse_url("http://192.168.1.1/upnp/control").unwrap();
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 80);
        assert_eq!(path, "/upnp/control");
    }

    #[test]
    fn test_parse_url_no_path() {
        let (host, port, path) =
            SoapClient::parse_url("http://10.0.0.1:8080").unwrap();
        assert_eq!(host, "10.0.0.1");
        assert_eq!(port, 8080);
        assert_eq!(path, "/");
    }

    #[tokio::test]
    async fn test_soap_send_unreachable_host() {
        // Verify that connection to unreachable host returns clean error, not panic
        let soap = SoapClient::new(
            "http://203.0.113.1:5431/ctl/IPConn".to_string(), // RFC 5737 TEST-NET-3
            crate::WANIP_SERVICE_TYPE.to_string(),
        );

        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );

        // Should fail cleanly with timeout or connection error
        let result = soap.add_port_mapping(&req).await;
        assert!(result.is_err(), "Should fail for unreachable host");
    }
}
