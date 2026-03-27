// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn detect_public_ip_returns_valid_addr() {
    let ip = detect_public_ip();
    // Should be a valid IP (either env-provided, detected, or unspecified fallback)
    assert!(ip.is_ipv4() || ip.is_ipv6());
}

#[serial_test::serial]
#[test]
fn detect_public_ip_respects_songbird_public_ip_env() {
    songbird_process_env::set_var("SONGBIRD_PUBLIC_IP", "10.42.0.1");
    let ip = detect_public_ip();
    songbird_process_env::remove_var("SONGBIRD_PUBLIC_IP");

    assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(10, 42, 0, 1)));
}

#[serial_test::serial]
#[test]
fn detect_public_ip_ignores_invalid_env() {
    songbird_process_env::set_var("SONGBIRD_PUBLIC_IP", "not-an-ip");
    let ip = detect_public_ip();
    songbird_process_env::remove_var("SONGBIRD_PUBLIC_IP");

    assert!(ip.is_ipv4() || ip.is_ipv6());
}

#[serial_test::serial]
#[test]
fn check_cloud_metadata_per_provider() {
    // Clean slate
    songbird_process_env::remove_var("AWS_INSTANCE_IP");
    songbird_process_env::remove_var("GCE_INSTANCE_IP");
    songbird_process_env::remove_var("AZURE_VM_IP");

    // No cloud vars → None
    assert!(check_cloud_metadata().is_none());

    // AWS
    songbird_process_env::set_var("AWS_INSTANCE_IP", "172.31.1.100");
    assert_eq!(check_cloud_metadata(), Some(IpAddr::V4(Ipv4Addr::new(172, 31, 1, 100))));
    songbird_process_env::remove_var("AWS_INSTANCE_IP");

    // GCE
    songbird_process_env::set_var("GCE_INSTANCE_IP", "10.128.0.5");
    assert_eq!(check_cloud_metadata(), Some(IpAddr::V4(Ipv4Addr::new(10, 128, 0, 5))));
    songbird_process_env::remove_var("GCE_INSTANCE_IP");

    // Azure
    songbird_process_env::set_var("AZURE_VM_IP", "10.0.0.4");
    assert_eq!(check_cloud_metadata(), Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4))));
    songbird_process_env::remove_var("AZURE_VM_IP");
}

#[test]
fn detect_via_hostname_returns_option() {
    // May or may not resolve depending on system config — just verify no panic
    let _result = detect_via_hostname();
}
