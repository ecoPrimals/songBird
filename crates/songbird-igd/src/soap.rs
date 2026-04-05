// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! SOAP (Simple Object Access Protocol) control for `UPnP` IGD
//!
//! Implements SOAP XML envelope construction and parsing for `UPnP` port mapping operations.
//! Uses Songbird's HTTP client for the actual HTTP POSTs.

use crate::error::{IgdError, Result, SoapErrorCode};
use crate::mapping::PortMappingRequest;
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, trace};

/// SOAP client for `UPnP` IGD control
pub struct SoapClient {
    /// Control URL for SOAP actions
    control_url: String,

    /// Service type (for SOAP namespace)
    service_type: String,
}

impl SoapClient {
    /// Create new SOAP client
    #[must_use]
    pub const fn new(control_url: String, service_type: String) -> Self {
        Self {
            control_url,
            service_type,
        }
    }

    /// Add a port mapping
    ///
    /// # Errors
    ///
    /// Returns an error if the SOAP request fails or the gateway returns an error.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the SOAP request fails or the gateway returns an error.
    pub async fn delete_port_mapping(&self, external_port: u16, protocol: &str) -> Result<()> {
        debug!("Deleting port mapping: {}:{}", external_port, protocol);

        let body = self.build_delete_port_mapping_xml(external_port, protocol);
        let response = self.send_soap_action("DeletePortMapping", &body).await?;

        // Check for errors
        if let Some(error_code) = Self::parse_soap_error(&response) {
            return Err(IgdError::SoapError(format!(
                "Delete failed with code {error_code}: {}",
                SoapErrorCode::from_code(error_code).map_or("Unknown error", |e| e.description())
            )));
        }

        debug!("Port mapping deleted successfully");
        Ok(())
    }

    /// Get external IP address from gateway
    ///
    /// # Errors
    ///
    /// Returns an error if the SOAP request fails or the response cannot be parsed.
    pub async fn get_external_ip(&self) -> Result<IpAddr> {
        debug!("Querying external IP address");

        let body = self.build_get_external_ip_xml();
        let response = self.send_soap_action("GetExternalIPAddress", &body).await?;

        // Parse IP from response
        let ip = Self::parse_external_ip(&response)?;
        debug!("External IP: {}", ip);

        Ok(ip)
    }

    /// Build `AddPortMapping` SOAP XML
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

    /// Build `DeletePortMapping` SOAP XML
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

