// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Network address detection for production environments
//!
//! Detects the public-facing IP address using multiple strategies:
//! 1. Cloud provider metadata environment variables (AWS, GCP, Azure)
//! 2. Linux `/proc/net/route` default interface detection
//! 3. Hostname-based DNS resolution
//! 4. Safe fallback to `0.0.0.0` (all interfaces)
//!
//! Zero external dependencies — uses only `std` and process-env overlay.

use std::net::{IpAddr, Ipv4Addr};

/// Detect public IP address for production environments.
///
/// # Strategy
/// 1. Check `SONGBIRD_PUBLIC_IP` environment variable
/// 2. Auto-detect from network interfaces
/// 3. Fall back to unspecified (let discovery resolve)
pub(crate) fn detect_public_ip() -> IpAddr {
    detect_public_ip_with(|k| songbird_process_env::var(k))
}

/// Injectable variant for concurrent testing without global env mutation.
pub(crate) fn detect_public_ip_with(
    env: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> IpAddr {
    if let Ok(ip_str) = env("SONGBIRD_PUBLIC_IP")
        && let Ok(ip) = ip_str.parse::<IpAddr>()
    {
        return ip;
    }

    detect_from_network_interfaces()
}

/// Detect IP from network interfaces using platform-specific strategies.
fn detect_from_network_interfaces() -> IpAddr {
    if let Some(ip) = check_cloud_metadata() {
        return ip;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(ip) = detect_linux_default_interface() {
            return ip;
        }
    }

    if let Some(ip) = detect_via_hostname() {
        return ip;
    }

    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

/// Check cloud provider metadata via environment variables.
///
/// Fast, zero-cost in non-cloud environments (no network calls).
fn check_cloud_metadata() -> Option<IpAddr> {
    check_cloud_metadata_with(|k| songbird_process_env::var(k))
}

/// Injectable variant for concurrent testing without global env mutation.
fn check_cloud_metadata_with(
    env: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Option<IpAddr> {
    for var in ["AWS_INSTANCE_IP", "GCE_INSTANCE_IP", "AZURE_VM_IP"] {
        if let Ok(ip_str) = env(var)
            && let Ok(ip) = ip_str.parse::<IpAddr>()
        {
            return Some(ip);
        }
    }
    None
}

/// Linux-specific: detect default route interface via `/proc/net/route`.
#[cfg(target_os = "linux")]
fn detect_linux_default_interface() -> Option<IpAddr> {
    let route_content = std::fs::read_to_string("/proc/net/route").ok()?;

    for line in route_content.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 2 && fields[1] == "00000000" {
            return Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        }
    }

    None
}

#[cfg(test)]
#[path = "network_detection_tests.rs"]
mod tests;

/// Detect IP via hostname DNS resolution.
fn detect_via_hostname() -> Option<IpAddr> {
    use std::net::ToSocketAddrs;

    let hostname = songbird_process_env::var("HOSTNAME")
        .or_else(|_| songbird_process_env::var("HOST"))
        .ok()?;

    let socket_addr_str = format!("{hostname}:0");

    socket_addr_str.to_socket_addrs().ok().and_then(|mut addrs| {
        addrs
            .find(|addr| {
                let ip = addr.ip();
                ip.is_ipv4() && !ip.is_loopback()
            })
            .map(|addr| addr.ip())
    })
}
