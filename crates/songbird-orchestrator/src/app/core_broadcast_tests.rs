// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for `SongbirdOrchestrator::discover_broadcast_addresses`
//!
//! Extracted from `core.rs` for file-size discipline (<1000 lines).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::net::SocketAddr;

use songbird_process_env;

use super::SongbirdOrchestrator;
use super::broadcast_test_lock;

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    broadcast_test_lock::guard()
}

/// Isolate from a developer shell that exports discovery-related ports.
fn clear_discovery_port_env() {
    songbird_process_env::remove_var("SONGBIRD_DISCOVERY_PORT");
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_DISCOVERY_PORT");
}

fn parse(s: &str) -> SocketAddr {
    s.parse().expect("valid socket in test")
}

#[test]
fn discover_broadcast_prefers_env_when_set() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var(
        "SONGBIRD_BROADCAST_ADDRESSES",
        "224.0.0.10:2300,224.0.0.11:2301",
    );
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&["10.0.0.1:9999".to_string()]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert_eq!(addrs.len(), 2);
    assert_eq!(addrs[0], parse("224.0.0.10:2300"));
    assert_eq!(addrs[1], parse("224.0.0.11:2301"));
}

#[test]
fn discover_broadcast_env_invalid_tokens_fall_through_to_config() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "not-a-socket,, , also-bad");
    let configured = vec!["192.168.55.1:2300".to_string()];
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&configured);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert!(
        addrs.iter().any(|a| *a == parse("192.168.55.1:2300")),
        "expected configured address present: {addrs:?}"
    );
}

#[test]
fn discover_broadcast_merges_config_with_subnet_fallbacks() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&["10.0.0.5:2300".to_string()]);
    assert!(addrs.iter().any(|a| *a == parse("10.0.0.5:2300")));
    assert!(addrs.iter().any(|a| *a == parse("192.168.1.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("192.168.0.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("10.0.0.255:2300")));
}

#[test]
fn discover_broadcast_skips_duplicate_fallback_ip() {
    let _g = lock_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs =
        SongbirdOrchestrator::discover_broadcast_addresses(&["192.168.1.255:2300".to_string()]);
    let count_192_168_1 = addrs.iter().filter(|a| a.ip().to_string() == "192.168.1.255").count();
    assert_eq!(count_192_168_1, 1, "duplicate subnet IP: {addrs:?}");
}

#[test]
fn discover_broadcast_empty_config_uses_standard_fallback_list() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    assert!(addrs.len() >= 3);
    assert!(addrs.iter().any(|a| *a == parse("192.168.1.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("192.168.0.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("10.0.0.255:2300")));
}

#[test]
fn discover_broadcast_subnet_fallbacks_use_broadcast_discovery_port_env() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    songbird_process_env::set_var("SONGBIRD_BROADCAST_DISCOVERY_PORT", "2400");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_DISCOVERY_PORT");
    assert!(addrs.iter().any(|a| *a == parse("192.168.1.255:2400")));
    assert!(addrs.iter().any(|a| *a == parse("192.168.0.255:2400")));
    assert!(addrs.iter().any(|a| *a == parse("10.0.0.255:2400")));
}

#[test]
fn discover_broadcast_env_whitespace_trimmed() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var(
        "SONGBIRD_BROADCAST_ADDRESSES",
        " 239.255.0.1:4242 , 239.255.0.2:4243 ",
    );
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert_eq!(addrs.len(), 2);
    assert_eq!(addrs[0], parse("239.255.0.1:4242"));
    assert_eq!(addrs[1], parse("239.255.0.2:4243"));
}

#[test]
fn discover_broadcast_env_empty_string_ignored() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert!(addrs.len() >= 3);
}

#[test]
fn discover_broadcast_env_first_token_invalid_second_valid() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "not-a-socket,203.0.113.5:2300");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0], parse("203.0.113.5:2300"));
}

#[test]
fn discover_broadcast_config_filters_invalid_entries() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[
        "not-valid".to_string(),
        "172.16.0.255:2300".to_string(),
        String::new(),
    ]);
    assert!(addrs.iter().any(|a| *a == parse("172.16.0.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("192.168.1.255:2300")), "fallback merge: {addrs:?}");
}

#[test]
fn discover_broadcast_config_only_invalid_gets_fallbacks() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[
        ":::not-a-port".to_string(),
        "99999.0.0.1:1".to_string(),
    ]);
    assert!(addrs.iter().any(|a| *a == parse("192.168.1.255:2300")));
    assert!(addrs.iter().any(|a| *a == parse("10.0.0.255:2300")));
}

#[test]
fn discover_broadcast_preserves_config_order_before_fallbacks() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[
        "198.51.100.1:4000".to_string(),
        "198.51.100.2:4001".to_string(),
    ]);
    assert_eq!(addrs[0], parse("198.51.100.1:4000"));
    assert_eq!(addrs[1], parse("198.51.100.2:4001"));
}

#[test]
fn discover_broadcast_duplicate_192_168_0_skipped_once() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs =
        SongbirdOrchestrator::discover_broadcast_addresses(&["192.168.0.255:2300".to_string()]);
    let n = addrs.iter().filter(|a| a.ip().to_string() == "192.168.0.255").count();
    assert_eq!(n, 1, "{addrs:?}");
}

#[test]
fn discover_broadcast_duplicate_10_subnet_skipped_once() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs =
        SongbirdOrchestrator::discover_broadcast_addresses(&["10.0.0.255:2300".to_string()]);
    let n = addrs.iter().filter(|a| a.ip().to_string() == "10.0.0.255").count();
    assert_eq!(n, 1, "{addrs:?}");
}

#[test]
fn discover_broadcast_env_single_trailing_comma() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "239.1.1.1:1111,");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&[]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0], parse("239.1.1.1:1111"));
}

#[test]
fn discover_broadcast_env_all_invalid_falls_through_to_config() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::set_var("SONGBIRD_BROADCAST_ADDRESSES", "bad, worse");
    let addrs = SongbirdOrchestrator::discover_broadcast_addresses(&["10.5.5.5:7777".to_string()]);
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    assert!(addrs.iter().any(|a| *a == parse("10.5.5.5:7777")));
}

#[test]
fn discover_broadcast_merges_unique_fallback_ips_only() {
    let _g = lock_env();
    clear_discovery_port_env();
    songbird_process_env::remove_var("SONGBIRD_BROADCAST_ADDRESSES");
    let addrs =
        SongbirdOrchestrator::discover_broadcast_addresses(&["172.20.0.1:2300".to_string()]);
    let ips: Vec<_> = addrs.iter().map(|a| a.ip().to_string()).collect();
    assert!(ips.contains(&"172.20.0.1".to_string()));
    assert!(ips.contains(&"192.168.1.255".to_string()));
    assert!(ips.contains(&"192.168.0.255".to_string()));
    assert!(ips.contains(&"10.0.0.255".to_string()));
}
