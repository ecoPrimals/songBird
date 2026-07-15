// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Local route detection without hardcoded public resolver IPs.
//!
//! Uses [`netdev::get_default_interface`] first, then a UDP “route probe” to
//! [`SONGBIRD_ROUTE_DETECT_ADDR`] (default [RFC 5737] `192.0.2.1:80`). IPv6 probes use
//! [RFC 3849] documentation space unless overridden via `SONGBIRD_ROUTE_DETECT_ADDR_V6`.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};

/// Linux `ip route get` target — documentation IPv4 ([RFC 5737] TEST-NET-1).
#[cfg(target_os = "linux")]
pub const ROUTE_GET_TARGET_V4: &str = "192.0.2.1";

const DEFAULT_ROUTE_DETECT_V4: &str = "192.0.2.1:80";
const DEFAULT_ROUTE_DETECT_V6: &str = "[2001:db8::1]:80";

/// `SONGBIRD_ROUTE_DETECT_ADDR` or documentation IPv4 `192.0.2.1:80`.
#[must_use]
pub fn route_detect_addr_v4() -> String {
    songbird_process_env::var("SONGBIRD_ROUTE_DETECT_ADDR")
        .unwrap_or_else(|_| DEFAULT_ROUTE_DETECT_V4.to_string())
}

/// Optional override for IPv6 UDP route probe (default `[2001:db8::1]:80`).
#[must_use]
pub fn route_detect_addr_v6() -> String {
    songbird_process_env::var("SONGBIRD_ROUTE_DETECT_ADDR_V6")
        .unwrap_or_else(|_| DEFAULT_ROUTE_DETECT_V6.to_string())
}

#[must_use]
pub fn primary_ipv4_from_default_interface() -> Option<Ipv4Addr> {
    let default_iface = netdev::get_default_interface().ok()?;
    for addr in &default_iface.ipv4 {
        let ip = addr.addr();
        if ip != Ipv4Addr::LOCALHOST && !ip.is_link_local() {
            return Some(ip);
        }
    }
    None
}

#[must_use]
pub fn primary_ipv6_from_default_interface() -> Option<Ipv6Addr> {
    let default_iface = netdev::get_default_interface().ok()?;
    for addr in &default_iface.ipv6 {
        let ip = addr.addr();
        if ip.is_loopback() || ip.is_unicast_link_local() {
            continue;
        }
        return Some(ip);
    }
    None
}

/// Resolve a non-loopback local IPv4 for connectivity / binding hints.
///
/// Order: default interface addresses → UDP route probe to [`route_detect_addr_v4`].
pub fn resolve_local_ipv4() -> anyhow::Result<String> {
    if let Some(ip) = primary_ipv4_from_default_interface() {
        return Ok(ip.to_string());
    }

    let socket = UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR)?;
    socket.connect(route_detect_addr_v4().as_str())?;

    if let Ok(local_addr) = socket.local_addr() {
        let ip = local_addr.ip();
        if let IpAddr::V4(ipv4) = ip
            && ipv4 != Ipv4Addr::LOCALHOST
            && !ipv4.is_unspecified()
        {
            return Ok(ip.to_string());
        }
    }

    anyhow::bail!("Could not determine local IPv4")
}
