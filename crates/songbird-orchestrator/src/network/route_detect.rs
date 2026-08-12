// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Local route detection without hardcoded public resolver IPs.
//!
//! Delegates to [`songbird_types::network_info`] for `/proc`-based detection and
//! UDP route probes. Retains optional `netdev` enrichment for IPv6.

use std::net::{Ipv4Addr, Ipv6Addr};

/// Linux `ip route get` target — documentation IPv4 ([RFC 5737] TEST-NET-1).
#[cfg(target_os = "linux")]
pub const ROUTE_GET_TARGET_V4: &str = "192.0.2.1";

const DEFAULT_ROUTE_DETECT_V6: &str = "[2001:db8::1]:80";

/// `SONGBIRD_ROUTE_DETECT_ADDR` or documentation IPv4 `192.0.2.1:80`.
#[must_use]
pub fn route_detect_addr_v4() -> String {
    songbird_types::network_info::route_detect_addr_v4()
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
/// Order: netdev default interface → [`songbird_types::network_info::resolve_local_ipv4`].
pub fn resolve_local_ipv4() -> anyhow::Result<String> {
    if let Some(ip) = primary_ipv4_from_default_interface() {
        return Ok(ip.to_string());
    }

    songbird_types::network_info::resolve_local_ipv4()
        .map(|ip| ip.to_string())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn route_detect_addr_v4_uses_documentation_space_by_default() {
        songbird_process_env::remove_var("SONGBIRD_ROUTE_DETECT_ADDR");
        assert!(route_detect_addr_v4().contains("192.0.2.1"));
    }

    #[test]
    fn resolve_local_ipv4_returns_parseable_ip_or_err() {
        match resolve_local_ipv4() {
            Ok(ip) => {
                let parsed: std::net::IpAddr = ip.parse().expect("must parse as IP");
                assert!(!parsed.is_loopback());
            }
            Err(_) => {}
        }
    }
}
