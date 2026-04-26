// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network Discovery API
//!
//! Headless API for discovering Songbird networks that biomeOS can consume

use super::{DiscoveredNetwork, DiscoveryParameters};
use crate::errors::{CliError, SongbirdResult};
use std::net::IpAddr;

/// RFC 6762 mDNS multicast group and port (IANA assigned)
const MDNS_MULTICAST_ADDR: &str = "224.0.0.251:5353";

/// Maximum time to wait for discovery responses before giving up
const MAX_DISCOVERY_TIMEOUT_MS: u64 = 5000;

/// Resolve the discovery HTTP port from environment or canonical defaults
fn discovery_http_port() -> u16 {
    songbird_process_env::var("SONGBIRD_DISCOVERY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(songbird_config::canonical::constants::network::default_orchestrator_port)
}

/// Network discovery API endpoint
pub async fn discover_networks_api(
    params: DiscoveryParameters,
) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    let mut networks = Vec::new();

    for method in &params.methods {
        match method.as_str() {
            "subnet" => {
                networks.extend(discover_via_subnet_scan(params.timeout_ms).await?);
            }
            "dns" => {
                networks.extend(discover_via_dns(params.timeout_ms).await?);
            }
            "multicast" => {
                networks.extend(discover_via_multicast(params.timeout_ms).await?);
            }
            "mdns" => {
                networks.extend(discover_via_mdns(params.timeout_ms).await?);
            }
            "broadcast" => {
                networks.extend(discover_via_broadcast(params.timeout_ms).await?);
            }
            _ => {
                return Err(CliError::Config {
                    message: format!("Unknown discovery method: {method}"),
                    field: Some("discovery_method".to_string()),
                    suggestion: Some(
                        "Use 'subnet', 'dns', 'multicast', 'mdns', or 'broadcast'".to_string(),
                    ),
                }
                .into());
            }
        }

        if networks.len() >= params.max_results {
            networks.truncate(params.max_results);
            break;
        }
    }

    for network in &mut networks {
        network.compatibility_score = calculate_compatibility_score(network);
    }

    Ok(networks)
}

/// Calculate compatibility score for a network
fn calculate_compatibility_score(network: &DiscoveredNetwork) -> f64 {
    let mut score: f64 = 0.5;

    if network.node_count >= 3 && network.node_count <= 20 {
        score += 0.2;
    } else if network.node_count > 20 {
        score += 0.1;
    }

    if network.network_type.contains("Academic") || network.network_type.contains("Research") {
        score += 0.2;
    }

    if network.institution.is_some() {
        score += 0.1;
    }

    score.min(1.0)
}

/// Discover networks by probing the local subnet for Songbird TCP endpoints
async fn discover_via_subnet_scan(timeout_ms: u64) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    use std::net::UdpSocket;
    use tokio::net::TcpStream;

    let port = discovery_http_port();
    let timeout = std::time::Duration::from_millis(timeout_ms.min(MAX_DISCOVERY_TIMEOUT_MS));

    let local_ip = {
        let probe = UdpSocket::bind("0.0.0.0:0").map_err(|e| CliError::Network {
            message: format!("Failed to bind probe socket: {e}"),
            interface: None,
            suggestion: Some("Check network permissions".to_string()),
        })?;
        let route_target = songbird_process_env::var("SONGBIRD_ROUTE_DETECT_ADDR")
            .unwrap_or_else(|_| "192.0.2.1:80".to_string());
        let _ = probe.connect(route_target.as_str());
        probe.local_addr().map_or_else(|_| IpAddr::from([127, 0, 0, 1]), |a| a.ip())
    };

    let mut networks = Vec::new();

    if let IpAddr::V4(ipv4) = local_ip {
        let octets = ipv4.octets();

        let candidates: Vec<u8> = (1u8..=10).chain(std::iter::once(254)).collect();
        for last in candidates {
            if last == octets[3] {
                continue;
            }
            let addr = std::net::SocketAddr::from((
                std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], last),
                port,
            ));
            if tokio::time::timeout(timeout, TcpStream::connect(addr)).await.is_ok() {
                networks.push(DiscoveredNetwork {
                    name: format!("songbird-{}.{}.{}.{}", octets[0], octets[1], octets[2], last),
                    node_count: 1,
                    network_type: "Subnet".to_string(),
                    institution: None,
                    endpoint: addr.to_string(),
                    compatibility_score: 0.0,
                });
            }
        }
    }

    Ok(networks)
}

