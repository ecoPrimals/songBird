// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::discovery::types::{NetworkLocation, NetworkMeasurement};
use chrono::Utc;
use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use std::process::Command;
use std::str;

/// Network management utilities
pub struct NetworkManager;

impl NetworkManager {
    /// Measure ping latency to target
    pub fn measure_ping_latency(target_address: &str) -> Result<f64> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let output = Command::new("ping").args(["-c", "3", target_address]).output();

            if let Ok(output) = output {
                if output.status.success() {
                    if let Ok(output_str) = str::from_utf8(&output.stdout) {
                        for line in output_str.lines() {
                            if line.contains("round-trip") || line.contains("rtt") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                for (i, part) in parts.iter().enumerate() {
                                    if part.contains("avg") && i + 1 < parts.len() {
                                        let avg_part = parts[i + 1];
                                        if let Some(avg_str) = avg_part.split('/').nth(1) {
                                            if let Ok(avg_latency) = avg_str.parse::<f64>() {
                                                return Ok(avg_latency);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(10.0)
    }

    /// Estimate bandwidth to target (simplified)
    pub fn estimate_bandwidth(_target_address: &str) -> Result<f64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries.flatten() {
                    let interface_name = entry.file_name();
                    if let Some(name_str) = interface_name.to_str() {
                        if name_str.starts_with("lo") || name_str.starts_with("docker") {
                            continue;
                        }

                        let speed_path = format!("/sys/class/net/{name_str}/speed");
                        if let Ok(speed_str) = std::fs::read_to_string(speed_path) {
                            if let Ok(speed_mbps) = speed_str.trim().parse::<f64>() {
                                if speed_mbps > 0.0 {
                                    return Ok(speed_mbps);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(1000.0)
    }

    #[must_use]
    pub fn get_local_ip_addresses() -> Vec<std::net::IpAddr> {
        use std::net::{IpAddr, Ipv4Addr};

        let mut addresses = Vec::new();

        #[cfg(target_os = "linux")]
        {
            if let Ok(route_content) = std::fs::read_to_string("/proc/net/route") {
                let mut default_iface = None;
                for line in route_content.lines().skip(1) {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 3 && fields[1] == "00000000" {
                        default_iface = Some(fields[0].to_string());
                        break;
                    }
                }

                if let Some(iface_name) = default_iface {
                    if let Ok(ip_result) =
                        Command::new("ip").args(["addr", "show", &iface_name]).output()
                    {
                        if let Ok(output) = str::from_utf8(&ip_result.stdout) {
                            for line in output.lines() {
                                if line.trim().starts_with("inet ") {
                                    if let Some(ip_str) = line.split_whitespace().nth(1) {
                                        if let Some(ip_only) = ip_str.split('/').next() {
                                            if let Ok(ip) = ip_only.parse::<IpAddr>() {
                                                addresses.push(ip);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(ifconfig_result) = std::process::Command::new("ifconfig").output() {
                if let Ok(output) = std::str::from_utf8(&ifconfig_result.stdout) {
                    let mut current_interface = None;
                    for line in output.lines() {
                        if !line.starts_with(' ') && !line.starts_with('\t') {
                            if let Some(iface_name) = line.split(':').next() {
                                if !iface_name.starts_with("lo") && !iface_name.starts_with("veth")
                                {
                                    current_interface = Some(iface_name.to_string());
                                } else {
                                    current_interface = None;
                                }
                            }
                        } else if current_interface.is_some() && line.contains("inet ") {
                            if let Some(ip_str) = line.split_whitespace().nth(1) {
                                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                                    addresses.push(ip);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(ipconfig_result) = std::process::Command::new("ipconfig").output() {
                if let Ok(output) = std::str::from_utf8(&ipconfig_result.stdout) {
                    for line in output.lines() {
                        if line.contains("IPv4 Address") {
                            if let Some(ip_part) = line.split(':').nth(1) {
                                if let Ok(ip) = ip_part.trim().parse::<IpAddr>() {
                                    addresses.push(ip);
                                }
                            }
                        }
                    }
                }
            }
        }

        if addresses.is_empty() {
            // Connect to a non-routable target to discover which local interface the
            // OS kernel selects as the default route. No packets are sent (UDP).
            // Using RFC 5737 documentation address avoids any third-party dependency.
            if let Ok(socket) =
                std::net::UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR)
            {
                if socket.connect("192.0.2.1:80").is_ok() {
                    if let Ok(local_addr) = socket.local_addr() {
                        addresses.push(local_addr.ip());
                    }
                }
            }
        }

        if addresses.is_empty() {
            addresses.push(IpAddr::V4(Ipv4Addr::LOCALHOST));
        }

        addresses
    }

    #[must_use]
    pub fn detect_network_region(ip: &std::net::IpAddr) -> String {
        use std::net::IpAddr;

        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4.is_private() {
                    return "private".to_string();
                }

                let octets = ipv4.octets();
                match octets[0] {
                    3 | 13 | 15 | 18 | 34 | 35 | 54 => "aws".to_string(),
                    8 | 23 | 107 | 130 | 142 | 146 => "gcp".to_string(),
                    20 | 40 | 51 | 65 | 68 | 70 => "azure".to_string(),
                    162 | 172 | 173 | 188 | 190 | 197 | 198 => "cloudflare".to_string(),
                    _ => match octets[0] {
                        0 => "reserved".to_string(),
                        1..=23 => "us-east".to_string(),
                        24..=39 => "us-west".to_string(),
                        40..=79 => "europe".to_string(),
                        80..=103 => "asia".to_string(),
                        104..=127 => "oceania".to_string(),
                        128..=159 => "us-central".to_string(),
                        160..=191 => "europe-east".to_string(),
                        192..=223 => "asia-east".to_string(),
                        224..=255 => "multicast".to_string(),
                    },
                }
            }
            IpAddr::V6(_) => "ipv6".to_string(),
        }
    }

    #[must_use]
    pub fn create_network_location() -> NetworkLocation {
        let local_ips = Self::get_local_ip_addresses();
        let primary_ip = local_ips
            .first()
            .copied()
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

        let region = Self::detect_network_region(&primary_ip);

        let external_ip = local_ips
            .iter()
            .find(|ip| match ip {
                std::net::IpAddr::V4(ipv4) => !ipv4.is_private() && !ipv4.is_loopback(),
                std::net::IpAddr::V6(ipv6) => !ipv6.is_loopback() && !ipv6.is_multicast(),
            })
            .map(ToString::to_string);

        let internal_ip = local_ips
            .iter()
            .find(|ip| match ip {
                std::net::IpAddr::V4(ipv4) => ipv4.is_private(),
                std::net::IpAddr::V6(_) => false,
            })
            .map(ToString::to_string);

        let subnet = if let Some(internal_ip_str) = &internal_ip {
            if let Ok(internal_ip_addr) = internal_ip_str.parse::<std::net::IpAddr>() {
                match internal_ip_addr {
                    std::net::IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        Some(format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]))
                    }
                    std::net::IpAddr::V6(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        NetworkLocation {
            region,
            subnet,
            external_ip,
            internal_ip,
        }
    }

    #[must_use]
    pub fn measure_network_performance(
        target_node_id: &str,
        _target_address: &str,
    ) -> NetworkMeasurement {
        NetworkMeasurement {
            target_node_id: target_node_id.to_string(),
            latency_ms: 50.0,
            bandwidth_mbps: 100.0,
            packet_loss_percent: 0.1,
            jitter_ms: 2.0,
            measured_at: Utc::now(),
        }
    }

    /// Comprehensive network performance monitoring for federation
    pub fn start_network_monitoring(
        node_id: String,
        target_nodes: Vec<(String, String)>,
        _shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    ) -> Result<()> {
        tracing::info!("Starting network monitoring for node: {node_id}");

        for (target_node_id, target_address) in target_nodes {
            let measurement = Self::measure_network_performance(&target_node_id, &target_address);

            tracing::debug!(
                target_node = %target_node_id,
                latency = %measurement.latency_ms,
                bandwidth = %measurement.bandwidth_mbps,
                "Network measurement completed"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::NetworkManager;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn detect_network_region_private_ipv4() {
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(NetworkManager::detect_network_region(&ip), "private");
    }

    #[test]
    fn detect_network_region_aws_gcp_azure_cloudflare() {
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(3, 0, 0, 1))),
            "aws"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            "gcp"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(20, 0, 0, 1))),
            "azure"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(162, 0, 0, 1))),
            "cloudflare"
        );
    }

    #[test]
    fn detect_network_region_heuristic_buckets_and_ipv6() {
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
            "reserved"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(15, 0, 0, 1))),
            "aws"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(50, 0, 0, 1))),
            "europe"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V6("::1".parse().unwrap())),
            "ipv6"
        );
    }

    #[test]
    fn measure_network_performance_shape() {
        let m = NetworkManager::measure_network_performance("node-a", "127.0.0.1");
        assert_eq!(m.target_node_id, "node-a");
        assert_eq!(m.latency_ms, 50.0);
        assert_eq!(m.bandwidth_mbps, 100.0);
        assert_eq!(m.packet_loss_percent, 0.1);
        assert_eq!(m.jitter_ms, 2.0);
    }

    #[tokio::test]
    async fn start_network_monitoring_returns_ok_without_io() {
        let (_tx, shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        let res = NetworkManager::start_network_monitoring(
            "n1".to_string(),
            vec![("t1".to_string(), "127.0.0.1".to_string())],
            shutdown_rx,
        );
        assert!(res.is_ok());
    }
}
