// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network route and local IP detection.
//!
//! Linux reads from `/proc/net/route` and `/proc/net/fib_trie`. Non-Linux
//! platforms use a UDP route-probe fallback.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod stub;

use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use crate::{SongbirdError, SongbirdResult};

#[cfg(target_os = "linux")]
pub use linux::{
    RouteEntry, default_route, has_default_route, has_public_ipv4_interface,
    local_ipv4_from_fib_trie,
};
#[cfg(not(target_os = "linux"))]
pub use stub::{
    RouteEntry, default_route, has_default_route, has_public_ipv4_interface,
    local_ipv4_from_fib_trie,
};

const DEFAULT_ROUTE_DETECT_V4: &str = "192.0.2.1:80";

/// IPv4 route-probe target from `SONGBIRD_ROUTE_DETECT_ADDR` or RFC 5737 documentation space.
#[must_use]
pub fn route_detect_addr_v4() -> String {
    songbird_process_env::var("SONGBIRD_ROUTE_DETECT_ADDR")
        .unwrap_or_else(|_| DEFAULT_ROUTE_DETECT_V4.to_string())
}

/// Default gateway from the routing table (Linux `/proc/net/route`).
#[must_use]
pub fn default_gateway() -> Option<Ipv4Addr> {
    default_route().map(|entry| entry.gateway)
}

/// Default route interface name (Linux `/proc/net/route`).
#[must_use]
pub fn default_interface() -> Option<String> {
    default_route().map(|entry| entry.interface)
}

/// Resolve a non-loopback local IPv4 using default-interface addresses or a UDP route probe.
///
/// # Errors
///
/// Returns [`SongbirdError::Network`] when no suitable address can be determined.
pub fn resolve_local_ipv4() -> SongbirdResult<Ipv4Addr> {
    #[cfg(target_os = "linux")]
    {
        for ip in local_ipv4_from_fib_trie() {
            if ip != Ipv4Addr::LOCALHOST && !ip.is_unspecified() && !ip.is_link_local() {
                return Ok(ip);
            }
        }
    }

    let socket = UdpSocket::bind(crate::constants::EPHEMERAL_BIND_ADDR).map_err(|e| {
        SongbirdError::Network {
            message: format!("Failed to bind route-probe socket: {e}"),
            interface: None,
            suggestion: Some("Check network permissions".into()),
        }
    })?;
    socket.connect(route_detect_addr_v4().as_str()).map_err(|e| SongbirdError::Network {
        message: format!("Failed UDP route probe: {e}"),
        interface: None,
        suggestion: None,
    })?;

    if let Ok(local_addr) = socket.local_addr()
        && let IpAddr::V4(ipv4) = local_addr.ip()
        && ipv4 != Ipv4Addr::LOCALHOST
        && !ipv4.is_unspecified()
    {
        return Ok(ipv4);
    }

    Err(SongbirdError::Network {
        message: String::from("Could not determine local IPv4"),
        interface: None,
        suggestion: Some("Set SONGBIRD_ROUTE_DETECT_ADDR or SONGBIRD_PUBLIC_IP".into()),
    })
}

/// Collect local IP addresses, falling back to route-probe and then localhost.
#[must_use]
pub fn local_ip_addresses() -> Vec<IpAddr> {
    let mut addresses = Vec::new();

    #[cfg(target_os = "linux")]
    {
        for ipv4 in local_ipv4_from_fib_trie() {
            addresses.push(IpAddr::V4(ipv4));
        }
    }

    if addresses.is_empty()
        && let Ok(ipv4) = resolve_local_ipv4()
    {
        addresses.push(IpAddr::V4(ipv4));
    }

    if addresses.is_empty() {
        addresses.push(IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    addresses
}

/// Parse dotted-quad IPv4 into octets.
#[must_use]
pub fn parse_ipv4_octets(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    Some([
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
        parts[3].parse().ok()?,
    ])
}

/// Returns true for RFC1918, link-local, loopback, and unspecified addresses.
#[must_use]
pub fn is_private_or_special(octets: [u8; 4]) -> bool {
    matches!(octets, [10 | 127 | 0, ..] | [172, 16..=31, ..] | [192, 168, ..] | [169, 254, ..])
}

/// Check if two IPv4 addresses share the same /24 subnet.
#[must_use]
pub fn same_subnet_24(a: [u8; 4], b: [u8; 4]) -> bool {
    a[0] == b[0] && a[1] == b[1] && a[2] == b[2]
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
