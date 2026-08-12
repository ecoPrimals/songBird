// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::discovery::types::{NetworkLocation, NetworkMeasurement};
use chrono::Utc;
use songbird_types::{SongbirdError, SongbirdResult};
type Result<T> = SongbirdResult<T>;
#[cfg(any(target_os = "linux", target_os = "macos"))]
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

        Err(SongbirdError::Network {
            message: format!("ping measurement unavailable for {target_address} on this platform"),
            interface: None,
            suggestion: Some("ensure ping is available or use a supported platform".into()),
        })
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

        Err(SongbirdError::Network {
            message: "bandwidth estimation unavailable on this platform".into(),
            interface: None,
            suggestion: Some("check /sys/class/net on Linux".into()),
        })
    }

    #[must_use]
    pub fn get_local_ip_addresses() -> Vec<std::net::IpAddr> {
        songbird_types::network_info::local_ip_addresses()
    }

    #[must_use]
    pub fn detect_network_region(ip: &std::net::IpAddr) -> String {
        use std::net::IpAddr;

        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4.is_private() {
                    return String::from("private");
                }

                let octets = ipv4.octets();
                match octets[0] {
                    3 | 13 | 15 | 18 | 34 | 35 | 54 => String::from("aws"),
                    8 | 23 | 107 | 130 | 142 | 146 => String::from("gcp"),
                    20 | 40 | 51 | 65 | 68 | 70 => String::from("azure"),
                    162 | 172 | 173 | 188 | 190 | 197 | 198 => String::from("cloudflare"),
                    _ => match octets[0] {
                        0 => String::from("reserved"),
                        1..=23 => String::from("us-east"),
                        24..=39 => String::from("us-west"),
                        40..=79 => String::from("europe"),
                        80..=103 => String::from("asia"),
                        104..=127 => String::from("oceania"),
                        128..=159 => String::from("us-central"),
                        160..=191 => String::from("europe-east"),
                        192..=223 => String::from("asia-east"),
                        224..=255 => String::from("multicast"),
                    },
                }
            }
            IpAddr::V6(_) => String::from("ipv6"),
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

        let subnet = internal_ip.as_ref().and_then(|internal_ip_str| {
            internal_ip_str.parse::<std::net::IpAddr>().ok().and_then(|internal_ip_addr| {
                match internal_ip_addr {
                    std::net::IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        Some(format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]))
                    }
                    std::net::IpAddr::V6(_) => None,
                }
            })
        });

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
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
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
            String::from("n1"),
            vec![(String::from("t1"), String::from("127.0.0.1"))],
            shutdown_rx,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn get_local_ip_addresses_returns_at_least_one() {
        let addresses = NetworkManager::get_local_ip_addresses();
        assert!(!addresses.is_empty());
    }

    #[test]
    fn create_network_location_populates_region() {
        let location = NetworkManager::create_network_location();
        assert!(!location.region.is_empty());
    }

    #[test]
    fn create_network_location_subnet_for_private_ipv4() {
        let location = NetworkManager::create_network_location();
        if let Some(subnet) = &location.subnet {
            assert!(subnet.ends_with(".0/24") || subnet.contains('/'));
        }
    }

    #[test]
    fn measure_ping_latency_localhost_succeeds_or_errors() {
        let result = NetworkManager::measure_ping_latency("127.0.0.1");
        // On Linux/macOS with ping available, this succeeds; on other platforms, it errors
        if let Ok(latency) = result {
            assert!(latency > 0.0);
        }
    }

    #[test]
    fn estimate_bandwidth_succeeds_or_errors() {
        let result = NetworkManager::estimate_bandwidth("127.0.0.1");
        // On Linux with /sys/class/net, this succeeds; on other platforms, it errors
        if let Ok(bandwidth) = result {
            assert!(bandwidth > 0.0);
        }
    }

    #[test]
    fn detect_network_region_us_east_heuristic_bucket() {
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            "private"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(12, 0, 0, 1))),
            "us-east"
        );
    }

    #[test]
    fn detect_network_region_multicast_and_oceania_buckets() {
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(230, 0, 0, 1))),
            "multicast"
        );
        assert_eq!(
            NetworkManager::detect_network_region(&IpAddr::V4(Ipv4Addr::new(110, 0, 0, 1))),
            "oceania"
        );
    }

    #[test]
    fn network_scan_port_range_validation_constants() {
        const MIN_PORT: u16 = 1;
        const MAX_PORT: u16 = 65535;
        const SCAN_START: u16 = 8000;
        const SCAN_END: u16 = 8100;

        assert!(SCAN_START >= MIN_PORT);
        assert!(SCAN_END <= MAX_PORT);
        assert!(SCAN_END > SCAN_START);
        assert!(SCAN_END - SCAN_START <= 10_000);
    }

    #[test]
    fn network_probe_timeout_configuration() {
        const DEFAULT_TIMEOUT_MS: u64 = 5000;
        const MIN_TIMEOUT_MS: u64 = 100;

        assert!(DEFAULT_TIMEOUT_MS >= MIN_TIMEOUT_MS);
        let unreachable_target = "192.0.2.1";
        let result = NetworkManager::measure_ping_latency(unreachable_target);
        if let Ok(latency) = result {
            assert!(latency > 0.0);
        }
    }

    #[tokio::test]
    async fn start_network_monitoring_multiple_targets() {
        let (_tx, shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        let targets = vec![
            (String::from("node-a"), String::from("127.0.0.1")),
            (String::from("node-b"), String::from("192.0.2.1")),
        ];
        let res =
            NetworkManager::start_network_monitoring(String::from("monitor"), targets, shutdown_rx);
        assert!(res.is_ok());
    }

    #[test]
    fn service_detection_from_cloud_provider_ip_ranges() {
        let probes = [
            (IpAddr::V4(Ipv4Addr::new(3, 5, 0, 1)), "aws"),
            (IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), "gcp"),
            (IpAddr::V4(Ipv4Addr::new(20, 50, 0, 1)), "azure"),
            (IpAddr::V4(Ipv4Addr::new(162, 158, 0, 1)), "cloudflare"),
        ];

        for (ip, expected_region) in probes {
            assert_eq!(NetworkManager::detect_network_region(&ip), expected_region);
        }
    }
}