    /// Build `GetExternalIPAddress` SOAP XML
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
    /// `UPnP` SOAP calls are plain HTTP (not HTTPS) to local LAN addresses,
    /// so we use raw TCP rather than songbird-http-client (which is for TLS).
    async fn send_soap_action(&self, action: &str, body: &str) -> Result<String> {
        trace!("SOAP Action: {} to {}", action, self.control_url);
        trace!("SOAP Body:\n{}", body);

        // Parse the control URL to extract host, port, and path
        let (host, port, path) = Self::parse_url(&self.control_url)?;

        let soap_action = format!("\"{}\"#{action}", self.service_type);
        let content_length = body.len();

        // Build HTTP POST request
        let request = format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}:{port}\r\n\
             Content-Type: text/xml; charset=\"utf-8\"\r\n\
             Content-Length: {content_length}\r\n\
             SOAPAction: {soap_action}\r\n\
             Connection: close\r\n\
             \r\n\
             {body}"
        );

        // Connect and send

        let addr = format!("{host}:{port}");
        let mut stream =
            tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(&addr))
                .await
                .map_err(|_| IgdError::Timeout)?
                .map_err(|e| IgdError::SoapError(format!("Failed to connect to {addr}: {e}")))?;

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| IgdError::SoapError(format!("Failed to send SOAP request: {e}")))?;

        // Read response
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .map_err(|e| IgdError::SoapError(format!("Failed to read SOAP response: {e}")))?;

        let response_str = String::from_utf8_lossy(&response).to_string();
        trace!("SOAP Response:\n{}", response_str);

        // Extract body from HTTP response (skip headers)
        let body_start = response_str.find("\r\n\r\n").map_or(0, |i| i + 4);
        let response_body = &response_str[body_start..];

        // Check HTTP status
        if let Some(status_line) = response_str.lines().next()
            && !status_line.contains("200")
            && !status_line.contains("OK")
        {
            // UPnP sends 500 for SOAP faults — the body contains the error details
            if status_line.contains("500") {
                debug!("SOAP fault received (HTTP 500), parsing error details");
                return Ok(response_body.to_string());
            }
            return Err(IgdError::SoapError(format!("HTTP error: {status_line}")));
        }

        Ok(response_body.to_string())
    }

    /// Parse a URL into (host, port, path) components
    ///
    /// Handles: `http://192.168.1.254:5431/ctl/IPConn`
    fn parse_url(url: &str) -> Result<(String, u16, String)> {
        let url = url.strip_prefix("http://").or_else(|| url.strip_prefix("HTTP://")).ok_or_else(
            || IgdError::InvalidParameter(format!("Expected http:// URL, got: {url}")),
        )?;

        let (host_port, path) = url.find('/').map_or((url, "/"), |idx| (&url[..idx], &url[idx..]));

        let (host, port) = if let Some(idx) = host_port.rfind(':') {
            let port = host_port[idx + 1..]
                .parse::<u16>()
                .map_err(|_| IgdError::InvalidParameter("Invalid port in URL".to_string()))?;
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
            if let Some(end) = response[start..].find("</errorCode>")
                && let Ok(code) = response[start..start + end].parse::<u16>()
            {
                return Some(code);
            }
        }
        None
    }

    /// Parse external IP from `GetExternalIPAddress` response
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

        Err(IgdError::InvalidResponse("Could not parse external IP from SOAP response".to_string()))
    }

    /// Map SOAP error code to `IgdError`
    fn map_soap_error(code: u16, port: u16) -> IgdError {
        match SoapErrorCode::from_code(code) {
            Some(SoapErrorCode::ConflictInMappingEntry) => {
                IgdError::MappingConflict(port, "unknown host".to_string())
            }
            Some(err) => IgdError::SoapError(format!("SOAP error {code}: {}", err.description())),
            None => IgdError::SoapError(format!("Unknown SOAP error code: {code}")),
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
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::error::SoapErrorCode;
    use crate::mapping::Protocol;
    use std::net::Ipv4Addr;

    #[test]
    fn xml_escape_escapes_special_chars() {
        assert_eq!(
            xml_escape("test & <test>"),
            "test &amp; &lt;test&gt;",
            "&, <, > must be escaped for SOAP/XML"
        );
        assert_eq!(xml_escape("Songbird's mapping"), "Songbird&apos;s mapping");
        assert_eq!(
            xml_escape("\"q\""),
            "&quot;q&quot;",
            "double quotes must be escaped in attributes/text"
        );
    }

    #[test]
    fn build_add_port_mapping_xml_contains_expected_elements() {
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
        assert!(xml.contains("<NewInternalClient>192.0.2.10</NewInternalClient>"));
        assert!(xml.contains("<NewProtocol>TCP</NewProtocol>"));
        assert!(xml.contains("<NewLeaseDuration>86400</NewLeaseDuration>"));
    }

    #[test]
    fn build_delete_port_mapping_xml_contains_port_and_protocol() {
        let soap = SoapClient::new(
            "http://192.168.1.1/ctl".to_string(),
            crate::WANIP_SERVICE_TYPE.to_string(),
        );
        let xml = soap.build_delete_port_mapping_xml(443, "UDP");
        assert!(xml.contains("<NewExternalPort>443</NewExternalPort>"));
        assert!(xml.contains("<NewProtocol>UDP</NewProtocol>"));
    }

    #[test]
    fn build_get_external_ip_xml_includes_service_namespace() {
        let soap = SoapClient::new(
            "http://192.168.1.1/ctl".to_string(),
            crate::WANIP_SERVICE_TYPE.to_string(),
        );
        let xml = soap.build_get_external_ip_xml();
        assert!(xml.contains("GetExternalIPAddress"));
        assert!(xml.contains(crate::WANIP_SERVICE_TYPE));
    }

    #[test]
    fn parse_soap_error_finds_error_code() {
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
    fn parse_soap_error_returns_none_when_missing() {
        assert_eq!(SoapClient::parse_soap_error("<xml></xml>"), None);
    }

    #[test]
    fn parse_external_ip_reads_ipv4() {
        let response = r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
  <s:Body>
    <u:GetExternalIPAddressResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
      <NewExternalIPAddress>198.51.100.1</NewExternalIPAddress>
    </u:GetExternalIPAddressResponse>
  </s:Body>
</s:Envelope>"#;

        let ip = SoapClient::parse_external_ip(response).expect("valid SOAP body");
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(162, 226, 225, 148)));
    }

    #[test]
    fn parse_external_ip_rejects_malformed_body() {
        let err = SoapClient::parse_external_ip("<empty/>").expect_err("no NewExternalIPAddress");
        assert!(
            matches!(err, IgdError::InvalidResponse(_)),
            "expected InvalidResponse, got {err:?}"
        );
    }

    #[test]
    fn map_soap_error_maps_known_codes() {
        let e = SoapClient::map_soap_error(718, 8080);
        assert!(
            matches!(e, IgdError::MappingConflict(8080, _)),
            "718 should become MappingConflict: {e:?}"
        );

        let e = SoapClient::map_soap_error(501, 0);
        match e {
            IgdError::SoapError(s) => {
                assert!(s.contains("501"), "message should mention code: {s}");
                assert!(s.contains(SoapErrorCode::ActionFailed.description()));
            }
            other => panic!("expected SoapError, got {other:?}"),
        }

        let e = SoapClient::map_soap_error(9999, 1);
        assert!(
            matches!(e, IgdError::SoapError(_)),
            "unknown code maps to SoapError string: {e:?}"
        );
    }

    #[test]
    fn parse_url_splits_host_port_path() {
        let (host, port, path) =
            SoapClient::parse_url("http://192.168.1.254:5431/ctl/IPConn").expect("valid URL");
        assert_eq!(host, "192.168.1.254");
        assert_eq!(port, 5431);
        assert_eq!(path, "/ctl/IPConn");
    }

    #[test]
    fn parse_url_default_port_80() {
        let (host, port, path) =
            SoapClient::parse_url("http://192.168.1.1/upnp/control").expect("valid URL");
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 80);
        assert_eq!(path, "/upnp/control");
    }

    #[test]
    fn parse_url_no_explicit_path_defaults_to_slash() {
        let (host, port, path) = SoapClient::parse_url("http://10.0.0.1:8080").expect("valid URL");
        assert_eq!(host, "10.0.0.1");
        assert_eq!(port, 8080);
        assert_eq!(path, "/");
    }

    #[test]
    fn parse_url_rejects_non_http_scheme() {
        let err =
            SoapClient::parse_url("https://192.168.1.1/ctl").expect_err("only http:// supported");
        assert!(
            matches!(err, IgdError::InvalidParameter(_)),
            "expected InvalidParameter, got {err:?}"
        );
    }

    #[test]
    fn parse_url_rejects_invalid_port() {
        let err =
            SoapClient::parse_url("http://192.168.1.1:bad/").expect_err("port must be numeric");
        assert!(
            matches!(err, IgdError::InvalidParameter(_)),
            "expected InvalidParameter, got {err:?}"
        );
    }

    #[tokio::test]
    async fn add_port_mapping_unreachable_host_returns_error() {
        let soap = SoapClient::new(
            "http://203.0.113.1:5431/ctl/IPConn".to_string(),
            crate::WANIP_SERVICE_TYPE.to_string(),
        );

        let req = PortMappingRequest::new(
            3492,
            3492,
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 144)),
            Protocol::Tcp,
        );

        let result = soap.add_port_mapping(&req).await;
        assert!(result.is_err(), "unreachable TEST-NET-3 host should yield error, got {result:?}");
    }
}
