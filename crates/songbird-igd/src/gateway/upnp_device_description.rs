// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `UPnP` device description fetch and lightweight XML parsing for IGD control URLs.

use crate::error::{IgdError, Result};
use tracing::debug;

/// Fetch and parse `UPnP` device description XML
///
/// Returns (controlURL, serviceType, friendlyName)
pub(super) async fn fetch_device_description(
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
    let control_url = extract_control_url(xml, location_url)?;
    let service_type = extract_xml_value(xml, "serviceType")
        .filter(|st| st.contains("WANIPConnection") || st.contains("WANPPPConnection"))
        .unwrap_or_else(|| crate::WANIP_SERVICE_TYPE.to_string());
    let friendly_name = extract_xml_value(xml, "friendlyName");

    Ok((control_url, service_type, friendly_name))
}

/// Extract controlURL from device description XML
///
/// Finds the `WANIPConnection` or `WANPPPConnection` service block and
/// extracts its controlURL. The controlURL may be relative (needs
/// base URL prepended) or absolute.
pub(super) fn extract_control_url(xml: &str, base_url: &str) -> Result<String> {
    // Find the WANIPConnection or WANPPPConnection service section
    let wan_markers = ["WANIPConnection", "WANPPPConnection"];

    for marker in &wan_markers {
        if let Some(service_pos) = xml.find(marker) {
            // Look for controlURL after this marker
            let after_marker = &xml[service_pos..];
            if let Some(ctl) = extract_xml_value(after_marker, "controlURL") {
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
pub(super) fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
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
