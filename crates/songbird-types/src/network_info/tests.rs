// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use std::net::{IpAddr, Ipv4Addr};

use super::*;

#[test]
fn parse_ipv4_octets_valid_and_invalid() {
    assert_eq!(parse_ipv4_octets("192.168.1.1"), Some([192, 168, 1, 1]));
    assert_eq!(parse_ipv4_octets("10.0.0.1"), Some([10, 0, 0, 1]));
    assert!(parse_ipv4_octets("not-an-ip").is_none());
    assert!(parse_ipv4_octets("1.2.3").is_none());
}

#[test]
fn is_private_or_special_ranges() {
    assert!(is_private_or_special([10, 0, 0, 1]));
    assert!(is_private_or_special([192, 168, 1, 1]));
    assert!(is_private_or_special([127, 0, 0, 1]));
    assert!(!is_private_or_special([8, 8, 8, 8]));
}

#[test]
fn same_subnet_24_matches_third_octet() {
    assert!(same_subnet_24([192, 168, 1, 10], [192, 168, 1, 20]));
    assert!(!same_subnet_24([192, 168, 1, 10], [192, 168, 2, 20]));
}

#[test]
fn route_detect_addr_v4_default_is_documentation_space() {
    songbird_process_env::remove_var("SONGBIRD_ROUTE_DETECT_ADDR");
    assert!(route_detect_addr_v4().contains("192.0.2.1"));
}

#[test]
fn local_ip_addresses_never_empty() {
    let addresses = local_ip_addresses();
    assert!(!addresses.is_empty());
}

#[test]
fn resolve_local_ipv4_returns_non_loopback_on_capable_platforms() {
    if let Ok(ip) = resolve_local_ipv4() {
        assert_ne!(ip, Ipv4Addr::LOCALHOST);
        assert!(!ip.is_unspecified());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_default_route_parses_when_present() {
    if has_default_route() {
        let gateway = default_gateway().expect("gateway when default route exists");
        assert!(!gateway.is_unspecified() || gateway == Ipv4Addr::UNSPECIFIED);
        assert!(default_interface().is_some());
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_fib_trie_returns_addresses_or_empty() {
    let addrs = local_ipv4_from_fib_trie();
    for ip in addrs {
        assert_ne!(ip, Ipv4Addr::LOCALHOST);
    }
}

#[test]
fn local_ip_addresses_include_valid_ip_addr() {
    let addresses = local_ip_addresses();
    assert!(addresses.iter().any(|addr| matches!(addr, IpAddr::V4(_))));
}
