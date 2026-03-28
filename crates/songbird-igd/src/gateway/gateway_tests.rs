// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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
    let err = gw.unmap_port(443, "QUIC").await.unwrap_err();
    assert!(matches!(err, IgdError::InvalidParameter(_)));
}
