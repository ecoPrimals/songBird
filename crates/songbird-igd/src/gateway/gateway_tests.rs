// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::upnp_device_description;
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

    assert_eq!(
        upnp_device_description::extract_xml_value(xml, "friendlyName"),
        Some("BGW320-505".to_string())
    );
    assert_eq!(
        upnp_device_description::extract_xml_value(xml, "controlURL"),
        Some("/ctl/IPConn".to_string())
    );
    assert_eq!(upnp_device_description::extract_xml_value(xml, "nonexistent"), None);
}

#[test]
fn test_extract_control_url_relative() {
    let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>/ctl/IPConn</controlURL>
        </service>";

    let result =
        upnp_device_description::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
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

    let result =
        upnp_device_description::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
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

    let result =
        upnp_device_description::extract_control_url(xml, "http://192.168.0.1:49000/rootDesc.xml");
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

    let result =
        upnp_device_description::extract_control_url(xml, "http://192.168.1.254:5431/rootDesc.xml");
    assert!(result.is_err());
}

#[test]
fn test_extract_xml_value_with_attributes() {
    // XML tags might have attributes
    let xml = r#"<controlURL xmlns="urn:schemas-upnp-org:device-1-0">/ctl/IPConn</controlURL>"#;
    assert_eq!(
        upnp_device_description::extract_xml_value(xml, "controlURL"),
        Some("/ctl/IPConn".to_string())
    );
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
    let err = gw.unmap_port(443, "QUIC").await.expect_err("QUIC is not a valid IGD protocol");
    assert!(matches!(err, IgdError::InvalidParameter(_)));
}

#[test]
fn gateway_nat_pmp_is_available() {
    let gw = Gateway {
        ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        protocol: GatewayProtocol::NatPmp,
        external_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))),
        device_name: None,
        other_devices: Vec::new(),
    };
    assert!(gw.is_available(), "NAT-PMP should count as an available IGD protocol");
}

#[tokio::test]
async fn unmap_port_unsupported_when_protocol_none() {
    let gw = Gateway {
        ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        protocol: GatewayProtocol::None,
        external_ip: None,
        device_name: None,
        other_devices: Vec::new(),
    };
    let err =
        gw.unmap_port(80, "TCP").await.expect_err("unmap should fail without a backing protocol");
    assert!(
        matches!(err, IgdError::ProtocolNotSupported(_)),
        "expected ProtocolNotSupported, got {err:?}"
    );
}

#[test]
fn discovery_diagnostics_serde_roundtrip() {
    let d = DiscoveryDiagnostics {
        gateway_ip: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        gateway_reachable: true,
        upnp: UpnpDiscoveryStatus {
            ssdp_sent: true,
            igd_found: false,
        },
        upnp_devices_found: vec!["urn:device (192.168.1.1:1900)".to_string()],
        nat_pmp: NatPmpDiscoveryStatus {
            probe_sent: true,
            responded: false,
        },
        manual_instructions: vec!["open router".to_string()],
        alternative_tiers: vec!["onion".to_string()],
    };
    let json = serde_json::to_string(&d).expect("serialize diagnostics");
    let back: DiscoveryDiagnostics = serde_json::from_str(&json).expect("deserialize diagnostics");
    assert_eq!(d.gateway_ip, back.gateway_ip);
    assert_eq!(d.upnp.ssdp_sent, back.upnp.ssdp_sent);
    assert_eq!(d.nat_pmp.responded, back.nat_pmp.responded);
}

#[tokio::test]
async fn fetch_device_description_rejects_non_http_location() {
    let err = upnp_device_description::fetch_device_description("https://192.168.1.1/desc.xml")
        .await
        .expect_err("only http:// locations are supported");
    assert!(
        matches!(err, IgdError::InvalidResponse(_)),
        "expected InvalidResponse for bad scheme, got {err:?}"
    );
}

#[test]
fn extract_xml_value_empty_element_returns_none() {
    let xml = "<friendlyName></friendlyName>";
    assert_eq!(
        upnp_device_description::extract_xml_value(xml, "friendlyName"),
        None,
        "empty trimmed content should not produce Some(\"\")"
    );
}

#[test]
fn gateway_protocol_none_serde_roundtrip() {
    let p = GatewayProtocol::None;
    let json = serde_json::to_string(&p).expect("serialize");
    let back: GatewayProtocol = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(back, GatewayProtocol::None));
}

#[test]
fn gateway_protocol_upnp_igd_serde_roundtrip() {
    let p = GatewayProtocol::UpnpIgd {
        control_url: "http://192.168.1.254/ctl".to_string(),
        service_type: crate::WANIP_SERVICE_TYPE.to_string(),
        device_name: Some("gw".to_string()),
    };
    let json = serde_json::to_string(&p).expect("serialize");
    let back: GatewayProtocol = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(back, GatewayProtocol::UpnpIgd { .. }));
}

#[test]
fn extract_control_url_relative_without_leading_slash() {
    let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>ctl/IPConn</controlURL>
        </service>";
    let url = upnp_device_description::extract_control_url(xml, "http://192.168.1.2:8080/root.xml")
        .expect("relative path without leading slash should resolve");
    assert_eq!(url, "http://192.168.1.2:8080/ctl/IPConn");
}

#[test]
fn extract_control_url_https_absolute_passthrough() {
    let xml = r"
        <service>
            <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
            <controlURL>https://192.168.1.254:8443/ctl/IPConn</controlURL>
        </service>";
    let url = upnp_device_description::extract_control_url(xml, "http://192.168.1.254/root.xml")
        .expect("https absolute controlURL");
    assert_eq!(url, "https://192.168.1.254:8443/ctl/IPConn");
}

#[test]
fn extract_xml_value_trims_inner_whitespace() {
    let xml = "<friendlyName>  My Router  </friendlyName>";
    assert_eq!(
        upnp_device_description::extract_xml_value(xml, "friendlyName"),
        Some("My Router".to_string())
    );
}

#[tokio::test]
async fn fetch_device_description_unreachable_host_returns_error() {
    let err =
        upnp_device_description::fetch_device_description("http://203.0.113.9:49000/desc.xml")
            .await
            .expect_err("unreachable TEST-NET-3 host");
    assert!(
        matches!(err, IgdError::Timeout | IgdError::SoapError(_)),
        "expected timeout or connection error, got {err:?}"
    );
}
