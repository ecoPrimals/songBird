// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Linux `/proc` network parsers.

use std::net::Ipv4Addr;

use super::{is_private_or_special, parse_ipv4_octets};

/// A default-route entry from `/proc/net/route`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry {
    /// Network interface name (e.g. `eth0`).
    pub interface: String,
    /// Gateway IPv4 address (little-endian hex in `/proc`).
    pub gateway: Ipv4Addr,
}

/// Parse the default route from `/proc/net/route`.
#[must_use]
pub fn default_route() -> Option<RouteEntry> {
    let contents = std::fs::read_to_string("/proc/net/route").ok()?;

    for line in contents.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 3 && fields[1] == "00000000" {
            let interface = fields[0].to_string();
            let gateway = parse_route_gateway_hex(fields[2])?;
            return Some(RouteEntry {
                interface,
                gateway,
            });
        }
    }

    None
}

/// Returns true when a default route (destination `00000000`) exists.
#[must_use]
pub fn has_default_route() -> bool {
    default_route().is_some()
}

/// Collect local IPv4 addresses from `/proc/net/fib_trie` LOCAL entries.
#[must_use]
pub fn local_ipv4_from_fib_trie() -> Vec<Ipv4Addr> {
    let Ok(content) = std::fs::read_to_string("/proc/net/fib_trie") else {
        return Vec::new();
    };

    let mut addrs = Vec::new();
    let mut prev_ip: Option<[u8; 4]> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(ip_str) = trimmed.strip_prefix("|-- ").or_else(|| trimmed.strip_prefix("+-- "))
            && let Some(octets) = parse_ipv4_octets(ip_str.trim())
        {
            prev_ip = Some(octets);
        }
        if trimmed.contains("/32 host LOCAL")
            && let Some(octets) = prev_ip
            && octets[0] != 127
            && octets != [0, 0, 0, 0]
        {
            addrs.push(Ipv4Addr::from(octets));
        }
    }

    addrs
}

/// Returns true when any interface has a non-private LOCAL IPv4 address.
#[must_use]
pub fn has_public_ipv4_interface() -> bool {
    let Ok(content) = std::fs::read_to_string("/proc/net/fib_trie") else {
        return has_default_route();
    };

    let mut prev_ip: Option<[u8; 4]> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(ip_str) = trimmed.strip_prefix("|-- ").or_else(|| trimmed.strip_prefix("+-- "))
            && let Some(octets) = parse_ipv4_octets(ip_str.trim())
        {
            prev_ip = Some(octets);
        }
        if trimmed.contains("/32 host LOCAL")
            && let Some(octets) = prev_ip
            && !is_private_or_special(octets)
        {
            return true;
        }
    }

    false
}

fn parse_route_gateway_hex(gw_hex: &str) -> Option<Ipv4Addr> {
    let gw_int = u32::from_str_radix(gw_hex, 16).ok()?;
    Some(Ipv4Addr::new(
        (gw_int & 0xFF) as u8,
        ((gw_int >> 8) & 0xFF) as u8,
        ((gw_int >> 16) & 0xFF) as u8,
        ((gw_int >> 24) & 0xFF) as u8,
    ))
}