/// Discover networks via DNS-SD SRV record lookup
async fn discover_via_dns(timeout_ms: u64) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    use hickory_resolver::TokioAsyncResolver;
    use hickory_resolver::config::{ResolverConfig, ResolverOpts};

    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let service_name = "_songbird._tcp.local.";
    let timeout = std::time::Duration::from_millis(timeout_ms.min(MAX_DISCOVERY_TIMEOUT_MS));

    let mut networks = Vec::new();
    if let Ok(Ok(lookup)) = tokio::time::timeout(timeout, resolver.srv_lookup(service_name)).await {
        for record in lookup.iter() {
            let target_host = record.target().to_string();
            let port = record.port();
            if let Ok(Ok(addrs)) =
                tokio::time::timeout(timeout, resolver.lookup_ip(target_host.as_str())).await
                && let Some(ip) = addrs.iter().next()
            {
                networks.push(DiscoveredNetwork {
                    name: target_host.trim_end_matches('.').to_string(),
                    node_count: 1,
                    network_type: "DNS-SD".to_string(),
                    institution: None,
                    endpoint: format!("{ip}:{port}"),
                    compatibility_score: 0.0,
                });
            }
        }
    }

    Ok(networks)
}

/// Discover networks via multicast
async fn discover_via_multicast(timeout_ms: u64) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    use std::net::UdpSocket;
    use std::time::Duration;

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| CliError::Network {
        message: format!("Failed to create socket: {e}"),
        interface: Some("0.0.0.0:0".to_string()),
        suggestion: Some("Check network permissions and available ports".to_string()),
    })?;

    socket
        .set_read_timeout(Some(Duration::from_millis(timeout_ms.min(MAX_DISCOVERY_TIMEOUT_MS))))
        .map_err(|e| CliError::Network {
            message: format!("Failed to set timeout: {e}"),
            interface: Some("socket".to_string()),
            suggestion: Some("Check socket configuration".to_string()),
        })?;

    let discovery_msg = b"SONGBIRD_DISCOVERY_v1";
    let _ = socket.send_to(discovery_msg, MDNS_MULTICAST_ADDR);

    let mut buf = [0u8; 2048];
    let mut networks = Vec::new();

    for _ in 0..5 {
        if let Ok((len, addr)) = socket.recv_from(&mut buf)
            && let Ok(response) = std::str::from_utf8(&buf[..len])
            && let Some(network) = parse_discovery_response(response, addr.ip())
        {
            networks.push(network);
        }
    }

    Ok(networks)
}

/// Discover networks via mDNS multicast query
async fn discover_via_mdns(timeout_ms: u64) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    use std::net::UdpSocket;
    use std::time::Duration;

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| CliError::Network {
        message: format!("Failed to bind mDNS socket: {e}"),
        interface: None,
        suggestion: Some("Check network permissions".to_string()),
    })?;

    let timeout = Duration::from_millis(timeout_ms.min(MAX_DISCOVERY_TIMEOUT_MS));
    socket.set_read_timeout(Some(timeout)).map_err(|e| CliError::Network {
        message: format!("Failed to set mDNS timeout: {e}"),
        interface: None,
        suggestion: None,
    })?;

    let _ = socket.send_to(b"SONGBIRD_MDNS_QUERY_v1", MDNS_MULTICAST_ADDR);

    let mut buf = [0u8; 2048];
    let mut networks = Vec::new();

    for _ in 0..5 {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(text) = std::str::from_utf8(&buf[..len])
                    && let Some(net) = parse_discovery_response(text, addr.ip())
                {
                    networks.push(net);
                }
            }
            Err(_) => break,
        }
    }

    Ok(networks)
}

