// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! SSDP (`Simple Service Discovery Protocol`) message helpers.
//!
//! Pure parsing and framing — no sockets — for unit tests and callers that
//! build or interpret multicast discovery traffic.

use std::collections::HashMap;

/// Parsed SSDP response (HTTP 200 advertisement or search reply).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsdpMessage {
    /// `LOCATION` header (device description URL).
    pub location: String,
    /// `ST` / `NT` header (service or device type URN).
    pub service_type: String,
    /// `USN` header when present.
    pub usn: Option<String>,
    /// `SERVER` header when present.
    pub server: Option<String>,
    /// `CACHE-CONTROL` raw value when present.
    pub cache_control: Option<String>,
}

fn header_map(body: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in body.lines() {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_uppercase(), v.trim().to_string());
        }
    }
    headers
}

/// Parse an SSDP HTTP-style response (UTF-8).
///
/// Accepts `HTTP/1.1 200 OK` replies commonly seen on UDP/1900.
#[must_use]
pub fn parse_ssdp_response(text: &str) -> Option<SsdpMessage> {
    let mut lines = text.lines();
    let status = lines.next()?;
    if !status.starts_with("HTTP/1.") || !status.contains("200") {
        return None;
    }
    let rest: String = lines.fold(String::new(), |mut acc, line| {
        if !acc.is_empty() {
            acc.push('\n');
        }
        acc.push_str(line);
        acc
    });
    let h = header_map(&rest);
    Some(SsdpMessage {
        location: h.get("LOCATION")?.clone(),
        service_type: h.get("ST").or_else(|| h.get("NT"))?.clone(),
        usn: h.get("USN").cloned(),
        server: h.get("SERVER").cloned(),
        cache_control: h.get("CACHE-CONTROL").cloned(),
    })
}

/// Build an `M-SEARCH` request for multicast SSDP.
#[must_use]
pub fn build_msearch(service_type: &str, mx: u8) -> String {
    format!(
        "M-SEARCH * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         MAN: \"ssdp:discover\"\r\n\
         ST: {service_type}\r\n\
         MX: {mx}\r\n\
         \r\n"
    )
}

/// Build a `NOTIFY` alive advertisement (root device).
#[must_use]
pub fn build_notify_alive(location: &str, nt: &str, usn: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\n\
         HOST: 239.255.255.250:1900\r\n\
         CACHE-CONTROL: max-age=1800\r\n\
         LOCATION: {location}\r\n\
         NT: {nt}\r\n\
         NTS: ssdp:alive\r\n\
         SERVER: songbird-discovery/1.0 UPnP/1.0\r\n\
         USN: {usn}\r\n\
         \r\n"
    )
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn parse_ssdp_response_extracts_core_headers() {
        let raw = "HTTP/1.1 200 OK\r\n\
            CACHE-CONTROL: max-age=1800\r\n\
            LOCATION: http://192.168.0.1/root.xml\r\n\
            ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
            USN: uuid:abc::urn:device:IGD\r\n\
            SERVER: test/1.0\r\n\
            \r\n";
        let m = parse_ssdp_response(raw).expect("parse");
        assert_eq!(m.location, "http://192.168.0.1/root.xml");
        assert!(m.service_type.contains("InternetGatewayDevice"));
        assert_eq!(m.usn.as_deref(), Some("uuid:abc::urn:device:IGD"));
        assert_eq!(m.server.as_deref(), Some("test/1.0"));
        assert_eq!(m.cache_control.as_deref(), Some("max-age=1800"));
    }

    #[test]
    fn parse_ssdp_accepts_nt_instead_of_st() {
        let raw = "HTTP/1.1 200 OK\r\n\
            LOCATION: http://10.0.0.1/desc.xml\r\n\
            NT: urn:schemas-upnp-org:service:WANIPConnection:1\r\n\
            \r\n";
        let m = parse_ssdp_response(raw).expect("parse");
        assert_eq!(m.location, "http://10.0.0.1/desc.xml");
        assert!(m.service_type.contains("WANIPConnection"));
    }

    #[test]
    fn parse_rejects_non_200_status() {
        let raw = "HTTP/1.1 404 Not Found\r\nLOCATION: http://x\r\nST: x\r\n\r\n";
        assert!(parse_ssdp_response(raw).is_none());
    }

    #[test]
    fn parse_rejects_missing_location() {
        let raw = "HTTP/1.1 200 OK\r\nST: urn:x\r\n\r\n";
        assert!(parse_ssdp_response(raw).is_none());
    }

    #[test]
    fn msearch_contains_host_and_st() {
        let m = build_msearch("urn:schemas-upnp-org:device:InternetGatewayDevice:1", 3);
        assert!(m.contains("M-SEARCH * HTTP/1.1"));
        assert!(m.contains("HOST: 239.255.255.250:1900"));
        assert!(m.contains("MAN: \"ssdp:discover\""));
        assert!(m.contains("InternetGatewayDevice"));
        assert!(m.contains("MX: 3"));
    }

    #[test]
    fn msearch_mx_varies() {
        assert!(build_msearch("st:test", 1).contains("MX: 1"));
        assert!(build_msearch("st:test", 10).contains("MX: 10"));
    }

    #[test]
    fn notify_alive_contains_nts_and_location() {
        let n = build_notify_alive(
            "http://192.168.1.1/device.xml",
            "upnp:rootdevice",
            "uuid:deadbeef::upnp:rootdevice",
        );
        assert!(n.starts_with("NOTIFY * HTTP/1.1"));
        assert!(n.contains("LOCATION: http://192.168.1.1/device.xml"));
        assert!(n.contains("NT: upnp:rootdevice"));
        assert!(n.contains("NTS: ssdp:alive"));
        assert!(n.contains("uuid:deadbeef::upnp:rootdevice"));
    }

    #[test]
    fn parse_empty_string_fails() {
        assert!(parse_ssdp_response("").is_none());
    }

    #[test]
    fn parse_utf8_multiline_headers() {
        let raw = "HTTP/1.1 200 OK\r\nLOCATION: http://[::1]:5000/x\r\nST: st\r\n\r\n";
        let m = parse_ssdp_response(raw).expect("parse");
        assert_eq!(m.location, "http://[::1]:5000/x");
    }

    #[test]
    fn header_map_skips_lines_without_colon() {
        let raw = "HTTP/1.1 200 OK\r\nnot_a_header\r\nLOCATION: http://a\r\nST: s\r\n\r\n";
        let m = parse_ssdp_response(raw).expect("parse");
        assert_eq!(m.location, "http://a");
    }

    #[test]
    fn notify_alive_includes_server_banner() {
        let n = build_notify_alive("http://loc", "nt", "usn");
        assert!(n.contains("SERVER: songbird-discovery/1.0"));
    }

    #[test]
    fn parse_preserves_trimming_on_header_values() {
        let raw = "HTTP/1.1 200 OK\r\n  LOCATION:  http://x  \r\n  ST:  y  \r\n\r\n";
        let m = parse_ssdp_response(raw).expect("parse");
        assert_eq!(m.location, "http://x");
        assert_eq!(m.service_type, "y");
    }
}