/// Discover networks via UDP broadcast on the local subnet
async fn discover_via_broadcast(timeout_ms: u64) -> SongbirdResult<Vec<DiscoveredNetwork>> {
    use std::net::UdpSocket;
    use std::time::Duration;

    let port = discovery_http_port();
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| CliError::Network {
        message: format!("Failed to bind broadcast socket: {e}"),
        interface: None,
        suggestion: Some("Check network permissions".to_string()),
    })?;

    socket.set_broadcast(true).map_err(|e| CliError::Network {
        message: format!("Failed to enable broadcast: {e}"),
        interface: None,
        suggestion: Some("Broadcast may not be supported on this interface".to_string()),
    })?;

    let timeout = Duration::from_millis(timeout_ms.min(MAX_DISCOVERY_TIMEOUT_MS));
    socket.set_read_timeout(Some(timeout)).map_err(|e| CliError::Network {
        message: format!("Failed to set broadcast timeout: {e}"),
        interface: None,
        suggestion: None,
    })?;

    let broadcast_addr = format!("255.255.255.255:{port}");
    let _ = socket.send_to(b"SONGBIRD_BROADCAST_DISCOVERY_v1", &broadcast_addr);

    let mut buf = [0u8; 2048];
    let mut networks = Vec::new();

    for _ in 0..5 {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(text) = std::str::from_utf8(&buf[..len])
                    && let Some(net) = parse_discovery_response(text, addr.ip())
                {
                    networks.push(net);
                }
            }
            Err(_) => break,
        }
    }

    Ok(networks)
}

/// Parse discovery response from network
fn parse_discovery_response(response: &str, source_ip: IpAddr) -> Option<DiscoveredNetwork> {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(response) {
        let name = data["name"].as_str()?.to_string();
        let node_count = usize::try_from(data["nodes"].as_u64()?).ok()?;
        let network_type = data["type"].as_str()?.to_string();
        let institution = data["institution"].as_str().map(std::string::ToString::to_string);

        Some(DiscoveredNetwork {
            name,
            node_count,
            network_type,
            institution,
            endpoint: format!("{source_ip}:{}", discovery_http_port()),
            compatibility_score: 0.0,
        })
    } else {
        None
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{calculate_compatibility_score, discover_networks_api, parse_discovery_response};
    use crate::cli::commands::quick::{DiscoveredNetwork, DiscoveryParameters};
    use songbird_process_env;
    use std::net::IpAddr;

    #[test]
    fn compatibility_score_caps_at_one() {
        let net = DiscoveredNetwork {
            name: "big".into(),
            node_count: 10,
            network_type: "Academic grid".into(),
            institution: Some("Inst".into()),
            endpoint: "10.0.0.1:80".into(),
            compatibility_score: 0.0,
        };
        let s = calculate_compatibility_score(&net);
        assert!((s - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compatibility_score_mid_range_without_bonuses() {
        let net = DiscoveredNetwork {
            name: "solo".into(),
            node_count: 1,
            network_type: "Subnet".into(),
            institution: None,
            endpoint: "127.0.0.1:1".into(),
            compatibility_score: 0.0,
        };
        let s = calculate_compatibility_score(&net);
        assert!((s - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_discovery_response_reads_json_fields() {
        songbird_process_env::set_var("SONGBIRD_DISCOVERY_PORT", "7777");
        let src: IpAddr = "198.51.100.2".parse().unwrap();
        let json = r#"{"name":"lab-net","nodes":4,"type":"Research","institution":"U"}"#;
        let net = parse_discovery_response(json, src).unwrap();
        songbird_process_env::reset_overlay();
        assert_eq!(net.name, "lab-net");
        assert_eq!(net.node_count, 4);
        assert_eq!(net.network_type, "Research");
        assert_eq!(net.institution.as_deref(), Some("U"));
        assert_eq!(net.endpoint, "198.51.100.2:7777");
    }

    #[test]
    fn parse_discovery_response_rejects_non_json() {
        let src: IpAddr = "127.0.0.1".parse().unwrap();
        assert!(parse_discovery_response("not-json", src).is_none());
    }

    #[tokio::test]
    async fn discover_networks_api_unknown_method_err() {
        let params = DiscoveryParameters {
            methods: vec!["quantum-entanglement".into()],
            timeout_ms: 100,
            max_results: 3,
        };
        let err = discover_networks_api(params).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unknown discovery method"), "{msg}");
    }
}
